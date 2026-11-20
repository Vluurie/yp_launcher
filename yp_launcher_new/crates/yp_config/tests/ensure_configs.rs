mod support;

use std::path::Path;

use serde_json::Value;
use support::{GameDir, parse, table};
use yp_config::fields::{self, AnyField, bindings, content, lodmod, nams, texture_injection};
use yp_config::{ConfigFile, ensure_configs, templates};
use yp_core::{GamePaths, HostEnv, Os};

fn scope<'a>(values: &'a toml::Table, path: &[&str]) -> &'a toml::Table {
    path.iter()
        .fold(values, |current, part| table(current, part))
}

fn assert_default(values: &toml::Table, path: &[&str], field: &dyn AnyField, file: &str) {
    let found = scope(values, path).get(field.meta().key);
    assert_eq!(
        found,
        Some(&field.default_toml()),
        "{file}: {} does not match the registry default",
        field.meta().key
    );
}

#[test]
fn every_template_parses_as_toml_1_0() {
    for (file, text) in templates::shipped() {
        parse(text);
        assert!(yp_core::toml_io::parse(text).is_ok(), "{file:?}");
    }
    for text in templates::migration_templates() {
        parse(text);
    }
}

#[test]
fn template_defaults_match_the_registry() {
    let nams_values = parse(templates::NAMS);
    for field in nams::ALL {
        match field.meta().section {
            None => assert_default(&nams_values, &[], field, "nams.toml"),
            Some("cutscene") => assert_default(&nams_values, &["cutscene"], field, "nams.toml"),
            Some("mouse") => {}
            Some(other) => panic!("unexpected nams section {other}"),
        }
    }
    for field in content::all() {
        assert_default(&nams_values, &["content"], field, "nams.toml");
    }

    let naiom_values = parse(templates::NAIOM);
    for field in nams::MOUSE {
        assert_default(&naiom_values, &[], field, "naiom.toml");
    }
    for field in bindings::all() {
        assert_default(&naiom_values, &["bindings"], field, "naiom.toml");
    }

    let lodmod_values = parse(templates::LODMOD);
    for field in lodmod::ALL {
        let path: Vec<&str> = field.meta().section_path();
        assert_default(&lodmod_values, &path, field, "lodmod.toml");
    }

    let texture_values = parse(templates::TEXTURE_INJECTION);
    for field in texture_injection::ALL {
        assert_default(&texture_values, &[], field, "texture_injection.toml");
    }
}

#[test]
fn shipped_templates_are_written_unchanged_on_a_fresh_install() {
    let game = GameDir::empty();
    let report = ensure_configs(&game.game, &game.host).unwrap();
    assert_eq!(
        report.created,
        vec![
            ConfigFile::Nams,
            ConfigFile::LodMod,
            ConfigFile::TextureInjection,
            ConfigFile::Naiom
        ]
    );
    assert!(report.migrated.is_empty());
    assert!(report.overlay_settings_moved_from.is_none());
    for (file, text) in templates::shipped() {
        assert_eq!(game.read_nams_file(file.file_name()), text, "{file:?}");
    }
}

fn snapshot(game: &GamePaths) -> Vec<(ConfigFile, String)> {
    ConfigFile::ALL
        .into_iter()
        .map(|file| (file, std::fs::read_to_string(file.path(game)).unwrap()))
        .collect()
}

const LEGACY_FIXTURES: [&str; 4] = [
    "validate_model_data = false\n",
    "content_items = false\n\n[content]\nnovels = false\n",
    "# header\nvalidate_model_data = true\ncontent_mcd = false\n\n[mouse]\nfix_aim_acceleration = true\nevade_key = 70\n[mouse.bindings]\nnon_standard_evade = \"\"\n",
    "[cutscene]\nhd_cutscenes = true\n",
];

#[test]
fn ensure_configs_twice_is_byte_identical() {
    for fixture in [None].into_iter().chain(LEGACY_FIXTURES.map(Some)) {
        let game = GameDir::with_nams(fixture);
        game.write_nams_file("lodmod.toml", "enabled = true\n[high_grids]\nfxaa = true\n");
        game.write_nams_file(
            "texture_injection.toml",
            "preload_all = true\nvram_budget_mb = 4096\n",
        );
        game.ensure();
        let first = snapshot(&game.game);
        let report = ensure_configs(&game.game, &game.host).unwrap();
        assert!(!report.changed(), "{fixture:?}: {report:?}");
        assert_eq!(snapshot(&game.game), first, "{fixture:?}");
    }
}

#[test]
fn every_written_file_parses_as_toml_1_0() {
    for fixture in [None].into_iter().chain(LEGACY_FIXTURES.map(Some)) {
        let game = GameDir::with_nams(fixture);
        game.ensure();
        for (file, text) in snapshot(&game.game) {
            let strict: Result<toml::Table, _> = toml::from_str(&text);
            assert!(strict.is_ok(), "{fixture:?} {file:?}:\n{text}");
        }
    }
}

#[test]
fn legacy_evade_key_lands_in_the_old_bindings_table_and_the_new_naiom() {
    let game = GameDir::with_nams(Some(LEGACY_FIXTURES[2]));
    game.ensure();
    let nams_values = game.nams();
    assert_eq!(
        scope(&nams_values, &["mouse", "bindings"])["non_standard_evade"].as_str(),
        Some("F")
    );
    assert_eq!(
        table(&nams_values, "mouse")["aim_mode"].as_bool(),
        Some(true)
    );
    assert!(!table(&nams_values, "mouse").contains_key("evade_key"));
    assert_eq!(
        table(&game.naiom(), "bindings")["non_standard_evade"].as_str(),
        Some("F")
    );
    assert!(game.raw_nams().starts_with("# header\n"));
}

#[test]
fn an_existing_naiom_toml_is_never_regenerated() {
    let game = GameDir::with_nams(Some("[mouse]\nsensitivity = 9.0\n"));
    game.write_nams_file("naiom.toml", "sensitivity = 1.0\n");
    game.ensure();
    assert_eq!(game.raw_naiom(), "sensitivity = 1.0\n");
}

#[test]
fn texture_injection_preload_keys_are_removed_and_the_old_budget_reset() {
    let game = GameDir::empty();
    game.write_nams_file(
        "texture_injection.toml",
        "# budget\nvram_budget_mb = 4096\npreload_max_dimension = 2048\n\nstreaming_enabled = false\n",
    );
    game.ensure();
    let raw = game.read_nams_file("texture_injection.toml");
    let values = parse(&raw);
    assert_eq!(values["vram_budget_mb"].as_integer(), Some(0));
    assert!(!values.contains_key("preload_max_dimension"));
    assert_eq!(values["streaming_enabled"].as_bool(), Some(false));
    assert!(raw.starts_with("# budget\nvram_budget_mb = 0\nstreaming_enabled = false\n"));
    for field in texture_injection::ALL {
        if field.meta().key != "disabled_packs" {
            assert!(
                values.contains_key(field.meta().key),
                "{}",
                field.meta().key
            );
        }
    }
}

#[test]
fn every_registry_label_and_tooltip_has_text_in_every_locale() {
    for locale in yp_i18n::Locale::ALL {
        let strings = locale.strings();
        for field in fields::all() {
            let meta = field.meta();
            assert!(!strings.get(meta.label).is_empty(), "{} label", meta.key);
            if let Some(tooltip) = meta.tooltip {
                assert!(!strings.get(tooltip).is_empty(), "{} tooltip", meta.key);
            }
        }
        for group in content::GROUPS {
            assert!(!strings.get(group.title).is_empty());
        }
    }
}

fn write_settings(dir: &Path, text: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("settings.json"), text).unwrap();
}

#[test]
fn overlay_settings_move_from_appdata_on_windows() {
    let root = tempfile::tempdir().unwrap();
    let app_data = root.path().join("Roaming");
    write_settings(
        &app_data.join("NAMS"),
        "{\"impeller\": true, \"soundEnabled\": false}",
    );
    let game = GamePaths::new(root.path().join("game"));
    let host = HostEnv::for_test(
        Os::Windows,
        &[("APPDATA", app_data.to_str().unwrap())],
        root.path(),
    );

    let report = ensure_configs(&game, &host).unwrap();
    assert_eq!(
        report.overlay_settings_moved_from,
        Some(app_data.join("NAMS").join("settings.json"))
    );
    assert!(!app_data.join("NAMS").join("settings.json").exists());
    let moved: Value = serde_json::from_str(
        &std::fs::read_to_string(game.internal_cache_dir().join("settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(moved["impeller"], Value::Bool(false));
    assert_eq!(moved["soundEnabled"], Value::Bool(false));

    write_settings(&app_data.join("NAMS"), "{\"impeller\": true}");
    let second = ensure_configs(&game, &host).unwrap();
    assert!(
        second.overlay_settings_moved_from.is_none(),
        "an existing target is never overwritten"
    );
}

#[test]
fn overlay_settings_move_from_the_wine_prefix_roaming_dir() {
    let root = tempfile::tempdir().unwrap();
    let prefix = root.path().join("bottle");
    let roaming = prefix
        .join("drive_c")
        .join("users")
        .join("steamuser")
        .join("AppData")
        .join("Roaming");
    write_settings(
        &roaming.join("NAMS"),
        "{\"impeller\": false, \"overlayLanguage\": \"de\"}",
    );
    let game = GamePaths::new(prefix.join("drive_c").join("Games").join("NieRAutomata"));
    let host = HostEnv::for_test(
        Os::Linux,
        &[("HOME", root.path().join("home").to_str().unwrap())],
        root.path(),
    );

    let report = ensure_configs(&game, &host).unwrap();
    assert!(report.overlay_settings_moved_from.is_some());
    let moved = std::fs::read_to_string(game.internal_cache_dir().join("settings.json")).unwrap();
    assert_eq!(moved, "{\"impeller\": false, \"overlayLanguage\": \"de\"}");
    assert!(!roaming.join("NAMS").join("settings.json").exists());
}

#[test]
fn legacy_settings_roots_cover_prefix_steam_and_appdata() {
    let root = tempfile::tempdir().unwrap();
    let library = root.path().join("library");
    let game_dir = library
        .join("steamapps")
        .join("common")
        .join("NieRAutomata");
    let host = HostEnv::for_test(
        Os::Linux,
        &[
            ("HOME", root.path().join("home").to_str().unwrap()),
            ("APPDATA", "/appdata"),
        ],
        root.path(),
    );
    let roots = yp_config::migrate::legacy_settings_roots(&game_dir, &host);
    assert_eq!(roots.len(), 2);
    let steam_root = yp_core::wine_paths::to_posix(&roots[0]);
    assert!(
        steam_root.contains("steamapps/compatdata/524220/pfx/drive_c/users/"),
        "{steam_root}"
    );
    assert!(steam_root.ends_with("/AppData/Roaming"));
    assert_eq!(roots[1], Path::new("/appdata"));

    let windows = HostEnv::for_test(Os::Windows, &[], root.path());
    assert!(yp_config::migrate::legacy_settings_roots(&game_dir, &windows).is_empty());
}
