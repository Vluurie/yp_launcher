use super::*;
use crate::text_reveal::{self, Cadence};
use gpui::{Context, Render, ScrollStrategy, SharedString, TestAppContext};
use std::{cell::RefCell, collections::BTreeMap};

type Labels = Rc<RefCell<BTreeMap<usize, SharedString>>>;

#[derive(IntoElement)]
struct Label(usize, Labels);

impl RenderOnce for Label {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = format!("Texture Pack {}", self.0).into();
        let shown = text_reveal::display(&label, ("label", self.0), 20, Cadence::Row, window, cx);
        self.1
            .borrow_mut()
            .entry(self.0)
            .or_insert_with(|| shown.clone());
        div().child(shown)
    }
}

struct Catalog {
    scroll: UniformListScrollHandle,
    labels: Labels,
}

impl Render for Catalog {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let labels = self.labels.clone();
        div().w(px(510.)).h(px(512.)).child(
            MenuList::new("mods", 20_000, move |i, _, _| {
                div().child(Label(i, labels.clone()))
            })
            .track_scroll(self.scroll.clone()),
        )
    }
}

#[gpui::test]
fn distant_and_revisited_rows_show_their_full_label_on_first_layout(cx: &mut TestAppContext) {
    let scroll = UniformListScrollHandle::new();
    let labels = Labels::default();
    let (view, cx) = cx.add_window_view(|_, _| Catalog {
        scroll: scroll.clone(),
        labels: labels.clone(),
    });
    cx.run_until_parked();
    assert!(scroll.0.borrow().base_handle.bounds().size.height > px(0.));
    for first in [19_992, 0, 12_000] {
        labels.borrow_mut().clear();
        view.update(cx, |_, cx| {
            scroll.scroll_to_item_strict(first, ScrollStrategy::Top);
            cx.notify();
        });
        cx.run_until_parked();
        let visible = labels.borrow();
        for i in first..first + 8 {
            assert_eq!(visible[&i].as_ref(), format!("Texture Pack {i}"));
        }
        assert!(visible.len() <= 9);
    }
}
