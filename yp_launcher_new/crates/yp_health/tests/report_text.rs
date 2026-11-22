mod support;

use std::collections::BTreeSet;

use indexmap::IndexMap;
use jiff::Timestamp;
use support::{Rig, bare_report};
use yp_core::Os;
use yp_cutscenes::{CutsceneDetection, CutsceneFile, CutsceneMod};
use yp_health::diagnostics::model::{
    CompatLayerReport, ConfigDelta, ConfigReport, DISCORD_LIMIT, DiagnosticsReport, DlcSource,
    FilesystemRole, FindingLevel, GameIdentity, LogsReport, NamsHealth, ReportHost, VanillaBucket,
    VanillaDirFinding,
};
use yp_health::diagnostics::{
    build_findings, format_full_report, format_summary, report_file_name, write_full_report,
};
use yp_health::probes::{FileFacts, MountFacts, TextExcerpt};
use yp_host::game_probe::{
    ExeBuildStatus, ExeVariant, STEAM_2021_TIME_DATE_STAMP, STEAM_2026_SHA1, STEAM_2026_SIZE,
    STEAM_2026_TIME_DATE_STAMP, SteamAppManifest,
};
use yp_host::launch::record::LaunchRecord;
use yp_host::nams_cli::NamsTexturePack;
use yp_host::nams_log::{LogEntry, LogLevel};
use yp_host::wine::runtime::WineSource;
use yp_mods::model::{DataDirEntry, DataSummary, ManifestInfo, NativeSummary, PlayerModelEntry};
use yp_mods::{DataCategory, InstalledMod, ModConflict, ModConflictKind, ModKind};
use yp_thirdparty::{
    GameModEntry, GameModSettings, GameModsInfo, MigotoConfig, MigotoHunting, MigotoInfo,
    MigotoMarking, ReShadeConfig, ReShadeInfo, RuntimeStatus, ThirdPartyStatuses,
};

const RULE: &str = "========================================================================";

fn stamp(text: &str) -> Timestamp {
    text.parse().unwrap()
}

fn text_map(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn section<'a>(full: &'a str, title: &str) -> &'a str {
    let header = format!("\n{RULE}\n{title}\n{RULE}\n");
    let start = full
        .find(&header)
        .unwrap_or_else(|| panic!("section {title:?} is missing in:\n{full}"))
        + header.len();
    let rest = &full[start..];
    let end = rest.find(&format!("\n{RULE}\n")).unwrap_or(rest.len());
    &rest[..end]
}

fn delta(
    file: &str,
    key: &str,
    value: &str,
    default_value: &str,
    note: Option<&str>,
) -> ConfigDelta {
    ConfigDelta {
        file: file.to_owned(),
        key: key.to_owned(),
        value: value.to_owned(),
        default_value: default_value.to_owned(),
        note: note.map(str::to_owned),
    }
}

fn facts(name: &str, size: u64, modified: &str, short_sha1: Option<&str>) -> FileFacts {
    FileFacts {
        name: name.to_owned(),
        size,
        modified: Some(stamp(modified)),
        short_sha1: short_sha1.map(str::to_owned),
    }
}

fn excerpt(name: &str, size: u64, modified: &str, lines: &[&str]) -> TextExcerpt {
    TextExcerpt {
        name: name.to_owned(),
        size,
        modified: Some(stamp(modified)),
        lines: strings(lines),
    }
}

fn issue(timestamp: &str, level: LogLevel, module: &str, message: &str) -> LogEntry {
    LogEntry {
        timestamp: timestamp.to_owned(),
        level,
        module: module.to_owned(),
        message: message.to_owned(),
    }
}

fn alpha_mod() -> InstalledMod {
    let mut alpha = InstalledMod::new(
        "alpha",
        "/games/NieRAutomata/nams/mods/alpha",
        ModKind::Native,
    );
    alpha.display_name = "Alpha Mod".to_owned();
    alpha.manifest = Some(ManifestInfo {
        id: Some("alpha".to_owned()),
        display_name: Some("Alpha Mod".to_owned()),
        author: Some("Vluurie".to_owned()),
        version: Some("1.2.0".to_owned()),
        nams_min_version: Some("0.9".to_owned()),
        requires: strings(&["beta"]),
        requires_plugins: strings(&["yorha_protocol"]),
        replaces: strings(&["old_alpha"]),
    });
    alpha.native = Some(NativeSummary {
        bundles_by_kind: IndexMap::from([("outfit".to_owned(), 2), ("weapon".to_owned(), 1)]),
        total_entity_files: 3,
    });
    alpha.data = Some(DataSummary {
        entries: vec![
            DataDirEntry {
                dir_name: "wp".to_owned(),
                category: DataCategory::Weapon,
                file_count: 1,
            },
            DataDirEntry {
                dir_name: "pl".to_owned(),
                category: DataCategory::Player,
                file_count: 2,
            },
        ],
        players: vec![
            PlayerModelEntry {
                file_name: "pl000d.dtt".to_owned(),
                label: "fallback".to_owned(),
            },
            PlayerModelEntry {
                file_name: "plzzzz.dat".to_owned(),
                label: "Custom label".to_owned(),
            },
        ],
        ..DataSummary::default()
    });
    alpha.requires_missing = strings(&["beta"]);
    alpha.conflicts = vec![ModConflict {
        other_mod_id: "delta".to_owned(),
        kind: ModConflictKind::OutfitIdCollision,
        detail: "outfit id 5".to_owned(),
    }];
    alpha.installed_at = stamp("2026-10-01T12:00:00Z");
    alpha.bundled_texture_packs = strings(&["x"]);
    alpha
}

fn texture_pack(name: &str, source: &str, path: &str) -> NamsTexturePack {
    NamsTexturePack {
        name: name.to_owned(),
        source: source.to_owned(),
        mod_id: None,
        character: None,
        outfit_conditional: false,
        enabled: true,
        dds_count: 0,
        load_order_index: None,
        path: path.to_owned(),
    }
}

fn third_party() -> ThirdPartyStatuses {
    ThirdPartyStatuses {
        reshade: RuntimeStatus {
            installed: true,
            enabled: false,
            shaders_missing: true,
            reshade_info: Some(ReShadeInfo {
                presets: strings(&["A.ini", "B.ini"]),
                shader_repos: strings(&["qUINT"]),
                shader_count: 42,
                addons: strings(&["x.addon64"]),
                is_addon_build: true,
                version: Some("6.3.0".to_owned()),
                dll_name: Some("dxgi.dll".to_owned()),
                d3d_compiler_missing: true,
                config: ReShadeConfig {
                    performance_mode: true,
                    active_preset: Some("A.ini".to_owned()),
                    ..ReShadeConfig::default()
                },
            }),
            ..RuntimeStatus::default()
        },
        migoto: RuntimeStatus {
            installed: true,
            enabled: true,
            has_shader_fixes: true,
            migoto_info: Some(MigotoInfo {
                files: strings(&["d3d11.dll", "d3dx.ini"]),
                shader_fix_count: 7,
                shader_fix_names: strings(&["HUD toggle"]),
                loader_target: Some("NieRAutomata.exe".to_owned()),
                loader_target_ok: false,
                config: MigotoConfig {
                    hunting: MigotoHunting::NoMarking,
                    marking_mode: MigotoMarking::Pink,
                    verbose_overlay: true,
                    cache_shaders: false,
                    ..MigotoConfig::default()
                },
            }),
            ..RuntimeStatus::default()
        },
        game_mods: RuntimeStatus {
            installed: true,
            enabled: true,
            game_mods_info: Some(GameModsInfo {
                mods: vec![
                    GameModEntry {
                        file_name: "celestial.dll".to_owned(),
                        mod_dir: "Celestial".to_owned(),
                        size_bytes: Some(1536),
                        disabled: false,
                        settings: Some(GameModSettings::default()),
                    },
                    GameModEntry {
                        file_name: "old.dll".to_owned(),
                        mod_dir: String::new(),
                        size_bytes: None,
                        disabled: true,
                        settings: Some(GameModSettings {
                            fake_module_handle: false,
                            fake_exe_path: false,
                            size_of_image: GameModSettings::STEAM_2026_SIZE_OF_IMAGE,
                            disabled: true,
                            report_patches: true,
                            wait_for_patch_phase: true,
                            patch_phase_timeout_ms: 5000,
                        }),
                    },
                ],
                has_config: true,
            }),
            ..RuntimeStatus::default()
        },
        graphic_mods: RuntimeStatus::default(),
    }
}

fn rich_report() -> DiagnosticsReport {
    let mut report = bare_report("/games/NieRAutomata");
    report.host = ReportHost {
        os: Os::Linux,
        user_home: "/home/kiro".to_owned(),
        user_name: "kiro".to_owned(),
    };
    report.system_info = text_map(&[
        ("OS", "linux"),
        ("OS version", "Linux 6.11"),
        ("Locale", "de"),
        ("Game running", "no"),
        ("Steam running", "yes"),
        ("Prefer dedicated GPU", "yes"),
        ("Distro", "Arch Linux"),
        ("Session", "wayland"),
        ("Desktop", "KDE"),
        ("AppImage", "yes"),
        ("Flatpak", "no"),
        ("Steam Deck", "no"),
        ("NVIDIA driver", "no"),
        ("DRM render nodes", "1"),
        ("Dedicated GPU env", "none"),
    ]);
    report.launcher_info = text_map(&[
        ("Launcher version", "v3.1.0"),
        ("Launcher dir", "/home/kiro/.local/share/yp_launcher/bins"),
        ("Game dir", "/games/NieRAutomata"),
    ]);
    report.filesystems = IndexMap::from([
        (
            FilesystemRole::GameFolder,
            Some(MountFacts {
                target: "/games".to_owned(),
                fs_type: "fuseblk".to_owned(),
                options: "rw,relatime".to_owned(),
            }),
        ),
        (
            FilesystemRole::LauncherFolder,
            Some(MountFacts {
                target: "/".to_owned(),
                fs_type: "ext4".to_owned(),
                options: String::new(),
            }),
        ),
        (FilesystemRole::WinePrefix, None),
    ]);
    report.game_identity = GameIdentity {
        exe_variant: ExeVariant::Original,
        has_dlc: true,
        dlc_source: DlcSource::SteamManifest,
        exe_size: Some(STEAM_2026_SIZE),
        exe_sha1: Some(STEAM_2026_SHA1.to_owned()),
        exe_time_date_stamp: Some(STEAM_2026_TIME_DATE_STAMP),
        build_status: ExeBuildStatus::Known,
        build_label: Some("Steam 2026-11"),
        stamp_build_label: Some("Steam 2026-11"),
        steam_manifest: Some(SteamAppManifest {
            build_id: Some("26534214".to_owned()),
            target_build_id: Some("26534214".to_owned()),
            state_flags: Some("4".to_owned()),
            last_updated: Some("1793892324".to_owned()),
            beta_key: None,
            installed_dlc: BTreeSet::from(["580600".to_owned()]),
            disabled_dlc: BTreeSet::new(),
        }),
        build_info: text_map(&[("Last_SVN_REVISION", "7788")]),
        compat_exe_present: true,
        data100_size: Some(975_000_000),
        game_bin: Some(facts("game.bin", 26_000_000, "2026-11-01T08:00:00Z", None)),
        game_bin_time_date_stamp: Some(STEAM_2021_TIME_DATE_STAMP),
    };
    report.nams_health = NamsHealth {
        missing_files: Vec::new(),
        nams_exe_present: true,
        launch_command_preview: Some(
            "cd /home/kiro/.local/share/yp_launcher/bins\nproton run NAMS.exe run --nier-path /games/NieRAutomata"
                .to_owned(),
        ),
        nams_exe: Some(facts("NAMS.exe", 2048, "2026-11-01T08:00:00Z", Some("0123456789ab"))),
        plugins: vec![facts(
            "yorha_protocol.dll",
            5 * 1024 * 1024,
            "2026-11-01T08:00:00Z",
            Some("ba9876543210"),
        )],
        nams_version: None,
        nier_install_error: Some("registry key missing".to_owned()),
        steam_running: Some(true),
    };
    report.compat_layer = Some(CompatLayerReport {
        source: WineSource::Proton,
        label: "Proton".to_owned(),
        binary: "/home/kiro/.steam/steam/steamapps/common/Proton - Experimental/proton".to_owned(),
        prefix: Some("/games/steamapps/compatdata/524220/pfx".to_owned()),
        prefix_exists: false,
        tool_version: Some("1730000000 experimental-9.0".to_owned()),
        prefix_version: None,
    });
    report.last_launch = Some(LaunchRecord {
        started_at: stamp("2026-11-05T22:00:00Z"),
        ended_at: Some(stamp("2026-11-05T22:00:12.500Z")),
        exit_code: Some(3),
        label: "Proton".to_owned(),
        output_tail: "wine: starting\r\nNAMS panicked  \n".to_owned(),
    });
    report.logs = LogsReport {
        nams_log: Some(excerpt(
            "nams.log",
            1536,
            "2026-11-05T22:00:11Z",
            &["a", "b"],
        )),
        crash_dump_count: 2,
        newest_crash_dump: Some(excerpt(
            "crashdump_2026-11-05.txt",
            10,
            "2026-11-05T22:00:12Z",
            &["EXCEPTION", "", "at x"],
        )),
        heartbeat: Some(excerpt(
            "heartbeat.log",
            5,
            "2026-11-05T22:00:10Z",
            &["stalled 30s"],
        )),
        captured_launch_logs: vec![facts(
            "nams_2026-11-05T22-00-12-000000.log",
            100,
            "2026-11-05T22:00:12Z",
            None,
        )],
    };
    report.recent_log_issues = vec![
        issue("10:00:00.000", LogLevel::Warn, "modloader", "slow load"),
        issue("10:00:01.000", LogLevel::Error, "yorha", "bad pack"),
        issue("10:00:02.000", LogLevel::Warn, "modloader", "slow load"),
    ];
    report.vanilla_dropped = vec![
        VanillaDirFinding {
            path: "data/enemy.dtt".to_owned(),
            size_bytes: 8,
            bucket: VanillaBucket::DataRootLoose,
        },
        VanillaDirFinding {
            path: "data/notes.txt".to_owned(),
            size_bytes: 2048,
            bucket: VanillaBucket::DataRootLoose,
        },
    ];
    report.data_dir_contents = IndexMap::from([("mods".to_owned(), 2), ("backup".to_owned(), 10)]);
    report.third_party = third_party();
    report.texture_packs = vec![
        NamsTexturePack {
            dds_count: 120,
            load_order_index: Some(0),
            ..texture_pack(
                "HD Pack",
                "standalone",
                "/games/NieRAutomata/nams/inject/textures/HD Pack",
            )
        },
        NamsTexturePack {
            mod_id: Some("alpha".to_owned()),
            character: Some("2B".to_owned()),
            outfit_conditional: true,
            dds_count: 8,
            ..texture_pack(
                "2B Kimono",
                "mod",
                "/games/NieRAutomata/nams/mods/delta/textures/2B Kimono",
            )
        },
    ];
    report.texture_packs_available = true;
    report.nams_textures = strings(&["HD Pack"]);
    report.sk_res_textures = strings(&["SK1"]);
    report.sk_res_packs = strings(&["SK1", "SK2"]);
    report.wax_textures = strings(&["w"]);
    report.config = ConfigReport {
        deltas: vec![delta(
            "nams.toml",
            "disable_splash_screen",
            "true",
            "false",
            None,
        )],
        unknown_keys: vec![
            delta("nams.toml", "typo_key", "1", "", None),
            delta("naiom.toml", "bindings.k1", "\"A\"", "", None),
            delta("naiom.toml", "bindings.k2", "\"A\"", "", None),
            delta("naiom.toml", "bindings.k3", "\"A\"", "", None),
            delta("naiom.toml", "bindings.k4", "\"A\"", "", None),
            delta("naiom.toml", "bindings.k5", "\"A\"", "", None),
            delta("naiom.toml", "other.x", "2", "", None),
        ],
        legacy_keys: vec![
            delta(
                "nams.toml",
                "mouse.sensitivity",
                "3.0",
                "",
                Some("moved to naiom.toml, NAMS ignores it once naiom.toml exists"),
            ),
            delta(
                "nams.toml",
                "cutscene.hd_cutscenes",
                "true",
                "",
                Some("removed"),
            ),
        ],
        parse_errors: IndexMap::from([(
            "lodmod.toml".to_owned(),
            "line 1, column 11: expected value".to_owned(),
        )]),
        missing_files: vec!["texture_injection.toml".to_owned()],
    };
    report.config_raw = IndexMap::from([("lodmod.toml".to_owned(), "enabled = = true\n".to_owned())]);
    report.active_profile = Some("speedrun".to_owned());
    report.default_mods_entries = strings(&["outfit_bare = [\"mods/alpha\"]"]);
    report.mods = vec![
        alpha_mod(),
        InstalledMod::new(
            "delta",
            "/games/NieRAutomata/nams/mods/delta",
            ModKind::Data,
        ),
    ];
    report.hidden_mod_folders = strings(&["_parked"]);
    report.disabled_mods_entries = strings(&["mods/delta"]);
    report.cutscene_mods = vec![
        CutsceneMod {
            name: "hd".to_owned(),
            path: "/games/NieRAutomata/nams/cutscenes/hd".into(),
            files: vec![CutsceneFile {
                name: "a.usm".to_owned(),
                video: None,
            }],
            bundled_with_mod_id: None,
        },
        CutsceneMod {
            name: "intro".to_owned(),
            path: "/games/NieRAutomata/nams/mods/alpha/cutscenes/intro".into(),
            files: Vec::new(),
            bundled_with_mod_id: Some("alpha".to_owned()),
        },
    ];
    report.direct_overrides = strings(&["op.usm"]);
    report.cutscene_detection = CutsceneDetection {
        has_hd_cutscenes: true,
        needs_h264: true,
        files_scanned: 3,
        largest_width: 3840,
        largest_height: 2160,
    };
    report.reshade_contents = strings(&["ReShade.ini", "Presets"]);
    report.game_root_extras = strings(&["SK_Res/", "dinput8.dll"]);
    report.collection_errors = text_map(&[("texture folders", "list failed")]);
    report.findings = build_findings(&report);
    report
}

const RICH_FINDINGS: &str = "\
[X] lodmod.toml does not parse: line 1, column 11: expected value
[X] The game folder is on fuseblk. Under Proton, Play and Verify hang on NTFS/FAT drives: move it to a Linux filesystem (ext4, btrfs).
[!] Steam reports build 26534214 (Steam 2026-11 (signed)) but NieRAutomata.exe is Steam 2026-11: the exe was replaced by hand.
[!] NAMS: registry key missing
[!] Last launch ended with exit code 3 after 12s.
[!] NAMS wrote a crash dump in the last session: crashdump_2026-11-05.txt.
[!] The heartbeat watchdog reported a freeze in the last session (heartbeat.log).
[!] 1 game file(s) dropped loose into vanilla data/ (enemy.dtt). NAMS mods belong in nams/mods.
[!] Game DLL mods (celestial.dll) are told the game is the 2021 build, but Steam 2026-11 is installed. A mod made for 2021 reads the wrong memory there; check that it supports the update or turn it off in Third Party.
[!] Mod alpha needs missing mod(s): beta.
[!] 3DMigoto d3dx.ini [Loader] target is not NAMS.exe.
[!] ReShade: d3dcompiler_47.dll missing, shaders will not compile under Wine.
[!] Could not collect texture folders: list failed
[i] runtime/game.bin was built from Steam 2021; NAMS rebuilds it from the installed exe on the next launch.
[i] Unknown setting(s) NAMS does not read (typo?): nams.toml: typo_key, naiom.toml: bindings.k1, naiom.toml: bindings.k2, naiom.toml: bindings.k3 and 3 more.
[i] Special K (dinput8.dll in the game folder) does not run under NAMS. NAMS reads SK_Res texture packs itself.
[i] 1 mod folder(s) start with \"_\": hidden in the launcher, NAMS still loads them.
";

#[test]
fn full_report_starts_with_the_header_and_the_sorted_findings() {
    let full = format_full_report(&rich_report());
    assert!(full.starts_with(&format!(
        "YP Launcher Diagnostics Report\nFormat: 2\nGenerated: 2026-11-06T00:00:00Z\n\n{RULE}\nFindings (17)\n{RULE}\n"
    )));
    assert_eq!(section(&full, "Findings (17)"), RICH_FINDINGS);
}

#[test]
fn full_report_lists_system_launcher_and_filesystems() {
    let full = format_full_report(&rich_report());
    assert_eq!(
        section(&full, "System"),
        "\
OS: linux
OS version: Linux 6.11
Locale: de
Game running: no
Steam running: yes
Prefer dedicated GPU: yes
Distro: Arch Linux
Session: wayland
Desktop: KDE
AppImage: yes
Flatpak: no
Steam Deck: no
NVIDIA driver: no
DRM render nodes: 1
Dedicated GPU env: none

Filesystems:
  game folder: fuseblk (mount /games, rw,relatime)
  launcher folder: ext4 (mount /)
  wine prefix: unknown
"
    );
    assert_eq!(section(&full, "Launcher"), "Launcher version: v3.1.0\n");
}

#[test]
fn full_report_describes_the_game_build_and_the_nams_runtime() {
    let full = format_full_report(&rich_report());
    assert_eq!(
        section(&full, "Game identity"),
        "\
build: Steam 2026-11 (supported)
exe variant: original
exe: 16.8 MB (17601536 bytes)  sha1 537be3ea1083114cc1e444b85ac270f2043ac4d6  PE stamp 0x6ac89519
known builds: Steam 2021 (17837952 bytes), Steam 2026-11 (17601536 bytes), Steam 2026-11 (signed) (17613568 bytes)
NieRAutomataCompat.exe: present
Latest_BuildInfo.properties: Last_SVN_REVISION=7788
Steam manifest: build 26534214  state 4  updated 2026-11-05T15:25:24Z
Steam DLC: installed [580600]  disabled []
DLC (3C3C Concert & Costume): true (Steam manifest)  data100.cpk 975000000 bytes
runtime/game.bin: game.bin  24.8 MB  modified 2026-11-01T08:00:00Z  PE stamp 0x60d3a8d2 (Steam 2021, rebuilt on next launch)
"
    );
    assert_eq!(
        section(&full, "NAMS runtime health"),
        "\
NAMS.exe: NAMS.exe  2 KB  modified 2026-11-01T08:00:00Z  sha1 0123456789ab
NAMS version: not queried (Windows only)
NAMS install lookup: registry key missing
Plugins (1):
  - yorha_protocol.dll  5.0 MB  modified 2026-11-01T08:00:00Z  sha1 ba9876543210
Game running: false
Steam running: yes
Missing runtime files: none

Launch command:
cd <launcherDir>
proton run NAMS.exe run --nier-path <gameDir>
"
    );
    assert_eq!(
        section(&full, "Compatibility layer"),
        "\
source: proton  label: Proton
binary: <userHome>/.steam/steam/steamapps/common/Proton - Experimental/proton
tool version: 1730000000 experimental-9.0
prefix: /games/steamapps/compatdata/524220/pfx (not created yet)
"
    );
}

#[test]
fn full_report_shows_the_last_session_and_its_logs() {
    let full = format_full_report(&rich_report());
    assert_eq!(
        section(&full, "Last launch"),
        "\
started: 2026-11-05T22:00:00Z  via: Proton
ended: 2026-11-05T22:00:12Z  exit code: 3  duration: 12s
output tail:
  wine: starting
  NAMS panicked
"
    );
    assert_eq!(
        section(&full, "Logs"),
        "nams.log: 2 KB  modified 2026-11-05T22:00:11Z  (last 2 lines)\n  a\n  b\n\ncrash dumps: 2\nnewest: crashdump_2026-11-05.txt  modified 2026-11-05T22:00:12Z\n  EXCEPTION\n  \n  at x\n\nheartbeat.log: modified 2026-11-05T22:00:10Z\n  stalled 30s\n\ncaptured early-exit logs:\n  - nams_2026-11-05T22-00-12-000000.log  100 B  modified 2026-11-05T22:00:12Z\n"
    );
    assert_eq!(
        section(&full, "Recent NAMS warnings/errors (3)"),
        "  10:00:01.000 ERROR yorha: bad pack\n  10:00:02.000 WARN modloader: slow load  (x2)\n"
    );
    assert_eq!(
        section(&full, "Vanilla data/ drops (user-added files, 2)"),
        "  data/enemy.dtt  8 B  [data-root-loose]\n  data/notes.txt  2 KB  [data-root-loose]\n\nNon-vanilla folders in data/:\n  backup/  (10 files)\n  mods/  (2 files)\n"
    );
}

#[test]
fn full_report_details_third_party_runtimes_and_dll_mods() {
    let full = format_full_report(&rich_report());
    assert_eq!(
        section(&full, "ReShade"),
        "\
installed: true   enabled: false   shadersMissing: true
version: 6.3.0
dll: dxgi.dll   addonBuild: true
presets (2): A.ini, B.ini
shader repos (1): qUINT   effects: 42
addons (1): x.addon64
!! d3dcompiler_47.dll missing (Wine shaders will not compile)
config: performanceMode=true showFps=false showClock=false activePreset=A.ini
"
    );
    assert_eq!(
        section(&full, "3DMigoto"),
        "\
installed: true   enabled: true   hasShaderFixes: true
files: d3d11.dll, d3dx.ini
loader target: NieRAutomata.exe (ok: false)
shader fixes: 7
  from: HUD toggle
config: hunting=noMarking marking=pink cacheShaders=false verboseOverlay=true
"
    );
    assert_eq!(
        section(&full, "Third-party game & graphic DLLs"),
        "\
nams/thirdparty/game: loading enabled
  - Celestial/celestial.dll  2 KB  module=game.bin  exePath=NieRAutomata.exe  sizeOfImage=26177536
  - old.dll    [disabled]  module=NAMS.exe  exePath=NAMS.exe  sizeOfImage=25944064  reportPatches  waitForPatchPhase=5000ms
nams/thirdparty/graphic: (none)
"
    );
}

#[test]
fn full_report_lists_texture_packs_settings_and_profiles() {
    let full = format_full_report(&rich_report());
    assert_eq!(
        section(&full, "Texture packs (2)"),
        "\
- HD Pack  [standalone]  enabled  120 dds  order=0
- 2B Kimono  [mod]  disabled (disabled_mods.toml)  8 dds  mod=alpha  char=2B  outfit-conditional

nams/inject/textures/ (1): HD Pack
SK_Res/inject/textures/ (1): SK1
SK_Res packs (all exe folders): SK1, SK2
WAX textures (1): w
"
    );
    assert_eq!(
        section(&full, "Settings"),
        "\
missing or empty: texture_injection.toml
!! lodmod.toml does not parse: line 1, column 11: expected value

Non-default (1):
  [nams.toml] disable_splash_screen = true  (default false)

Unknown keys, NAMS does not read them (7):
  [nams.toml] typo_key = 1
  [naiom.toml] [bindings] 5 keys
  [naiom.toml] other.x = 2

Old keys left in the files (2):
  [nams.toml] mouse.sensitivity = 3.0  (moved to naiom.toml, NAMS ignores it once naiom.toml exists)
  [nams.toml] cutscene.hd_cutscenes = true  (removed)
"
    );
    assert_eq!(
        section(&full, "Profiles"),
        "active profile: speedrun\ndefault_mods.toml:\n  outfit_bare = [\"mods/alpha\"]\n"
    );
}

#[test]
fn full_report_details_mods_with_warnings_and_lists_the_rest() {
    let full = format_full_report(&rich_report());
    assert_eq!(
        section(&full, "Installed mods (2)"),
        "\
Totals by kind: native=1, data=1
With warnings: 1   With bundled assets: 1   Disabled prefixes affecting mods: 1

Hidden \"_\" folders (NAMS still loads them): _parked

- alpha  [enabled]  [native]  v1.2.0  !! warnings
    name:     Alpha Mod
    installed: 2026-10-01T12:00:00Z
    author:   Vluurie
    NAMS min version: 0.9
    requires mods:    beta
    requires plugins: yorha_protocol
    replaces:         old_alpha
    !! missing requires: beta
    native:   bundles=[outfit=2, weapon=1]  entityFiles=3
    data dirs: pl, wp
    player models:
      - pl000d.dtt  ->  2B (DLC clothes)
      - plzzzz.dat  ->  Custom label
    conflicts (1):
      - vs delta [outfit ID collision] outfit id 5
- delta  [DISABLED]  [data]
"
    );
    assert_eq!(
        section(&full, "Disabled mod prefixes (1)"),
        "- mods/delta\n"
    );
    assert_eq!(
        section(&full, "Cutscene mods (2)"),
        "\
- hd  [standalone]  1 USM  MPEG-2
- intro  [bundled with alpha]  0 USM  MPEG-2

Custom files in data/movie/ (1): op.usm

cutscene detection: hasHd=true, needsH264=true, filesScanned=3, largest=3840x2160
"
    );
    assert_eq!(
        section(&full, "Other folders"),
        "wax/mods/: (none)\nreshade/: ReShade.ini, Presets\ngame root extras: SK_Res/, dinput8.dll\n"
    );
    assert_eq!(
        section(&full, "nams/lodmod.toml (does not parse)"),
        "enabled = = true\n\n"
    );
    assert!(full.ends_with("enabled = = true\n\n"));
}

#[test]
fn rich_summary_is_cut_to_the_discord_limit_after_the_headline_block() {
    let summary = format_summary(&rich_report());
    let headline = "\
YP Launcher Diagnostics (format 2)
Generated: 2026-11-06T00:00:00Z
Launcher: v3.1.0   OS: linux Linux 6.11
Distro: Arch Linux   Session: wayland   AppImage: yes   Flatpak: no
Game: Steam 2026-11   Steam build 26534214   DLC=true   running=false
NAMS: exe=true   missing=0   Steam running=yes
Compat: Proton (1730000000 experimental-9.0)
FS game folder: fuseblk
FS launcher folder: ext4
Last launch: 2026-11-05T22:00:00Z   exit=3   12s
Mods: 2   disabled prefixes: 1   texture packs: 1/2 on   cutscene mods: 2   non-default settings: 1
Third-party: ReShade (off), 3DMigoto, game DLLs 1

Findings:
[X] lodmod.toml does not parse: line 1, column 11: expected value
";
    assert!(summary.starts_with(headline), "{summary}");
    assert_eq!(summary.chars().count(), DISCORD_LIMIT);
    assert!(summary.ends_with("\n... (see the full report)"));
}

#[test]
fn short_summary_is_printed_whole() {
    let mut report = bare_report("");
    report.system_info = text_map(&[
        ("OS", "windows"),
        ("OS version", "Windows 10 (Build 19045)"),
        ("Steam running", "unknown"),
    ]);
    report.launcher_info = text_map(&[
        ("Launcher version", "v3.1.0"),
        ("Game dir", "<not selected>"),
    ]);
    report.nams_health.missing_files = vec!["NAMS.exe"];
    report.nams_textures = strings(&["a", "b", "c"]);
    report.last_launch = Some(LaunchRecord {
        started_at: stamp("2026-11-05T22:00:00Z"),
        ended_at: None,
        exit_code: None,
        label: "Windows".to_owned(),
        output_tail: String::new(),
    });
    report.findings = build_findings(&report);

    assert_eq!(
        format_summary(&report),
        "\
YP Launcher Diagnostics (format 2)
Generated: 2026-11-06T00:00:00Z
Launcher: v3.1.0   OS: windows Windows 10 (Build 19045)
Game: missing   DLC=false   running=false
NAMS: exe=false   missing=1   Steam running=unknown
Last launch: 2026-11-05T22:00:00Z   exit=running/unknown
Mods: 0   disabled prefixes: 0   texture packs: 3   cutscene mods: 0   non-default settings: 0

Findings:
[X] Launcher files missing: NAMS.exe (antivirus quarantine?). Restore them or reinstall the launcher.
[!] No game folder selected.
"
    );

    let mut quiet = bare_report("");
    quiet.game_dir = None;
    let text = format_summary(&quiet);
    assert!(text.starts_with("YP Launcher Diagnostics (format 2)\nGenerated: 2026-11-06T00:00:00Z\nLauncher: ?   OS: ? ?\n"));
    assert!(text.ends_with("\n\nFindings: none\n"));
}

#[test]
fn empty_sections_say_so_instead_of_staying_blank() {
    let mut report = bare_report("");
    report.host.os = Os::Windows;
    let full = format_full_report(&report);
    assert_eq!(section(&full, "Findings (0)"), "(none)\n");
    assert_eq!(section(&full, "System"), "");
    assert!(!full.contains("Compatibility layer"));
    assert_eq!(
        section(&full, "Game identity"),
        "build: missing (UNSUPPORTED)\nexe variant: unknown\nknown builds: Steam 2021 (17837952 bytes), Steam 2026-11 (17601536 bytes), Steam 2026-11 (signed) (17613568 bytes)\nDLC (3C3C Concert & Costume): false (size check)\nruntime/game.bin: not built yet\n"
    );
    assert_eq!(
        section(&full, "NAMS runtime health"),
        "NAMS.exe: missing\nNAMS version: unavailable\nPlugins (0):\nGame running: false\nSteam running: ?\nMissing runtime files: none\n"
    );
    assert_eq!(section(&full, "Last launch"), "(no launch recorded)\n");
    assert_eq!(
        section(&full, "Logs"),
        "nams.log: missing\n\ncrash dumps: 0\n"
    );
    assert_eq!(
        section(&full, "Recent NAMS warnings/errors (0)"),
        "(none in nams.log)\n"
    );
    assert_eq!(
        section(&full, "Vanilla data/ drops (user-added files, 0)"),
        "(none, vanilla data/ is clean)\n"
    );
    assert_eq!(section(&full, "ReShade"), "(not installed)\n");
    assert_eq!(section(&full, "3DMigoto"), "(not installed)\n");
    assert_eq!(
        section(&full, "Third-party game & graphic DLLs"),
        "nams/thirdparty/game: (none)\nnams/thirdparty/graphic: (none)\n"
    );
    assert_eq!(
        section(&full, "Texture packs (0)"),
        "NAMS query: unavailable\n\nnams/inject/textures/ (0): \n"
    );
    assert_eq!(
        section(&full, "Settings"),
        "\nNon-default (0):\n  (all defaults)\n"
    );
    assert_eq!(section(&full, "Profiles"), "active profile: default\n");
    assert_eq!(section(&full, "Installed mods (0)"), "");
    assert_eq!(section(&full, "Disabled mod prefixes (0)"), "");
    assert_eq!(
        section(&full, "Cutscene mods (0)"),
        "\ncutscene detection: hasHd=false, needsH264=false, filesScanned=0, largest=0x0\n"
    );
    assert_eq!(
        section(&full, "Other folders"),
        "wax/mods/: (none)\nreshade/: (none)\ngame root extras: (none, clean install root)\n"
    );

    report.host.os = Os::MacOs;
    assert_eq!(
        section(&format_full_report(&report), "Compatibility layer"),
        "(no game folder)\n"
    );
    report.game_dir = Some("/Games/NieRAutomata".into());
    let with_game = format_full_report(&report);
    assert_eq!(section(&with_game, "Compatibility layer"), "(none found)\n");
    assert!(with_game.contains("Steam manifest: not found (game outside steamapps/common?)\n"));
    report.texture_packs_available = true;
    assert!(format_full_report(&report).contains("\nTexture packs (0)\n"));
    assert_eq!(
        section(&format_full_report(&report), "Texture packs (0)"),
        "(none)\n\nnams/inject/textures/ (0): \n"
    );
}

#[test]
fn findings_are_ordered_errors_then_warnings_then_infos() {
    let report = rich_report();
    let levels: Vec<FindingLevel> = report
        .findings
        .iter()
        .map(|finding| finding.level)
        .collect();
    let mut sorted = levels.clone();
    sorted.sort();
    assert_eq!(levels, sorted);
    assert_eq!(
        levels
            .iter()
            .filter(|level| **level == FindingLevel::Error)
            .count(),
        2
    );
    assert_eq!(
        levels
            .iter()
            .filter(|level| **level == FindingLevel::Warning)
            .count(),
        11
    );
    assert_eq!(
        levels
            .iter()
            .filter(|level| **level == FindingLevel::Info)
            .count(),
        4
    );
}

#[test]
fn full_report_is_written_next_to_the_logs() {
    let rig = Rig::native();
    let report = rich_report();
    assert_eq!(
        report_file_name(&report),
        "diagnostics_2026-11-06T00-00-00-000000Z.txt"
    );

    let file = write_full_report(&report, &rig.paths).unwrap();
    assert_eq!(
        file,
        rig.paths
            .logs_dir
            .join("diagnostics_2026-11-06T00-00-00-000000Z.txt")
    );
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format_full_report(&report)
    );
}
