use super::{MenuChoice, MenuDropdown};
use gpui::{AnyElement, App, ElementId, IntoElement, RenderOnce, Window};

pub(super) struct Trigger {
    pub size: [f32; 2],
    pub render: Box<dyn FnOnce(bool) -> AnyElement>,
}

#[derive(IntoElement)]
pub struct MenuActionMenu(MenuDropdown);
impl MenuActionMenu {
    pub fn new<E: IntoElement>(
        id: impl Into<ElementId>,
        choices: impl IntoIterator<Item = MenuChoice>,
        trigger: impl FnOnce(bool) -> E + 'static,
        on_action: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        let mut dropdown = MenuDropdown::new(id, "", choices, 0, on_action);
        dropdown.custom_trigger = Some(Trigger {
            size: [447., 50.],
            render: Box::new(|open| trigger(open).into_any_element()),
        });
        dropdown.anchor = Some([435., 5.]);
        Self(dropdown)
    }
    pub fn anchor(mut self, x: f32, y: f32) -> Self {
        self.0.anchor = Some([x, y]);
        self
    }
    pub fn trigger_size(mut self, width: f32, height: f32) -> Self {
        if let Some(trigger) = &mut self.0.custom_trigger {
            trigger.size = [width.max(0.), height.max(0.)];
        }
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.0.enabled = enabled;
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.0.scale = scale.max(0.);
        self
    }
}
impl RenderOnce for MenuActionMenu {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.0
    }
}
