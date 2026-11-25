use gpui::{
    AnyElement, App, Bounds, Element, ElementId, Global, GlobalElementId, InspectorElementId,
    LayoutId, Pixels, Window,
};
use std::{cell::Cell, panic::Location, time::Instant};

#[derive(Clone, Copy, Default)]
pub(super) enum Playback {
    #[default]
    OnMount,
    Opening(Instant),
    Complete,
}

#[derive(Default)]
struct ActivePlayback(Cell<Playback>);
impl Global for ActivePlayback {}

impl Playback {
    pub(super) fn current(cx: &App) -> Self {
        cx.try_global::<ActivePlayback>()
            .map_or(Self::OnMount, |scope| scope.0.get())
    }
}

pub(crate) struct ListRowText {
    child: AnyElement,
    playback: Playback,
}

impl ListRowText {
    pub(crate) fn new(child: AnyElement, starts: Option<Instant>) -> Self {
        Self {
            child,
            playback: starts.map_or(Playback::Complete, Playback::Opening),
        }
    }
}

impl Element for ListRowText {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        if !cx.has_global::<ActivePlayback>() {
            cx.set_global(ActivePlayback::default());
        }
        let previous = cx.global::<ActivePlayback>().0.replace(self.playback);
        let layout = self.child.request_layout(window, cx);
        cx.global::<ActivePlayback>().0.set(previous);
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}
