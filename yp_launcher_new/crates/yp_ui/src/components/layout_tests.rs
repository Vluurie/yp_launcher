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
fn menu_slider_previews_hover_commits_on_second_click_and_cancels_on_escape(
    cx: &mut TestAppContext,
) {
    let saved = Rc::new(Cell::new(0.125));
    let commits = Rc::new(Cell::new(0usize));
    let render_value = saved.clone();
    let render_commits = commits.clone();
    let (_, cx) = open(cx, move |_, _| {
        let value = render_value.clone();
        let count = render_commits.clone();
        tagged("menu-scale")
            .w(px(300.))
            .child(
                Slider::new("test-slider")
                    .min(0.)
                    .max(1.)
                    .step(0.001)
                    .value(value.get())
                    .on_commit(move |n, w, _| {
                        value.set(*n);
                        count.set(count.get() + 1);
                        w.refresh();
                    }),
            )
            .into_any_element()
    });
    cx.update(|w, cx| {
        set_theme(ThemeId::NierNew, cx);
        cx.set_global(yp_nier_ui::ReducedMotion(true));
        w.refresh();
    });
    cx.run_until_parked();
    let track = bounds(cx, "menu-scale");
    let start = point(track.left() + px(10.), track.center().y);
    let hover = point(track.left() + px(214.), track.center().y);
    cx.simulate_click(start, Modifiers::default());
    cx.simulate_mouse_move(hover, None, Modifiers::default());
    cx.run_until_parked();
    assert_eq!(commits.get(), 0);
    cx.simulate_keystrokes("escape");
    assert_eq!(saved.get(), 0.125);
    cx.simulate_click(start, Modifiers::default());
    cx.simulate_mouse_move(hover, None, Modifiers::default());
    cx.simulate_click(hover, Modifiers::default());
    assert_eq!(commits.get(), 1);
    let picked = saved.get();
    assert!((0.65..0.99).contains(&picked), "hover picked {picked}");
    cx.simulate_keystrokes("right");
    assert!((saved.get() - picked - 0.001).abs() < 0.00001);
    cx.simulate_keystrokes("enter end enter");
    assert_eq!(saved.get(), 1.);

    let before = commits.get();
    let outside = point(track.left() + px(50.), track.bottom() + px(24.));
    cx.simulate_mouse_down(outside, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(hover, Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_up(hover, MouseButton::Left, Modifiers::none());
    assert_eq!(commits.get(), before);
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(hover, Some(MouseButton::Left), Modifiers::none());
    assert_eq!(commits.get(), before);
    cx.simulate_mouse_up(hover, MouseButton::Left, Modifiers::none());
    assert_eq!(commits.get(), before + 1);
    assert!((0.65..0.99).contains(&saved.get()));
}

#[gpui::test]
fn labelled_menu_slider_accepts_single_clicks_across_its_full_width(cx: &mut TestAppContext) {
    let saved = Rc::new(Cell::new(0.75));
    let width = Rc::new(Cell::new(600.));
    let render_width = width.clone();
    let render_value = saved.clone();
    let (_, cx) = open(cx, move |_, _| {
        let value = render_value.clone();
        tagged("camera-setting")
            .w(px(render_width.get()))
            .child(
                SettingSlider::new("camera-side", "Camera side offset", value.get())
                    .min(-3.).max(3.).step(0.05).default_mark(0.)
                    .on_commit(move |next, w, _| {
                        value.set(*next);
                        w.refresh();
                    }),
            )
            .into_any_element()
    });
    cx.update(|w, cx| {
        set_theme(ThemeId::NierNew, cx);
        cx.set_global(yp_nier_ui::ReducedMotion(true));
        w.refresh();
    });
    cx.run_until_parked();
    let face = bounds(cx, "nier-setting-slider-face");
    let left = point(face.left() + px(1.), face.center().y);
    let right = point(face.right() - px(1.), face.center().y);
    cx.simulate_click(left, Modifiers::default());
    assert_eq!(saved.get(), -3.);
    cx.simulate_click(right, Modifiers::default());
    assert_eq!(saved.get(), 3.);
    cx.simulate_keystrokes("left");
    assert!((saved.get() - 2.95).abs() < 0.0001);
    cx.simulate_click(face.center(), Modifiers::default());
    assert_eq!(saved.get(), 0.);
    let outside = point(face.left() - px(15.), face.bottom() + px(8.));
    cx.simulate_mouse_down(right, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(outside, Some(MouseButton::Left), Modifiers::default());
    cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::default());
    assert_eq!(saved.get(), -3.);

    assert!(face.size.width <= px(364.));
    cx.simulate_keystrokes("tab secondary-a");
    cx.simulate_input("-1.25");
    cx.simulate_keystrokes("enter");
    assert_eq!(saved.get(), -1.25);

    width.set(225.);
    cx.update(|w, _| w.refresh());
    cx.run_until_parked();
    let container = bounds(cx, "camera-setting");
    let compact = bounds(cx, "nier-setting-slider-face");
    assert!(compact.left() >= container.left() && compact.right() <= container.right());
    assert!(compact.top() > container.top(), "the full label should wrap above the scale");
}

#[gpui::test]
fn slider_number_fits_signed_limits_beside_the_live_marker(cx: &mut TestAppContext) {
    let (_, cx) = open(cx, |_, _| {
        tagged("small-slider-panel").w(px(180.)).child(
            SettingSlider::new("percentage", "Scale", 100.)
                .min(-100.).max(100.).step(0.1).decimals(1).apply_marker(ApplyKind::Live)
        ).into_any_element()
    });
    cx.update(|w, cx| { set_theme(ThemeId::NierNew, cx); w.refresh(); });
    cx.run_until_parked();
    let field = bounds(cx, "nier-setting-slider-number");
    let panel = bounds(cx, "small-slider-panel");
    let text_width = cx.update(|w, cx| {
        use crate::theme::ResponsiveExt;
        let run = w.text_style().to_run(6);
        w.text_system().shape_line("-100.0".into(), w.sizes(cx).font_md, &[run], None).width
    });
    assert!(field.size.width >= text_width + px(22.));
    assert!(field.right() <= panel.right());
    assert!(bounds(cx, "nier-setting-slider-face").right() <= field.left());
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
fn scrollbars_reserve_space_outside_full_width_and_full_height_content(cx: &mut TestAppContext) {
    let vertical = gpui::ScrollHandle::new();
    let horizontal = gpui::ScrollHandle::new();
    let y = vertical.clone();
    let x = horizontal.clone();
    let (_, cx) = open(cx, move |_, _| {
        v_flex().size_full()
            .child(div().id("vertical-pane").w(px(200.)).h(px(80.))
                .child(tagged("wide-row").w_full().h(px(200.)))
                .overflow_y_scrollbar(&y))
            .child(div().id("horizontal-pane").w(px(200.)).h(px(80.))
                .child(tagged("tall-row").w(px(400.)).h_full())
                .overflow_x_scrollbar(&x))
            .into_any_element()
    });
    assert!(vertical.max_offset().y > px(0.));
    assert!(horizontal.max_offset().x > px(0.));
    assert!(bounds(cx, "wide-row").right() <= vertical.bounds().right() - Sizes::SCROLLBAR_WIDTH);
    assert!(bounds(cx, "tall-row").bottom() <= horizontal.bounds().bottom() - Sizes::SCROLLBAR_WIDTH);
}

#[gpui::test]
fn virtual_list_scrollbars_can_be_dragged_outside_the_rows(cx: &mut TestAppContext) {
    let uniform = gpui::UniformListScrollHandle::new();
    let list = gpui::ListState::new(80, gpui::ListAlignment::Top, px(60.));
    let uniform_handle = uniform.clone();
    let list_state = list.clone();
    let (_, cx) = open(cx, move |_, _| {
        v_flex().w(px(220.))
            .child(tagged("uniform-pane").w_full().h(px(100.)).child(VirtualList::uniform(
                "uniform", 80, &uniform_handle, |range, _, _| range.map(|_| {
                    div().h(px(25.)).w_full()
                }).collect(),
            )))
            .child(tagged("variable-pane").w_full().h(px(100.)).child(VirtualList::list(
                "variable", &list_state, |_, _, _| div().h(px(25.)).w_full().into_any_element(),
            )))
            .into_any_element()
    });
    let uniform_bounds = uniform.0.borrow().base_handle.bounds();
    for (tag, content) in [("uniform-pane", uniform_bounds), ("variable-pane", list.viewport_bounds())] {
        let pane = bounds(cx, tag);
        assert!(content.right() <= pane.right() - Sizes::SCROLLBAR_WIDTH);
        let start = point(pane.right() - Sizes::SCROLLBAR_WIDTH / 2., pane.top() + px(5.));
        let end = point(start.x, pane.bottom() - px(5.));
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(end, Some(MouseButton::Left), Modifiers::none());
        cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
    }
    assert!(uniform.0.borrow().base_handle.offset().y < px(0.));
    assert!(list.scroll_px_offset_for_scrollbar().y < px(0.));
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
fn nier_spinner_draws_the_loading_icon_with_reduced_motion(cx: &mut TestAppContext) {
    let (_, cx) = open(cx, |_, _| {
        tagged("activity-icon")
            .size(px(48.))
            .child(Spinner::new().size(IconSize::Custom(px(48.))))
            .into_any_element()
    });
    cx.update(|w, cx| {
        set_theme(ThemeId::NierNew, cx);
        cx.set_global(yp_nier_ui::ReducedMotion(true));
        w.refresh();
    });
    cx.run_until_parked();
    let squares = cx.update(|window, _| {
        let scale = window.scale_factor();
        window
            .painted_quads()
            .iter()
            .filter(|quad| {
                let size = quad.bounds.size;
                (size.width.0 - size.height.0).abs() <= 1.
                    && (3.0..=6.0).contains(&(size.width.0 / scale))
            })
            .count()
    });
    assert_eq!(squares, 12);
}

#[gpui::test]
fn external_file_drag_highlights_only_an_accepting_target(cx: &mut TestAppContext) {
    use gpui::{ExternalPaths, FileDropEvent};
    let drops = Rc::new(Cell::new(0));
    let browses = Rc::new(Cell::new(0));
    let busy = Rc::new(Cell::new(false));
    let render_busy = busy.clone();
    let render_browses = browses.clone();
    let render_drops = drops.clone();
    let (_, cx) = open(cx, move |_, _| {
        let drops = render_drops.clone();
        let browses = render_browses.clone();
        tagged("file-drop-target")
            .w(px(360.))
            .child(
                DropZone::new("textures")
                    .title("Textures")
                    .accept(|path| path.extension().is_some_and(|ext| ext == "dds"))
                    .busy(render_busy.get().then(|| DropProgress::new("Installing")))
                    .on_browse_files(move |_, _| browses.set(browses.get() + 1))
                    .on_drop_paths(move |paths, _, _| drops.set(drops.get() + paths.len())),
            )
            .into_any_element()
    });
    cx.update(|w, cx| {
        set_theme(ThemeId::NierNew, cx);
        cx.set_global(yp_nier_ui::ReducedMotion(true));
        w.refresh();
    });
    cx.run_until_parked();
    let zone = bounds(cx, "file-drop-target");
    let inside = zone.center();
    let outside = point(zone.right() + px(20.), inside.y);
    let enter = |path: &str| FileDropEvent::Entered {
        position: inside,
        paths: ExternalPaths([std::path::PathBuf::from(path)].into_iter().collect()),
    };
    let highlighted = |cx: &mut VisualTestContext| {
        cx.update(|w, _| {
            w.painted_quads().iter().any(|q| {
                q.border_color.a > 0.8
                    && q.border_widths.left.0 >= 4.
                    && q.bounds.size.width.0 / w.scale_factor() >= f32::from(zone.size.width) - 4.
            })
        })
    };
    assert!(!highlighted(cx));
    cx.simulate_event(enter("mesh.wmb"));
    assert!(!highlighted(cx));
    cx.simulate_event(FileDropEvent::Exited);
    cx.simulate_event(enter("cloth.dds"));
    assert!(
        highlighted(cx),
        "accepted file should outline the entire target"
    );
    let hacking_static = |cx: &mut VisualTestContext| {
        let shades = [0x738d920f, 0x9f776811].map(|value| gpui::Hsla::from(gpui::rgba(value)));
        cx.update(|w, _| {
            w.painted_quads().iter().any(|q| {
                q.background.as_solid().is_some_and(|color| shades.contains(&color))
            })
        })
    };
    assert!(!hacking_static(cx));
    cx.update(|w, cx| {
        cx.set_global(yp_nier_ui::ReducedMotion(false));
        w.refresh();
    });
    cx.run_until_parked();
    assert!(hacking_static(cx));
    cx.simulate_event(FileDropEvent::Pending { position: outside });
    assert!(!highlighted(cx));
    assert!(!hacking_static(cx));
    cx.simulate_event(FileDropEvent::Pending { position: inside });
    assert!(highlighted(cx));
    cx.simulate_event(FileDropEvent::Submit { position: inside });
    assert_eq!(drops.get(), 1);
    assert!(!highlighted(cx));
    assert!(!hacking_static(cx));
    assert_eq!(browses.get(), 0);
    cx.simulate_click(inside, Modifiers::default());
    assert_eq!(browses.get(), 1);
    busy.set(true);
    cx.update(|w, _| w.refresh());
    cx.run_until_parked();
    cx.simulate_event(enter("cloth.dds"));
    assert!(!highlighted(cx));
    cx.simulate_event(FileDropEvent::Exited);
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
