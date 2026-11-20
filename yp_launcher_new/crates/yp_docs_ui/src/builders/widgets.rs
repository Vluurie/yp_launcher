use gpui::{
    AnyElement, App, ClickEvent, Context, Div, Entity, IntoElement, Pixels, ScrollHandle, SharedString,
    Subscription, Window, div, prelude::*, px,
};
use yp_builders::outfit::{MAX_SHOP_STATE, SHOP_NAMES, SHOP_STATE_KEYS};
use yp_i18n::Strings;
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, ButtonStyle, InputEvent, SelectItem, Sizes, TintColor, Tooltip};

use super::forms::{ShopChoice, ThumbnailChoice};
use crate::markdown::{Look, RichBlock, code_runs, code_scroller};
use crate::syntax::{Language, tokenize};

pub(crate) const VALUE_WIDTH: f32 = 64.;
pub(crate) const WIDE_VALUE_WIDTH: f32 = 76.;
pub(crate) const ID_FIELD_WIDTH: f32 = 140.;
const SHOP_SELECT_WIDTH: f32 = 230.;

pub(crate) fn bound_input<V: 'static>(
    placeholder: impl Into<SharedString>,
    initial: &str,
    rows: usize,
    mono: bool,
    apply: impl Fn(&mut V, &str, &mut Window, &mut Context<V>) + 'static,
    window: &mut Window,
    cx: &mut Context<V>,
) -> (Entity<InputState>, Subscription) {
    let placeholder = placeholder.into();
    let initial = initial.to_owned();
    let input = cx.new(|cx| {
        let mut state = InputState::new(window, cx).placeholder(placeholder);
        if rows > 1 {
            state = state.multi_line(rows).max_rows(rows.max(6));
        }
        if mono {
            state = state.mono();
        }
        state.default_value(initial)
    });
    let subscription = cx.subscribe_in(
        &input,
        window,
        move |this, input, event: &InputEvent, window, cx| {
            if matches!(event, InputEvent::Change) {
                let value = input.read(cx).value().to_owned();
                apply(this, &value, window, cx);
                cx.notify();
            }
        },
    );
    (input, subscription)
}

pub(crate) fn field_label(text: impl Into<SharedString>) -> Label {
    Label::new(text)
        .size(LabelSize::XSmall)
        .color(Color::Secondary)
}

pub(crate) fn text_field(
    label: impl Into<SharedString>,
    input: &Entity<InputState>,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    v_flex()
        .w_full()
        .gap(sizes.spacing_sm)
        .child(field_label(label))
        .child(Input::new(input).size(InputSize::Small))
}

pub(crate) fn read_only_field(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    let colors = cx.theme().colors;
    v_flex()
        .w_full()
        .gap(sizes.spacing_sm)
        .child(field_label(label))
        .child(
            v_flex()
                .w_full()
                .px(px(8.))
                .py(px(4.))
                .rounded(Sizes::ROW_RADIUS)
                .border_1()
                .border_color(colors.border_light)
                .bg(colors.surface_light)
                .child(
                    Label::new(value)
                        .size(LabelSize::Small)
                        .color(Color::Accent)
                        .semibold()
                        .mono(),
                ),
        )
}

pub(crate) fn section_title(text: impl Into<SharedString>) -> Label {
    Label::new(text)
        .size(LabelSize::XSmall)
        .color(Color::Secondary)
        .bold()
}

pub(crate) fn hint(text: impl Into<SharedString>) -> Label {
    Label::new(text)
        .size(LabelSize::XSmall)
        .color(Color::Muted)
        .line_height(1.4)
}

pub(crate) type ClickFn = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

pub(crate) fn click<V: 'static>(
    cx: &Context<V>,
    handler: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
) -> ClickFn {
    Box::new(cx.listener(move |this, _: &ClickEvent, window, cx| {
        handler(this, window, cx);
        cx.notify();
    }))
}

pub(crate) fn stepper(
    id: impl Into<SharedString>,
    value: impl Into<SharedString>,
    width: Pixels,
    decrease: Option<ClickFn>,
    increase: Option<ClickFn>,
    cx: &App,
) -> Div {
    let id = id.into();
    let strings = cx.t();
    let minus = StepButton::new(
        SharedString::from(format!("{id}-decrease")),
        IconName::Remove,
    )
    .disabled(decrease.is_none())
    .tooltip(Tooltip::text(strings.ui_decrease()));
    let plus = StepButton::new(SharedString::from(format!("{id}-increase")), IconName::Add)
        .disabled(increase.is_none())
        .tooltip(Tooltip::text(strings.ui_increase()));
    h_flex()
        .flex_none()
        .child(match decrease {
            Some(handler) => minus.on_click(handler),
            None => minus,
        })
        .child(
            div().w(width).flex().justify_center().child(
                Label::new(value)
                    .size(LabelSize::Small)
                    .color(Color::Accent)
                    .semibold()
                    .mono(),
            ),
        )
        .child(match increase {
            Some(handler) => plus.on_click(handler),
            None => plus,
        })
}

pub(crate) fn remove_button(id: impl Into<SharedString>, handler: ClickFn, cx: &App) -> StepButton {
    StepButton::new(id.into(), IconName::Close)
        .tooltip(Tooltip::text(cx.t().thumbnail_remove()))
        .on_click(handler)
}

pub(crate) fn labelled_row(
    label: impl Into<SharedString>,
    label_color: Color,
    control: impl IntoElement,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    h_flex()
        .w_full()
        .gap(sizes.spacing_lg)
        .py(sizes.spacing_sm)
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(Label::new(label).size(LabelSize::Small).color(label_color)),
        )
        .child(control)
}

pub(crate) fn switch_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    hint_text: Option<&'static str>,
    on: bool,
    on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    h_flex()
        .w_full()
        .items_start()
        .gap(sizes.spacing_lg)
        .py(sizes.spacing_sm)
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(Label::new(label).size(LabelSize::Small))
                .children(hint_text.map(hint)),
        )
        .child(
            Switch::new(id.into(), on)
                .on_click(move |state, window, cx| on_change(state.selected(), window, cx)),
        )
}

pub(crate) fn toml_preview(
    id: impl Into<SharedString>,
    file_name: impl Into<SharedString>,
    content: &str,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = id.into();
    let sizes = window.sizes(cx);
    let colors = cx.theme().colors;
    let look = Look::new(false, window, cx);
    let code = content.trim_end();
    let tokens = tokenize(Language::Toml, code);
    let runs = code_runs(code, &tokens, Some(Language::Toml), &look);
    let text = SharedString::from(code.to_owned());
    let group = SharedString::from(format!("{id}-group"));
    let scroll_id = SharedString::from(format!("{id}-scroll"));
    let handle = window
        .use_keyed_state(scroll_id.clone(), cx, |_, _| ScrollHandle::new())
        .read(cx)
        .clone();
    let scroller = code_scroller(
        scroll_id,
        &handle,
        sizes.padding_md,
        sizes.font_xs,
        &look,
        RichBlock::new(
            SharedString::from(format!("{id}-text")),
            text.clone(),
            runs,
        ),
    );
    v_flex()
        .group(group.clone())
        .relative()
        .w_full()
        .rounded(Sizes::ROW_RADIUS)
        .border_1()
        .border_color(colors.border_light)
        .bg(colors.surface_light)
        .child(
            h_flex()
                .w_full()
                .px(sizes.padding_md)
                .py(sizes.padding_xs)
                .rounded_t(Sizes::ROW_RADIUS)
                .bg(colors.surface_medium)
                .child(
                    h_flex().flex_1().min_w_0().child(
                        Label::new(file_name)
                            .size(LabelSize::XSmall)
                            .color(Color::Accent)
                            .mono()
                            .truncate(),
                    ),
                )
                .child(
                    div()
                        .visible_on_hover(group)
                        .child(CopyButton::new(SharedString::from(format!("{id}-copy")), text)),
                ),
        )
        .child(scroller)
        .into_any_element()
}

pub(crate) fn preview_caption(
    text: impl Into<SharedString>,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    h_flex()
        .gap(sizes.spacing_md)
        .child(
            Icon::new(IconName::DescriptionOutlined)
                .size(IconSize::Small)
                .color(Color::Muted),
        )
        .child(Label::new(text).size(LabelSize::XSmall).color(Color::Muted))
}

pub(crate) fn tint_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    tint: TintColor,
) -> Button {
    Button::new(id.into(), label).style(ButtonStyle::Tinted(tint))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn texture_row(
    id: &'static str,
    label: impl Into<SharedString>,
    empty_hint: impl Into<SharedString>,
    chosen: Option<(&ThumbnailChoice, SharedString)>,
    on_pick: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_remove: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    let strings = cx.t();
    let has = chosen.is_some();
    let detail = match chosen {
        Some((choice, note)) => SharedString::from(format!(
            "{}\u{d7}{}  \u{b7}  {note}",
            choice.width, choice.height
        )),
        None => empty_hint.into(),
    };
    let tone = if has { Color::Success } else { Color::Muted };
    h_flex()
        .w_full()
        .gap(sizes.spacing_lg)
        .py(sizes.spacing_sm)
        .child(
            Icon::new(if has {
                IconName::Image
            } else {
                IconName::ImageOutlined
            })
            .size(IconSize::Medium)
            .color(tone),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(Label::new(label).size(LabelSize::Small))
                .child(
                    Label::new(detail)
                        .size(LabelSize::XSmall)
                        .color(tone)
                        .line_height(1.3)
                        .line_clamp(2),
                ),
        )
        .child(
            tint_button(
                format!("{id}-pick"),
                if has {
                    strings.thumbnail_change()
                } else {
                    strings.thumbnail_open()
                },
                TintColor::Accent,
            )
            .on_click(on_pick),
        )
        .when(has, |this| {
            this.child(
                tint_button(
                    format!("{id}-remove"),
                    strings.thumbnail_remove(),
                    TintColor::Muted,
                )
                .on_click(on_remove),
            )
        })
}

pub(crate) fn shop_state_label(strings: &Strings, state: i64) -> &'static str {
    let key = usize::try_from(state)
        .ok()
        .and_then(|index| SHOP_STATE_KEYS.get(index))
        .copied()
        .unwrap_or_default();
    match key {
        "routeABeforeRuinsCollapse" => strings.shop_state_route_a_before_ruins_collapse(),
        "routeAAfterRuinsCollapse" => strings.shop_state_route_a_after_ruins_collapse(),
        "routeBBeforeRuinsCollapse" => strings.shop_state_route_b_before_ruins_collapse(),
        "routeBAfterRuinsCollapse" => strings.shop_state_route_b_after_ruins_collapse(),
        "routeC" => strings.shop_state_route_c(),
        "endingCOrD" => strings.shop_state_ending_c_or_d(),
        _ => strings.shop_state_ending_c_and_d(),
    }
}

pub(crate) fn shop_select(
    id: &'static str,
    shop: &ShopChoice,
    on_select: impl Fn(&String, &mut Window, &mut App) + 'static,
) -> Select<String> {
    Select::new(id, SharedString::from(shop.shop_name.clone()))
        .items(
            SHOP_NAMES
                .iter()
                .map(|name| SelectItem::new(*name, (*name).to_owned())),
        )
        .min_width(px(SHOP_SELECT_WIDTH))
        .max_width(px(SHOP_SELECT_WIDTH))
        .menu_matches_trigger_width(true)
        .on_select(on_select)
}

pub(crate) fn shop_states(
    id: &'static str,
    shop: &ShopChoice,
    on_toggle: impl Fn(i64, &mut Window, &mut App) + 'static,
    on_toggle_all: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    let strings = cx.t();
    let on_toggle = std::rc::Rc::new(on_toggle);
    v_flex()
        .w_full()
        .gap(sizes.spacing_sm)
        .child(
            Label::new(strings.outfit_builder_shop_state())
                .size(LabelSize::Small)
                .color(Color::Secondary),
        )
        .child(hint(strings.outfit_builder_shop_state_hint()))
        .children((0..=MAX_SHOP_STATE).map(|state| {
            let handler = on_toggle.clone();
            Checkbox::new(
                SharedString::from(format!("{id}-state-{state}")),
                shop.states.contains(&state),
            )
            .label(SharedString::from(format!(
                "{state}  {}",
                shop_state_label(strings, state)
            )))
            .label_size(LabelSize::XSmall)
            .on_click(move |_, window, cx| handler(state, window, cx))
        }))
        .child(
            Link::new(
                SharedString::from(format!("{id}-all")),
                strings.outfit_builder_shop_state_all(),
            )
            .size(LabelSize::XSmall)
            .color(Color::Secondary)
            .hover_color(Color::Accent)
            .weight(gpui::FontWeight::BOLD)
            .underline()
            .on_click(on_toggle_all),
        )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_row(
    id: &'static str,
    create_label: &'static str,
    open_label: &'static str,
    busy: bool,
    written: bool,
    on_create: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_open: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    window: &Window,
    cx: &App,
) -> Div {
    let sizes = window.sizes(cx);
    h_flex()
        .gap(sizes.spacing_lg)
        .child(
            tint_button(format!("{id}-create"), create_label, TintColor::Success)
                .loading(busy)
                .disabled(busy)
                .on_click(on_create),
        )
        .when(written, |this| {
            this.child(
                tint_button(format!("{id}-open"), open_label, TintColor::Accent)
                    .controller_support(false)
                    .on_click(on_open),
            )
        })
}
