mod support;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use support::{file_names, mod_dir, put_bytes, staging, temp};
use yp_core::{CancelToken, Error, GamePaths, NoProgress, Progress};
use yp_mods::bulk::install_auto_named;
use yp_mods::loose::{
    DropSplit, PROGRESS_INTERVAL, STAGE_TEMP_PREFIX, count_files, split_drop, stage_files,
    stage_folder,
};

fn folder_with(root: &Path, names: &[&str]) -> PathBuf {
    let folder = root.join("loose");
    std::fs::create_dir_all(&folder).unwrap();
    for name in names {
        put_bytes(&folder, name, 4, 1);
    }
    folder
}

#[test]
fn counts_only_top_level_dat_and_dtt_files() {
    let dir = temp("yp_loose_count_");
    let folder = folder_with(
        dir.path(),
        &[
            "pl0000.dat",
            "pl0000.DTT",
            "unknown_stem.dat",
            "data100.cpk",
            "readme.txt",
            ".dat",
            "sub/wp0100.dat",
        ],
    );
    std::fs::create_dir_all(folder.join("folder.dat")).unwrap();

    assert_eq!(count_files(&folder), 3);
    assert_eq!(count_files(&dir.path().join("missing")), 0);
    assert_eq!(count_files(&folder.join("readme.txt")), 0);
}

#[test]
fn staging_copies_the_loose_files_and_leaves_the_folder_alone() {
    let dir = temp("yp_loose_stage_src_");
    let folder = folder_with(
        dir.path(),
        &["pl0000.dat", "pl0000.dtt", "notes.txt", "sub/wp0100.dat"],
    );

    let staged = stage_folder(&staging(), &folder, &NoProgress, &CancelToken::new())
        .unwrap()
        .unwrap();

    assert_eq!(file_names(staged.path()), vec!["pl0000.dat", "pl0000.dtt"]);
    assert!(
        staged
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with(STAGE_TEMP_PREFIX)
    );
    assert_eq!(
        file_names(&folder),
        vec!["notes.txt", "pl0000.dat", "pl0000.dtt"]
    );
    let staged_path = staged.path().to_path_buf();
    drop(staged);
    assert!(!staged_path.exists());
}

#[test]
fn staging_reports_every_hundred_files_and_the_final_count() {
    let dir = temp("yp_loose_progress_");
    let folder = dir.path().join("loose");
    std::fs::create_dir_all(&folder).unwrap();
    for index in 0..250 {
        std::fs::write(folder.join(format!("q{index:03}.dat")), [0u8; 2]).unwrap();
    }
    let reports: Mutex<Vec<Progress>> = Mutex::new(Vec::new());
    let sink = |progress: Progress| reports.lock().unwrap().push(progress);

    let staged = stage_folder(&staging(), &folder, &sink, &CancelToken::new())
        .unwrap()
        .unwrap();

    assert_eq!(PROGRESS_INTERVAL, 100);
    assert_eq!(file_names(staged.path()).len(), 250);
    let seen = reports.lock().unwrap();
    let counts: Vec<&str> = seen
        .iter()
        .filter_map(|progress| progress.current.as_deref())
        .collect();
    assert_eq!(counts, vec!["100", "200", "250"]);
    let fractions: Vec<Option<f32>> = seen.iter().map(|progress| progress.fraction).collect();
    assert_eq!(fractions, vec![Some(0.4), Some(0.8), Some(1.0)]);
}

#[test]
fn a_single_file_still_reports_its_final_count() {
    let dir = temp("yp_loose_progress_one_");
    let folder = folder_with(dir.path(), &["corehap.dat"]);
    let reports: Mutex<Vec<Progress>> = Mutex::new(Vec::new());
    let sink = |progress: Progress| reports.lock().unwrap().push(progress);

    stage_folder(&staging(), &folder, &sink, &CancelToken::new())
        .unwrap()
        .unwrap();

    assert_eq!(
        *reports.lock().unwrap(),
        vec![Progress {
            fraction: Some(1.0),
            current: Some("1".to_owned())
        }]
    );
}

#[test]
fn nothing_is_staged_without_loose_files() {
    let dir = temp("yp_loose_none_");
    let folder = folder_with(dir.path(), &["readme.txt", "sub/pl0000.dat"]);

    for root in [folder, dir.path().join("missing")] {
        let staged = stage_folder(&staging(), &root, &NoProgress, &CancelToken::new()).unwrap();
        assert!(staged.is_none());
    }
}

#[test]
fn a_cancelled_staging_stops_with_cancelled() {
    let dir = temp("yp_loose_cancel_");
    let folder = folder_with(dir.path(), &["pl0000.dat"]);
    let cancel = CancelToken::new();
    cancel.cancel();

    let result = stage_folder(&staging(), &folder, &NoProgress, &cancel);

    assert!(matches!(result, Err(Error::Cancelled)));
}

#[test]
fn a_staged_folder_installs_under_data() {
    let dir = temp("yp_loose_install_");
    let game_dir = temp("yp_loose_install_g_");
    let folder = folder_with(dir.path(), &["corehap.dat", "pl0000.dat", "pl0000.dtt"]);

    let staged = stage_folder(&staging(), &folder, &NoProgress, &CancelToken::new())
        .unwrap()
        .unwrap();
    let installed = install_auto_named(
        &staging(),
        &GamePaths::new(game_dir.path()),
        staged.path(),
        "Loose Files",
        &NoProgress,
        &CancelToken::new(),
    )
    .unwrap();

    let data = mod_dir(game_dir.path(), &installed.id).join("data");
    assert_eq!(installed.id, "loose_files");
    assert!(data.join("core").join("corehap.dat").is_file());
    assert_eq!(
        file_names(&data.join("pl")),
        vec!["pl0000.dat", "pl0000.dtt"]
    );
}

#[test]
fn dropped_files_are_staged_in_a_folder_named_after_the_first_file() {
    let dir = temp("yp_loose_files_");
    let first = put_bytes(dir.path(), "a/pl000d.dat", 4, 1);
    let second = put_bytes(dir.path(), "b/pl000d.dtt", 4, 2);
    let third = put_bytes(dir.path(), "b/wp0100.dat", 4, 3);
    let missing = dir.path().join("b").join("gone.dat");

    let staged = stage_files(&staging(), &[first, second, missing, third])
        .unwrap()
        .unwrap();

    assert_eq!(staged.copied(), 3);
    assert_eq!(staged.source().file_name().unwrap(), "pl000d");
    assert_eq!(staged.source().parent().unwrap(), staged.root());
    assert_eq!(
        file_names(staged.source()),
        vec!["pl000d.dat", "pl000d.dtt", "wp0100.dat"]
    );
    let root = staged.root().to_path_buf();
    drop(staged);
    assert!(!root.exists());
}

#[test]
fn nothing_is_staged_when_no_dropped_file_can_be_copied() {
    let dir = temp("yp_loose_files_none_");
    let missing = vec![
        dir.path().join("gone.dat"),
        dir.path().join("also_gone.dtt"),
    ];

    assert!(stage_files(&staging(), &[]).unwrap().is_none());
    assert!(stage_files(&staging(), &missing).unwrap().is_none());
}

#[test]
fn staged_dropped_files_install_as_one_mod() {
    let dir = temp("yp_loose_files_install_");
    let game_dir = temp("yp_loose_files_install_g_");
    let dat = put_bytes(dir.path(), "pl000d.dat", 4, 1);
    let dtt = put_bytes(dir.path(), "pl000d.dtt", 4, 2);

    let staged = stage_files(&staging(), &[dat.clone(), dtt])
        .unwrap()
        .unwrap();
    let detected = support::detect(staged.source());
    let installed = support::install_named(game_dir.path(), staged.source(), "outfit").unwrap();

    assert_eq!(detected.suggested_id, "pl000d");
    assert_eq!(
        file_names(
            &mod_dir(game_dir.path(), &installed.id)
                .join("data")
                .join("pl")
        ),
        vec!["pl000d.dat", "pl000d.dtt"]
    );
    assert!(dat.is_file());
}

#[test]
fn a_drop_is_split_into_loose_files_and_everything_else() {
    let dir = temp("yp_loose_split_");
    let dat = put_bytes(dir.path(), "pl000d.dat", 4, 0);
    let dtt = put_bytes(dir.path(), "pl000d.dtt", 4, 0);
    let cpk = put_bytes(dir.path(), "data100.cpk", 4, 0);
    let archive = put_bytes(dir.path(), "mod.zip", 4, 0);
    let readme = put_bytes(dir.path(), "readme.txt", 4, 0);
    let unknown = put_bytes(dir.path(), "zzzz.dat", 4, 0);
    let folder = dir.path().join("pl0000.dat");
    std::fs::create_dir_all(&folder).unwrap();
    let missing = dir.path().join("wp0100.dat");

    let split = split_drop(&[
        archive.clone(),
        dat.clone(),
        folder.clone(),
        dtt.clone(),
        readme.clone(),
        cpk.clone(),
        unknown.clone(),
        missing.clone(),
    ]);

    assert_eq!(
        split,
        DropSplit {
            loose_files: vec![dat.clone(), dtt, cpk],
            rest: vec![archive.clone(), folder, readme.clone(), unknown, missing],
        }
    );

    let single = split_drop(&[archive.clone(), dat.clone(), readme.clone()]);
    assert_eq!(
        single,
        DropSplit {
            loose_files: Vec::new(),
            rest: vec![dat, archive, readme],
        }
    );
    assert_eq!(split_drop(&[]), DropSplit::default());
}
