use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ClickEvent, ElementId, FontWeight, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Transformation, Window, div, prelude::*, px,
    radians, relative, svg,
};

use super::divider::Divider;
use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_TIGHT, Label, LabelSize};
use super::text::Text;
use super::tooltip::Tooltip;
use super::util::KeyboardFocus;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, Elevation, ResponsiveExt, Sizes, easing};
use crate::traits::{ClickHandler, ValueHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CardPadding {
    None,
    Compact,
    #[default]
    Default,
}

impl CardPadding {
    fn pixels(self, sizes: &Sizes) -> (Pixels, Pixels) {
        match self {
            CardPadding::None => (px(0.), px(0.)),
            CardPadding::Compact => (sizes.card_padding_h, sizes.card_padding_v),
            CardPadding::Default => (sizes.card_padding_h, sizes.card_padding_h),
        }
    }
}

pub struct Card {
    title: Option<SharedString>,
    header: Option<AnyElement>,
    header_end: Option<AnyElement>,
    elevation: Elevation,
    translucent: bool,
    prominent_header: bool,
    padding: CardPadding,
    full_height: bool,
    accent: Option<Color>,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(Card);

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    pub fn new() -> Self {
        Self {
            title: None,
            header: None,
            header_end: None,
            elevation: Elevation::None,
            translucent: false,
            prominent_header: false,
            padding: CardPadding::default(),
            full_height: false,
            accent: None,
            children: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn header_end(mut self, slot: impl IntoElement) -> Self {
        self.header_end = Some(slot.into_any_element());
        self
    }

    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn prominent_header(mut self) -> Self {
        self.prominent_header = true;
        self
    }

    pub fn translucent(mut self, translucent: bool) -> Self {
        self.translucent = translucent;
        self
    }

    pub fn padding(mut self, padding: CardPadding) -> Self {
        self.padding = padding;
        self
    }

    pub fn full_height(mut self) -> Self {
        self.full_height = true;
        self
    }

    pub fn accent(mut self, color: Color) -> Self {
        self.accent = Some(color);
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let (padding_h, padding_v) = self.padding.pixels(&sizes);
        let background = if self.translucent {
            theme.colors.background_card.opacity(theme.alpha.panel)
        } else {
            theme.colors.background_card
        };
        let border = self
            .accent
            .map(|accent| accent.color(cx).opacity(theme.alpha.accent_border))
            .unwrap_or(theme.colors.border_light);
        let has_header = self.title.is_some() || self.header.is_some() || self.header_end.is_some();
        v_flex()
            .w_full()
            .min_w_0()
            .when(self.full_height, |this| this.h_full().min_h_0())
            .rounded(if theme.id == crate::ThemeId::NierNew {
                px(0.)
            } else {
                sizes.border_radius
            })
            .bg(background)
            .border_1()
            .border_color(border)
            .elevation(self.elevation, cx)
            .when(has_header, |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .gap(sizes.spacing_md)
                        .px(sizes.card_padding_h)
                        .py(sizes.card_padding_v)
                        .when(theme.id != crate::ThemeId::NierNew, |el| {
                            el.rounded_t(sizes.border_radius)
                                .bg(theme.colors.surface_medium)
                        })
                        .when(theme.id == crate::ThemeId::NierNew, |el| {
                            el.border_b_1().border_color(theme.colors.border_medium)
                        })
                        .when(self.prominent_header, |el| {
                            el.bg(theme.colors.accent_primary)
                        })
                        .when_some(self.title, |this, title| {
                            this.child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .text_size(sizes.font_sm)
                                    .line_height(relative(LINE_HEIGHT_TIGHT))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if self.prominent_header {
                                        theme.colors.button_text
                                    } else {
                                        theme.colors.accent_primary
                                    })
                                    .child(Text::new(title)),
                            )
                        })
                        .when_some(self.header, |this, header| {
                            this.child(div().flex_1().min_w_0().child(header))
                        })
                        .when_some(self.header_end, |this, slot| {
                            this.child(div().flex_none().child(slot))
                        }),
                )
            })
            .child(
                v_flex()
                    .w_full()
                    .min_w_0()
                    .when(self.full_height, |this| this.flex_1().min_h_0())
                    .px(padding_h)
                    .py(padding_v)
                    .children(self.children),
            )
    }
}

#[derive(Clone)]
pub struct ChangedBadge {
    id: ElementId,
    labels: Vec<SharedString>,
}

crate::impl_into_element!(ChangedBadge);

impl ChangedBadge {
    const PULSES: f32 = 3.;

    pub fn new(id: impl Into<ElementId>, labels: Vec<SharedString>) -> Self {
        Self {
            id: id.into(),
            labels,
        }
    }
}

impl RenderOnce for ChangedBadge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let color = theme.colors.success;
        let count = i64::try_from(self.labels.len()).unwrap_or(i64::MAX);
        let dot = Sizes::INDICATOR_SIZE;
        let halo = dot * 2.;
        let pulse_id = ElementId::from((self.id.clone(), SharedString::from(count.to_string())));
        h_flex()
            .id(self.id)
            .flex_none()
            .gap(sizes.spacing_sm)
            .child(
                div()
                    .relative()
                    .size(halo)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().absolute().rounded(halo).bg(color).with_animation(
                        pulse_id,
                        Animation::new(theme.motion.pulse * 3),
                        move |this, delta| {
                            let finished = delta >= 1.;
                            let phase = (delta * Self::PULSES).fract();
                            let scale = 0.5 + 0.5 * phase;
                            let inset = halo * ((1. - scale) / 2.);
                            let opacity = if finished { 0. } else { 0.45 * (1. - phase) };
                            this.size(halo * scale)
                                .top(inset)
                                .left(inset)
                                .opacity(opacity)
                        },
                    ))
                    .child(div().size(dot).rounded(dot).bg(color)),
            )
            .child(
                v_flex()
                    .text_size(sizes.font_xs)
                    .line_height(relative(LINE_HEIGHT_TIGHT))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(color)
                    .child(Text::new(strings.card_changed_count(count))),
            )
            .tooltip(Tooltip::lines(
                strings.card_changed_tooltip_header(),
                self.labels,
            ))
    }
}

pub(crate) struct DisclosureState {
    pub(crate) open: bool,
    pub(crate) generation: usize,
}

pub(crate) fn chevron(
    open: bool,
    generation: usize,
    size: IconSize,
    color: Color,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let duration = cx.theme().motion.base;
    let size = size.pixels(window, cx);
    let closed_turn = -0.25_f32;
    let (from, to) = if open {
        (closed_turn, 0.)
    } else {
        (0., closed_turn)
    };
    let icon = svg()
        .path(IconName::KeyboardArrowDown.path())
        .size(size)
        .flex_none()
        .text_color(color.color(cx));
    if generation == 0 {
        return icon
            .with_transformation(Transformation::rotate(radians(to * std::f32::consts::TAU)))
            .into_any_element();
    }
    icon.with_animation(
        ("yp-chevron", generation),
        Animation::new(duration),
        move |this, delta| {
            let turn = from + (to - from) * delta;
            this.with_transformation(Transformation::rotate(radians(
                turn * std::f32::consts::TAU,
            )))
        },
    )
    .into_any_element()
}

pub struct CollapsibleCard {
    id: ElementId,
    title: SharedString,
    default_open: bool,
    open: Option<bool>,
    changed: Vec<SharedString>,
    header_end: Option<AnyElement>,
    on_toggle: Option<ValueHandler<bool>>,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(CollapsibleCard);

impl CollapsibleCard {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            default_open: false,
            open: None,
            changed: Vec::new(),
            header_end: None,
            on_toggle: None,
            children: Vec::new(),
        }
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn changed(mut self, labels: Vec<SharedString>) -> Self {
        self.changed = labels;
        self
    }

    pub fn header_end(mut self, slot: impl IntoElement) -> Self {
        self.header_end = Some(slot.into_any_element());
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for CollapsibleCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CollapsibleCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let default_open = self.default_open;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| DisclosureState {
            open: default_open,
            generation: 0,
        });
        if let Some(controlled) = self.open {
            state.update(cx, |state, _| {
                if state.open != controlled {
                    state.open = controlled;
                    state.generation += 1;
                }
            });
        }
        let (open, generation) = {
            let state = state.read(cx);
            (state.open, state.generation)
        };
        let radius = sizes.border_radius;
        let hover_background = theme.colors.accent_primary.opacity(theme.alpha.subtle);
        let accent = theme.colors.accent_primary;
        let on_toggle = self.on_toggle;
        let toggle_state = state.clone();
        let header = h_flex()
            .id((self.id.clone(), "header"))
            .role(gpui::Role::Button)
            .aria_label(self.title.clone())
            .aria_expanded(open)
            .w_full()
            .gap(sizes.spacing_md)
            .px(sizes.card_padding_h)
            .py(sizes.card_padding_v)
            .rounded_t(radius)
            .when(!open, |this| this.rounded_b(radius))
            .when(theme.id != crate::ThemeId::NierNew, |el| {
                el.bg(theme.colors.surface_medium)
            })
            .when(theme.id == crate::ThemeId::NierNew && open, |el| {
                el.border_b_1().border_color(theme.colors.border_medium)
            })
            .border_l(Sizes::ACCENT_BAR_WIDTH)
            .border_color(theme.colors.transparent)
            .cursor_pointer()
            .keyboard_activate(cx)
            .hover(move |style| style.bg(hover_background).border_color(accent))
            .on_disclosure_toggle(open, move |_, window, cx| {
                let next = toggle_state.update(cx, |state, cx| {
                    state.open = !state.open;
                    state.generation += 1;
                    cx.notify();
                    state.open
                });
                if let Some(handler) = &on_toggle {
                    handler(&next, window, cx);
                }
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .text_size(sizes.font_sm)
                    .line_height(relative(LINE_HEIGHT_TIGHT))
                    .font_weight(FontWeight::BOLD)
                    .text_color(accent)
                    .child(Text::new(self.title)),
            )
            .when(!self.changed.is_empty(), |this| {
                this.child(ChangedBadge::new(
                    (self.id.clone(), "changed"),
                    self.changed,
                ))
            })
            .when_some(self.header_end, |this, slot| {
                this.child(div().flex_none().child(slot))
            })
            .child(chevron(
                open,
                generation,
                IconSize::Medium,
                Color::Accent,
                window,
                cx,
            ));
        let content = open.then(|| {
            let content = v_flex()
                .w_full()
                .p(sizes.card_padding_h)
                .children(self.children);
            if generation == 0 {
                content.into_any_element()
            } else {
                content
                    .with_animation(
                        ("yp-card-content", generation),
                        Animation::new(theme.motion.expand).with_easing(easing::nier_smooth),
                        |this, delta| this.opacity(delta),
                    )
                    .into_any_element()
            }
        });
        v_flex()
            .w_full()
            .rounded(radius)
            .bg(theme.colors.background_card)
            .border_1()
            .border_color(theme.colors.border_light)
            .child(header)
            .children(content)
    }
}

pub struct Disclosure {
    id: ElementId,
    open: bool,
    size: IconSize,
    color: Color,
    on_toggle: Option<ClickHandler>,
}

crate::impl_into_element!(Disclosure);

impl Disclosure {
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            size: IconSize::Small,
            color: Color::Muted,
            on_toggle: None,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Disclosure {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let strings = cx.t();
        let open = self.open;
        let state =
            window.use_keyed_state((self.id.clone(), "state"), cx, |_, _| DisclosureState {
                open,
                generation: 0,
            });
        state.update(cx, |state, _| {
            if state.open != open {
                state.open = open;
                state.generation += 1;
            }
        });
        let generation = state.read(cx).generation;
        let hover_background = theme.colors.accent_primary.opacity(theme.alpha.subtle);
        let on_toggle = self.on_toggle;
        let tooltip = if open {
            strings.ui_collapse()
        } else {
            strings.ui_expand()
        };
        div()
            .id(self.id)
            .flex_none()
            .p(px(2.))
            .rounded(Sizes::CONTROL_RADIUS)
            .cursor_pointer()
            .hover(move |style| style.bg(hover_background))
            .child(chevron(open, generation, self.size, self.color, window, cx))
            .tooltip(Tooltip::text(tooltip))
            .when_some(on_toggle, |this, handler| {
                this.on_click(move |event, window, cx| {
                    cx.stop_propagation();
                    handler(event, window, cx);
                })
            })
    }
}

pub struct SectionHeader {
    label: SharedString,
    end_slot: Option<AnyElement>,
    divider: bool,
    color: Color,
    uppercase: bool,
    icon: Option<IconName>,
}

crate::impl_into_element!(SectionHeader);

impl SectionHeader {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            end_slot: None,
            divider: true,
            color: Color::Accent,
            uppercase: false,
            icon: None,
        }
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.end_slot = Some(slot.into_any_element());
        self
    }

    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let label = Label::new(self.label)
            .size(LabelSize::Small)
            .color(self.color)
            .bold()
            .line_height(LINE_HEIGHT_TIGHT);
        let label = if self.uppercase {
            label.uppercase()
        } else {
            label
        };
        v_flex()
            .w_full()
            .gap(sizes.spacing_sm)
            .child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .when_some(self.icon, |this, icon| {
                        this.child(Icon::new(icon).size(IconSize::Small).color(self.color))
                    })
                    .child(div().flex_1().min_w_0().child(label))
                    .when_some(self.end_slot, |this, slot| {
                        this.child(div().flex_none().child(slot))
                    }),
            )
            .when(self.divider, |this| this.child(Divider::horizontal()))
    }
}

pub struct Section {
    header: SectionHeader,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(Section);

impl Section {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            header: SectionHeader::new(label),
            children: Vec::new(),
        }
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.header = self.header.end_slot(slot);
        self
    }

    pub fn divider(mut self, divider: bool) -> Self {
        self.header = self.header.divider(divider);
        self
    }

    pub fn header(mut self, header: SectionHeader) -> Self {
        self.header = header;
        self
    }
}

impl ParentElement for Section {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Section {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        v_flex()
            .w_full()
            .gap(sizes.spacing_sm)
            .pb(sizes.padding_md)
            .child(self.header)
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{KeyUpEvent, Keystroke, TestAppContext, VisualTestContext};

    use super::*;
    use crate::components::test_support::open_root;

    fn release(cx: &mut VisualTestContext, key: &'static str) {
        cx.simulate_event(KeyUpEvent {
            keystroke: Keystroke::parse(key).unwrap(),
        });
    }

    #[gpui::test]
    fn the_card_header_is_a_tab_stop_that_toggles_with_enter_and_space(cx: &mut TestAppContext) {
        let toggles = Rc::new(RefCell::new(Vec::new()));
        let sink = toggles.clone();
        let (_root, cx) = open_root(cx, move |_, _| {
            let sink = sink.clone();
            CollapsibleCard::new("card", "Title")
                .on_toggle(move |open, _, _| sink.borrow_mut().push(*open))
                .child(div().child("body"))
                .into_any_element()
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab enter");
        release(cx, "enter");
        assert_eq!(*toggles.borrow(), vec![true]);
        cx.simulate_keystrokes("space");
        release(cx, "space");
        assert_eq!(*toggles.borrow(), vec![true, false]);
    }

    #[gpui::test]
    fn a_key_release_without_its_press_on_the_same_element_does_not_click(cx: &mut TestAppContext) {
        let toggles = Rc::new(RefCell::new(Vec::new()));
        let sink = toggles.clone();
        let (_root, cx) = open_root(cx, move |_, _| {
            let sink = sink.clone();
            CollapsibleCard::new("card", "Title")
                .on_toggle(move |open, _, _| sink.borrow_mut().push(*open))
                .into_any_element()
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab");
        release(cx, "enter");
        assert!(toggles.borrow().is_empty());
        cx.simulate_keystrokes("space");
        release(cx, "enter");
        assert!(toggles.borrow().is_empty());
    }
}
