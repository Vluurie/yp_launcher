use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use yp_core::{Result, SelectionRejection, SettingKey, SettingsStore};

pub const NIER_DIRECTORY: SettingKey<String> = SettingKey::new("nier_directory", String::new);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlayButtonState {
    #[default]
    Idle,
    Loading,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LaunchStatus {
    Launching,
    Running,
    Stopped,
    Stopping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    InvalidExe,
    FileNotExist,
    Rejected(SelectionRejection),
    Message(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppSession {
    pub selected_directory: String,
    pub play_button: PlayButtonState,
    pub error: Option<SessionError>,
    pub status: Option<LaunchStatus>,
}

impl AppSession {
    pub fn load(settings: &SettingsStore) -> Self {
        Self {
            selected_directory: settings.get(&NIER_DIRECTORY),
            ..Self::default()
        }
    }

    pub fn saved_directory(settings: &SettingsStore) -> Option<PathBuf> {
        Some(settings.get(&NIER_DIRECTORY))
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
    }

    pub fn is_directory_selected(&self) -> bool {
        !self.selected_directory.is_empty()
    }

    pub fn can_play(&self) -> bool {
        self.is_directory_selected() && self.play_button != PlayButtonState::Loading
    }

    pub fn directory(&self) -> Option<PathBuf> {
        self.is_directory_selected().then(|| PathBuf::from(&self.selected_directory))
    }

    pub fn set_directory(&mut self, settings: &SettingsStore, path: &str) -> Result<()> {
        self.selected_directory = path.to_owned();
        self.error = None;
        self.status = None;
        settings.set(&NIER_DIRECTORY, &path.to_owned())
    }

    pub fn set_play_button(&mut self, state: PlayButtonState) {
        self.play_button = state;
        self.error = None;
        self.status = None;
    }

    pub fn set_error(&mut self, error: SessionError) {
        self.error = Some(error);
        self.status = None;
    }

    pub fn clear_error(&mut self) {
        self.error = None;
        self.status = None;
    }

    pub fn set_status(&mut self, status: Option<LaunchStatus>) {
        self.status = status;
        self.error = None;
    }

    pub fn sync_game_running(&mut self, running: bool) -> bool {
        match (running, self.play_button) {
            (true, PlayButtonState::Idle) => {
                self.play_button = PlayButtonState::Running;
                self.status = Some(LaunchStatus::Running);
                self.error = None;
                true
            }
            (false, PlayButtonState::Running) => {
                self.play_button = PlayButtonState::Idle;
                self.status = Some(LaunchStatus::Stopped);
                self.error = None;
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabSelectionRequest {
    pub tab_index: usize,
    pub key: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionFlags {
    pub auto_searching: bool,
    pub active_tab: usize,
    pub detection_refresh: u64,
    pub textures_busy: bool,
    pub pending_tab_selection: Option<TabSelectionRequest>,
}

impl SessionFlags {
    pub fn bump_detection_refresh(&mut self) -> u64 {
        self.detection_refresh = self.detection_refresh.wrapping_add(1);
        self.detection_refresh
    }

    pub fn request_tab_selection(&mut self, tab_index: usize, key: impl Into<String>) {
        self.pending_tab_selection = Some(TabSelectionRequest {
            tab_index,
            key: key.into(),
        });
        self.active_tab = tab_index;
    }

    pub fn take_tab_selection(&mut self, tab_index: usize) -> Option<String> {
        if self.pending_tab_selection.as_ref().is_some_and(|request| request.tab_index == tab_index) {
            return self.pending_tab_selection.take().map(|request| request.key);
        }
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct BusyCounter(Arc<AtomicUsize>);

#[derive(Debug)]
pub struct BusyGuard(Arc<AtomicUsize>);

impl BusyCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    pub fn is_busy(&self) -> bool {
        self.count() > 0
    }

    pub fn enter(&self) -> BusyGuard {
        self.0.fetch_add(1, Ordering::SeqCst);
        BusyGuard(self.0.clone())
    }

    pub fn run<T>(&self, task: impl FnOnce() -> T) -> T {
        let _guard = self.enter();
        task()
    }
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
