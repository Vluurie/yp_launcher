use std::path::{Path, PathBuf};
use std::sync::Arc;

use yp_core::consts::LOGS_DIR_NAME;
use yp_core::paths::SETTINGS_FILE_NAME;
use yp_core::{
    AppPaths, CancelToken, HostEnv, LaunchUnavailable, NamsCommandRequest, NierInstallation, Os,
    PlatformAdapter, ProcessSpec, SelectionRejection, SettingsStore, SystemRunner,
};
use yp_host::HostPlatform;
use yp_host::launch::command::LaunchCommand;
use yp_host::platform::VerifyTimeoutHint;
use yp_state::AppEnv;

pub const SANDBOX_DIR_VAR: &str = "YP_SANDBOX_DIR";
pub const DATA_DIR_NAME: &str = "data";
pub const RUNTIME_DIR_NAME: &str = "runtime";
pub const HOME_DIR_NAME: &str = "home";
pub const TEMP_DIR_NAME: &str = "tmp";

const PROFILE_VARS: [&str; 9] = [
    "APPDATA",
    "LOCALAPPDATA",
    "USERPROFILE",
    "HOME",
    "XDG_DATA_HOME",
    "XDG_CONFIG_HOME",
    "TEMP",
    "TMP",
    "TMPDIR",
];

pub struct SandboxPlatform {
    inner: Arc<dyn HostPlatform>,
    launch_allowed: bool,
}

impl SandboxPlatform {
    pub fn new(inner: Arc<dyn HostPlatform>) -> Self {
        Self {
            inner,
            launch_allowed: true,
        }
    }

    pub fn without_launch(mut self) -> Self {
        self.launch_allowed = false;
        self
    }
}

impl PlatformAdapter for SandboxPlatform {
    fn host(&self) -> &HostEnv {
        self.inner.host()
    }

    fn can_launch_game(&self) -> bool {
        self.launch_allowed && self.inner.can_launch_game()
    }

    fn is_experimental(&self) -> bool {
        self.inner.is_experimental()
    }

    fn uses_native_title_bar(&self) -> bool {
        self.inner.uses_native_title_bar()
    }

    fn supports_desktop_shortcut(&self) -> bool {
        false
    }

    fn build_nams_command(
        &self,
        request: &NamsCommandRequest<'_>,
    ) -> Result<ProcessSpec, LaunchUnavailable> {
        self.inner.build_nams_command(request)
    }

    fn resolve_nams_settings_path(&self, game_dir: Option<&Path>) -> Option<PathBuf> {
        self.inner.resolve_nams_settings_path(game_dir)
    }

    fn is_game_running(&self) -> bool {
        false
    }

    fn terminate_game(&self) -> bool {
        false
    }

    fn discover_fast(&self) -> Vec<NierInstallation> {
        Vec::new()
    }

    fn discover_deep(&self, _cancel: &CancelToken) -> Vec<NierInstallation> {
        Vec::new()
    }

    fn reject_game_selection(&self, exe: &Path) -> Option<SelectionRejection> {
        self.inner.reject_game_selection(exe)
    }

    fn reveal(&self, _target: &Path, _is_file: bool) -> yp_core::Result<()> {
        Ok(())
    }
}

impl HostPlatform for SandboxPlatform {
    fn build_launch_command(
        &self,
        request: &NamsCommandRequest<'_>,
    ) -> Result<LaunchCommand, LaunchUnavailable> {
        self.inner.build_launch_command(request)
    }

    fn verify_timeout_hint(&self) -> VerifyTimeoutHint {
        self.inner.verify_timeout_hint()
    }
}

pub fn sandbox_paths(root: &Path, assets_dir: PathBuf) -> AppPaths {
    let data_dir = root.join(DATA_DIR_NAME);
    let runtime_dir = root.join(RUNTIME_DIR_NAME);
    AppPaths {
        settings_file: data_dir.join(SETTINGS_FILE_NAME),
        logs_dir: runtime_dir.join(LOGS_DIR_NAME),
        bundled_bins_dir: runtime_dir.clone(),
        data_dir,
        runtime_dir,
        assets_dir,
    }
}

pub fn sandbox_host(root: &Path, real: &HostEnv) -> HostEnv {
    let home = root.join(HOME_DIR_NAME).to_string_lossy().into_owned();
    let temp = root.join(TEMP_DIR_NAME).to_string_lossy().into_owned();
    let mut vars: Vec<(String, String)> = real
        .vars()
        .filter(|(name, _)| {
            !PROFILE_VARS
                .iter()
                .any(|blocked| name.eq_ignore_ascii_case(blocked))
        })
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
    for name in PROFILE_VARS {
        let value = if name.starts_with('T') { &temp } else { &home };
        vars.push((name.to_owned(), value.clone()));
    }
    let borrowed: Vec<(&str, &str)> = vars
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    HostEnv::for_test(Os::native(), &borrowed, root)
        .with_system_language(real.system_language().as_deref())
}

pub fn sandbox_env(root: &Path) -> yp_core::Result<AppEnv> {
    let real = HostEnv::current();
    let assets_dir = AppPaths::detect(&real)
        .map(|paths| paths.assets_dir)
        .unwrap_or_else(|_| root.join("assets"));
    let host = sandbox_host(root, &real);
    let paths = sandbox_paths(root, assets_dir);
    for dir in [
        &paths.data_dir,
        &paths.runtime_dir,
        &paths.logs_dir,
        &root.join(HOME_DIR_NAME),
        &root.join(TEMP_DIR_NAME),
    ] {
        std::fs::create_dir_all(dir).map_err(|source| yp_core::Error::Io {
            op: yp_core::IoOp::Create,
            path: dir.to_path_buf(),
            source,
        })?;
    }
    let (settings, report) = SettingsStore::open(paths.settings_file.clone())?;
    let mut env = AppEnv::new(host, paths, Arc::new(settings), Arc::new(SystemRunner));
    env.settings_report = Some(report);
    let platform = Arc::new(SandboxPlatform::new(env.platform.clone()));
    Ok(env.with_platform(platform).with_gpu_registry(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_keeps_every_path_inside_its_folder() {
        let root = tempfile::tempdir().unwrap();
        let env = sandbox_env(root.path()).unwrap();
        for path in [
            &env.paths.data_dir,
            &env.paths.runtime_dir,
            &env.paths.logs_dir,
            &env.paths.settings_file,
            &env.paths.bundled_bins_dir,
        ] {
            assert!(path.starts_with(root.path()), "{}", path.display());
        }
        assert!(env.host.temp_dir().starts_with(root.path()));
        assert!(env.host.home().unwrap().starts_with(root.path()));
        for name in ["APPDATA", "LOCALAPPDATA", "USERPROFILE", "TEMP"] {
            let value = env.host.var(name).unwrap();
            assert!(Path::new(value).starts_with(root.path()), "{name}={value}");
        }
        assert!(!env.paths.needs_runtime_extraction());
        assert!(!env.write_gpu_registry);
    }

    #[test]
    fn sandbox_never_reaches_the_real_game() {
        let root = tempfile::tempdir().unwrap();
        let env = sandbox_env(root.path()).unwrap();
        assert!(env.platform.discover_fast().is_empty());
        assert!(env.platform.discover_deep(&CancelToken::new()).is_empty());
        assert!(!env.platform.is_game_running());
        assert!(!env.platform.terminate_game());
        assert!(!env.platform.supports_desktop_shortcut());
        assert!(env.platform.reveal(root.path(), false).is_ok());
        assert!(yp_host::session::AppSession::saved_directory(&env.settings).is_none());
    }

    #[test]
    fn sandbox_settings_stay_in_the_sandbox() {
        let root = tempfile::tempdir().unwrap();
        let env = sandbox_env(root.path()).unwrap();
        env.settings
            .set(&yp_host::session::NIER_DIRECTORY, &"G:/fake".to_owned())
            .unwrap();
        assert!(env.paths.settings_file.is_file());
        let again = sandbox_env(root.path()).unwrap();
        assert_eq!(
            yp_host::session::AppSession::saved_directory(&again.settings),
            Some(PathBuf::from("G:/fake"))
        );
    }
}
