use std::path::PathBuf;

use futures::channel::oneshot;
use gpui::AppContext as _;
use gpui::{App, Context, EventEmitter, Task};
use yp_core::{CancelToken, GamePaths, NierInstallation, SelectionRejection};
use yp_health::{CacheClearResult, Collector, DiagnosticsReport};
use yp_host::game_probe::ExeSelection;
use yp_host::launch::LaunchOutcome;
use yp_host::launch::failure::LaunchFailure;
use yp_host::launch::record::LaunchRecord;
use yp_host::nams_cli::verify::VerifyOutcome;
use yp_host::session::AppSession;
use yp_host::win10_compat;

pub use yp_host::session::{LaunchStatus, PlayButtonState as PlayState, SessionError};

use crate::cell::CancelOnDrop;
use crate::env::AppEnv;
use crate::prefs::{NIER_DIRECTORY, save_setting};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LaunchRequest {
    pub vanilla: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionCheck {
    Accepted,
    Missing,
    NotNierExe,
    Rejected(SelectionRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectoryChange {
    Restored,
    Selected,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameError {
    Session(SessionError),
    LaunchFailed(LaunchFailure),
    StopFailed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    DirectoryChanged {
        game: GamePaths,
        change: DirectoryChange,
    },
    PlayStateChanged {
        state: PlayState,
        status: Option<LaunchStatus>,
    },
    LaunchRequested {
        vanilla: bool,
    },
    Started(LaunchRecord),
    Stopped(Option<LaunchRecord>),
    LaunchFailed(LaunchFailure),
    StopFailed,
    CacheCleared(CacheClearResult),
}

pub struct GameSession {
    env: AppEnv,
    session: AppSession,
    error: Option<GameError>,
    auto_searching: bool,
    preparing: bool,
    prepare_task: Option<Task<()>>,
    launch_task: Option<Task<()>>,
    stop_task: Option<Task<()>>,
    sync_task: Option<Task<()>>,
}

impl EventEmitter<SessionEvent> for GameSession {}

impl GameSession {
    pub fn new(env: AppEnv, _cx: &mut Context<Self>) -> Self {
        let session = AppSession::load(&env.settings);
        Self {
            env,
            session,
            error: None,
            auto_searching: false,
            preparing: false,
            prepare_task: None,
            launch_task: None,
            stop_task: None,
            sync_task: None,
        }
    }

    pub fn selected_directory(&self) -> &str {
        &self.session.selected_directory
    }

    pub fn selected_dir(&self) -> Option<PathBuf> {
        self.session.directory()
    }

    pub fn game(&self) -> Option<GamePaths> {
        self.session.directory().map(GamePaths::new)
    }

    pub fn is_directory_selected(&self) -> bool {
        self.session.is_directory_selected()
    }

    pub fn is_preparing(&self) -> bool {
        self.preparing
    }

    pub fn play_state(&self) -> PlayState {
        self.session.play_button
    }

    pub fn status(&self) -> Option<LaunchStatus> {
        self.session.status
    }

    pub fn error(&self) -> Option<&GameError> {
        self.error.as_ref()
    }

    pub fn launch_failure(&self) -> Option<&LaunchFailure> {
        match &self.error {
            Some(GameError::LaunchFailed(failure)) => Some(failure),
            _ => None,
        }
    }

    pub fn auto_searching(&self) -> bool {
        self.auto_searching
    }

    pub fn can_play(&self) -> bool {
        self.session.can_play()
    }

    pub fn can_launch_game(&self) -> bool {
        self.env.can_launch_game()
    }

    pub fn set_auto_searching(&mut self, searching: bool, cx: &mut Context<Self>) {
        if self.auto_searching != searching {
            self.auto_searching = searching;
            cx.notify();
        }
    }

    pub fn set_error(&mut self, error: SessionError, cx: &mut Context<Self>) {
        self.session.set_error(error.clone());
        self.error = Some(GameError::Session(error));
        cx.notify();
    }

    pub fn clear_error(&mut self, cx: &mut Context<Self>) {
        self.session.clear_error();
        self.error = None;
        cx.notify();
    }

    pub fn restore(&mut self, cx: &mut Context<Self>) {
        self.sync_game_running(cx);
        let Some(game) = self.game() else {
            return;
        };
        self.prepare(game, DirectoryChange::Restored, cx);
    }

    pub fn set_directory(&mut self, path: impl Into<PathBuf>, cx: &mut Context<Self>) {
        let path: PathBuf = path.into();
        let text = path.to_string_lossy().into_owned();
        self.session.selected_directory = text.clone();
        self.session.clear_error();
        self.error = None;
        save_setting(&self.env.settings, &NIER_DIRECTORY, &text, cx);
        cx.notify();
        self.prepare(GamePaths::new(path), DirectoryChange::Selected, cx);
    }

    fn prepare(&mut self, game: GamePaths, change: DirectoryChange, cx: &mut Context<Self>) {
        self.preparing = true;
        cx.notify();
        let env = self.env.clone();
        let target = game.clone();
        let job = cx.background_spawn(async move {
            if change == DirectoryChange::Selected {
                win10_compat::ensure_ran_for_directory(
                    &env.host,
                    env.runner.as_ref(),
                    &env.settings,
                    target.root(),
                );
            }
        });
        self.prepare_task = Some(cx.spawn(async move |this, cx| {
            job.await;
            this.update(cx, |this, cx| {
                this.preparing = false;
                cx.emit(SessionEvent::DirectoryChanged { game, change });
                cx.notify();
            })
            .ok();
        }));
    }

    fn set_play(&mut self, state: PlayState, status: Option<LaunchStatus>, cx: &mut Context<Self>) {
        let changed = self.session.play_button != state || self.session.status != status;
        self.session.set_play_button(state);
        self.session.set_status(status);
        if changed {
            cx.emit(SessionEvent::PlayStateChanged { state, status });
        }
        cx.notify();
    }

    pub fn sync_game_running(&mut self, cx: &mut Context<Self>) {
        let platform = self.env.platform.clone();
        let probe = cx.background_spawn(async move { platform.is_game_running() });
        self.sync_task = Some(cx.spawn(async move |this, cx| {
            let running = probe.await;
            this.update(cx, |this, cx| {
                if this.session.sync_game_running(running) {
                    this.error = None;
                    cx.emit(SessionEvent::PlayStateChanged {
                        state: this.session.play_button,
                        status: this.session.status,
                    });
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    pub fn launch(&mut self, request: LaunchRequest, cx: &mut Context<Self>) {
        if self.session.play_button != PlayState::Idle {
            return;
        }
        let Some(dir) = self.session.directory() else {
            return;
        };
        self.error = None;
        cx.emit(SessionEvent::LaunchRequested {
            vanilla: request.vanilla,
        });
        self.set_play(PlayState::Loading, Some(LaunchStatus::Launching), cx);

        let env = self.env.clone();
        let job = cx.background_spawn(async move {
            let (stopped, on_stopped) = oneshot::channel::<LaunchRecord>();
            let outcome = env.launcher().start(&dir, request.vanilla, move |record| {
                let _ = stopped.send(record);
            });
            (outcome, on_stopped)
        });
        self.launch_task = Some(cx.spawn(async move |this, cx| {
            let (outcome, on_stopped) = job.await;
            let started = this
                .update(cx, |this, cx| match outcome {
                    LaunchOutcome::Started { record } => {
                        this.set_play(PlayState::Running, Some(LaunchStatus::Running), cx);
                        cx.emit(SessionEvent::Started(record));
                        true
                    }
                    LaunchOutcome::Failed(failure) => {
                        this.set_play(PlayState::Idle, None, cx);
                        this.error = Some(GameError::LaunchFailed(failure.clone()));
                        cx.emit(SessionEvent::LaunchFailed(failure));
                        cx.notify();
                        false
                    }
                })
                .unwrap_or(false);
            if !started {
                return;
            }
            let Ok(record) = on_stopped.await else {
                return;
            };
            this.update(cx, |this, cx| {
                if this.session.play_button != PlayState::Idle {
                    this.set_play(PlayState::Idle, Some(LaunchStatus::Stopped), cx);
                    cx.emit(SessionEvent::Stopped(Some(record)));
                }
            })
            .ok();
        }));
    }

    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if self.session.play_button != PlayState::Running {
            return;
        }
        self.error = None;
        self.set_play(PlayState::Loading, Some(LaunchStatus::Stopping), cx);
        let platform = self.env.platform.clone();
        let job = cx.background_spawn(async move { platform.terminate_game() });
        self.stop_task = Some(cx.spawn(async move |this, cx| {
            let stopped = job.await;
            this.update(cx, |this, cx| {
                if this.session.play_button != PlayState::Loading {
                    return;
                }
                if stopped {
                    this.set_play(PlayState::Idle, Some(LaunchStatus::Stopped), cx);
                    cx.emit(SessionEvent::Stopped(None));
                } else {
                    this.set_play(PlayState::Running, None, cx);
                    this.error = Some(GameError::StopFailed);
                    cx.emit(SessionEvent::StopFailed);
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    pub fn discover_fast(&self, cx: &App) -> Task<Vec<NierInstallation>> {
        let platform = self.env.platform.clone();
        cx.background_spawn(async move { platform.discover_fast() })
    }

    pub fn discover_deep(&self, cx: &App) -> Task<Vec<NierInstallation>> {
        let platform = self.env.platform.clone();
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let job = cx.background_spawn(async move { platform.discover_deep(&cancel) });
        cx.foreground_executor().spawn(async move {
            let _guard = guard;
            job.await
        })
    }

    pub fn check_selection(&self, exe: impl Into<PathBuf>, cx: &App) -> Task<SelectionCheck> {
        let platform = self.env.platform.clone();
        let exe = exe.into();
        cx.background_spawn(async move {
            match yp_host::game_probe::check_exe_selection(&exe) {
                ExeSelection::Missing => SelectionCheck::Missing,
                ExeSelection::NotNierExe => SelectionCheck::NotNierExe,
                ExeSelection::Valid => match platform.reject_game_selection(&exe) {
                    Some(rejection) => SelectionCheck::Rejected(rejection),
                    None => SelectionCheck::Accepted,
                },
            }
        })
    }

    pub fn launch_command_preview(&self, cx: &App) -> Task<Option<String>> {
        let env = self.env.clone();
        let dir = self.session.directory();
        cx.background_spawn(async move { dir.and_then(|dir| env.launcher().preview(&dir)) })
    }

    pub fn verify_install(&self, cx: &App) -> Task<Option<VerifyOutcome>> {
        let env = self.env.clone();
        let dir = self.session.directory();
        cx.background_spawn(async move {
            dir.map(|dir| {
                yp_host::nams_cli::verify(
                    env.platform.as_ref(),
                    env.runner.as_ref(),
                    &env.paths,
                    &dir,
                )
            })
        })
    }

    pub fn create_desktop_shortcut(&self, cx: &App) -> Task<bool> {
        let env = self.env.clone();
        let dir = self.session.directory();
        cx.background_spawn(async move {
            dir.is_some_and(|dir| yp_host::shortcut::create_desktop_shortcut(&env.paths, &dir))
        })
    }

    pub fn collect_diagnostics(&self, cx: &App) -> Task<yp_health::Result<DiagnosticsReport>> {
        let env = self.env.clone();
        let game = self.game();
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let job = cx.background_spawn(async move {
            let preview = game
                .as_ref()
                .and_then(|game| env.launcher().preview(game.root()));
            Collector::new(
                env.host.clone(),
                env.paths.clone(),
                env.settings.clone(),
                env.runner.clone(),
                env.platform_adapter(),
            )
            .collect(game.as_ref(), preview, &cancel)
        });
        cx.foreground_executor().spawn(async move {
            let _guard = guard;
            job.await
        })
    }

    pub fn clear_cache(&mut self, cx: &mut Context<Self>) -> Task<CacheClearResult> {
        let env = self.env.clone();
        let game = self.game();
        let job = cx.background_spawn(async move {
            yp_health::clear_all(
                &env.paths,
                &env.settings,
                env.platform.as_ref(),
                game.as_ref(),
            )
        });
        cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |_, cx| cx.emit(SessionEvent::CacheCleared(result)))
                .ok();
            result
        })
    }
}
