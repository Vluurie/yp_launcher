mod layer;

pub(crate) use layer::layer;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::{
    AnyView, App, Context, CursorStyle, DismissEvent, Entity, EntityId, FocusHandle, Focusable,
    IntoElement, ManagedView, MouseButton, Pixels, Render, ResizeEdge, SharedString, Subscription,
    Task, Window, div, prelude::*, px, relative,
};

use super::modal::ModalView;
use super::title_bar::window_frame_is_ours;
use super::toast::{Toast, ToastId, ToastStack};
use super::util::{ControllerMode, FocusVisible, KeyClickGuard};
use crate::actions::{
    Cancel, FocusNext, FocusPrevious, MODAL_CONTEXT, ROOT_CONTEXT, SHEET_CONTEXT,
};
use crate::styled_ext::StyledExt;
use crate::theme::{ActiveTheme, ResponsiveExt};

type DismissGuard = Rc<dyn Fn(&mut Window, &mut App) -> bool>;

struct ActiveModal {
    id: EntityId,
    view: AnyView,
    focus_handle: FocusHandle,
    previous_focus: Option<FocusHandle>,
    before_dismiss: DismissGuard,
    dismiss_on_scrim_click: bool,
    scrim_opacity: Option<f32>,
    _subscription: Subscription,
}

struct ClosingModal {
    view: AnyView,
    began: Instant,
    shade: f32,
}

struct ActiveSheet {
    id: EntityId,
    view: AnyView,
    focus_handle: FocusHandle,
    previous_focus: Option<FocusHandle>,
    closing: bool,
    close_task: Option<Task<()>>,
    _subscription: Subscription,
}

struct ActiveOverlay {
    key: SharedString,
    transparent: bool,
    view: AnyView,
    focus_handle: FocusHandle,
    previous_focus: Option<FocusHandle>,
}

pub struct Root {
    menu_page_start: std::cell::Cell<Option<Instant>>,
    title_bar: Option<AnyView>,
    content: AnyView,
    overlays: Vec<ActiveOverlay>,
    sheet: Option<ActiveSheet>,
    modals: Vec<ActiveModal>,
    closing_modal: Option<ClosingModal>,
    toasts: Entity<ToastStack>,
    focus_handle: FocusHandle,
}

impl Root {
    pub fn menu_page_start(&self) -> Option<Instant> {
        self.menu_page_start.get()
    }

    pub fn set_menu_page_start(&self, time: Instant) {
        self.menu_page_start.set(Some(time));
    }

    const FOCUS_TRAP_STEPS: usize = 256;

    pub fn new(content: impl Into<AnyView>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            menu_page_start: Default::default(),
            title_bar: None,
            content: content.into(),
            overlays: Vec::new(),
            sheet: None,
            modals: Vec::new(),
            closing_modal: None,
            toasts: cx.new(|_| ToastStack::new()),
            focus_handle,
        }
    }

    pub fn set_content(&mut self, content: impl Into<AnyView>, cx: &mut Context<Self>) {
        self.content = content.into();
        cx.notify();
    }

    pub fn set_title_bar(&mut self, title_bar: Option<AnyView>, cx: &mut Context<Self>) {
        self.title_bar = title_bar;
        cx.notify();
    }

    pub fn toasts(&self) -> &Entity<ToastStack> {
        &self.toasts
    }

    pub fn has_modal(&self) -> bool {
        !self.modals.is_empty()
    }

    pub fn modal_count(&self) -> usize {
        self.modals.len()
    }

    pub fn has_sheet(&self) -> bool {
        self.sheet.as_ref().is_some_and(|sheet| !sheet.closing)
    }

    pub fn is_sheet_closing(&self) -> bool {
        self.sheet.as_ref().is_some_and(|sheet| sheet.closing)
    }

    pub fn has_overlay(&self, key: &str) -> bool {
        self.overlays
            .iter()
            .any(|overlay| overlay.key.as_ref() == key)
    }

    pub fn overlay_keys(&self) -> Vec<SharedString> {
        self.overlays
            .iter()
            .map(|overlay| overlay.key.clone())
            .collect()
    }

    fn push_modal<V: ModalView>(
        &mut self,
        modal: Entity<V>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = modal.entity_id();
        let subscription = cx.subscribe_in(
            &modal,
            window,
            move |this, _, _: &DismissEvent, window, cx| this.remove_modal(id, window, cx),
        );
        let focus_handle = modal.focus_handle(cx);
        let previous_focus = window.focused(cx);
        window.focus(&focus_handle, cx);
        let guard_modal = modal.downgrade();
        let before_dismiss: DismissGuard = Rc::new(move |window, cx| {
            guard_modal
                .update(cx, |modal, cx| modal.on_before_dismiss(window, cx))
                .unwrap_or(true)
        });
        let dismiss_on_scrim_click = modal.read(cx).dismiss_on_scrim_click();
        let scrim_opacity = modal.read(cx).scrim_opacity(cx);
        self.closing_modal = None;
        self.modals.push(ActiveModal {
            id,
            view: modal.into(),
            focus_handle,
            previous_focus,
            before_dismiss,
            dismiss_on_scrim_click,
            scrim_opacity,
            _subscription: subscription,
        });
        cx.notify();
    }

    fn remove_modal(&mut self, id: EntityId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.modals.iter().position(|modal| modal.id == id) else {
            return;
        };
        let modal = self.modals.remove(index);
        let was_top = index == self.modals.len();
        if was_top
            && cx.theme().id == crate::ThemeId::NierNew
            && !yp_nier_ui::ReducedMotion::enabled(cx)
        {
            self.closing_modal = Some(ClosingModal {
                view: modal.view,
                began: Instant::now(),
                shade: modal.scrim_opacity.unwrap_or(0.22),
            });
        }
        if was_top {
            match self.modals.last() {
                Some(below) => window.focus(&below.focus_handle, cx),
                None => match modal.previous_focus {
                    Some(previous) => window.focus(&previous, cx),
                    None => window.focus(&self.focus_handle, cx),
                },
            }
        }
        cx.notify();
    }

    fn move_focus(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let scope = self
            .modals
            .last()
            .map(|modal| modal.focus_handle.clone())
            .or_else(|| {
                self.sheet
                    .as_ref()
                    .filter(|sheet| !sheet.closing)
                    .map(|sheet| sheet.focus_handle.clone())
            })
            .or_else(|| self.overlays.last().map(|overlay| overlay.focus_handle.clone()));
        let step = |window: &mut Window, cx: &mut App| {
            if forward {
                window.focus_next(cx);
            } else {
                window.focus_prev(cx);
            }
        };
        step(window, cx);
        let Some(scope) = scope else {
            return;
        };
        for _ in 0..Self::FOCUS_TRAP_STEPS {
            if scope.contains_focused(window, cx) {
                return;
            }
            step(window, cx);
        }
        if !scope.contains_focused(window, cx) {
            window.focus(&scope, cx);
        }
    }

    fn top_modal_guard(&self) -> Option<(EntityId, DismissGuard, bool)> {
        self.modals.last().map(|modal| {
            (
                modal.id,
                modal.before_dismiss.clone(),
                modal.dismiss_on_scrim_click,
            )
        })
    }

    fn set_sheet<V: ManagedView>(
        &mut self,
        sheet: Entity<V>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = sheet.entity_id();
        let subscription = cx.subscribe_in(
            &sheet,
            window,
            move |this, _, _: &DismissEvent, window, cx| this.remove_sheet(Some(id), window, cx),
        );
        let focus_handle = sheet.focus_handle(cx);
        let previous_focus = match self.sheet.take() {
            Some(previous) if !previous.closing => previous.previous_focus,
            _ => window.focused(cx),
        };
        window.focus(&focus_handle, cx);
        self.sheet = Some(ActiveSheet {
            id,
            view: sheet.into(),
            focus_handle,
            previous_focus,
            closing: false,
            close_task: None,
            _subscription: subscription,
        });
        cx.notify();
    }

    fn remove_sheet(&mut self, id: Option<EntityId>, window: &mut Window, cx: &mut Context<Self>) {
        let matches = match (&self.sheet, id) {
            (Some(sheet), Some(id)) => sheet.id == id,
            (Some(_), None) => true,
            (None, _) => false,
        };
        if !matches {
            return;
        }
        let duration = cx.theme().motion.cross;
        let restores_focus = self.modals.is_empty();
        let root_focus = self.focus_handle.clone();
        let Some(sheet) = self.sheet.as_mut().filter(|sheet| !sheet.closing) else {
            return;
        };
        if restores_focus {
            match sheet.previous_focus.take() {
                Some(previous) => window.focus(&previous, cx),
                None => window.focus(&root_focus, cx),
            }
        }
        let closing_id = sheet.id;
        sheet.closing = true;
        sheet.close_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(duration).await;
            this.update(cx, |this, cx| {
                let finished = this
                    .sheet
                    .as_ref()
                    .is_some_and(|sheet| sheet.closing && sheet.id == closing_id);
                if finished {
                    this.sheet = None;
                    cx.notify();
                }
            })
            .ok();
        }));
        cx.notify();
    }

    fn push_overlay(
        &mut self,
        key: SharedString,
        view: AnyView,
        focus: Option<FocusHandle>,
        transparent: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous_focus = window.focused(cx);
        let focus_handle = focus.unwrap_or_else(|| cx.focus_handle());
        self.overlays.retain(|overlay| overlay.key != key);
        self.overlays.push(ActiveOverlay {
            key,
            transparent,
            view,
            focus_handle: focus_handle.clone(),
            previous_focus,
        });
        window.focus(&focus_handle, cx);
        cx.notify();
    }

    fn remove_overlay(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self
            .overlays
            .iter()
            .position(|overlay| overlay.key.as_ref() == key)
        else {
            return;
        };
        let overlay = self.overlays.remove(index);
        if self.modals.is_empty()
            && let Some(previous) = overlay.previous_focus
        {
            window.focus(&previous, cx);
        }
        cx.notify();
    }
}

impl Focusable for Root {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

const RESIZE_EDGE: Pixels = px(5.);
const RESIZE_CORNER: Pixels = px(12.);

fn resize_grip(edge: ResizeEdge) -> gpui::Div {
    let (cursor, horizontal, vertical) = match edge {
        ResizeEdge::Top => (CursorStyle::ResizeUpDown, None, Some(true)),
        ResizeEdge::Bottom => (CursorStyle::ResizeUpDown, None, Some(false)),
        ResizeEdge::Left => (CursorStyle::ResizeLeftRight, Some(true), None),
        ResizeEdge::Right => (CursorStyle::ResizeLeftRight, Some(false), None),
        ResizeEdge::TopLeft => (CursorStyle::ResizeUpLeftDownRight, Some(true), Some(true)),
        ResizeEdge::BottomRight => (CursorStyle::ResizeUpLeftDownRight, Some(false), Some(false)),
        ResizeEdge::TopRight => (CursorStyle::ResizeUpRightDownLeft, Some(false), Some(true)),
        ResizeEdge::BottomLeft => (CursorStyle::ResizeUpRightDownLeft, Some(true), Some(false)),
    };
    let is_corner = horizontal.is_some() && vertical.is_some();
    let thickness = if is_corner { RESIZE_CORNER } else { RESIZE_EDGE };
    let grip = div().absolute().occlude().cursor(cursor);
    let grip = match horizontal {
        Some(true) => grip.left_0().w(thickness),
        Some(false) => grip.right_0().w(thickness),
        None => grip.left(RESIZE_CORNER).right(RESIZE_CORNER),
    };
    let grip = match vertical {
        Some(true) => grip.top_0().h(thickness),
        Some(false) => grip.bottom_0().h(thickness),
        None => grip.top(RESIZE_CORNER).bottom(RESIZE_CORNER),
    };
    grip.on_mouse_down(MouseButton::Left, move |_, window, cx| {
        cx.stop_propagation();
        window.start_window_resize(edge);
    })
}

fn resize_grips(window: &Window) -> Vec<gpui::Div> {
    if !window_frame_is_ours(window) || window.is_maximized() || window.is_fullscreen() {
        return Vec::new();
    }
    [
        ResizeEdge::Top,
        ResizeEdge::Bottom,
        ResizeEdge::Left,
        ResizeEdge::Right,
        ResizeEdge::TopLeft,
        ResizeEdge::TopRight,
        ResizeEdge::BottomLeft,
        ResizeEdge::BottomRight,
    ]
    .into_iter()
    .map(resize_grip)
    .collect()
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let scrim_color = theme.colors.scrim;
        let menu = theme.id == crate::ThemeId::NierNew;
        let default_scrim = if menu { 0.22 } else { theme.alpha.vignette };
        let fade_duration = Duration::from_millis(140);
        if self
            .closing_modal
            .as_ref()
            .is_some_and(|exit| exit.began.elapsed() >= fade_duration)
        {
            self.closing_modal = None;
        }
        if self.closing_modal.is_some() {
            window.request_animation_frame();
        }
        let modal_count = self.modals.len();
        if window.focused(cx).is_none() {
            window.focus(&self.focus_handle, cx);
        }
        let modal_open = modal_count > 0 || self.closing_modal.is_some();
        let covered = modal_open || self.sheet.is_some();
        let body = div()
            .relative()
            .flex_1()
            .min_h_0()
            .w_full()
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .child(layer("content-layer", covered || !self.overlays.is_empty(), div().absolute().inset_0().child(self.content.clone())))
            .children(self.overlays.iter().enumerate().map(|(i, overlay)| {
                layer(("overlay-layer", i), covered || i + 1 < self.overlays.len(), div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .when(!overlay.transparent, |el| el.bg(theme.colors.background_primary))
                    .track_focus(&overlay.focus_handle)
                    .child(overlay.view.clone()))
            }))
            .when_some(self.sheet.as_ref(), |this, sheet| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .key_context(SHEET_CONTEXT)
                        .track_focus(&sheet.focus_handle)
                        .on_action(|_: &Cancel, window, cx| {
                            window.defer(cx, |window, cx| window.close_sheet(cx));
                        })
                        .child(layer("sheet-layer", modal_open, sheet.view.clone())),
                )
            })
            .children(self.modals.iter().enumerate().map(|(index, modal)| {
                let is_top = index + 1 == modal_count;
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(is_top, |this| {
                        this.bg(scrim_color.opacity(modal.scrim_opacity.unwrap_or(default_scrim)))
                    })
                    .key_context(MODAL_CONTEXT)
                    .track_focus(&modal.focus_handle)
                    .on_action(|_: &Cancel, window, cx| {
                        window.defer(cx, |window, cx| {
                            window.close_modal(cx);
                        });
                    })
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.defer(cx, |window, cx| window.dismiss_modal_from_scrim(cx));
                    })
                    .child(
                        div()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(layer(("modal-layer", index), !is_top || self.closing_modal.is_some(), modal.view.clone())),
                    )
            }))
            .when_some(self.closing_modal.as_ref(), |el, exit| {
                let alpha = (1. - exit.began.elapsed().as_secs_f32() / fade_duration.as_secs_f32())
                    .clamp(0., 1.);
                el.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(scrim_color.opacity(exit.shade * alpha))
                        .child(layer("outgoing-modal", true, div().opacity(alpha).child(exit.view.clone())))
                        .child(div().absolute().inset_0().occlude()),
                )
            })
            .child(self.toasts.clone());
        div()
            .id("yp-root")
            .key_context(ROOT_CONTEXT)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .font_ui(cx)
            .text_size(sizes.font_md)
            .line_height(relative(1.3))
            .text_color(theme.colors.text_primary)
            .bg(theme.colors.background_primary)
            .capture_any_mouse_down(|_, _, cx| {
                ControllerMode::set_active(false, cx);
                FocusVisible::set(cx, false);
            })
            .on_mouse_move(|_, _, cx| {
                ControllerMode::set_active(false, cx);
            })
            .capture_key_up(|event, window, cx| {
                if !KeyClickGuard::allows(cx, &event.keystroke.key, window.focused(cx)) {
                    window.prevent_default();
                }
            })
            .on_action(cx.listener(|this, _: &FocusNext, window, cx| {
                this.move_focus(true, window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusPrevious, window, cx| {
                this.move_focus(false, window, cx);
            }))
            .children(
                self.title_bar
                    .clone()
                    .map(|title_bar| div().flex_none().w_full().child(title_bar)),
            )
            .child(body)
            .children(resize_grips(window))
            .when(menu, |el| el.child(yp_nier_ui::menu_screen_overlay()))
    }
}

pub trait WindowExt {
    fn yp_root(&self) -> Option<Entity<Root>>;

    fn open_modal<V: ModalView>(
        &mut self,
        cx: &mut App,
        build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
    ) -> Option<Entity<V>>;
    fn close_modal(&mut self, cx: &mut App) -> bool;
    fn close_all_modals(&mut self, cx: &mut App);
    fn dismiss_modal_from_scrim(&mut self, cx: &mut App);
    fn has_modal(&self, cx: &App) -> bool;

    fn push_toast(&mut self, toast: Toast, cx: &mut App) -> Option<ToastId>;
    fn dismiss_toast(&mut self, id: &ToastId, cx: &mut App);
    fn toast_stack(&self, cx: &App) -> Option<Entity<ToastStack>>;

    fn open_sheet<V: ManagedView>(
        &mut self,
        cx: &mut App,
        build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
    ) -> Option<Entity<V>>;
    fn close_sheet(&mut self, cx: &mut App);
    fn has_sheet(&self, cx: &App) -> bool;

    fn show_overlay(
        &mut self,
        key: impl Into<SharedString>,
        view: impl Into<AnyView>,
        cx: &mut App,
    );
    fn show_transparent_overlay(&mut self, key: impl Into<SharedString>, view: impl Into<AnyView>, cx: &mut App);
    fn show_focused_overlay<V: Focusable + Render>(
        &mut self,
        key: impl Into<SharedString>,
        view: Entity<V>,
        cx: &mut App,
    );
    fn hide_overlay(&mut self, key: &str, cx: &mut App);
    fn has_overlay(&self, key: &str, cx: &App) -> bool;
}

impl WindowExt for Window {
    fn yp_root(&self) -> Option<Entity<Root>> {
        self.root::<Root>().flatten()
    }

    fn open_modal<V: ModalView>(
        &mut self,
        cx: &mut App,
        build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
    ) -> Option<Entity<V>> {
        let root = self.yp_root()?;
        let modal = cx.new(|cx| build(self, cx));
        root.update(cx, |root, cx| root.push_modal(modal.clone(), self, cx));
        Some(modal)
    }

    fn close_modal(&mut self, cx: &mut App) -> bool {
        let Some(root) = self.yp_root() else {
            return false;
        };
        let Some((id, guard, _)) = root.read(cx).top_modal_guard() else {
            return false;
        };
        if !guard(self, cx) {
            return false;
        }
        root.update(cx, |root, cx| root.remove_modal(id, self, cx));
        true
    }

    fn close_all_modals(&mut self, cx: &mut App) {
        let Some(root) = self.yp_root() else {
            return;
        };
        let ids: Vec<EntityId> = root.read(cx).modals.iter().map(|modal| modal.id).collect();
        root.update(cx, |root, cx| {
            for id in ids.into_iter().rev() {
                root.remove_modal(id, self, cx);
            }
        });
    }

    fn dismiss_modal_from_scrim(&mut self, cx: &mut App) {
        let Some(root) = self.yp_root() else {
            return;
        };
        let allowed = root
            .read(cx)
            .top_modal_guard()
            .is_some_and(|(_, _, dismiss_on_scrim_click)| dismiss_on_scrim_click);
        if allowed {
            self.close_modal(cx);
        }
    }

    fn has_modal(&self, cx: &App) -> bool {
        self.yp_root().is_some_and(|root| root.read(cx).has_modal())
    }

    fn push_toast(&mut self, toast: Toast, cx: &mut App) -> Option<ToastId> {
        let stack = self.toast_stack(cx)?;
        Some(stack.update(cx, |stack, cx| stack.push(toast, cx)))
    }

    fn dismiss_toast(&mut self, id: &ToastId, cx: &mut App) {
        if let Some(stack) = self.toast_stack(cx) {
            stack.update(cx, |stack, cx| stack.dismiss(id, cx));
        }
    }

    fn toast_stack(&self, cx: &App) -> Option<Entity<ToastStack>> {
        self.yp_root().map(|root| root.read(cx).toasts.clone())
    }

    fn open_sheet<V: ManagedView>(
        &mut self,
        cx: &mut App,
        build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
    ) -> Option<Entity<V>> {
        let root = self.yp_root()?;
        let sheet = cx.new(|cx| build(self, cx));
        root.update(cx, |root, cx| root.set_sheet(sheet.clone(), self, cx));
        Some(sheet)
    }

    fn close_sheet(&mut self, cx: &mut App) {
        if let Some(root) = self.yp_root() {
            root.update(cx, |root, cx| root.remove_sheet(None, self, cx));
        }
    }

    fn has_sheet(&self, cx: &App) -> bool {
        self.yp_root().is_some_and(|root| root.read(cx).has_sheet())
    }

    fn show_overlay(
        &mut self,
        key: impl Into<SharedString>,
        view: impl Into<AnyView>,
        cx: &mut App,
    ) {
        if let Some(root) = self.yp_root() {
            let key = key.into();
            let view = view.into();
            root.update(cx, |root, cx| root.push_overlay(key, view, None, false, self, cx));
        }
    }

    fn show_transparent_overlay(&mut self, key: impl Into<SharedString>, view: impl Into<AnyView>, cx: &mut App) {
        if let Some(root) = self.yp_root() {
            root.update(cx, |root, cx| root.push_overlay(key.into(), view.into(), None, true, self, cx));
        }
    }

    fn show_focused_overlay<V: Focusable + Render>(
        &mut self,
        key: impl Into<SharedString>,
        view: Entity<V>,
        cx: &mut App,
    ) {
        if let Some(root) = self.yp_root() {
            let key = key.into();
            let focus = view.focus_handle(cx);
            let view: AnyView = view.into();
            root.update(cx, |root, cx| {
                root.push_overlay(key, view, Some(focus), false, self, cx)
            });
        }
    }

    fn hide_overlay(&mut self, key: &str, cx: &mut App) {
        if let Some(root) = self.yp_root() {
            root.update(cx, |root, cx| root.remove_overlay(key, self, cx));
        }
    }

    fn has_overlay(&self, key: &str, cx: &App) -> bool {
        self.yp_root()
            .is_some_and(|root| root.read(cx).has_overlay(key))
    }
}

#[cfg(test)]
mod tests {
    use gpui::{EventEmitter, TestAppContext, VisualTestContext};

    use super::*;
    use crate::components::modal::Modal;
    use crate::components::test_support::{has_modal, open_empty_root};

    struct TestModal {
        focus_handle: FocusHandle,
        allow_dismiss: bool,
    }

    impl EventEmitter<DismissEvent> for TestModal {}

    impl Focusable for TestModal {
        fn focus_handle(&self, _: &App) -> FocusHandle {
            self.focus_handle.clone()
        }
    }

    impl ModalView for TestModal {
        fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> bool {
            self.allow_dismiss
        }
    }

    impl Render for TestModal {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .track_focus(&self.focus_handle)
                .child(Modal::new("test-modal", None).section(div()))
        }
    }

    fn open(cx: &mut VisualTestContext, allow_dismiss: bool) -> Entity<TestModal> {
        let modal = cx.update(|window, cx| {
            window.open_modal(cx, |_, cx| TestModal {
                focus_handle: cx.focus_handle(),
                allow_dismiss,
            })
        });
        cx.run_until_parked();
        modal.expect("window has a Root")
    }

    #[gpui::test]
    fn escape_closes_the_top_modal_and_focus_walks_back(cx: &mut TestAppContext) {
        let (root, cx) = open_empty_root(cx);
        let root_focus = root.read_with(cx, |root, cx| root.focus_handle(cx));
        cx.update(|window, _| assert!(root_focus.is_focused(window)));
        let first = open(cx, true);
        let second = open(cx, true);
        root.read_with(cx, |root, _| assert_eq!(root.modal_count(), 2));
        cx.update(|window, cx| assert!(second.focus_handle(cx).is_focused(window)));
        cx.simulate_keystrokes("escape");
        root.read_with(cx, |root, _| assert_eq!(root.modal_count(), 1));
        cx.update(|window, cx| assert!(first.focus_handle(cx).is_focused(window)));
        cx.simulate_keystrokes("escape");
        assert!(!has_modal(cx));
        cx.update(|window, _| assert!(root_focus.is_focused(window)));
    }

    #[gpui::test]
    fn a_modal_can_veto_escape_but_not_its_own_dismiss_event(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let modal = open(cx, false);
        cx.simulate_keystrokes("escape");
        assert!(has_modal(cx));
        cx.update(|window, cx| {
            assert!(!window.close_modal(cx));
            window.dismiss_modal_from_scrim(cx);
        });
        assert!(has_modal(cx));
        modal.update(cx, |_, cx| cx.emit(DismissEvent));
        cx.run_until_parked();
        assert!(!has_modal(cx));
    }

    #[gpui::test]
    fn close_all_modals_ignores_guards(cx: &mut TestAppContext) {
        let (root, cx) = open_empty_root(cx);
        open(cx, false);
        open(cx, true);
        cx.update(|window, cx| window.close_all_modals(cx));
        root.read_with(cx, |root, _| assert_eq!(root.modal_count(), 0));
    }

    #[gpui::test]
    fn a_closed_sheet_stays_for_its_slide_out_and_is_removed_after_it(cx: &mut TestAppContext) {
        let (root, cx) = open_empty_root(cx);
        cx.update(|window, cx| {
            window.open_sheet(cx, |_, cx| TestModal {
                focus_handle: cx.focus_handle(),
                allow_dismiss: true,
            });
        });
        cx.run_until_parked();
        let duration = cx.update(|_, cx| cx.theme().motion.cross);

        cx.update(|window, cx| window.close_sheet(cx));
        cx.run_until_parked();
        cx.update(|window, cx| assert!(!window.has_sheet(cx)));
        root.read_with(cx, |root, _| assert!(root.is_sheet_closing()));

        cx.update(|window, cx| window.close_sheet(cx));
        root.read_with(cx, |root, _| assert!(root.is_sheet_closing()));

        cx.executor().advance_clock(duration * 2);
        cx.run_until_parked();
        root.read_with(cx, |root, _| assert!(!root.is_sheet_closing()));
        cx.update(|window, cx| assert!(!window.has_sheet(cx)));
    }

    #[gpui::test]
    fn reopening_during_the_slide_out_keeps_the_new_sheet(cx: &mut TestAppContext) {
        let (root, cx) = open_empty_root(cx);
        let open = |cx: &mut VisualTestContext| {
            cx.update(|window, cx| {
                window.open_sheet(cx, |_, cx| TestModal {
                    focus_handle: cx.focus_handle(),
                    allow_dismiss: true,
                });
            });
            cx.run_until_parked();
        };
        open(cx);
        let duration = cx.update(|_, cx| cx.theme().motion.cross);
        cx.update(|window, cx| window.close_sheet(cx));
        open(cx);
        cx.executor().advance_clock(duration * 2);
        cx.run_until_parked();
        cx.update(|window, cx| assert!(window.has_sheet(cx)));
        root.read_with(cx, |root, _| assert!(!root.is_sheet_closing()));
    }

    #[gpui::test]
    fn sheets_close_on_escape_and_overlays_are_keyed(cx: &mut TestAppContext) {
        let (root, cx) = open_empty_root(cx);
        cx.update(|window, cx| {
            window.open_sheet(cx, |_, cx| TestModal {
                focus_handle: cx.focus_handle(),
                allow_dismiss: true,
            });
        });
        cx.run_until_parked();
        cx.update(|window, cx| assert!(window.has_sheet(cx)));
        cx.simulate_keystrokes("escape");
        cx.update(|window, cx| assert!(!window.has_sheet(cx)));

        let view = cx.update(|_, cx| {
            cx.new(|cx| TestModal {
                focus_handle: cx.focus_handle(),
                allow_dismiss: true,
            })
        });
        cx.update(|window, cx| {
            window.show_overlay("docs", view.clone(), cx);
            window.show_focused_overlay("inspector", view.clone(), cx);
            window.show_overlay("docs", view.clone(), cx);
        });
        root.read_with(cx, |root, _| {
            let keys = root.overlay_keys();
            assert_eq!(keys.len(), 2);
            assert_eq!(keys[1].as_ref(), "docs");
        });
        cx.update(|window, cx| {
            assert!(window.has_overlay("inspector", cx));
            window.hide_overlay("inspector", cx);
            window.hide_overlay("missing", cx);
            assert!(!window.has_overlay("inspector", cx));
            assert!(window.has_overlay("docs", cx));
        });
    }

    #[gpui::test]
    fn toasts_reach_the_stack_through_the_window(cx: &mut TestAppContext) {
        let (root, cx) = open_empty_root(cx);
        let id = cx
            .update(|window, cx| window.push_toast(Toast::warning("careful"), cx))
            .expect("toast id");
        let stack = root.read_with(cx, |root, _| root.toasts().clone());
        stack.read_with(cx, |stack, _| assert!(stack.contains(&id)));
        cx.update(|window, cx| window.dismiss_toast(&id, cx));
        stack.read_with(cx, |stack, _| assert!(stack.is_leaving(&id)));
    }
}
