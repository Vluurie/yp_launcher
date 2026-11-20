use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError};

use futures::StreamExt;
use futures::channel::{mpsc, oneshot};
use gpui::BackgroundExecutor;

type Job = Box<dyn FnOnce() + Send>;
type Queued<S> = Box<dyn FnOnce(&mut S) + Send>;

struct Inner<S> {
    state: Mutex<S>,
    queued: Mutex<Vec<Queued<S>>>,
}

pub(crate) struct Guarded<S> {
    inner: Arc<Inner<S>>,
    jobs: mpsc::UnboundedSender<Job>,
}

impl<S> Clone for Guarded<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            jobs: self.jobs.clone(),
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn drain<S>(inner: &Inner<S>, state: &mut S) {
    let queued = std::mem::take(&mut *lock(&inner.queued));
    for op in queued {
        op(state);
    }
}

impl<S: Send + 'static> Guarded<S> {
    pub fn new(state: S, executor: &BackgroundExecutor) -> Self {
        let (jobs, mut pending) = mpsc::unbounded::<Job>();
        executor
            .spawn(async move {
                while let Some(job) = pending.next().await {
                    if catch_unwind(AssertUnwindSafe(job)).is_err() {
                        log::error!("a background store job panicked");
                    }
                }
            })
            .detach();
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(state),
                queued: Mutex::new(Vec::new()),
            }),
            jobs,
        }
    }

    fn try_state(&self) -> Option<MutexGuard<'_, S>> {
        match self.inner.state.try_lock() {
            Ok(state) => Some(state),
            Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => None,
        }
    }

    pub fn edit<R>(
        &self,
        op: impl FnOnce(&mut S) + Send + 'static,
        read: impl FnOnce(&S) -> R,
    ) -> Option<R> {
        match self.try_state() {
            Some(mut state) => {
                drain(&self.inner, &mut state);
                op(&mut state);
                Some(read(&state))
            }
            None => {
                lock(&self.inner.queued).push(Box::new(op));
                None
            }
        }
    }

    pub fn take<R>(&self, change: impl FnOnce(&mut S) -> R) -> Option<R> {
        let mut state = self.try_state()?;
        drain(&self.inner, &mut state);
        Some(change(&mut state))
    }

    pub fn settled(&self) -> oneshot::Receiver<()> {
        self.spawn(|_| ())
    }

    pub fn peek<R>(&self, read: impl FnOnce(&S) -> R) -> Option<R> {
        let mut state = self.try_state()?;
        drain(&self.inner, &mut state);
        Some(read(&state))
    }

    pub fn spawn<R: Send + 'static>(
        &self,
        work: impl FnOnce(&mut S) -> R + Send + 'static,
    ) -> oneshot::Receiver<R> {
        let (done, result) = oneshot::channel();
        let inner = self.inner.clone();
        let job: Job = Box::new(move || {
            let mut state = lock(&inner.state);
            drain(&inner, &mut state);
            let _ = done.send(work(&mut state));
        });
        let _ = self.jobs.unbounded_send(job);
        result
    }
}

pub(crate) struct CancelOnDrop(pub yp_core::CancelToken);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
