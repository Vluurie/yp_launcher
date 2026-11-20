use futures::StreamExt;
use futures::channel::mpsc;
use gpui::{Context, EventEmitter, Task};
use yp_host::session::{BusyCounter, BusyGuard};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BusyScope {
    #[default]
    General,
    Textures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusyEvent {
    Changed { count: usize, textures: bool },
}

pub struct BusyHandle {
    general: Option<BusyGuard>,
    textures: Option<BusyGuard>,
    changed: mpsc::UnboundedSender<()>,
}

impl Drop for BusyHandle {
    fn drop(&mut self) {
        self.general.take();
        self.textures.take();
        let _ = self.changed.unbounded_send(());
    }
}

pub struct BusyTracker {
    general: BusyCounter,
    textures: BusyCounter,
    changed: mpsc::UnboundedSender<()>,
    _listener: Task<()>,
}

impl EventEmitter<BusyEvent> for BusyTracker {}

impl BusyTracker {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (changed, mut changes) = mpsc::unbounded::<()>();
        let listener = cx.spawn(async move |this, cx| {
            while changes.next().await.is_some() {
                if this.update(cx, |this, cx| this.announce(cx)).is_err() {
                    break;
                }
            }
        });
        Self {
            general: BusyCounter::new(),
            textures: BusyCounter::new(),
            changed,
            _listener: listener,
        }
    }

    pub fn count(&self) -> usize {
        self.general.count()
    }

    pub fn is_busy(&self) -> bool {
        self.general.is_busy()
    }

    pub fn textures(&self) -> bool {
        self.textures.is_busy()
    }

    fn announce(&mut self, cx: &mut Context<Self>) {
        cx.emit(BusyEvent::Changed {
            count: self.count(),
            textures: self.textures(),
        });
        cx.notify();
    }

    pub fn enter(&mut self, scope: BusyScope, cx: &mut Context<Self>) -> BusyHandle {
        let handle = BusyHandle {
            general: Some(self.general.enter()),
            textures: (scope == BusyScope::Textures).then(|| self.textures.enter()),
            changed: self.changed.clone(),
        };
        self.announce(cx);
        handle
    }

    pub fn track<R: 'static>(
        &mut self,
        scope: BusyScope,
        future: impl Future<Output = R> + 'static,
        cx: &mut Context<Self>,
    ) -> Task<R> {
        let handle = self.enter(scope, cx);
        cx.foreground_executor().spawn(async move {
            let _handle = handle;
            future.await
        })
    }
}
