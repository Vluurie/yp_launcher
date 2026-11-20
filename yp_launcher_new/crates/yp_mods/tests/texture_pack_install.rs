mod support;

use std::path::{Path, PathBuf};

use support::{put_bytes, staging, temp, walk_files};
use yp_core::{CancelToken, GamePaths, NoProgress};
use yp_mods::ModVariant;
use yp_mods::textures::pack_install::{MergeTarget, TexturePackPrompts, run};
use yp_mods::textures::{TextureCopyProgress, TextureRoots, load_textures, texture_inject_dir};

#[derive(Default)]
struct RecordingPrompts {
    copy_started: bool,
    offered: Vec<Vec<String>>,
    pick_first: bool,
    merge: Option<MergeTarget>,
    progress: Vec<TextureCopyProgress>,
}

impl TexturePackPrompts for RecordingPrompts {
    fn choose_packs(&mut self, packs: &[ModVariant]) -> Option<Vec<ModVariant>> {
        self.offered
            .push(packs.iter().map(|pack| pack.label.clone()).collect());
        if self.pick_first {
            Some(vec![packs[0].clone()])
        } else {
            Some(packs.to_vec())
        }
    }

    fn merge_target(&mut self) -> MergeTarget {
        self.merge.clone().unwrap_or(MergeTarget::KeepNames)
    }

    fn copy_started(&mut self) {
        self.copy_started = true;
    }

    fn copy_progress(&mut self, progress: &TextureCopyProgress) {
        self.progress.push(*progress);
    }
}

fn zip_of(work: &Path, name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let borrowed: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(rel, bytes)| (*rel, bytes.as_slice()))
        .collect();
    support::archive_of(work, name, &borrowed)
}

fn file_names_under(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = walk_files(root)
        .iter()
        .map(|file| file.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_texture_pack_zip_lands_in_its_own_folder() {
    let work = temp("yp_tex_pack_");
    let zip = zip_of(
        work.path(),
        "HD Pack.zip",
        &[
            ("Read me.txt", vec![1, 2, 3]),
            ("SK_Res/desktop.ini", vec![1]),
            ("SK_Res/inject/textures/04ad8a8f.dds", vec![7; 64]),
            ("SK_Res/inject/textures/0a2b11c3.dds", vec![9; 64]),
        ],
    );
    let game = GamePaths::new(work.path().join("game"));
    let texture_dir = texture_inject_dir(&game);
    let mut prompts = RecordingPrompts::default();

    let count = run(
        &staging(),
        &[zip],
        &texture_dir,
        &mut prompts,
        &NoProgress,
        &CancelToken::new(),
    )
    .unwrap();

    assert_eq!(count, Some(2));
    assert!(prompts.copy_started);
    assert!(prompts.offered.is_empty());
    assert_eq!(
        file_names_under(&texture_dir.join("HD Pack")),
        vec!["04ad8a8f.dds", "0a2b11c3.dds"]
    );
}

#[test]
fn several_packs_in_one_archive_are_offered_and_merged_under_the_archive_name() {
    let work = temp("yp_tex_multi_");
    let zip = zip_of(
        work.path(),
        "Bundle.zip",
        &[("2B/a.dds", vec![1; 16]), ("A2/b.dds", vec![2; 16])],
    );
    let texture_dir = work.path().join("inject");
    let mut prompts = RecordingPrompts::default();

    let count = run(
        &staging(),
        std::slice::from_ref(&zip),
        &texture_dir,
        &mut prompts,
        &NoProgress,
        &CancelToken::new(),
    )
    .unwrap();
    assert_eq!(count, Some(2));
    assert_eq!(
        prompts.offered,
        vec![vec!["2B".to_owned(), "A2".to_owned()]]
    );
    assert_eq!(
        file_names_under(&texture_dir.join("Bundle")),
        vec!["a.dds", "b.dds"]
    );

    let single_dir = work.path().join("inject_single");
    let mut single = RecordingPrompts {
        pick_first: true,
        ..RecordingPrompts::default()
    };
    run(
        &staging(),
        &[zip],
        &single_dir,
        &mut single,
        &NoProgress,
        &CancelToken::new(),
    )
    .unwrap();
    assert_eq!(file_names_under(&single_dir.join("2B")), vec!["a.dds"]);
}

#[test]
fn a_single_folder_can_be_merged_into_an_existing_pack_or_cancelled() {
    let work = temp("yp_tex_merge_");
    let folder = work.path().join("Extra");
    put_bytes(&folder, "x.dds", 8, 0);
    let texture_dir = work.path().join("inject");

    let mut merge = RecordingPrompts {
        merge: Some(MergeTarget::Into("Existing".to_owned())),
        ..RecordingPrompts::default()
    };
    let count = run(
        &staging(),
        std::slice::from_ref(&folder),
        &texture_dir,
        &mut merge,
        &NoProgress,
        &CancelToken::new(),
    )
    .unwrap();
    assert_eq!(count, Some(1));
    assert_eq!(
        file_names_under(&texture_dir.join("Existing")),
        vec!["x.dds"]
    );

    let mut cancel = RecordingPrompts {
        merge: Some(MergeTarget::Cancel),
        ..RecordingPrompts::default()
    };
    let none = run(
        &staging(),
        &[folder],
        &texture_dir,
        &mut cancel,
        &NoProgress,
        &CancelToken::new(),
    )
    .unwrap();
    assert_eq!(none, None);
    assert!(!cancel.copy_started);
}

#[test]
fn texture_listing_reports_folders_disabled_packs_and_conflicts() {
    let work = temp("yp_tex_list_");
    let game = GamePaths::new(work.path());
    let inject = game.inject_textures_dir();
    put_bytes(&inject, "PackA/shared.dds", 4, 0);
    put_bytes(&inject, "PackA/only_a.dds", 4, 0);
    put_bytes(&inject, "PackB/sub/SHARED.dds", 4, 0);
    put_bytes(&inject, "PackC/shared.dds", 4, 0);
    put_bytes(&inject, "Off/shared.dds", 4, 0);
    put_bytes(&inject, "loose.dds", 4, 0);
    put_bytes(
        &game.special_k_textures_dir().join("NieRAutomata.exe"),
        "SkPack/a.dds",
        4,
        0,
    );
    put_bytes(&game.wax_mods_dir(), "!base/a.dds", 4, 0);
    put_bytes(&game.wax_mods_dir(), "WaxTex/t/a.dds", 4, 0);
    put_bytes(&game.wax_mods_dir(), "WaxNoTex/a.json", 4, 0);

    let listing = load_textures(&TextureRoots::for_game(&game), &["Off".to_owned()]);

    assert_eq!(listing.nams, vec!["PackA", "PackB", "PackC", "loose.dds"]);
    assert_eq!(listing.nams_folders, vec!["PackA", "PackB", "PackC"]);
    assert_eq!(listing.sk, vec!["SkPack"]);
    assert_eq!(listing.wax, vec!["WaxTex"]);
    assert_eq!(
        listing.conflicts.get("PackA").unwrap(),
        &vec!["PackB".to_owned(), "PackC".to_owned()]
    );
    assert_eq!(
        listing.conflicts.get("PackB").unwrap(),
        &vec!["PackA".to_owned(), "PackC".to_owned()]
    );
    assert_eq!(
        listing.conflicts.keys().cloned().collect::<Vec<_>>(),
        vec!["PackA", "PackB", "PackC"]
    );
}
