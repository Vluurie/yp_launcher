use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, CursorStyle, ElementId, IntoElement, Render, RenderOnce,
    SharedString, Window, div, prelude::*, px,
};

use super::icon::{Icon, IconName, IconSize};
use super::tooltip::{Tooltip, tooltip_container};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};

#[derive(Clone, Debug, PartialEq)]
pub struct SortableDrag {
    pub list: ElementId,
    pub index: usize,
    pub label: SharedString,
}

struct SortablePreview {
    label: SharedString,
}

impl Render for SortablePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        tooltip_container(window, cx).child(
            h_flex()
                .gap(sizes.spacing_md)
                .child(
                    Icon::new(IconName::DragHandle)
                        .size(IconSize::Small)
                        .color(Color::Accent),
                )
                .child(super::text::Text::new(self.label.clone())),
        )
    }
}

pub fn reorder<T>(items: &mut Vec<T>, from: usize, to: usize) -> bool {
    if from >= items.len() || to >= items.len() || from == to {
        return false;
    }
    let item = items.remove(from);
    items.insert(to, item);
    true
}

type RowBuilder<T> = Rc<dyn Fn(&T, usize, &mut Window, &mut App) -> AnyElement>;
type LabelBuilder<T> = Rc<dyn Fn(&T) -> SharedString>;
type ReorderHandler = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

pub struct SortableList<T: 'static> {
    id: ElementId,
    items: Vec<T>,
    row: Option<RowBuilder<T>>,
    drag_label: Option<LabelBuilder<T>>,
    on_reorder: Option<ReorderHandler>,
    disabled: bool,
    show_handle: bool,
}

crate::impl_into_element!([T: 'static] SortableList<T>);

impl<T: 'static> SortableList<T> {
    pub fn new(id: impl Into<ElementId>, items: Vec<T>) -> Self {
        Self {
            id: id.into(),
            items,
            row: None,
            drag_label: None,
            on_reorder: None,
            disabled: false,
            show_handle: true,
        }
    }

    pub fn row(
        mut self,
        row: impl Fn(&T, usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.row = Some(Rc::new(row));
        self
    }

    pub fn drag_label(mut self, label: impl Fn(&T) -> SharedString + 'static) -> Self {
        self.drag_label = Some(Rc::new(label));
        self
    }

    pub fn on_reorder(
        mut self,
        handler: impl Fn(usize, usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reorder = Some(Rc::new(handler));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn show_handle(mut self, show: bool) -> Self {
        self.show_handle = show;
        self
    }
}

impl<T: 'static> RenderOnce for SortableList<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let accent = theme.colors.accent_primary;
        let hover_background = accent.opacity(theme.alpha.hover);
        let drop_background = accent.opacity(theme.alpha.faint);
        let controller_mode = super::util::ControllerMode::is_active(cx);
        let handle_tooltip = if controller_mode {
            cx.t().controller_requires_keyboard()
        } else {
            cx.t().ui_drag_to_reorder()
        };
        let list_id = self.id.clone();
        let reorderable = !self.disabled && self.on_reorder.is_some();
        let sortable = reorderable && !controller_mode;
        let mut rows = Vec::with_capacity(self.items.len());
        for (index, item) in self.items.iter().enumerate() {
            let content = match &self.row {
                Some(row) => row(item, index, window, cx),
                None => div().into_any_element(),
            };
            let label = self
                .drag_label
                .as_ref()
                .map(|label| label(item))
                .unwrap_or_default();
            let payload = SortableDrag {
                list: list_id.clone(),
                index,
                label,
            };
            let drop_list = list_id.clone();
            let accept_list = list_id.clone();
            let on_reorder = self.on_reorder.clone();
            let handle = (self.show_handle && reorderable).then(|| {
                div()
                    .id(("yp-sortable-handle", index))
                    .flex_none()
                    .p(px(2.))
                    .rounded(Sizes::CONTROL_RADIUS)
                    .when(!sortable, |this| this.opacity(theme.alpha.half).cursor_not_allowed())
                    .child(
                        Icon::new(IconName::DragHandle)
                            .size(IconSize::Medium)
                            .color(Color::Muted),
                    )
                    .tooltip(Tooltip::text(handle_tooltip))
                    .when(sortable, |this| {
                        this.cursor(CursorStyle::OpenHand)
                            .hover(move |style| style.bg(hover_background))
                            .on_drag(payload.clone(), |payload: &SortableDrag, _, _, cx| {
                                let label = payload.label.clone();
                                cx.new(|_| SortablePreview { label })
                            })
                    })
            });
            let row = h_flex()
                .id(("yp-sortable-row", index))
                .w_full()
                .gap(sizes.spacing_md)
                .border_b_2()
                .border_color(theme.colors.transparent)
                .children(handle)
                .child(div().flex_1().min_w_0().child(content))
                .when(sortable, |this| {
                    this.can_drop(move |dragged, _, _| {
                        dragged
                            .downcast_ref::<SortableDrag>()
                            .is_some_and(|drag| drag.list == accept_list && drag.index != index)
                    })
                    .drag_over::<SortableDrag>(move |style, drag, _, _| {
                        let style = style.bg(drop_background).border_color(accent);
                        if drag.index < index {
                            style
                        } else {
                            style.border_b_0().border_t_2()
                        }
                    })
                    .on_drop(move |drag: &SortableDrag, window, cx| {
                        if drag.list != drop_list || drag.index == index {
                            return;
                        }
                        if let Some(handler) = &on_reorder {
                            handler(drag.index, index, window, cx);
                        }
                    })
                });
            rows.push(row);
        }
        v_flex().id(self.id).w_full().children(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reorder_moves_an_item_to_its_final_index() {
        let mut items = vec!['a', 'b', 'c', 'd'];
        assert!(reorder(&mut items, 0, 2));
        assert_eq!(items, vec!['b', 'c', 'a', 'd']);
        assert!(reorder(&mut items, 3, 0));
        assert_eq!(items, vec!['d', 'b', 'c', 'a']);
    }

    #[test]
    fn reorder_ignores_out_of_range_and_no_op_moves() {
        let mut items = vec![1, 2, 3];
        assert!(!reorder(&mut items, 1, 1));
        assert!(!reorder(&mut items, 5, 0));
        assert!(!reorder(&mut items, 0, 3));
        assert_eq!(items, vec![1, 2, 3]);
    }
}
