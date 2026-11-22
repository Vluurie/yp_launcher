mod support;

use std::path::{Path, PathBuf};

use support::{dir_names, rel_slash, staging, temp, walk_files};
use yp_core::CancelToken;
use yp_mods::textures::{TextureCopyProgress, TextureSource, install_textures};

fn pack_with(root: &Path, name: &str, dds_files: &[&str]) -> PathBuf {
    let dir = root.join(name);
    for file in dds_files {
        let path = file
            .split('/')
            .fold(dir.clone(), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, [0u8; 8]).unwrap();
    }
    dir
}

fn install(sources: &[TextureSource], texture_dir: &Path) -> usize {
    let mut reports: Vec<TextureCopyProgress> = Vec::new();
    let count = install_textures(
        &staging(),
        sources,
        texture_dir,
        &mut |progress| reports.push(progress),
        &CancelToken::new(),
    )
    .unwrap();
    if count > 0 {
        let last = reports.last().unwrap();
        assert_eq!(last.files, count);
        assert_eq!(last.copied, last.total);
    }
    count
}

fn rel_dds_paths(root: &Path) -> Vec<String> {
    let mut out: Vec<String> = walk_files(root)
        .into_iter()
        .filter(|file| file.to_string_lossy().to_lowercase().ends_with(".dds"))
        .map(|file| rel_slash(root, &file))
        .collect();
    out.sort();
    out
}

struct Layout {
    _tmp: tempfile::TempDir,
    src: PathBuf,
    texture_dir: PathBuf,
}

fn layout() -> Layout {
    let tmp = temp("yp_tex_");
    let src = tmp.path().join("src");
    let texture_dir = tmp.path().join("inject");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::create_dir_all(&texture_dir).unwrap();
    Layout {
        _tmp: tmp,
        src,
        texture_dir,
    }
}

#[test]
fn picks_sharing_a_pack_name_merge_into_one_folder() {
    let fx = layout();
    let a = pack_with(&fx.src, "2B", &["a.dds"]);
    let b = pack_with(&fx.src, "A2", &["b.dds"]);
    let c = pack_with(&fx.src, "Desert", &["c.dds"]);

    let count = install(
        &[
            TextureSource::into_folder(a, "HD Pack"),
            TextureSource::into_folder(b, "HD Pack"),
            TextureSource::into_folder(c, "HD Pack"),
        ],
        &fx.texture_dir,
    );

    assert_eq!(count, 3);
    assert_eq!(dir_names(&fx.texture_dir), vec!["HD Pack"]);
    assert_eq!(
        rel_dds_paths(&fx.texture_dir),
        vec!["HD Pack/a.dds", "HD Pack/b.dds", "HD Pack/c.dds"]
    );
}

#[test]
fn distinct_pack_names_stay_separate_folders() {
    let fx = layout();
    let a = pack_with(&fx.src, "2B", &["a.dds"]);
    let b = pack_with(&fx.src, "A2", &["b.dds"]);

    install(
        &[
            TextureSource::into_folder(a, "2B"),
            TextureSource::into_folder(b, "A2"),
        ],
        &fx.texture_dir,
    );

    assert_eq!(dir_names(&fx.texture_dir), vec!["2B", "A2"]);
}

#[test]
fn merging_keeps_each_picks_subfolder_structure() {
    let fx = layout();
    let a = pack_with(&fx.src, "City", &["area/city.dds"]);
    let b = pack_with(&fx.src, "Desert", &["area/desert.dds"]);

    install(
        &[
            TextureSource::into_folder(a, "HD Pack"),
            TextureSource::into_folder(b, "HD Pack"),
        ],
        &fx.texture_dir,
    );

    assert_eq!(
        rel_dds_paths(&fx.texture_dir),
        vec!["HD Pack/area/city.dds", "HD Pack/area/desert.dds"]
    );
}

#[test]
fn sk_res_inject_textures_wrappers_are_stripped_when_merging() {
    let fx = layout();
    let a = pack_with(&fx.src, "2B", &["SK_Res/inject/textures/a.dds"]);

    install(&[TextureSource::into_folder(a, "HD Pack")], &fx.texture_dir);

    assert_eq!(rel_dds_paths(&fx.texture_dir), vec!["HD Pack/a.dds"]);
}

#[test]
fn a_folder_without_a_name_uses_its_own_name_and_loose_dds_land_at_the_root() {
    let fx = layout();
    let a = pack_with(&fx.src, "Mine", &["x.dds", "notes.txt"]);
    let loose = fx.src.join("loose.DDS");
    std::fs::write(&loose, [1u8; 4]).unwrap();

    let count = install(
        &[TextureSource::new(a), TextureSource::new(loose)],
        &fx.texture_dir,
    );

    assert_eq!(count, 2);
    assert_eq!(
        rel_dds_paths(&fx.texture_dir),
        vec!["Mine/x.dds", "loose.DDS"]
    );
}

#[test]
fn nothing_to_copy_reports_zero() {
    let fx = layout();
    let empty = fx.src.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    assert_eq!(install(&[TextureSource::new(empty)], &fx.texture_dir), 0);
}

#[test]
fn installing_an_installed_pack_onto_itself_keeps_its_textures() {
    let root = temp("self-install");
    let texture_dir = root.path().join("nams").join("inject").join("textures");
    let pack = texture_dir.join("Pack A");
    let body = b"DDS texture body";
    for file in ["a.dds", "nested/b.dds"] {
        let path = file.split('/').fold(pack.clone(), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
    let count = install(
        &[TextureSource {
            path: pack.clone(),
            folder: Some("Pack A".to_owned()),
        }],
        &texture_dir,
    );
    assert_eq!(count, 2);
    assert_eq!(std::fs::read(pack.join("a.dds")).unwrap(), body);
    assert_eq!(std::fs::read(pack.join("nested").join("b.dds")).unwrap(), body);
    assert_eq!(walk_files(&pack).len(), 2);
}
