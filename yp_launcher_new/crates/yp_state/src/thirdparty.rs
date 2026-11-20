use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gpui::AppContext as _;
use gpui::{App, Context, EventEmitter, Task};
use yp_core::{CancelToken, GamePaths, NoProgress};
use yp_core::watch::Recursive;
use yp_host::session::BusyCounter;
use yp_thirdparty::{
    Classification, GameModSettings, InstallOutcome, MigotoConfig, ReShadeConfig, RuntimePresence,
    RuntimeStatus, StageFailure, StagedSource, ThirdPartyRuntime, ThirdPartyState,
    ThirdPartyStatuses, ThirdPartyUpdateInfo,
};

pub use yp_thirdparty::ThirdPartyNotice;

use crate::cell::{CancelOnDrop, Guarded};
use crate::env::AppEnv;
use crate::watch::Watch;

pub const GAME_ROOT_IMPORT_DEBOUNCE: Duration = Duration::from_millis(600);

pub type ThirdPartyError = Arc<yp_thirdparty::Error>;

#[derive(Debug, Clone)]
pub enum ThirdPartyEvent {
    Changed,
    Notice(ThirdPartyNotice),
    Failed(ThirdPartyError),
}

pub struct ThirdPartyStore {
    env: AppEnv,
    game: Option<GamePaths>,
    cell: Guarded<ThirdPartyState>,
    statuses: ThirdPartyStatuses,
    presence: [RuntimePresence; 4],
    loading: bool,
    migrating: bool,
    busy: BusyCounter,
    error: Option<ThirdPartyError>,
    refresh_task: Option<Task<()>>,
    import_task: Option<Task<()>>,
    sync_task: Option<Task<()>>,
    watch: Option<Watch>,
}

impl EventEmitter<ThirdPartyEvent> for ThirdPartyStore {}

impl ThirdPartyStore {
    pub fn new(env: AppEnv, cx: &mut Context<Self>) -> Self {
        let state = ThirdPartyState::new(&env.host, &env.paths);
        Self {
            cell: Guarded::new(state, cx.background_executor()),
            env,
            game: None,
            statuses: ThirdPartyStatuses::default(),
            presence: [RuntimePresence::NotInstalled; 4],
            loading: false,
            migrating: false,
            busy: BusyCounter::new(),
            error: None,
            refresh_task: None,
            import_task: None,
            sync_task: None,
            watch: None,
        }
    }

    pub fn statuses(&self) -> &ThirdPartyStatuses {
        &self.statuses
    }

    pub fn status(&self, which: ThirdPartyRuntime) -> &RuntimeStatus {
        self.statuses.get(which)
    }

    pub fn reshade(&self) -> &RuntimeStatus {
        &self.statuses.reshade
    }

    pub fn migoto(&self) -> &RuntimeStatus {
        &self.statuses.migoto
    }

    pub fn game_mods(&self) -> &RuntimeStatus {
        &self.statuses.game_mods
    }

    pub fn graphic_mods(&self) -> &RuntimeStatus {
        &self.statuses.graphic_mods
    }

    pub fn presence(&self, which: ThirdPartyRuntime) -> RuntimePresence {
        ThirdPartyRuntime::ALL
            .iter()
            .position(|runtime| *runtime == which)
            .map_or(RuntimePresence::NotInstalled, |index| self.presence[index])
    }

    pub fn existing_install_dir(
        &self,
        which: ThirdPartyRuntime,
        cx: &App,
    ) -> Task<Option<PathBuf>> {
        let job = self.cell.spawn(move |state| state.existing_install_dir(which));
        cx.foreground_executor()
            .spawn(async move { job.await.ok().flatten() })
    }

    pub fn existing_shaders_dir(&self, cx: &App) -> Task<Option<PathBuf>> {
        let job = self.cell.spawn(|state| state.existing_shaders_dir());
        cx.foreground_executor()
            .spawn(async move { job.await.ok().flatten() })
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn is_busy(&self) -> bool {
        self.busy.is_busy()
    }

    pub fn error(&self) -> Option<&ThirdPartyError> {
        self.error.as_ref()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        if self.game == game {
            return;
        }
        self.game = game.clone();
        self.statuses = ThirdPartyStatuses::default();
        self.loading = false;
        self.migrating = false;
        self.refresh_task = None;
        self.import_task = None;
        self.watch = None;
        self.error = None;
        self.cell.edit(move |state| state.set_game(game), |_| ());
        cx.notify();
    }

    fn sync(&mut self, cx: &mut Context<Self>) {
        let Some((statuses, presence)) = self.cell.peek(|state| {
            (
                state.statuses().clone(),
                ThirdPartyRuntime::ALL.map(|which| state.presence(which)),
            )
        }) else {
            let settled = self.cell.settled();
            self.sync_task = Some(cx.spawn(async move |this, cx| {
                if settled.await.is_ok() {
                    this.update(cx, |this, cx| {
                        this.sync(cx);
                        cx.notify();
                    })
                    .ok();
                }
            }));
            return;
        };
        if self.statuses != statuses || self.presence != presence {
            self.statuses = statuses;
            self.presence = presence;
            cx.emit(ThirdPartyEvent::Changed);
        }
    }

    fn fail(&mut self, error: yp_thirdparty::Error, cx: &mut Context<Self>) {
        log::warn!("third-party operation failed: {error}");
        let error = Arc::new(error);
        self.error = Some(error.clone());
        cx.emit(ThirdPartyEvent::Failed(error));
    }

    fn run<R: Send + 'static>(
        &mut self,
        cx: &mut Context<Self>,
        marks_busy: bool,
        work: impl FnOnce(&mut ThirdPartyState) -> R + Send + 'static,
        done: impl FnOnce(&mut Self, R, &mut Context<Self>) + 'static,
    ) -> Task<()> {
        let busy = marks_busy.then(|| self.busy.enter());
        if marks_busy {
            cx.notify();
        }
        let job = self.cell.spawn(work);
        cx.spawn(async move |this, cx| {
            let outcome = job.await;
            drop(busy);
            let Ok(result) = outcome else {
                return;
            };
            this.update(cx, |this, cx| {
                this.sync(cx);
                done(this, result, cx);
                cx.notify();
            })
            .ok();
        })
    }

    fn apply(
        &mut self,
        cx: &mut Context<Self>,
        marks_busy: bool,
        work: impl FnOnce(&mut ThirdPartyState) -> yp_thirdparty::Result<()> + Send + 'static,
    ) {
        self.run(cx, marks_busy, work, |this, result, cx| {
            if let Err(error) = result {
                this.fail(error, cx);
            }
        })
        .detach();
    }

    fn announce(&mut self, notices: Vec<ThirdPartyNotice>, cx: &mut Context<Self>) {
        for notice in notices {
            cx.emit(ThirdPartyEvent::Notice(notice));
        }
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.game.is_none() {
            return;
        }
        self.loading = true;
        self.error = None;
        cx.notify();
        let platform = self.env.platform.clone();
        let task = self.run(
            cx,
            false,
            move |state| state.refresh(platform.is_game_running()),
            |this, result, cx| {
                this.loading = false;
                match result {
                    Ok(notices) => this.announce(notices, cx),
                    Err(error) => this.fail(error, cx),
                }
                this.start_watching(cx);
            },
        );
        self.refresh_task = Some(task);
    }

    pub fn migrate_legacy(&mut self, cx: &mut Context<Self>) {
        if self.game.is_none() || self.migrating {
            return;
        }
        self.migrating = true;
        let platform = self.env.platform.clone();
        self.run(
            cx,
            false,
            move |state| state.migrate_legacy(platform.is_game_running()),
            |this, notices, cx| {
                this.migrating = false;
                this.announce(notices, cx);
            },
        )
        .detach();
    }

    pub fn stage(
        &self,
        source: impl Into<PathBuf>,
        cx: &App,
    ) -> Task<Result<StagedSource, StageFailure>> {
        let env = self.env.clone();
        let game = self.game.clone();
        let source = source.into();
        let cancel = CancelToken::new();
        let cancel_when_dropped = CancelOnDrop(cancel.clone());
        let job = cx.background_spawn(async move {
            yp_thirdparty::stage_source(
                &env.extractor(),
                &env.host,
                game.as_ref(),
                &source,
                &NoProgress,
                &cancel,
            )
        });
        cx.foreground_executor().spawn(async move {
            let staged = job.await;
            drop(cancel_when_dropped);
            staged
        })
    }

    pub fn classify(&self, extracted_root: impl Into<PathBuf>, cx: &App) -> Task<Classification> {
        let root = extracted_root.into();
        cx.background_spawn(async move { yp_thirdparty::classify(&root) })
    }

    pub fn would_update(
        &self,
        classification: Classification,
        cx: &App,
    ) -> Task<Option<ThirdPartyUpdateInfo>> {
        let job = self
            .cell
            .spawn(move |state| state.would_update(&classification));
        cx.foreground_executor()
            .spawn(async move { job.await.ok().flatten() })
    }

    pub fn install(
        &mut self,
        classification: Classification,
        cx: &mut Context<Self>,
    ) -> Task<Result<Option<InstallOutcome>, ThirdPartyError>> {
        let busy = self.busy.enter();
        cx.notify();
        let job = self.cell.spawn(move |state| state.install(&classification));
        cx.spawn(async move |this, cx| {
            let outcome = job.await;
            drop(busy);
            let Ok(result) = outcome else {
                return Ok(None);
            };
            let result = result.map_err(Arc::new);
            this.update(cx, |this, cx| {
                this.sync(cx);
                if let Err(error) = &result {
                    this.error = Some(error.clone());
                    cx.emit(ThirdPartyEvent::Failed(error.clone()));
                }
                cx.notify();
            })
            .ok();
            result
        })
    }

    pub fn import_from_game_root(&mut self, which: ThirdPartyRuntime, cx: &mut Context<Self>) {
        self.apply(cx, true, move |state| {
            state.import_from_game_root(which).map(drop)
        });
    }

    pub fn set_enabled(&mut self, which: ThirdPartyRuntime, enabled: bool, cx: &mut Context<Self>) {
        self.apply(cx, false, move |state| state.set_enabled(which, enabled));
    }

    pub fn apply_reshade_config(&mut self, config: ReShadeConfig, cx: &mut Context<Self>) {
        self.apply(cx, false, move |state| state.apply_reshade_config(&config));
    }

    pub fn apply_migoto_config(&mut self, config: MigotoConfig, cx: &mut Context<Self>) {
        self.apply(cx, false, move |state| state.apply_migoto_config(&config));
    }

    pub fn set_game_mod_disabled(
        &mut self,
        file_name: &str,
        disabled: bool,
        cx: &mut Context<Self>,
    ) {
        let file_name = file_name.to_owned();
        self.apply(cx, false, move |state| {
            state.set_game_mod_disabled(&file_name, disabled)
        });
    }

    pub fn set_game_mod_settings(
        &mut self,
        file_name: &str,
        settings: GameModSettings,
        cx: &mut Context<Self>,
    ) {
        let file_name = file_name.to_owned();
        self.apply(cx, false, move |state| {
            state.set_game_mod_settings(&file_name, &settings)
        });
    }

    pub fn set_graphic_mod_disabled(
        &mut self,
        mod_dir: &str,
        disabled: bool,
        cx: &mut Context<Self>,
    ) {
        let mod_dir = mod_dir.to_owned();
        self.apply(cx, false, move |state| {
            state.set_graphic_mod_disabled(&mod_dir, disabled)
        });
    }

    pub fn remove_graphic_mod(&mut self, mod_dir: &str, cx: &mut Context<Self>) {
        let mod_dir = mod_dir.to_owned();
        self.apply(cx, true, move |state| state.remove_graphic_mod(&mod_dir));
    }

    pub fn remove_game_mod(&mut self, mod_dir: &str, cx: &mut Context<Self>) {
        let mod_dir = mod_dir.to_owned();
        self.apply(cx, true, move |state| state.remove_game_mod(&mod_dir));
    }

    pub fn move_mod(&mut self, mod_dir: &str, from: ThirdPartyRuntime, cx: &mut Context<Self>) {
        let mod_dir = mod_dir.to_owned();
        self.apply(cx, true, move |state| {
            state.move_mod(&mod_dir, from).map(drop)
        });
    }

    pub fn remove(&mut self, which: ThirdPartyRuntime, cx: &mut Context<Self>) {
        self.apply(cx, true, move |state| state.remove(which));
    }

    fn start_watching(&mut self, cx: &mut Context<Self>) {
        if !self.env.watch_files || self.watch.is_some() {
            return;
        }
        let Some(game) = &self.game else {
            return;
        };
        let targets = vec![(game.root().to_path_buf(), Recursive::NO)];
        self.watch = Some(Watch::start(targets, cx, |this, paths, cx| {
            this.on_external_change(paths, cx)
        }));
    }

    pub fn on_external_change(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        if !paths
            .iter()
            .any(|path| ThirdPartyState::is_relevant_change(path))
        {
            return;
        }
        self.import_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(GAME_ROOT_IMPORT_DEBOUNCE)
                .await;
            let import = this.update(cx, |this, cx| {
                (!this.is_busy()).then(|| {
                    this.run(
                        cx,
                        false,
                        move |state| state.on_game_root_changed(&paths),
                        |this, result, cx| {
                            if let Err(error) = result {
                                this.fail(error, cx);
                            }
                        },
                    )
                })
            });
            if let Ok(Some(import)) = import {
                import.await;
            }
        }));
    }
}
