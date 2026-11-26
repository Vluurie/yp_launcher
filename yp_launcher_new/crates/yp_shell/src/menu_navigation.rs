use crate::{play_button::PlayButton, sidebar};
use gpui::Entity;
use std::time::Instant;
use yp_core::consts::APP_VERSION;
use yp_i18n::Strings;
use yp_state::{SidebarSection, prelude::*};
use yp_ui::{nier::PageReveal, prelude::*};

const GROUPS: [&[TabId]; 4] = [
    &[TabId::Launcher, TabId::LauncherSettings],
    &[
        TabId::Mods,
        TabId::Textures,
        TabId::Cutscenes,
        TabId::ThirdParty,
    ],
    SidebarSection::Config.tabs(),
    &[TabId::Tools, TabId::Docs],
];

fn group_label(index: usize, strings: &Strings) -> &'static str {
    match index {
        0 => strings.tab_section_general(),
        1 => strings.menu_library(),
        2 => strings.menu_game_settings(),
        _ => strings.menu_resources(),
    }
}

fn destinations(
    group: &[TabId],
    has_dir: bool,
    optional: sidebar::OptionalTabs,
) -> impl Iterator<Item = TabId> + '_ {
    group
        .iter()
        .copied()
        .filter(move |tab| optional.shows(*tab) && (has_dir || !tab.needs_game_dir()))
}

fn open(tab: TabId, window: &mut Window, cx: &mut App) {
    window.dispatch_action(Box::new(OpenTab::new(tab)), cx);
}

struct IndexState {
    open: bool,
    started: Instant,
    focus: gpui::FocusHandle,
    scroll: gpui::ScrollHandle,
}

fn navigation(
    active: TabId,
    has_dir: bool,
    state: &Entity<IndexState>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let strings = cx.t();
    let colors = cx.theme().colors;
    let sizes = window.sizes(cx);
    let row_height = px(29. * yp_ui::theme::MenuTextSize::factor(cx));
    let optional = sidebar::OptionalTabs::read(cx);
    let scroll = state.read(cx).scroll.clone();
    let age = state.read(cx).started.elapsed().as_secs_f32();
    let motion = !yp_ui::nier::ReducedMotion::enabled(cx) && age < 0.45;
    if motion {
        window.request_animation_frame();
    }
    let mut index = v_flex()
        .id("launcher-sections")
        .debug_selector(|| "launcher-sections".to_owned())
        .role(gpui::Role::Navigation)
        .aria_label(strings.menu_browse_sections())
        .size_full()
        .min_w_0()
        .gap(px(1.))
        .pr(px(9.));
    let mut position = 0;
    for (i, group) in GROUPS.iter().enumerate() {
        let mut pages = destinations(group, has_dir, optional).peekable();
        if pages.peek().is_none() {
            continue;
        }
        index = index.child(
            div()
                .id(("nav-heading", i))
                .flex_shrink_0()
                .role(gpui::Role::Heading)
                .aria_level(2)
                .aria_label(group_label(i, strings))
                .pt(px(if i == 0 { 3. } else { 12. }))
                .pb(px(4.))
                .border_b_1()
                .border_color(colors.border_medium)
                .child(
                    Label::new(group_label(i, strings))
                        .uppercase()
                        .size(LabelSize::XSmall)
                        .semibold()
                        .color(Color::Secondary),
                ),
        );
        for tab in pages {
            let label = if tab == TabId::LodMod {
                strings.menu_graphics()
            } else {
                sidebar::tab_label(tab, strings)
            };
            let selected = active == tab;
            let state = state.clone();
            let progress = if motion {
                ((age - position as f32 * 0.013) / 0.13).clamp(0., 1.)
            } else {
                1.
            };
            position += 1;
            let fg = if selected {
                Color::OnAccent
            } else {
                Color::Default
            };
            let content = |color| {
                h_flex()
                    .w_full()
                    .min_w_0()
                    .min_h(row_height)
                    .gap(px(8.))
                    .px(px(7.))
                    .py(px(3.))
                    .child(
                        Icon::new(sidebar::tab_icon(tab))
                            .size(IconSize::Small)
                            .color(color),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            Label::new(label)
                                .size(LabelSize::Small)
                                .color(color)
                                .when(selected, |l| l.semibold()),
                        ),
                    )
            };
            let hover = menu_hover_fill(
                ("route-hover", tab.index()).into(),
                if selected {
                    colors.border_medium
                } else {
                    colors.accent_primary
                },
                Some(content(Color::OnAccent).into_any_element()),
                window,
                cx,
            );
            index = index.child(
                div()
                    .id(("workspace-route", tab.index()))
                    .role(gpui::Role::Link)
                    .aria_label(label)
                    .keyboard_activate(cx)
                    .relative()
                    .w_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .opacity(progress)
                    .when(selected, |el| el.bg(colors.accent_primary))
                    .cursor_pointer()
                    .child(content(fg))
                    .child(hover)
                    .on_click(move |_, w, cx| {
                        state.update(cx, |n, cx| {
                            if n.open {
                                n.open = false;
                                n.focus.focus(w, cx);
                            }
                            cx.notify();
                        });
                        open(tab, w, cx);
                    })
                    .tour_anchor(tab.tour_anchor_id()),
            );
        }
    }
    index
        .pb(sizes.padding_md)
        .overflow_y_scrollbar(&scroll)
        .into_any_element()
}

fn sections_button(state: &Entity<IndexState>, cx: &App) -> impl IntoElement + use<> {
    let open = state.read(cx).open;
    let label = cx.t().menu_browse_sections();
    let colors = ButtonStyle::Filled.colors(cx);
    let state = state.clone();
    h_flex()
        .id("sections-button")
        .debug_selector(|| "sections-button".to_owned())
        .role(gpui::Role::Button)
        .aria_label(label)
        .aria_expanded(open)
        .track_focus(&state.read(cx).focus)
        .keyboard_activate(cx)
        .cursor_pointer()
        .min_w_0()
        .max_w_full()
        .min_h(px(36. * yp_ui::theme::MenuTextSize::factor(cx)))
        .gap(px(8.))
        .px(px(12.))
        .bg(colors.background)
        .hover(move |el| el.bg(colors.hover_background))
        .active(move |el| el.bg(colors.active_background))
        .child(
            Icon::new(if open { IconName::Close } else { IconName::Menu })
                .size(IconSize::Medium)
                .color(colors.fg),
        )
        .child(Label::new(label).semibold().color(colors.fg))
        .on_disclosure_toggle(open, move |_, w, cx| {
            state.update(cx, |n, cx| {
                n.open = !n.open;
                if n.open {
                    n.started = Instant::now();
                }
                n.focus.focus(w, cx);
                cx.notify();
            });
        })
}

pub(super) fn workspace(
    active: TabId,
    has_dir: bool,
    content: AnyElement,
    start: Instant,
    window: &mut Window,
    cx: &mut App,
) -> (AnyElement, AnyElement) {
    let strings = cx.t();
    let colors = cx.theme().colors;
    let width = f32::from(window.viewport_size().width);
    let margin = px((width * 0.016).clamp(8., 24.));
    let text_factor = yp_ui::theme::MenuTextSize::factor(cx);
    let compact = width < 940. * text_factor;
    let state = window.use_keyed_state("launcher-section-index", cx, |_, cx| IndexState {
        open: false,
        started: Instant::now(),
        focus: cx.focus_handle().tab_stop(true),
        scroll: gpui::ScrollHandle::new(),
    });
    let show_play = active != TabId::Launcher && has_dir;
    let play = || {
        div()
            .debug_selector(|| "workspace-play".to_owned())
            .w_full()
            .child(PlayButton::compact(true))
    };
    let workspace = div()
        .id(("shell-tab-host", active.index()))
        .debug_selector(|| "workspace-page".to_owned())
        .role(gpui::Role::Main)
        .aria_label(sidebar::tab_label(active, strings))
        .relative()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .child(
            div()
                .absolute()
                .inset_0()
                .child(PageReveal::new(content, start)),
        );
    let body = if compact {
        let open = state.read(cx).open;
        let escape_state = state.clone();
        let page = if open {
            navigation(active, has_dir, &state, window, cx)
        } else {
            workspace.into_any_element()
        };
        v_flex()
            .size_full()
            .min_h_0()
            .min_w_0()
            .px(margin)
            .on_key_down(move |event: &gpui::KeyDownEvent, w, cx| {
                if event.keystroke.key == "escape" && escape_state.read(cx).open {
                    escape_state.update(cx, |n, cx| {
                        n.open = false;
                        n.focus.focus(w, cx);
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
            })
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .flex_none()
                    .py(px(4.))
                    .border_b_1()
                    .border_color(colors.border_medium)
                    .child(sections_button(&state, cx)),
            )
            .child(v_flex().flex_1().min_h_0().w_full().child(page))
            .into_any_element()
    } else {
        if state.read(cx).open {
            state.update(cx, |n, _| n.open = false);
        }
        h_flex()
            .size_full()
            .min_h_0()
            .min_w_0()
            .items_stretch()
            .px(margin)
            .gap(margin)
            .child(
                v_flex()
                    .w(px((width * 0.157).clamp(162., 228.)))
                    .h_full()
                    .pt(px(10.))
                    .flex_none()
                    .border_r_1()
                    .border_color(colors.border_medium)
                    .child(
                        div().relative().flex_1().min_h_0().w_full().child(
                            div().absolute().inset_0().child(navigation(active, has_dir, &state, window, cx)),
                        ),
                    )
                    .when(show_play, |el| {
                        el.child(div().w_full().flex_none().pr(px(9.)).py(px(6.)).child(play()))
                    }),
            )
            .child(workspace)
            .into_any_element()
    };
    let footer = h_flex()
        .debug_selector(|| "workspace-footer".to_owned())
        .when(compact, |el| el.flex_wrap())
        .w_full()
        .min_w_0()
        .flex_none()
        .gap(px(12.))
        .px(margin)
        .py(px(4.))
        .when(compact && show_play, |el| {
            el.child(div().w(px(108. * text_factor)).max_w_full().child(play()))
        })
        .when(!compact, |el| {
            el.child(
                Label::new(format!("YoRHa Protocol   {APP_VERSION}"))
                    .size(LabelSize::XSmall)
                    .color(Color::Secondary),
            )
        })
        .child(div().flex_1())
        .when_some(window.toast_stack(cx).and_then(|stack| {
            stack.read(cx).history_button(stack.downgrade(), cx, true)
        }), |el, history| {
            el.child(div().flex_none().debug_selector(|| "footer-history".to_owned()).child(history))
        })
        .child(
            Button::new("workspace-logs", strings.info_bar_logs())
                .style(ButtonStyle::Link)
                .start_icon(IconName::DescriptionOutlined)
                .on_click(|_, w, cx| w.dispatch_action(Box::new(OpenLogs), cx)),
        );
    (body, footer.into_any_element())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_launcher_destination_has_one_home() {
        for tab in TabId::ALL {
            assert_eq!(GROUPS.iter().filter(|g| g.contains(&tab)).count(), 1);
        }
    }
}
