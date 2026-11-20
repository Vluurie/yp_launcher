use std::sync::atomic::{AtomicU64, Ordering};

use gpui::App;
use yp_i18n::Msg;
use yp_state::prelude::*;
use yp_state::{Notice, NoticeKind, NoticeSeverity};

static NEXT_NOTICE: AtomicU64 = AtomicU64::new(1);

pub fn unique_id(prefix: &str) -> String {
    let serial = NEXT_NOTICE.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}_{serial}")
}

pub fn push(
    prefix: &str,
    msg: impl Into<Msg>,
    kind: NoticeKind,
    severity: NoticeSeverity,
    cx: &mut App,
) {
    let notice = Notice::new(unique_id(prefix), msg, kind, severity);
    let notifications = cx.stores().notifications.clone();
    notifications.update(cx, |store, cx| store.push(notice, cx));
}

pub fn success(prefix: &str, msg: impl Into<Msg>, cx: &mut App) {
    push(prefix, msg, NoticeKind::General, NoticeSeverity::Success, cx);
}

pub fn error(prefix: &str, msg: impl Into<Msg>, cx: &mut App) {
    push(prefix, msg, NoticeKind::General, NoticeSeverity::Error, cx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_keep_their_prefix() {
        let first = unique_id("shortcut");
        let second = unique_id("shortcut");
        assert_ne!(first, second);
        assert!(first.starts_with("shortcut_"));
    }
}
