use gpui::AppContext as _;
use gpui::{Context, EventEmitter, Task};
use yp_core::{CancelToken, GamePaths};

use crate::cell::CancelOnDrop;
use crate::health;

pub use crate::health::{DetectionSnapshot, TextureChip};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionEvent {
    Invalidated { generation: u64 },
    Updated { generation: u64 },
}

pub struct DetectionStore {
    game: Option<GamePaths>,
    generation: u64,
    snapshot: Option<DetectionSnapshot>,
    checking: bool,
    probe_task: Option<Task<()>>,
}

impl EventEmitter<DetectionEvent> for DetectionStore {}

impl DetectionStore {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            game: None,
            generation: 0,
            snapshot: None,
            checking: false,
            probe_task: None,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn snapshot(&self) -> Option<&DetectionSnapshot> {
        self.snapshot.as_ref()
    }

    pub fn is_checking(&self) -> bool {
        self.checking
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        if self.game != game {
            self.game = game;
            self.snapshot = None;
        }
        self.invalidate(cx);
    }

    pub fn invalidate(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        cx.emit(DetectionEvent::Invalidated { generation });
        let Some(game) = self.game.clone() else {
            self.probe_task = None;
            self.checking = false;
            self.snapshot = None;
            cx.notify();
            return;
        };
        self.checking = true;
        cx.notify();
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let job = cx.background_spawn(async move { health::probe(&game, &cancel) });
        self.probe_task = Some(cx.spawn(async move |this, cx| {
            let _guard = guard;
            let snapshot = job.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.snapshot = Some(snapshot);
                this.checking = false;
                cx.emit(DetectionEvent::Updated { generation });
                cx.notify();
            })
            .ok();
        }));
    }
}
