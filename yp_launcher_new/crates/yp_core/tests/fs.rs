use std::path::{Path, PathBuf};

use yp_core::fs::{
    any_file, copy_dir, copy_file_into, copy_file_into_as, file_size_label, files_differ, files_where, merge_dir,
    merge_dir_keep_existing, move_dir, move_merging, scoped_temp_dir, sweep_stale_temp, temp_sweep_roots,
    unique_child_name, unwrap_single_child, walk_entities, EntryKind,
};
use yp_core::{GamePaths, HostEnv, Os};

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn files_differ_checks_presence_size_then_content() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    write(&a, "same");
    write(&b, "same");
    assert!(!files_differ(&a, &b).unwrap());
    write(&b, "diff");
    assert!(files_differ(&a, &b).unwrap());
    write(&b, "longer");
    assert!(files_differ(&a, &b).unwrap());
    assert!(files_differ(&a, &dir.path().join("missing")).unwrap());
}

#[test]
fn merge_overwrites_and_keep_existing_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    let dest = dir.path().join("dest");
    write(&src.join("a.txt"), "new");
    write(&src.join("sub").join("b.txt"), "b");
    write(&dest.join("a.txt"), "old");
    write(&dest.join("keep.txt"), "keep");

    merge_dir_keep_existing(&src, &dest).unwrap();
    assert_eq!(read(&dest.join("a.txt")), "old");
    assert_eq!(read(&dest.join("sub").join("b.txt")), "b");

    merge_dir(&src, &dest).unwrap();
    assert_eq!(read(&dest.join("a.txt")), "new");
    assert_eq!(read(&dest.join("keep.txt")), "keep");
    assert!(src.join("a.txt").exists());

    copy_dir(&src, &dir.path().join("copy")).unwrap();
    assert_eq!(read(&dir.path().join("copy").join("sub").join("b.txt")), "b");
    merge_dir(&dir.path().join("missing"), &dest).unwrap();
}

#[test]
fn move_dir_renames_and_creates_parents() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    write(&src.join("x.dat"), "x");
    let dest = dir.path().join("deep").join("dest");
    move_dir(&src, &dest).unwrap();
    assert!(!src.exists());
    assert_eq!(read(&dest.join("x.dat")), "x");
    move_dir(&src, &dest).unwrap();
}

#[test]
fn move_merging_moves_new_files_drops_duplicates_and_leaves_conflicts() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    let dest = dir.path().join("dest");
    write(&src.join("new.txt"), "new");
    write(&src.join("same.txt"), "same");
    write(&src.join("conflict.txt"), "mine");
    write(&src.join("nested").join("inner.txt"), "inner");
    write(&src.join("blocked").join("child.txt"), "child");
    write(&dest.join("same.txt"), "same");
    write(&dest.join("conflict.txt"), "theirs");
    write(&dest.join("nested").join("other.txt"), "other");
    write(&dest.join("blocked"), "a file in the way");

    let left = move_merging(&src, &dest).unwrap();

    assert_eq!(read(&dest.join("new.txt")), "new");
    assert_eq!(read(&dest.join("nested").join("inner.txt")), "inner");
    assert_eq!(read(&dest.join("nested").join("other.txt")), "other");
    assert_eq!(read(&dest.join("conflict.txt")), "theirs");
    assert!(!src.join("same.txt").exists());
    assert!(!src.join("nested").exists());
    let mut left: Vec<PathBuf> = left;
    left.sort();
    assert_eq!(left, vec![src.join("blocked"), src.join("conflict.txt")]);
    assert!(src.join("conflict.txt").exists());
}

#[test]
fn move_merging_renames_a_whole_dir_into_a_free_spot() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    write(&src.join("a").join("b.txt"), "b");
    let dest = dir.path().join("x").join("dest");
    assert!(move_merging(&src, &dest).unwrap().is_empty());
    assert_eq!(read(&dest.join("a").join("b.txt")), "b");
    assert!(!src.exists());
    assert!(move_merging(&src, &dest).unwrap().is_empty());
}

#[test]
fn copy_file_into_creates_the_dir_and_supports_renames() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.dll");
    write(&file, "dll");
    let target = copy_file_into(&file, &dir.path().join("out")).unwrap();
    assert_eq!(target, dir.path().join("out").join("a.dll"));
    assert_eq!(read(&target), "dll");
    let renamed = copy_file_into_as(&file, &dir.path().join("out"), "b.dll").unwrap();
    assert_eq!(read(&renamed), "dll");
    assert!(copy_file_into(&dir.path().join("missing"), &dir.path().join("out")).is_err());
}

#[test]
fn unwrap_single_child_skips_readme_noise() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("extract");
    write(&root.join("ModName").join("data").join("pl.dat"), "x");
    write(&root.join("README.md"), "r");
    write(&root.join("info.txt"), "i");
    write(&root.join("Nexus.url"), "u");
    assert_eq!(unwrap_single_child(&root).unwrap(), root.join("ModName"));

    write(&root.join("extra.dat"), "x");
    assert_eq!(unwrap_single_child(&root).unwrap(), root);

    let missing = dir.path().join("missing");
    assert_eq!(unwrap_single_child(&missing).unwrap(), missing);
}

#[test]
fn files_where_reports_lower_slash_relative_paths_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(&root.join("B.toml"), "");
    write(&root.join("a.dat"), "");
    write(&root.join("Data").join("PL0000.dat"), "");
    write(&root.join("Data").join("Sub").join("x.DAT"), "");
    let mut seen = Vec::new();
    let found = files_where(root, &mut |relative, _| {
        seen.push(relative.to_owned());
        relative.ends_with(".dat")
    })
    .unwrap();
    assert_eq!(seen, ["a.dat", "b.toml", "data/pl0000.dat", "data/sub/x.dat"]);
    assert_eq!(
        found,
        vec![
            root.join("a.dat"),
            root.join("Data").join("PL0000.dat"),
            root.join("Data").join("Sub").join("x.DAT"),
        ]
    );
    assert!(any_file(root, &mut |relative, _| relative == "data/sub/x.dat"));
    assert!(!any_file(root, &mut |relative, _| relative == "nope"));
    assert!(files_where(&root.join("missing"), &mut |_, _| true).unwrap().is_empty());
}

#[test]
fn walk_entities_stops_when_asked() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("a").join("1.txt"), "");
    write(&dir.path().join("b").join("2.txt"), "");
    let mut visited = Vec::new();
    walk_entities(dir.path(), &mut |path, kind| {
        visited.push((path.file_name().unwrap().to_string_lossy().into_owned(), kind));
        kind != EntryKind::File
    });
    assert_eq!(
        visited,
        vec![
            ("a".to_owned(), EntryKind::Dir),
            ("b".to_owned(), EntryKind::Dir),
            ("1.txt".to_owned(), EntryKind::File),
        ]
    );
}

#[test]
fn unique_child_name_counts_up_from_two() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(unique_child_name(dir.path(), "Mod"), "Mod");
    std::fs::create_dir(dir.path().join("Mod")).unwrap();
    write(&dir.path().join("Mod (2)"), "");
    assert_eq!(unique_child_name(dir.path(), "Mod"), "Mod (3)");
}

#[test]
fn file_size_label_needs_an_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f");
    std::fs::write(&file, vec![0u8; 2048]).unwrap();
    assert_eq!(file_size_label(&file).as_deref(), Some("2 KB"));
    assert_eq!(file_size_label(dir.path()), None);
    assert_eq!(file_size_label(&dir.path().join("missing")), None);
}

#[test]
fn scoped_temp_prefers_the_game_volume_and_sweeps_stale_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let host = HostEnv::for_test(Os::Linux, &[], dir.path());
    let game = GamePaths::new(dir.path().join("game"));
    let scoped = scoped_temp_dir(Some(&game), &host, "archive_").unwrap();
    assert!(scoped.path().starts_with(game.scoped_temp_root()));
    let fallback = scoped_temp_dir(None, &host, "texture_").unwrap();
    assert!(fallback.path().starts_with(host.temp_dir()));

    let roots = temp_sweep_roots(Some(&game), &host);
    assert_eq!(roots, vec![host.temp_dir().to_path_buf(), game.scoped_temp_root()]);
    std::fs::create_dir_all(host.temp_dir().join("unrelated")).unwrap();
    write(&host.temp_dir().join("archive_file"), "not a dir");
    let scoped_path = scoped.path().to_path_buf();
    let fallback_path = fallback.path().to_path_buf();
    sweep_stale_temp(&roots, &["archive_", "texture_"]);
    assert!(!scoped_path.exists());
    assert!(!fallback_path.exists());
    assert!(host.temp_dir().join("unrelated").exists());
    assert!(host.temp_dir().join("archive_file").exists());
}
