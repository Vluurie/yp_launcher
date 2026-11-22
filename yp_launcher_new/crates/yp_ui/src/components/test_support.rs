use gpui::{
    AnyElement, App, Context, Entity, IntoElement, Render, TestAppContext, VisualTestContext,
    Window, div, prelude::*,
};

use super::root::Root;

type HostRender = Box<dyn Fn(&mut Window, &mut Context<Host>) -> AnyElement>;

pub(crate) struct Host {
    render: HostRender,
}

impl Render for Host {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child((self.render)(window, cx))
    }
}

pub(crate) fn open_root(
    cx: &mut TestAppContext,
    render: impl Fn(&mut Window, &mut Context<Host>) -> AnyElement + 'static,
) -> (Entity<Root>, &mut VisualTestContext) {
    cx.update(crate::init_for_tests);
    cx.add_window_view(move |window, cx| {
        let host = cx.new(|_| Host {
            render: Box::new(render),
        });
        Root::new(host, window, cx)
    })
}

pub(crate) fn open_empty_root(cx: &mut TestAppContext) -> (Entity<Root>, &mut VisualTestContext) {
    open_root(cx, |_, _| div().into_any_element())
}

pub(crate) fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

pub(crate) fn has_modal(cx: &mut VisualTestContext) -> bool {
    use super::root::WindowExt;
    cx.update(|window, cx: &mut App| window.has_modal(cx))
}
