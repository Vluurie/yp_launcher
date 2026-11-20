use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use gpui::{Context, EventEmitter, Task};
use serde_json::Value as JsonValue;
use yp_config::ConfigError;
use yp_config::overlay::Settings;
use yp_config::state::overlay_store::OverlayStore as Logic;
use yp_core::GamePaths;
use yp_core::watch::Recursive;

pub use yp_config::state::overlay_store::OverlayData;

use crate::cell::Guarded;
use crate::env::AppEnv;
use crate::watch::{RELOAD_DEBOUNCE, Watch, touches};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamsSettingsOp {
    Load,
    Save,
    Discard,
    Reload,
}

#[derive(Debug, Clone)]
pub struct NamsSettingsFailure {
    pub op: NamsSettingsOp,
    pub error: Arc<ConfigError>,
}

#[derive(Debug, Clone)]
pub enum NamsSettingsEvent {
    Loaded,
    Saved { written: bool },
    Reloaded,
    Failed(NamsSettingsFailure),
}

pub struct NamsSettingsStore {
    watch_files: bool,
    game: Option<GamePaths>,
    cell: Guarded<Logic>,
    data: OverlayData,
    loading: bool,
    failure: Option<NamsSettingsFailure>,
    load_task: Option<Task<()>>,
    reload_task: Option<Task<()>>,
    sync_task: Option<Task<()>>,
    watch: Option<Watch>,
    watched_file: Option<PathBuf>,
    rearmed: bool,
}

impl EventEmitter<NamsSettingsEvent> for NamsSettingsStore {}

impl NamsSettingsStore {
    pub fn new(env: &AppEnv, cx: &mut Context<Self>) -> Self {
        Self {
            watch_files: env.watch_files,
            game: None,
            cell: Guarded::new(Logic::new(None), cx.background_executor()),
            data: OverlayData::default(),
            loading: false,
            failure: None,
            load_task: None,
            reload_task: None,
            sync_task: None,
            watch: None,
            watched_file: None,
            rearmed: false,
        }
    }

    pub fn data(&self) -> &OverlayData {
        &self.data
    }

    pub fn settings(&self) -> &Settings {
        &self.data.settings
    }

    pub fn path(&self) -> Option<&Path> {
        self.data.settings_path.as_deref()
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn is_dirty(&self) -> bool {
        self.data.has_unsaved_changes
    }

    pub fn failure(&self) -> Option<&NamsSettingsFailure> {
        self.failure.as_ref()
    }

    pub fn impeller(&self) -> bool {
        self.data.impeller()
    }

    pub fn game_keybinds_global(&self) -> bool {
        self.data.game_keybinds_global()
    }

    pub fn loading_speedup(&self) -> bool {
        self.data.loading_speedup_enabled()
    }

    pub fn shaders(&self) -> bool {
        self.data.shaders_enabled()
    }

    pub fn sound(&self) -> bool {
        self.data.sound_enabled()
    }

    pub fn editor_mcp(&self) -> bool {
        self.data.editor_mcp_enabled()
    }

    pub fn keybinds(&self) -> Settings {
        self.data.keybinds()
    }

    pub fn randomizer_config(&self) -> Settings {
        self.data.randomizer_config()
    }

    pub fn cheats_config(&self) -> Settings {
        self.data.cheats_config()
    }

    pub fn default_outfits(&self) -> Settings {
        self.data.default_outfits()
    }

    pub fn default_outfit_for(&self, character_type: &str) -> Option<String> {
        self.data.default_outfit_for(character_type)
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        if self.game != game {
            self.game = game.clone();
            self.watch = None;
            self.watched_file = None;
            self.rearmed = false;
            self.reload_task = None;
            self.cell.edit(move |logic| logic.set_game(game), |_| ());
        }
        self.load(cx);
    }

    fn sync(&mut self, cx: &mut Context<Self>) {
        match self.cell.peek(|logic| logic.data().clone()) {
            Some(data) => self.data = data,
            None => self.sync_when_settled(cx),
        }
        cx.notify();
    }

    fn sync_when_settled(&mut self, cx: &mut Context<Self>) {
        let settled = self.cell.settled();
        self.sync_task = Some(cx.spawn(async move |this, cx| {
            if settled.await.is_ok() {
                this.update(cx, |this, cx| this.sync(cx)).ok();
            }
        }));
    }

    fn fail(&mut self, op: NamsSettingsOp, error: ConfigError, cx: &mut Context<Self>) {
        log::warn!("overlay settings {op:?} failed: {error}");
        let failure = NamsSettingsFailure {
            op,
            error: Arc::new(error),
        };
        self.failure = Some(failure.clone());
        cx.emit(NamsSettingsEvent::Failed(failure));
        cx.notify();
    }

    fn run<R: Send + 'static>(
        &mut self,
        cx: &mut Context<Self>,
        work: impl FnOnce(&mut Logic) -> R + Send + 'static,
        done: impl FnOnce(&mut Self, R, &mut Context<Self>) + 'static,
    ) -> Task<()> {
        let result = self.cell.spawn(move |logic| {
            let result = work(logic);
            (result, logic.data().clone())
        });
        cx.spawn(async move |this, cx| {
            let Ok((result, data)) = result.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.data = data;
                this.sync(cx);
                done(this, result, cx);
            })
            .ok();
        })
    }

    fn edit(&mut self, cx: &mut Context<Self>, op: impl FnOnce(&mut Logic) + Send + 'static) {
        match self.cell.edit(op, |logic| logic.data().clone()) {
            Some(data) => self.data = data,
            None => self.sync_when_settled(cx),
        }
        cx.notify();
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        self.failure = None;
        cx.notify();
        let task = self.run(cx, Logic::load, |this, result, cx| {
            this.loading = false;
            match result {
                Ok(()) => {
                    cx.emit(NamsSettingsEvent::Loaded);
                    this.start_watching(cx);
                }
                Err(error) => this.fail(NamsSettingsOp::Load, error, cx),
            }
            cx.notify();
        });
        self.load_task = Some(task);
    }

    pub fn save(&mut self, cx: &mut Context<Self>) {
        self.run(cx, Logic::save, |this, result, cx| match result {
            Ok(written) => {
                cx.emit(NamsSettingsEvent::Saved { written });
                if written && !this.rearmed {
                    this.rearmed = true;
                    this.watch = None;
                    this.watched_file = None;
                    this.start_watching(cx);
                }
            }
            Err(error) => this.fail(NamsSettingsOp::Save, error, cx),
        })
        .detach();
    }

    pub fn discard(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        cx.notify();
        let task = self.run(cx, Logic::discard_changes, |this, result, cx| {
            this.loading = false;
            match result {
                Ok(()) => cx.emit(NamsSettingsEvent::Loaded),
                Err(error) => this.fail(NamsSettingsOp::Discard, error, cx),
            }
            cx.notify();
        });
        self.load_task = Some(task);
    }

    pub fn reload_if_clean(&mut self, cx: &mut Context<Self>) {
        if self.is_dirty() {
            return;
        }
        let task = self.run(
            cx,
            Logic::reload_if_clean,
            |this, result, cx| match result {
                Ok(true) => cx.emit(NamsSettingsEvent::Reloaded),
                Ok(false) => {}
                Err(error) => this.fail(NamsSettingsOp::Reload, error, cx),
            },
        );
        self.reload_task = Some(task);
    }

    pub fn update_keybind(
        &mut self,
        category: &str,
        action: &str,
        key: &str,
        cx: &mut Context<Self>,
    ) {
        let (category, action, key) = (category.to_owned(), action.to_owned(), key.to_owned());
        self.edit(cx, move |logic| {
            logic.update_keybind(&category, &action, &key)
        });
    }

    pub fn update_toggle(&mut self, key: &str, value: bool, cx: &mut Context<Self>) {
        let key = key.to_owned();
        self.edit(cx, move |logic| logic.update_toggle(&key, value));
    }

    pub fn set_default_outfit(
        &mut self,
        character_type: &str,
        folder_name: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let character_type = character_type.to_owned();
        let folder_name = folder_name.map(str::to_owned);
        self.edit(cx, move |logic| {
            logic.set_default_outfit(&character_type, folder_name.as_deref())
        });
    }

    pub fn update_cheat(&mut self, key: &str, value: JsonValue, cx: &mut Context<Self>) {
        let key = key.to_owned();
        self.edit(cx, move |logic| logic.update_cheat(&key, value));
    }

    pub fn update_randomizer(&mut self, key: &str, value: JsonValue, cx: &mut Context<Self>) {
        let key = key.to_owned();
        self.edit(cx, move |logic| logic.update_randomizer(&key, value));
    }

    fn start_watching(&mut self, cx: &mut Context<Self>) {
        if !self.watch_files {
            return;
        }
        let Some(file) = self.data.settings_path.clone() else {
            return;
        };
        if self.watch.is_some() && self.watched_file.as_ref() == Some(&file) {
            return;
        }
        let Some(dir) = file.parent().map(Path::to_path_buf) else {
            return;
        };
        self.watched_file = Some(file);
        self.watch = Some(Watch::start(
            vec![(dir, Recursive::NO)],
            cx,
            |this, paths, cx| this.on_external_change(&paths, cx),
        ));
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let Some(file) = self.data.settings_path.as_ref() else {
            return;
        };
        if !touches(paths, file) {
            return;
        }
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DEBOUNCE).await;
            let reload = this.update(cx, |this, cx| {
                this.run(
                    cx,
                    |logic| logic.reload_external(Instant::now()),
                    |this, result, cx| match result {
                        Ok(true) => cx.emit(NamsSettingsEvent::Reloaded),
                        Ok(false) => {}
                        Err(error) => this.fail(NamsSettingsOp::Reload, error, cx),
                    },
                )
            });
            if let Ok(reload) = reload {
                reload.await;
            }
        }));
    }
}
