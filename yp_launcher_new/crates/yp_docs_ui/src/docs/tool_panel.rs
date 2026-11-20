use gpui::{AnyView, Context, IntoElement, Render, SharedString, Window, prelude::*, px};
use yp_ui::prelude::*;
use yp_ui::{CardPadding, Disclosure};

use crate::builders::ToolKind;

pub struct ToolPanel {
    kind: ToolKind,
    open: bool,
    content: AnyView,
}

impl ToolPanel {
    pub fn new(kind: ToolKind, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            kind,
            open: kind.initially_open(),
            content: kind.build(window, cx),
        }
    }

    pub fn kind(&self) -> ToolKind {
        self.kind
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn content(&self) -> &AnyView {
        &self.content
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open != open {
            self.open = open;
            cx.notify();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }
}

impl Render for ToolPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let group = SharedString::from(format!("docs-tool-{}", self.kind.marker_id()));
        let header = h_flex()
            .id(SharedString::from(format!(
                "docs-tool-header-{}",
                self.kind.marker_id()
            )))
            .group(group.clone())
            .w_full()
            .gap(sizes.spacing_lg)
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| this.toggle(cx)))
            .child(
                Icon::new(self.kind.icon())
                    .size(IconSize::Medium)
                    .color(Color::Accent),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        v_flex()
                            .text_color(Color::Default.color(cx))
                            .group_hover(group, |style| style.opacity(0.8))
                            .child(
                                Label::new(self.kind.title(strings))
                                    .size(LabelSize::Default)
                                    .bold(),
                            ),
                    )
                    .child(
                        Label::new(self.kind.subtitle(strings))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .line_height(1.4),
                    ),
            );
        v_flex().w_full().py(sizes.spacing_md).child(
            Card::new()
                .accent(Color::Accent)
                .padding(if self.open {
                    CardPadding::Default
                } else {
                    CardPadding::None
                })
                .header(header)
                .header_end(
                    Disclosure::new(
                        SharedString::from(format!(
                            "docs-tool-toggle-{}",
                            self.kind.marker_id()
                        )),
                        self.open,
                    )
                    .size(IconSize::Medium)
                    .color(Color::Secondary)
                    .on_toggle(cx.listener(|this, _, _, cx| this.toggle(cx))),
                )
                .children(self.open.then(|| self.content.clone())),
        )
    }
}
