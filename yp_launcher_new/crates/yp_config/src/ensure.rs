use std::path::PathBuf;

use yp_core::{GamePaths, HostEnv, IoOp, IoResultExt};

use crate::error::Result;
use crate::files::ConfigFile;
use crate::migrate;
use crate::templates;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnsureReport {
    pub created: Vec<ConfigFile>,
    pub migrated: Vec<ConfigFile>,
    pub overlay_settings_moved_from: Option<PathBuf>,
}

impl EnsureReport {
    pub fn changed(&self) -> bool {
        !self.created.is_empty()
            || !self.migrated.is_empty()
            || self.overlay_settings_moved_from.is_some()
    }
}

fn write_if_missing(game: &GamePaths, file: ConfigFile, report: &mut EnsureReport) -> Result<()> {
    let path = file.path(game);
    if !path.exists() {
        yp_core::fs::write_text_atomic(&path, templates::template(file))?;
        report.created.push(file);
    }
    Ok(())
}

fn rewrite(
    game: &GamePaths,
    file: ConfigFile,
    text: &str,
    report: &mut EnsureReport,
) -> Result<()> {
    yp_core::fs::write_text_atomic(&file.path(game), text)?;
    if !report.created.contains(&file) && !report.migrated.contains(&file) {
        report.migrated.push(file);
    }
    Ok(())
}

pub fn ensure_configs(game: &GamePaths, host: &HostEnv) -> Result<EnsureReport> {
    let mut report = EnsureReport::default();
    let nams_dir = game.nams_dir();
    std::fs::create_dir_all(&nams_dir).io(IoOp::Create, &nams_dir)?;

    write_if_missing(game, ConfigFile::Nams, &mut report)?;
    write_if_missing(game, ConfigFile::LodMod, &mut report)?;
    write_if_missing(game, ConfigFile::TextureInjection, &mut report)?;

    let nams_path = ConfigFile::Nams.path(game);
    let mut nams_raw = yp_core::fs::read_text_opt(&nams_path)?;
    let mut evade_key_name = None;
    if let Some(raw) = &nams_raw {
        let migration = migrate::migrate_nams(raw);
        evade_key_name = migration.evade_key_name;
        if let Some(text) = migration.text {
            rewrite(game, ConfigFile::Nams, &text, &mut report)?;
            nams_raw = Some(text);
        }
    }

    let naiom_path = ConfigFile::Naiom.path(game);
    if !naiom_path.exists() {
        let text = migrate::initial_naiom(nams_raw.as_deref(), evade_key_name);
        yp_core::fs::write_text_atomic(&naiom_path, &text)?;
        report.created.push(ConfigFile::Naiom);
    }

    let lodmod_path = ConfigFile::LodMod.path(game);
    let lodmod_raw = std::fs::read_to_string(&lodmod_path).io(IoOp::Read, &lodmod_path)?;
    let migrated_lodmod = migrate::migrate_lodmod(&lodmod_raw);
    if migrated_lodmod != lodmod_raw {
        rewrite(game, ConfigFile::LodMod, &migrated_lodmod, &mut report)?;
    }

    let texture_path = ConfigFile::TextureInjection.path(game);
    if let Some(raw) = yp_core::fs::read_text_opt(&texture_path)?
        && let Some(text) = migrate::migrate_texture_injection(&raw)
    {
        rewrite(game, ConfigFile::TextureInjection, &text, &mut report)?;
    }

    report.overlay_settings_moved_from = migrate::migrate_overlay_settings(game, host)?;
    Ok(report)
}
