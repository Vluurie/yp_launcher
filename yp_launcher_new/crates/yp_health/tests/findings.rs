mod support;

use indexmap::IndexMap;
use jiff::{SignedDuration, Timestamp};
use support::bare_report;
use yp_core::Os;
use yp_health::diagnostics::build_findings;
use yp_health::diagnostics::model::{
    ConfigDelta, DiagnosticsReport, FilesystemRole, FindingLevel, GameIdentity,
};
use yp_health::probes::{FileFacts, MountFacts, TextExcerpt};
use yp_host::game_probe::{
    ExeBuildStatus, ExeVariant, STEAM_2026_TIME_DATE_STAMP, SteamAppManifest,
};
use yp_host::launch::record::LaunchRecord;
use yp_mods::{InstalledMod, ModKind};
use yp_thirdparty::{GameModEntry, GameModsInfo, RuntimeStatus};

use FindingLevel::{Error, Info, Warning};

fn stamp(text: &str) -> Timestamp {
    text.parse().unwrap()
}

fn with_game() -> DiagnosticsReport {
    let mut report = bare_report("/games/NieRAutomata");
    report.host.os = Os::Windows;
    report.game_identity = GameIdentity {
        build_status: ExeBuildStatus::Known,
        build_label: Some("Steam 2021"),
        ..GameIdentity::default()
    };
    report
}

fn found(report: &DiagnosticsReport) -> Vec<(FindingLevel, String)> {
    build_findings(report)
        .into_iter()
        .map(|finding| (finding.level, finding.message))
        .collect()
}

fn one(level: FindingLevel, message: &str) -> Vec<(FindingLevel, String)> {
    vec![(level, message.to_owned())]
}

fn delta(file: &str, key: &str, value: &str) -> ConfigDelta {
    ConfigDelta {
        file: file.to_owned(),
        key: key.to_owned(),
        value: value.to_owned(),
        default_value: String::new(),
        note: None,
    }
}

#[test]
fn healthy_install_has_no_findings() {
    assert!(found(&with_game()).is_empty());
}

#[test]
fn every_unusable_exe_gets_its_own_explanation() {
    let mut report = with_game();
    let mut identity = |change: &dyn Fn(&mut GameIdentity)| {
        let mut identity = GameIdentity::default();
        change(&mut identity);
        report.game_identity = identity;
        found(&report)
    };

    assert_eq!(
        identity(&|identity| identity.build_status = ExeBuildStatus::Missing),
        one(Error, "NieRAutomata.exe not found in the game folder.")
    );
    assert_eq!(
        identity(&|identity| identity.build_status = ExeBuildStatus::LegacyWindows7),
        one(
            Error,
            "NieRAutomata.exe is the Windows 7/8 build. NAMS needs the Windows 10/11 Steam build: set Proton Experimental, delete the exe and verify game files."
        )
    );
    assert_eq!(
        identity(&|identity| {
            identity.build_status = ExeBuildStatus::Modified;
            identity.stamp_build_label = Some("Steam 2026-11");
        }),
        one(
            Error,
            "NieRAutomata.exe carries the build stamp of Steam 2026-11 but a different SHA-1. Either a NieR:Automata update changed the exe and this NAMS does not support it yet (update the launcher), or it is a modified copy (verify game files in Steam)."
        )
    );
    assert_eq!(
        identity(&|identity| {
            identity.build_status = ExeBuildStatus::Modified;
            identity.stamp_build_label = Some("Steam 2021");
            identity.exe_variant = ExeVariant::WolfLimitBreak;
        }),
        one(
            Error,
            "NieRAutomata.exe carries the build stamp of Steam 2021 but a different SHA-1 (Wolf Limit Break). Either a NieR:Automata update changed the exe and this NAMS does not support it yet (update the launcher), or it is a modified copy (verify game files in Steam)."
        )
    );
    assert_eq!(
        identity(&|identity| identity.build_status = ExeBuildStatus::Newer),
        one(
            Error,
            "NieRAutomata.exe is a newer game update than this NAMS knows. Update the launcher."
        )
    );
    assert_eq!(
        identity(&|identity| {
            identity.build_status = ExeBuildStatus::Unknown;
            identity.exe_size = Some(123);
        }),
        one(
            Error,
            "NieRAutomata.exe is not a build NAMS knows (size 123, sha1 ?)."
        )
    );
}

#[test]
fn build_display_names_every_status() {
    let display = |status: ExeBuildStatus, variant: ExeVariant| {
        GameIdentity {
            build_status: status,
            exe_variant: variant,
            build_label: Some("Steam 2021"),
            stamp_build_label: Some("Steam 2026-11"),
            ..GameIdentity::default()
        }
        .build_display()
    };
    assert_eq!(
        display(ExeBuildStatus::Known, ExeVariant::Original),
        "Steam 2021"
    );
    assert_eq!(
        display(ExeBuildStatus::Missing, ExeVariant::Missing),
        "missing"
    );
    assert_eq!(
        display(ExeBuildStatus::LegacyWindows7, ExeVariant::LegacyWindows7),
        "Windows 7/8 legacy build"
    );
    assert_eq!(
        display(ExeBuildStatus::Modified, ExeVariant::Original),
        "changed or modified copy of Steam 2026-11"
    );
    assert_eq!(
        display(ExeBuildStatus::Modified, ExeVariant::WolfLimitBreak),
        "changed or modified copy of Steam 2026-11 (Wolf Limit Break)"
    );
    assert_eq!(
        display(ExeBuildStatus::Newer, ExeVariant::Original),
        "newer than every build NAMS knows"
    );
    assert_eq!(
        display(ExeBuildStatus::Unknown, ExeVariant::Unknown),
        "unknown build"
    );
    let unlabelled = GameIdentity {
        build_status: ExeBuildStatus::Known,
        ..GameIdentity::default()
    };
    assert_eq!(unlabelled.build_display(), "?");
}

#[test]
fn steam_manifest_reports_pending_updates_and_beta_branches() {
    let mut report = with_game();
    report.game_identity.steam_manifest = Some(SteamAppManifest {
        build_id: Some("25901062".to_owned()),
        target_build_id: Some("25901063".to_owned()),
        beta_key: Some("legacy".to_owned()),
        ..SteamAppManifest::default()
    });
    assert_eq!(
        found(&report),
        vec![
            (
                Warning,
                "Steam has a game update pending (build 25901062 -> 25901063). Let Steam finish it.".to_owned()
            ),
            (
                Warning,
                "Steam reports build 25901062 (Steam 2026-11) but NieRAutomata.exe is Steam 2021: the exe was replaced by hand."
                    .to_owned()
            ),
            (Info, "Steam beta branch selected: legacy.".to_owned()),
        ]
    );

    report.game_identity.build_label = Some("Steam 2026-11");
    report.game_identity.steam_manifest = Some(SteamAppManifest {
        build_id: Some("25901062".to_owned()),
        target_build_id: Some("0".to_owned()),
        ..SteamAppManifest::default()
    });
    assert!(found(&report).is_empty());

    report.game_identity.steam_manifest = Some(SteamAppManifest {
        build_id: Some("11111111".to_owned()),
        ..SteamAppManifest::default()
    });
    assert!(found(&report).is_empty());
}

#[test]
fn game_bin_from_another_exe_is_only_mentioned_when_the_exe_stamp_is_known() {
    let mut report = with_game();
    report.game_identity.game_bin = Some(FileFacts {
        name: "game.bin".to_owned(),
        size: 1,
        modified: None,
        short_sha1: None,
    });
    report.game_identity.game_bin_time_date_stamp = Some(0x1234);
    assert!(found(&report).is_empty());

    report.game_identity.exe_time_date_stamp = Some(STEAM_2026_TIME_DATE_STAMP);
    assert_eq!(
        found(&report),
        one(
            Info,
            "runtime/game.bin was built from another exe; NAMS rebuilds it from the installed exe on the next launch."
        )
    );

    report.game_identity.game_bin_time_date_stamp = Some(STEAM_2026_TIME_DATE_STAMP);
    assert!(found(&report).is_empty());
}

#[test]
fn launcher_runtime_problems_are_spelled_out() {
    let mut report = with_game();
    report.nams_health.missing_files = vec!["yorha_protocol.dll"];
    report.nams_health.steam_running = Some(false);
    assert_eq!(
        found(&report),
        vec![
            (
                Error,
                "Launcher files missing: yorha_protocol.dll (antivirus quarantine?). Restore them or reinstall the launcher."
                    .to_owned()
            ),
            (
                Warning,
                "Steam is not running. NAMS needs the Steam client to start the game.".to_owned()
            ),
        ]
    );
    report.nams_health.missing_files.clear();
    report.nams_health.steam_running = None;
    assert!(found(&report).is_empty());
}

#[test]
fn switched_off_plugin_loading_and_texture_injection_are_called_out() {
    let mut report = with_game();
    report.config.deltas = vec![
        delta("nams.toml", "disable_plugin_loading", "true"),
        delta("nams.toml", "disable_texture_injection", "true"),
    ];
    assert_eq!(
        found(&report),
        one(
            Warning,
            "Plugin loading is disabled (disable_plugin_loading): YoRHa Protocol and all plugins are off."
        )
    );

    report.nams_textures = vec!["HD".to_owned()];
    assert_eq!(
        found(&report)[1],
        (
            Info,
            "Texture packs are installed but texture injection is disabled.".to_owned()
        )
    );

    report.config.deltas = vec![
        delta("naiom.toml", "disable_plugin_loading", "true"),
        delta("nams.toml", "disable_texture_injection", "\"true\""),
    ];
    assert!(found(&report).is_empty());
}

#[test]
fn few_unknown_settings_are_listed_without_a_remainder() {
    let mut report = with_game();
    report.config.unknown_keys = vec![
        delta("nams.toml", "a", "1"),
        delta("lodmod.toml", "b.c", "2"),
    ];
    assert_eq!(
        found(&report),
        one(
            Info,
            "Unknown setting(s) NAMS does not read (typo?): nams.toml: a, lodmod.toml: b.c."
        )
    );
    report.config.unknown_keys.clear();
    report.config.legacy_keys = vec![delta("nams.toml", "mouse.sensitivity", "3.0")];
    assert!(found(&report).is_empty());
}

#[test]
fn risky_filesystems_are_errors_for_the_game_and_warnings_elsewhere() {
    let mount = |fs_type: &str| {
        Some(MountFacts {
            target: "/mnt".to_owned(),
            fs_type: fs_type.to_owned(),
            options: String::new(),
        })
    };
    let mut report = with_game();
    report.host.os = Os::Linux;
    report.filesystems = IndexMap::from([
        (FilesystemRole::GameFolder, mount("btrfs")),
        (FilesystemRole::LauncherFolder, mount("vfat")),
        (FilesystemRole::WinePrefix, mount("exfat")),
    ]);
    assert_eq!(
        found(&report),
        vec![
            (Error, "No Proton, Wine or CrossOver found to run NAMS.exe.".to_owned()),
            (
                Warning,
                "The launcher folder is on vfat. Under Proton, Play and Verify hang on NTFS/FAT drives: move it to a Linux filesystem (ext4, btrfs)."
                    .to_owned()
            ),
            (
                Warning,
                "The wine prefix is on exfat. Under Proton, Play and Verify hang on NTFS/FAT drives: move it to a Linux filesystem (ext4, btrfs)."
                    .to_owned()
            ),
        ]
    );
}

fn launch(started: &str, ended: Option<&str>, exit_code: Option<i32>) -> LaunchRecord {
    LaunchRecord {
        started_at: stamp(started),
        ended_at: ended.map(stamp),
        exit_code,
        label: "NAMS".to_owned(),
        output_tail: String::new(),
    }
}

#[test]
fn quick_failed_exits_warn_and_late_ones_inform() {
    let mut report = with_game();
    report.last_launch = Some(launch(
        "2026-11-05T10:00:00Z",
        Some("2026-11-05T10:00:59Z"),
        Some(-1073741819),
    ));
    assert_eq!(
        found(&report),
        one(
            Warning,
            "Last launch ended with exit code -1073741819 after 59s."
        )
    );

    report.last_launch = Some(launch(
        "2026-11-05T10:00:00Z",
        Some("2026-11-05T10:01:15Z"),
        Some(1),
    ));
    assert_eq!(
        found(&report),
        one(Info, "Last launch ended with exit code 1 after 75s.")
    );

    report.last_launch = Some(launch("2026-11-05T10:00:00Z", None, Some(1)));
    assert_eq!(
        found(&report),
        one(Info, "Last launch ended with exit code 1.")
    );

    report.last_launch = Some(launch(
        "2026-11-05T10:00:00Z",
        Some("2026-11-05T10:00:05Z"),
        Some(0),
    ));
    assert!(found(&report).is_empty());
    report.last_launch = Some(launch("2026-11-05T10:00:00Z", None, None));
    assert!(found(&report).is_empty());
}

fn log_file(name: &str, modified: Timestamp) -> TextExcerpt {
    TextExcerpt {
        name: name.to_owned(),
        size: 1,
        modified: Some(modified),
        lines: Vec::new(),
    }
}

#[test]
fn crash_dumps_matter_for_the_last_session_or_the_last_week() {
    let mut report = with_game();
    let generated = report.generated_at;

    report.logs.newest_crash_dump = Some(log_file(
        "crashdump_a.txt",
        generated - SignedDuration::from_hours(3 * 24),
    ));
    assert_eq!(
        found(&report),
        one(Info, "NAMS wrote a crash dump: crashdump_a.txt.")
    );

    report.logs.newest_crash_dump = Some(log_file(
        "crashdump_a.txt",
        generated - SignedDuration::from_hours(8 * 24),
    ));
    assert!(found(&report).is_empty());

    let old_dump = generated - SignedDuration::from_hours(30 * 24);
    report.last_launch = Some(LaunchRecord {
        started_at: old_dump + SignedDuration::from_secs(30),
        ended_at: None,
        exit_code: None,
        label: "NAMS".to_owned(),
        output_tail: String::new(),
    });
    report.logs.newest_crash_dump = Some(log_file("crashdump_b.txt", old_dump));
    assert_eq!(
        found(&report),
        one(
            Warning,
            "NAMS wrote a crash dump in the last session: crashdump_b.txt."
        )
    );

    report.logs.newest_crash_dump = Some(log_file(
        "crashdump_b.txt",
        old_dump - SignedDuration::from_mins(5),
    ));
    assert!(found(&report).is_empty());

    report.logs.newest_crash_dump = Some(TextExcerpt {
        modified: None,
        ..log_file("crashdump_c.txt", generated)
    });
    assert!(found(&report).is_empty());
}

#[test]
fn heartbeat_log_counts_only_when_written_after_the_last_start() {
    let mut report = with_game();
    let started = stamp("2026-11-05T10:00:00Z");
    report.logs.heartbeat = Some(log_file(
        "heartbeat.log",
        started + SignedDuration::from_secs(1),
    ));
    assert!(found(&report).is_empty());

    report.last_launch = Some(launch("2026-11-05T10:00:00Z", None, None));
    assert_eq!(
        found(&report),
        one(
            Warning,
            "The heartbeat watchdog reported a freeze in the last session (heartbeat.log)."
        )
    );

    report.logs.heartbeat = Some(log_file("heartbeat.log", started));
    assert!(found(&report).is_empty());
}

#[test]
fn several_proxy_dlls_are_listed_together() {
    let mut report = with_game();
    report.game_root_extras = [
        "d3d11.dll",
        "DXGI.dll",
        "dinput8.dll",
        "notes.txt",
        "SK_Res",
    ]
    .map(str::to_owned)
    .to_vec();
    assert_eq!(
        found(&report),
        one(
            Warning,
            "d3d11.dll, DXGI.dll, dinput8.dll in the game folder are never loaded: NAMS runs the game from its own folder. Install DLL mods through Third Party, which puts them in nams/thirdparty/graphic or nams/thirdparty/game."
        )
    );

    report.game_root_extras = vec!["SK_Res/".to_owned()];
    assert!(found(&report).is_empty());
}

fn game_dll(name: &str, disabled: bool) -> GameModEntry {
    GameModEntry {
        file_name: name.to_owned(),
        disabled,
        ..GameModEntry::default()
    }
}

#[test]
fn game_dlls_without_settings_count_as_2021_and_long_lists_are_capped() {
    let mut report = with_game();
    report.game_identity.build_label = Some("Steam 2026-11");
    report.third_party.game_mods = RuntimeStatus {
        installed: true,
        enabled: true,
        game_mods_info: Some(GameModsInfo {
            mods: vec![
                game_dll("a.dll", false),
                game_dll("b.dll", false),
                game_dll("off.dll", true),
                game_dll("c.dll", false),
                game_dll("d.dll", false),
                game_dll("e.dll", false),
            ],
            has_config: false,
        }),
        ..RuntimeStatus::default()
    };
    assert_eq!(
        found(&report),
        one(
            Warning,
            "Game DLL mods (a.dll, b.dll, c.dll, d.dll, ... 1 more) are told the game is the 2021 build, but Steam 2026-11 is installed. A mod made for 2021 reads the wrong memory there; check that it supports the update or turn it off in Third Party."
        )
    );

    report.third_party.game_mods.enabled = false;
    assert!(found(&report).is_empty());
    report.third_party.game_mods.enabled = true;
    report.game_identity.build_label = Some("Steam 2021");
    assert!(found(&report).is_empty());
}

#[test]
fn missing_requirements_are_listed_for_the_first_five_mods() {
    let mut report = with_game();
    report.mods = (1..=7)
        .map(|index| {
            let mut installed = InstalledMod::new(format!("mod{index}"), "x", ModKind::Native);
            installed.requires_missing = vec!["base".to_owned(), format!("extra{index}")];
            installed
        })
        .collect();
    report
        .mods
        .push(InstalledMod::new("complete", "x", ModKind::Data));

    let findings = found(&report);
    assert_eq!(findings.len(), 5);
    assert_eq!(
        findings[0],
        (
            Warning,
            "Mod mod1 needs missing mod(s): base, extra1.".to_owned()
        )
    );
    assert_eq!(
        findings[4],
        (
            Warning,
            "Mod mod5 needs missing mod(s): base, extra5.".to_owned()
        )
    );
}

#[test]
fn collection_errors_are_listed_in_the_order_they_happened() {
    let mut report = with_game();
    report.collection_errors = IndexMap::from([
        ("mods".to_owned(), "list failed".to_owned()),
        ("lodmod.toml".to_owned(), "read failed".to_owned()),
    ]);
    assert_eq!(
        found(&report),
        vec![
            (Warning, "Could not collect mods: list failed".to_owned()),
            (
                Warning,
                "Could not collect lodmod.toml: read failed".to_owned()
            ),
        ]
    );
}
