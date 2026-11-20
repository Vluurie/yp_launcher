use std::cell::RefCell;
use std::rc::Rc;

use futures::StreamExt;
use gpui::TestAppContext;
use yp_mods::defaults::DefaultKind;
use yp_mods::{InstallFailure, InstallRequest, ProfileRejection};

use super::Harness;
use crate::mods::{ModsEvent, ProfilesEvent, progress_channel};
use crate::{DetectionEvent, Stores};

fn mod_ids(stores: &Stores, cx: &mut TestAppContext) -> Vec<String> {
    stores.mods.read_with(cx, |mods, _| {
        mods.mods()
            .iter()
            .map(|installed| installed.id.clone())
            .collect()
    })
}

fn install(harness: &Harness, stores: &Stores, name: &str, cx: &mut TestAppContext) {
    let source = harness.write_data_mod(name);
    let task = stores.mods.update(cx, |mods, cx| {
        mods.install(source, InstallRequest::named(name), None, cx)
    });
    cx.run_until_parked();
    let installed = cx.foreground_executor().block_test(task).unwrap();
    assert_eq!(installed.id, name);
    cx.run_until_parked();
}

#[gpui::test]
fn installing_a_mod_refreshes_the_list_and_dependents(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    assert!(mod_ids(&stores, cx).is_empty());

    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    let generation = stores
        .detection
        .read_with(cx, |detection, _| detection.generation());
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.mods, move |_, event, _| {
            if let ModsEvent::Installed { ids } = event {
                seen.borrow_mut().extend(ids.iter().cloned());
            }
        })
    });

    install(&harness, &stores, "kimono", cx);

    assert_eq!(mod_ids(&stores, cx), vec!["kimono".to_owned()]);
    assert_eq!(*events.borrow(), vec!["kimono".to_owned()]);
    assert!(harness.game.mods_dir().join("kimono").is_dir());
    assert!(!stores.mods.read_with(cx, |mods, _| mods.is_installing()));
    stores.detection.read_with(cx, |detection, _| {
        assert!(detection.generation() > generation);
        assert_eq!(detection.snapshot().map(|snapshot| snapshot.mods), Some(1));
    });
}

#[gpui::test]
fn install_reports_progress_and_typed_failures(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    install(&harness, &stores, "kimono", cx);

    let source = harness.write_data_mod("kimono");
    let (progress, mut updates) = progress_channel();
    let task = stores.mods.update(cx, |mods, cx| {
        mods.install(source, InstallRequest::named("kimono"), Some(progress), cx)
    });
    cx.run_until_parked();
    let failure = cx.foreground_executor().block_test(task).unwrap_err();

    assert_eq!(
        failure,
        InstallFailure::Exists {
            id: "kimono".to_owned()
        }
    );
    assert!(cx.foreground_executor().block_test(updates.next()).is_none());
    assert_eq!(mod_ids(&stores, cx).len(), 1);
}

#[gpui::test]
fn uninstalling_removes_the_mod_and_announces_it(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    install(&harness, &stores, "kimono", cx);
    let removed = Rc::new(RefCell::new(Vec::new()));
    let seen = removed.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.mods, move |_, event, _| {
            if let ModsEvent::Uninstalled { id } = event {
                seen.borrow_mut().push(id.clone());
            }
        })
    });

    let task = stores
        .mods
        .update(cx, |mods, cx| mods.uninstall("kimono", false, cx));
    cx.run_until_parked();

    assert!(cx.foreground_executor().block_test(task));
    cx.run_until_parked();
    assert!(mod_ids(&stores, cx).is_empty());
    assert_eq!(*removed.borrow(), vec!["kimono".to_owned()]);
    assert!(!harness.game.mods_dir().join("kimono").exists());
}

#[gpui::test]
fn disabling_renaming_and_ordering_persist(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    install(&harness, &stores, "kimono", cx);
    install(&harness, &stores, "armor", cx);

    stores.disabled_mods.update(cx, |disabled, cx| {
        disabled.set_disabled("mods/kimono", true, cx);
    });
    stores.mod_names.update(cx, |names, cx| {
        names.rename("kimono", Some("Summer Kimono"), cx);
    });
    stores.load_order.update(cx, |order, cx| {
        order.save(vec!["mods/armor".to_owned(), "mods/kimono".to_owned()], cx);
    });
    cx.run_until_parked();

    stores.disabled_mods.read_with(cx, |disabled, _| {
        assert!(disabled.is_disabled("mods/kimono"));
        assert!(!disabled.is_disabled("mods/armor"));
    });
    stores.mod_names.read_with(cx, |names, _| {
        assert_eq!(names.custom_name_of("kimono"), Some("Summer Kimono"));
    });
    stores.load_order.read_with(cx, |order, _| {
        assert_eq!(order.index_of("mods/armor"), Some(0));
        assert_eq!(order.index_of("mods/kimono"), Some(1));
    });
    assert!(harness.game.disabled_mods_toml().is_file());
    assert!(harness.game.mod_load_order_toml().is_file());

    stores.disabled_mods.update(cx, |disabled, cx| {
        disabled.set_disabled("mods/kimono", false, cx);
    });
    stores
        .mod_names
        .update(cx, |names, cx| names.rename("kimono", None, cx));
    cx.run_until_parked();
    assert!(
        !stores
            .disabled_mods
            .read_with(cx, |disabled, _| disabled.is_disabled("mods/kimono"))
    );
    assert!(
        stores
            .mod_names
            .read_with(cx, |names, _| names.custom_name_of("kimono").is_none())
    );
}

#[gpui::test]
fn quick_toggles_are_applied_in_order(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    stores.disabled_mods.update(cx, |disabled, cx| {
        for toggle in [true, false, true, false, true] {
            disabled.set_disabled("mods/kimono", toggle, cx);
        }
    });
    cx.run_until_parked();

    assert!(
        stores
            .disabled_mods
            .read_with(cx, |disabled, _| disabled.is_disabled("mods/kimono"))
    );
}

#[gpui::test]
fn switching_profiles_reloads_the_dependent_stores(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    install(&harness, &stores, "kimono", cx);
    stores.disabled_mods.update(cx, |disabled, cx| {
        disabled.set_disabled("mods/kimono", true, cx);
    });
    cx.run_until_parked();

    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    let invalidations = Rc::new(RefCell::new(0));
    let counted = invalidations.clone();
    let _subscriptions = cx.update(|cx| {
        [
            cx.subscribe(&stores.profiles, move |_, event: &ProfilesEvent, _| {
                seen.borrow_mut().push(event.clone());
            }),
            cx.subscribe(&stores.detection, move |_, event, _| {
                if matches!(event, DetectionEvent::Invalidated { .. }) {
                    *counted.borrow_mut() += 1;
                }
            }),
        ]
    });

    let created = stores
        .profiles
        .update(cx, |profiles, cx| profiles.create("hard run", cx));
    cx.run_until_parked();
    assert!(cx.foreground_executor().block_test(created));
    cx.run_until_parked();

    let active = stores
        .profiles
        .read_with(cx, |profiles, _| profiles.active_name().to_owned());
    stores.profiles.read_with(cx, |profiles, _| {
        assert_eq!(profiles.profiles().len(), 2);
        assert!(!profiles.is_loading());
        assert!(profiles.error().is_none());
    });
    assert_ne!(active, "default");
    assert!(mod_ids(&stores, cx).is_empty());
    assert!(
        !stores
            .disabled_mods
            .read_with(cx, |disabled, _| disabled.is_disabled("mods/kimono"))
    );
    assert!(matches!(
        events.borrow().last(),
        Some(ProfilesEvent::Created { .. })
    ));
    assert!(*invalidations.borrow() >= 1);

    let switched = stores
        .profiles
        .update(cx, |profiles, cx| profiles.switch_to("default", cx));
    cx.run_until_parked();
    assert!(cx.foreground_executor().block_test(switched));
    cx.run_until_parked();

    assert_eq!(mod_ids(&stores, cx), vec!["kimono".to_owned()]);
    assert!(
        stores
            .disabled_mods
            .read_with(cx, |disabled, _| disabled.is_disabled("mods/kimono"))
    );
    assert!(matches!(
        events.borrow().last(),
        Some(ProfilesEvent::Switched { name }) if name == "default"
    ));
}

#[gpui::test]
fn rejected_profile_requests_keep_a_typed_error(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    let deleted = stores
        .profiles
        .update(cx, |profiles, cx| profiles.delete("default", cx));
    cx.run_until_parked();

    assert!(!cx.foreground_executor().block_test(deleted));
    stores.profiles.read_with(cx, |profiles, _| {
        let rejection = profiles.error().and_then(|error| error.profile_rejection());
        assert!(matches!(
            rejection,
            Some(ProfileRejection::CannotDeleteLast | ProfileRejection::CannotDeleteActive)
        ));
        assert!(!profiles.is_loading());
    });
}

#[gpui::test]
fn default_mods_follow_the_feature_switch_on_save(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let set_feature = |enabled: bool, cx: &mut TestAppContext| {
        stores.config.update(cx, |config, cx| {
            config.update_nams(
                yp_config::fields::nams::EXPERIMENTAL_DEFAULT_OUTFITS.key(),
                toml::Value::Boolean(enabled),
                None,
                cx,
            );
            config.save(cx);
        });
        cx.run_until_parked();
    };
    assert!(
        stores
            .default_mods
            .read_with(cx, |defaults, _| defaults.enabled())
    );

    stores.default_mods.update(cx, |defaults, cx| {
        defaults.set_default(
            "mods/kimono/pl/pl000d",
            Some(DefaultKind::OutfitBare),
            None,
            cx,
        );
    });
    cx.run_until_parked();
    stores.default_mods.read_with(cx, |defaults, _| {
        assert!(defaults.is_default("mods/kimono/pl/pl000d"));
        assert_eq!(
            defaults.kind_of("mods/kimono/pl/pl000d"),
            Some(DefaultKind::OutfitBare)
        );
        assert_eq!(defaults.entries_under("mods/kimono").len(), 1);
        assert!(
            defaults
                .would_replace("mods/armor/pl/pl000d", DefaultKind::OutfitConfig)
                .is_some()
        );
    });

    set_feature(false, cx);
    stores.default_mods.read_with(cx, |defaults, _| {
        assert!(!defaults.enabled());
        assert!(defaults.entries().is_empty());
    });

    set_feature(true, cx);
    stores.default_mods.read_with(cx, |defaults, _| {
        assert!(defaults.enabled());
        assert_eq!(defaults.entries().len(), 1);
    });
}

#[gpui::test]
fn external_changes_reload_the_touched_store(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let source = harness.write_data_mod("manual");
    let target = harness.game.mods_dir().join("manual");
    std::fs::create_dir_all(harness.game.mods_dir()).unwrap();
    std::fs::rename(&source, &target).unwrap();

    stores.mods.update(cx, |mods, cx| {
        mods.on_external_change(std::slice::from_ref(&target), cx);
    });
    cx.run_until_parked();
    assert!(mod_ids(&stores, cx).is_empty());

    super::settle(cx, std::time::Duration::from_millis(400));
    assert_eq!(mod_ids(&stores, cx), vec!["manual".to_owned()]);
}

#[gpui::test]
fn auto_named_installs_pick_a_free_name_and_announce_each_mod(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.mods, move |_, event, _| {
            if let ModsEvent::Installed { ids } = event {
                seen.borrow_mut().extend(ids.iter().cloned());
            }
        })
    });

    let mut installed = Vec::new();
    for _ in 0..2 {
        let source = harness.write_data_mod("kimono");
        let task = stores
            .mods
            .update(cx, |mods, cx| mods.install_auto_named(source, "kimono", None, cx));
        cx.run_until_parked();
        installed.push(cx.foreground_executor().block_test(task).unwrap().id);
        cx.run_until_parked();
    }

    assert_eq!(installed.len(), 2);
    assert_ne!(installed[0], installed[1]);
    assert_eq!(*events.borrow(), installed);
    let mut listed = mod_ids(&stores, cx);
    listed.sort();
    installed.sort();
    assert_eq!(listed, installed);
    assert_eq!(
        stores
            .detection
            .read_with(cx, |detection, _| detection.snapshot().map(|snapshot| snapshot.mods)),
        Some(2)
    );
}
