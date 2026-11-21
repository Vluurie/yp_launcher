mod support;

use indexmap::IndexMap;
use jiff::{SignedDuration, Timestamp};
use support::{FakeGame, Rig, bare_report, manifest, messages, pe_header, write};
use serde_json::json;
use yp_config::testing::sample_schema;
use yp_config::{Schema, ValidateReport, Values};
use yp_core::consts::{NAMS_EXE_NAME, YORHA_DLL_NAME};
use yp_health::diagnostics::model::{
    ConfigReport, DISCORD_LIMIT, DlcSource, FilesystemRole, FindingLevel, GameIdentity, PackSwitch,
    VanillaBucket, exe_variant_name,
};
use yp_health::diagnostics::{
    build_findings, collect_config, format_full_report, format_summary, pack_disabled_by, redact,
};
use yp_health::probes::parse_findmnt;
use yp_host::game_probe::{
    self, DLC_APP_ID, ExeBuildStatus, ExeVariant, GameBuild, STEAM_2021_TIME_DATE_STAMP,
    STEAM_2026_SHA1, STEAM_2026_SIGNED_SHA1, STEAM_2026_SIGNED_SIZE, STEAM_2026_SIZE,
    STEAM_2026_TIME_DATE_STAMP, SteamAppManifest, WINDOWS10_SHA1, WINDOWS10_SIZE, build_of_exe,
    build_of_time_date_stamp, pe_time_date_stamp, probe_exe,
};
use yp_host::launch::record::{self, LaunchRecord};
use yp_host::nams_cli::NamsTexturePack;
use yp_thirdparty::{
    GameModEntry, GameModSettings, GameModsInfo, RuntimeStatus, ThirdPartyStatuses,
};

#[test]
fn vanilla_drops_list_loose_files_in_data_not_cpk_or_subfolders() {
    let rig = Rig::native();
    let game = FakeGame::create();

    let report = rig.collect(Some(game.path()));
    let paths: Vec<&str> = report
        .vanilla_dropped
        .iter()
        .map(|drop| drop.path.as_str())
        .collect();

    assert!(paths.contains(&"data/hack.dll"));
    assert!(paths.contains(&"data/enemy.dtt"));
    assert!(!paths.contains(&"data/data000.cpk"));
    assert!(!paths.contains(&"data/sound/bgm.wsp"));
    let hack = report
        .vanilla_dropped
        .iter()
        .find(|drop| drop.path == "data/hack.dll")
        .unwrap();
    assert_eq!(hack.bucket, VanillaBucket::DataRootLoose);
    assert_eq!(hack.bucket.label(), "data-root-loose");
}

#[test]
fn game_identity_detects_dlc_and_a_supported_exe() {
    let rig = Rig::native();
    let game = FakeGame::create().with_dlc_archive();

    let report = rig.collect(Some(game.path()));
    let identity = &report.game_identity;
    assert!(identity.has_dlc);
    assert_eq!(identity.dlc_source, DlcSource::SizeCheck);
    assert_eq!(identity.dlc_source.label(), "size check");
    assert_eq!(identity.exe_size, Some(WINDOWS10_SIZE));
    assert_eq!(identity.build_status, ExeBuildStatus::Unknown);
    assert!(!identity.exe_variant_supported());
    assert!(report.findings.iter().any(|finding| {
        finding.level == FindingLevel::Error && finding.message.contains("not a build NAMS knows")
    }));
    assert_eq!(identity.exe_variant, ExeVariant::Original);
    assert!(
        [
            "missing",
            "original",
            "wolfLimitBreak",
            "legacyWindows7",
            "unknown"
        ]
        .contains(&exe_variant_name(identity.exe_variant))
    );
}

#[test]
fn exe_with_the_2026_pe_stamp_but_another_sha1_is_a_modified_copy() {
    let rig = Rig::native();
    let game = FakeGame::create();
    write(&game.exe(), pe_header(STEAM_2026_TIME_DATE_STAMP, 0x80));

    let probe = probe_exe(game.path());
    assert_eq!(probe.time_date_stamp, Some(STEAM_2026_TIME_DATE_STAMP));
    assert_eq!(probe.status(), ExeBuildStatus::Modified);
    assert_eq!(
        probe.stamp_build().map(|build| build.build),
        Some(GameBuild::Steam2026_11)
    );

    let report = rig.collect(Some(game.path()));
    assert!(
        report
            .game_identity
            .build_display()
            .contains("Steam 2026-11")
    );
    let first = &report.findings[0].message;
    assert!(first.contains("build stamp of Steam 2026-11"));
    assert!(first.contains("modified copy"));
}

#[test]
fn exe_newer_than_every_known_build_asks_for_a_launcher_update() {
    let rig = Rig::native();
    let game = FakeGame::create();
    write(&game.exe(), pe_header(STEAM_2026_TIME_DATE_STAMP + 1, 0x80));

    let report = rig.collect(Some(game.path()));
    assert_eq!(report.game_identity.build_status, ExeBuildStatus::Newer);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.message.contains("Update the launcher"))
    );
}

#[test]
fn known_builds_map_size_and_sha1_to_2021_and_2026() {
    assert_eq!(
        build_of_exe(WINDOWS10_SIZE, WINDOWS10_SHA1).map(|build| build.build),
        Some(GameBuild::Steam2021)
    );
    assert_eq!(
        build_of_exe(STEAM_2026_SIZE, STEAM_2026_SHA1).map(|build| build.build),
        Some(GameBuild::Steam2026_11)
    );
    let signed = build_of_exe(STEAM_2026_SIGNED_SIZE, STEAM_2026_SIGNED_SHA1).unwrap();
    assert_eq!(signed.build, GameBuild::Steam2026_11);
    assert_eq!(signed.label, "Steam 2026-11 (signed)");
    assert!(build_of_exe(STEAM_2026_SIZE, WINDOWS10_SHA1).is_none());
    assert_eq!(
        build_of_time_date_stamp(Some(STEAM_2021_TIME_DATE_STAMP)).map(|build| build.label),
        Some("Steam 2021")
    );
    assert_eq!(pe_time_date_stamp(&[0x4d, 0x5a]), None);
    assert_eq!(
        pe_time_date_stamp(&pe_header(0x1234_5678, 0x40)),
        Some(0x1234_5678)
    );
}

#[test]
fn steam_manifest_gives_the_build_id_and_the_dlc_depot() {
    let rig = Rig::native();
    let game = FakeGame::create_at(&["steamapps", "common", "NieRAutomata"]);
    let steamapps = game.dir.path().join("steamapps");
    std::fs::create_dir_all(game.path().join("data")).unwrap();
    write(&game.exe(), pe_header(STEAM_2021_TIME_DATE_STAMP, 0x80));
    write(&game.path().join("NieRAutomataCompat.exe"), [0u8]);
    write(
        &game.path().join("Latest_BuildInfo.properties"),
        "Last_SVN_REVISION=7788 \n",
    );
    write(
        &steamapps.join("appmanifest_524220.acf"),
        manifest("25901063", ""),
    );

    assert!(game_probe::has_dlc(game.path()));

    let report = rig.collect(Some(game.path()));
    let identity = &report.game_identity;
    let steam = identity.steam_manifest.as_ref().unwrap();
    assert_eq!(steam.build_id.as_deref(), Some("25901062"));
    assert!(steam.update_pending());
    assert_eq!(identity.dlc_source, DlcSource::SteamManifest);
    assert_eq!(identity.dlc_source.label(), "Steam manifest");
    assert_eq!(
        identity
            .build_info
            .get("Last_SVN_REVISION")
            .map(String::as_str),
        Some("7788")
    );
    assert!(identity.compat_exe_present);
    assert!(
        !report
            .game_root_extras
            .iter()
            .any(|name| name == "NieRAutomataCompat.exe")
    );
    assert!(
        !report
            .game_root_extras
            .iter()
            .any(|name| name == "Latest_BuildInfo.properties")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.message.contains("update pending"))
    );

    let disabled = SteamAppManifest::parse(&manifest("25901062", DLC_APP_ID)).unwrap();
    assert!(!disabled.dlc_installed(DLC_APP_ID));
}

#[test]
fn findmnt_output_flags_ntfs_game_folders() {
    let fuse = parse_findmnt(
        "{\"filesystems\": [{\"target\": \"/mnt/games\", \"fstype\": \"fuseblk\", \"options\": \"rw,relatime\"}]}",
    )
    .unwrap();
    assert!(fuse.risky());
    assert_eq!(fuse.target, "/mnt/games");
    let ext4 = parse_findmnt(
        "{\"filesystems\": [{\"target\": \"/\", \"fstype\": \"ext4\", \"options\": \"rw\"}]}",
    )
    .unwrap();
    assert!(!ext4.risky());

    let mut report = bare_report("");
    report.filesystems = IndexMap::from([(FilesystemRole::GameFolder, Some(fuse))]);
    let findings = build_findings(&report);
    assert!(
        findings
            .iter()
            .any(|finding| finding.level == FindingLevel::Error
                && finding.message.contains("fuseblk"))
    );
}

fn settings_report(values: serde_json::Value, problems: serde_json::Value) -> ConfigReport {
    let schema: Schema = serde_json::from_value(sample_schema()).unwrap();
    let values: Values = serde_json::from_value(values).unwrap();
    let validate: ValidateReport =
        serde_json::from_value(json!({ "valid": false, "problems": problems })).unwrap();
    collect_config(&schema, &values, Some(&validate))
}

#[test]
fn settings_diff_covers_every_key_unknown_keys_and_parse_errors() {
    let config = settings_report(
        json!({
            "files": {
                "nams.toml": { "exists": true },
                "naiom.toml": { "exists": true },
                "lodmod.toml": { "exists": true, "error": "line 1, column 11: expected value" },
                "texture_injection.toml": { "exists": false },
            },
            "not_in_file": ["naiom.misc_custom_cursor_menu"],
            "values": {
                "nams.disable_plugin_loading": false,
                "naiom.sensitivity": 1.7000000000000002,
                "naiom.misc_custom_cursor_menu": "cursor.png",
                "lodmod.lod_multiplier": 5.0,
            },
        }),
        json!([
            { "code": "unknown_key", "severity": "warning", "message": "unknown key", "file": "nams.toml", "key": "disable_splash_screeen" },
            { "code": "deprecated_key", "severity": "warning", "message": "moved to naiom.toml", "file": "nams.toml", "key": "mouse.sensitivity" },
            { "code": "deprecated_key", "severity": "warning", "message": "removed", "file": "nams.toml", "key": "cutscene.hd_cutscenes" },
            { "code": "out_of_range", "severity": "error", "message": "too big", "id": "lodmod.lod_multiplier" },
        ]),
    );
    let unknown: Vec<&str> = config
        .unknown_keys
        .iter()
        .map(|delta| delta.key.as_str())
        .collect();
    assert_eq!(unknown, ["disable_splash_screeen"]);
    let legacy: Vec<&str> = config
        .legacy_keys
        .iter()
        .map(|delta| delta.key.as_str())
        .collect();
    assert_eq!(legacy, ["mouse.sensitivity", "cutscene.hd_cutscenes"]);
    assert!(config.legacy_keys[0].note.as_deref().unwrap().contains("naiom.toml"));
    assert_eq!(config.deltas.len(), 1);
    assert_eq!(config.deltas[0].file, "naiom.toml");
    assert_eq!(config.deltas[0].key, "sensitivity");
    assert_eq!(config.deltas[0].value, "1.7");
    assert_eq!(config.deltas[0].default_value, "2.0");
    assert_eq!(config.missing_files, ["texture_injection.toml"]);
    assert!(config.parse_errors.contains_key("lodmod.toml"));

    let mut report = bare_report("/games/NieRAutomata");
    report.config = config;
    report.config_raw = IndexMap::from([("lodmod.toml".to_owned(), "enabled = = true\n".to_owned())]);
    let findings = build_findings(&report);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("disable_splash_screeen"))
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.message.contains("hd_cutscenes"))
    );
    assert!(findings.iter().any(|finding| {
        finding.level == FindingLevel::Error
            && finding.message.starts_with("lodmod.toml does not parse")
    }));
    let full = format_full_report(&report);
    assert!(full.contains("nams/lodmod.toml (does not parse)"));
    assert!(full.contains("enabled = = true"));
    assert!(full.contains("  [nams.toml] mouse.sensitivity"));
    assert!(!full.contains("nams/nams.toml\n"));
}

#[test]
fn last_launch_crash_dump_and_hidden_mod_folders_reach_the_findings() {
    let rig = Rig::native();
    let game = FakeGame::create();
    std::fs::create_dir_all(game.nams().join("mods").join("_parked")).unwrap();

    let started = Timestamp::now() - SignedDuration::from_secs(20);
    let launched = LaunchRecord {
        started_at: started,
        ended_at: None,
        exit_code: None,
        label: "NAMS".to_owned(),
        output_tail: String::new(),
    };
    record::write(&rig.paths, &launched.finished(3, "boom\nstack line")).unwrap();
    rig.write_log(
        "crashdump_2026-11-06.txt",
        "EXCEPTION_ACCESS_VIOLATION\nat game.bin+0x1234\n",
    );

    let report = rig.collect(Some(game.path()));
    assert_eq!(
        report.last_launch.as_ref().and_then(|last| last.exit_code),
        Some(3)
    );
    assert_eq!(report.logs.crash_dump_count, 1);
    assert_eq!(
        report.logs.newest_crash_dump.as_ref().unwrap().lines[0],
        "EXCEPTION_ACCESS_VIOLATION"
    );
    assert_eq!(report.hidden_mod_folders, ["_parked"]);
    let messages = messages(&report);
    assert!(messages.contains("exit code 3"));
    assert!(messages.contains("crash dump in the last session"));
    assert!(messages.contains("start with \"_\""));

    let full = format_full_report(&report);
    assert!(full.contains("exit code: 3"));
    assert!(full.contains("stack line"));
    assert!(full.contains("at game.bin+0x1234"));
}

fn pack(game_dir: &std::path::Path, name: &str, enabled: bool) -> NamsTexturePack {
    NamsTexturePack {
        name: name.to_owned(),
        source: "standalone".to_owned(),
        mod_id: None,
        character: None,
        outfit_conditional: false,
        enabled,
        dds_count: 1,
        load_order_index: None,
        path: game_dir
            .join("nams")
            .join("inject")
            .join("textures")
            .join(name)
            .to_string_lossy()
            .replace('\\', "/"),
    }
}

#[test]
fn texture_packs_disabled_in_disabled_mods_toml_are_not_shown_enabled() {
    let game_dir = std::env::temp_dir().join("NieRAutomata");
    let mut report = bare_report(&game_dir.to_string_lossy());
    report.disabled_mods_entries = vec!["inject/textures/Main Menu".to_owned()];
    report.texture_packs = vec![
        pack(&game_dir, "Main Menu", true),
        pack(&game_dir, "Main Menu Extra", true),
        pack(&game_dir, "Off", false),
    ];
    report.texture_packs_available = true;

    assert_eq!(
        pack_disabled_by(&report, &report.texture_packs[0]),
        Some(PackSwitch::DisabledModsToml)
    );
    assert_eq!(
        PackSwitch::DisabledModsToml.file_name(),
        "disabled_mods.toml"
    );
    assert_eq!(pack_disabled_by(&report, &report.texture_packs[1]), None);
    assert_eq!(
        pack_disabled_by(&report, &report.texture_packs[2]),
        Some(PackSwitch::TextureInjectionToml)
    );
    assert_eq!(
        PackSwitch::TextureInjectionToml.file_name(),
        "texture_injection.toml"
    );
    let full = format_full_report(&report);
    assert!(full.contains("Main Menu  [standalone]  disabled (disabled_mods.toml)"));
    assert!(full.contains("Main Menu Extra  [standalone]  enabled  1 dds"));
    assert!(full.contains("Off  [standalone]  disabled (texture_injection.toml)"));
}

#[test]
fn game_root_proxy_dlls_are_flagged_plugin_files_in_data_are_not() {
    let rig = Rig::native();
    let game = FakeGame::create();
    for name in ["hack.dll", "enemy.dtt"] {
        std::fs::remove_file(game.path().join("data").join(name)).unwrap();
    }
    write(
        &game.path().join("data").join("nao-nier-settings-0.bin"),
        [0u8],
    );
    write(&game.path().join("dinput8.dll"), [0u8]);
    write(&game.path().join("d3d11.dll"), [0u8]);
    std::fs::create_dir_all(game.path().join("SK_Res")).unwrap();

    let report = rig.collect(Some(game.path()));
    assert!(
        report
            .vanilla_dropped
            .iter()
            .any(|drop| drop.path == "data/nao-nier-settings-0.bin")
    );
    let messages = messages(&report);
    assert!(!messages.contains("dropped loose"));
    assert!(messages.contains("d3d11.dll in the game folder is never loaded"));
    assert!(messages.contains("Special K (dinput8.dll in the game folder)"));
    assert!(!messages.contains("dinput8.dll in the game folder is never"));
}

fn with_game_dll(settings: GameModSettings) -> ThirdPartyStatuses {
    ThirdPartyStatuses {
        game_mods: RuntimeStatus {
            installed: true,
            enabled: true,
            game_mods_info: Some(GameModsInfo {
                mods: vec![GameModEntry {
                    file_name: "celestial.dll".to_owned(),
                    settings: Some(settings),
                    ..GameModEntry::default()
                }],
                has_config: false,
            }),
            ..RuntimeStatus::default()
        },
        ..ThirdPartyStatuses::default()
    }
}

fn joined(findings: &[yp_health::DiagnosticFinding]) -> String {
    findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect()
}

#[test]
fn game_dll_mods_from_2021_on_the_2026_build_are_flagged() {
    let on_2026 = GameIdentity {
        build_status: ExeBuildStatus::Known,
        build_label: Some("Steam 2026-11"),
        ..GameIdentity::default()
    };

    let mut report = bare_report("");
    report.game_identity = on_2026.clone();
    report.third_party = with_game_dll(GameModSettings::default());
    assert!(joined(&build_findings(&report)).contains("celestial.dll"));

    let mut told_2026 = bare_report("");
    told_2026.game_identity = on_2026;
    told_2026.third_party = with_game_dll(GameModSettings {
        size_of_image: GameModSettings::STEAM_2026_SIZE_OF_IMAGE,
        ..GameModSettings::default()
    });
    assert!(!joined(&build_findings(&told_2026)).contains("celestial.dll"));

    let mut signed = bare_report("");
    signed.game_identity = GameIdentity {
        build_status: ExeBuildStatus::Known,
        build_label: Some("Steam 2026-11 (signed)"),
        ..GameIdentity::default()
    };
    signed.third_party = with_game_dll(GameModSettings::default());
    assert!(joined(&build_findings(&signed)).contains("celestial.dll"));
}

#[test]
fn launch_record_survives_a_json_round_trip() {
    let start: Timestamp = "2026-11-06T10:00:00Z".parse().unwrap();
    let launched = LaunchRecord {
        started_at: start,
        ended_at: None,
        exit_code: None,
        label: "Proton".to_owned(),
        output_tail: String::new(),
    };
    let finished = launched.finished(0, "ok");
    let back = LaunchRecord::from_json(&finished.to_json()).unwrap();
    assert_eq!(back.started_at, start);
    assert_eq!(back.exit_code, Some(0));
    assert_eq!(back.label, "Proton");
    assert_eq!(back.output_tail, "ok");
    assert!(launched.still_running());
}

#[test]
fn redaction_covers_wine_style_and_forward_slash_paths() {
    let report = bare_report("/home/alice/Games/NieRAutomata");
    let out = redact(
        "Z:\\home\\alice\\Games\\NieRAutomata\\data and Z:/home/alice/Games/NieRAutomata/nams and /home/alice/Games/NieRAutomata/logs",
        &report,
    );
    assert!(!out.contains("alice"));
    assert_eq!(out.matches("<gameDir>").count(), 3);
}

#[test]
fn username_is_only_redacted_inside_home_paths() {
    let mut report = bare_report("");

    report.host.user_name = "Windows".to_owned();
    let win = redact(
        "OS: \"Windows 10 Home\" at D:\\Users\\Windows\\AppData and C:\\Windows\\System32",
        &report,
    );
    assert!(win.contains("\"Windows 10 Home\""));
    assert!(win.contains("D:\\Users\\<user>\\AppData"));
    assert!(win.contains("C:\\Windows\\System32"));

    report.host.user_name = "user".to_owned();
    let linux = redact(
        "user-added files in /home/user/Games and Z:\\home\\user\\x",
        &report,
    );
    assert!(linux.starts_with("user-added files"));
    assert!(linux.contains("/home/<user>/Games"));
    assert!(linux.contains("Z:\\home\\<user>\\x"));
}

#[test]
fn summary_fits_a_discord_message() {
    let rig = Rig::native();
    let game = FakeGame::create();
    for index in 0..60 {
        write(
            &game
                .path()
                .join("data")
                .join(format!("loose_file_number_{index}.dat")),
            [0u8],
        );
    }
    let report = rig.collect(Some(game.path()));
    let summary = format_summary(&report);
    assert!(summary.chars().count() <= DISCORD_LIMIT);
    assert!(summary.contains("Findings:"));
    assert!(summary.contains("62 game file(s) dropped loose"));
}

#[test]
fn third_party_runtimes_report_not_installed_for_a_clean_dir() {
    let rig = Rig::native();
    let game = FakeGame::create();

    let report = rig.collect(Some(game.path()));
    assert!(!report.third_party.reshade.installed);
    assert!(!report.third_party.migoto.installed);
}

#[test]
fn config_deltas_surface_a_non_default_nams_toml_setting() {
    let config = settings_report(
        json!({
            "files": { "nams.toml": { "exists": true } },
            "values": { "nams.skip_startup_logos": true, "nams.graphics_backend": "d3d11" },
        }),
        json!([]),
    );
    assert_eq!(config.deltas.len(), 1);
    let delta = &config.deltas[0];
    assert_eq!(delta.file, "nams.toml");
    assert_eq!(delta.key, "skip_startup_logos");
    assert_eq!(delta.value, "true");
    assert_eq!(delta.default_value, "false");
    assert!(config.unknown_keys.is_empty());
    assert!(config.legacy_keys.is_empty());
}

#[test]
fn nams_health_reports_missing_runtime_files_and_unavailable_textures() {
    let rig = Rig::native();
    let game = FakeGame::create();

    let report = rig.collect(Some(game.path()));
    assert!(report.nams_health.missing_files.contains(&NAMS_EXE_NAME));
    assert!(report.nams_health.missing_files.contains(&YORHA_DLL_NAME));
    assert!(!report.nams_health.nams_exe_present);
    assert!(!report.texture_packs_available);
    assert!(report.recent_log_issues.is_empty());
}

#[test]
fn empty_game_dir_returns_an_all_empty_report_without_failing() {
    let rig = Rig::native();
    let report = rig.collect(None);
    assert!(report.vanilla_dropped.is_empty());
    assert!(report.mods.is_empty());
    assert!(!report.third_party.reshade.installed);
    assert!(report.config_deltas().is_empty());
    assert!(format_full_report(&report).contains("YP Launcher Diagnostics Report"));
    assert!(format_summary(&report).contains("[!] No game folder selected."));

    let blank = rig.collect(Some(std::path::Path::new("")));
    assert!(blank.game_dir.is_none());
    assert_eq!(blank.findings, report.findings);
}

#[test]
fn full_report_is_english_and_redacts_the_launch_command_path() {
    let rig = Rig::native();
    let game = FakeGame::create();

    let preview = format!(
        "{}{}NAMS.exe run --nier-path {}",
        game.text(),
        std::path::MAIN_SEPARATOR,
        game.text()
    );
    let report = rig.collect_with_preview(Some(game.path()), Some(preview));
    let full = format_full_report(&report);

    assert!(full.contains("YP Launcher Diagnostics Report"));
    assert!(full.contains("Game identity"));
    assert!(full.contains("<gameDir>"));
    assert!(
        full.contains("<gameDir>\\NAMS.exe run --nier-path <gameDir>")
            || full.contains("<gameDir>/NAMS.exe run --nier-path <gameDir>")
    );
    assert!(!full.contains(&game.text()));
}
