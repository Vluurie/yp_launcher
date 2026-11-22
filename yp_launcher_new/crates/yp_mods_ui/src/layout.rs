use gpui::{Div, IntoElement, ParentElement as _, Styled as _, div};

pub(crate) fn fill(content: impl IntoElement) -> Div {
    div()
        .relative()
        .flex_1()
        .min_h_0()
        .child(div().absolute().inset_0().child(content))
}

pub(crate) fn shrinkable(content: impl IntoElement) -> Div {
    div().flex_1().min_w_0().child(content)
}
