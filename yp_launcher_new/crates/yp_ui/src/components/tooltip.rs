use std::rc::Rc;

use gpui::{
    AnyElement, AnyView, App, Context, Div, IntoElement, Render, SharedString, Window, div,
    prelude::*, px, relative,
};

use super::text::Text;
use crate::styled_ext::{StyledExt, v_flex};
use crate::theme::{ActiveTheme, ResponsiveExt, Sizes};

type TooltipContent = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

pub struct Tooltip {
    title: SharedString,
    meta: Option<SharedString>,
    content: Option<TooltipContent>,
}

impl Tooltip {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            meta: None,
            content: None,
        }
    }

    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    pub fn build(self) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let title = self.title;
        let meta = self.meta;
        move |_, cx| {
            cx.new(|_| Tooltip {
                title: title.clone(),
                meta: meta.clone(),
                content: None,
            })
            .into()
        }
    }

    pub fn text(
        title: impl Into<SharedString>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        Tooltip::new(title).build()
    }

    pub fn with_meta(
        title: impl Into<SharedString>,
        meta: impl Into<SharedString>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        Tooltip::new(title).meta(meta).build()
    }

    pub fn simple(title: impl Into<SharedString>, cx: &mut App) -> AnyView {
        let title = title.into();
        cx.new(|_| Tooltip {
            title,
            meta: None,
            content: None,
        })
        .into()
    }

    pub fn element(
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let content: TooltipContent = Rc::new(content);
        move |_, cx| {
            let content = content.clone();
            cx.new(|_| Tooltip {
                title: SharedString::default(),
                meta: None,
                content: Some(content),
            })
            .into()
        }
    }

    pub fn lines(
        header: impl Into<SharedString>,
        lines: Vec<SharedString>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let header = header.into();
        let body = lines
            .iter()
            .map(|line| format!("\u{2022} {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        Tooltip::new(header).meta(body).build()
    }
}

pub fn tooltip_container(window: &Window, cx: &App) -> Div {
    let theme = cx.theme();
    let sizes = window.sizes(cx);
    div()
        .max_w(Sizes::TOOLTIP_MAX_WIDTH)
        .px(px(10.))
        .py(px(6.))
        .rounded(Sizes::ROW_RADIUS)
        .bg(theme.colors.background_card.opacity(theme.alpha.tooltip))
        .border_1()
        .border_color(
            theme
                .colors
                .accent_primary
                .opacity(theme.alpha.accent_border),
        )
        .elevation_2(cx)
        .font_ui(cx)
        .text_size(sizes.font_xs)
        .line_height(relative(1.35))
        .text_color(theme.colors.text_secondary)
}

pub(super) fn finish_tooltip(body: Div, cx: &App) -> Div {
    body.when(cx.theme().id == crate::ThemeId::NierNew, |el| {
        el.relative()
            .rounded(px(0.))
            .bg(cx.theme().colors.background_card)
            .child(yp_nier_ui::menu_popup_overlay())
    })
}

impl Render for Tooltip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().colors.text_muted;
        let meta_gap = window.sizes(cx).spacing_xs;
        let container = tooltip_container(window, cx);
        let body = match self.content.clone() {
            Some(content) => container.child(content(window, cx)),
            None => container
                .child(v_flex().child(Text::new(self.title.clone())))
                .when_some(self.meta.clone(), |this, meta| {
                    this.child(
                        v_flex()
                            .mt(meta_gap)
                            .text_color(muted)
                            .child(Text::new(meta)),
                    )
                }),
        };
        div().p(px(4.)).child(finish_tooltip(body, cx))
    }
}
