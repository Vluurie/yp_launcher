use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    AnyElement, AnyView, App, ListAlignment, ListState, Modifiers, ScrollHandle, SharedString,
    TestAppContext, UniformListScrollHandle, VisualTestContext, Window, div, point, prelude::*, px,
    size,
};

use super::test_support::open_root;
use super::*;
use crate::assets::asset_image;
use crate::i18n::I18n;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{Breakpoint, Color, Severity, ThemeId, set_breakpoint_override, set_theme};
use crate::traits::{
    ButtonCommon, Clickable, Disableable, FixedWidth, SelectableButton, ToggleState, Toggleable,
    VisibleOnHover,
};
use crate::{Locale, StyledExt};

const LONG: &str = "The android stood in the ruins and counted the stars, twice, to be sure.";

fn typography() -> AnyElement {
    v_flex()
        .child(
            Headline::new("Hero")
                .size(HeadlineSize::Hero)
                .uppercase()
                .centered(),
        )
        .child(Headline::new("Page").color(Color::Accent))
        .children(
            [
                LabelSize::XSmall,
                LabelSize::Small,
                LabelSize::Default,
                LabelSize::Large,
                LabelSize::XLarge,
            ]
            .into_iter()
            .map(|size| Label::new(LONG).size(size).color(Color::Secondary)),
        )
        .child(
            Label::new(LONG)
                .bold()
                .italic()
                .underline()
                .strikethrough()
                .uppercase()
                .tracking(Tracking::Caps)
                .mono()
                .truncate()
                .alpha(0.5),
        )
        .child(
            Label::new(LONG)
                .line_clamp(2)
                .centered()
                .semibold()
                .line_height(1.1),
        )
        .child(Label::new(""))
        .child(
            HighlightedLabel::new(LONG, vec![0..3, 2..9, 400..500])
                .mono()
                .truncate(),
        )
        .child(HighlightedLabel::matching("\u{00e4}\u{6587}", "\u{00c4}").size(LabelSize::Large))
        .child(
            Icon::new(IconName::Tune)
                .size(IconSize::XSmall)
                .rotate(gpui::radians(-1.)),
        )
        .child(
            Icon::new(IconName::ProgressRing)
                .size(IconSize::Custom(px(40.)))
                .alpha(0.3),
        )
        .child(Divider::horizontal().inset())
        .child(
            h_flex()
                .h(px(20.))
                .child(Divider::vertical().color(DividerColor::Medium).inset()),
        )
        .child(
            Link::new("smoke-link", "Link")
                .href("https://example.invalid")
                .icon(IconName::Link)
                .end_icon(IconName::ArrowOutward)
                .underline()
                .tooltip("tip"),
        )
        .child(Kbd::new("Ctrl"))
        .child(Kbd::new("K").active(true))
        .child(SelectableText::new("smoke-selectable", LONG).mono())
        .child(SelectableText::new("smoke-selectable-empty", ""))
        .into_any_element()
}

fn buttons() -> AnyElement {
    let styles = [
        ButtonStyle::Filled,
        ButtonStyle::Tinted(TintColor::Accent),
        ButtonStyle::Tinted(TintColor::Success),
        ButtonStyle::Tinted(TintColor::Warning),
        ButtonStyle::Tinted(TintColor::Error),
        ButtonStyle::Tinted(TintColor::Muted),
        ButtonStyle::Tinted(TintColor::Primary),
        ButtonStyle::Outlined,
        ButtonStyle::Subtle,
        ButtonStyle::Transparent,
        ButtonStyle::Link,
    ];
    let sizes = [
        ButtonSize::Compact,
        ButtonSize::Default,
        ButtonSize::Medium,
        ButtonSize::Large,
        ButtonSize::Hero,
    ];
    v_flex()
        .children(styles.into_iter().enumerate().map(|(index, style)| {
            h_flex()
                .child(
                    Button::new(("smoke-button", index), "Save")
                        .style(style)
                        .start_icon(IconName::Save)
                        .end_icon(IconName::ArrowForward)
                        .tooltip(Tooltip::text("tip"))
                        .on_click(|_, _, _| {}),
                )
                .child(
                    Button::new(("smoke-button-disabled", index), "Save")
                        .style(style)
                        .disabled(true)
                        .on_click(|_, _, _| {}),
                )
                .child(
                    Button::new(("smoke-button-selected", index), "Save")
                        .style(style)
                        .toggle_state(true)
                        .selected_label("Saved")
                        .selected_style(ButtonStyle::Filled),
                )
                .child(
                    Button::new(("smoke-button-loading", index), "Save")
                        .style(style)
                        .loading(true)
                        .color(Color::Warning)
                        .label_size(LabelSize::XSmall)
                        .truncate()
                        .full_width(),
                )
        }))
        .children(sizes.into_iter().enumerate().map(|(index, size)| {
            h_flex()
                .child(
                    Button::new(("smoke-size", index), "Size")
                        .size(size)
                        .width(px(120.))
                        .tab_index(3),
                )
                .child(
                    IconButton::new(("smoke-icon-size", index), IconName::Refresh)
                        .size(size)
                        .shape(IconButtonShape::Wide)
                        .bordered(true)
                        .on_click(|_, _, _| {}),
                )
        }))
        .child(
            ButtonLike::new_rounded_left("smoke-like-left")
                .child(Label::new("L"))
                .on_click(|_, _, _| {}),
        )
        .child(ButtonLike::new_rounded_right("smoke-like-right").child(Label::new("R")))
        .child(ButtonLike::new_rounded_all("smoke-like-all").radius(px(12.)))
        .child(ButtonLike::new_square("smoke-like-square").padding(px(1.), px(1.)))
        .child(
            IconButton::new("smoke-icon-toggle", IconName::StarOutline)
                .selected_icon(IconName::Star)
                .toggle_state(ToggleState::Selected)
                .icon_color(Color::Warning)
                .icon_size(IconSize::Large),
        )
        .child(IconButton::new("smoke-icon-loading", IconName::Save).loading(true))
        .child(CopyButton::new("smoke-copy", "text"))
        .child(
            CopyButton::new("smoke-copy-label", "text")
                .label("Copy")
                .bordered(true),
        )
        .child(
            ToggleButtonGroup::new("smoke-toggle-empty")
                .selected(4)
                .on_select(|_, _, _| {}),
        )
        .child(
            ToggleButtonGroup::new("smoke-toggle")
                .option("One", IconName::Bolt)
                .option("Two", None)
                .option_with(
                    ToggleOption::new("Three")
                        .count(0)
                        .tooltip("tip")
                        .disabled(true),
                )
                .selected(1)
                .full_width()
                .on_select(|_, _, _| {}),
        )
        .child(
            ChoiceCard::new("smoke-choice", "Choice")
                .description(LONG)
                .icon(IconName::ShieldOutlined)
                .selected(true)
                .on_click(|_, _, _| {}),
        )
        .child(
            ChoiceCard::new("smoke-choice-disabled", "Choice")
                .compact()
                .disabled(true),
        )
        .child(
            div()
                .group("smoke-hover")
                .child(div().visible_on_hover("smoke-hover")),
        )
        .into_any_element()
}

fn inputs(window: &mut Window, cx: &mut App) -> AnyElement {
    let single = window.use_keyed_state("smoke-input", cx, |window, cx| {
        InputState::new(window, cx)
            .placeholder("placeholder")
            .max_length(12)
            .default_value("hello world")
    });
    let empty = window.use_keyed_state("smoke-input-empty", cx, InputState::new);
    let multi = window.use_keyed_state("smoke-input-multi", cx, |window, cx| {
        InputState::new(window, cx)
            .multi_line(2)
            .max_rows(3)
            .mono()
            .validate(|text| {
                if text.is_empty() {
                    Err("required".into())
                } else {
                    Ok(())
                }
            })
            .default_value("one\ntwo\n\nfour with a very long line that has to wrap around\nfive")
    });
    let invalid = window.use_keyed_state("smoke-input-invalid", cx, |window, cx| {
        InputState::new(window, cx)
            .read_only(true)
            .validate(|_| Err("always".into()))
            .default_value("x")
    });
    let number = window.use_keyed_state("smoke-number", cx, |window, cx| {
        InputState::new(window, cx).default_value("abc")
    });
    v_flex()
        .w(px(360.))
        .child(
            Input::new(&single)
                .prefix(Icon::new(IconName::Search))
                .suffix(Kbd::new("/"))
                .cleanable(true)
                .size(InputSize::Small),
        )
        .child(Input::new(&empty).size(InputSize::Large).width(px(80.)))
        .child(Input::new(&empty).disabled(true).bordered(false))
        .child(Input::new(&multi))
        .child(Input::new(&invalid).invalid(true))
        .child(
            NumberInput::new(&number)
                .kind(NumberKind::Float { decimals: 0 })
                .min(1.)
                .max(-1.)
                .step(0.)
                .steppers(true)
                .on_change(|_, _, _| {})
                .on_commit(|_, _, _| {}),
        )
        .child(
            NumberField::new("smoke-field-nan", f64::NAN).kind(NumberKind::Float { decimals: 3 }),
        )
        .child(
            NumberField::new("smoke-field-hex", -255.)
                .kind(NumberKind::Hex)
                .steppers(true),
        )
        .child(
            NumberField::new("smoke-field-int", 1e30)
                .disabled(true)
                .width(px(60.)),
        )
        .children(
            [
                ToggleState::Unselected,
                ToggleState::Indeterminate,
                ToggleState::Selected,
            ]
            .into_iter()
            .enumerate()
            .map(|(index, state)| {
                h_flex()
                    .child(
                        Checkbox::new(("smoke-checkbox", index), state)
                            .label("Check")
                            .label_size(LabelSize::Small)
                            .on_click(|_, _, _| {}),
                    )
                    .child(Checkbox::new(("smoke-checkbox-disabled", index), state).disabled(true))
                    .child(
                        Switch::new(("smoke-switch", index), state)
                            .label("Switch")
                            .on_click(|_, _, _| {}),
                    )
                    .child(
                        Switch::new(("smoke-switch-small", index), state)
                            .size(SwitchSize::Small)
                            .disabled(true),
                    )
            }),
        )
        .child(RadioGroup::new("smoke-radio-empty"))
        .child(
            RadioGroup::new("smoke-radio")
                .option("A")
                .option_with_description("B", LONG)
                .selected(7)
                .horizontal()
                .on_change(|_, _, _| {}),
        )
        .child(
            RadioGroup::new("smoke-radio-disabled")
                .option("A")
                .selected(0)
                .disabled(true),
        )
        .child(
            Slider::new("smoke-slider")
                .value(0.5)
                .on_change(|_, _, _| {}),
        )
        .child(
            Slider::new("smoke-slider-odd")
                .value(f32::NAN)
                .min(10.)
                .max(10.)
                .step(-1.)
                .on_commit(|_, _, _| {}),
        )
        .child(
            Slider::new("smoke-slider-ticks")
                .value(99.)
                .min(4.)
                .max(1.)
                .ticks(vec![1., 2., f32::INFINITY])
                .width(px(100.))
                .disabled(true),
        )
        .child(KeyCapture::new("smoke-key", "F1").on_capture(|_, _, _| {}))
        .child(
            KeyCapture::new("smoke-key-disabled", "")
                .disabled(true)
                .invalid(true),
        )
        .into_any_element()
}

fn select_items() -> Vec<SelectItem<u8>> {
    vec![
        SelectItem::new("Auto", 0).icon(IconName::Tune),
        SelectItem::new(LONG, 1),
        SelectItem::new("Off", 2).disabled(true),
    ]
}

fn menus() -> AnyElement {
    v_flex()
        .child(
            Select::new("smoke-select", "Auto")
                .items(select_items())
                .highlight(true)
                .searchable(true)
                .min_width(px(50.))
                .max_width(px(90.))
                .on_select(|_, _, _| {}),
        )
        .child(Select::<u8>::new("smoke-select-empty", "").on_select(|_, _, _| {}))
        .child(
            Select::new("smoke-select-unknown", "Not in the list")
                .items(select_items())
                .selected_index(99)
                .full_width()
                .menu_matches_trigger_width(true)
                .disabled(true),
        )
        .child(
            Select::new("smoke-select-icon", "Auto")
                .items(select_items())
                .trigger_icon(IconName::Language)
                .icon(IconName::ExpandMore)
                .tooltip("tip")
                .on_select(|_, _, _| {}),
        )
        .child(
            PopoverMenu::<ContextMenu>::new("smoke-popover-menu")
                .trigger(Button::new("smoke-popover-trigger", "Menu"))
                .menu(|_, _| None)
                .full_width(),
        )
        .child(PopoverMenu::<ContextMenu>::new("smoke-popover-bare").disabled(true))
        .child(
            right_click_menu::<ContextMenu>("smoke-right-click")
                .trigger_with(|open, _, _| {
                    Label::new(if open { "open" } else { "closed" }).into_any_element()
                })
                .menu(|window, cx| ContextMenu::build(window, cx, |menu, _, _| menu)),
        )
        .child(
            Popover::new("smoke-popover")
                .trigger(Chip::new("Popover"))
                .content(|_, _| Label::new("content").into_any_element())
                .padded(false),
        )
        .child(LanguageSelector::new())
        .child(ThemeToggle::new())
        .into_any_element()
}

fn containers(window: &mut Window, cx: &mut App) -> AnyElement {
    let scroll = window
        .use_keyed_state("smoke-scroll", cx, |_, _| ScrollHandle::new())
        .read(cx)
        .clone();
    let uniform = window
        .use_keyed_state("smoke-uniform", cx, |_, _| UniformListScrollHandle::new())
        .read(cx)
        .clone();
    let list = window
        .use_keyed_state("smoke-list", cx, |_, _| {
            ListState::new(40, ListAlignment::Top, px(100.))
        })
        .read(cx)
        .clone();
    v_flex()
        .w(px(700.))
        .child(Card::new())
        .child(
            Card::new()
                .title("Card")
                .header(Label::new("header"))
                .header_end(Chip::new("end"))
                .elevation(crate::theme::Elevation::Card)
                .translucent(true)
                .padding(CardPadding::None)
                .accent(Color::Warning)
                .child(Label::new("body")),
        )
        .child(
            Card::new()
                .padding(CardPadding::Compact)
                .full_height()
                .child(Label::new("x")),
        )
        .child(CollapsibleCard::new("smoke-card-closed", "Closed").child(Label::new("hidden")))
        .child(
            CollapsibleCard::new("smoke-card-open", "Open")
                .default_open(true)
                .changed(vec!["One".into(), "Two".into()])
                .header_end(Chip::new("end"))
                .on_toggle(|_, _, _| {})
                .child(
                    Section::new("Section")
                        .end_slot(Chip::new("3"))
                        .child(Label::new("row")),
                )
                .child(
                    Section::new("Plain").divider(false).header(
                        SectionHeader::new("Custom")
                            .uppercase()
                            .icon(IconName::Tune)
                            .color(Color::Muted),
                    ),
                ),
        )
        .child(CollapsibleCard::new("smoke-card-controlled", "Controlled").open(false))
        .child(ChangedBadge::new("smoke-changed", Vec::new()))
        .child(Disclosure::new("smoke-disclosure-open", true).on_toggle(|_, _, _| {}))
        .child(Disclosure::new("smoke-disclosure-closed", false).size(IconSize::Large))
        .child(
            PageHeader::new("Page")
                .info("tip", |_, _| {})
                .dirty(true)
                .start_slot(Icon::new(IconName::ArrowBack))
                .actions(Chip::new("action"))
                .save_discard(|_, _| {}, |_, _| {}),
        )
        .child(
            PageHeader::new("")
                .info_tooltip("tip")
                .save_discard(|_, _| {}, |_, _| {}),
        )
        .child(TwoColumn::new(Label::new("left"), Label::new("right")).id("smoke-two-a"))
        .child(
            TwoColumn::new(Label::new("left"), Label::new("right"))
                .id("smoke-two-b")
                .breakpoint(px(99999.))
                .ratio(6., 5.)
                .fill_height(true),
        )
        .child(
            div().h(px(60.)).child(
                h_resizable("smoke-resizable-h")
                    .child(
                        resizable_panel()
                            .size(px(50.))
                            .min(px(20.))
                            .max(px(10.))
                            .child(Label::new("a")),
                    )
                    .child(resizable_panel().flexible().child(Label::new("b")))
                    .child(resizable_panel())
                    .on_resize(|_, _, _| {}),
            ),
        )
        .child(
            div()
                .h(px(60.))
                .child(v_resizable("smoke-resizable-v").child(resizable_panel())),
        )
        .child(div().h(px(20.)).child(h_resizable("smoke-resizable-empty")))
        .child(
            div()
                .id("smoke-scroll-y")
                .h(px(40.))
                .child(div().h(px(400.)))
                .overflow_y_scrollbar(&scroll),
        )
        .child(
            div()
                .id("smoke-scroll-both")
                .size(px(40.))
                .child(div().size(px(400.)))
                .overflow_scrollbar(&scroll),
        )
        .child(
            div()
                .id("smoke-scroll-x")
                .w(px(40.))
                .child(div().w(px(400.)).h(px(4.)))
                .overflow_x_scrollbar(&scroll)
                .child(
                    Scrollbar::vertical(&scroll)
                        .id("smoke-always")
                        .always_visible(true),
                ),
        )
        .child(div().h(px(60.)).child(VirtualList::uniform(
            "smoke-uniform-list",
            5000,
            &uniform,
            |range, _, _| {
                range
                    .map(|index| ListItem::new(index).child(Label::new(index.to_string())))
                    .collect::<Vec<_>>()
            },
        )))
        .child(div().h(px(60.)).child(VirtualList::uniform(
            "smoke-uniform-empty",
            0,
            &uniform,
            |_, _, _| Vec::<gpui::Div>::new(),
        )))
        .child(
            div()
                .h(px(60.))
                .child(VirtualList::list("smoke-var-list", &list, |index, _, _| {
                    div()
                        .h(px(10. + (index % 5) as f32 * 7.))
                        .into_any_element()
                })),
        )
        .child(
            SortableList::new("smoke-sortable", vec!["a", "b", "c"])
                .drag_label(|item| SharedString::new_static(item))
                .row(|item, _, _, _| Label::new(*item).into_any_element())
                .on_reorder(|_, _, _, _| {}),
        )
        .child(SortableList::<u8>::new("smoke-sortable-empty", Vec::new()))
        .child(
            SortableList::new("smoke-sortable-static", vec![1, 2])
                .show_handle(false)
                .disabled(true),
        )
        .child(
            div().relative().h(px(120.)).child(
                Sheet::new("smoke-sheet")
                    .side(SheetSide::Left)
                    .width(px(5.))
                    .resizable(true)
                    .animate(false)
                    .on_dismiss(|_, _| {})
                    .on_resize(|_, _, _| {})
                    .child(Label::new("sheet")),
            ),
        )
        .child(
            div()
                .relative()
                .h(px(120.))
                .child(Sheet::new("smoke-sheet-right").scrim(false)),
        )
        .into_any_element()
}

fn navigation() -> AnyElement {
    let group = || {
        SidebarGroup::new("Config")
            .collapsible()
            .count(4)
            .on_toggle(|_, _, _| {})
            .item(
                SidebarItem::new("smoke-nav-a", IconName::Tune, "NAMS")
                    .active(true)
                    .tour_anchor("smoke-anchor")
                    .on_click(|_, _, _| {}),
            )
            .items([SidebarItem::new("smoke-nav-b", IconName::Mouse, LONG)])
    };
    h_flex()
        .h(px(300.))
        .child(
            Sidebar::new()
                .id("smoke-sidebar-labels")
                .icon_only(false)
                .header(Label::new("header"))
                .footer(Chip::new("v3.1.0"))
                .group(SidebarGroup::new("General"))
                .group(group().collapsed(false))
                .group(group().collapsed(true)),
        )
        .child(
            Sidebar::new()
                .id("smoke-sidebar-icons")
                .icon_only(true)
                .group(group().collapsed(false))
                .group(group().collapsed(true)),
        )
        .child(Sidebar::new().id("smoke-sidebar-empty"))
        .child(
            v_flex()
                .w(px(400.))
                .child(
                    TitleBar::new("Title")
                        .icon(IconName::SportsEsports)
                        .on_close(|_, _| {})
                        .child(Chip::new("slot")),
                )
                .child(TitleBar::new("").show_controls(false).title("Renamed"))
                .child(WindowControls::new())
                .child(
                    TabBar::new()
                        .start_slot(Chip::new("start"))
                        .end_slot(Chip::new("end"))
                        .tab(
                            Tab::new("smoke-tab-a")
                                .label("A")
                                .toggle_state(true)
                                .count(2),
                        )
                        .tabs([
                            Tab::new("smoke-tab-b")
                                .label("B")
                                .icon(IconName::Bolt)
                                .end_slot(Chip::new("x"))
                                .tooltip(Tooltip::text("tip"))
                                .on_click(|_, _, _| {}),
                            Tab::new("smoke-tab-c").disabled(true),
                        ]),
                )
                .child(TabBar::new().borderless())
                .child(EmptyState::new("Empty"))
                .child(
                    EmptyState::new("Empty")
                        .icon(IconName::Inventory2Outlined)
                        .description(LONG)
                        .action(Button::new("smoke-empty-action", "Add"))
                        .compact(),
                )
                .child(
                    KeyValueRow::new("Key", "Value")
                        .mono()
                        .value_color(Color::Success),
                )
                .child(KeyValueRow::with_slot("Key", Chip::new("slot")).label_width(px(40.)))
                .child(
                    DescriptionList::new()
                        .label_width(px(90.))
                        .row("A", "1")
                        .item(KeyValueRow::new("B", "2").label_width(px(10.))),
                )
                .child(DescriptionList::new())
                .child(StepDots::new(0, 0))
                .child(StepDots::new(6, 9))
                .child(StepButton::new("smoke-step", IconName::ArrowBack).on_click(|_, _, _| {}))
                .child(
                    StepButton::new("smoke-step-disabled", IconName::ArrowForward)
                        .disabled(true)
                        .tooltip(Tooltip::text("tip")),
                )
                .child(ListHeader::new("smoke-list-header", "Group").count(3))
                .child(
                    ListHeader::new("smoke-list-header-closed", "Group")
                        .collapsible(true)
                        .start_slot(Icon::new(IconName::Folder))
                        .end_slot(Chip::new("end"))
                        .on_toggle(|_, _, _| {}),
                )
                .child(ListHeader::new("smoke-list-header-open", "Group").collapsible(false))
                .child(
                    ListItem::new("smoke-list-item")
                        .start_slot(Icon::new(IconName::Folder))
                        .end_slot(Chip::new("end"))
                        .end_slot_on_hover(Chip::new("hover"))
                        .selected(true)
                        .spacing(ListItemSpacing::Sparse)
                        .indent_level(3)
                        .rounded(false)
                        .group_name("smoke-row")
                        .tooltip(Tooltip::text("tip"))
                        .on_click(|_, _, _| {})
                        .on_hover(|_, _, _| {})
                        .on_secondary_mouse_down(|_, _, _| {})
                        .child(Label::new("row")),
                )
                .child(
                    ListItem::new("smoke-list-item-disabled")
                        .disabled(true)
                        .spacing(ListItemSpacing::ExtraDense)
                        .on_click(|_, _, _| {}),
                )
                .child(ListSeparator::new()),
        )
        .into_any_element()
}

fn feedback() -> AnyElement {
    let severities = [
        Severity::Info,
        Severity::Accent,
        Severity::Success,
        Severity::Warning,
        Severity::Error,
    ];
    let borders = [
        BorderPosition::All,
        BorderPosition::Left,
        BorderPosition::Top,
        BorderPosition::Bottom,
        BorderPosition::None,
    ];
    let tones = [
        ChipTone::Neutral,
        ChipTone::Accent,
        ChipTone::Success,
        ChipTone::Warning,
        ChipTone::Error,
        ChipTone::Muted,
        ChipTone::Info,
    ];
    v_flex()
        .w(px(500.))
        .children(severities.into_iter().zip(borders).enumerate().map(
            |(index, (severity, border))| {
                Callout::new()
                    .id(("smoke-callout", index))
                    .severity(severity)
                    .title("Title")
                    .title_size(LabelSize::Small)
                    .description(LONG)
                    .description_slot(Label::new("slot"))
                    .actions_slot(Button::new(("smoke-callout-action", index), "Act"))
                    .border_position(border)
                    .pulse(index % 2 == 0)
                    .filled(index % 2 == 1)
                    .on_dismiss(|_, _| {})
                    .child(Label::new("child"))
            },
        ))
        .child(
            Callout::new()
                .no_icon()
                .icon(IconName::Bolt)
                .dismiss_action(Chip::new("x")),
        )
        .child(ConfigErrorBanner::new("nams.toml", "line 1\nline 2").id("smoke-config-error"))
        .children(tones.into_iter().enumerate().map(|(index, tone)| {
            h_flex()
                .child(
                    Chip::new("Chip")
                        .id(("smoke-chip", index))
                        .tone(tone)
                        .icon(IconName::Bolt)
                        .end_icon(IconName::ArrowOutward)
                        .size(ChipSize::Small)
                        .outlined(true)
                        .uppercase()
                        .truncate()
                        .tooltip_text("tip")
                        .on_click(|_, _, _| {}),
                )
                .child(Chip::new("Selected").tone(tone).selected(true))
                .child(CountBadge::new(index * 1000).tone(tone))
                .child(
                    Indicator::dot()
                        .tone(tone)
                        .pulse(index % 2 == 0)
                        .pulse_key(format!("pulse-{index}")),
                )
        }))
        .child(ApplyMarker::live().id("smoke-apply-live"))
        .child(ApplyMarker::restart().id("smoke-apply-restart"))
        .child(ApplyMarker::for_restart_required(true).id("smoke-apply-flag"))
        .child(Spinner::new().size(IconSize::XSmall).color(Color::Error))
        .child(ProgressBar::new("smoke-progress", 3., 10.).color(Color::Success))
        .child(ProgressBar::new("smoke-progress-nan", f32::NAN, 0.))
        .child(ProgressBar::indeterminate("smoke-progress-busy"))
        .child(LoadingState::new())
        .child(LoadingState::new().message("Loading").compact())
        .child(div().relative().h(px(80.)).child(BusyOverlay::new(LONG)))
        .into_any_element()
}

fn composites() -> AnyElement {
    v_flex()
        .w(px(520.))
        .child(
            SettingRow::new("smoke-setting-row", "Row")
                .description(LONG)
                .apply_marker(ApplyKind::Live)
                .changed(true)
                .leading(Icon::new(IconName::Tune))
                .control(Chip::new("control"))
                .value_label("1.0")
                .below(Label::new("below"))
                .error(SharedString::new_static("bad"))
                .on_click(|_, _, _| {}),
        )
        .child(
            SettingRow::new("smoke-setting-row-plain", "")
                .disabled(true)
                .hoverable(false),
        )
        .child(
            SettingSwitch::new("smoke-setting-switch", "Switch", true)
                .description(LONG)
                .apply_marker(ApplyKind::Restart)
                .changed(true)
                .on_change(|_, _, _| {}),
        )
        .child(SettingSwitch::new("smoke-setting-switch-disabled", "Switch", false).disabled(true))
        .child(
            SettingNumber::new("smoke-setting-number", "Number", 0.25)
                .kind(NumberKind::Float { decimals: 2 })
                .min(0.)
                .max(1.)
                .step(0.05)
                .steppers(true)
                .description(LONG)
                .apply_marker(ApplyKind::Live)
                .changed(true)
                .width(px(120.))
                .on_change(|_, _, _| {})
                .on_commit(|_, _, _| {}),
        )
        .child(SettingNumber::new("smoke-setting-number-disabled", "Number", -3.).disabled(true))
        .child(
            SettingSlider::new("smoke-setting-slider", "Slider", 2.5)
                .min(0.)
                .max(4.)
                .step(0.5)
                .decimals(1)
                .ticks(vec![0., 2., 4.])
                .description(LONG)
                .apply_marker(ApplyKind::Live)
                .changed(true)
                .on_change(|_, _, _| {})
                .on_commit(|_, _, _| {}),
        )
        .child(
            SettingSlider::new("smoke-setting-slider-odd", "Slider", f32::NAN)
                .min(5.)
                .max(-5.)
                .value_label("Vanilla")
                .disabled(true),
        )
        .child(
            SettingSelect::new("smoke-setting-select", "Select", "Auto")
                .items(select_items())
                .selected_index(0)
                .description(LONG)
                .apply_marker(ApplyKind::Restart)
                .changed(true)
                .searchable(true)
                .max_width(px(120.))
                .on_select(|_, _, _| {}),
        )
        .child(SettingSelect::<u8>::new("smoke-setting-select-empty", "Select", "").disabled(true))
        .child(ConfigSection::new("Section").child(Label::new("row")))
        .child(PreviewImage::new(
            "smoke-preview",
            asset_image("images/config/crosshair.webp"),
            "Preview",
        ))
        .child(ImageComparison::new(
            "smoke-comparison",
            asset_image("images/missing-a.jpg"),
            asset_image("images/missing-b.jpg"),
            "Before",
            "After",
        ))
        .child(
            Thumbnail::new("smoke-thumb", asset_image("images/missing.png"))
                .fit(gpui::ObjectFit::Contain)
                .rounded(false)
                .fallback(IconName::ImageNotSupported)
                .size(px(40.), px(30.))
                .lightbox(true)
                .tooltip("tip")
                .on_click(|_, _, _| {}),
        )
        .child(Thumbnail::new(
            "smoke-thumb-plain",
            asset_image("images/missing.png"),
        ))
        .child(
            DropZone::new("smoke-drop")
                .title("Drop here")
                .hint("or click")
                .icon(IconName::FileDownload)
                .accept(|path| path.is_dir())
                .footer("Installed to: nams/")
                .folder_label("Pick a folder")
                .on_drop_paths(|_, _, _| {})
                .on_browse_files(|_, _| {})
                .on_browse_folder(|_, _| {}),
        )
        .child(DropZone::new("smoke-drop-busy").busy(DropProgress::new("Installing").percent(140.)))
        .child(DropZone::new("smoke-drop-indeterminate").busy(DropProgress::new("Installing")))
        .child(
            DropZone::new("smoke-drop-disabled")
                .disabled(true)
                .on_browse_files(|_, _| {}),
        )
        .child(
            div().h(px(80.)).child(
                FrameViewport::new("smoke-viewport")
                    .frame(render_image_from_rgba(2, 2, vec![255; 16]))
                    .loading(true)
                    .loading_label("Rendering")
                    .fit(gpui::ObjectFit::Cover)
                    .on_orbit(|_, _, _| {})
                    .on_pan(|_, _, _| {})
                    .on_zoom(|_, _, _| {})
                    .overlay(Chip::new("overlay")),
            ),
        )
        .child(
            div()
                .h(px(40.))
                .child(FrameViewport::new("smoke-viewport-empty")),
        )
        .child(
            div().h(px(120.)).child(
                NierBackground::new()
                    .id("smoke-nier")
                    .child(Label::new("x")),
            ),
        )
        .child(
            div().h(px(40.)).child(
                NierBackground::new()
                    .id("smoke-nier-static")
                    .animate(false)
                    .grid(false)
                    .rings(false)
                    .borders(false),
            ),
        )
        .into_any_element()
}

fn styled_helpers(cx: &App) -> AnyElement {
    div()
        .h_flex()
        .v_flex()
        .elevation_2(cx)
        .elevation_3(cx)
        .surface(cx, crate::theme::Surface::Input)
        .font_ui(cx)
        .font_mono(cx)
        .child(div().elevation_1(cx))
        .children(
            [
                crate::theme::Surface::Primary,
                crate::theme::Surface::Secondary,
                crate::theme::Surface::Card,
                crate::theme::Surface::Light,
                crate::theme::Surface::Medium,
            ]
            .into_iter()
            .map(|surface| div().size(px(4.)).surface(cx, surface)),
        )
        .children(
            [
                Color::Default,
                Color::Secondary,
                Color::Muted,
                Color::Accent,
                Color::AccentSecondary,
                Color::Highlight,
                Color::Success,
                Color::Warning,
                Color::Error,
                Color::OnAccent,
                Color::OnDark,
                Color::Disabled,
                Color::Placeholder,
                Color::LogText,
                Color::LogDebug,
            ]
            .into_iter()
            .map(|color| Label::new("c").color(color)),
        )
        .into_any_element()
}

fn everything(views: &Rc<RefCell<Vec<AnyView>>>, window: &mut Window, cx: &mut App) -> AnyElement {
    let tour = TourOverlay::new(vec![
        TourStep::new("Anchored", LONG).anchor("smoke-anchor"),
        TourStep::new("Reading", LONG).body_slot(|_, _| Label::new("slot").into_any_element()),
    ])
    .id("smoke-tour")
    .on_step(|_, _, _| {})
    .on_finish(|_, _| {});
    div()
        .id("smoke-everything")
        .size_full()
        .overflow_y_scroll()
        .child(typography())
        .child(buttons())
        .child(inputs(window, cx))
        .child(menus())
        .child(containers(window, cx))
        .child(navigation())
        .child(feedback())
        .child(composites())
        .child(styled_helpers(cx))
        .child(TourOverlay::new(Vec::new()).id("smoke-tour-empty"))
        .child(tour)
        .children(views.borrow().iter().cloned())
        .into_any_element()
}

fn open(cx: &mut TestAppContext) -> (Rc<RefCell<Vec<AnyView>>>, &mut VisualTestContext) {
    let views = Rc::new(RefCell::new(Vec::new()));
    let render_views = views.clone();
    let (root, cx) = open_root(cx, move |window, cx| everything(&render_views, window, cx));
    cx.run_until_parked();
    let extra: Vec<AnyView> = cx.update(|window, cx| {
        let menu = ContextMenu::build(window, cx, |menu, _, _| {
            menu.header("Header")
                .entry("Entry", |_, _| {})
                .icon_entry("Icon", IconName::Bolt, |_, _| {})
                .toggleable_entry("On", true, |_, _| {})
                .toggleable_entry("Off", false, |_, _| {})
                .item(
                    ContextMenuEntry::new("Selected")
                        .selected(true)
                        .icon_color(Color::Error),
                )
                .item(
                    ContextMenuEntry::new("Disabled")
                        .disabled(true)
                        .icon(IconName::Close),
                )
                .separator()
                .link("Link", "https://example.invalid")
                .custom_entry(|_, _| Label::new("custom").into_any_element(), |_, _| {})
                .custom_row(|_, _| Label::new("row").into_any_element())
                .min_width(px(120.))
                .max_width(px(200.))
                .max_height(px(10.))
        });
        let select = cx.new(|cx| {
            SelectMenu::new(
                vec![
                    SelectMenuEntry {
                        label: "One".into(),
                        icon: Some(IconName::Bolt),
                        disabled: false,
                    },
                    SelectMenuEntry {
                        label: LONG.into(),
                        icon: None,
                        disabled: true,
                    },
                ],
                Some(1),
                true,
                |_, _, _| {},
                window,
                cx,
            )
        });
        let empty_select =
            cx.new(|cx| SelectMenu::new(Vec::new(), None, false, |_, _, _| {}, window, cx));
        let tooltips = vec![
            Tooltip::text("text")(window, cx),
            Tooltip::with_meta("title", LONG)(window, cx),
            Tooltip::simple("simple", cx),
            Tooltip::element(|_, _| Label::new("element").into_any_element())(window, cx),
            Tooltip::lines("header", vec!["a".into(), "b".into()])(window, cx),
            Tooltip::new("built").meta("meta").build()(window, cx),
        ];
        let mut views: Vec<AnyView> = vec![menu.into(), select.into(), empty_select.into()];
        views.extend(tooltips);
        views
    });
    *views.borrow_mut() = extra;
    root.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    (views, cx)
}

#[gpui::test]
fn every_component_renders_in_both_themes_and_all_breakpoints(cx: &mut TestAppContext) {
    let (_views, cx) = open(cx);
    for theme in ThemeId::ALL {
        for breakpoint in Breakpoint::ALL {
            cx.update(|_, cx| {
                set_theme(theme, cx);
                set_breakpoint_override(Some(breakpoint), cx);
            });
            cx.run_until_parked();
        }
    }
    cx.update(|_, cx| set_breakpoint_override(None, cx));
    for (width, height) in [
        (1300., 750.),
        (1000., 680.),
        (780., 560.),
        (600., 420.),
        (120., 90.),
    ] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.run_until_parked();
    }
}

#[gpui::test]
fn every_component_renders_in_every_locale_with_overlays_open(cx: &mut TestAppContext) {
    let (_views, cx) = open(cx);
    cx.update(|window, cx| {
        for severity in [Severity::Info, Severity::Warning, Severity::Error] {
            window.push_toast(Toast::new(LONG).severity(severity).icon(IconName::Bolt), cx);
        }
        window.push_toast(Toast::success("ok").id("fixed"), cx);
        window.push_toast(Toast::info("again").id("fixed"), cx);
        confirm(ConfirmOptions::new("Title", LONG), window, cx).detach();
        prompt_text(
            PromptOptions::new("Title")
                .label(LONG)
                .max_len(4)
                .allow_empty(true),
            window,
            cx,
        )
        .detach();
        pick_one(
            PickOptions::new("Pick", vec![PickItem::new("A", 1).icon(IconName::Bolt)]).body(LONG),
            window,
            cx,
        )
        .detach();
        pick_many(
            PickOptions::new(
                "Pick",
                vec![
                    PickItem::new("A", 1).description("first").selected(true),
                    PickItem::new("B", 2).disabled(true),
                ],
            )
            .confirm_label("Go")
            .cancel_label("Stop"),
            window,
            cx,
        )
        .detach();
        Lightbox::open(asset_image("images/config/crosshair.webp"), window, cx);
    });
    for locale in Locale::ALL {
        cx.update(|_, cx| I18n::set(locale, cx));
        cx.run_until_parked();
    }
    cx.simulate_keystrokes("tab tab shift-tab escape escape escape escape escape");
    cx.simulate_click(point(px(30.), px(30.)), Modifiers::default());
    cx.update(|window, cx| assert!(!window.has_modal(cx)));
}

#[gpui::test]
fn clicking_through_the_page_never_panics(cx: &mut TestAppContext) {
    let (_views, cx) = open(cx);
    for y in (5..700).step_by(53) {
        for x in (5..1000).step_by(131) {
            cx.simulate_click(point(px(x as f32), px(y as f32)), Modifiers::default());
        }
        cx.simulate_keystrokes("escape");
    }
    cx.simulate_keystrokes("tab enter space escape");
}
