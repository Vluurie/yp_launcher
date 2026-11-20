use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnyElement, AvailableSpace, Bounds, DismissEvent, EventEmitter, FocusHandle, Focusable,
    Modifiers, MouseButton, MouseDownEvent, Pixels, Render, ScrollEvent, TestAppContext,
    TouchPhase, VisualTestContext, Window, div, point, prelude::*, px, size,
};

use super::test_support::{Host, open_root};
use super::text::probe::painted;
use super::*;
use crate::assets::asset_image;
use crate::styled_ext::{fill_parent, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, Sizes, ThemeId, set_theme};
use crate::traits::{ButtonCommon, Clickable, FixedWidth};

const LONG: &str = "A very long mod name that should end in an ellipsis because it cannot fit";
const WINDOW: (f32, f32) = (1000., 700.);

fn tagged(name: &'static str) -> gpui::Div {
    div().debug_selector(move || name.to_owned())
}

fn bounds(cx: &mut VisualTestContext, name: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(name)
        .unwrap_or_else(|| panic!("no element tagged {name}"))
}

fn open(
    cx: &mut TestAppContext,
    render: impl Fn(&mut Window, &mut gpui::Context<Host>) -> AnyElement + 'static,
) -> (gpui::Entity<Root>, &mut VisualTestContext) {
    let (root, cx) = open_root(cx, render);
    cx.simulate_resize(size(px(WINDOW.0), px(WINDOW.1)));
    cx.run_until_parked();
    (root, cx)
}

struct Bar;

impl Render for Bar {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        TitleBar::new("Launcher")
            .child(IconButton::new("bar-action", IconName::Tune).on_click(|_, _, _| {}))
    }
}

fn mouse_down_is_prevented(cx: &mut VisualTestContext, position: gpui::Point<Pixels>) -> bool {
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position,
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    cx.update(|window, _| window.default_prevented())
}

#[gpui::test]
fn a_mouse_down_on_the_title_bar_is_left_to_the_platform(cx: &mut TestAppContext) {
    let (root, cx) = open(cx, |_, _| {
        v_flex()
            .size_full()
            .child(Button::new("first", "First").on_click(|_, _, _| {}))
            .into_any_element()
    });
    root.update(cx, |root, cx| {
        let bar = cx.new(|_| Bar);
        root.set_title_bar(Some(bar.into()), cx);
    });
    cx.run_until_parked();
    let drag_area = point(px(300.), Sizes::TITLE_BAR_HEIGHT / 2.);
    assert!(!mouse_down_is_prevented(cx, drag_area));
    let title_text = point(px(30.), Sizes::TITLE_BAR_HEIGHT / 2.);
    assert!(!mouse_down_is_prevented(cx, title_text));
    let empty_body = point(px(600.), px(500.));
    assert!(mouse_down_is_prevented(cx, empty_body));
}

#[gpui::test]
fn tab_reaches_controls_after_the_focused_element_is_gone(cx: &mut TestAppContext) {
    let show_second = Rc::new(Cell::new(true));
    let render_second = show_second.clone();
    let (root, cx) = open(cx, move |_, _| {
        v_flex()
            .size_full()
            .child(Button::new("first", "First").on_click(|_, _, _| {}))
            .when(render_second.get(), |this| {
                this.child(Button::new("second", "Second").on_click(|_, _, _| {}))
            })
            .into_any_element()
    });
    let root_focus = root.read_with(cx, |root, cx| root.focus_handle(cx));
    cx.simulate_keystrokes("tab tab");
    cx.update(|window, _| assert!(!root_focus.is_focused(window)));
    show_second.set(false);
    root.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    cx.update(|window, cx| assert!(window.focused(cx).is_some()));
    cx.simulate_keystrokes("tab");
    cx.update(|window, cx| {
        assert!(window.focused(cx).is_some());
        assert!(!root_focus.is_focused(window));
    });
    cx.update(|window, cx| window.blur(cx));
    root.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    cx.simulate_keystrokes("tab");
    cx.update(|window, cx| {
        assert!(window.focused(cx).is_some());
        assert!(!root_focus.is_focused(window));
    });
}

struct TrapModal {
    focus_handle: FocusHandle,
}

impl EventEmitter<DismissEvent> for TrapModal {}

impl Focusable for TrapModal {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for TrapModal {}

impl Render for TrapModal {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div().track_focus(&self.focus_handle).child(
            Modal::new("trap", None)
                .section(Button::new("modal-a", "A").on_click(|_, _, _| {}))
                .section(Button::new("modal-b", "B").on_click(|_, _, _| {})),
        )
    }
}

#[gpui::test]
fn tab_stays_inside_the_top_modal(cx: &mut TestAppContext) {
    let (_root, cx) = open(cx, |_, _| {
        v_flex()
            .size_full()
            .child(Button::new("behind-a", "Behind").on_click(|_, _, _| {}))
            .child(Button::new("behind-b", "Behind").on_click(|_, _, _| {}))
            .into_any_element()
    });
    let modal = cx
        .update(|window, cx| {
            window.open_modal(cx, |_, cx| TrapModal {
                focus_handle: cx.focus_handle(),
            })
        })
        .expect("root");
    cx.run_until_parked();
    let scope = modal.read_with(cx, |modal, _| modal.focus_handle.clone());
    for keys in ["tab", "tab", "tab", "shift-tab", "shift-tab", "tab"] {
        cx.simulate_keystrokes(keys);
        cx.update(|window, cx| assert!(scope.contains_focused(window, cx)));
    }
}

#[gpui::test]
fn long_tooltips_grow_downwards_instead_of_overflowing(cx: &mut TestAppContext) {
    let (_root, cx) = open(cx, |_, _| div().into_any_element());
    let long = format!("{LONG}. {LONG}. {LONG}.");
    let tooltip = long.clone();
    cx.draw(
        point(px(10.), px(10.)),
        AvailableSpace::min_size(),
        move |window, cx| Tooltip::with_meta(tooltip.clone(), "meta line")(window, cx).into_any_element(),
    );
    let title = painted(&long);
    let meta = painted("meta line");
    assert!(title.lines >= 3);
    assert!(title.bounds.size.width <= Sizes::TOOLTIP_MAX_WIDTH);
    assert!(title.bounds.size.height >= px(40.));
    assert!(meta.bounds.top() >= title.bounds.bottom());
}

#[gpui::test]
fn percent_sized_content_fills_every_fill_slot(cx: &mut TestAppContext) {
    let (_root, cx) = open(cx, |_, _| {
        let left = v_flex()
            .size_full()
            .child(tagged("left-top").w_full().h(px(40.)))
            .child(fill_parent(tagged("left-fill").size_full()));
        let right = Card::new()
            .full_height()
            .padding(CardPadding::None)
            .child(tagged("card-fill").size_full());
        v_flex()
            .size_full()
            .child(PageHeader::new("Page"))
            .child(
                div().flex_1().min_h_0().child(
                    tagged("columns").size_full().child(
                        TwoColumn::new(left, right)
                            .id("columns")
                            .breakpoint(px(0.))
                            .fill_height(true),
                    ),
                ),
            )
            .child(
                div().h(px(120.)).w_full().child(
                    h_resizable("panels")
                        .child(
                            resizable_panel()
                                .size(px(200.))
                                .child(tagged("panel-fixed").size_full()),
                        )
                        .child(resizable_panel().child(tagged("panel-flex").size_full())),
                ),
            )
            .into_any_element()
    });
    let columns = bounds(cx, "columns");
    assert_eq!(columns.size.width, px(WINDOW.0));
    assert!(columns.size.height > px(400.));
    let left_fill = bounds(cx, "left-fill");
    assert_eq!(left_fill.size.height, columns.size.height - px(40.));
    assert!(left_fill.size.width > px(400.));
    let card_fill = bounds(cx, "card-fill");
    assert!(card_fill.size.height > columns.size.height - px(8.));
    assert!(card_fill.size.width > px(400.));
    assert_eq!(bounds(cx, "panel-fixed").size, size(px(200.), px(120.)));
    let flexible = bounds(cx, "panel-flex");
    assert_eq!(flexible.size.height, px(120.));
    assert_eq!(
        flexible.size.width,
        px(WINDOW.0) - px(200.) - Sizes::RESIZE_HANDLE_WIDTH
    );
}

#[gpui::test]
fn full_width_controls_keep_their_width_in_plain_wrappers(cx: &mut TestAppContext) {
    let (_root, cx) = open(cx, |window, cx| {
        let input = window.use_keyed_state("input", cx, InputState::new);
        v_flex()
            .size_full()
            .child(
                div().flex_1().min_h_0().child(
                    div()
                        .size_full()
                        .p_4()
                        .child(div().child(tagged("plain").child(Input::new(&input)))),
                ),
            )
            .child(
                h_flex()
                    .w_full()
                    .child(tagged("row-auto").child(div().w_full().h(px(8.))))
                    .child(tagged("row-grow").flex_1().child(div().w_full().h(px(8.)))),
            )
            .into_any_element()
    });
    assert_eq!(bounds(cx, "plain").size.width, px(WINDOW.0) - px(32.));
    assert_eq!(bounds(cx, "row-auto").size.width, px(0.));
    assert_eq!(bounds(cx, "row-grow").size.width, px(WINDOW.0));
}

#[gpui::test]
fn aspect_ratio_derives_the_height_from_the_width(cx: &mut TestAppContext) {
    let (_root, cx) = open(cx, |_, _| {
        v_flex()
            .w(px(320.))
            .child(tagged("wide").w_full().aspect_ratio(16. / 9.))
            .child(tagged("square").w(px(50.)).aspect_square())
            .into_any_element()
    });
    assert_eq!(bounds(cx, "wide").size, size(px(320.), px(180.)));
    assert_eq!(bounds(cx, "square").size, size(px(50.), px(50.)));
}

#[gpui::test]
fn long_names_in_rows_end_in_an_ellipsis(cx: &mut TestAppContext) {
    let (_root, _cx) = open(cx, |_, _| {
        v_flex()
            .w(px(260.))
            .child(
                ListItem::new("direct")
                    .end_slot(Chip::new("DATA"))
                    .child(Label::new(format!("{LONG} 1")).truncate()),
            )
            .child(
                ListItem::new("wrapped").child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Label::new(format!("{LONG} 2")).truncate()),
                ),
            )
            .child(
                ListItem::new("column").child(
                    v_flex()
                        .min_w_0()
                        .child(Label::new(format!("{LONG} 3")).truncate())
                        .child(Label::new("short")),
                ),
            )
            .child(Link::new("link", format!("{LONG} 4")).truncate())
            .child(Label::new(format!("{LONG} 5")).line_clamp(2))
            .into_any_element()
    });
    for index in 1..=4 {
        let text = painted(&format!("{LONG} {index}"));
        assert!(
            text.shown.ends_with('\u{2026}'),
            "row {index}: {}",
            text.shown
        );
        assert!(
            text.shown.chars().count() > 8,
            "row {index}: {}",
            text.shown
        );
        assert!(text.bounds.size.width <= px(260.));
        assert_eq!(text.lines, 1);
    }
    assert_eq!(painted("short").shown, "short");
    let clamped = painted(&format!("{LONG} 5"));
    assert_eq!(clamped.lines, 2);
    assert!(clamped.shown.ends_with('\u{2026}'), "{}", clamped.shown);
}

#[gpui::test]
fn narrow_groups_and_links_stay_on_one_line(cx: &mut TestAppContext) {
    let (_root, cx) = open(cx, |_, _| {
        v_flex()
            .w(px(170.))
            .child(
                ToggleButtonGroup::new("group")
                    .option("First long option", None)
                    .option("Second long option", None)
                    .full_width()
                    .selected(0)
                    .on_select(|_, _, _| {}),
            )
            .child(tagged("link-row").child(Link::new("link", "A link label that is long")))
            .into_any_element()
    });
    for text in ["First long option", "Second long option"] {
        let label = painted(text);
        assert!(label.shown.ends_with('\u{2026}'), "{}", label.shown);
        assert!(label.bounds.size.width < px(85.));
        assert_eq!(label.lines, 1);
    }
    let link = painted("A link label that is long");
    assert_eq!(link.lines, 1);
    assert!(bounds(cx, "link-row").size.height < link.bounds.size.height * 1.5);
}

#[gpui::test]
fn padding_on_a_scroll_container_does_not_extend_its_scroll_range(cx: &mut TestAppContext) {
    let fits_y = gpui::ScrollHandle::new();
    let fits_x = gpui::ScrollHandle::new();
    let tall = gpui::ScrollHandle::new();
    let handles = (fits_y.clone(), fits_x.clone(), tall.clone());
    let (_root, _cx) = open(cx, move |_, _| {
        v_flex()
            .size_full()
            .child(
                div()
                    .id("fits-y")
                    .h(px(200.))
                    .w_full()
                    .p(px(16.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().h(px(50.)).w_full())
                    .child(div().h(px(50.)).w_full())
                    .overflow_y_scrollbar(&handles.0),
            )
            .child(
                div()
                    .id("fits-x")
                    .w(px(300.))
                    .p(px(16.))
                    .child(div().w_full().h(px(20.)))
                    .overflow_x_scrollbar(&handles.1),
            )
            .child(
                div()
                    .id("tall")
                    .h(px(100.))
                    .w_full()
                    .p(px(16.))
                    .child(div().h(px(300.)).w_full())
                    .overflow_y_scrollbar(&handles.2),
            )
            .into_any_element()
    });
    assert_eq!(fits_y.max_offset().y, px(0.));
    assert_eq!(fits_x.max_offset().x, px(0.));
    assert_eq!(tall.max_offset().y, px(232.));
}

#[gpui::test]
fn labels_in_content_sized_controls_are_never_cut(cx: &mut TestAppContext) {
    let (_root, _cx) = open(cx, |_, _| {
        let items = vec![
            SelectItem::new("Outfit A", 0),
            SelectItem::new("Outfit B", 1),
        ];
        v_flex()
            .size_full()
            .items_start()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Select::new("content-select", "Outfit A")
                            .items(items.clone())
                            .on_select(|_: &i32, _, _| {}),
                    )
                    .child(
                        Select::new("clamped-select", format!("{LONG} select"))
                            .items(items)
                            .max_width(px(140.))
                            .on_select(|_: &i32, _, _| {}),
                    )
                    .child(Button::new("content-button", "Apply").on_click(|_, _, _| {}))
                    .child(Button::new("truncating-button", "Reset").truncate())
                    .child(Chip::new("Chip label").truncate())
                    .child(Tab::new("tab").label("Tab label")),
            )
            .child(div().w(px(420.)).child(ImageComparison::new(
                "comparison",
                asset_image("images/config/missing-a.webp"),
                asset_image("images/config/missing-b.webp"),
                "2021",
                "2017",
            )))
            .child(
                h_flex().flex_wrap().gap_3().child(
                    Select::new("wrapped-select", format!("{LONG} wrapped"))
                        .items(vec![SelectItem::new("One", 0)])
                        .max_width(px(220.))
                        .on_select(|_: &i32, _, _| {}),
                ),
            )
            .child(
                div().w(px(120.)).child(
                    Button::new("narrow-button", format!("{LONG} button"))
                        .full_width()
                        .truncate(),
                ),
            )
            .into_any_element()
    });
    for text in [
        "Outfit A",
        "Apply",
        "Reset",
        "Chip label",
        "Tab label",
        "2021",
        "2017",
    ] {
        let label = painted(text);
        assert_eq!(label.shown, text);
        assert!(
            label.bounds.size.width > px(10.),
            "{text}: {:?}",
            label.bounds
        );
    }
    for text in [
        format!("{LONG} select"),
        format!("{LONG} wrapped"),
        format!("{LONG} button"),
    ] {
        let label = painted(&text);
        assert!(label.shown.ends_with('\u{2026}'), "{}", label.shown);
        assert!(label.bounds.size.width > px(40.), "{:?}", label.bounds);
    }
}

#[gpui::test]
fn the_drop_zone_title_keeps_its_line_in_absolute_panes(cx: &mut TestAppContext) {
    let (_root, _cx) = open(cx, |_, _| {
        let pane = v_flex().size_full().child(
            div().relative().flex_none().child(
                DropZone::new("zone")
                    .title("Drop a mod archive here")
                    .hint("zip, 7z, rar or a folder"),
            ),
        );
        h_flex()
            .size_full()
            .child(div().w(px(500.)).h_full())
            .child(
                div().flex_1().h_full().child(
                    div()
                        .relative()
                        .size_full()
                        .child(div().absolute().inset_0().child(pane)),
                ),
            )
            .into_any_element()
    });
    let title = painted("Drop a mod archive here");
    assert_eq!(title.lines, 1);
    assert!(title.bounds.size.width > px(100.));
    let hint = painted("zip, 7z, rar or a folder");
    assert_eq!(hint.lines, 1);
    assert!(hint.bounds.top() >= title.bounds.bottom());
}

fn nier_buttons(top: Pixels) -> AnyElement {
    v_flex()
        .size_full()
        .child(div().h(px(60.) + top).flex_none())
        .child(
            h_flex().child(
                Button::new("tinted", "Tinted")
                    .style(ButtonStyle::Tinted(TintColor::Muted))
                    .on_click(|_, _, _| {}),
            ),
        )
        .child(div().h(px(40.)).flex_none())
        .child(
            h_flex().child(
                Button::new("coloured", "Coloured")
                    .style(ButtonStyle::Tinted(TintColor::Error))
                    .color(Color::Error)
                    .on_click(|_, _, _| {}),
            ),
        )
        .into_any_element()
}

#[gpui::test]
fn hovered_labels_take_the_hover_colour_even_when_the_layout_moves(cx: &mut TestAppContext) {
    let offset = Rc::new(Cell::new(px(0.)));
    let render_offset = offset.clone();
    let (_root, cx) = open(cx, move |_, _| nier_buttons(render_offset.get()));
    cx.update(|_, cx| set_theme(ThemeId::Nier, cx));
    cx.run_until_parked();
    let (ink, on_ink, error) = cx.update(|_, cx| {
        (
            Color::Muted.color(cx),
            Color::OnAccent.color(cx),
            cx.theme().colors.error,
        )
    });
    assert_eq!(painted("Tinted").color, ink);
    assert_eq!(painted("Coloured").color, error);
    let over_first = painted("Tinted").bounds.center();
    cx.simulate_mouse_move(over_first, None, Modifiers::default());
    cx.run_until_parked();
    assert_eq!(painted("Tinted").color, on_ink);
    assert_eq!(painted("Coloured").color, error);
    let distance = painted("Coloured").bounds.top() - painted("Tinted").bounds.top();
    offset.set(-distance);
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    assert!(painted("Coloured").bounds.contains(&over_first));
    assert_eq!(painted("Tinted").color, ink);
    assert_eq!(painted("Coloured").color, on_ink);
    cx.simulate_event(ScrollEvent {
        position: over_first,
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-10.))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
        source: gpui::ScrollSource::Pointer,
    });
    cx.run_until_parked();
    assert_eq!(painted("Coloured").color, on_ink);
}

#[gpui::test]
fn an_open_menu_shields_its_trigger_and_a_click_on_it_closes_the_menu(cx: &mut TestAppContext) {
    let handle = PopoverMenuHandle::<ContextMenu>::new();
    let render_handle = handle.clone();
    let (_root, cx) = open(cx, move |_, _| {
        div()
            .p_4()
            .child(
                PopoverMenu::<ContextMenu>::new("menu")
                    .with_handle(render_handle.clone())
                    .trigger(
                        tagged("trigger").child(
                            Button::new("menu-trigger", "Help")
                                .tooltip(Tooltip::text("Opens the help menu")),
                        ),
                    )
                    .menu(|window, cx| {
                        Some(ContextMenu::build(window, cx, |menu, _, _| {
                            menu.entry("Only", |_, _| {})
                        }))
                    }),
            )
            .into_any_element()
    });
    let trigger = bounds(cx, "trigger").center();
    let hover = |cx: &mut VisualTestContext| {
        super::text::probe::reset();
        cx.simulate_mouse_move(trigger + point(px(2.), px(1.)), None, Modifiers::default());
        cx.simulate_mouse_move(trigger, None, Modifiers::default());
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(900));
        cx.run_until_parked();
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        super::text::probe::find("Opens the help menu").is_some()
    };
    assert!(hover(cx));
    cx.simulate_mouse_move(point(px(400.), px(400.)), None, Modifiers::default());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(900));
    cx.run_until_parked();
    cx.simulate_mouse_move(trigger, None, Modifiers::default());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(150));
    cx.simulate_click(trigger, Modifiers::default());
    cx.run_until_parked();
    cx.update(|_, cx| assert!(handle.is_deployed(cx)));
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    assert!(!hover(cx));
    cx.simulate_click(trigger, Modifiers::default());
    cx.run_until_parked();
    cx.update(|_, cx| assert!(!handle.is_deployed(cx)));
}
