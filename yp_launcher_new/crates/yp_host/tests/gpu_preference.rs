use std::path::Path;

use indexmap::IndexMap;
use yp_core::{HostEnv, Os, SettingsStore};
use yp_host::gpu::{
    linux_gpu_env, merged_launch_env, prefer_dedicated, GpuProbe, NVIDIA_OFFLOAD_ENV, PREFER_DEDICATED_GPU,
};

fn probe(nvidia: bool, nodes: usize) -> GpuProbe {
    GpuProbe {
        nvidia_driver_loaded: nvidia,
        render_nodes: nodes,
    }
}

fn as_map(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect()
}

#[test]
fn a_single_gpu_gets_no_offload_variables() {
    assert!(linux_gpu_env(&probe(false, 1)).is_empty());
}

#[test]
fn a_single_nvidia_gpu_has_nothing_to_offload_to() {
    assert!(linux_gpu_env(&probe(true, 1)).is_empty());
}

#[test]
fn no_render_nodes_yields_nothing() {
    assert!(linux_gpu_env(&probe(false, 0)).is_empty());
}

#[test]
fn an_nvidia_hybrid_gets_the_prime_offload_set_and_no_dri_prime() {
    let env = as_map(linux_gpu_env(&probe(true, 2)));
    assert_eq!(env, as_map(&NVIDIA_OFFLOAD_ENV));
    assert!(!env.contains_key("DRI_PRIME"));
    assert_eq!(env["__GLX_VENDOR_LIBRARY_NAME"], "nvidia");
    assert_eq!(env["__VK_LAYER_NV_optimus"], "NVIDIA_only");
}

#[test]
fn a_mesa_hybrid_gets_dri_prime_only() {
    let env = as_map(linux_gpu_env(&probe(false, 2)));
    assert_eq!(env, as_map(&[("DRI_PRIME", "1")]));
    assert!(!env.contains_key("__GLX_VENDOR_LIBRARY_NAME"));
    assert!(!env.contains_key("__VK_LAYER_NV_optimus"));
}

#[test]
fn disabled_leaves_the_base_environment_untouched() {
    let base = Some(as_map(&[("HOME", "/home/u")]));
    assert_eq!(merged_launch_env(base.clone(), false, Os::Linux, &probe(true, 2)), base);
}

#[test]
fn merging_only_happens_on_linux_hybrids() {
    let base = Some(as_map(&[("HOME", "/home/u")]));
    assert_eq!(merged_launch_env(base.clone(), true, Os::Windows, &probe(true, 2)), base);
    assert_eq!(merged_launch_env(base.clone(), true, Os::Linux, &probe(true, 1)), base);
    let merged = merged_launch_env(base, true, Os::Linux, &probe(false, 3)).unwrap();
    assert_eq!(merged.keys().collect::<Vec<_>>(), ["HOME", "DRI_PRIME"]);
    assert_eq!(merged_launch_env(None, true, Os::Linux, &probe(false, 2)), Some(as_map(&[("DRI_PRIME", "1")])));
}

#[test]
fn detect_never_fails_and_reports_no_hybrid_off_linux() {
    let windows = GpuProbe::detect(&HostEnv::for_test(Os::Windows, &[], Path::new("/r")));
    assert_eq!(windows, GpuProbe::default());
    assert_eq!(windows.is_hybrid(), windows.render_nodes > 1);
}

#[test]
fn detect_counts_render_nodes_and_the_nvidia_driver() {
    let dir = tempfile::tempdir().unwrap();
    let dri = dir.path().join("dri");
    std::fs::create_dir_all(&dri).unwrap();
    for name in ["card0", "renderD128", "renderD129"] {
        std::fs::write(dri.join(name), "").unwrap();
    }
    let version = dir.path().join("nvidia_version");
    std::fs::write(&version, "NVRM").unwrap();
    assert_eq!(GpuProbe::detect_at(&dri, &version), probe(true, 2));
    assert_eq!(GpuProbe::detect_at(&dir.path().join("none"), &dir.path().join("none")), probe(false, 0));
}

#[test]
fn preference_defaults_to_dedicated() {
    let settings = SettingsStore::in_memory();
    assert!(prefer_dedicated(&settings));
    settings.set(&PREFER_DEDICATED_GPU, &false).unwrap();
    assert!(!prefer_dedicated(&settings));
}

#[cfg(windows)]
#[test]
fn registry_preference_is_written_and_removed() {
    let exe = std::env::temp_dir().join(format!("yp_gpu_test_{}.exe", std::process::id()));
    assert!(yp_host::gpu::apply(&exe, true));
    assert_eq!(yp_host::gpu::registered_preference(&exe).as_deref(), Some("GpuPreference=2;"));
    assert!(yp_host::gpu::apply(&exe, false));
    assert_eq!(yp_host::gpu::registered_preference(&exe), None);
}
