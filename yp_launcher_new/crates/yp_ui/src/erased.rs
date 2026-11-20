use gpui::{AnyElement, App, IntoElement, RenderOnce, ViewElement, Window};

type RenderFn = Box<dyn FnOnce(&mut Window, &mut App) -> AnyElement>;

pub struct ErasedComponent(RenderFn);

impl ErasedComponent {
    pub fn new<C: RenderOnce>(component: C) -> Self {
        Self(Box::new(move |window, cx| {
            component.render(window, cx).into_any_element()
        }))
    }
}

impl RenderOnce for ErasedComponent {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        (self.0)(window, cx)
    }
}

impl IntoElement for ErasedComponent {
    type Element = ViewElement<Self>;

    #[track_caller]
    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

#[macro_export]
macro_rules! impl_into_element {
    ([$($generics:tt)*] $ty:ty) => {
        impl<$($generics)*> $crate::__gpui::IntoElement for $ty {
            type Element = $crate::__gpui::ViewElement<$crate::ErasedComponent>;

            #[track_caller]
            fn into_element(self) -> Self::Element {
                $crate::__gpui::IntoElement::into_element($crate::ErasedComponent::new(self))
            }
        }
    };
    ($ty:ty) => {
        impl $crate::__gpui::IntoElement for $ty {
            type Element = $crate::__gpui::ViewElement<$crate::ErasedComponent>;

            #[track_caller]
            fn into_element(self) -> Self::Element {
                $crate::__gpui::IntoElement::into_element($crate::ErasedComponent::new(self))
            }
        }
    };
}
