use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{App, AppContext as _};
use yp_state::prelude::*;
use yp_state::{Notice, NoticeKind};
use yp_ui::{Msg, Severity};

static NEXT_NOTICE: AtomicU64 = AtomicU64::new(0);

fn push(cx: &mut App, topic: &str, msg: Msg, kind: NoticeKind, severity: Severity) {
    let id = format!("{topic}_{}", NEXT_NOTICE.fetch_add(1, Ordering::Relaxed));
    let notifications = cx.stores().notifications.clone();
    notifications.update(cx, |store, cx| {
        store.push(Notice::new(id, msg, kind, severity), cx);
    });
}

pub(crate) fn toast(cx: &mut App, topic: &str, msg: impl Into<Msg>, severity: Severity) {
    push(cx, topic, msg.into(), NoticeKind::General, severity);
}

pub(crate) fn brief_toast(cx: &mut App, topic: &str, msg: impl Into<Msg>, severity: Severity) {
    push(cx, topic, msg.into(), NoticeKind::Shortcut, severity);
}

pub(crate) fn reveal(path: PathBuf, is_file: bool, cx: &App) {
    if yp_ui::ControllerMode::is_active(cx) {
        return;
    }
    let platform = cx.env().platform.clone();
    cx.background_spawn(async move {
        if let Err(error) = platform.reveal(&path, is_file) {
            log::warn!("could not reveal {}: {error}", path.display());
        }
    })
    .detach();
}
