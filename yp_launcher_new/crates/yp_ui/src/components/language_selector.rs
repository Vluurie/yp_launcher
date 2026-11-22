use gpui::{App, Anchor, ElementId, IntoElement, RenderOnce, Window, prelude::*, px, relative};
use yp_i18n::{Locale, Strings};

use super::button::{ButtonSize, IconButton};
use super::icon::{Icon, IconName, IconSize};
use super::menu::{ContextMenu, ContextMenuEntry, PopoverMenu};
use super::tooltip::Tooltip;
use crate::actions::{SetLocale, ToggleTheme};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, ThemeId};
use crate::traits::{ButtonCommon, Clickable, Toggleable};

pub struct LanguageSelector {
    id: ElementId,
    icon_size: IconSize,
    anchor: Anchor,
    attach: Anchor,
}

crate::impl_into_element!(LanguageSelector);

impl Default for LanguageSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageSelector {
    pub fn new() -> Self {
        Self {
            id: ElementId::Name("yp-language-selector".into()),
            icon_size: IconSize::Medium,
            anchor: Anchor::TopRight,
            attach: Anchor::BottomRight,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.icon_size = size;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn attach(mut self, attach: Anchor) -> Self {
        self.attach = attach;
        self
    }
}

impl RenderOnce for LanguageSelector {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tooltip = cx.t().tooltip_language();
        let icon_size = self.icon_size;
        PopoverMenu::<ContextMenu>::new(self.id)
            .anchor(self.anchor)
            .attach(self.attach)
            .trigger_with(move |open, _, _| {
                IconButton::new("yp-language-trigger", IconName::Language)
                    .icon_size(icon_size)
                    .icon_color(Color::Accent)
                    .size(ButtonSize::Compact)
                    .toggle_state(open)
                    .tooltip(Tooltip::text(tooltip))
                    .into_any_element()
            })
            .menu(|window, cx| {
                let current = cx.active_locale();
                Some(ContextMenu::build(window, cx, |mut menu, _, _| {
                    for locale in Locale::ALL {
                        let selected = locale == current;
                        menu = menu.item(
                            ContextMenuEntry::new(locale.display_name())
                                .icon(if selected {
                                    IconName::Check
                                } else {
                                    IconName::Language
                                })
                                .icon_color(if selected {
                                    Color::Accent
                                } else {
                                    Color::Secondary
                                })
                                .selected(selected)
                                .handler(move |window, cx| {
                                    window.dispatch_action(Box::new(SetLocale { locale }), cx);
                                }),
                        );
                    }
                    menu.separator().custom_row(move |window, cx| {
                        let sizes = window.sizes(cx);
                        let theme = cx.theme().clone();
                        let notice = cx.t().language_support_notice();
                        let english = Strings::for_locale(Locale::En).language_support_notice();
                        let show_english = cx.active_locale() != Locale::En;
                        h_flex()
                            .items_start()
                            .gap(sizes.spacing_sm)
                            .max_w(px(240.))
                            .child(
                                Icon::new(IconName::InfoOutline)
                                    .size(IconSize::Small)
                                    .color(Color::Warning),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap(sizes.spacing_xs)
                                    .text_size(sizes.font_xs)
                                    .line_height(relative(1.35))
                                    .text_color(theme.colors.text_muted)
                                    .child(v_flex().child(super::text::Text::new(notice)))
                                    .when(show_english, |this| {
                                        this.child(
                                            v_flex()
                                                .italic()
                                                .text_color(
                                                    theme
                                                        .colors
                                                        .text_muted
                                                        .opacity(theme.alpha.glass),
                                                )
                                                .child(super::text::Text::new(english)),
                                        )
                                    }),
                            )
                            .into_any_element()
                    })
                }))
            })
    }
}

pub struct ThemeToggle {
    id: ElementId,
    icon_size: IconSize,
}

crate::impl_into_element!(ThemeToggle);

impl Default for ThemeToggle {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeToggle {
    pub fn new() -> Self {
        Self {
            id: ElementId::Name("yp-theme-toggle".into()),
            icon_size: IconSize::Medium,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.icon_size = size;
        self
    }
}

impl RenderOnce for ThemeToggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let (icon, tooltip) = match cx.theme().id {
            ThemeId::Dark => (IconName::WbSunnyOutlined, strings.theme_toggle_to_nier()),
            ThemeId::Nier => (IconName::DarkModeOutlined, strings.theme_toggle_to_dark()),
        };
        IconButton::new(self.id, icon)
            .icon_size(self.icon_size)
            .icon_color(Color::Accent)
            .size(ButtonSize::Compact)
            .tooltip(Tooltip::text(tooltip))
            .on_click(|_, window, cx| window.dispatch_action(Box::new(ToggleTheme), cx))
    }
}
