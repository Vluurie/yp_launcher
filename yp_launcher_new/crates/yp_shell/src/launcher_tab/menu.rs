use super::*;
use yp_ui::{Card, ContextMenu, PageHeader, PopoverMenu, ScrollableElement};

impl LauncherTab {
    fn support_menu(&self, cx: &App) -> impl IntoElement + use<> {
        let strings = cx.t();
        PopoverMenu::<ContextMenu>::new("shell-support")
            .accessible_label(strings.menu_support())
            .anchor(Anchor::TopRight)
            .attach(Anchor::BottomRight)
            .trigger(
                Button::new("support-label", strings.menu_support())
                    .style(ButtonStyle::Link)
                    .end_icon(IconName::KeyboardArrowDown),
            )
            .menu(move |w, cx| {
                Some(ContextMenu::build(w, cx, move |m, _, _| {
                    [
                        (strings.tooltip_guide(), GUIDE_URL),
                        (strings.help_discord(), DISCORD_URL),
                        (strings.tooltip_launcher_source(), GITHUB_URL),
                    ]
                    .into_iter()
                    .fold(m, |m, (label, url)| {
                        m.entry(label, move |_, cx| folders::open_link(url, cx))
                    })
                }))
            })
    }

    pub(super) fn menu(&mut self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let s = cx.t();
        let selected = cx.stores().session.read(cx).is_directory_selected();
        let shortcuts = cx.env().platform.supports_desktop_shortcut();
        let colors = cx.theme().colors.clone();
        let command = v_flex()
            .min_w_0()
            .w_full()
            .gap(px(14.))
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .flex_wrap()
                    .gap(px(14.))
                    .child(PlayButton::hero())
                    .child(
                        div()
                            .flex_basis(px(172.))
                            .flex_grow_1()
                            .min_w_0()
                            .child(self.status_line(window, cx)),
                    ),
            )
            .children(Self::nams_dll_line(cx))
            .children(Self::launch_options(cx.env().host.ui_os(), window, cx))
            .when(selected && shortcuts, |el| {
                el.child(
                    Button::new("shell-create-shortcut", s.tooltip_create_shortcut())
                        .style(ButtonStyle::Link)
                        .on_click(cx.listener(|v, _, _, cx| v.create_shortcut(cx))),
                )
            })
            .children(self.warnings(window, cx));
        let installation = Card::new()
            .title(s.menu_game_installation())
            .child(self.directory.clone());
        let status = Card::new()
            .prominent_header()
            .title(s.menu_game_status())
            .child(DetectionStrip::new());
        v_flex()
            .size_full()
            .min_h_0()
            .min_w_0()
            .child(
                PageHeader::new(s.tab_launcher()).actions(
                    h_flex()
                        .min_w_0()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(self.support_menu(cx))
                        .child(ThemeToggle::new().id("shell-theme-toggle"))
                        .child(LanguageSelector::new().id("shell-language")),
                ),
            )
            .child(
                v_flex()
                    .id("home-scroll")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .pb(px(16.))
                    .child(if selected {
                        h_flex()
                            .w_full()
                            .min_w_0()
                            .flex_wrap()
                            .items_start()
                            .justify_between()
                            .gap(px(14.))
                            .child(
                                v_flex()
                                    .flex_basis(px(390.))
                                    .flex_grow_1()
                                    .min_w_0()
                                    .max_w(px(820.))
                                    .border_t_2()
                                    .border_color(colors.accent_primary)
                                    .pt(px(10.))
                                    .gap(px(16.))
                                    .child(command)
                                    .child(installation),
                            )
                            .child(
                                div()
                                    .flex_basis(px(244.))
                                    .flex_grow_1()
                                    .max_w(px(292.))
                                    .min_w_0()
                                    .child(status),
                            )
                            .into_any_element()
                    } else {
                        v_flex()
                            .w_full()
                            .gap(px(12.))
                            .child(installation)
                            .child(command)
                            .into_any_element()
                    })
                    .overflow_y_scrollbar(&self.scroll),
            )
            .into_any_element()
    }
}
