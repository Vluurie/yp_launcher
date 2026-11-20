use std::path::PathBuf;

use indexmap::IndexMap;
use yp_core::{AppPaths, IoOp, IoResultExt, Os};
use yp_host::game_probe::KNOWN_BUILDS;
use yp_host::nams_cli::NamsTexturePack;
use yp_host::nams_log::LogEntry;
use yp_mods::{InstalledMod, ModConflictKind, ModKind};
use yp_thirdparty::{GameModEntry, MigotoHunting, MigotoMarking, RuntimeStatus};

use super::findings::pack_disabled_by;
use super::model::{
    ConfigDelta, DiagnosticsReport, GAME_DIR_KEY, LAUNCHER_DIR_KEY, REPORT_FORMAT,
    STEAM_RUNNING_KEY, VANILLA_DROP_CAP, exe_variant_name, wine_source_name,
};
use super::redact::redact;
use super::text::{
    DEFAULT_LIST_CAP, Lines, cap_list, file_line, hex_label, human_size, time_label,
};
use crate::error::Result;

pub const RULE_WIDTH: usize = 72;
pub const OUTPUT_TAIL_LINES: usize = 40;
pub const ROOT_EXTRAS_CAP: usize = 40;
pub const KEY_GROUP_COLLAPSE_OVER: usize = 4;
pub const REPORT_FILE_PREFIX: &str = "diagnostics_";
pub const REPORT_FILE_SUFFIX: &str = ".txt";
const REPORT_FILE_STAMP: &str = "%Y-%m-%dT%H-%M-%S-%6fZ";
const UNKNOWN: &str = "?";

pub fn mod_kind_name(kind: ModKind) -> &'static str {
    match kind {
        ModKind::Native => "native",
        ModKind::Data => "data",
        ModKind::Texture => "texture",
        ModKind::Unknown => "unknown",
    }
}

pub fn conflict_kind_label(kind: ModConflictKind) -> &'static str {
    match kind {
        ModConflictKind::OverlappingDataFile => "overlapping data file",
        ModConflictKind::OutfitIdCollision => "outfit ID collision",
        ModConflictKind::VanillaOutfitId => "vanilla outfit ID",
    }
}

pub fn migoto_hunting_name(hunting: MigotoHunting) -> &'static str {
    match hunting {
        MigotoHunting::Off => "off",
        MigotoHunting::On => "on",
        MigotoHunting::NoMarking => "noMarking",
    }
}

pub fn migoto_marking_name(marking: MigotoMarking) -> &'static str {
    match marking {
        MigotoMarking::Skip => "skip",
        MigotoMarking::Original => "original",
        MigotoMarking::Pink => "pink",
        MigotoMarking::Mono => "mono",
    }
}

fn section(out: &mut Lines, title: impl AsRef<str>) {
    out.blank();
    out.line("=".repeat(RULE_WIDTH));
    out.line(title);
    out.line("=".repeat(RULE_WIDTH));
}

fn suffix(present: bool, text: impl FnOnce() -> String) -> String {
    if present { text() } else { String::new() }
}

fn option_suffix<T>(value: Option<T>, text: impl FnOnce(T) -> String) -> String {
    value.map(text).unwrap_or_default()
}

fn indented(out: &mut Lines, lines: &[String]) {
    for line in lines {
        out.line(format!("  {line}"));
    }
}

fn write_findings(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, format!("Findings ({})", report.findings.len()));
    if report.findings.is_empty() {
        out.line("(none)");
    }
    for finding in &report.findings {
        out.line(format!("{} {}", finding.tag(), finding.message));
    }
}

fn write_system(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, "System");
    for (key, value) in &report.system_info {
        out.line(format!("{key}: {value}"));
    }
    if !report.filesystems.is_empty() {
        out.blank();
        out.line("Filesystems:");
        for (role, mount) in &report.filesystems {
            out.line(match mount {
                None => format!("  {}: unknown", role.label()),
                Some(mount) => format!(
                    "  {}: {} (mount {}{})",
                    role.label(),
                    mount.fs_type,
                    mount.target,
                    suffix(!mount.options.is_empty(), || format!(", {}", mount.options))
                ),
            });
        }
    }

    section(out, "Launcher");
    for (key, value) in &report.launcher_info {
        if key != LAUNCHER_DIR_KEY && key != GAME_DIR_KEY {
            out.line(format!("{key}: {value}"));
        }
    }
}

fn write_game_identity(out: &mut Lines, report: &DiagnosticsReport) {
    let identity = &report.game_identity;
    section(out, "Game identity");
    out.line(format!(
        "build: {} ({})",
        identity.build_display(),
        if identity.exe_variant_supported() {
            "supported"
        } else {
            "UNSUPPORTED"
        }
    ));
    out.line(format!(
        "exe variant: {}",
        exe_variant_name(identity.exe_variant)
    ));
    if let Some(size) = identity.exe_size {
        out.line(format!(
            "exe: {} ({size} bytes)  sha1 {}  PE stamp {}",
            human_size(size),
            identity.exe_sha1.as_deref().unwrap_or(UNKNOWN),
            hex_label(identity.exe_time_date_stamp)
        ));
    }
    let known: Vec<String> = KNOWN_BUILDS
        .iter()
        .map(|build| format!("{} ({} bytes)", build.label, build.exe_size))
        .collect();
    out.line(format!("known builds: {}", known.join(", ")));
    if identity.compat_exe_present {
        out.line("NieRAutomataCompat.exe: present");
    }
    if !identity.build_info.is_empty() {
        let pairs: Vec<String> = identity
            .build_info
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        out.line(format!("Latest_BuildInfo.properties: {}", pairs.join("  ")));
    }
    if let Some(manifest) = &identity.steam_manifest {
        out.line(format!(
            "Steam manifest: build {}{}  state {}  updated {}{}",
            manifest.build_id.as_deref().unwrap_or(UNKNOWN),
            suffix(manifest.update_pending(), || format!(
                " (update pending -> {})",
                manifest.target_build_id.as_deref().unwrap_or(UNKNOWN)
            )),
            manifest.state_flags.as_deref().unwrap_or(UNKNOWN),
            time_label(manifest.last_updated_at()),
            option_suffix(manifest.beta_key.as_deref(), |beta| format!(
                "  beta {beta}"
            ))
        ));
        let joined = |ids: &std::collections::BTreeSet<String>| {
            ids.iter().cloned().collect::<Vec<_>>().join(", ")
        };
        out.line(format!(
            "Steam DLC: installed [{}]  disabled [{}]",
            joined(&manifest.installed_dlc),
            joined(&manifest.disabled_dlc)
        ));
    } else if report.has_game() {
        out.line("Steam manifest: not found (game outside steamapps/common?)");
    }
    out.line(format!(
        "DLC (3C3C Concert & Costume): {} ({}){}",
        identity.has_dlc,
        identity.dlc_source.label(),
        option_suffix(identity.data100_size, |size| format!(
            "  data100.cpk {size} bytes"
        ))
    ));
    match &identity.game_bin {
        Some(game_bin) => out.line(format!(
            "runtime/game.bin: {}  PE stamp {} ({}, {})",
            file_line(game_bin),
            hex_label(identity.game_bin_time_date_stamp),
            identity.game_bin_build_label().unwrap_or("unknown"),
            if identity.game_bin_matches_exe() {
                "matches exe"
            } else {
                "rebuilt on next launch"
            }
        )),
        None => out.line("runtime/game.bin: not built yet"),
    }
}

fn write_nams_health(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, "NAMS runtime health");
    let health = &report.nams_health;
    out.line(format!(
        "NAMS.exe: {}",
        health
            .nams_exe
            .as_ref()
            .map_or_else(|| "missing".to_owned(), file_line)
    ));
    let version_fallback = if report.host.os == Os::Windows {
        "unavailable"
    } else {
        "not queried (Windows only)"
    };
    out.line(format!(
        "NAMS version: {}",
        health.nams_version.as_deref().unwrap_or(version_fallback)
    ));
    if let Some(error) = &health.nier_install_error {
        out.line(format!("NAMS install lookup: {error}"));
    }
    out.line(format!("Plugins ({}):", health.plugins.len()));
    for plugin in &health.plugins {
        out.line(format!("  - {}", file_line(plugin)));
    }
    out.line(format!("Game running: {}", report.game_running));
    out.line(format!(
        "Steam running: {}",
        report.system_value(STEAM_RUNNING_KEY).unwrap_or(UNKNOWN)
    ));
    if health.missing_files.is_empty() {
        out.line("Missing runtime files: none");
    } else {
        out.line(format!(
            "!! Missing runtime files (AV/quarantine?): {}",
            health.missing_files.join(", ")
        ));
    }
    if let Some(preview) = &health.launch_command_preview {
        out.blank();
        out.line("Launch command:");
        out.line(preview);
    }
}

fn write_compat_layer(out: &mut Lines, report: &DiagnosticsReport) {
    if report.host.os == Os::Windows {
        return;
    }
    section(out, "Compatibility layer");
    let Some(compat) = &report.compat_layer else {
        out.line(if report.has_game() {
            "(none found)"
        } else {
            "(no game folder)"
        });
        return;
    };
    out.line(format!(
        "source: {}  label: {}",
        wine_source_name(compat.source),
        compat.label
    ));
    out.line(format!("binary: {}", compat.binary));
    if let Some(version) = &compat.tool_version {
        out.line(format!("tool version: {version}"));
    }
    out.line(format!(
        "prefix: {} ({})",
        compat.prefix.as_deref().unwrap_or(UNKNOWN),
        if compat.prefix_exists {
            "exists"
        } else {
            "not created yet"
        }
    ));
    if let Some(version) = &compat.prefix_version {
        out.line(format!("prefix created by: {version}"));
    }
}

fn write_last_launch(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, "Last launch");
    let Some(last) = &report.last_launch else {
        out.line("(no launch recorded)");
        return;
    };
    out.line(format!(
        "started: {}  via: {}",
        time_label(Some(last.started_at)),
        last.label
    ));
    if last.still_running() {
        out.line("ended: not recorded (still running, or the launcher closed first)");
    } else {
        out.line(format!(
            "ended: {}  exit code: {}  duration: {}s",
            time_label(last.ended_at),
            last.exit_code
                .map_or_else(|| UNKNOWN.to_owned(), |code| code.to_string()),
            last.duration().map_or_else(
                || UNKNOWN.to_owned(),
                |duration| duration.as_secs().to_string()
            )
        ));
    }
    let tail = last.output_tail.trim();
    if !tail.is_empty() {
        out.line("output tail:");
        let lines: Vec<&str> = tail.split('\n').collect();
        for line in lines
            .iter()
            .skip(lines.len().saturating_sub(OUTPUT_TAIL_LINES))
        {
            out.line(format!("  {}", line.trim_end()));
        }
    }
}

fn write_logs(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, "Logs");
    let logs = &report.logs;
    match &logs.nams_log {
        None => out.line("nams.log: missing"),
        Some(log) => {
            out.line(format!(
                "nams.log: {}  modified {}  (last {} lines)",
                human_size(log.size),
                time_label(log.modified),
                log.lines.len()
            ));
            indented(out, &log.lines);
        }
    }
    out.blank();
    out.line(format!("crash dumps: {}", logs.crash_dump_count));
    if let Some(dump) = &logs.newest_crash_dump {
        out.line(format!(
            "newest: {}  modified {}",
            dump.name,
            time_label(dump.modified)
        ));
        indented(out, &dump.lines);
    }
    if let Some(heartbeat) = &logs.heartbeat {
        out.blank();
        out.line(format!(
            "heartbeat.log: modified {}",
            time_label(heartbeat.modified)
        ));
        indented(out, &heartbeat.lines);
    }
    if !logs.captured_launch_logs.is_empty() {
        out.blank();
        out.line("captured early-exit logs:");
        for file in &logs.captured_launch_logs {
            out.line(format!("  - {}", file_line(file)));
        }
    }
}

fn write_recent_issues(out: &mut Lines, issues: &[LogEntry]) {
    section(
        out,
        format!("Recent NAMS warnings/errors ({})", issues.len()),
    );
    if issues.is_empty() {
        out.line("(none in nams.log)");
        return;
    }
    let mut grouped: IndexMap<String, (&LogEntry, usize)> = IndexMap::new();
    for entry in issues {
        let key = format!("{} {}: {}", entry.level, entry.module, entry.message);
        let seen = grouped.shift_remove(&key).map_or(0, |(_, count)| count);
        grouped.insert(key, (entry, seen + 1));
    }
    for (key, (last, count)) in &grouped {
        out.line(format!(
            "  {} {key}{}",
            last.timestamp,
            suffix(*count > 1, || format!("  (x{count})"))
        ));
    }
}

fn write_vanilla_drops(out: &mut Lines, report: &DiagnosticsReport) {
    section(
        out,
        format!(
            "Vanilla data/ drops (user-added files, {})",
            report.vanilla_dropped.len()
        ),
    );
    if report.vanilla_dropped.is_empty() {
        out.line("(none, vanilla data/ is clean)");
    } else {
        for drop in &report.vanilla_dropped {
            out.line(format!(
                "  {}  {}  [{}]",
                drop.path,
                human_size(drop.size_bytes),
                drop.bucket.label()
            ));
        }
        if report.vanilla_dropped.len() >= VANILLA_DROP_CAP {
            out.line(format!("  ... (capped at {VANILLA_DROP_CAP})"));
        }
    }
    if !report.data_dir_contents.is_empty() {
        out.blank();
        out.line("Non-vanilla folders in data/:");
        let mut folders: Vec<(&String, &usize)> = report.data_dir_contents.iter().collect();
        folders.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (name, count) in folders {
            out.line(format!("  {name}/  ({count} files)"));
        }
    }
}

fn write_reshade(out: &mut Lines, status: &RuntimeStatus) {
    section(out, "ReShade");
    if !status.installed {
        out.line("(not installed)");
        return;
    }
    out.line(format!(
        "installed: true   enabled: {}   shadersMissing: {}",
        status.enabled, status.shaders_missing
    ));
    let Some(info) = &status.reshade_info else {
        return;
    };
    if let Some(version) = &info.version {
        out.line(format!("version: {version}"));
    }
    if let Some(dll) = &info.dll_name {
        out.line(format!("dll: {dll}   addonBuild: {}", info.is_addon_build));
    }
    out.line(format!(
        "presets ({}): {}",
        info.presets.len(),
        info.presets.join(", ")
    ));
    out.line(format!(
        "shader repos ({}): {}   effects: {}",
        info.shader_repos.len(),
        info.shader_repos.join(", "),
        info.shader_count
    ));
    if !info.addons.is_empty() {
        out.line(format!(
            "addons ({}): {}",
            info.addons.len(),
            info.addons.join(", ")
        ));
    }
    if info.d3d_compiler_missing {
        out.line("!! d3dcompiler_47.dll missing (Wine shaders will not compile)");
    }
    let config = &info.config;
    out.line(format!(
        "config: performanceMode={} showFps={} showClock={}{}",
        config.performance_mode,
        config.show_fps,
        config.show_clock,
        option_suffix(config.active_preset.as_deref(), |preset| format!(
            " activePreset={preset}"
        ))
    ));
}

fn write_migoto(out: &mut Lines, status: &RuntimeStatus) {
    section(out, "3DMigoto");
    if !status.installed {
        out.line("(not installed)");
        return;
    }
    out.line(format!(
        "installed: true   enabled: {}   hasShaderFixes: {}",
        status.enabled, status.has_shader_fixes
    ));
    let Some(info) = &status.migoto_info else {
        return;
    };
    out.line(format!("files: {}", info.files.join(", ")));
    out.line(format!(
        "loader target: {} (ok: {})",
        info.loader_target.as_deref().unwrap_or(UNKNOWN),
        info.loader_target_ok
    ));
    out.line(format!("shader fixes: {}", info.shader_fix_count));
    if !info.shader_fix_names.is_empty() {
        out.line(format!("  from: {}", info.shader_fix_names.join(", ")));
    }
    let config = &info.config;
    out.line(format!(
        "config: hunting={} marking={} cacheShaders={} verboseOverlay={}",
        migoto_hunting_name(config.hunting),
        migoto_marking_name(config.marking_mode),
        config.cache_shaders,
        config.verbose_overlay
    ));
}

fn dll_line(entry: &GameModEntry) -> String {
    format!(
        "  - {}  {}{}",
        entry.key(),
        entry
            .size_bytes
            .map(yp_core::fs::size_label)
            .unwrap_or_default(),
        suffix(entry.disabled, || "  [disabled]".to_owned())
    )
}

fn loading_line(dir: &str, status: &RuntimeStatus) -> String {
    let state = if !status.installed {
        "(none)"
    } else if status.enabled {
        "loading enabled"
    } else {
        "loading DISABLED"
    };
    format!("nams/thirdparty/{dir}: {state}")
}

fn write_dll_mods(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, "Third-party game & graphic DLLs");
    let game_mods = &report.third_party.game_mods;
    out.line(loading_line("game", game_mods));
    for entry in game_mods
        .game_mods_info
        .iter()
        .flat_map(|info| info.mods.iter())
    {
        let settings = option_suffix(entry.settings, |settings| {
            format!(
                "  module={}  exePath={}  sizeOfImage={}{}{}",
                if settings.fake_module_handle {
                    "game.bin"
                } else {
                    "NAMS.exe"
                },
                if settings.fake_exe_path {
                    "NieRAutomata.exe"
                } else {
                    "NAMS.exe"
                },
                settings.size_of_image,
                suffix(settings.report_patches, || "  reportPatches".to_owned()),
                suffix(settings.wait_for_patch_phase, || format!(
                    "  waitForPatchPhase={}ms",
                    settings.patch_phase_timeout_ms
                ))
            )
        });
        out.line(format!("{}{settings}", dll_line(entry)));
    }
    let graphic_mods = &report.third_party.graphic_mods;
    out.line(loading_line("graphic", graphic_mods));
    for entry in graphic_mods
        .graphic_mods_info
        .iter()
        .flat_map(|info| info.mods.iter())
    {
        out.line(dll_line(entry));
    }
}

fn texture_pack_line(report: &DiagnosticsReport, pack: &NamsTexturePack) -> String {
    let state = match pack_disabled_by(report, pack) {
        None => "enabled".to_owned(),
        Some(switch) => format!("disabled ({})", switch.file_name()),
    };
    format!(
        "- {}  [{}]  {state}  {} dds{}{}{}{}",
        pack.name,
        pack.source,
        pack.dds_count,
        option_suffix(pack.mod_id.as_deref(), |id| format!("  mod={id}")),
        option_suffix(pack.character.as_deref(), |character| format!(
            "  char={character}"
        )),
        suffix(pack.outfit_conditional, || "  outfit-conditional"
            .to_owned()),
        option_suffix(pack.load_order_index, |index| format!("  order={index}"))
    )
}

fn write_texture_packs(out: &mut Lines, report: &DiagnosticsReport) {
    let listed = if report.texture_packs_available {
        report.texture_packs.len()
    } else {
        report.nams_textures.len()
    };
    section(out, format!("Texture packs ({listed})"));
    if !report.texture_packs_available {
        out.line(format!(
            "NAMS query: {}",
            report.texture_packs_note.unwrap_or("unavailable")
        ));
    } else if report.texture_packs.is_empty() {
        out.line("(none)");
    } else {
        for pack in &report.texture_packs {
            out.line(texture_pack_line(report, pack));
        }
    }
    out.blank();
    out.line(format!(
        "nams/inject/textures/ ({}): {}",
        report.nams_textures.len(),
        cap_list(&report.nams_textures, DEFAULT_LIST_CAP)
    ));
    if !report.sk_res_textures.is_empty() {
        out.line(format!(
            "SK_Res/inject/textures/ ({}): {}",
            report.sk_res_textures.len(),
            cap_list(&report.sk_res_textures, DEFAULT_LIST_CAP)
        ));
    }
    if !report.sk_res_packs.is_empty() {
        out.line(format!(
            "SK_Res packs (all exe folders): {}",
            cap_list(&report.sk_res_packs, DEFAULT_LIST_CAP)
        ));
    }
    if !report.wax_textures.is_empty() {
        out.line(format!(
            "WAX textures ({}): {}",
            report.wax_textures.len(),
            cap_list(&report.wax_textures, DEFAULT_LIST_CAP)
        ));
    }
}

fn key_section(key: &str) -> Option<&str> {
    key.split_once('.').map(|(section, _)| section)
}

fn write_key_groups(out: &mut Lines, keys: &[ConfigDelta]) {
    let mut groups: IndexMap<(&str, &str, &str), Vec<&ConfigDelta>> = IndexMap::new();
    for delta in keys {
        let group = (
            delta.file.file_name(),
            key_section(&delta.key).unwrap_or_default(),
            delta.note.unwrap_or_default(),
        );
        groups.entry(group).or_default().push(delta);
    }
    for ((file, _, note), group) in &groups {
        let note = suffix(!note.is_empty(), || format!("  ({note})"));
        let collapsed = group
            .first()
            .and_then(|first| key_section(&first.key))
            .filter(|_| group.len() > KEY_GROUP_COLLAPSE_OVER);
        if let Some(section) = collapsed {
            out.line(format!("  [{file}] [{section}] {} keys{note}", group.len()));
            continue;
        }
        for delta in group {
            out.line(format!("  [{file}] {} = {}{note}", delta.key, delta.value));
        }
    }
}

fn write_settings(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, "Settings");
    let config = &report.config;
    if !config.missing_files.is_empty() {
        let names: Vec<&str> = config
            .missing_files
            .iter()
            .map(|file| file.file_name())
            .collect();
        out.line(format!("missing or empty: {}", names.join(", ")));
    }
    for (file, error) in &config.parse_errors {
        out.line(format!("!! {} does not parse: {error}", file.file_name()));
    }
    out.blank();
    out.line(format!("Non-default ({}):", config.deltas.len()));
    if config.deltas.is_empty() {
        out.line("  (all defaults)");
    }
    for delta in &config.deltas {
        out.line(format!(
            "  [{}] {} = {}  (default {})",
            delta.file.file_name(),
            delta.key,
            delta.value,
            delta.default_value
        ));
    }
    if !config.unknown_keys.is_empty() {
        out.blank();
        out.line(format!(
            "Unknown keys, NAMS does not read them ({}):",
            config.unknown_keys.len()
        ));
        write_key_groups(out, &config.unknown_keys);
    }
    if !config.legacy_keys.is_empty() {
        out.blank();
        out.line(format!(
            "Old keys left in the files ({}):",
            config.legacy_keys.len()
        ));
        write_key_groups(out, &config.legacy_keys);
    }

    section(out, "Profiles");
    out.line(format!(
        "active profile: {}",
        report.active_profile.as_deref().unwrap_or("default")
    ));
    if !report.default_mods_entries.is_empty() {
        out.line("default_mods.toml:");
        indented(out, &report.default_mods_entries);
    }
}

fn disabled_by_prefix(report: &DiagnosticsReport, installed: &InstalledMod) -> bool {
    yp_mods::disabled::matches(
        &report.disabled_mods_entries,
        &format!("mods/{}", installed.id),
    )
}

fn write_mod_details(out: &mut Lines, installed: &InstalledMod) {
    if !installed.display_name.is_empty() && installed.display_name != installed.id {
        out.line(format!("    name:     {}", installed.display_name));
    }
    out.line(format!(
        "    installed: {}",
        time_label(Some(installed.installed_at))
    ));
    if let Some(manifest) = &installed.manifest {
        if let Some(author) = &manifest.author {
            out.line(format!("    author:   {author}"));
        }
        if let Some(version) = &manifest.nams_min_version {
            out.line(format!("    NAMS min version: {version}"));
        }
        if !manifest.requires.is_empty() {
            out.line(format!(
                "    requires mods:    {}",
                manifest.requires.join(", ")
            ));
        }
        if !manifest.requires_plugins.is_empty() {
            out.line(format!(
                "    requires plugins: {}",
                manifest.requires_plugins.join(", ")
            ));
        }
        if !manifest.replaces.is_empty() {
            out.line(format!(
                "    replaces:         {}",
                manifest.replaces.join(", ")
            ));
        }
    }
    if !installed.requires_missing.is_empty() {
        out.line(format!(
            "    !! missing requires: {}",
            installed.requires_missing.join(", ")
        ));
    }
    if let Some(native) = &installed.native
        && (!native.bundles_by_kind.is_empty() || native.total_entity_files > 0)
    {
        let bundles: Vec<String> = native
            .bundles_by_kind
            .iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect();
        out.line(format!(
            "    native:   bundles=[{}]  entityFiles={}",
            bundles.join(", "),
            native.total_entity_files
        ));
    }
    if let Some(data) = &installed.data {
        if !data.entries.is_empty() {
            let mut names: Vec<&str> = data
                .entries
                .iter()
                .map(|entry| entry.dir_name.as_str())
                .collect();
            names.sort_unstable();
            out.line(format!("    data dirs: {}", names.join(", ")));
        }
        if !data.players.is_empty() {
            out.line("    player models:");
            for player in &data.players {
                let stem = player.file_name.split('.').next().unwrap_or_default();
                let label =
                    yp_mods::tables::player_model_label(stem).unwrap_or(player.label.as_str());
                out.line(format!("      - {}  ->  {label}", player.file_name));
            }
        }
    }
    if !installed.conflicts.is_empty() {
        out.line(format!("    conflicts ({}):", installed.conflicts.len()));
        for conflict in &installed.conflicts {
            out.line(format!(
                "      - vs {} [{}] {}",
                conflict.other_mod_id,
                conflict_kind_label(conflict.kind),
                conflict.detail
            ));
        }
    }
}

fn write_mods(out: &mut Lines, report: &DiagnosticsReport) {
    section(out, format!("Installed mods ({})", report.mods.len()));
    if !report.mods.is_empty() {
        let mut by_kind: IndexMap<ModKind, usize> = IndexMap::new();
        for installed in &report.mods {
            *by_kind.entry(installed.kind).or_default() += 1;
        }
        let totals: Vec<String> = by_kind
            .iter()
            .map(|(kind, count)| format!("{}={count}", mod_kind_name(*kind)))
            .collect();
        let with_warnings = report
            .mods
            .iter()
            .filter(|installed| installed.has_warnings())
            .count();
        let with_bundles = report
            .mods
            .iter()
            .filter(|installed| {
                !installed.bundled_texture_packs.is_empty()
                    || !installed.bundled_cutscenes.is_empty()
            })
            .count();
        let disabled = report
            .mods
            .iter()
            .filter(|installed| disabled_by_prefix(report, installed))
            .count();
        out.line(format!("Totals by kind: {}", totals.join(", ")));
        out.line(format!(
            "With warnings: {with_warnings}   With bundled assets: {with_bundles}   Disabled prefixes \
             affecting mods: {disabled}"
        ));
        out.blank();
    }
    if !report.hidden_mod_folders.is_empty() {
        out.line(format!(
            "Hidden \"_\" folders (NAMS still loads them): {}",
            report.hidden_mod_folders.join(", ")
        ));
        out.blank();
    }
    for installed in &report.mods {
        out.line(format!(
            "- {}  {}  [{}]{}{}",
            installed.id,
            if disabled_by_prefix(report, installed) {
                "[DISABLED]"
            } else {
                "[enabled]"
            },
            mod_kind_name(installed.kind),
            option_suffix(
                installed
                    .manifest
                    .as_ref()
                    .and_then(|manifest| manifest.version.as_deref()),
                |version| format!("  v{version}")
            ),
            suffix(installed.has_warnings(), || "  !! warnings".to_owned())
        ));
        if installed.has_warnings() {
            write_mod_details(out, installed);
        }
    }

    section(
        out,
        format!(
            "Disabled mod prefixes ({})",
            report.disabled_mods_entries.len()
        ),
    );
    for entry in &report.disabled_mods_entries {
        out.line(format!("- {entry}"));
    }
}

fn write_cutscenes(out: &mut Lines, report: &DiagnosticsReport) {
    section(
        out,
        format!("Cutscene mods ({})", report.cutscene_mods.len()),
    );
    for cutscene in &report.cutscene_mods {
        let origin = match &cutscene.bundled_with_mod_id {
            None => "standalone".to_owned(),
            Some(mod_id) => format!("bundled with {mod_id}"),
        };
        out.line(format!(
            "- {}  [{origin}]  {} USM{}{}",
            cutscene.name,
            cutscene.usm_count(),
            if cutscene.has_h264() {
                "  H264"
            } else {
                "  MPEG-2"
            },
            suffix(cutscene.max_width() > 0, || format!(
                "  {}x{}",
                cutscene.max_width(),
                cutscene.max_height()
            ))
        ));
    }
    if !report.direct_overrides.is_empty() {
        out.blank();
        out.line(format!(
            "Custom files in data/movie/ ({}): {}",
            report.direct_overrides.len(),
            cap_list(&report.direct_overrides, DEFAULT_LIST_CAP)
        ));
    }
    out.blank();
    let detection = &report.cutscene_detection;
    out.line(format!(
        "cutscene detection: hasHd={}, needsH264={}, filesScanned={}, largest={}x{}",
        detection.has_hd_cutscenes,
        detection.needs_h264,
        detection.files_scanned,
        detection.largest_width,
        detection.largest_height
    ));
}

fn write_other_folders(out: &mut Lines, report: &DiagnosticsReport) {
    let listed = |items: &[String], empty: &str, cap: usize| {
        if items.is_empty() {
            empty.to_owned()
        } else {
            cap_list(items, cap)
        }
    };
    section(out, "Other folders");
    out.line(format!(
        "wax/mods/: {}",
        listed(&report.wax_mods_root, "(none)", DEFAULT_LIST_CAP)
    ));
    out.line(format!(
        "reshade/: {}",
        listed(&report.reshade_contents, "(none)", DEFAULT_LIST_CAP)
    ));
    out.line(format!(
        "game root extras: {}",
        listed(
            &report.game_root_extras,
            "(none, clean install root)",
            ROOT_EXTRAS_CAP
        )
    ));
}

fn write_unparsed_configs(out: &mut Lines, report: &DiagnosticsReport) {
    for file in report.config.parse_errors.keys() {
        section(out, format!("nams/{} (does not parse)", file.file_name()));
        out.line(
            report
                .config_raw
                .get(file)
                .map(String::as_str)
                .unwrap_or_default(),
        );
    }
}

pub fn format_full_report(report: &DiagnosticsReport) -> String {
    let mut out = Lines::default();
    out.line("YP Launcher Diagnostics Report");
    out.line(format!("Format: {REPORT_FORMAT}"));
    out.line(format!(
        "Generated: {}",
        time_label(Some(report.generated_at))
    ));
    write_findings(&mut out, report);
    write_system(&mut out, report);
    write_game_identity(&mut out, report);
    write_nams_health(&mut out, report);
    write_compat_layer(&mut out, report);
    write_last_launch(&mut out, report);
    write_logs(&mut out, report);
    write_recent_issues(&mut out, &report.recent_log_issues);
    write_vanilla_drops(&mut out, report);
    write_reshade(&mut out, &report.third_party.reshade);
    write_migoto(&mut out, &report.third_party.migoto);
    write_dll_mods(&mut out, report);
    write_texture_packs(&mut out, report);
    write_settings(&mut out, report);
    write_mods(&mut out, report);
    write_cutscenes(&mut out, report);
    write_other_folders(&mut out, report);
    write_unparsed_configs(&mut out, report);
    redact(&out.finish(), report)
}

pub fn report_file_name(report: &DiagnosticsReport) -> String {
    format!(
        "{REPORT_FILE_PREFIX}{}{REPORT_FILE_SUFFIX}",
        report.generated_at.strftime(REPORT_FILE_STAMP)
    )
}

pub fn write_full_report(report: &DiagnosticsReport, paths: &AppPaths) -> Result<PathBuf> {
    let dir = &paths.logs_dir;
    std::fs::create_dir_all(dir).io(IoOp::Create, dir)?;
    let file = dir.join(report_file_name(report));
    yp_core::fs::write_text_atomic(&file, &format_full_report(report))?;
    Ok(file)
}
