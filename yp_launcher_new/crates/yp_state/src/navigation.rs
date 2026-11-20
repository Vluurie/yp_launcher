use gpui::{App, Context, Entity, EventEmitter};
use yp_host::session::SessionFlags;

use crate::busy::BusyTracker;
use crate::config::ConfigStore;
use crate::nams_settings::NamsSettingsStore;
use crate::registry::OverlayRequest;
use crate::tabs::{OverlayId, TabId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabRequest {
    pub tab: TabId,
    pub select: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationBlock {
    TexturesBusy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabRequestOutcome {
    Switched,
    AlreadyActive,
    OverlayOpened(OverlayId),
    Blocked(NavigationBlock),
    ConfirmDiscard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseGuard {
    Busy { operations: usize },
    UnsavedChanges,
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationEvent {
    TabChanged { from: TabId, to: TabId },
    SelectionRequested(TabId),
    OverlayOpened(OverlayRequest),
    OverlayClosed(OverlayId),
    Blocked(NavigationBlock),
    DiscardRequested(TabRequest),
    LogsToggled(bool),
}

pub struct Navigation {
    flags: SessionFlags,
    overlays: Vec<OverlayRequest>,
    log_open: bool,
    awaiting_discard: Option<TabRequest>,
    busy: Entity<BusyTracker>,
    config: Entity<ConfigStore>,
    settings: Entity<NamsSettingsStore>,
}

impl EventEmitter<NavigationEvent> for Navigation {}

impl Navigation {
    pub fn new(
        busy: Entity<BusyTracker>,
        config: Entity<ConfigStore>,
        settings: Entity<NamsSettingsStore>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            flags: SessionFlags::default(),
            overlays: Vec::new(),
            log_open: false,
            awaiting_discard: None,
            busy,
            config,
            settings,
        }
    }

    pub fn active(&self) -> TabId {
        TabId::from_index(self.flags.active_tab).unwrap_or_default()
    }

    pub fn has_pending_selection(&self, tab: TabId) -> bool {
        self.flags
            .pending_tab_selection
            .as_ref()
            .is_some_and(|request| request.tab_index == tab.index())
    }

    pub fn overlays(&self) -> &[OverlayRequest] {
        &self.overlays
    }

    pub fn top_overlay(&self) -> Option<&OverlayRequest> {
        self.overlays.last()
    }

    pub fn is_overlay_open(&self, overlay: OverlayId) -> bool {
        self.overlays.iter().any(|request| request.id() == overlay)
    }

    pub fn log_open(&self) -> bool {
        self.log_open
    }

    pub fn awaiting_discard(&self) -> Option<&TabRequest> {
        self.awaiting_discard.as_ref()
    }

    pub fn has_unsaved_changes(&self, cx: &App) -> bool {
        self.config.read(cx).is_dirty() || self.settings.read(cx).is_dirty()
    }

    pub fn close_guard(&self, cx: &App) -> CloseGuard {
        let operations = self.busy.read(cx).count();
        if operations > 0 {
            CloseGuard::Busy { operations }
        } else if self.has_unsaved_changes(cx) {
            CloseGuard::UnsavedChanges
        } else {
            CloseGuard::Clear
        }
    }

    pub fn request_tab(
        &mut self,
        tab: TabId,
        select: Option<String>,
        cx: &mut Context<Self>,
    ) -> TabRequestOutcome {
        if let Some(overlay) = tab.overlay() {
            self.open_overlay(OverlayRequest::Docs { page: select }, cx);
            return TabRequestOutcome::OverlayOpened(overlay);
        }
        if self.active() == tab {
            if let Some(key) = select {
                self.flags.request_tab_selection(tab.index(), key);
                cx.emit(NavigationEvent::SelectionRequested(tab));
                cx.notify();
            }
            return TabRequestOutcome::AlreadyActive;
        }
        if self.busy.read(cx).textures() {
            let block = NavigationBlock::TexturesBusy;
            cx.emit(NavigationEvent::Blocked(block));
            return TabRequestOutcome::Blocked(block);
        }
        let request = TabRequest { tab, select };
        if self.has_unsaved_changes(cx) {
            self.awaiting_discard = Some(request.clone());
            cx.emit(NavigationEvent::DiscardRequested(request));
            cx.notify();
            return TabRequestOutcome::ConfirmDiscard;
        }
        self.switch(request, cx);
        TabRequestOutcome::Switched
    }

    pub fn confirm_discard(&mut self, cx: &mut Context<Self>) {
        let Some(request) = self.awaiting_discard.take() else {
            return;
        };
        self.discard_changes(cx);
        self.switch(request, cx);
    }

    pub fn cancel_discard(&mut self, cx: &mut Context<Self>) {
        if self.awaiting_discard.take().is_some() {
            cx.notify();
        }
    }

    pub fn discard_changes(&mut self, cx: &mut Context<Self>) {
        self.config.update(cx, |config, cx| config.discard(cx));
        self.settings
            .update(cx, |settings, cx| settings.discard(cx));
    }

    fn switch(&mut self, request: TabRequest, cx: &mut Context<Self>) {
        let from = self.active();
        self.awaiting_discard = None;
        match request.select {
            Some(key) => self.flags.request_tab_selection(request.tab.index(), key),
            None => self.flags.active_tab = request.tab.index(),
        }
        cx.emit(NavigationEvent::TabChanged {
            from,
            to: request.tab,
        });
        cx.notify();
    }

    pub fn take_selection(&mut self, tab: TabId) -> Option<String> {
        self.flags.take_tab_selection(tab.index())
    }

    pub fn open_overlay(&mut self, request: OverlayRequest, cx: &mut Context<Self>) {
        self.overlays.retain(|open| open.id() != request.id());
        self.overlays.push(request.clone());
        cx.emit(NavigationEvent::OverlayOpened(request));
        cx.notify();
    }

    pub fn close_overlay(&mut self, overlay: OverlayId, cx: &mut Context<Self>) {
        let before = self.overlays.len();
        self.overlays.retain(|open| open.id() != overlay);
        if self.overlays.len() != before {
            cx.emit(NavigationEvent::OverlayClosed(overlay));
            cx.notify();
        }
    }

    pub fn close_top_overlay(&mut self, cx: &mut Context<Self>) {
        if let Some(request) = self.overlays.pop() {
            cx.emit(NavigationEvent::OverlayClosed(request.id()));
            cx.notify();
        }
    }

    pub fn set_log_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.log_open != open {
            self.log_open = open;
            cx.emit(NavigationEvent::LogsToggled(open));
            cx.notify();
        }
    }

    pub fn open_logs(&mut self, cx: &mut Context<Self>) {
        self.set_log_open(true, cx);
    }

    pub fn close_logs(&mut self, cx: &mut Context<Self>) {
        self.set_log_open(false, cx);
    }

    pub fn toggle_logs(&mut self, cx: &mut Context<Self>) {
        self.set_log_open(!self.log_open, cx);
    }
}
