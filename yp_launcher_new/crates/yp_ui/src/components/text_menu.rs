use std::rc::Rc;

use gpui::{
    App, Anchor, ElementId, Entity, Global, IntoElement, Pixels, Point, SharedString, WeakEntity,
    Window, anchored, deferred, div, prelude::*,
};

use super::menu::{ContextMenu, ContextMenuEntry, PopoverState, hide_popover, show_popover};
use crate::i18n::ActiveStrings;
use crate::theme::Sizes;

pub(crate) const KEY_CUT: &str = "x";
pub(crate) const KEY_COPY: &str = "c";
pub(crate) const KEY_PASTE: &str = "v";
pub(crate) const KEY_SELECT_ALL: &str = "a";

type MenuBuilder = Rc<dyn Fn(&mut Window, &mut App) -> Option<Entity<ContextMenu>>>;

#[derive(Default)]
struct ActiveTextMenu(Option<WeakEntity<PopoverState<ContextMenu>>>);

impl Global for ActiveTextMenu {}

pub(crate) fn text_menu_is_open(cx: &App) -> bool {
    cx.try_global::<ActiveTextMenu>()
        .and_then(|active| active.0.as_ref())
        .and_then(|state| state.upgrade())
        .is_some_and(|state| state.read(cx).menu.is_some())
}

pub(crate) fn shortcut(key: &str, cx: &App) -> SharedString {
    let strings = cx.t();
    let modifier = if cfg!(target_os = "macos") {
        strings.ui_key_cmd()
    } else {
        strings.ui_key_ctrl()
    };
    SharedString::from(strings.ui_shortcut(modifier, &key.to_uppercase()))
}

pub(crate) fn entry(
    label: &'static str,
    key: &str,
    enabled: bool,
    cx: &App,
    handler: impl Fn(&mut Window, &mut App) + 'static,
) -> ContextMenuEntry {
    ContextMenuEntry::new(label)
        .shortcut(shortcut(key, cx))
        .disabled(!enabled)
        .handler(handler)
}

#[derive(Clone)]
pub(crate) struct TextMenu {
    state: Entity<PopoverState<ContextMenu>>,
}

impl TextMenu {
    pub(crate) fn new(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> Self {
        Self {
            state: window.use_keyed_state(id.into(), cx, |_, _| PopoverState::default()),
        }
    }

    pub(crate) fn open(
        &self,
        position: Point<Pixels>,
        entries: Vec<ContextMenuEntry>,
        window: &mut Window,
        cx: &mut App,
    ) {
        hide_popover(&self.state.downgrade(), window, cx);
        let entries = Rc::new(std::cell::RefCell::new(Some(entries)));
        let builder: MenuBuilder = Rc::new(move |window, cx| {
            let entries = entries.borrow_mut().take()?;
            Some(ContextMenu::build(window, cx, |menu, _, _| {
                entries
                    .into_iter()
                    .fold(menu, |menu, entry| menu.item(entry))
            }))
        });
        show_popover(&self.state, &builder, Some(position), window, cx);
        cx.set_global(ActiveTextMenu(Some(self.state.downgrade())));
    }

    pub(crate) fn overlay(&self, cx: &App) -> Option<impl IntoElement + use<>> {
        let (menu, position) = {
            let state = self.state.read(cx);
            (state.menu.clone()?, state.position()?)
        };
        let dismiss = self.state.downgrade();
        Some(
            deferred(
                anchored()
                    .position(position)
                    .anchor(Anchor::TopLeft)
                    .snap_to_window_with_margin(Sizes::WINDOW_MARGIN)
                    .child(
                        div()
                            .occlude()
                            .on_mouse_down_out(move |_, window, cx| {
                                hide_popover(&dismiss, window, cx);
                            })
                            .child(menu),
                    ),
            )
            .with_priority(3),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{
        ClipboardItem, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px, size,
    };

    use super::*;
    use crate::components::test_support::open_root;
    use crate::components::text::probe;
    use crate::components::{Input, InputState, SelectableText};
    use crate::theme::Color;

    const QUOTE: &str = "Everything that lives is designed to end.";

    fn tagged(name: &'static str) -> gpui::Div {
        div().debug_selector(move || name.to_owned())
    }

    fn right_click(cx: &mut VisualTestContext, position: Point<Pixels>) {
        probe::reset();
        cx.simulate_mouse_down(position, MouseButton::Right, Modifiers::default());
        cx.simulate_mouse_up(position, MouseButton::Right, Modifiers::default());
        cx.run_until_parked();
    }

    fn pick(cx: &mut VisualTestContext, label: &str) {
        let row = probe::painted(label).bounds.center();
        cx.simulate_click(row, Modifiers::default());
        cx.run_until_parked();
    }

    fn enabled(cx: &mut VisualTestContext, label: &str) -> bool {
        let disabled = cx.update(|_, cx| Color::Disabled.color(cx));
        probe::painted(label).color != disabled
    }

    fn clipboard(cx: &mut VisualTestContext) -> String {
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()))
            .unwrap_or_default()
    }

    #[gpui::test]
    fn inputs_get_cut_copy_paste_and_select_all_on_right_click(cx: &mut TestAppContext) {
        let fields = Rc::new(RefCell::new(Vec::new()));
        let render_fields = fields.clone();
        let (_root, cx) = open_root(cx, move |window, cx| {
            let editable = window.use_keyed_state("editable", cx, |window, cx| {
                InputState::new(window, cx).default_value("hello world")
            });
            let locked = window.use_keyed_state("locked", cx, |window, cx| {
                InputState::new(window, cx)
                    .default_value("read only")
                    .read_only(true)
            });
            *render_fields.borrow_mut() = vec![editable.clone(), locked.clone()];
            div()
                .p_4()
                .child(tagged("editable").w(px(300.)).child(Input::new(&editable)))
                .child(div().h(px(200.)))
                .child(tagged("locked").w(px(300.)).child(Input::new(&locked)))
                .into_any_element()
        });
        cx.simulate_resize(size(px(900.), px(700.)));
        cx.run_until_parked();
        let strings = cx.update(|_, cx| cx.t());
        let (cut, copy, paste, all) = (
            strings.ui_cut(),
            strings.docs_copy(),
            strings.ui_paste(),
            strings.ui_select_all(),
        );
        let editable = fields.borrow()[0].clone();
        let field = cx.debug_bounds("editable").expect("editable");
        let far_right = point(field.right() - px(12.), field.center().y);
        cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("2B".to_owned())));

        right_click(cx, far_right);
        assert!(!enabled(cx, cut));
        assert!(!enabled(cx, copy));
        assert!(enabled(cx, paste));
        assert!(enabled(cx, all));
        assert!(probe::painted(cut).bounds.left() >= far_right.x);
        editable.read_with(cx, |state, _| {
            assert_eq!(state.cursor(), "hello world".len())
        });
        pick(cx, copy);
        assert_eq!(clipboard(cx), "2B");
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();

        right_click(cx, far_right);
        pick(cx, all);
        editable.read_with(cx, |state, _| {
            assert_eq!(state.selected_text(), "hello world")
        });
        cx.update(|window, cx| assert!(editable.read(cx).is_focused(window)));

        let inside = point(field.left() + px(30.), field.center().y);
        right_click(cx, inside);
        editable.read_with(cx, |state, _| {
            assert_eq!(state.selected_text(), "hello world")
        });
        assert!(enabled(cx, cut));
        pick(cx, copy);
        assert_eq!(clipboard(cx), "hello world");

        right_click(cx, inside);
        pick(cx, cut);
        editable.read_with(cx, |state, _| assert_eq!(state.value(), ""));
        assert_eq!(clipboard(cx), "hello world");

        right_click(cx, inside);
        assert!(!enabled(cx, all));
        pick(cx, paste);
        editable.read_with(cx, |state, _| assert_eq!(state.value(), "hello world"));

        right_click(cx, inside);
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        probe::reset();
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        assert!(probe::find(cut).is_none());
        cx.update(|window, cx| assert!(editable.read(cx).is_focused(window)));

        let locked = fields.borrow()[1].clone();
        let locked_field = cx.debug_bounds("locked").expect("locked");
        right_click(cx, locked_field.center());
        pick(cx, all);
        right_click(
            cx,
            point(locked_field.left() + px(20.), locked_field.center().y),
        );
        assert!(!enabled(cx, cut));
        assert!(!enabled(cx, paste));
        assert!(enabled(cx, copy));
        pick(cx, paste);
        locked.read_with(cx, |state, _| assert_eq!(state.value(), "read only"));
        pick(cx, copy);
        assert_eq!(clipboard(cx), "read only");
    }

    #[gpui::test]
    fn selectable_text_offers_copy_and_select_all(cx: &mut TestAppContext) {
        let (_root, cx) = open_root(cx, |_, _| {
            div()
                .p_4()
                .child(tagged("quote").child(SelectableText::new("quote", QUOTE)))
                .into_any_element()
        });
        cx.simulate_resize(size(px(900.), px(700.)));
        cx.run_until_parked();
        let strings = cx.update(|_, cx| cx.t());
        let quote = cx.debug_bounds("quote").expect("quote");
        let inside = point(quote.left() + px(40.), quote.center().y);
        cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("old".to_owned())));

        right_click(cx, inside);
        assert!(probe::find(strings.ui_cut()).is_none());
        assert!(enabled(cx, strings.docs_copy()));
        pick(cx, strings.docs_copy());
        assert_eq!(clipboard(cx), QUOTE);

        cx.simulate_click(inside, Modifiers::default());
        cx.simulate_click(inside, Modifiers::default());
        right_click(cx, inside);
        pick(cx, strings.ui_select_all());
        cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("old".to_owned())));
        cx.simulate_keystrokes(&format!("secondary-{KEY_COPY}"));
        assert_eq!(clipboard(cx), QUOTE);
    }
}
