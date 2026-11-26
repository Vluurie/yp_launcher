use std::rc::Rc;

use gpui::{
    AnyElement, AnyView, App, ClickEvent, Corners, CursorStyle, DefiniteLength, ElementId,
    FocusHandle, FontWeight, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, SharedString,
    Window, prelude::*, px, relative,
};

use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_TIGHT, LabelSize};
use super::progress::Spinner;
use super::text::Text;
use super::util::{ControllerMode, KeyboardFocus};
use super::tooltip::Tooltip;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, ButtonHover, Color, ResponsiveExt, Sizes};
use crate::traits::{
    ButtonCommon, ClickHandler, Clickable, Disableable, FixedWidth, SelectableButton, ToggleState,
    Toggleable, TooltipBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TintColor {
    #[default]
    Accent,
    Success,
    Warning,
    Error,
    Muted,
    Primary,
}

impl TintColor {
    pub fn color(self) -> Color {
        match self {
            TintColor::Accent => Color::Accent,
            TintColor::Success => Color::Success,
            TintColor::Warning => Color::Warning,
            TintColor::Error => Color::Error,
            TintColor::Muted => Color::Muted,
            TintColor::Primary => Color::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonStyle {
    Filled,
    Solid(TintColor),
    Tinted(TintColor),
    Outlined,
    #[default]
    Subtle,
    Transparent,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonColors {
    pub fg: Color,
    pub hover_fg: Color,
    pub background: Hsla,
    pub hover_background: Hsla,
    pub active_background: Hsla,
    pub border: Hsla,
    pub hover_border: Hsla,
}

impl ButtonColors {
    pub fn inverts_on_hover(&self) -> bool {
        self.hover_fg != self.fg && self.hover_background.a >= 1.
    }
}

impl ButtonStyle {
    pub fn colors(self, cx: &App) -> ButtonColors {
        let theme = cx.theme();
        let colors = &theme.colors;
        let alpha = &theme.alpha;
        let clear = colors.transparent;
        match self {
            ButtonStyle::Filled => ButtonColors {
                fg: Color::OnAccent,
                hover_fg: Color::OnAccent,
                background: colors.button_background,
                hover_background: colors.accent_highlight,
                active_background: colors.accent_secondary,
                border: clear,
                hover_border: clear,
            },
            ButtonStyle::Solid(tint) => ButtonColors {
                fg: Color::OnAccent,
                hover_fg: Color::OnAccent,
                background: tint.color().color(cx),
                hover_background: colors.accent_primary,
                active_background: colors.border_medium,
                border: clear,
                hover_border: clear,
            },
            ButtonStyle::Tinted(tint) => {
                let fg = tint.color();
                let base = fg.color(cx);
                match theme.style.button_hover {
                    ButtonHover::Tint => ButtonColors {
                        fg,
                        hover_fg: fg,
                        background: clear,
                        hover_background: base.opacity(alpha.hover),
                        active_background: base.opacity(alpha.soft),
                        border: base,
                        hover_border: base,
                    },
                    ButtonHover::Invert => ButtonColors {
                        fg,
                        hover_fg: Color::OnAccent,
                        background: clear,
                        hover_background: base,
                        active_background: base.opacity(alpha.glass),
                        border: base,
                        hover_border: base,
                    },
                }
            }
            ButtonStyle::Outlined => ButtonColors {
                fg: Color::Default,
                hover_fg: Color::Default,
                background: clear,
                hover_background: colors.surface_light,
                active_background: colors.accent_primary.opacity(alpha.hover),
                border: colors.border_medium,
                hover_border: colors.accent_primary.opacity(alpha.half),
            },
            ButtonStyle::Subtle => ButtonColors {
                fg: Color::Accent,
                hover_fg: Color::Accent,
                background: clear,
                hover_background: colors.accent_primary.opacity(alpha.subtle),
                active_background: colors.accent_primary.opacity(alpha.line),
                border: clear,
                hover_border: clear,
            },
            ButtonStyle::Transparent => ButtonColors {
                fg: Color::Muted,
                hover_fg: Color::Accent,
                background: clear,
                hover_background: clear,
                active_background: colors.accent_primary.opacity(alpha.faint),
                border: clear,
                hover_border: clear,
            },
            ButtonStyle::Link => ButtonColors {
                fg: Color::Accent,
                hover_fg: Color::Highlight,
                background: clear,
                hover_background: clear,
                active_background: clear,
                border: clear,
                hover_border: clear,
            },
        }
    }

    fn selected_colors(cx: &App) -> ButtonColors {
        let theme = cx.theme();
        let accent = theme.colors.accent_primary;
        ButtonColors {
            fg: Color::Accent,
            hover_fg: Color::Accent,
            background: accent.opacity(theme.alpha.hover),
            hover_background: accent.opacity(theme.alpha.line),
            active_background: accent.opacity(theme.alpha.soft),
            border: accent.opacity(theme.alpha.vignette),
            hover_border: accent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonSize {
    Compact,
    #[default]
    Default,
    Medium,
    Large,
    Hero,
}

impl ButtonSize {
    pub fn padding(self, sizes: &Sizes) -> (Pixels, Pixels) {
        match self {
            ButtonSize::Compact => (px(8.), px(2.)),
            ButtonSize::Default => (px(10.), px(4.)),
            ButtonSize::Medium => (px(12.), px(6.)),
            ButtonSize::Large => (px(16.), px(8.)),
            ButtonSize::Hero => (sizes.padding_lg, sizes.padding_sm),
        }
    }

    pub fn label_size(self) -> LabelSize {
        match self {
            ButtonSize::Compact => LabelSize::XSmall,
            ButtonSize::Default => LabelSize::Small,
            ButtonSize::Medium => LabelSize::Default,
            ButtonSize::Large => LabelSize::Large,
            ButtonSize::Hero => LabelSize::XLarge,
        }
    }

    pub fn icon_size(self) -> IconSize {
        match self {
            ButtonSize::Compact => IconSize::Small,
            ButtonSize::Default => IconSize::Small,
            ButtonSize::Medium => IconSize::Medium,
            ButtonSize::Large => IconSize::Medium,
            ButtonSize::Hero => IconSize::Large,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ButtonRounding {
    #[default]
    All,
    Left,
    Right,
    None,
}

pub struct ButtonLike {
    id: ElementId,
    accessible_label: Option<SharedString>,
    style: ButtonStyle,
    selected_style: Option<ButtonStyle>,
    size: ButtonSize,
    rounding: ButtonRounding,
    radius: Option<Pixels>,
    disabled: bool,
    controller_supported: bool,
    selected: bool,
    width: Option<DefiniteLength>,
    full_width: bool,
    padding: Option<(Pixels, Pixels)>,
    cursor_style: CursorStyle,
    on_click: Option<ClickHandler>,
    tooltip: Option<TooltipBuilder>,
    tab_index: Option<isize>,
    focus_handle: Option<FocusHandle>,
    group: Option<SharedString>,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(ButtonLike);

impl ButtonLike {
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            accessible_label: None,
            style: ButtonStyle::default(),
            selected_style: None,
            size: ButtonSize::default(),
            rounding: ButtonRounding::All,
            radius: None,
            disabled: false,
            controller_supported: true,
            selected: false,
            width: None,
            full_width: false,
            padding: None,
            cursor_style: CursorStyle::PointingHand,
            on_click: None,
            tooltip: None,
            tab_index: None,
            focus_handle: None,
            group: None,
            children: Vec::new(),
        }
    }

    pub fn new_rounded_left(id: impl Into<ElementId>) -> Self {
        Self::new(id).rounding(ButtonRounding::Left)
    }

    pub fn new_rounded_right(id: impl Into<ElementId>) -> Self {
        Self::new(id).rounding(ButtonRounding::Right)
    }

    pub fn new_rounded_all(id: impl Into<ElementId>) -> Self {
        Self::new(id).rounding(ButtonRounding::All)
    }

    pub fn new_square(id: impl Into<ElementId>) -> Self {
        Self::new(id).rounding(ButtonRounding::None)
    }

    fn rounding(mut self, rounding: ButtonRounding) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn radius(mut self, radius: Pixels) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn padding(mut self, horizontal: Pixels, vertical: Pixels) -> Self {
        self.padding = Some((horizontal, vertical));
        self
    }

    pub fn group_name(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn hover_group(id: &ElementId) -> SharedString {
        SharedString::from(format!("yp-button-{id}"))
    }
}

impl ParentElement for ButtonLike {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Clickable for ButtonLike {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = cursor_style;
        self
    }
}

impl Disableable for ButtonLike {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Toggleable for ButtonLike {
    fn toggle_state(mut self, state: impl Into<ToggleState>) -> Self {
        self.selected = state.into().selected();
        self
    }
}

impl SelectableButton for ButtonLike {
    fn selected_style(mut self, style: ButtonStyle) -> Self {
        self.selected_style = Some(style);
        self
    }
}

impl ButtonCommon for ButtonLike {
    fn controller_support(mut self, supported: bool) -> Self {
        self.controller_supported = supported;
        self
    }

    fn id(&self) -> &ElementId {
        &self.id
    }

    fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Rc::new(tooltip));
        self
    }

    fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = Some(tab_index);
        self
    }

    fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }
}

impl FixedWidth for ButtonLike {
    fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
}

pub(crate) fn resolved_button_colors(
    style: ButtonStyle,
    selected_style: Option<ButtonStyle>,
    selected: bool,
    cx: &App,
) -> ButtonColors {
    if !selected {
        return style.colors(cx);
    }
    match selected_style {
        Some(selected_style) => selected_style.colors(cx),
        None => ButtonStyle::selected_colors(cx),
    }
}

impl RenderOnce for ButtonLike {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let theme = cx.theme().clone();
        let colors = resolved_button_colors(self.style, self.selected_style, self.selected, cx);
        let (padding_h, padding_v) = self.padding.unwrap_or_else(|| self.size.padding(&sizes));
        let menu = theme.id == crate::ThemeId::NierNew;
        let radius = self
            .radius
            .unwrap_or(if menu { px(0.) } else { Sizes::CONTROL_RADIUS });
        let corners = match self.rounding {
            ButtonRounding::All => Corners::all(radius),
            ButtonRounding::Left => Corners {
                top_left: radius,
                bottom_left: radius,
                top_right: px(0.),
                bottom_right: px(0.),
            },
            ButtonRounding::Right => Corners {
                top_left: px(0.),
                bottom_left: px(0.),
                top_right: radius,
                bottom_right: radius,
            },
            ButtonRounding::None => Corners::all(px(0.)),
        };
        let fg = colors.fg.color(cx);
        let hover_fg = colors.hover_fg.color(cx);
        let is_link = self.style == ButtonStyle::Link;
        let is_hero = self.size == ButtonSize::Hero;
        let controller_blocked = ControllerMode::is_active(cx) && !self.controller_supported;
        let disabled = self.disabled || controller_blocked;
        let on_click = self.on_click;
        let group = self
            .group
            .unwrap_or_else(|| ButtonLike::hover_group(&self.id));

        let sweeps = menu
            && matches!(self.style, ButtonStyle::Filled | ButtonStyle::Solid(_))
            && !disabled
            && !yp_nier_ui::ReducedMotion::enabled(cx);
        let fill = sweeps
            .then(|| super::util::menu_hover_fill(self.id.clone(), colors.hover_background, None, window, cx));
        h_flex()
            .id(self.id)
            .relative()
            .when(disabled || on_click.is_some(), |el| {
                el.role(gpui::Role::Button)
                    .when_some(self.accessible_label, |el, label| el.aria_label(label))
                    .when(disabled, |el| el.aria_description(cx.t().mod_disabled()))
            })
            .group(group)
            .map(|el| {
                if menu {
                    el.min_w_0().max_w_full()
                } else {
                    el.flex_none()
                }
            })
            .justify_center()
            .gap(sizes.spacing_md)
            .px(padding_h)
            .py(padding_v)
            .rounded_tl(corners.top_left)
            .rounded_tr(corners.top_right)
            .rounded_bl(corners.bottom_left)
            .rounded_br(corners.bottom_right)
            .border_1()
            .border_color(colors.border)
            .bg(colors.background)
            .text_color(fg)
            .text_size(self.size.label_size().pixels(&sizes))
            .line_height(relative(LINE_HEIGHT_TIGHT))
            .font_weight(if menu {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::BOLD
            })
            .when(is_hero, |this| {
                this.w(sizes.play_button_width).h(sizes.play_button_height)
            })
            .when_some(self.width, |this, width| this.w(width))
            .when(self.full_width, |this| this.w_full())
            .when_some(self.focus_handle.as_ref(), |this, handle| {
                this.track_focus(&handle.clone().tab_stop(!disabled && handle.tab_stop))
            })
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor(self.cursor_style)
                    .hover(move |style| {
                        let style = style
                            .bg(if sweeps {
                                colors.background
                            } else {
                                colors.hover_background
                            })
                            .border_color(colors.hover_border)
                            .text_color(hover_fg);
                        if is_link {
                            style.underline().text_decoration_color(hover_fg)
                        } else {
                            style
                        }
                    })
                    .active(move |style| style.bg(colors.active_background))
                    .when_some(on_click, |this, handler| {
                        this.keyboard_activate(cx)
                            .on_click(move |event, window, cx| {
                                cx.stop_propagation();
                                handler(event, window, cx);
                            })
                    })
            })
            .when_some(self.tab_index.filter(|_| !disabled), |this, index| this.tab_index(index))
            .when_some(self.tooltip.filter(|_| !controller_blocked), |this, tooltip| {
                this.tooltip(move |window, cx| tooltip(window, cx))
            })
            .when(controller_blocked, |this| {
                this.tooltip(Tooltip::text(cx.t().controller_requires_keyboard()))
            })
            .children(fill)
            .children(self.children)
    }
}

pub struct Button {
    base: ButtonLike,
    label: SharedString,
    selected_label: Option<SharedString>,
    start_icon: Option<IconName>,
    end_icon: Option<IconName>,
    icon_size: Option<IconSize>,
    loading: bool,
    color: Option<Color>,
    label_size: Option<LabelSize>,
    weight: Option<FontWeight>,
    truncate: bool,
}

crate::impl_into_element!(Button);

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            base: ButtonLike::new(id),
            label: label.into(),
            selected_label: None,
            start_icon: None,
            end_icon: None,
            icon_size: None,
            loading: false,
            color: None,
            label_size: None,
            weight: None,
            truncate: false,
        }
    }

    pub fn start_icon(mut self, icon: impl Into<Option<IconName>>) -> Self {
        self.start_icon = icon.into();
        self
    }

    pub fn end_icon(mut self, icon: impl Into<Option<IconName>>) -> Self {
        self.end_icon = icon.into();
        self
    }

    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.icon_size = Some(size);
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn label_size(mut self, size: LabelSize) -> Self {
        self.label_size = Some(size);
        self
    }

    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn selected_label(mut self, label: impl Into<SharedString>) -> Self {
        self.selected_label = Some(label.into());
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    pub fn radius(mut self, radius: Pixels) -> Self {
        self.base = self.base.radius(radius);
        self
    }
}

impl Clickable for Button {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.base = self.base.on_click(handler);
        self
    }

    fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.base = self.base.cursor_style(cursor_style);
        self
    }
}

impl Disableable for Button {
    fn disabled(mut self, disabled: bool) -> Self {
        self.base = self.base.disabled(disabled);
        self
    }
}

impl Toggleable for Button {
    fn toggle_state(mut self, state: impl Into<ToggleState>) -> Self {
        self.base = self.base.toggle_state(state);
        self
    }
}

impl SelectableButton for Button {
    fn selected_style(mut self, style: ButtonStyle) -> Self {
        self.base = self.base.selected_style(style);
        self
    }
}

impl ButtonCommon for Button {
    fn controller_support(mut self, supported: bool) -> Self {
        self.base = self.base.controller_support(supported);
        self
    }

    fn id(&self) -> &ElementId {
        self.base.id()
    }

    fn style(mut self, style: ButtonStyle) -> Self {
        self.base = self.base.style(style);
        self
    }

    fn size(mut self, size: ButtonSize) -> Self {
        self.base = self.base.size(size);
        self
    }

    fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.base = self.base.tooltip(tooltip);
        self
    }

    fn tab_index(mut self, tab_index: isize) -> Self {
        self.base = self.base.tab_index(tab_index);
        self
    }

    fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.base = self.base.track_focus(focus_handle);
        self
    }
}

impl FixedWidth for Button {
    fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.base = self.base.width(width);
        self
    }

    fn full_width(mut self) -> Self {
        self.base = self.base.full_width();
        self
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let colors = resolved_button_colors(
            self.base.style,
            self.base.selected_style,
            self.base.selected,
            cx,
        );
        let fg = self.color.unwrap_or(colors.fg);
        let hover_fg = if self.color.is_some() && !colors.inverts_on_hover() {
            fg
        } else {
            colors.hover_fg
        };
        let group = ButtonLike::hover_group(&self.base.id);
        let icon_size = self.icon_size.unwrap_or_else(|| self.base.size.icon_size());
        let label = match (self.base.selected, self.selected_label) {
            (true, Some(selected_label)) => selected_label,
            _ => self.label,
        };
        let label_color = self.color.map(|color| color.color(cx));
        let label_size = self.label_size.map(|size| size.pixels(&sizes));
        let loading = self.loading;
        let disabled = self.base.disabled || loading;
        let label_hover = (!disabled).then(|| hover_fg.color(cx));
        let label_group = group.clone();
        let make_icon = |icon: IconName| {
            Icon::new(icon)
                .size(icon_size)
                .color(fg)
                .group_hover_color(group.clone(), hover_fg)
        };

        self.base
            .accessible_label(label.clone())
            .disabled(disabled)
            .group_name(group.clone())
            .when(loading, |this| {
                this.child(Spinner::new().size(icon_size).color(fg))
            })
            .when_some(self.start_icon.filter(|_| !loading), |this, icon| {
                this.child(make_icon(icon))
            })
            .child(
                v_flex()
                    .min_w_0()
                    .when_some(label_color, |this, color| this.text_color(color))
                    .when_some(label_size, |this, size| this.text_size(size))
                    .when_some(self.weight, |this, weight| this.font_weight(weight))
                    .when(self.truncate, |this| this.min_w_0().truncate())
                    .when_some(label_hover, |this, color| {
                        this.group_hover(label_group, move |style| style.text_color(color))
                    })
                    .child(Text::new(label)),
            )
            .when_some(self.end_icon, |this, icon| this.child(make_icon(icon)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconButtonShape {
    #[default]
    Square,
    Wide,
}

pub struct IconButton {
    base: ButtonLike,
    icon: IconName,
    selected_icon: Option<IconName>,
    icon_size: IconSize,
    icon_color: Option<Color>,
    shape: IconButtonShape,
    bordered: bool,
    loading: bool,
}

crate::impl_into_element!(IconButton);

impl IconButton {
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.base = self.base.accessible_label(label);
        self
    }

    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            base: ButtonLike::new(id),
            icon,
            selected_icon: None,
            icon_size: IconSize::Medium,
            icon_color: None,
            shape: IconButtonShape::default(),
            bordered: false,
            loading: false,
        }
    }

    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.icon_size = size;
        self
    }

    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self
    }

    pub fn selected_icon(mut self, icon: IconName) -> Self {
        self.selected_icon = Some(icon);
        self
    }

    pub fn shape(mut self, shape: IconButtonShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn radius(mut self, radius: Pixels) -> Self {
        self.base = self.base.radius(radius);
        self
    }
}

impl Clickable for IconButton {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.base = self.base.on_click(handler);
        self
    }

    fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.base = self.base.cursor_style(cursor_style);
        self
    }
}

impl Disableable for IconButton {
    fn disabled(mut self, disabled: bool) -> Self {
        self.base = self.base.disabled(disabled);
        self
    }
}

impl Toggleable for IconButton {
    fn toggle_state(mut self, state: impl Into<ToggleState>) -> Self {
        self.base = self.base.toggle_state(state);
        self
    }
}

impl SelectableButton for IconButton {
    fn selected_style(mut self, style: ButtonStyle) -> Self {
        self.base = self.base.selected_style(style);
        self
    }
}

impl ButtonCommon for IconButton {
    fn controller_support(mut self, supported: bool) -> Self {
        self.base = self.base.controller_support(supported);
        self
    }

    fn id(&self) -> &ElementId {
        self.base.id()
    }

    fn style(mut self, style: ButtonStyle) -> Self {
        self.base = self.base.style(style);
        self
    }

    fn size(mut self, size: ButtonSize) -> Self {
        self.base = self.base.size(size);
        self
    }

    fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.base = self.base.tooltip(tooltip);
        self
    }

    fn tab_index(mut self, tab_index: isize) -> Self {
        self.base = self.base.tab_index(tab_index);
        self
    }

    fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.base = self.base.track_focus(focus_handle);
        self
    }
}

impl FixedWidth for IconButton {
    fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.base = self.base.width(width);
        self
    }

    fn full_width(mut self) -> Self {
        self.base = self.base.full_width();
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = if self.bordered && self.base.style == ButtonStyle::Subtle {
            ButtonStyle::Tinted(TintColor::Accent)
        } else {
            self.base.style
        };
        let colors =
            resolved_button_colors(style, self.base.selected_style, self.base.selected, cx);
        let fg = self.icon_color.unwrap_or(colors.fg);
        let hover_fg = if self.icon_color.is_some() && !colors.inverts_on_hover() {
            fg
        } else {
            colors.hover_fg
        };
        let group = ButtonLike::hover_group(&self.base.id);
        let icon = match (self.base.selected, self.selected_icon) {
            (true, Some(selected_icon)) => selected_icon,
            _ => self.icon,
        };
        let (padding_h, padding_v) = match (self.shape, self.base.size) {
            (IconButtonShape::Square, ButtonSize::Compact) => (px(3.), px(3.)),
            (IconButtonShape::Square, _) => (px(5.), px(5.)),
            (IconButtonShape::Wide, ButtonSize::Compact) => (px(8.), px(3.)),
            (IconButtonShape::Wide, _) => (px(10.), px(5.)),
        };
        let loading = self.loading;
        let disabled = self.base.disabled || loading;
        self.base
            .style(style)
            .disabled(disabled)
            .group_name(group.clone())
            .padding(padding_h, padding_v)
            .map(|this| {
                if loading {
                    this.child(Spinner::new().size(self.icon_size).color(fg))
                } else {
                    this.child(
                        Icon::new(icon)
                            .size(self.icon_size)
                            .color(fg)
                            .group_hover_color(group, hover_fg),
                    )
                }
            })
    }
}
