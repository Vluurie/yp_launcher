use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use gpui::AppContext as _;
use gpui::{Context, EventEmitter, Task};
use yp_config::state::config_store::ConfigStore as Logic;
use yp_config::{ConfigError, ConfigFile};
use yp_core::toml_io::TomlError;
use yp_core::watch::Recursive;
use yp_core::{CancelToken, GamePaths, HostEnv};

pub use yp_config::state::config_store::{
    AppliedChange, ConfigData, CutsceneFlags, NAIOM_SECTION, SaveReport,
};

use crate::cell::{CancelOnDrop, Guarded};
use crate::env::AppEnv;
use crate::watch::{RELOAD_DEBOUNCE, Watch};

pub const NAIOM_AUTOSAVE_DELAY: Duration = Duration::from_millis(300);
pub const LODMOD_AUTOSAVE_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigOp {
    Load,
    Save,
    Discard,
    Autosave(ConfigFile),
    WriteNow,
    LodModPreset,
    LodModReset,
    CutsceneFlags,
    ExternalReload,
}

#[derive(Debug, Clone)]
pub struct ConfigFailure {
    pub op: ConfigOp,
    pub error: Arc<ConfigError>,
}

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Loaded,
    Saved(SaveReport),
    Written(ConfigFile),
    ReloadedExternally,
    CutsceneFlagsApplied(Vec<AppliedChange>),
    Failed(ConfigFailure),
}

pub struct ConfigStore {
    host: HostEnv,
    watch_files: bool,
    game: Option<GamePaths>,
    cell: Option<Guarded<Logic>>,
    data: ConfigData,
    loading: bool,
    loaded: bool,
    failure: Option<ConfigFailure>,
    load_task: Option<Task<()>>,
    naiom_task: Option<Task<()>>,
    lodmod_task: Option<Task<()>>,
    reload_task: Option<Task<()>>,
    cutscene_task: Option<Task<()>>,
    sync_task: Option<Task<()>>,
    watch: Option<Watch>,
}

impl EventEmitter<ConfigEvent> for ConfigStore {}

impl ConfigStore {
    pub fn new(env: &AppEnv, cx: &mut Context<Self>) -> Self {
        cx.on_app_quit(|this, _| this.flush_on_quit()).detach();
        Self {
            host: env.host.clone(),
            watch_files: env.watch_files,
            game: None,
            cell: None,
            data: ConfigData::default(),
            loading: false,
            loaded: false,
            failure: None,
            load_task: None,
            naiom_task: None,
            lodmod_task: None,
            reload_task: None,
            cutscene_task: None,
            sync_task: None,
            watch: None,
        }
    }

    pub fn game(&self) -> Option<&GamePaths> {
        self.game.as_ref()
    }

    pub fn data(&self) -> &ConfigData {
        &self.data
    }

    pub fn nams(&self) -> &toml::Table {
        &self.data.nams_values
    }

    pub fn lodmod(&self) -> &toml::Table {
        &self.data.lodmod_values
    }

    pub fn texture_injection(&self) -> &toml::Table {
        &self.data.texture_injection_values
    }

    pub fn naiom(&self) -> toml::Table {
        self.data.naiom_values()
    }

    pub fn raw(&self, file: ConfigFile) -> &str {
        match file {
            ConfigFile::Nams => &self.data.nams_raw,
            ConfigFile::Naiom => &self.data.naiom_raw,
            ConfigFile::LodMod => &self.data.lodmod_raw,
            ConfigFile::TextureInjection => &self.data.texture_injection_raw,
        }
    }

    pub fn error(&self, file: ConfigFile) -> Option<&TomlError> {
        self.data.error(file)
    }

    pub fn path(&self, file: ConfigFile) -> Option<PathBuf> {
        self.game.as_ref().map(|game| file.path(game))
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn is_dirty(&self) -> bool {
        self.data.has_unsaved_changes
    }

    pub fn failure(&self) -> Option<&ConfigFailure> {
        self.failure.as_ref()
    }

    pub fn has_pending_autosave(&self) -> bool {
        self.naiom_task.is_some() || self.lodmod_task.is_some()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        if self.game == game && self.cell.is_some() == game.is_some() {
            self.load(cx);
            return;
        }
        self.flush_pending(cx).detach();
        self.load_task = None;
        self.reload_task = None;
        self.cutscene_task = None;
        self.sync_task = None;
        self.watch = None;
        self.data = ConfigData::default();
        self.loading = false;
        self.loaded = false;
        self.failure = None;
        self.cell = game.as_ref().map(|game| {
            Guarded::new(
                Logic::new(game.clone(), self.host.clone()),
                cx.background_executor(),
            )
        });
        self.game = game;
        cx.notify();
        self.load(cx);
    }

    fn sync(&mut self, cx: &mut Context<Self>) {
        let Some(cell) = &self.cell else {
            return;
        };
        match cell.peek(|logic| logic.data().clone()) {
            Some(data) => self.data = data,
            None => self.sync_when_settled(cx),
        }
        cx.notify();
    }

    fn sync_when_settled(&mut self, cx: &mut Context<Self>) {
        let Some(cell) = &self.cell else {
            return;
        };
        let settled = cell.settled();
        self.sync_task = Some(cx.spawn(async move |this, cx| {
            if settled.await.is_ok() {
                this.update(cx, |this, cx| this.sync(cx)).ok();
            }
        }));
    }

    fn fail(&mut self, op: ConfigOp, error: ConfigError, cx: &mut Context<Self>) {
        log::warn!("config {op:?} failed: {error}");
        let failure = ConfigFailure {
            op,
            error: Arc::new(error),
        };
        self.failure = Some(failure.clone());
        cx.emit(ConfigEvent::Failed(failure));
        cx.notify();
    }

    fn run<R: Send + 'static>(
        &mut self,
        cx: &mut Context<Self>,
        work: impl FnOnce(&mut Logic) -> R + Send + 'static,
        done: impl FnOnce(&mut Self, R, &mut Context<Self>) + 'static,
    ) -> Task<()> {
        let Some(cell) = self.cell.clone() else {
            return Task::ready(());
        };
        let result = cell.spawn(move |logic| {
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
        let Some(cell) = &self.cell else {
            return;
        };
        match cell.edit(op, |logic| logic.data().clone()) {
            Some(data) => self.data = data,
            None => self.sync_when_settled(cx),
        }
        cx.notify();
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        if self.cell.is_none() {
            return;
        }
        self.loading = true;
        self.failure = None;
        self.naiom_task = None;
        cx.notify();
        let task = self.run(cx, Logic::load, |this, result, cx| {
            this.loading = false;
            match result {
                Ok(()) => {
                    this.loaded = true;
                    cx.emit(ConfigEvent::Loaded);
                    this.start_watching(cx);
                    this.auto_detect_cutscenes(cx);
                }
                Err(error) => this.fail(ConfigOp::Load, error, cx),
            }
            cx.notify();
        });
        self.load_task = Some(task);
    }

    pub fn save(&mut self, cx: &mut Context<Self>) {
        self.naiom_task = None;
        self.lodmod_task = None;
        self.run(cx, Logic::save, |this, result, cx| match result {
            Ok(report) => cx.emit(ConfigEvent::Saved(report)),
            Err(error) => this.fail(ConfigOp::Save, error, cx),
        })
        .detach();
    }

    pub fn discard(&mut self, cx: &mut Context<Self>) {
        if self.cell.is_none() {
            return;
        }
        self.loading = true;
        self.naiom_task = None;
        self.lodmod_task = None;
        cx.notify();
        let task = self.run(cx, Logic::discard_changes, |this, result, cx| {
            this.loading = false;
            match result {
                Ok(()) => cx.emit(ConfigEvent::Loaded),
                Err(error) => this.fail(ConfigOp::Discard, error, cx),
            }
            cx.notify();
        });
        self.load_task = Some(task);
    }

    pub fn reload_if_clean(&mut self, cx: &mut Context<Self>) {
        let task = self.run(
            cx,
            |logic| {
                let busy = logic.data().has_unsaved_changes
                    || logic.has_pending_naiom()
                    || logic.has_pending_lodmod();
                if busy {
                    return Ok(false);
                }
                logic.load().map(|()| true)
            },
            |this, result, cx| match result {
                Ok(true) => cx.emit(ConfigEvent::ReloadedExternally),
                Ok(false) => {}
                Err(error) => this.fail(ConfigOp::ExternalReload, error, cx),
            },
        );
        self.reload_task = Some(task);
    }

    pub fn update_nams(
        &mut self,
        key: &str,
        value: toml::Value,
        section: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let key = key.to_owned();
        let section = section.map(str::to_owned);
        self.edit(cx, move |logic| {
            logic.update_nams(&key, value, section.as_deref())
        });
    }

    pub fn update_nams_now(
        &mut self,
        key: &str,
        value: toml::Value,
        section: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let key = key.to_owned();
        let section = section.map(str::to_owned);
        self.run(
            cx,
            move |logic| logic.update_nams_now(&key, value, section.as_deref()),
            |this, result, cx| match result {
                Ok(true) => cx.emit(ConfigEvent::Written(ConfigFile::Nams)),
                Ok(false) => {}
                Err(error) => this.fail(ConfigOp::WriteNow, error, cx),
            },
        )
        .detach();
    }

    pub fn update_naiom(
        &mut self,
        key: &str,
        value: toml::Value,
        section: &str,
        cx: &mut Context<Self>,
    ) {
        let key = key.to_owned();
        let section = section.to_owned();
        self.edit(cx, move |logic| {
            logic.update_naiom_live(&key, value, &section)
        });
        self.naiom_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(NAIOM_AUTOSAVE_DELAY).await;
            this.update(cx, |this, cx| {
                this.naiom_task = None;
                this.flush_naiom(cx).detach();
            })
            .ok();
        }));
    }

    fn flush_naiom(&mut self, cx: &mut Context<Self>) -> Task<()> {
        self.run(cx, Logic::flush_naiom, |this, result, cx| match result {
            Ok(true) => cx.emit(ConfigEvent::Written(ConfigFile::Naiom)),
            Ok(false) => {}
            Err(error) => this.fail(ConfigOp::Autosave(ConfigFile::Naiom), error, cx),
        })
    }

    pub fn update_lodmod(
        &mut self,
        key: &str,
        value: toml::Value,
        section: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let key = key.to_owned();
        let section = section.map(str::to_owned);
        self.edit(cx, move |logic| {
            logic.update_lodmod(&key, value, section.as_deref())
        });
    }

    pub fn update_lodmod_live(
        &mut self,
        key: &str,
        value: toml::Value,
        section: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let key = key.to_owned();
        let section = section.map(str::to_owned);
        self.edit(cx, move |logic| {
            logic.update_lodmod_live(&key, value, section.as_deref())
        });
        self.lodmod_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(LODMOD_AUTOSAVE_DELAY).await;
            this.update(cx, |this, cx| {
                this.lodmod_task = None;
                this.flush_lodmod(cx).detach();
            })
            .ok();
        }));
    }

    fn flush_lodmod(&mut self, cx: &mut Context<Self>) -> Task<()> {
        self.run(cx, Logic::flush_lodmod, |this, result, cx| match result {
            Ok(true) => cx.emit(ConfigEvent::Written(ConfigFile::LodMod)),
            Ok(false) => {}
            Err(error) => this.fail(ConfigOp::Autosave(ConfigFile::LodMod), error, cx),
        })
    }

    pub fn flush_pending(&mut self, cx: &mut Context<Self>) -> Task<()> {
        let naiom = self.naiom_task.take().map(|_| self.flush_naiom(cx));
        let lodmod = self.lodmod_task.take().map(|_| self.flush_lodmod(cx));
        cx.foreground_executor().spawn(async move {
            if let Some(naiom) = naiom {
                naiom.await;
            }
            if let Some(lodmod) = lodmod {
                lodmod.await;
            }
        })
    }

    fn flush_on_quit(&mut self) -> impl Future<Output = ()> + use<> {
        let naiom = self.naiom_task.take().is_some();
        let lodmod = self.lodmod_task.take().is_some();
        let writes = self.cell.as_ref().map(|cell| {
            (
                naiom.then(|| cell.spawn(|logic| logic.flush_naiom().is_ok())),
                lodmod.then(|| cell.spawn(|logic| logic.flush_lodmod().is_ok())),
            )
        });
        async move {
            let Some((naiom, lodmod)) = writes else {
                return;
            };
            for write in [naiom, lodmod].into_iter().flatten() {
                if write.await != Ok(true) {
                    log::warn!("an automatic config save did not finish before quitting");
                }
            }
        }
    }

    pub fn apply_lodmod_bloom_2017_preset(&mut self, cx: &mut Context<Self>) {
        self.lodmod_task = None;
        self.run(
            cx,
            Logic::apply_lodmod_bloom_2017_preset,
            |this, result, cx| match result {
                Ok(true) => cx.emit(ConfigEvent::Written(ConfigFile::LodMod)),
                Ok(false) => {}
                Err(error) => this.fail(ConfigOp::LodModPreset, error, cx),
            },
        )
        .detach();
    }

    pub fn reset_lodmod_defaults(&mut self, cx: &mut Context<Self>) {
        self.lodmod_task = None;
        self.run(
            cx,
            Logic::reset_lodmod_to_defaults,
            |this, result, cx| match result {
                Ok(true) => cx.emit(ConfigEvent::Written(ConfigFile::LodMod)),
                Ok(false) => {}
                Err(error) => this.fail(ConfigOp::LodModReset, error, cx),
            },
        )
        .detach();
    }

    pub fn update_texture_injection(
        &mut self,
        key: &str,
        value: toml::Value,
        cx: &mut Context<Self>,
    ) {
        let key = key.to_owned();
        self.edit(cx, move |logic| logic.update_texture_injection(&key, value));
    }

    pub fn update_texture_injection_silent(
        &mut self,
        key: &str,
        value: toml::Value,
        cx: &mut Context<Self>,
    ) {
        let key = key.to_owned();
        self.edit(cx, move |logic| {
            logic.update_texture_injection_silent(&key, value)
        });
    }

    pub fn reconcile_texture_load_order(&mut self, folders: Vec<String>, cx: &mut Context<Self>) {
        let key = yp_config::fields::texture_injection::LOAD_ORDER.key();
        self.run(
            cx,
            move |logic| {
                let data = logic.data();
                if data.texture_injection_raw.is_empty() {
                    return false;
                }
                let current: Vec<String> = data
                    .texture_injection_values
                    .get(key)
                    .and_then(toml::Value::as_array)
                    .map(|packs| {
                        packs
                            .iter()
                            .filter_map(|pack| pack.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
                let Some(order) = yp_mods::textures::reconciled_load_order(&current, &folders)
                else {
                    return false;
                };
                let value =
                    toml::Value::Array(order.into_iter().map(toml::Value::String).collect());
                logic.update_texture_injection_silent(key, value);
                true
            },
            |this, changed, cx| {
                if changed {
                    this.save(cx);
                }
            },
        )
        .detach();
    }

    pub fn auto_detect_cutscenes(&mut self, cx: &mut Context<Self>) {
        let (Some(game), Some(cell)) = (self.game.clone(), self.cell.clone()) else {
            return;
        };
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let scan = cx.background_spawn(async move { yp_cutscenes::detect(&game, &cancel) });
        self.cutscene_task = Some(cx.spawn(async move |this, cx| {
            let _guard = guard;
            let detection = match scan.await {
                Ok(detection) => detection,
                Err(error) => {
                    if !error.is_cancelled() {
                        log::warn!("cutscene detection failed: {error}");
                    }
                    return;
                }
            };
            let flags = CutsceneFlags {
                needs_h264: detection.needs_h264,
            };
            let applied: oneshot::Receiver<_> =
                cell.spawn(move |logic| logic.apply_cutscene_flags(flags));
            let Ok(applied) = applied.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.sync(cx);
                match applied {
                    Ok(changes) if changes.is_empty() => {}
                    Ok(changes) => cx.emit(ConfigEvent::CutsceneFlagsApplied(changes)),
                    Err(error) => this.fail(ConfigOp::CutsceneFlags, error, cx),
                }
            })
            .ok();
        }));
    }

    fn start_watching(&mut self, cx: &mut Context<Self>) {
        if !self.watch_files || self.watch.is_some() {
            return;
        }
        let Some(game) = &self.game else {
            return;
        };
        let targets = vec![(game.nams_dir(), Recursive::NO)];
        self.watch = Some(Watch::start(targets, cx, |this, paths, cx| {
            this.on_external_change(&paths, cx)
        }));
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let relevant = paths.iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(Logic::is_watched_file)
        });
        if !relevant {
            return;
        }
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DEBOUNCE).await;
            let reload = this.update(cx, |this, cx| {
                this.run(
                    cx,
                    |logic| logic.reload_external(Instant::now()),
                    |this, result, cx| match result {
                        Ok(true) => {
                            cx.emit(ConfigEvent::ReloadedExternally);
                            this.auto_detect_cutscenes(cx);
                        }
                        Ok(false) => {}
                        Err(error) => this.fail(ConfigOp::ExternalReload, error, cx),
                    },
                )
            });
            if let Ok(reload) = reload {
                reload.await;
            }
        }));
    }
}
