use yp_core::GamePaths;

use crate::dll_id::{self, GraphicsDll};
use crate::error::Result;
use crate::flags::{self, LoadingFlag};

pub const RESHADE_DLL_CANDIDATES: [&str; 5] =
    ["ReShade64.dll", "reshade64.dll", "ReShade64_Addon.dll", "dxgi.dll", "d3d11.dll"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReShadeStatus {
    NotFound,
    Detected,
    IncompatibleAddon,
}

pub fn detect_reshade(game: &GamePaths) -> ReShadeStatus {
    let detected = RESHADE_DLL_CANDIDATES
        .iter()
        .any(|candidate| dll_id::identify_file(&game.root().join(candidate)) == GraphicsDll::ReShade);
    if detected { ReShadeStatus::Detected } else { ReShadeStatus::NotFound }
}

pub fn auto_disable_reshade_loading(game: &GamePaths) -> Result<bool> {
    let raw = yp_core::fs::read_text_opt(&game.nams_toml())?.unwrap_or_default();
    if raw.is_empty() || flags::is_disabled(game, LoadingFlag::ReShade) {
        return Ok(false);
    }
    flags::set_disabled(game, LoadingFlag::ReShade, true)?;
    Ok(true)
}
