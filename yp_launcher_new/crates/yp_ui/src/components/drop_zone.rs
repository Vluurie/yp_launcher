use std::path::{Path, PathBuf};
use std::rc::Rc;

use gpui::{
    App, ElementId, ExternalPaths, FontWeight, IntoElement, RenderOnce, SharedString, Window, div,
    prelude::*, relative, svg,
};

use super::icon::{IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LINE_HEIGHT_TIGHT};
use super::link::Link;
use super::progress::{ProgressBar, Spinner};
use super::text::Text;
use super::util::KeyboardFocus;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt};
use crate::traits::ActionHandler;

#[derive(Debug, Clone, PartialEq)]
pub struct DropProgress {
    pub text: SharedString,
    pub percent: Option<f32>,
    pub detail: Option<SharedString>,
}

impl DropProgress {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            percent: None,
            detail: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn percent(mut self, percent: f32) -> Self {
        self.percent = Some(percent);
        self
    }
}

type AcceptFilter = Rc<dyn Fn(&Path) -> bool>;
type DropHandler = Rc<dyn Fn(&[PathBuf], &mut Window, &mut App)>;

pub fn accepted_paths(paths: &[PathBuf], accept: Option<&AcceptFilter>) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|path| accept.is_none_or(|accept| accept(path)))
        .cloned()
        .collect()
}

pub struct DropZone {
    id: ElementId,
    title: SharedString,
    hint: Option<SharedString>,
    icon: IconName,
    accept: Option<AcceptFilter>,
    busy: Option<DropProgress>,
    disabled: bool,
    on_drop_paths: Option<DropHandler>,
    on_browse_files: Option<ActionHandler>,
    on_browse_folder: Option<ActionHandler>,
    folder_label: Option<SharedString>,
    footer: Option<SharedString>,
}

crate::impl_into_element!(DropZone);

impl DropZone {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: SharedString::default(),
            hint: None,
            icon: IconName::AddBoxOutlined,
            accept: None,
            busy: None,
            disabled: false,
            on_drop_paths: None,
            on_browse_files: None,
            on_browse_folder: None,
            folder_label: None,
            footer: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }

    pub fn accept(mut self, accept: impl Fn(&Path) -> bool + 'static) -> Self {
        self.accept = Some(Rc::new(accept));
        self
    }

    pub fn busy(mut self, progress: impl Into<Option<DropProgress>>) -> Self {
        self.busy = progress.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_drop_paths(
        mut self,
        handler: impl Fn(&[PathBuf], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_drop_paths = Some(Rc::new(handler));
        self
    }

    pub fn on_browse_files(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_browse_files = Some(Rc::new(handler));
        self
    }

    pub fn on_browse_folder(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_browse_folder = Some(Rc::new(handler));
        self
    }

    pub fn folder_label(mut self, label: impl Into<SharedString>) -> Self {
        self.folder_label = Some(label.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<SharedString>) -> Self {
        self.footer = Some(footer.into());
        self
    }
}

impl RenderOnce for DropZone {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let accent = theme.colors.accent_primary;
        let group = SharedString::from(format!("yp-drop-zone-{}", self.id));
        let interactive = !self.disabled && self.busy.is_none() && !super::util::ControllerMode::is_active(cx);
        let hover_background = accent.opacity(theme.alpha.faint);
        let hover_border = accent.opacity(theme.alpha.half);
        let drag_background = accent.opacity(theme.alpha.subtle);
        let accept = self.accept;
        let can_accept = accept.clone();
        let on_drop = self.on_drop_paths;
        let on_browse_files = self.on_browse_files;
        let clickable = interactive && on_browse_files.is_some();

        let body = match self.busy {
            Some(progress) => v_flex()
                .w_full()
                .items_center()
                .gap(sizes.spacing_md)
                .child(
                    h_flex()
                        .w_full()
                        .justify_center()
                        .gap(sizes.spacing_md)
                        .child(Spinner::new().size(IconSize::Medium))
                        .child(
                            v_flex()
                                .min_w_0()
                                .text_size(sizes.font_md)
                                .line_height(relative(LINE_HEIGHT_TIGHT))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.colors.text_secondary)
                                .child(Text::new(progress.text)),
                        ),
                )
                .when_some(progress.detail, |this, detail| {
                    this.child(
                        v_flex()
                            .w_full()
                            .truncate()
                            .text_center()
                            .text_size(sizes.font_xs)
                            .line_height(relative(LINE_HEIGHT_BODY))
                            .text_color(theme.colors.text_muted)
                            .child(Text::new(detail)),
                    )
                })
                .child(
                    div()
                        .w_full()
                        .max_w(relative(0.7))
                        .child(match progress.percent {
                            Some(percent) => {
                                ProgressBar::new("yp-drop-zone-progress", percent, 100.)
                            }
                            None => ProgressBar::indeterminate("yp-drop-zone-progress"),
                        }),
                )
                .into_any_element(),
            None => {
                let folder_label = self
                    .folder_label
                    .unwrap_or_else(|| SharedString::new_static(strings.drop_zone_browse_folder()));
                v_flex()
                    .w_full()
                    .items_center()
                    .gap(sizes.spacing_sm)
                    .child(
                        h_flex()
                            .w_full()
                            .justify_center()
                            .gap(sizes.spacing_md)
                            .child(
                                svg()
                                    .path(self.icon.path())
                                    .size(sizes.icon_md)
                                    .flex_none()
                                    .text_color(theme.colors.text_muted)
                                    .group_drag_over::<ExternalPaths>(
                                        group.clone(),
                                        move |style| style.text_color(accent),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .text_size(sizes.font_md)
                                    .line_height(relative(LINE_HEIGHT_TIGHT))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.colors.text_secondary)
                                    .group_drag_over::<ExternalPaths>(group.clone(), move |style| {
                                        style.text_color(accent)
                                    })
                                    .child(Text::new(self.title)),
                            ),
                    )
                    .when_some(self.hint, |this, hint| {
                        this.child(
                            v_flex()
                                .w_full()
                                .text_center()
                                .text_size(sizes.font_sm)
                                .line_height(relative(LINE_HEIGHT_BODY))
                                .text_color(theme.colors.text_muted)
                                .child(Text::new(hint)),
                        )
                    })
                    .when_some(
                        self.on_browse_folder.filter(|_| interactive),
                        |this, handler| {
                            this.child(
                                Link::new("yp-drop-zone-folder", folder_label)
                                    .icon(IconName::FolderOpen)
                                    .color(Color::Muted)
                                    .on_click(move |_, window, cx| handler(window, cx)),
                            )
                        },
                    )
                    .when_some(self.footer, |this, footer| {
                        this.child(
                            v_flex()
                                .w_full()
                                .text_center()
                                .text_size(sizes.font_xs)
                                .line_height(relative(LINE_HEIGHT_BODY))
                                .text_color(theme.colors.text_muted)
                                .child(Text::new(footer)),
                        )
                    })
                    .into_any_element()
            }
        };

        div()
            .id(self.id)
            .group(group)
            .w_full()
            .px(sizes.card_padding_h)
            .py(sizes.drop_zone_padding_v / 2.)
            .rounded(sizes.border_radius)
            .border_1()
            .border_dashed()
            .border_color(theme.colors.border_medium)
            .bg(theme.colors.surface_light)
            .when(self.disabled || super::util::ControllerMode::is_active(cx), |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(interactive, |this| {
                this.can_drop(move |dragged, _, _| {
                    dragged
                        .downcast_ref::<ExternalPaths>()
                        .is_some_and(|paths| {
                            !accepted_paths(paths.paths(), can_accept.as_ref()).is_empty()
                        })
                })
                .drag_over::<ExternalPaths>(move |style, _, _, _| {
                    style.bg(drag_background).border_color(accent)
                })
                .on_drop(move |paths: &ExternalPaths, window, cx| {
                    let accepted = accepted_paths(paths.paths(), accept.as_ref());
                    if accepted.is_empty() {
                        return;
                    }
                    if let Some(handler) = &on_drop {
                        handler(&accepted, window, cx);
                    }
                })
            })
            .when(clickable, |this| {
                this.cursor_pointer()
                    .keyboard_focus(cx)
                    .hover(move |style| style.bg(hover_background).border_color(hover_border))
                    .when_some(on_browse_files, |this, handler| {
                        this.on_click(move |_, window, cx| handler(window, cx))
                    })
            })
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_filter_keeps_only_matching_paths_in_order() {
        let paths = vec![
            PathBuf::from("a.zip"),
            PathBuf::from("b.txt"),
            PathBuf::from("c.7z"),
        ];
        let accept: AcceptFilter = Rc::new(|path: &Path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension == "zip" || extension == "7z")
        });
        assert_eq!(
            accepted_paths(&paths, Some(&accept)),
            vec![PathBuf::from("a.zip"), PathBuf::from("c.7z")]
        );
        assert_eq!(accepted_paths(&paths, None), paths);
        assert!(accepted_paths(&[], Some(&accept)).is_empty());
    }
}
