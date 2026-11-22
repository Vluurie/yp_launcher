use std::rc::Rc;

use gpui::{
    AnyView, App, ClickEvent, CursorStyle, DefiniteLength, ElementId, FocusHandle,
    InteractiveElement, SharedString, Styled, Window,
};

use crate::components::{ButtonSize, ButtonStyle};

pub type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
pub type TooltipBuilder = Rc<dyn Fn(&mut Window, &mut App) -> AnyView>;
pub type ValueHandler<V> = Rc<dyn Fn(&V, &mut Window, &mut App)>;
pub type ActionHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToggleState {
    #[default]
    Unselected,
    Indeterminate,
    Selected,
}

impl ToggleState {
    pub fn selected(self) -> bool {
        self == ToggleState::Selected
    }

    pub fn inverse(self) -> Self {
        match self {
            ToggleState::Selected => ToggleState::Unselected,
            ToggleState::Unselected | ToggleState::Indeterminate => ToggleState::Selected,
        }
    }

    pub fn from_any_and_all(any_checked: bool, all_checked: bool) -> Self {
        match (any_checked, all_checked) {
            (true, true) => ToggleState::Selected,
            (false, false) => ToggleState::Unselected,
            _ => ToggleState::Indeterminate,
        }
    }
}

impl From<bool> for ToggleState {
    fn from(selected: bool) -> Self {
        if selected {
            ToggleState::Selected
        } else {
            ToggleState::Unselected
        }
    }
}

impl From<Option<bool>> for ToggleState {
    fn from(selected: Option<bool>) -> Self {
        match selected {
            Some(true) => ToggleState::Selected,
            Some(false) => ToggleState::Unselected,
            None => ToggleState::Indeterminate,
        }
    }
}

pub trait Clickable {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self;
    fn cursor_style(self, cursor_style: CursorStyle) -> Self;
}

pub trait Disableable {
    fn disabled(self, disabled: bool) -> Self;
}

pub trait Toggleable {
    fn toggle_state(self, state: impl Into<ToggleState>) -> Self;
}

pub trait SelectableButton: Toggleable {
    fn selected_style(self, style: ButtonStyle) -> Self;
}

pub trait ButtonCommon: Clickable + Disableable {
    fn id(&self) -> &ElementId;
    fn controller_support(self, supported: bool) -> Self;
    fn style(self, style: ButtonStyle) -> Self;
    fn size(self, size: ButtonSize) -> Self;
    fn tooltip(self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self;
    fn tab_index(self, tab_index: isize) -> Self;
    fn track_focus(self, focus_handle: &FocusHandle) -> Self;
}

pub trait FixedWidth {
    fn width(self, width: impl Into<DefiniteLength>) -> Self;
    fn full_width(self) -> Self;
}

pub trait VisibleOnHover {
    fn visible_on_hover(self, group: impl Into<SharedString>) -> Self;
}

impl<E: InteractiveElement + Styled> VisibleOnHover for E {
    fn visible_on_hover(self, group: impl Into<SharedString>) -> Self {
        self.invisible().group_hover(group, |style| style.visible())
    }
}
