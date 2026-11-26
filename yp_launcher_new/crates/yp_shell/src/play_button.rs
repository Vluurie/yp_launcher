use gpui::{App, Entity, FontWeight, HitboxBehavior, IntoElement, RenderOnce, Window, canvas};
use yp_state::prelude::*;
use yp_state::{Play, PlayState, Stop};
use yp_ui::prelude::*;
use yp_ui::{ButtonLike, ButtonSize, ButtonStyle, IconSize, Sizes, Spinner, TintColor, Tooltip};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayButtonModel {
    pub state: PlayState,
    pub enabled: bool,
    pub can_launch: bool,
    pub tint: TintColor,
}

impl PlayButtonModel {
    pub fn resolve(state: PlayState, can_play: bool, searching: bool, can_launch: bool) -> Self {
        let tint = if !can_launch {
            TintColor::Muted
        } else if state == PlayState::Running {
            TintColor::Error
        } else if !can_play {
            TintColor::Muted
        } else {
            TintColor::Success
        };
        Self {
            state,
            enabled: can_play && !searching && can_launch,
            can_launch,
            tint,
        }
    }

    pub fn read(cx: &App) -> Self {
        let session = cx.stores().session.read(cx);
        Self::resolve(
            session.play_state(),
            session.can_play(),
            session.auto_searching(),
            session.can_launch_game(),
        )
    }

    pub fn is_running(&self) -> bool {
        self.state == PlayState::Running
    }

    pub fn is_loading(&self) -> bool {
        self.state == PlayState::Loading
    }
}

fn activate(running: bool, window: &mut Window, cx: &mut App) {
    if running {
        window.dispatch_action(Box::new(Stop), cx);
    } else {
        window.dispatch_action(Box::new(Play), cx);
    }
}

pub struct PlayButton {
    compact: bool,
    show_label: bool,
}

yp_ui::impl_into_element!(PlayButton);

impl PlayButton {
    pub fn hero() -> Self {
        Self {
            compact: false,
            show_label: true,
        }
    }

    pub fn compact(show_label: bool) -> Self {
        Self {
            compact: true,
            show_label,
        }
    }
}

impl RenderOnce for PlayButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let model = PlayButtonModel::read(cx);
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let theme = cx.theme().clone();
        let running = model.is_running();
        let label = if running {
            strings.stop_button()
        } else {
            strings.play_button()
        };
        let (id, hover_id) = if self.compact {
            ("shell-play-compact", "shell-play-compact-hover")
        } else {
            ("shell-play", "shell-play-hover")
        };
        if theme.id == ThemeId::NierNew {
            let compact = self.compact;
            let loading = model.is_loading();
            let status = if loading {
                strings.status_launching()
            } else {
                label
            };
            let tint = if model.enabled {
                model.tint
            } else {
                TintColor::Muted
            };
            let show_label = !compact || (self.show_label && !loading);
            let height = if compact {
                sizes.font_sm * 2.1
            } else {
                sizes.play_button_height
            };
            let icon = if running {
                IconName::StopRounded
            } else {
                IconName::PlayArrowRounded
            };
            let symbol = if loading {
                Spinner::new()
                    .size(IconSize::Medium)
                    .color(Color::OnAccent)
                    .into_any_element()
            } else {
                Icon::new(icon)
                    .size(if compact {
                        IconSize::Small
                    } else {
                        IconSize::Large
                    })
                    .color(Color::OnAccent)
                    .into_any_element()
            };
            let action = ButtonLike::new_square(id)
                .accessible_label(status)
                .style(ButtonStyle::Solid(tint))
                .size(ButtonSize::Compact)
                .padding(px(0.), px(0.))
                .full_width()
                .disabled(!model.enabled || loading)
                .child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .min_h(height)
                        .items_stretch()
                        .child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(!compact, |el| {
                                    el.w(height)
                                        .bg(theme.colors.button_text.opacity(0.10))
                                        .border_r_1()
                                        .border_color(theme.colors.button_text.opacity(0.35))
                                })
                                .when(compact, |el| el.pl(px(9.)).pr(px(3.)))
                                .child(symbol),
                        )
                        .when(show_label, |row| {
                            row.child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .px(px(if compact { 5. } else { 10. }))
                                    .child(
                                        Label::new(status)
                                            .size(if compact {
                                                LabelSize::Small
                                            } else {
                                                LabelSize::XLarge
                                            })
                                            .color(Color::OnAccent)
                                            .semibold()
                                            .centered(),
                                    ),
                            )
                        }),
                )
                .when(!model.can_launch, |button| {
                    button.tooltip(Tooltip::text(strings.play_disabled_tooltip()))
                })
                .when(compact && !show_label, |button| {
                    button.tooltip(Tooltip::text(status))
                })
                .on_click(move |_, w, cx| activate(running, w, cx));
            return div()
                .min_w_0()
                .max_w_full()
                .when(compact, |el| el.w_full())
                .when(!compact, |el| el.w(sizes.play_button_width))
                .child(action)
                .into_any_element();
        }
        let hover_state = window.use_keyed_state(hover_id, cx, |_, _| false);
        let hovered = model.enabled && *hover_state.read(cx);
        let color = if hovered {
            hover_color(model.tint, cx)
        } else {
            model.tint.color()
        };
        let text_color = color.color(cx);
        let button = ButtonLike::new(id)
            .style(ButtonStyle::Tinted(model.tint))
            .disabled(!model.enabled)
            .on_click(move |_, window, cx| activate(running, window, cx));
        let button = if self.compact {
            let icon = if running {
                IconName::StopRounded
            } else {
                IconName::PlayArrowRounded
            };
            button.size(ButtonSize::Medium).full_width().map(|this| {
                if model.is_loading() {
                    this.child(Spinner::new().size(IconSize::Medium).color(Color::Accent))
                } else {
                    this.child(Icon::new(icon).size(IconSize::Large).color(color))
                        .when(self.show_label, |this| {
                            this.child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(sizes.font_sm)
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_color)
                                    .child(label),
                            )
                        })
                }
            })
        } else {
            button.size(ButtonSize::Hero).map(|this| {
                if model.is_loading() {
                    this.child(
                        Spinner::new()
                            .size(IconSize::Custom(sizes.play_button_loading_size))
                            .color(Color::Accent),
                    )
                } else {
                    this.child(
                        div()
                            .text_size(sizes.font_xxl)
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_color)
                            .child(label),
                    )
                }
            })
        };
        let button = if !model.can_launch {
            button.tooltip(Tooltip::text(strings.play_disabled_tooltip()))
        } else if self.compact && !self.show_label {
            button.tooltip(Tooltip::text(label))
        } else {
            button
        };
        let probe = hover_probe(hover_state.clone());
        v_flex()
            .id(hover_id)
            .relative()
            .flex_none()
            .when(self.compact, |this| this.w_full())
            .rounded(Sizes::CONTROL_RADIUS)
            .bg(theme.colors.surface_light)
            .on_hover(move |hovered, _, cx| set_hovered(&hover_state, *hovered, cx))
            .child(button)
            .child(probe)
            .into_any_element()
    }
}

fn hover_color(tint: TintColor, cx: &App) -> Color {
    ButtonStyle::Tinted(tint).colors(cx).hover_fg
}

fn set_hovered(state: &Entity<bool>, hovered: bool, cx: &mut App) {
    state.update(cx, |state, cx| {
        if *state != hovered {
            *state = hovered;
            cx.notify();
        }
    });
}

fn hover_probe(state: Entity<bool>) -> impl IntoElement {
    canvas(
        |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
        move |_, hitbox, window, cx| {
            let hovered = !cx.has_active_drag() && hitbox.is_hovered(window);
            if *state.read(cx) != hovered {
                window.defer(cx, move |_, cx| set_hovered(&state, hovered, cx));
            }
        },
    )
    .absolute()
    .inset_0()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_is_enabled_only_when_everything_allows_it() {
        let ready = PlayButtonModel::resolve(PlayState::Idle, true, false, true);
        assert!(ready.enabled);
        assert_eq!(ready.tint, TintColor::Success);

        let searching = PlayButtonModel::resolve(PlayState::Idle, true, true, true);
        assert!(!searching.enabled);

        let no_game = PlayButtonModel::resolve(PlayState::Idle, false, false, true);
        assert!(!no_game.enabled);
        assert_eq!(no_game.tint, TintColor::Muted);

        let unsupported = PlayButtonModel::resolve(PlayState::Idle, true, false, false);
        assert!(!unsupported.enabled);
        assert_eq!(unsupported.tint, TintColor::Muted);
    }

    #[test]
    fn running_game_turns_the_button_into_stop() {
        let running = PlayButtonModel::resolve(PlayState::Running, true, false, true);
        assert!(running.enabled);
        assert!(running.is_running());
        assert_eq!(running.tint, TintColor::Error);

        let loading = PlayButtonModel::resolve(PlayState::Loading, false, false, true);
        assert!(loading.is_loading());
        assert!(!loading.enabled);
    }
}
