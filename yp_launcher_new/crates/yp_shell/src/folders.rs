use std::path::PathBuf;

use gpui::{App, AppContext as _};
use yp_state::prelude::*;

use crate::ShellOptions;

pub fn open_link(url: &str, cx: &App) {
    if ShellOptions::is_sandboxed(cx) {
        log::info!("sandbox: link not opened: {url}");
        return;
    }
    cx.open_url(url);
}

pub fn open_dir(dir: impl Into<PathBuf>, cx: &App) {
    if yp_ui::ControllerMode::is_active(cx) {
        return;
    }
    let dir: PathBuf = dir.into();
    if dir.as_os_str().is_empty() {
        return;
    }
    let platform = cx.env().platform.clone();
    cx.background_spawn(async move {
        if let Err(err) = std::fs::create_dir_all(&dir) {
            log::warn!("could not create {}: {err}", dir.display());
            return;
        }
        if let Err(err) = platform.reveal(&dir, false) {
            log::warn!("could not open {}: {err}", dir.display());
        }
    })
    .detach();
}
