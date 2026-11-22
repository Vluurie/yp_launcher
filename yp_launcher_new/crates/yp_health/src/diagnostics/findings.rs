use std::path::Path;

use jiff::SignedDuration;
use yp_core::{GamePaths, Os};
use yp_host::game_probe::{ExeBuildStatus, GameBuild, KNOWN_BUILDS};
use yp_host::launch::nams_dll;
use yp_host::nams_cli::NamsTexturePack;
use yp_thirdparty::GameModSettings;

use super::model::{
    DiagnosticFinding, DiagnosticsReport, FilesystemRole, FindingLevel, PackSwitch,
};
use super::text::cap_list;

pub const FINDING_LIST_CAP: usize = 4;
pub const UNKNOWN_KEYS_SHOWN: usize = 4;
pub const MISSING_REQUIRES_SHOWN: usize = 5;
pub const QUICK_EXIT_SECONDS: u64 = 60;
pub const CRASH_DUMP_SESSION_SLACK: SignedDuration = SignedDuration::from_mins(1);
pub const CRASH_DUMP_RECENT: SignedDuration = SignedDuration::from_hours(7 * 24);
pub const SPECIAL_K_DIR_ENTRY: &str = "sk_res/";
pub const SPECIAL_K_DLL: &str = "dinput8.dll";

pub fn is_steam_2026_build(label: Option<&str>) -> bool {
    label.is_some_and(|label| {
        KNOWN_BUILDS.iter().any(|info| info.build == GameBuild::Steam2026_11 && info.label == label)
    })
}

pub const STEAM_BUILD_IDS: [(&str, &str); 2] = [
    ("25901062", "Steam 2026-11"),
    ("26534214", "Steam 2026-11 (signed)"),
];

pub const GAME_FILE_EXTENSIONS: [&str; 21] = [
    "dat", "dtt", "cpk", "usm", "wsp", "wem", "bnk", "wtp", "wta", "wtb", "wmb", "bxm", "mcd",
    "tmd", "smd", "ktb", "eff", "evn", "mot", "dll", "exe",
];

pub const PROXY_DLL_NAMES: [&str; 12] = [
    "d3d9.dll",
    "d3d10.dll",
    "d3d11.dll",
    "d3d12.dll",
    "dxgi.dll",
    "dinput8.dll",
    "xinput1_3.dll",
    "xinput1_4.dll",
    "xinput9_1_0.dll",
    "winmm.dll",
    "version.dll",
    "dsound.dll",
];

pub fn steam_build_label(build_id: &str) -> Option<&'static str> {
    STEAM_BUILD_IDS
        .iter()
        .find(|(id, _)| *id == build_id)
        .map(|(_, label)| *label)
}

pub fn pack_disabled_by(report: &DiagnosticsReport, pack: &NamsTexturePack) -> Option<PackSwitch> {
    if !pack.enabled {
        return Some(PackSwitch::TextureInjectionToml);
    }
    let game_dir = report.game_dir.as_ref()?;
    if report.disabled_mods_entries.is_empty() {
        return None;
    }
    let relative =
        yp_mods::disabled::to_relative(&GamePaths::new(game_dir), Path::new(&pack.path))?;
    yp_mods::disabled::matches(&report.disabled_mods_entries, &relative)
        .then_some(PackSwitch::DisabledModsToml)
}

fn is_game_file(path: &str) -> bool {
    Path::new(path).extension().is_some_and(|extension| {
        GAME_FILE_EXTENSIONS.contains(&extension.to_string_lossy().to_lowercase().as_str())
    })
}

fn base_name(path: &str) -> String {
    yp_core::fs::file_name_lossy(Path::new(path))
}

const NAMS_FILE: &str = "nams.toml";
const DISABLE_PLUGIN_LOADING_KEY: &str = "disable_plugin_loading";
const DISABLE_TEXTURE_INJECTION_KEY: &str = "disable_texture_injection";

fn delta_is(report: &DiagnosticsReport, file: &str, key: &str, value: &str) -> bool {
    report
        .config
        .deltas
        .iter()
        .any(|delta| delta.file == file && delta.key == key && delta.value == value)
}

struct Findings {
    out: Vec<DiagnosticFinding>,
}

impl Findings {
    fn add(&mut self, level: FindingLevel, message: impl Into<String>) {
        self.out.push(DiagnosticFinding::new(level, message));
    }

    fn error(&mut self, message: impl Into<String>) {
        self.add(FindingLevel::Error, message);
    }

    fn warning(&mut self, message: impl Into<String>) {
        self.add(FindingLevel::Warning, message);
    }

    fn info(&mut self, message: impl Into<String>) {
        self.add(FindingLevel::Info, message);
    }
}

fn game_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    let identity = &report.game_identity;
    match identity.build_status {
        ExeBuildStatus::Missing => findings.error("NieRAutomata.exe not found in the game folder."),
        ExeBuildStatus::LegacyWindows7 => findings.error(
            "NieRAutomata.exe is the Windows 7/8 build. NAMS needs the Windows 10/11 Steam build: set \
             Proton Experimental, delete the exe and verify game files.",
        ),
        ExeBuildStatus::Modified => findings.error(format!(
            "NieRAutomata.exe carries the build stamp of {} but a different SHA-1{}. Either a \
             NieR:Automata update changed the exe and this NAMS does not support it yet (update the \
             launcher), or it is a modified copy (verify game files in Steam).",
            identity.stamp_build_label.unwrap_or("?"),
            identity.wolf_limit_break_suffix()
        )),
        ExeBuildStatus::Newer => findings.error(
            "NieRAutomata.exe is a newer game update than this NAMS knows. Update the launcher.",
        ),
        ExeBuildStatus::Unknown => findings.error(format!(
            "NieRAutomata.exe is not a build NAMS knows (size {}, sha1 {}).",
            identity
                .exe_size
                .map_or_else(|| "?".to_owned(), |size| size.to_string()),
            identity.exe_sha1.as_deref().unwrap_or("?")
        )),
        ExeBuildStatus::Known => {}
    }
    if let Some(manifest) = &identity.steam_manifest {
        let build_id = manifest.build_id.as_deref();
        if manifest.update_pending() {
            findings.warning(format!(
                "Steam has a game update pending (build {} -> {}). Let Steam finish it.",
                build_id.unwrap_or("?"),
                manifest.target_build_id.as_deref().unwrap_or("?")
            ));
        }
        if let Some(beta) = &manifest.beta_key {
            findings.info(format!("Steam beta branch selected: {beta}."));
        }
        let steam_build = build_id.and_then(steam_build_label);
        if let (Some(build_id), Some(steam_build)) = (build_id, steam_build)
            && identity.build_status == ExeBuildStatus::Known
            && identity.build_label != Some(steam_build)
        {
            findings.warning(format!(
                "Steam reports build {build_id} ({steam_build}) but NieRAutomata.exe is {}: the exe was \
                 replaced by hand.",
                identity.build_label.unwrap_or("?")
            ));
        }
    }
    if identity.game_bin.is_some()
        && identity.exe_time_date_stamp.is_some()
        && !identity.game_bin_matches_exe()
    {
        findings.info(format!(
            "runtime/game.bin was built from {}; NAMS rebuilds it from the installed exe on the next \
             launch.",
            identity.game_bin_build_label().unwrap_or("another exe")
        ));
    }
}

fn runtime_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    let health = &report.nams_health;
    if !health.missing_files.is_empty() {
        findings.error(format!(
            "Launcher files missing: {} (antivirus quarantine?). Restore them or reinstall the launcher.",
            health.missing_files.join(", ")
        ));
    }
    if let Some(error) = &health.nier_install_error {
        findings.warning(format!("NAMS: {error}"));
    }
    if health.steam_running == Some(false) {
        findings.warning("Steam is not running. NAMS needs the Steam client to start the game.");
    }
}

fn config_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    for (file, error) in &report.config.parse_errors {
        findings.error(format!("{file} does not parse: {error}"));
    }
    let unknown = &report.config.unknown_keys;
    if !unknown.is_empty() {
        let keys: Vec<String> = unknown
            .iter()
            .take(UNKNOWN_KEYS_SHOWN)
            .map(|delta| format!("{}: {}", delta.file, delta.key))
            .collect();
        let more = if unknown.len() > UNKNOWN_KEYS_SHOWN {
            format!(" and {} more", unknown.len() - UNKNOWN_KEYS_SHOWN)
        } else {
            String::new()
        };
        findings.info(format!(
            "Unknown setting(s) NAMS does not read (typo?): {}{more}.",
            keys.join(", ")
        ));
    }
    if delta_is(report, NAMS_FILE, DISABLE_PLUGIN_LOADING_KEY, "true") {
        findings.warning(
            "Plugin loading is disabled (disable_plugin_loading): YoRHa Protocol and all plugins are off.",
        );
    }
    if delta_is(report, NAMS_FILE, DISABLE_TEXTURE_INJECTION_KEY, "true")
        && (!report.nams_textures.is_empty() || !report.texture_packs.is_empty())
    {
        findings.info("Texture packs are installed but texture injection is disabled.");
    }
}

fn host_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    for (role, mount) in &report.filesystems {
        let Some(mount) = mount.as_ref().filter(|mount| mount.risky()) else {
            continue;
        };
        let level = if *role == FilesystemRole::GameFolder {
            FindingLevel::Error
        } else {
            FindingLevel::Warning
        };
        findings.add(
            level,
            format!(
                "The {} is on {}. Under Proton, Play and Verify hang on NTFS/FAT drives: move it to a \
                 Linux filesystem (ext4, btrfs).",
                role.label(),
                mount.fs_type
            ),
        );
    }
    if report.host.os != Os::Windows && report.has_game() && report.compat_layer.is_none() {
        findings.error("No Proton, Wine or CrossOver found to run NAMS.exe.");
    }
}

fn session_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    let last = report.last_launch.as_ref();
    if let Some(last) = last
        && let Some(code) = last.exit_code.filter(|code| *code != 0)
    {
        let seconds = last.duration().map(|duration| duration.as_secs());
        let level = if seconds.is_some_and(|seconds| seconds < QUICK_EXIT_SECONDS) {
            FindingLevel::Warning
        } else {
            FindingLevel::Info
        };
        let after = seconds
            .map(|seconds| format!(" after {seconds}s"))
            .unwrap_or_default();
        findings.add(
            level,
            format!("Last launch ended with exit code {code}{after}."),
        );
    }
    if let Some(crash) = &report.logs.newest_crash_dump
        && let Some(modified) = crash.modified
    {
        let from_last_session = last.is_some_and(|last| {
            last.started_at
                .checked_sub(CRASH_DUMP_SESSION_SLACK)
                .is_ok_and(|earliest| modified > earliest)
        });
        let recent = report.generated_at.duration_since(modified) < CRASH_DUMP_RECENT;
        if from_last_session || recent {
            let level = if from_last_session {
                FindingLevel::Warning
            } else {
                FindingLevel::Info
            };
            findings.add(
                level,
                format!(
                    "NAMS wrote a crash dump{}: {}.",
                    if from_last_session {
                        " in the last session"
                    } else {
                        ""
                    },
                    crash.name
                ),
            );
        }
    }
    if let Some(heartbeat) = &report.logs.heartbeat
        && let Some(modified) = heartbeat.modified
        && last.is_some_and(|last| modified > last.started_at)
    {
        findings.warning(
            "The heartbeat watchdog reported a freeze in the last session (heartbeat.log).",
        );
    }
}

fn game_folder_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    let game_files: Vec<String> = report
        .vanilla_dropped
        .iter()
        .filter(|drop| is_game_file(&drop.path))
        .map(|drop| base_name(&drop.path))
        .collect();
    if !game_files.is_empty() {
        findings.warning(format!(
            "{} game file(s) dropped loose into vanilla data/ ({}). NAMS mods belong in nams/mods.",
            game_files.len(),
            cap_list(&game_files, FINDING_LIST_CAP)
        ));
    }

    let proxies: Vec<&str> = report
        .game_root_extras
        .iter()
        .map(String::as_str)
        .filter(|name| PROXY_DLL_NAMES.contains(&name.to_lowercase().as_str()))
        .filter(|name| !(report.nams_dll_picked && name.eq_ignore_ascii_case(nams_dll::PROXY_NAME)))
        .collect();
    let is_special_k_dll = |name: &str| name.to_lowercase() == SPECIAL_K_DLL;
    let special_k = report
        .game_root_extras
        .iter()
        .any(|name| name.to_lowercase() == SPECIAL_K_DIR_ENTRY)
        && proxies.iter().any(|name| is_special_k_dll(name));
    let others: Vec<&str> = proxies
        .into_iter()
        .filter(|name| !(special_k && is_special_k_dll(name)))
        .collect();
    if special_k {
        findings.info(
            "Special K (dinput8.dll in the game folder) does not run under NAMS. NAMS reads SK_Res \
             texture packs itself.",
        );
    }
    if !others.is_empty() {
        findings.warning(format!(
            "{} in the game folder {} never loaded: NAMS runs the game from its own folder. Install DLL \
             mods through Third Party, which puts them in nams/thirdparty/graphic or \
             nams/thirdparty/game.",
            others.join(", "),
            if others.len() == 1 { "is" } else { "are" }
        ));
    }
}

fn mod_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    let game_mods = &report.third_party.game_mods;
    let active_dlls = game_mods
        .game_mods_info
        .iter()
        .flat_map(|info| info.mods.iter())
        .filter(|entry| !entry.disabled);
    if game_mods.enabled && is_steam_2026_build(report.game_identity.build_label) {
        let told_2021: Vec<&str> = active_dlls
            .filter(|entry| {
                entry.settings.is_none_or(|settings| {
                    settings.size_of_image == GameModSettings::STEAM_2021_SIZE_OF_IMAGE
                })
            })
            .map(|entry| entry.file_name.as_str())
            .collect();
        if !told_2021.is_empty() {
            findings.warning(format!(
                "Game DLL mods ({}) are told the game is the 2021 build, but Steam 2026-11 is installed. \
                 A mod made for 2021 reads the wrong memory there; check that it supports the update or \
                 turn it off in Third Party.",
                cap_list(&told_2021, FINDING_LIST_CAP)
            ));
        }
    }

    if !report.texture_packs.is_empty()
        && report
            .texture_packs
            .iter()
            .all(|pack| pack_disabled_by(report, pack).is_some())
    {
        findings.info("Every texture pack is disabled.");
    }
    for installed in report
        .mods
        .iter()
        .filter(|installed| !installed.requires_missing.is_empty())
        .take(MISSING_REQUIRES_SHOWN)
    {
        findings.warning(format!(
            "Mod {} needs missing mod(s): {}.",
            installed.id,
            installed.requires_missing.join(", ")
        ));
    }
    if !report.hidden_mod_folders.is_empty() {
        findings.info(format!(
            "{} mod folder(s) start with \"_\": hidden in the launcher, NAMS still loads them.",
            report.hidden_mod_folders.len()
        ));
    }
}

fn third_party_findings(report: &DiagnosticsReport, findings: &mut Findings) {
    let migoto = &report.third_party.migoto;
    if migoto.installed
        && migoto
            .migoto_info
            .as_ref()
            .is_some_and(|info| !info.loader_target_ok)
    {
        findings.warning("3DMigoto d3dx.ini [Loader] target is not NAMS.exe.");
    }
    let reshade = &report.third_party.reshade;
    if reshade.installed
        && reshade
            .reshade_info
            .as_ref()
            .is_some_and(|info| info.d3d_compiler_missing)
    {
        findings
            .warning("ReShade: d3dcompiler_47.dll missing, shaders will not compile under Wine.");
    }
}

pub fn build_findings(report: &DiagnosticsReport) -> Vec<DiagnosticFinding> {
    let mut findings = Findings { out: Vec::new() };
    if report.has_game() {
        game_findings(report, &mut findings);
    } else {
        findings.warning("No game folder selected.");
    }
    runtime_findings(report, &mut findings);
    config_findings(report, &mut findings);
    host_findings(report, &mut findings);
    session_findings(report, &mut findings);
    game_folder_findings(report, &mut findings);
    mod_findings(report, &mut findings);
    third_party_findings(report, &mut findings);
    for (section, error) in &report.collection_errors {
        findings.warning(format!("Could not collect {section}: {error}"));
    }
    findings.out.sort_by_key(|finding| finding.level);
    findings.out
}
