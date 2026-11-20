use std::path::PathBuf;

use yp_core::HostEnv;
use yp_core::wine_paths::NIER_STEAM_APP_ID;

const APPIMAGE_VAR: &str = "APPIMAGE";
const STEAM_GAME_VARS: [&str; 2] = ["SteamAppId", "SteamGameId"];

pub fn launcher_path(host: &HostEnv) -> PathBuf {
    host.var(APPIMAGE_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| host.exe_path().to_path_buf())
}

pub fn launch_option(host: &HostEnv) -> String {
    format!("\"{}\" %command%", launcher_path(host).display())
}

pub fn started_by_steam(host: &HostEnv) -> bool {
    STEAM_GAME_VARS
        .iter()
        .any(|name| host.var(name) == Some(NIER_STEAM_APP_ID))
}

#[cfg(test)]
mod tests {
    use yp_core::Os;

    use super::*;

    #[test]
    fn the_launch_option_starts_the_appimage_rather_than_its_mount() {
        let root = tempfile::tempdir().unwrap();
        let appimage = "/home/user/Apps/YoRHa Protocol Launcher.AppImage";
        let host = HostEnv::for_test(Os::Linux, &[(APPIMAGE_VAR, appimage)], root.path());
        assert_eq!(launch_option(&host), format!("\"{appimage}\" %command%"));

        let plain = HostEnv::for_test(Os::Linux, &[], root.path());
        assert_eq!(launcher_path(&plain), plain.exe_path());
    }

    #[test]
    fn only_steam_starting_nier_counts_as_started_by_steam() {
        let root = tempfile::tempdir().unwrap();
        let by_steam = HostEnv::for_test(Os::Linux, &[("SteamGameId", NIER_STEAM_APP_ID)], root.path());
        assert!(started_by_steam(&by_steam));
        let other_game = HostEnv::for_test(Os::Linux, &[("SteamAppId", "570")], root.path());
        assert!(!started_by_steam(&other_game));
        assert!(!started_by_steam(&HostEnv::for_test(Os::Linux, &[], root.path())));
    }
}
