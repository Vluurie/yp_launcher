use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn check(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Progress {
    pub fraction: Option<f32>,
    pub current: Option<String>,
}

impl Progress {
    pub fn fraction(fraction: f32) -> Self {
        Self {
            fraction: Some(fraction.clamp(0.0, 1.0)),
            current: None,
        }
    }

    pub fn step(done: usize, total: usize, current: impl Into<String>) -> Self {
        let fraction = if total == 0 {
            None
        } else {
            Some((done as f32 / total as f32).clamp(0.0, 1.0))
        };
        Self {
            fraction,
            current: Some(current.into()),
        }
    }

    pub fn message(current: impl Into<String>) -> Self {
        Self {
            fraction: None,
            current: Some(current.into()),
        }
    }
}

pub trait ProgressSink: Send + Sync {
    fn report(&self, progress: Progress);
}

impl<F: Fn(Progress) + Send + Sync> ProgressSink for F {
    fn report(&self, progress: Progress) {
        self(progress)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoProgress;

impl ProgressSink for NoProgress {
    fn report(&self, _progress: Progress) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn cancel_is_shared_between_clones() {
        let token = CancelToken::new();
        let clone = token.clone();
        assert!(token.check().is_ok());
        clone.cancel();
        assert!(token.is_cancelled());
        assert!(matches!(token.check(), Err(Error::Cancelled)));
    }

    #[test]
    fn closures_are_progress_sinks() {
        let seen = Mutex::new(Vec::new());
        let sink = |p: Progress| seen.lock().unwrap().push(p);
        sink.report(Progress::step(1, 4, "a"));
        assert_eq!(seen.lock().unwrap()[0].fraction, Some(0.25));
        NoProgress.report(Progress::fraction(2.0));
    }
}
