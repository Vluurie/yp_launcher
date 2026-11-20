use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use notify::event::{AccessKind, AccessMode};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recursive(pub bool);

impl Recursive {
    pub const YES: Recursive = Recursive(true);
    pub const NO: Recursive = Recursive(false);

    fn mode(self) -> RecursiveMode {
        if self.0 {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        }
    }
}

enum Signal {
    Changed(Vec<PathBuf>),
    Stop,
}

pub struct DirWatcher {
    watcher: Option<RecommendedWatcher>,
    signals: Sender<Signal>,
    worker: Option<JoinHandle<()>>,
    watched: Vec<PathBuf>,
}

impl DirWatcher {
    pub fn new(
        targets: &[(PathBuf, Recursive)],
        debounce: Duration,
        on_change: impl Fn(Vec<PathBuf>) + Send + 'static,
    ) -> Result<Self> {
        let (signals, receiver) = mpsc::channel::<Signal>();
        let forward = signals.clone();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            if let Ok(event) = event
                && is_relevant(&event.kind)
            {
                let _ = forward.send(Signal::Changed(event.paths));
            }
        })
        .map_err(watch_error)?;
        let mut watched = Vec::new();
        for (path, recursive) in targets {
            if !path.exists() {
                log::debug!("not watching missing {}", path.display());
                continue;
            }
            watcher.watch(path, recursive.mode()).map_err(watch_error)?;
            watched.push(path.clone());
        }
        let worker = std::thread::Builder::new()
            .name("yp-dir-watch".to_owned())
            .spawn(move || debounce_loop(&receiver, debounce, &on_change))
            .map_err(|err| Error::invalid("directory watch", err.to_string()))?;
        Ok(Self {
            watcher: Some(watcher),
            signals,
            worker: Some(worker),
            watched,
        })
    }

    pub fn watched(&self) -> &[PathBuf] {
        &self.watched
    }
}

impl Drop for DirWatcher {
    fn drop(&mut self) {
        drop(self.watcher.take());
        let _ = self.signals.send(Signal::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn watch_error(err: notify::Error) -> Error {
    Error::invalid("directory watch", err.to_string())
}

fn is_relevant(kind: &EventKind) -> bool {
    match kind {
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => true,
        EventKind::Access(_) => false,
        EventKind::Any | EventKind::Other => true,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => true,
    }
}

fn debounce_loop(receiver: &Receiver<Signal>, debounce: Duration, on_change: &dyn Fn(Vec<PathBuf>)) {
    loop {
        let mut pending = BTreeSet::new();
        match receiver.recv() {
            Ok(Signal::Changed(paths)) => pending.extend(paths),
            Ok(Signal::Stop) | Err(_) => return,
        }
        loop {
            match receiver.recv_timeout(debounce) {
                Ok(Signal::Changed(paths)) => pending.extend(paths),
                Ok(Signal::Stop) | Err(RecvTimeoutError::Disconnected) => return,
                Err(RecvTimeoutError::Timeout) => break,
            }
        }
        on_change(pending.into_iter().collect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    #[test]
    fn coalesces_bursts_into_one_callback() {
        let dir = tempfile::tempdir().unwrap();
        let calls: Arc<Mutex<Vec<Vec<PathBuf>>>> = Arc::default();
        let seen = calls.clone();
        let watcher = DirWatcher::new(
            &[(dir.path().to_path_buf(), Recursive::YES), (dir.path().join("missing"), Recursive::NO)],
            Duration::from_millis(150),
            move |paths| seen.lock().unwrap().push(paths),
        )
        .unwrap();
        assert_eq!(watcher.watched(), &[dir.path().to_path_buf()]);
        for index in 0..5 {
            std::fs::write(dir.path().join(format!("f{index}.toml")), "a = 1").unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        while calls.lock().unwrap().is_empty() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        std::thread::sleep(Duration::from_millis(400));
        drop(watcher);
        let calls = calls.lock().unwrap();
        assert!(!calls.is_empty());
        assert!(calls.len() <= 2);
        assert!(calls.iter().flatten().any(|path| path.ends_with("f4.toml")));
    }
}
