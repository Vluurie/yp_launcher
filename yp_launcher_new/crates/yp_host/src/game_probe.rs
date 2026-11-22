use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};
use yp_core::consts::{GAME_EXE_NAME, GAME_SIGNATURE};
use yp_core::hash::{sha1_file, sha256_file, to_hex};

use crate::vdf::{self, VdfObject};

pub const WOLF_LIMIT_BREAK_SHA256: &str = "c9904a39e448d6cb3c98c28a90159cf9314753b0cfea5ceb4e76a12d3308a355";
pub const LEGACY_WINDOWS7_SIZE: u64 = 17_786_752;
pub const LEGACY_WINDOWS7_SHA1: &str = "f734bef5403c77e29cfc55f57707df19c3655751";
pub const WINDOWS10_SIZE: u64 = 17_837_952;
pub const WINDOWS10_SHA1: &str = "420cbab7daa1d94a79473c0a6271ef9c30e516e5";
pub const STEAM_2026_SIZE: u64 = 17_601_536;
pub const STEAM_2026_SHA1: &str = "537be3ea1083114cc1e444b85ac270f2043ac4d6";
pub const STEAM_2026_SIGNED_SIZE: u64 = 17_613_568;
pub const STEAM_2026_SIGNED_SHA1: &str = "f2fbdbbdddb939a992185dfa9ff404c8e25dd605";
pub const SUPPORTED_EXE_SIZES: [u64; 3] = [WINDOWS10_SIZE, STEAM_2026_SIZE, STEAM_2026_SIGNED_SIZE];
pub const STEAM_2021_TIME_DATE_STAMP: u32 = 0x60d3_a8d2;
pub const STEAM_2026_TIME_DATE_STAMP: u32 = 0x6ac8_9519;
pub const STEAM_APP_ID: &str = "524220";
pub const DLC_APP_ID: &str = "580600";
pub const PE_HEADER_BYTES: usize = 0x1000;

const DLC_ARCHIVE: &str = "data100.cpk";
const DLC_SIZE_LOWER: u64 = (928 - 5) * 1024 * 1024;
const DLC_SIZE_UPPER: u64 = 937 * 1024 * 1024;
const PE_SIGNATURE: u32 = 0x0000_4550;
const NT_HEADER_OFFSET: usize = 0x3c;
const HD_TEXTURE_DIR: &str = "SK_Res";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExeVariant {
    Missing,
    Original,
    WolfLimitBreak,
    LegacyWindows7,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameBuild {
    Steam2021,
    Steam2026_11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameBuildInfo {
    pub build: GameBuild,
    pub label: &'static str,
    pub exe_size: u64,
    pub exe_sha1: &'static str,
    pub time_date_stamp: u32,
}

pub const KNOWN_BUILDS: [GameBuildInfo; 3] = [
    GameBuildInfo {
        build: GameBuild::Steam2021,
        label: "Steam 2021",
        exe_size: WINDOWS10_SIZE,
        exe_sha1: WINDOWS10_SHA1,
        time_date_stamp: STEAM_2021_TIME_DATE_STAMP,
    },
    GameBuildInfo {
        build: GameBuild::Steam2026_11,
        label: "Steam 2026-11",
        exe_size: STEAM_2026_SIZE,
        exe_sha1: STEAM_2026_SHA1,
        time_date_stamp: STEAM_2026_TIME_DATE_STAMP,
    },
    GameBuildInfo {
        build: GameBuild::Steam2026_11,
        label: "Steam 2026-11 (signed)",
        exe_size: STEAM_2026_SIGNED_SIZE,
        exe_sha1: STEAM_2026_SIGNED_SHA1,
        time_date_stamp: STEAM_2026_TIME_DATE_STAMP,
    },
];

pub fn build_of_exe(size: u64, sha1: &str) -> Option<&'static GameBuildInfo> {
    KNOWN_BUILDS.iter().find(|info| info.exe_size == size && info.exe_sha1 == sha1)
}

pub fn build_of_time_date_stamp(stamp: Option<u32>) -> Option<&'static GameBuildInfo> {
    let stamp = stamp?;
    KNOWN_BUILDS.iter().find(|info| info.time_date_stamp == stamp)
}

pub fn newer_than_every_build(stamp: Option<u32>) -> bool {
    stamp.is_some_and(|stamp| KNOWN_BUILDS.iter().all(|build| stamp > build.time_date_stamp))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    let slice: [u8; 4] = bytes.get(at..end)?.try_into().ok()?;
    Some(u32::from_le_bytes(slice))
}

pub fn pe_time_date_stamp(header: &[u8]) -> Option<u32> {
    if header.len() < 2 || header[0] != 0x4d || header[1] != 0x5a {
        return None;
    }
    let nt = usize::try_from(u32_at(header, NT_HEADER_OFFSET)?).ok()?;
    if u32_at(header, nt)? != PE_SIGNATURE {
        return None;
    }
    u32_at(header, nt.checked_add(8)?)
}

fn read_header(file: &mut File) -> std::io::Result<Vec<u8>> {
    let mut header = Vec::with_capacity(PE_HEADER_BYTES);
    file.take(PE_HEADER_BYTES as u64).read_to_end(&mut header)?;
    Ok(header)
}

pub fn file_time_date_stamp(path: &Path) -> Option<u32> {
    let mut file = File::open(path).ok()?;
    pe_time_date_stamp(&read_header(&mut file).ok()?)
}

const SIGNATURE_SCAN_CHUNK: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExeSelection {
    Valid,
    Missing,
    NotNierExe,
}

pub fn contains_bytes(mut reader: impl Read, needle: &[u8]) -> std::io::Result<bool> {
    if needle.is_empty() {
        return Ok(true);
    }
    let overlap = needle.len() - 1;
    let mut window = vec![0u8; overlap + SIGNATURE_SCAN_CHUNK];
    let mut carried = 0;
    loop {
        let read = reader.read(&mut window[carried..])?;
        if read == 0 {
            return Ok(false);
        }
        let filled = carried + read;
        if window[..filled].windows(needle.len()).any(|candidate| candidate == needle) {
            return Ok(true);
        }
        let keep = overlap.min(filled);
        window.copy_within(filled - keep..filled, 0);
        carried = keep;
    }
}

pub fn is_nier_exe(exe: &Path) -> bool {
    let Ok(mut file) = File::open(exe) else {
        return false;
    };
    let mut magic = [0u8; 2];
    if file.read_exact(&mut magic).is_err() || magic != [0x4d, 0x5a] {
        return false;
    }
    contains_bytes(file, GAME_SIGNATURE.as_bytes()).unwrap_or(false)
}

pub fn check_exe_selection(exe: &Path) -> ExeSelection {
    if !exe.is_file() {
        ExeSelection::Missing
    } else if is_nier_exe(exe) {
        ExeSelection::Valid
    } else {
        ExeSelection::NotNierExe
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExeBuildStatus {
    Missing,
    Known,
    LegacyWindows7,
    Modified,
    Newer,
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExeProbe {
    pub exists: bool,
    pub size: Option<u64>,
    pub sha1: Option<String>,
    pub time_date_stamp: Option<u32>,
}

impl ExeProbe {
    pub fn build(&self) -> Option<&'static GameBuildInfo> {
        build_of_exe(self.size?, self.sha1.as_deref()?)
    }

    pub fn stamp_build(&self) -> Option<&'static GameBuildInfo> {
        build_of_time_date_stamp(self.time_date_stamp)
    }

    pub fn status(&self) -> ExeBuildStatus {
        if !self.exists {
            return ExeBuildStatus::Missing;
        }
        if self.build().is_some() {
            return ExeBuildStatus::Known;
        }
        if self.size == Some(LEGACY_WINDOWS7_SIZE) && self.sha1.as_deref() == Some(LEGACY_WINDOWS7_SHA1) {
            return ExeBuildStatus::LegacyWindows7;
        }
        if self.stamp_build().is_some() {
            return ExeBuildStatus::Modified;
        }
        if newer_than_every_build(self.time_date_stamp) {
            return ExeBuildStatus::Newer;
        }
        ExeBuildStatus::Unknown
    }
}

pub fn game_exe(game_dir: &Path) -> PathBuf {
    game_dir.join(GAME_EXE_NAME)
}

pub fn probe_exe_file(exe: &Path) -> ExeProbe {
    if !exe.is_file() {
        return ExeProbe::default();
    }
    let size = std::fs::metadata(exe).ok().map(|meta| meta.len());
    let probed = File::open(exe).and_then(|mut file| {
        let header = read_header(&mut file)?;
        let mut hasher = Sha1::new();
        hasher.update(&header);
        let mut buffer = vec![0u8; 1 << 16];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok((to_hex(&hasher.finalize()), pe_time_date_stamp(&header)))
    });
    match probed {
        Ok((sha1, time_date_stamp)) => ExeProbe {
            exists: true,
            size,
            sha1: Some(sha1),
            time_date_stamp,
        },
        Err(_) => ExeProbe {
            exists: true,
            size,
            ..ExeProbe::default()
        },
    }
}

pub fn probe_exe(game_dir: &Path) -> ExeProbe {
    probe_exe_file(&game_exe(game_dir))
}

pub fn detect_exe_variant(game_dir: &Path) -> ExeVariant {
    let exe = game_exe(game_dir);
    let Ok(meta) = std::fs::metadata(&exe) else {
        return ExeVariant::Missing;
    };
    if !meta.is_file() {
        return ExeVariant::Missing;
    }
    if meta.len() == LEGACY_WINDOWS7_SIZE
        && sha1_file(&exe).is_ok_and(|hash| to_hex(&hash) == LEGACY_WINDOWS7_SHA1)
    {
        return ExeVariant::LegacyWindows7;
    }
    match sha256_file(&exe) {
        Err(_) => ExeVariant::Unknown,
        Ok(hash) if to_hex(&hash) == WOLF_LIMIT_BREAK_SHA256 => ExeVariant::WolfLimitBreak,
        Ok(_) => ExeVariant::Original,
    }
}

pub fn detect_hd_textures(game_dir: &Path) -> Vec<&'static str> {
    let sk_res = game_dir.join(HD_TEXTURE_DIR);
    let non_empty = std::fs::read_dir(&sk_res).is_ok_and(|mut entries| entries.next().is_some());
    if non_empty { vec!["SK_Res/"] } else { Vec::new() }
}

pub fn steam_manifest_path(game_dir: &Path) -> Option<PathBuf> {
    let common = game_dir.parent()?;
    let steamapps = common.parent()?;
    let is_named = |path: &Path, name: &str| {
        path.file_name()
            .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name))
    };
    if !is_named(common, "common") || !is_named(steamapps, "steamapps") {
        return None;
    }
    Some(steamapps.join(format!("appmanifest_{STEAM_APP_ID}.acf")))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SteamAppManifest {
    pub build_id: Option<String>,
    pub target_build_id: Option<String>,
    pub state_flags: Option<String>,
    pub last_updated: Option<String>,
    pub beta_key: Option<String>,
    pub installed_dlc: BTreeSet<String>,
    pub disabled_dlc: BTreeSet<String>,
}

impl SteamAppManifest {
    pub fn parse(content: &str) -> Result<Self, vdf::VdfError> {
        let root = vdf::parse(content)?;
        Ok(Self::from_vdf(&root))
    }

    pub fn from_vdf(root: &VdfObject) -> Self {
        let state = root.block("AppState").unwrap_or(root);
        let installed_dlc = state
            .block("InstalledDepots")
            .map(|depots| {
                depots
                    .blocks()
                    .filter_map(|(_, depot)| depot.string("dlcappid"))
                    .filter(|id| !id.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let user_config = state.block("UserConfig");
        let disabled_raw = state
            .block("MountedConfig")
            .and_then(|mounted| mounted.string("DisabledDLC"))
            .or_else(|| user_config.and_then(|config| config.string("DisabledDLC")))
            .unwrap_or_default();
        let owned = |key: &str| state.string(key).map(str::to_owned);
        Self {
            build_id: owned("buildid"),
            target_build_id: owned("TargetBuildID"),
            state_flags: owned("StateFlags"),
            last_updated: owned("LastUpdated"),
            beta_key: user_config
                .and_then(|config| config.string("BetaKey"))
                .filter(|beta| !beta.is_empty())
                .map(str::to_owned),
            installed_dlc,
            disabled_dlc: disabled_raw
                .split(',')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
                .collect(),
        }
    }

    pub fn dlc_installed(&self, app_id: &str) -> bool {
        self.installed_dlc.contains(app_id) && !self.disabled_dlc.contains(app_id)
    }

    pub fn update_pending(&self) -> bool {
        match (&self.target_build_id, &self.build_id) {
            (Some(target), Some(build)) => target != "0" && target != build,
            _ => false,
        }
    }

    pub fn last_updated_at(&self) -> Option<jiff::Timestamp> {
        let seconds: i64 = self.last_updated.as_deref()?.trim().parse().ok()?;
        if seconds <= 0 {
            return None;
        }
        jiff::Timestamp::from_second(seconds).ok()
    }
}

pub fn read_steam_manifest(game_dir: &Path) -> Option<SteamAppManifest> {
    let path = steam_manifest_path(game_dir)?;
    let text = std::fs::read_to_string(path).ok()?;
    SteamAppManifest::parse(&text).ok()
}

pub fn has_dlc(game_dir: &Path) -> bool {
    if let Some(manifest) = read_steam_manifest(game_dir) {
        return manifest.dlc_installed(DLC_APP_ID);
    }
    std::fs::metadata(game_dir.join("data").join(DLC_ARCHIVE))
        .is_ok_and(|meta| meta.is_file() && (DLC_SIZE_LOWER..DLC_SIZE_UPPER).contains(&meta.len()))
}
