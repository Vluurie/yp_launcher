mod support;

use support::{detect, file_names, fixture, install_named, mod_dir, temp};
use tempfile::TempDir;
use yp_mods::ModKind;

fn make_loose_data_dir_drop(prefix: &str, dir_name: &str) -> TempDir {
    let dir = temp(prefix);
    let model_dir = dir.path().join(dir_name).join("model");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("0315.dat"), [0u8; 32]).unwrap();
    std::fs::write(model_dir.join("0315.dtt"), [0u8; 32]).unwrap();
    dir
}

#[test]
fn bare_root_level_data_dirs_install_as_data_mods() {
    for dir_name in ["effect", "enlighten", "misctex", "quest"] {
        let drop = make_loose_data_dir_drop(&format!("yp_{dir_name}_"), dir_name);
        let game_dir = temp(&format!("yp_{dir_name}_g_"));

        let detected = detect(drop.path());
        assert_eq!(
            detected.kind,
            ModKind::Data,
            "{dir_name}/ drop rejected: {:?}",
            detected.rejection
        );

        let installed =
            install_named(game_dir.path(), drop.path(), &format!("{dir_name}_mod")).unwrap();
        let moved = mod_dir(game_dir.path(), &installed.id)
            .join("data")
            .join(dir_name)
            .join("model");
        assert!(
            moved.is_dir(),
            "loose {dir_name}/ must be normalized under data/"
        );
    }
}

#[test]
fn fixed_sand_footsteps_detects_as_a_data_mod_and_installs_effect_models() {
    let archive = fixture("sand_footsteps_359.7z");
    if !archive.is_file() {
        eprintln!("skipped: fixture not present at {}", archive.display());
        return;
    }
    let game_dir = temp("yp_sand_");

    let detected = detect(&archive);
    assert_eq!(
        detected.kind,
        ModKind::Data,
        "sand footsteps rejected: {:?}",
        detected.rejection
    );
    assert_eq!(detected.rejection, None);

    let installed = install_named(game_dir.path(), &archive, "fixed_sand_footsteps").unwrap();
    let model_dir = mod_dir(game_dir.path(), &installed.id)
        .join("data")
        .join("effect")
        .join("model");
    assert!(
        model_dir.is_dir(),
        "effect/model/ not installed under data/"
    );
    let names = file_names(&model_dir);
    assert!(names.contains(&"0315.dat".to_owned()));
    assert!(names.contains(&"0315.dtt".to_owned()));
}
