use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Role, Window, accesskit,
};

pub(crate) fn layer(
    id: impl Into<ElementId>,
    covered: bool,
    child: impl IntoElement,
) -> impl IntoElement {
    Layer {
        id: id.into(),
        covered,
        child: child.into_any_element(),
    }
}

struct Layer {
    id: ElementId,
    covered: bool,
    child: AnyElement,
}

impl Element for Layer {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn a11y_role(&self) -> Option<Role> {
        Some(Role::Group)
    }
    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        if self.covered {
            node.set_hidden();
        }
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}
