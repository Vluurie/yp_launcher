use std::path::PathBuf;
use std::sync::Arc;

use yp_core::{
    AppPaths, CancelToken, GamePaths, HostEnv, ProcessRunner, SettingsStore, SystemRunner,
};
use yp_health::diagnostics::model::DISCORD_LIMIT;
use yp_health::diagnostics::{Collector, format_full_report, format_summary};

#[test]
#[ignore = "needs YP_GAME_DIR pointing at a NieR:Automata install; YP_ASSETS_DIR picks the launcher runtime"]
fn real_install_report_is_complete_and_private() {
    let root = PathBuf::from(std::env::var_os("YP_GAME_DIR").expect("YP_GAME_DIR"));
    let host = HostEnv::current();
    let paths = AppPaths::detect(&host).unwrap();
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemRunner);
    let platform = yp_host::adapter_for(host.clone(), runner.clone());
    let collector = Collector::new(
        host.clone(),
        paths.clone(),
        Arc::new(SettingsStore::in_memory()),
        runner,
        platform,
    );

    let report = collector
        .collect(Some(&GamePaths::new(&root)), None, &CancelToken::new())
        .unwrap();
    let full = format_full_report(&report);
    let summary = format_summary(&report);
    println!("{summary}\n#####\n{full}");

    assert!(summary.chars().count() <= DISCORD_LIMIT);
    assert!(full.starts_with("YP Launcher Diagnostics Report\nFormat: 2\n"));
    let private = [
        root.to_string_lossy().into_owned(),
        paths.runtime_dir.to_string_lossy().into_owned(),
        host.home()
            .map(|home| home.to_string_lossy().into_owned())
            .unwrap_or_default(),
    ];
    for text in [&full, &summary] {
        let lowered = text.to_lowercase();
        for path in private.iter().filter(|path| path.chars().count() >= 3) {
            assert!(!lowered.contains(&path.to_lowercase()), "{path} leaked");
            assert!(
                !lowered.contains(&path.to_lowercase().replace('\\', "/")),
                "{path} leaked"
            );
        }
    }
    assert!(
        report.collection_errors.is_empty(),
        "{:?}",
        report.collection_errors
    );
}
