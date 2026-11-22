use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Bounds, ElementId, FocusHandle, FontWeight, Global,
    IntoElement, ParentElement, Pixels, Point, RenderOnce, ScrollHandle, SharedString, Size,
    Window, anchored, deferred, div, point, prelude::*, pulsating_between, px, relative, size,
};

use super::button::{Button, ButtonSize, ButtonStyle, IconButton};
use super::icon::{IconName, IconSize};
use super::label::{LINE_HEIGHT_TIGHT, LabelSize};
use super::scroll::ScrollableElement;
use super::util::bounds_tracker;
use crate::actions::{Cancel, Confirm, SelectNext, SelectPrevious, TOUR_CONTEXT};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::{ActionHandler, ButtonCommon, Clickable, ValueHandler};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnchorId(pub SharedString);

impl From<&'static str> for AnchorId {
    fn from(id: &'static str) -> Self {
        AnchorId(SharedString::new_static(id))
    }
}

impl From<SharedString> for AnchorId {
    fn from(id: SharedString) -> Self {
        AnchorId(id)
    }
}

impl From<String> for AnchorId {
    fn from(id: String) -> Self {
        AnchorId(SharedString::from(id))
    }
}

#[derive(Default, Clone)]
pub struct AnchorRegistry {
    bounds: HashMap<AnchorId, Bounds<Pixels>>,
}

impl Global for AnchorRegistry {}

impl AnchorRegistry {
    pub fn init(cx: &mut App) {
        if cx.try_global::<AnchorRegistry>().is_none() {
            cx.set_global(AnchorRegistry::default());
        }
    }

    pub fn get(cx: &App, id: &AnchorId) -> Option<Bounds<Pixels>> {
        cx.try_global::<AnchorRegistry>()
            .and_then(|registry| registry.bounds.get(id).copied())
    }

    pub fn set(cx: &mut App, id: AnchorId, bounds: Bounds<Pixels>) {
        if cx.has_global::<AnchorRegistry>() && Self::get(cx, &id) != Some(bounds) {
            cx.global_mut::<AnchorRegistry>().bounds.insert(id, bounds);
        }
    }

    pub fn remove(cx: &mut App, id: &AnchorId) {
        if cx.has_global::<AnchorRegistry>() && Self::get(cx, id).is_some() {
            cx.global_mut::<AnchorRegistry>().bounds.remove(id);
        }
    }
}

pub(crate) fn tour_anchor_tracker(id: AnchorId) -> impl IntoElement {
    bounds_tracker(move |bounds, _, cx| AnchorRegistry::set(cx, id, bounds))
}

pub trait TourAnchorExt: ParentElement + Sized {
    fn tour_anchor(self, id: impl Into<AnchorId>) -> Self {
        self.child(tour_anchor_tracker(id.into()))
    }
}

impl<E: ParentElement> TourAnchorExt for E {}

type StepBody = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;
type StepMove = Rc<dyn Fn(isize, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct TourStep {
    pub anchor: Option<AnchorId>,
    pub title: SharedString,
    pub body: SharedString,
    body_slot: Option<StepBody>,
}

impl TourStep {
    pub fn new(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self {
            anchor: None,
            title: title.into(),
            body: body.into(),
            body_slot: None,
        }
    }

    pub fn anchor(mut self, anchor: impl Into<AnchorId>) -> Self {
        self.anchor = Some(anchor.into());
        self
    }

    pub fn body_slot(
        mut self,
        body: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.body_slot = Some(Rc::new(body));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TourCardPlacement {
    pub origin: Point<Pixels>,
    pub width: Pixels,
    pub max_height: Pixels,
}

pub fn place_tour_card(anchor: Option<Bounds<Pixels>>, screen: Size<Pixels>) -> TourCardPlacement {
    let margin = px(24.);
    let clamp = |value: Pixels, low: Pixels, high: Pixels| value.max(low).min(high.max(low));
    let Some(rect) = anchor else {
        let width = clamp(screen.width - margin * 2., px(420.), px(720.)).min(screen.width);
        let max_height = clamp(screen.height - margin * 2., px(360.), px(760.)).min(screen.height);
        return TourCardPlacement {
            origin: point(
                ((screen.width - width) / 2.).max(px(0.)),
                ((screen.height - max_height) / 2.).max(px(0.)),
            ),
            width,
            max_height,
        };
    };
    let width = px(360.).min(screen.width);
    let max_height = clamp(screen.height - margin * 2., px(200.), px(560.)).min(screen.height);
    let space_right = screen.width - rect.right() - margin;
    let space_left = rect.left() - margin;
    let space_below = screen.height - rect.bottom() - margin;
    let centered = rect.center().x - width / 2.;
    let (left, top) = if space_right >= width {
        (rect.right() + margin, rect.top())
    } else if space_left >= width {
        (rect.left() - margin - width, rect.top())
    } else if space_below >= px(180.) {
        (centered, rect.bottom() + margin)
    } else {
        (centered, margin)
    };
    TourCardPlacement {
        origin: point(
            clamp(left, margin, screen.width - width - margin)
                .min((screen.width - width).max(px(0.))),
            clamp(top, margin, screen.height - max_height - margin)
                .min((screen.height - max_height).max(px(0.))),
        ),
        width,
        max_height,
    }
}

struct TourState {
    index: usize,
    focus_handle: FocusHandle,
    focused: bool,
    scroll_handle: ScrollHandle,
}

pub struct TourOverlay {
    id: ElementId,
    steps: Vec<TourStep>,
    on_finish: Option<ActionHandler>,
    on_step: Option<ValueHandler<usize>>,
}

crate::impl_into_element!(TourOverlay);

impl TourOverlay {
    pub fn new(steps: Vec<TourStep>) -> Self {
        Self {
            id: ElementId::Name("yp-tour".into()),
            steps,
            on_finish: None,
            on_step: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn on_finish(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_finish = Some(Rc::new(handler));
        self
    }

    pub fn on_step(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_step = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TourOverlay {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let viewport = window.viewport_size();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| TourState {
            index: 0,
            focus_handle: cx.focus_handle(),
            focused: false,
            scroll_handle: ScrollHandle::new(),
        });
        let (index, focus_handle, focused, scroll_handle) = {
            let state = state.read(cx);
            (
                state.index.min(self.steps.len().saturating_sub(1)),
                state.focus_handle.clone(),
                state.focused,
                state.scroll_handle.clone(),
            )
        };
        if !focused {
            window.focus(&focus_handle, cx);
            state.update(cx, |state, _| state.focused = true);
        }
        let Some(step) = self.steps.get(index).cloned() else {
            return div().into_any_element();
        };
        let total = self.steps.len();
        let is_last = index + 1 == total;
        let anchor = step
            .anchor
            .as_ref()
            .and_then(|anchor| AnchorRegistry::get(cx, anchor));
        let placement = place_tour_card(anchor, viewport);
        let is_reading = anchor.is_none();
        let scrim = theme.colors.scrim.opacity(theme.alpha.glass);
        let accent = theme.colors.accent_primary;

        let on_finish = self.on_finish;
        let on_step = self.on_step;
        let go: StepMove = {
            let state = state.clone();
            let on_finish = on_finish.clone();
            Rc::new(move |delta, window, cx| {
                let current = state.read(cx).index;
                let next = current as isize + delta;
                if next < 0 {
                    return;
                }
                let next = next as usize;
                if next >= total {
                    if let Some(handler) = &on_finish {
                        handler(window, cx);
                    }
                    return;
                }
                state.update(cx, |state, cx| {
                    state.index = next;
                    state.scroll_handle.set_offset(point(px(0.), px(0.)));
                    cx.notify();
                });
                if let Some(handler) = &on_step {
                    handler(&next, window, cx);
                }
            })
        };
        let finish: ActionHandler = Rc::new(move |window, cx| {
            if let Some(handler) = &on_finish {
                handler(window, cx);
            }
        });

        let scrim_rects: Vec<Bounds<Pixels>> = match anchor {
            None => vec![Bounds::new(point(px(0.), px(0.)), viewport)],
            Some(rect) => {
                let padding = Sizes::TOUR_SPOTLIGHT_PADDING;
                let left = (rect.left() - padding).max(px(0.));
                let top = (rect.top() - padding).max(px(0.));
                let right = (rect.right() + padding).min(viewport.width);
                let bottom = (rect.bottom() + padding).min(viewport.height);
                vec![
                    Bounds::new(point(px(0.), px(0.)), size(viewport.width, top)),
                    Bounds::new(
                        point(px(0.), bottom),
                        size(viewport.width, (viewport.height - bottom).max(px(0.))),
                    ),
                    Bounds::new(point(px(0.), top), size(left, (bottom - top).max(px(0.)))),
                    Bounds::new(
                        point(right, top),
                        size(
                            (viewport.width - right).max(px(0.)),
                            (bottom - top).max(px(0.)),
                        ),
                    ),
                ]
            }
        };

        let card_padding = if is_reading { px(28.) } else { px(18.) };
        let title_size = if is_reading {
            sizes.font_xl
        } else {
            sizes.font_lg
        };
        let body_size = if is_reading {
            sizes.font_md
        } else {
            sizes.font_sm
        };
        let body = match step.body_slot.as_ref() {
            Some(slot) => slot(window, cx),
            None => v_flex()
                .text_size(body_size)
                .line_height(relative(1.45))
                .text_color(theme.colors.text_secondary)
                .child(super::text::Text::new(step.body.clone()))
                .into_any_element(),
        };
        let back = go.clone();
        let next = go.clone();
        let skip = finish.clone();
        let close = finish.clone();
        let card = v_flex()
            .id("yp-tour-card")
            .absolute()
            .left(placement.origin.x)
            .top(placement.origin.y)
            .w(placement.width)
            .max_h(placement.max_height)
            .p(card_padding)
            .gap(px(10.))
            .rounded(px(8.))
            .bg(theme.colors.background_card)
            .border_1()
            .border_color(accent.opacity(theme.alpha.half))
            .elevation_3(cx)
            .occlude()
            .on_click(|_, _, cx| cx.stop_propagation())
            .child(
                h_flex()
                    .flex_none()
                    .child(
                        v_flex()
                            .px(px(8.))
                            .py(px(3.))
                            .rounded(Sizes::CONTROL_RADIUS)
                            .bg(accent.opacity(theme.alpha.hover))
                            .text_size(LabelSize::XSmall.pixels(&sizes))
                            .line_height(relative(LINE_HEIGHT_TIGHT))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(accent)
                            .child(super::text::Text::new(strings.ui_tour_step(
                                i64::try_from(index + 1).unwrap_or(i64::MAX),
                                i64::try_from(total).unwrap_or(i64::MAX),
                            ))),
                    )
                    .child(div().flex_1())
                    .child(
                        IconButton::new("yp-tour-close", IconName::Close)
                            .icon_size(IconSize::Small)
                            .icon_color(Color::Muted)
                            .size(ButtonSize::Compact)
                            .on_click(move |_, window, cx| close(window, cx)),
                    ),
            )
            .child(
                div()
                    .id(("yp-tour-body", index))
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(
                        v_flex()
                            .text_size(title_size)
                            .line_height(relative(LINE_HEIGHT_TIGHT))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.colors.text_primary)
                            .child(super::text::Text::new(step.title.clone())),
                    )
                    .child(body)
                    .overflow_y_scrollbar(&scroll_handle),
            )
            .child(
                h_flex()
                    .flex_none()
                    .pt(px(6.))
                    .gap(px(4.))
                    .when(index > 0, |this| {
                        this.child(
                            Button::new("yp-tour-back", strings.mods_tutorial_back())
                                .style(ButtonStyle::Transparent)
                                .on_click(move |_, window, cx| back(-1, window, cx)),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        Button::new("yp-tour-skip", strings.mods_tutorial_skip())
                            .style(ButtonStyle::Transparent)
                            .on_click(move |_, window, cx| skip(window, cx)),
                    )
                    .child(
                        Button::new(
                            "yp-tour-next",
                            if is_last {
                                strings.mods_tutorial_finish()
                            } else {
                                strings.mods_tutorial_next()
                            },
                        )
                        .style(ButtonStyle::Filled)
                        .on_click(move |_, window, cx| next(1, window, cx)),
                    ),
            );

        let key_next = go.clone();
        let key_confirm = go.clone();
        let key_back = go.clone();
        let key_cancel = finish.clone();
        let mut layer = div()
            .id(self.id)
            .key_context(TOUR_CONTEXT)
            .track_focus(&focus_handle)
            .relative()
            .w(viewport.width)
            .h(viewport.height)
            .font_ui(cx)
            .on_action(move |_: &SelectNext, window, cx| key_next(1, window, cx))
            .on_action(move |_: &Confirm, window, cx| key_confirm(1, window, cx))
            .on_action(move |_: &SelectPrevious, window, cx| key_back(-1, window, cx))
            .on_action(move |_: &Cancel, window, cx| key_cancel(window, cx));
        for (rect_index, rect) in scrim_rects.into_iter().enumerate() {
            let advance = go.clone();
            layer = layer.child(
                div()
                    .id(("yp-tour-scrim", rect_index))
                    .absolute()
                    .left(rect.left())
                    .top(rect.top())
                    .w(rect.size.width)
                    .h(rect.size.height)
                    .occlude()
                    .bg(scrim)
                    .on_click(move |_, window, cx| advance(1, window, cx)),
            );
        }
        if let Some(rect) = anchor {
            let inset = px(4.);
            layer = layer.child(
                div()
                    .absolute()
                    .left(rect.left() - inset)
                    .top(rect.top() - inset)
                    .w(rect.size.width + inset * 2.)
                    .h(rect.size.height + inset * 2.)
                    .rounded(Sizes::ROW_RADIUS)
                    .border_2()
                    .border_color(accent)
                    .with_animation(
                        "yp-tour-pulse",
                        Animation::new(theme.motion.pulse * 2)
                            .repeat()
                            .with_easing(pulsating_between(0.35, 1.)),
                        |this, delta| this.opacity(delta),
                    ),
            );
        }
        deferred(
            anchored()
                .position(point(px(0.), px(0.)))
                .child(layer.child(card)),
        )
        .with_priority(3)
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Size<Pixels> {
        size(px(1300.), px(750.))
    }

    #[test]
    fn reading_steps_are_centered_and_wide() {
        let placement = place_tour_card(None, screen());
        assert_eq!(placement.width, px(720.));
        assert_eq!(placement.origin.x, px(290.));
        assert_eq!(placement.max_height, px(702.));
    }

    #[test]
    fn card_prefers_the_right_side_of_the_anchor() {
        let anchor = Bounds::new(point(px(100.), px(120.)), size(px(200.), px(40.)));
        let placement = place_tour_card(Some(anchor), screen());
        assert_eq!(placement.origin, point(px(324.), px(120.)));
        let low = Bounds::new(point(px(100.), px(600.)), size(px(200.), px(40.)));
        assert_eq!(place_tour_card(Some(low), screen()).origin.y, px(166.));
        assert_eq!(placement.width, px(360.));
    }

    #[test]
    fn card_falls_back_to_the_left_then_below() {
        let right_edge = Bounds::new(point(px(1100.), px(100.)), size(px(180.), px(40.)));
        let left = place_tour_card(Some(right_edge), screen());
        assert_eq!(left.origin.x, px(1100.) - px(24.) - px(360.));
        let wide = Bounds::new(point(px(200.), px(100.)), size(px(1000.), px(40.)));
        let below = place_tour_card(Some(wide), screen());
        assert_eq!(below.origin.y, px(164.));
        assert_eq!(below.origin.x, px(700.) - px(180.));
    }

    #[test]
    fn card_always_stays_inside_small_windows() {
        let tiny = size(px(300.), px(180.));
        let anchor = Bounds::new(point(px(250.), px(150.)), size(px(40.), px(20.)));
        for placement in [
            place_tour_card(Some(anchor), tiny),
            place_tour_card(None, tiny),
        ] {
            assert!(placement.origin.x >= px(0.));
            assert!(placement.origin.y >= px(0.));
            assert!(placement.origin.x + placement.width <= tiny.width);
            assert!(placement.origin.y + placement.max_height <= tiny.height);
        }
    }

    #[gpui::test]
    fn anchors_are_recorded_without_touching_global_observers(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            AnchorRegistry::init(cx);
            let id = AnchorId::from("drop-zone");
            assert_eq!(AnchorRegistry::get(cx, &id), None);
            let bounds = Bounds::new(point(px(1.), px(2.)), size(px(3.), px(4.)));
            AnchorRegistry::set(cx, id.clone(), bounds);
            assert_eq!(AnchorRegistry::get(cx, &id), Some(bounds));
            AnchorRegistry::remove(cx, &id);
            assert_eq!(AnchorRegistry::get(cx, &id), None);
        });
    }
}
