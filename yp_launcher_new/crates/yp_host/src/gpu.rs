use std::path::Path;

use yp_core::{HostEnv, Os, SettingKey, SettingsStore};

use crate::launch::command::LaunchEnv;

pub const PREFER_DEDICATED_GPU: SettingKey<bool> = SettingKey::new("prefer_dedicated_gpu", || true);

pub const NVIDIA_OFFLOAD_ENV: [(&str, &str); 3] = [
    ("__NV_PRIME_RENDER_OFFLOAD", "1"),
    ("__GLX_VENDOR_LIBRARY_NAME", "nvidia"),
    ("__VK_LAYER_NV_optimus", "NVIDIA_only"),
];

pub const MESA_PRIME_ENV: [(&str, &str); 1] = [("DRI_PRIME", "1")];

pub const USER_GPU_PREFERENCES_KEY: &str = r"Software\Microsoft\DirectX\UserGpuPreferences";
pub const HIGH_PERFORMANCE_VALUE: &str = "GpuPreference=2;";

const DRI_DIR: &str = "/dev/dri";
const NVIDIA_VERSION_FILE: &str = "/proc/driver/nvidia/version";
const RENDER_NODE_PREFIX: &str = "renderD";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GpuProbe {
    pub nvidia_driver_loaded: bool,
    pub render_nodes: usize,
}

impl GpuProbe {
    pub fn detect(host: &HostEnv) -> Self {
        if host.os() != Os::Linux {
            return Self::default();
        }
        Self::detect_at(Path::new(DRI_DIR), Path::new(NVIDIA_VERSION_FILE))
    }

    pub fn detect_at(dri_dir: &Path, nvidia_version_file: &Path) -> Self {
        let render_nodes = std::fs::read_dir(dri_dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.file_name().to_string_lossy().starts_with(RENDER_NODE_PREFIX))
                    .count()
            })
            .unwrap_or(0);
        Self {
            nvidia_driver_loaded: nvidia_version_file.is_file(),
            render_nodes,
        }
    }

    pub fn is_hybrid(&self) -> bool {
        self.render_nodes > 1
    }
}

pub fn prefer_dedicated(settings: &SettingsStore) -> bool {
    settings.get(&PREFER_DEDICATED_GPU)
}

pub fn linux_gpu_env(probe: &GpuProbe) -> &'static [(&'static str, &'static str)] {
    if !probe.is_hybrid() {
        return &[];
    }
    if probe.nvidia_driver_loaded {
        &NVIDIA_OFFLOAD_ENV
    } else {
        &MESA_PRIME_ENV
    }
}

pub fn merged_launch_env(base: Option<LaunchEnv>, enabled: bool, os: Os, probe: &GpuProbe) -> Option<LaunchEnv> {
    if !enabled || os != Os::Linux {
        return base;
    }
    let extra = linux_gpu_env(probe);
    if extra.is_empty() {
        return base;
    }
    let mut merged = base.unwrap_or_default();
    for (key, value) in extra {
        merged.insert((*key).to_owned(), (*value).to_owned());
    }
    Some(merged)
}

#[cfg(windows)]
pub fn apply(nams_exe: &Path, enabled: bool) -> bool {
    let value_name = nams_exe.to_string_lossy();
    let Ok(key) = windows_registry::CURRENT_USER.create(USER_GPU_PREFERENCES_KEY) else {
        return false;
    };
    if enabled {
        key.set_string(&value_name, HIGH_PERFORMANCE_VALUE).is_ok()
    } else {
        let _ = key.remove_value(&value_name);
        true
    }
}

#[cfg(not(windows))]
pub fn apply(_nams_exe: &Path, _enabled: bool) -> bool {
    false
}

#[cfg(windows)]
pub fn registered_preference(nams_exe: &Path) -> Option<String> {
    windows_registry::CURRENT_USER
        .open(USER_GPU_PREFERENCES_KEY)
        .and_then(|key| key.get_string(nams_exe.to_string_lossy()))
        .ok()
}

#[cfg(not(windows))]
pub fn registered_preference(_nams_exe: &Path) -> Option<String> {
    None
}
