use std::path::PathBuf;

use gpui::{Action, App};
use yp_inspector::ArchivePaths;

pub use yp_ui::{SetLocale, ToggleTheme};

use crate::StateExt;
use crate::registry::OverlayRequest;
use crate::tabs::TabId;

#[derive(Debug, Clone, PartialEq, Eq, Action)]
#[action(namespace = yp, no_json)]
pub struct OpenTab {
    pub tab: TabId,
    pub select: Option<String>,
}

impl OpenTab {
    pub fn new(tab: TabId) -> Self {
        Self { tab, select: None }
    }

    pub fn selecting(tab: TabId, key: impl Into<String>) -> Self {
        Self {
            tab,
            select: Some(key.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct OpenDocs {
    pub page: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct OpenThreeDInspector {
    pub data: Option<PathBuf>,
    pub dtt: Option<PathBuf>,
}

impl OpenThreeDInspector {
    pub fn archives(&self) -> ArchivePaths {
        ArchivePaths::new(self.data.clone(), self.dtt.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct OpenLogs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct CloseLogs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct OpenDiagnostics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct Play;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct Stop;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Action)]
#[action(namespace = yp, no_json)]
pub struct RefreshDetections;

pub fn register_default_handlers(cx: &mut App) {
    cx.on_action(|_: &Play, cx| {
        let stores = cx.stores().clone();
        let request = stores.launch_options.read(cx).request();
        stores
            .session
            .update(cx, |session, cx| session.launch(request, cx));
    });
    cx.on_action(|_: &RefreshDetections, cx| {
        let stores = cx.stores().clone();
        match stores.session.read(cx).game() {
            Some(game) => stores
                .notifications
                .update(cx, |store, cx| store.refresh_detections(&game, cx)),
            None => stores
                .detection
                .update(cx, |store, cx| store.invalidate(cx)),
        }
    });
    cx.on_action(|_: &OpenLogs, cx| {
        let navigation = cx.stores().navigation.clone();
        navigation.update(cx, |navigation, cx| navigation.open_logs(cx));
    });
    cx.on_action(|_: &CloseLogs, cx| {
        let navigation = cx.stores().navigation.clone();
        navigation.update(cx, |navigation, cx| navigation.close_logs(cx));
    });
    cx.on_action(|action: &OpenTab, cx| {
        let navigation = cx.stores().navigation.clone();
        let (tab, select) = (action.tab, action.select.clone());
        navigation.update(cx, |navigation, cx| {
            navigation.request_tab(tab, select, cx);
        });
    });
    cx.on_action(|action: &OpenDocs, cx| {
        let navigation = cx.stores().navigation.clone();
        let request = OverlayRequest::Docs {
            page: action.page.clone(),
        };
        navigation.update(cx, |navigation, cx| navigation.open_overlay(request, cx));
    });
    cx.on_action(|action: &OpenThreeDInspector, cx| {
        let navigation = cx.stores().navigation.clone();
        let request = OverlayRequest::Inspector {
            archives: action.archives(),
        };
        navigation.update(cx, |navigation, cx| navigation.open_overlay(request, cx));
    });
}
