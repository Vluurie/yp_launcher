use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc;
use gpui::AppContext as _;
use gpui::{BackgroundExecutor, Context, Task};
use yp_core::watch::{DirWatcher, Recursive};

pub(crate) const COALESCE_WINDOW: Duration = Duration::from_millis(50);
pub(crate) const RELOAD_DEBOUNCE: Duration = Duration::from_millis(400);

struct WatchHandle {
    watcher: Option<DirWatcher>,
    executor: BackgroundExecutor,
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        if let Some(watcher) = self.watcher.take() {
            self.executor.spawn(async move { drop(watcher) }).detach();
        }
    }
}

pub(crate) struct Watch {
    _task: Task<()>,
}

impl Watch {
    pub fn start<T: 'static>(
        targets: Vec<(PathBuf, Recursive)>,
        cx: &mut Context<T>,
        on_change: impl Fn(&mut T, Vec<PathBuf>, &mut Context<T>) + 'static,
    ) -> Self {
        let (changes, mut incoming) = mpsc::unbounded::<Vec<PathBuf>>();
        let executor = cx.background_executor().clone();
        let create = cx.background_spawn(async move {
            let watcher = DirWatcher::new(&targets, COALESCE_WINDOW, move |paths| {
                let _ = changes.unbounded_send(paths);
            });
            watcher.map(|watcher| WatchHandle {
                watcher: Some(watcher),
                executor,
            })
        });
        let task = cx.spawn(async move |this, cx| {
            let _handle = match create.await {
                Ok(handle) => handle,
                Err(err) => {
                    log::warn!("file watching is unavailable: {err}");
                    return;
                }
            };
            while let Some(paths) = incoming.next().await {
                if this
                    .update(cx, |this, cx| on_change(this, paths, cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        Self { _task: task }
    }
}

pub(crate) fn touches(paths: &[PathBuf], file: &Path) -> bool {
    let Some(name) = file.file_name() else {
        return false;
    };
    paths.iter().any(|path| {
        path.file_name()
            .is_some_and(|changed| changed.eq_ignore_ascii_case(name))
    })
}
