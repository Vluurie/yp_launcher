use gpui::AppContext as _;
use gpui::{Context, EventEmitter, SharedString, Task};
use yp_core::GamePaths;
use yp_i18n::Msg;

use crate::env::AppEnv;
use crate::health;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoticeKind {
    Migration,
    Reshade,
    Textures,
    Shortcut,
    General,
}

pub type NoticeSeverity = yp_ui::Severity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub id: SharedString,
    pub msg: Msg,
    pub kind: NoticeKind,
    pub severity: NoticeSeverity,
}

impl Notice {
    pub fn new(
        id: impl Into<SharedString>,
        msg: impl Into<Msg>,
        kind: NoticeKind,
        severity: NoticeSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            msg: msg.into(),
            kind,
            severity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationsEvent {
    Pushed(Notice),
    Dismissed(SharedString),
    DetectionsFinished,
}

pub struct Notifications {
    env: AppEnv,
    items: Vec<Notice>,
    detected: Vec<SharedString>,
    detecting: bool,
    detect_task: Option<Task<()>>,
}

impl EventEmitter<NotificationsEvent> for Notifications {}

impl Notifications {
    pub fn new(env: AppEnv, _cx: &mut Context<Self>) -> Self {
        Self {
            env,
            items: Vec::new(),
            detected: Vec::new(),
            detecting: false,
            detect_task: None,
        }
    }

    pub fn items(&self) -> &[Notice] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.items.iter().any(|notice| notice.id.as_ref() == id)
    }

    pub fn is_detecting(&self) -> bool {
        self.detecting
    }

    pub fn push(&mut self, notice: Notice, cx: &mut Context<Self>) {
        self.items.retain(|existing| existing.id != notice.id);
        self.items.push(notice.clone());
        cx.emit(NotificationsEvent::Pushed(notice));
        cx.notify();
    }

    pub fn dismiss(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(index) = self
            .items
            .iter()
            .position(|notice| notice.id.as_ref() == id)
        else {
            return;
        };
        let removed = self.items.remove(index);
        cx.emit(NotificationsEvent::Dismissed(removed.id));
        cx.notify();
    }

    pub fn check_platform_support(&mut self, cx: &mut Context<Self>) {
        let Some(notice) = health::platform_notice(&self.env) else {
            return;
        };
        if !self.contains(notice.id.as_ref()) {
            self.push(notice, cx);
        }
    }

    pub fn refresh_detections(&mut self, game: &GamePaths, cx: &mut Context<Self>) {
        self.detect(game.clone(), true, cx);
    }

    pub fn run_detections(&mut self, game: &GamePaths, cx: &mut Context<Self>) {
        self.detect(game.clone(), false, cx);
    }

    fn detect(&mut self, game: GamePaths, force: bool, cx: &mut Context<Self>) {
        self.detecting = true;
        cx.notify();
        let env = self.env.clone();
        let job = cx.background_spawn(async move { health::detect_notices(&env, &game, force) });
        self.detect_task = Some(cx.spawn(async move |this, cx| {
            let found = job.await;
            this.update(cx, |this, cx| {
                if let Some(found) = found {
                    let previous = std::mem::take(&mut this.detected);
                    this.items.retain(|notice| !previous.contains(&notice.id));
                    this.detected = found.iter().map(|notice| notice.id.clone()).collect();
                    for notice in found {
                        this.items.retain(|existing| existing.id != notice.id);
                        this.items.push(notice.clone());
                        cx.emit(NotificationsEvent::Pushed(notice));
                    }
                }
                this.detecting = false;
                cx.emit(NotificationsEvent::DetectionsFinished);
                cx.notify();
            })
            .ok();
        }));
    }
}
