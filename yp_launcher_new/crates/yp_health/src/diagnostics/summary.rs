use crate::probes::{APPIMAGE_KEY, DISTRO_KEY, FLATPAK_KEY, SESSION_KEY};

use super::findings::pack_disabled_by;
use super::model::{
    DISCORD_LIMIT, DiagnosticsReport, LAUNCHER_VERSION_KEY, OS_KEY, OS_VERSION_KEY, REPORT_FORMAT,
    STEAM_RUNNING_KEY,
};
use super::redact::redact;
use super::text::{Lines, time_label, truncated};

pub const TRUNCATION_NOTICE: &str = "\n... (see the full report)";
const UNKNOWN: &str = "?";

fn third_party_line(report: &DiagnosticsReport) -> Option<String> {
    let third_party = &report.third_party;
    let off = |enabled: bool| if enabled { "" } else { " (off)" };
    let mut parts = Vec::new();
    if third_party.reshade.installed {
        parts.push(format!("ReShade{}", off(third_party.reshade.enabled)));
    }
    if third_party.migoto.installed {
        parts.push(format!("3DMigoto{}", off(third_party.migoto.enabled)));
    }
    if third_party.game_mods.installed {
        let enabled = third_party
            .game_mods
            .game_mods_info
            .as_ref()
            .map_or(0, |info| info.enabled_count());
        parts.push(format!("game DLLs {enabled}"));
    }
    if third_party.graphic_mods.installed {
        let enabled = third_party
            .graphic_mods
            .graphic_mods_info
            .as_ref()
            .map_or(0, |info| info.enabled_count());
        parts.push(format!("graphic DLLs {enabled}"));
    }
    (!parts.is_empty()).then(|| format!("Third-party: {}", parts.join(", ")))
}

fn texture_pack_count(report: &DiagnosticsReport) -> String {
    if !report.texture_packs_available {
        return report.nams_textures.len().to_string();
    }
    let enabled = report
        .texture_packs
        .iter()
        .filter(|pack| pack_disabled_by(report, pack).is_none())
        .count();
    format!("{enabled}/{} on", report.texture_packs.len())
}

pub fn format_summary(report: &DiagnosticsReport) -> String {
    let system = |key: &str| report.system_value(key).unwrap_or(UNKNOWN);
    let mut out = Lines::default();
    out.line(format!("YP Launcher Diagnostics (format {REPORT_FORMAT})"));
    out.line(format!(
        "Generated: {}",
        time_label(Some(report.generated_at))
    ));
    out.line(format!(
        "Launcher: {}   OS: {} {}",
        report
            .launcher_value(LAUNCHER_VERSION_KEY)
            .unwrap_or(UNKNOWN),
        system(OS_KEY),
        system(OS_VERSION_KEY)
    ));
    if let Some(distro) = report.system_value(DISTRO_KEY) {
        out.line(format!(
            "Distro: {distro}   Session: {}   AppImage: {}   Flatpak: {}",
            system(SESSION_KEY),
            system(APPIMAGE_KEY),
            system(FLATPAK_KEY)
        ));
    }
    let identity = &report.game_identity;
    let steam_build = identity
        .steam_manifest
        .as_ref()
        .and_then(|manifest| manifest.build_id.as_deref())
        .map(|build| format!("   Steam build {build}"))
        .unwrap_or_default();
    out.line(format!(
        "Game: {}{steam_build}   DLC={}   running={}",
        identity.build_display(),
        identity.has_dlc,
        report.game_running
    ));
    let health = &report.nams_health;
    let version = health
        .nams_version
        .as_deref()
        .map(|version| format!(" v{version}"))
        .unwrap_or_default();
    out.line(format!(
        "NAMS: exe={}{version}   missing={}   Steam running={}",
        health.nams_exe_present,
        health.missing_files.len(),
        system(STEAM_RUNNING_KEY)
    ));
    if let Some(compat) = &report.compat_layer {
        let tool = compat
            .tool_version
            .as_deref()
            .map(|version| format!(" ({version})"))
            .unwrap_or_default();
        out.line(format!("Compat: {}{tool}", compat.label));
    }
    for (role, mount) in &report.filesystems {
        if let Some(mount) = mount {
            out.line(format!("FS {}: {}", role.label(), mount.fs_type));
        }
    }
    if let Some(last) = &report.last_launch {
        let exit = match last.exit_code {
            Some(code) => code.to_string(),
            None if last.still_running() => "running/unknown".to_owned(),
            None => UNKNOWN.to_owned(),
        };
        let duration = last
            .duration()
            .map(|duration| format!("   {}s", duration.as_secs()))
            .unwrap_or_default();
        out.line(format!(
            "Last launch: {}   exit={exit}{duration}",
            time_label(Some(last.started_at))
        ));
    }
    out.line(format!(
        "Mods: {}   disabled prefixes: {}   texture packs: {}   cutscene mods: {}   non-default settings: {}",
        report.mods.len(),
        report.disabled_mods_entries.len(),
        texture_pack_count(report),
        report.cutscene_mods.len(),
        report.config.deltas.len()
    ));
    if let Some(line) = third_party_line(report) {
        out.line(line);
    }

    out.blank();
    if report.findings.is_empty() {
        out.line("Findings: none");
    } else {
        out.line("Findings:");
        for finding in &report.findings {
            out.line(format!("{} {}", finding.tag(), finding.message));
        }
    }

    let text = redact(&out.finish(), report);
    match truncated(&text, DISCORD_LIMIT) {
        Some(_) => {
            let keep = DISCORD_LIMIT - TRUNCATION_NOTICE.chars().count();
            let head = truncated(&text, keep).unwrap_or(&text);
            format!("{head}{TRUNCATION_NOTICE}")
        }
        None => text,
    }
}
