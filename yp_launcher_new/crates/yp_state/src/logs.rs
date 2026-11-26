use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{Context, EventEmitter, Task};
use yp_core::GamePaths;
use yp_core::consts::{NAMS_LOG_NAME, YORHA_LOG_NAME};
use yp_host::launch::nams_dll;
use yp_host::nams_log::{LogTail, POLL_INTERVAL_MS, TailUpdate};

pub use yp_host::nams_log::panel::{LOG_LEVELS, LogPanel, LogTab};
pub use yp_host::nams_log::{LogEntry, LogLevel};

use crate::cell::Guarded;
use crate::env::AppEnv;

pub const LOG_POLL_INTERVAL: Duration = Duration::from_millis(POLL_INTERVAL_MS);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogEvent {
    Appended { modloader: usize, yorha: usize },
    Reset,
    StreamingChanged(bool),
}

struct Tailing {
    nams: LogTail,
    yorha: LogTail,
    separate_files: Option<bool>,
    pending: Vec<(Option<LogTab>, TailUpdate)>,
}

impl Tailing {
    fn new(log_file: PathBuf) -> Self {
        Self {
            yorha: LogTail::new(log_file.with_file_name(YORHA_LOG_NAME)),
            nams: LogTail::new(log_file),
            separate_files: None,
            pending: Vec::new(),
        }
    }

    fn poll(&mut self) {
        let separate = self.yorha.path().is_file();
        if self.separate_files.replace(separate).is_some_and(|old| old != separate) {
            self.nams = LogTail::new(self.nams.path().to_owned());
            self.yorha = LogTail::new(self.yorha.path().to_owned());
            self.pending.push((None, TailUpdate::Reset));
        }
        self.pending.push((separate.then_some(LogTab::Modloader), self.nams.poll()));
        if separate {
            self.pending.push((Some(LogTab::Yorha), self.yorha.poll()));
        }
    }
}

pub struct LogStore {
    env: AppEnv,
    game_dir: Option<PathBuf>,
    dll_mode: bool,
    log_file: PathBuf,
    logs_dir: PathBuf,
    panel: LogPanel,
    filtered: Vec<LogEntry>,
    tail: Guarded<Tailing>,
    streaming: bool,
    stream_task: Option<Task<()>>,
}

impl EventEmitter<LogEvent> for LogStore {}

impl LogStore {
    pub fn new(env: &AppEnv, cx: &mut Context<Self>) -> Self {
        let log_file = env.paths.nams_log();
        let logs_dir = env.paths.logs_dir.clone();
        Self {
            tail: Guarded::new(Tailing::new(log_file.clone()), cx.background_executor()),
            dll_mode: nams_dll::picked(&env.settings).is_some(),
            env: env.clone(),
            game_dir: None,
            log_file,
            logs_dir,
            panel: LogPanel::default(),
            filtered: Vec::new(),
            streaming: false,
            stream_task: None,
        }
    }

    pub fn panel(&self) -> &LogPanel {
        &self.panel
    }

    pub fn log_file(&self) -> &Path {
        &self.log_file
    }

    pub fn logs_dir(&self) -> &Path {
        &self.logs_dir
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        self.game_dir = game.map(|game| game.root().to_path_buf());
        self.relocate(cx);
    }

    pub fn set_dll_mode(&mut self, dll_mode: bool, cx: &mut Context<Self>) {
        self.dll_mode = dll_mode;
        self.relocate(cx);
    }

    fn relocate(&mut self, cx: &mut Context<Self>) {
        let logs_dir = nams_dll::logs_dir_for(&self.env.paths, self.dll_mode, self.game_dir.as_deref());
        if logs_dir == self.logs_dir {
            return;
        }
        self.log_file = logs_dir.join(NAMS_LOG_NAME);
        self.logs_dir = logs_dir;
        if self.streaming {
            self.refresh(cx);
        } else {
            self.panel.reset();
            self.refilter();
            cx.emit(LogEvent::Reset);
            cx.notify();
        }
    }

    pub fn entries(&self, tab: LogTab) -> &[LogEntry] {
        match tab {
            LogTab::Modloader => &self.panel.modloader_entries,
            LogTab::Yorha => &self.panel.yorha_entries,
        }
    }

    pub fn active_tab(&self) -> LogTab {
        self.panel.active_tab
    }

    pub fn active_entries(&self) -> &[LogEntry] {
        self.panel.active_entries()
    }

    pub fn query(&self) -> &str {
        &self.panel.search_query
    }

    pub fn is_level_shown(&self, level: LogLevel) -> bool {
        self.panel.is_level_shown(level)
    }

    pub fn hidden_levels(&self) -> impl Iterator<Item = LogLevel> + '_ {
        self.panel.hidden_levels.iter().copied()
    }

    pub fn is_loading(&self) -> bool {
        self.panel.is_loading
    }

    pub fn is_streaming(&self) -> bool {
        self.streaming
    }

    pub fn filtered(&self) -> &[LogEntry] {
        &self.filtered
    }

    pub fn counts(&self) -> BTreeMap<LogLevel, usize> {
        self.panel.level_counts()
    }

    fn refilter(&mut self) {
        self.filtered = self.panel.filtered_entries().into_iter().cloned().collect();
    }

    fn apply(&mut self, source: Option<LogTab>, update: TailUpdate, cx: &mut Context<Self>) {
        let was_loading = std::mem::take(&mut self.panel.is_loading);
        match &update {
            TailUpdate::Unchanged => {
                if was_loading {
                    cx.notify();
                }
                return;
            }
            TailUpdate::Reset => cx.emit(LogEvent::Reset),
            TailUpdate::Appended(_) => {}
        }
        let before = (
            self.panel.modloader_entries.len(),
            self.panel.yorha_entries.len(),
        );
        let appended = matches!(update, TailUpdate::Appended(_));
        if let Some(tab) = source {
            self.panel.apply_from(tab, update);
        } else {
            self.panel.apply(update);
        }
        if appended {
            cx.emit(LogEvent::Appended {
                modloader: self.panel.modloader_entries.len().saturating_sub(before.0),
                yorha: self.panel.yorha_entries.len().saturating_sub(before.1),
            });
        }
        self.refilter();
        cx.notify();
    }

    fn drain(&mut self, cx: &mut Context<Self>) {
        let Some(updates) = self
            .tail
            .take(|tailing| std::mem::take(&mut tailing.pending))
        else {
            return;
        };
        for (source, update) in updates {
            self.apply(source, update, cx);
        }
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.panel.reset();
        self.panel.is_loading = true;
        self.refilter();
        let log_file = self.log_file.clone();
        self.tail
            .edit(move |tailing| *tailing = Tailing::new(log_file), |_| ());
        cx.emit(LogEvent::Reset);
        cx.notify();
        self.start_streaming(cx);
    }

    pub fn start_streaming(&mut self, cx: &mut Context<Self>) {
        let tail = self.tail.clone();
        if !self.streaming {
            self.streaming = true;
            cx.emit(LogEvent::StreamingChanged(true));
            cx.notify();
        }
        self.stream_task = Some(cx.spawn(async move |this, cx| {
            loop {
                if tail.spawn(Tailing::poll).await.is_err() {
                    break;
                }
                if this.update(cx, |this, cx| this.drain(cx)).is_err() {
                    break;
                }
                cx.background_executor().timer(LOG_POLL_INTERVAL).await;
            }
        }));
    }

    pub fn stop_streaming(&mut self, cx: &mut Context<Self>) {
        self.stream_task = None;
        if self.streaming {
            self.streaming = false;
            cx.emit(LogEvent::StreamingChanged(false));
            cx.notify();
        }
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.stop_streaming(cx);
        self.load(cx);
    }

    pub fn set_active_tab(&mut self, tab: LogTab, cx: &mut Context<Self>) {
        if self.panel.active_tab != tab {
            self.panel.set_active_tab(tab);
            self.refilter();
            cx.notify();
        }
    }

    pub fn set_search_query(&mut self, query: impl Into<String>, cx: &mut Context<Self>) {
        let query = query.into();
        if self.panel.search_query != query {
            self.panel.set_search_query(query);
            self.refilter();
            cx.notify();
        }
    }

    pub fn toggle_level(&mut self, level: LogLevel, cx: &mut Context<Self>) {
        self.panel.toggle_level(level);
        self.refilter();
        cx.notify();
    }
}
