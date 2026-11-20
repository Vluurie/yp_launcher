use gpui::App;
use yp_config::fields::nams;
use yp_core::GamePaths;
use yp_i18n::Msg;

use crate::Stores;
use crate::config::ConfigEvent;
use crate::mods::{ModsEvent, ProfilesEvent};
use crate::notifications::{Notice, NoticeKind, NoticeSeverity, NotificationsEvent};
use crate::session::{DirectoryChange, SessionEvent};
use crate::thirdparty::{ThirdPartyEvent, ThirdPartyNotice};

pub(crate) fn bind_game(stores: &Stores, game: Option<GamePaths>, cx: &mut App) {
    stores
        .config
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .nams_settings
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .profiles
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .mods
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .disabled_mods
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .default_mods
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .load_order
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .mod_names
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .thirdparty
        .update(cx, |store, cx| store.bind_game(game.clone(), cx));
    stores
        .detection
        .update(cx, |store, cx| store.bind_game(game, cx));
}

fn third_party_notice(notice: &ThirdPartyNotice) -> Notice {
    match notice {
        ThirdPartyNotice::Migrated { to } => Notice::new(
            notice.id(),
            Msg::NotifyThirdPartyMigrated {
                path: to.to_string_lossy().into_owned(),
            },
            NoticeKind::Migration,
            NoticeSeverity::Success,
        ),
        ThirdPartyNotice::MigrationIncomplete { from, to } => Notice::new(
            notice.id(),
            Msg::NotifyThirdPartyMigrationIncomplete {
                old_path: from.to_string_lossy().into_owned(),
                new_path: to.to_string_lossy().into_owned(),
            },
            NoticeKind::Migration,
            NoticeSeverity::Warning,
        ),
    }
}

fn launch_needs_warning(stores: &Stores, cx: &App) -> bool {
    let values = stores.config.read(cx).nams();
    nams::DISABLE_PLUGIN_LOADING.value_in(values)
        || nams::DISABLE_RESHADE_LOADING.value_in(values)
        || nams::DISABLE_TEXTURE_INJECTION.value_in(values)
}

pub(crate) fn wire(stores: &Stores, cx: &mut App) {
    let all = stores.clone();
    cx.subscribe(&stores.session, move |_, event, cx| match event {
        SessionEvent::DirectoryChanged { game, change } => {
            bind_game(&all, Some(game.clone()), cx);
            all.notifications.update(cx, |store, cx| match change {
                DirectoryChange::Restored => store.run_detections(game, cx),
                DirectoryChange::Selected => store.refresh_detections(game, cx),
            });
            all.thirdparty
                .update(cx, |store, cx| store.migrate_legacy(cx));
        }
        SessionEvent::CacheCleared(result) => {
            if result.nams_settings_deleted {
                all.nams_settings
                    .update(cx, |store, cx| store.reload_if_clean(cx));
            }
            if result.logs_deleted > 0 && all.logs.read(cx).is_streaming() {
                all.logs.update(cx, |store, cx| store.refresh(cx));
            }
        }
        SessionEvent::LaunchRequested { .. } if launch_needs_warning(&all, cx) => {
            all.launch_options
                .update(cx, |store, cx| store.set_show_launch_warnings(true, cx));
        }
        _ => {}
    })
    .detach();

    let all = stores.clone();
    cx.subscribe(&stores.notifications, move |_, event, cx| {
        if matches!(event, NotificationsEvent::DetectionsFinished) {
            all.config.update(cx, |store, cx| store.reload_if_clean(cx));
            all.detection.update(cx, |store, cx| store.invalidate(cx));
        }
    })
    .detach();

    let all = stores.clone();
    cx.subscribe(&stores.config, move |_, event, cx| {
        if let ConfigEvent::Saved(report) = event
            && report.default_outfits_changed
        {
            all.default_mods.update(cx, |store, cx| store.load(cx));
            all.detection.update(cx, |store, cx| store.invalidate(cx));
        }
    })
    .detach();

    let all = stores.clone();
    cx.subscribe(&stores.mods, move |_, event, cx| match event {
        ModsEvent::Installed { .. } | ModsEvent::Uninstalled { .. } => {
            all.config
                .update(cx, |store, cx| store.auto_detect_cutscenes(cx));
            all.detection.update(cx, |store, cx| store.invalidate(cx));
        }
        ModsEvent::ExternalChange(paths) => {
            all.profiles
                .update(cx, |store, cx| store.on_external_change(paths, cx));
            all.disabled_mods
                .update(cx, |store, cx| store.on_external_change(paths, cx));
            all.default_mods
                .update(cx, |store, cx| store.on_external_change(paths, cx));
            all.load_order
                .update(cx, |store, cx| store.on_external_change(paths, cx));
            all.mod_names
                .update(cx, |store, cx| store.on_external_change(paths, cx));
        }
        ModsEvent::Changed | ModsEvent::Failed(_) => {}
    })
    .detach();

    let all = stores.clone();
    cx.subscribe(&stores.profiles, move |_, event, cx| match event {
        ProfilesEvent::Switched { .. }
        | ProfilesEvent::Created { .. }
        | ProfilesEvent::Deleted { .. }
        | ProfilesEvent::Renamed { .. } => {
            all.mods.update(cx, |store, cx| store.rewatch(cx));
            all.detection.update(cx, |store, cx| store.invalidate(cx));
        }
        ProfilesEvent::Changed | ProfilesEvent::Failed(_) => {}
    })
    .detach();

    let all = stores.clone();
    cx.subscribe(&stores.thirdparty, move |_, event, cx| match event {
        ThirdPartyEvent::Notice(notice) => {
            let notice = third_party_notice(notice);
            all.notifications
                .update(cx, |store, cx| store.push(notice, cx));
        }
        ThirdPartyEvent::Changed => {
            all.config.update(cx, |store, cx| store.reload_if_clean(cx));
        }
        ThirdPartyEvent::Failed(_) => {}
    })
    .detach();
}
