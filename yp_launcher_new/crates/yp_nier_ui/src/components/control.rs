pub(super) mod layout;

use self::layout::ControlState;
use crate::{Rect, Tint};
use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Tab,
    Row,
    Item,
    Dialog,
    Setting,
    Choice,
    ScrollChoice,
}

pub(crate) fn mix(a: Tint, b: Tint, t: f32) -> Tint {
    std::array::from_fn(|i| {
        (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t.clamp(0., 1.)).round() as u8
    })
}

type Click = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type Hover = Rc<dyn Fn(bool, &mut Window, &mut App)>;
struct Control {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    width: f32,
    scale: f32,
    hover: Option<Hover>,
    selected: Option<bool>,
    enabled: bool,
    expanded: bool,
    editing: bool,
    value: Option<SharedString>,
    quantity: Option<u32>,
    slider: Option<(usize, usize, Option<usize>, bool)>,
    click: Option<Click>,
}
impl Control {
    fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, width: f32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            width,
            scale: 1.,
            hover: None,
            selected: None,
            enabled: true,
            expanded: false,
            editing: false,
            value: None,
            quantity: None,
            slider: None,
            click: None,
        }
    }
    fn render(self, kind: Kind, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| ControlState::new());
        let reduced = crate::ReducedMotion::enabled(cx);
        let (focus, extent, animating) = state.update(cx, |s, _| s.update(&self, kind, reduced));
        if animating {
            window.request_animation_frame();
        }
        let (left, height, after) = match kind {
            Kind::Tab => (50., 84., 12.),
            Kind::Item => (60., 50., 12.),
            Kind::ScrollChoice => (0., 53., 50.),
            Kind::Choice => (0., 45., 50.),
            Kind::Setting => (60., 64., 72.),
            _ => (60., 64., 12.),
        };
        let width = left + self.width + after;
        let shown = if matches!(kind, Kind::Tab | Kind::Row | Kind::Setting | Kind::Item) {
            crate::text_reveal::display(
                &self.label,
                gpui::ElementId::View(state.entity_id()),
                if kind == Kind::Row { 20 } else { 0 },
                crate::text_reveal::Cadence::Row,
                window,
                cx,
            )
        } else {
            self.label.clone()
        };
        let art = layout::art(&self, kind, width, height, focus, extent, &shown);
        let hit = match kind {
            Kind::Tab => Rect::new(left - 12., 0., self.width + 24., 84.),
            Kind::Setting => Rect::new(left, 12., self.width, 40.),
            Kind::ScrollChoice => Rect::new(0., 6.5, self.width, 40.),
            Kind::Choice => Rect::new(0., 2.5, self.width, 40.),
            Kind::Item => Rect::new(left, 5., self.width, 40.),
            Kind::Row => Rect::new(50., 12., self.width + 1., 40.),
            Kind::Dialog => Rect::new(left, 12., self.width, 40.),
        };
        let target = div()
            .id("hit")
            .role(match kind {
                Kind::Item | Kind::Choice | Kind::ScrollChoice => gpui::Role::ListBoxOption,
                Kind::Tab => gpui::Role::Tab,
                _ => gpui::Role::Button,
            })
            .aria_label(self.label.clone())
            .when_some(self.selected, |el, selected| el.aria_selected(selected))
            .when_some(self.value.clone(), |el, value| el.aria_value(value))
            .absolute()
            .left(px(hit.x * self.scale))
            .top(px(hit.y * self.scale))
            .w(px(hit.w * self.scale))
            .h(px(hit.h * self.scale))
            .when(self.enabled, |e| e.cursor_pointer())
            .when(self.enabled || kind.tracks_unavailable_cursor(), |e| {
                e.on_hover(move |over, window, cx| {
                    state.update(cx, |state, cx| {
                        state.hovered = *over;
                        cx.notify();
                    });
                    if let Some(hover) = &self.hover {
                        hover(*over, window, cx);
                    }
                })
            })
            .when_some(self.click.filter(|_| self.enabled), |e, click| {
                e.tab_index(0)
                    .focus(|s| s.border_2().border_color(gpui::rgb(0x48463d)))
                    .on_click(move |ev, w, cx| click(ev, w, cx))
            });
        div()
            .id(self.id)
            .relative()
            .w(px(width * self.scale))
            .h(px(height * self.scale))
            .when(
                !self.enabled && matches!(kind, Kind::ScrollChoice | Kind::Dialog),
                |el| el.opacity(0.5),
            )
            .child(art)
            .child(target)
    }
}

pub(super) fn slider_face(
    width: f32,
    height: f32,
    body_width: f32,
    label: &SharedString,
    enabled: bool,
    focus: f32,
    extent: f32,
) -> crate::Surface {
    let mut control = Control::new("long-slider-face", label.clone(), body_width);
    control.enabled = enabled;
    control.slider = Some((0, 20, None, true));
    layout::art(&control, Kind::Setting, width, height, focus, extent, label)
}

macro_rules! control {
    ($name:ident,$kind:expr,$width:expr) => {
        #[derive(IntoElement)]
        pub struct $name(Control);
        impl $name {
            pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
                Self(Control::new(id, label, $width))
            }
            pub fn selected(mut self, selected: bool) -> Self {
                self.0.selected = Some(selected);
                self
            }
            pub fn enabled(mut self, enabled: bool) -> Self {
                self.0.enabled = enabled;
                self
            }
            pub fn width(mut self, width: f32) -> Self {
                self.0.width = width.max(1.);
                self
            }
            pub fn scale(mut self, scale: f32) -> Self {
                self.0.scale = scale.max(0.);
                self
            }
            pub fn on_hover(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
                self.0.hover = Some(Rc::new(f));
                self
            }
            pub fn icon(mut self, name: impl Into<SharedString>) -> Self {
                self.0.icon = Some(name.into());
                self
            }
            pub fn on_click(
                mut self,
                f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
            ) -> Self {
                self.0.click = Some(Rc::new(f));
                self
            }
        }
        impl RenderOnce for $name {
            fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
                self.0.render($kind, window, cx)
            }
        }
    };
}
control!(MenuTab, Kind::Tab, 180.);
impl MenuTab {
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.0.expanded = expanded;
        self
    }
}
control!(MenuRow, Kind::Row, 420.);
control!(MenuItemRow, Kind::Item, 375.);
impl MenuItemRow {
    pub fn quantity(mut self, count: u32) -> Self {
        self.0.quantity = Some(count);
        self
    }
    pub fn confirmed(mut self, value: bool) -> Self {
        self.0.expanded = value;
        self
    }
}
control!(DialogButton, Kind::Dialog, 150.);

control!(SettingRow, Kind::Setting, 620.);
impl SettingRow {
    pub fn editing(mut self, editing: bool) -> Self {
        self.0.editing = editing;
        self
    }
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.0.value = Some(value.into());
        self
    }
    pub(super) fn steps(
        mut self,
        value: usize,
        steps: usize,
        mark: Option<usize>,
        editing: bool,
    ) -> Self {
        self.0.slider = Some((value, steps, mark, editing));
        self
    }
}
#[derive(IntoElement)]
pub struct PulldownOption {
    control: Control,
    scrolling: bool,
}
impl PulldownOption {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            control: Control::new(id, label, 236.),
            scrolling: false,
        }
    }
    pub fn scrolling(mut self, value: bool) -> Self {
        self.scrolling = value;
        self.control.width = if value { 302. } else { 236. };
        self
    }
    pub fn selected(mut self, value: bool) -> Self {
        self.control.selected = Some(value);
        self
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.control.enabled = value;
        self
    }
    pub fn width(mut self, value: f32) -> Self {
        self.control.width = value.max(1.);
        self
    }
    pub fn scale(mut self, value: f32) -> Self {
        self.control.scale = value.max(0.);
        self
    }
    pub fn on_hover(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.control.hover = Some(Rc::new(f));
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.control.click = Some(Rc::new(f));
        self
    }
}
impl RenderOnce for PulldownOption {
    fn render(self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.control.render(
            if self.scrolling {
                Kind::ScrollChoice
            } else {
                Kind::Choice
            },
            w,
            cx,
        )
    }
}
