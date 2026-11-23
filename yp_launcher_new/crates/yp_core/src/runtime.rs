use std::path::{Path, PathBuf};

use crate::consts;
use crate::error::{IoOp, IoResultExt, Result};
use crate::host::HostEnv;
use crate::paths::AppPaths;

pub const STALE_TEMP_PREFIXES: [&str; 2] = ["archive_", "texture_"];
pub const ASSET_KEY_PREFIX: &str = "assets/bins/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeStatus {
    Bundled,
    UpToDate,
    Extracted { files: usize },
    NoBundledAssets,
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeSetup<'a> {
    paths: &'a AppPaths,
    host: &'a HostEnv,
}

impl<'a> RuntimeSetup<'a> {
    pub fn new(paths: &'a AppPaths, host: &'a HostEnv) -> Self {
        Self { paths, host }
    }

    pub fn paths(&self) -> &AppPaths {
        self.paths
    }

    pub fn ensure_ready(&self) -> Result<RuntimeStatus> {
        let status = self.extract()?;
        crate::fs::sweep_stale_temp(&[self.host.temp_dir().to_path_buf()], &STALE_TEMP_PREFIXES);
        Ok(status)
    }

    fn extract(&self) -> Result<RuntimeStatus> {
        if !self.paths.needs_runtime_extraction() {
            return Ok(RuntimeStatus::Bundled);
        }
        let assets = bundled_assets(&self.paths.bundled_bins_dir)?;
        if assets.is_empty() {
            return Ok(RuntimeStatus::NoBundledAssets);
        }
        let expected = stamp_for(&assets);
        let stamp_file = self.paths.runtime_stamp();
        if crate::fs::read_text_opt(&stamp_file).ok().flatten().as_deref() == Some(expected.as_str()) {
            return Ok(RuntimeStatus::UpToDate);
        }
        for relative in &assets {
            let source = join_relative(&self.paths.bundled_bins_dir, relative);
            let dest = join_relative(&self.paths.runtime_dir, relative);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).io(IoOp::Create, parent)?;
            }
            std::fs::copy(&source, &dest).io(IoOp::Copy, &dest)?;
        }
        crate::fs::write_text_atomic(&stamp_file, &expected)?;
        Ok(RuntimeStatus::Extracted { files: assets.len() })
    }

    pub fn missing_files(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.paths.nams_exe().is_file() {
            missing.push(consts::NAMS_EXE_NAME);
        }
        if !self.paths.yorha_dll().is_file() {
            missing.push(consts::YORHA_DLL_NAME);
        }
        missing
    }

    pub fn is_writable(&self) -> bool {
        crate::fs::is_dir_writable(&self.paths.runtime_dir)
    }
}

pub fn bundled_assets(bins_dir: &Path) -> Result<Vec<String>> {
    if !bins_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut assets = Vec::new();
    for entry in walkdir::WalkDir::new(bins_dir).follow_links(false) {
        let entry = entry.map_err(|err| {
            let path = err.path().unwrap_or(bins_dir).to_path_buf();
            crate::error::Error::Io {
                op: IoOp::List,
                path,
                source: err.into_io_error().unwrap_or_else(|| std::io::Error::other("walk failed")),
            }
        })?;
        if !entry.file_type().is_file() || entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if let Some(relative) = crate::fs::rel_slash(bins_dir, entry.path())
            && relative != consts::NAMS_DLL_NAME
        {
            assets.push(relative);
        }
    }
    assets.sort();
    Ok(assets)
}

pub fn stamp_for(assets: &[String]) -> String {
    let keys: Vec<String> = assets.iter().map(|asset| format!("{ASSET_KEY_PREFIX}{asset}")).collect();
    let digest = crate::hash::sha256_bytes(keys.join("\n").as_bytes());
    format!("{}|{}", consts::APP_VERSION, crate::hash::to_hex(&digest))
}

fn join_relative(root: &Path, relative: &str) -> PathBuf {
    relative.split('/').fold(root.to_path_buf(), |path, part| path.join(part))
}
