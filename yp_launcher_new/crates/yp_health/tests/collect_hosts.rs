mod support;

use std::sync::Arc;

use jiff::{SignedDuration, Timestamp};
use serde_json::json;
use support::{FakeGame, FakePlatform, Rig, messages, pe_header, write};
use yp_core::testing::{FakeResponse, FakeRunner};
use yp_core::{CancelToken, GamePaths, Os, ProcessOutput, ProcessRunner, ProcessSpec};
use yp_health::cache::clear_all;
use yp_health::diagnostics::collect::{
    NOTE_NAMS_MISSING, NOTE_NO_GAME_FOLDER, NOTE_QUERY_FAILED, NOTE_WINDOWS_ONLY, short_error,
};
use yp_health::diagnostics::model::{FilesystemRole, FindingLevel, PackSwitch};
use yp_health::diagnostics::{Collector, format_full_report, format_summary, pack_disabled_by};
use yp_health::notifications::{NotificationKind, platform_support_notice};
use yp_health::probes::SystemFiles;
use yp_host::HostPlatform;
use yp_host::game_probe::{STEAM_2021_TIME_DATE_STAMP, STEAM_2026_TIME_DATE_STAMP};
use yp_host::gpu::{GpuProbe, PREFER_DEDICATED_GPU};
use yp_host::launch::record::{self, LaunchRecord};
use yp_host::nams_log::LogLevel;
use yp_host::platform::linux::linux_adapter_with;
use yp_host::wine::WineContext;

fn is_nams(spec: &ProcessSpec, first_arg: &str) -> bool {
    spec.program
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("NAMS.exe"))
        && spec.args_lossy().first().map(String::as_str) == Some(first_arg)
}

fn linux_rig() -> Rig {
    let mount = "{\"filesystems\": [{\"target\": \"/mnt/win\", \"fstype\": \"ntfs3\", \"options\": \"rw\"}]}";
    let runner = FakeRunner::new()
        .respond_program("findmnt", ProcessOutput::success(mount))
        .respond_program("pgrep", ProcessOutput::failure(1, ""));
    let rig = Rig::with_runner(Os::Linux, runner);
    let files = SystemFiles::under(&rig.system_root());
    write(&files.os_release, "PRETTY_NAME=\"Bazzite 42\"\n");
    write(&files.kernel_dir.join("ostype"), "Linux\n");
    write(&files.kernel_dir.join("osrelease"), "6.11.2\n");
    rig
}

fn probing_collector(rig: &Rig, gpu: GpuProbe) -> Collector {
    Collector::new(
        rig.host.clone(),
        rig.paths.clone(),
        rig.settings.clone(),
        Arc::new(rig.runner.clone()),
        Arc::new(rig.platform()),
    )
    .with_wine_context(WineContext::isolated(rig.host.clone()))
    .with_system_files(SystemFiles::under(&rig.system_root()))
    .with_gpu_probe(gpu)
}

#[test]
fn linux_host_reports_filesystems_gpu_and_a_missing_compat_layer() {
    let rig = linux_rig();
    let game = FakeGame::create();
    let hybrid = GpuProbe {
        nvidia_driver_loaded: true,
        render_nodes: 2,
    };

    let report = probing_collector(&rig, hybrid)
        .collect(Some(&game.paths()), None, &CancelToken::new())
        .unwrap();

    assert_eq!(report.host.os, Os::Linux);
    let keys: Vec<&str> = report.system_info.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "OS",
            "OS version",
            "Locale",
            "Game running",
            "Steam running",
            "Prefer dedicated GPU",
            "Distro",
            "Session",
            "Desktop",
            "AppImage",
            "Flatpak",
            "Steam Deck",
            "NVIDIA driver",
            "DRM render nodes",
            "Dedicated GPU env",
        ]
    );
    assert_eq!(report.system_info["OS"], "linux");
    assert_eq!(report.system_info["OS version"], "Linux 6.11.2");
    assert_eq!(report.system_info["Distro"], "Bazzite 42");
    assert_eq!(report.system_info["Steam running"], "no");
    assert_eq!(report.system_info["Prefer dedicated GPU"], "yes");
    assert_eq!(report.system_info["NVIDIA driver"], "yes");
    assert_eq!(report.system_info["DRM render nodes"], "2");
    assert_eq!(
        report.system_info["Dedicated GPU env"],
        "__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia __VK_LAYER_NV_optimus=NVIDIA_only"
    );

    let roles: Vec<FilesystemRole> = report.filesystems.keys().copied().collect();
    assert_eq!(
        roles,
        [FilesystemRole::GameFolder, FilesystemRole::LauncherFolder]
    );
    assert_eq!(
        report.filesystems[&FilesystemRole::GameFolder]
            .as_ref()
            .unwrap()
            .fs_type,
        "ntfs3"
    );
    assert_eq!(report.compat_layer, None);
    assert_eq!(report.texture_packs_note, Some(NOTE_WINDOWS_ONLY));
    assert_eq!(report.nams_health.steam_running, Some(false));

    let level_of = |needle: &str| {
        report
            .findings
            .iter()
            .find(|finding| finding.message.contains(needle))
            .map(|finding| finding.level)
    };
    assert_eq!(
        level_of("The game folder is on ntfs3"),
        Some(FindingLevel::Error)
    );
    assert_eq!(
        level_of("The launcher folder is on ntfs3"),
        Some(FindingLevel::Warning)
    );
    assert_eq!(
        level_of("No Proton, Wine or CrossOver found to run NAMS.exe."),
        Some(FindingLevel::Error)
    );
    assert_eq!(
        level_of("Steam is not running."),
        Some(FindingLevel::Warning)
    );

    let full = format_full_report(&report);
    assert!(full.contains("\nCompatibility layer\n"));
    assert!(full.contains("\n(none found)\n"));
    assert!(full.contains("NAMS version: not queried (Windows only)\n"));
    assert!(
        full.contains("NAMS query: NAMS texture query runs NAMS.exe natively (Windows only)\n")
    );
    assert!(full.contains("  game folder: ntfs3 (mount /mnt/win, rw)\n"));
    let summary = format_summary(&report);
    assert!(summary.contains("Distro: Bazzite 42   Session: ?   AppImage: no   Flatpak: no\n"));
    assert!(summary.contains("FS game folder: ntfs3\nFS launcher folder: ntfs3\n"));
}

#[test]
fn dedicated_gpu_env_is_none_when_the_preference_is_off() {
    let rig = linux_rig();
    rig.settings.set(&PREFER_DEDICATED_GPU, &false).unwrap();
    let hybrid = GpuProbe {
        nvidia_driver_loaded: false,
        render_nodes: 2,
    };
    let report = probing_collector(&rig, hybrid)
        .collect(None, None, &CancelToken::new())
        .unwrap();
    assert!(!report.prefer_dedicated_gpu);
    assert_eq!(report.system_info["Prefer dedicated GPU"], "no");
    assert_eq!(report.system_info["Dedicated GPU env"], "none");
    assert_eq!(report.texture_packs_note, Some(NOTE_NO_GAME_FOLDER));
    let roles: Vec<FilesystemRole> = report.filesystems.keys().copied().collect();
    assert_eq!(roles, [FilesystemRole::LauncherFolder]);
    assert!(format_full_report(&report).contains("\n(no game folder)\n"));

    rig.settings.set(&PREFER_DEDICATED_GPU, &true).unwrap();
    let mesa = probing_collector(&rig, hybrid)
        .collect(None, None, &CancelToken::new())
        .unwrap();
    assert_eq!(mesa.system_info["Dedicated GPU env"], "DRI_PRIME=1");
}

#[test]
fn windows_host_asks_nams_for_its_version_and_texture_packs() {
    let game = FakeGame::create();
    let paths = game.paths();
    let packs = json!({
        "packs": [
            {
                "name": "Off",
                "source": "standalone",
                "enabled": false,
                "dds_count": 4,
                "path": paths.inject_textures_dir().join("Off").to_string_lossy(),
            },
            {
                "name": "Kimono",
                "source": "mod",
                "mod": "x",
                "dds_count": 9,
                "load_order_index": 1,
                "path": paths.mods_dir().join("x").join("textures").join("Kimono").to_string_lossy(),
            },
        ]
    });
    let info = json!({
        "version": "1.2.3",
        "paths": { "nier_install_error": "Steam library not found" },
    });
    let runner = FakeRunner::new()
        .respond(
            |spec| is_nams(spec, "info"),
            FakeResponse::Output(ProcessOutput::success(info.to_string())),
        )
        .respond(
            |spec| is_nams(spec, "textures"),
            FakeResponse::Output(ProcessOutput::success(packs.to_string())),
        );
    let rig = Rig::with_runner(Os::Windows, runner);
    rig.install_runtime_files();
    write(&paths.disabled_mods_toml(), "disabled = [\"mods/x\"]\n");

    let report = rig.collect(Some(game.path()));
    let health = &report.nams_health;
    assert!(health.nams_exe_present);
    assert!(health.missing_files.is_empty());
    assert_eq!(health.nams_version.as_deref(), Some("1.2.3"));
    assert_eq!(
        health.nier_install_error.as_deref(),
        Some("Steam library not found")
    );
    assert_eq!(
        health
            .nams_exe
            .as_ref()
            .unwrap()
            .short_sha1
            .as_ref()
            .unwrap()
            .len(),
        12
    );
    let plugins: Vec<&str> = health
        .plugins
        .iter()
        .map(|plugin| plugin.name.as_str())
        .collect();
    assert_eq!(plugins, ["yorha_protocol.dll"]);

    assert!(report.texture_packs_available);
    assert_eq!(report.texture_packs_note, None);
    assert_eq!(report.texture_packs.len(), 2);
    assert_eq!(
        pack_disabled_by(&report, &report.texture_packs[0]),
        Some(PackSwitch::TextureInjectionToml)
    );
    assert_eq!(
        pack_disabled_by(&report, &report.texture_packs[1]),
        Some(PackSwitch::DisabledModsToml)
    );
    let messages = messages(&report);
    assert!(messages.contains("NAMS: Steam library not found"));
    assert!(messages.contains("Every texture pack is disabled."));
    assert!(!messages.contains("Launcher files missing"));

    let calls: Vec<Vec<String>> = rig
        .runner
        .calls()
        .iter()
        .filter(|spec| spec.program == rig.paths.nams_exe())
        .map(|spec| spec.args_lossy())
        .collect();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0][..2], ["info", "--nier-path"]);
    assert_eq!(calls[1][..3], ["textures", "list", "--nier-path"]);

    let full = format_full_report(&report);
    assert!(full.contains("NAMS version: 1.2.3\n"));
    assert!(full.contains("NAMS install lookup: Steam library not found\n"));
    assert!(full.contains("\nTexture packs (2)\n"));
    assert!(full.contains("- Off  [standalone]  disabled (texture_injection.toml)  4 dds\n"));
    assert!(
        full.contains("- Kimono  [mod]  disabled (disabled_mods.toml)  9 dds  mod=x  order=1\n")
    );
    assert!(!full.contains("Compatibility layer"));
    let summary = format_summary(&report);
    assert!(summary.contains("NAMS: exe=true v1.2.3   missing=0   Steam running=yes\n"));
    assert!(summary.contains("texture packs: 0/2 on"));
}

#[test]
fn failed_nams_queries_leave_a_note_instead_of_packs() {
    let runner = FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::failure(1, "boom"));
    let rig = Rig::with_runner(Os::Windows, runner);
    let game = FakeGame::create();

    let missing = rig.collect(Some(game.path()));
    assert_eq!(missing.texture_packs_note, Some(NOTE_NAMS_MISSING));
    assert!(rig.runner.calls().is_empty());

    rig.install_runtime_files();
    let failed = rig.collect(Some(game.path()));
    assert!(!failed.texture_packs_available);
    assert_eq!(failed.texture_packs_note, Some(NOTE_QUERY_FAILED));
    assert_eq!(failed.nams_health.nams_version, None);
    let full = format_full_report(&failed);
    assert!(full.contains("NAMS version: unavailable\n"));
    assert!(full.contains("NAMS query: NAMS texture query failed or timed out\n"));

    let none = rig.collect(None);
    assert_eq!(none.texture_packs_note, Some(NOTE_NO_GAME_FOLDER));
}

#[test]
fn cancelled_collection_returns_the_cancelled_error() {
    let rig = Rig::native();
    let game = FakeGame::create();
    let cancel = CancelToken::new();
    cancel.cancel();

    let error = rig
        .collector()
        .collect(Some(&game.paths()), None, &cancel)
        .unwrap_err();
    assert!(error.is_cancelled());
}

#[test]
fn mods_textures_profiles_and_stray_folders_are_read_from_the_game_folder() {
    let rig = Rig::native();
    let game = FakeGame::create();
    let paths = game.paths();
    let root = game.path();
    write(
        &paths
            .mods_dir()
            .join("alpha")
            .join("data")
            .join("pl")
            .join("pl000d.dat"),
        [0u8; 4],
    );
    std::fs::create_dir_all(paths.mods_dir().join("_parked")).unwrap();
    write(&paths.disabled_mods_toml(), "disabled = [\"mods/alpha\"]\n");
    write(&paths.profiles_toml(), "active = \"speedrun\"\n");
    write(
        &paths.default_mods_toml(),
        "outfit_bare = [\"mods/alpha\"]\noutfit_config = [{ path = \"mods/alpha\", outfit_id = 3 }]\n",
    );
    std::fs::create_dir_all(paths.inject_textures_dir().join("PackB")).unwrap();
    std::fs::create_dir_all(paths.inject_textures_dir().join("packA")).unwrap();
    write(&paths.inject_textures_dir().join("readme.txt"), "x");
    std::fs::create_dir_all(
        paths
            .special_k_textures_dir()
            .join("NieRAutomata.exe")
            .join("SkPack"),
    )
    .unwrap();
    std::fs::create_dir_all(
        paths
            .special_k_textures_dir()
            .join("Other.exe")
            .join("SkOther"),
    )
    .unwrap();
    std::fs::create_dir_all(paths.wax_mods_dir().join("w1").join("textures")).unwrap();
    std::fs::create_dir_all(paths.wax_mods_dir().join("w2")).unwrap();
    write(&root.join("data").join("extra").join("a.bin"), [0u8]);
    write(
        &root.join("data").join("extra").join("deep").join("b.bin"),
        [0u8],
    );
    write(&root.join("data").join("Movie").join("op.usm"), [0u8]);
    write(&root.join("reshade").join("ReShade.ini"), "x");
    write(&root.join("notes.txt"), "x");
    write(
        &paths.cutscenes_dir().join("hd").join("movie").join("a.usm"),
        [0u8; 8],
    );

    let report = rig.collect(Some(root));
    let ids: Vec<&str> = report
        .mods
        .iter()
        .map(|installed| installed.id.as_str())
        .collect();
    assert_eq!(ids, ["alpha"]);
    assert_eq!(report.hidden_mod_folders, ["_parked"]);
    assert_eq!(report.disabled_mods_entries, ["mods/alpha"]);
    assert_eq!(report.active_profile.as_deref(), Some("speedrun"));
    assert_eq!(
        report.default_mods_entries,
        [
            "outfit_bare = [\"mods/alpha\"]",
            "outfit_config = [{path = \"mods/alpha\", outfit_id = 3}]",
        ]
    );
    assert_eq!(report.nams_textures, ["packA", "PackB"]);
    assert_eq!(report.sk_res_textures, ["SkPack"]);
    assert_eq!(report.sk_res_packs, ["SkPack", "SkOther"]);
    assert_eq!(report.wax_textures, ["w1"]);
    assert_eq!(report.wax_mods_root, ["w1", "w2"]);
    let data_dirs: Vec<(&str, usize)> = report
        .data_dir_contents
        .iter()
        .map(|(name, count)| (name.as_str(), *count))
        .collect();
    assert_eq!(data_dirs, [("extra", 2)]);
    assert_eq!(report.reshade_contents, ["ReShade.ini"]);
    assert_eq!(
        report.game_root_extras,
        ["notes.txt", "reshade/", "SK_Res/", "wax/"]
    );
    assert_eq!(report.cutscene_mods.len(), 1);
    assert_eq!(report.cutscene_mods[0].name, "hd");
    assert_eq!(report.cutscene_mods[0].usm_count(), 1);
    let errors: Vec<&str> = report.collection_errors.keys().map(String::as_str).collect();
    assert_eq!(errors, ["settings"]);

    let full = format_full_report(&report);
    assert!(full.contains("\n- alpha  [DISABLED]  ["));
    assert!(full.contains(
        "With warnings: 0   With bundled assets: 0   Disabled prefixes affecting mods: 1\n"
    ));
    assert!(full.contains(
        "active profile: speedrun\ndefault_mods.toml:\n  outfit_bare = [\"mods/alpha\"]\n"
    ));
    assert!(full.contains("nams/inject/textures/ (2): packA, PackB\n"));
    assert!(full.contains("- hd  [standalone]  1 USM  MPEG-2\n"));
    assert!(full.contains("Non-vanilla folders in data/:\n  extra/  (2 files)\n"));
    assert!(
        format_summary(&report)
            .contains("Mods: 1   disabled prefixes: 1   texture packs: 2   cutscene mods: 1")
    );
}

fn config_answer(document: serde_json::Value) -> FakeResponse {
    FakeResponse::Output(ProcessOutput::success(&document.to_string()))
}

fn is_config(spec: &ProcessSpec, command: &str) -> bool {
    is_nams(spec, "config") && spec.args_lossy().get(1).map(String::as_str) == Some(command)
}

#[test]
fn unreadable_settings_file_becomes_a_collection_error_without_leaking_its_path() {
    let runner = FakeRunner::new()
        .respond(
            |spec| is_config(spec, "schema"),
            config_answer(yp_config::testing::sample_schema()),
        )
        .respond(
            |spec| is_config(spec, "get"),
            config_answer(json!({
                "ok": true,
                "files": {
                    "nams.toml": { "exists": true },
                    "naiom.toml": { "exists": true, "error": "not a file" },
                },
                "values": { "nams.skip_startup_logos": true },
            })),
        )
        .respond(
            |spec| is_config(spec, "validate"),
            config_answer(json!({ "ok": true, "valid": true, "problems": [] })),
        );
    let rig = Rig::with_runner(Os::native(), runner);
    rig.install_runtime_files();
    let game = FakeGame::create();
    std::fs::create_dir_all(game.paths().naiom_toml()).unwrap();

    let report = Collector::new(
        rig.host.clone(),
        rig.paths.clone(),
        rig.settings.clone(),
        Arc::new(rig.runner.clone()),
        Arc::new(FakePlatform::running_config(rig.host.clone())),
    )
    .with_wine_context(WineContext::isolated(rig.host.clone()))
    .with_system_files(SystemFiles::under(&rig.system_root()))
    .with_gpu_probe(GpuProbe::default())
    .with_steam_running(Some(true))
    .collect(Some(&game.paths()), None, &CancelToken::new())
    .unwrap();
    assert!(!report.collection_errors.contains_key("settings"));
    assert_eq!(report.config.parse_errors["naiom.toml"], "not a file");
    assert_eq!(report.config.deltas.len(), 1);
    assert_eq!(report.config.deltas[0].key, "skip_startup_logos");
    let error = &report.collection_errors["naiom.toml"];
    assert!(error.contains(&game.text()));
    let finding = report
        .findings
        .iter()
        .find(|finding| {
            finding
                .message
                .starts_with("Could not collect naiom.toml: ")
        })
        .unwrap();
    assert_eq!(finding.level, FindingLevel::Warning);

    for text in [format_full_report(&report), format_summary(&report)] {
        assert!(text.contains("Could not collect naiom.toml: "));
        assert!(text.contains("<gameDir>"));
        assert!(!text.contains(&game.text()));
    }
}

#[test]
fn settings_become_a_collection_error_while_nams_cannot_answer() {
    let rig = Rig::native();
    let game = FakeGame::create();

    let report = rig.collect(Some(game.path()));
    assert!(report.collection_errors.contains_key("settings"));
    assert!(report.config.deltas.is_empty());
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.message.starts_with("Could not collect settings: "))
    );
}

#[test]
fn collection_errors_keep_one_capped_line() {
    assert_eq!(short_error(&"first line\nsecond line"), "first line");
    assert_eq!(short_error(&""), "");
    let long = "x".repeat(250);
    let shown = short_error(&long);
    assert_eq!(shown.chars().count(), 203);
    assert!(shown.ends_with("..."));
    assert_eq!(short_error(&"y".repeat(200)), "y".repeat(200));
}

#[test]
fn stale_game_bin_heartbeat_and_log_issues_are_reported() {
    let rig = Rig::native();
    let game = FakeGame::create();
    write(&game.exe(), pe_header(STEAM_2026_TIME_DATE_STAMP, 0x80));
    write(
        &rig.paths.runtime_dir.join("runtime").join("game.bin"),
        pe_header(STEAM_2021_TIME_DATE_STAMP, 0x80),
    );
    let launched = LaunchRecord {
        started_at: Timestamp::now() - SignedDuration::from_secs(300),
        ended_at: None,
        exit_code: None,
        label: "NAMS".to_owned(),
        output_tail: String::new(),
    };
    record::write(&rig.paths, &launched).unwrap();
    rig.write_log("heartbeat.log", "main thread stalled 30s\n");
    let mut log = String::from("2026-11-06T10:00:00.000000Z  INFO modloader: Startup\n");
    for index in 0..70 {
        log.push_str(&format!(
            "2026-11-06T10:00:01.{index:06}Z  WARN nier_rs::content::outfits: slow outfit {index}\n"
        ));
    }
    log.push_str("2026-11-06T10:00:02.000000Z ERROR yorha: plugin failed\nnot a log line\n");
    rig.write_log("nams.log", &log);
    rig.write_log("nams_2026-11-06T10-00-00-000000.log", "captured\n");

    let report = rig.collect(Some(game.path()));
    let identity = &report.game_identity;
    assert_eq!(
        identity.game_bin_time_date_stamp,
        Some(STEAM_2021_TIME_DATE_STAMP)
    );
    assert!(!identity.game_bin_matches_exe());
    assert_eq!(identity.game_bin_build_label(), Some("Steam 2021"));
    assert_eq!(identity.game_bin.as_ref().unwrap().name, "game.bin");

    assert_eq!(report.recent_log_issues.len(), 60);
    assert_eq!(report.recent_log_issues[0].message, "slow outfit 11");
    assert_eq!(
        report.recent_log_issues[0].module,
        "nier_rs::content::outfits"
    );
    assert_eq!(report.recent_log_issues[59].level, LogLevel::Error);
    assert_eq!(report.logs.nams_log.as_ref().unwrap().lines.len(), 40);
    assert_eq!(
        report.logs.heartbeat.as_ref().unwrap().lines,
        ["main thread stalled 30s"]
    );
    assert_eq!(report.logs.captured_launch_logs.len(), 1);
    assert_eq!(report.logs.crash_dump_count, 0);

    let messages = messages(&report);
    assert!(messages.contains(
        "runtime/game.bin was built from Steam 2021; NAMS rebuilds it from the installed exe on the next launch."
    ));
    assert!(
        messages.contains(
            "The heartbeat watchdog reported a freeze in the last session (heartbeat.log)."
        )
    );
    assert!(!messages.contains("Last launch ended"));

    let full = format_full_report(&report);
    assert!(full.contains("\nRecent NAMS warnings/errors (60)\n"));
    assert!(full.contains("ERROR yorha: plugin failed\n"));
    assert!(full.contains("ended: not recorded (still running, or the launcher closed first)\n"));
    assert!(full.contains("(Steam 2021, rebuilt on next launch)\n"));
    assert!(full.contains(
        "captured early-exit logs:\n  - nams_2026-11-06T10-00-00-000000.log  9 B  modified "
    ));
    assert!(format_summary(&report).contains("exit=running/unknown\n"));
}

#[test]
fn game_folder_without_an_exe_is_the_first_error() {
    let rig = Rig::native();
    let game = FakeGame::create();
    std::fs::remove_file(game.exe()).unwrap();
    let report = rig.collect(Some(game.path()));
    assert_eq!(report.findings[0].level, FindingLevel::Error);
    assert_eq!(
        report.findings[0].message,
        "NieRAutomata.exe not found in the game folder."
    );
    assert_eq!(report.game_identity.build_display(), "missing");
    assert_eq!(
        report.game_dir.as_deref(),
        Some(GamePaths::new(game.path()).root())
    );
}

#[test]
fn host_platform_adapters_plug_into_collection_notifications_and_cache() {
    let runner = FakeRunner::new().respond_program("pgrep", ProcessOutput::success("4242\n"));
    let rig = Rig::with_runner(Os::Linux, runner);
    let shared_runner: Arc<dyn ProcessRunner> = Arc::new(rig.runner.clone());
    let platform: Arc<dyn HostPlatform> = Arc::new(linux_adapter_with(
        WineContext::isolated(rig.host.clone()),
        shared_runner.clone(),
    ));

    let collector = Collector::new(
        rig.host.clone(),
        rig.paths.clone(),
        rig.settings.clone(),
        shared_runner,
        platform.clone(),
    )
    .with_wine_context(WineContext::isolated(rig.host.clone()))
    .with_system_files(SystemFiles::under(&rig.system_root()))
    .with_gpu_probe(GpuProbe::default());
    let report = collector.collect(None, None, &CancelToken::new()).unwrap();
    assert!(report.game_running);
    assert_eq!(report.system_info["Game running"], "yes");
    assert_eq!(report.nams_health.steam_running, Some(true));
    assert!(format_summary(&report).contains("running=true\n"));

    assert_eq!(
        platform_support_notice(platform.as_ref()),
        Some(NotificationKind::PlatformUnsupported { os: Os::Linux })
    );
    rig.write_log("nams.log", "x\n");
    let cleared = clear_all(&rig.paths, &rig.settings, platform.as_ref(), None);
    assert_eq!(cleared.logs_deleted, 1);
}

#[test]
fn collector_and_report_can_cross_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Collector>();
    assert_send_sync::<yp_health::DiagnosticsReport>();

    let rig = Rig::native();
    let collector = rig.collector();
    let report =
        std::thread::spawn(move || collector.collect(None, None, &CancelToken::new()).unwrap())
            .join()
            .unwrap();
    assert_eq!(
        report.findings[0].message,
        "Launcher files missing: NAMS.exe, yorha_protocol.dll (antivirus quarantine?). Restore them or reinstall the launcher."
    );
}
