use std::path::{Path, PathBuf};

use yp_core::AppPaths;
use yp_host::shortcut::{desktop_shortcut_spec, shortcut_file_name};

#[test]
fn the_shortcut_targets_nams_with_the_game_dir() {
    let paths = AppPaths::for_test(Path::new("/app"));
    let spec = desktop_shortcut_spec(&paths, Path::new("/desk"), Path::new("D:/Games/NieR"));
    assert_eq!(shortcut_file_name(), "NieR Automata (YoRHa Protocol).lnk");
    assert_eq!(spec.shortcut_path, Path::new("/desk").join("NieR Automata (YoRHa Protocol).lnk"));
    assert_eq!(spec.target, paths.nams_exe());
    assert_eq!(spec.arguments, "run --nier-path \"D:/Games/NieR\"");
    assert_eq!(spec.working_dir, paths.runtime_dir);
    assert_eq!(spec.icon, PathBuf::from("D:/Games/NieR").join("NieRAutomata.exe"));
    assert_eq!(spec.description, "Launch NieR:Automata with YoRHa Protocol");
}

#[cfg(windows)]
#[test]
fn creates_a_real_lnk_file() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_test(dir.path());
    std::fs::create_dir_all(&paths.runtime_dir).unwrap();
    std::fs::write(paths.nams_exe(), "exe").unwrap();
    let desktop = dir.path().join("Desktop");
    std::fs::create_dir_all(&desktop).unwrap();
    let spec = desktop_shortcut_spec(&paths, &desktop, &dir.path().join("game"));
    assert!(yp_host::shortcut::create(&spec));
    let bytes = std::fs::read(&spec.shortcut_path).unwrap();
    assert_eq!(&bytes[..4], &[0x4c, 0x00, 0x00, 0x00]);
    std::fs::remove_file(&spec.shortcut_path).unwrap();
}

#[cfg(windows)]
#[test]
fn the_current_process_is_found_by_name_and_unknown_names_are_not() {
    let exe = std::env::current_exe().unwrap();
    let name = exe.file_name().unwrap().to_string_lossy().to_uppercase();
    let ids = yp_host::platform::win32::process_ids_named(&name);
    assert!(ids.contains(&std::process::id()));
    assert!(!yp_host::platform::win32::is_process_running("yp-definitely-not-running.exe"));
    assert!(!yp_host::platform::win32::terminate_processes_named("yp-definitely-not-running.exe"));
}

#[cfg(windows)]
#[test]
fn a_helper_the_launcher_started_is_known_while_it_runs() {
    use std::time::{Duration, Instant};

    use yp_core::process::{ProcessRunner, ProcessSpec, SystemRunner, excluding_helpers};

    let helper = std::thread::spawn(|| {
        let spec = ProcessSpec::new("ping")
            .args(["-n", "3", "127.0.0.1"])
            .hide_console();
        SystemRunner.output(&spec, Some(Duration::from_secs(20)))
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut known = false;
    while !known && Instant::now() < deadline {
        known = excluding_helpers(|helpers| {
            yp_host::platform::win32::process_ids_named("PING.EXE")
                .iter()
                .any(|id| helpers.contains(id))
        });
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(known);
    assert!(helper.join().unwrap().unwrap().is_success());
    assert!(excluding_helpers(|helpers| helpers.is_empty()));
}
