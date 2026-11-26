use super::model::InspectorModel;
use gpui::Anchor;
use yp_inspector::MotionSources;
use yp_state::inspector::ArchivePaths;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{Button, ButtonSize, ButtonStyle, IconName, PopoverMenu, SelectMenu, SelectMenuEntry};

pub(super) fn sources(archives: &ArchivePaths, cx: &App) -> MotionSources {
    let stores = cx.stores();
    let paths = archives.dat.as_ref().or(archives.dtt.as_ref());
    let game = stores.session.read(cx).game();
    let profiles = stores.profiles.read(cx);
    let mut libraries = Vec::new();
    if let Some(game) = game.as_ref() {
        libraries.push(game.mods_dir());
        libraries.extend(
            profiles
                .profiles()
                .iter()
                .filter(|profile| profile.name != profiles.active_name())
                .map(|profile| yp_mods::profiles::profile_mods_dir(game, &profile.name)),
        );
    }
    MotionSources {
        mod_root: paths
            .and_then(|p| {
                stores
                    .mods
                    .read(cx)
                    .mods()
                    .iter()
                    .find(|m| p.starts_with(&m.root_path))
            })
            .map(|m| m.root_path.clone()),
        game_data: game.map(|g| g.data_dir()),
        mod_libraries: libraries,
    }
}

pub(super) fn playback_label(player: &yp_state::ModelSession, cx: &App) -> &'static str {
    let s = cx.t();
    match (player.motion_index(cx).is_some(), player.motion_playing(cx)) {
        (true, true) => s.motion_pause(),
        (true, false) => s.motion_play(),
        (false, true) => s.cloth_pause(),
        (false, false) => s.cloth_play(),
    }
}

pub(super) fn controls(
    id: &'static str,
    model: &Entity<InspectorModel>,
    cx: &App,
) -> Option<AnyElement> {
    let s = cx.t();
    let state = model.read(cx);
    let session = state.session()?.clone();
    let player = session.read(cx);
    if !player.is_ready() {
        return None;
    }
    if !player.has_playback() {
        return None;
    }
    let play = session.clone();
    let pick = session.clone();
    let row = h_flex()
        .w_full()
        .min_w_0()
        .flex_wrap()
        .gap(px(6.))
        .px(px(6.))
        .py(px(4.));
    let toggle = playback_label(player, cx);
    let clip = (!player.motion_names().is_empty()).then(|| {
        let label: SharedString = player
            .motion_index(cx)
            .and_then(|i| player.motion_names().get(i))
            .map_or_else(|| s.motion_clip().into(), |name| name.clone().into());
        PopoverMenu::<SelectMenu>::new((id, 0usize))
            .accessible_label(format!("{} · {label}", s.motion_clip()))
            .anchor(Anchor::TopLeft)
            .attach(Anchor::BottomLeft)
            .trigger(
                Button::new((id, 1usize), label)
                    .size(ButtonSize::Compact)
                    .style(ButtonStyle::Link)
                    .end_icon(IconName::KeyboardArrowDown),
            )
            .menu(move |window, cx| {
                let state = pick.read(cx);
                let entries = state
                    .motion_names()
                    .iter()
                    .map(|name| SelectMenuEntry {
                        label: name.clone().into(),
                        icon: None,
                        disabled: false,
                    })
                    .collect();
                let index = state.motion_index(cx);
                let target = pick.clone();
                Some(cx.new(|cx| {
                    SelectMenu::new(
                        entries,
                        index,
                        true,
                        move |i, _, cx| {
                            target.update(cx, |state, cx| state.select_motion(i, cx));
                        },
                        window,
                        cx,
                    )
                }))
            })
    });
    Some(
        row.when_some(clip, |row, clip| {
            row.child(div().flex_1().min_w_0().child(clip))
        })
        .child(
            Button::new((id, 2usize), toggle)
                .disabled(!player.can_play(cx))
                .size(ButtonSize::Compact)
                .style(ButtonStyle::Outlined)
                .on_click(move |_, _, cx| play.update(cx, |state, cx| state.toggle_motion(cx))),
        )
        .into_any_element(),
    )
}
