mod support;

use support::{archive_of, put_bytes, staging, temp};
use yp_mods::textures::{dir_contains_dds, drop_contains_dds};

#[test]
fn folders_are_searched_recursively_and_case_insensitively() {
    let dir = temp("yp_tex_check_dir_");
    let nested = dir.path().join("nested");
    put_bytes(&nested, "SK_Res/inject/textures/04AD8A8F.DDS", 4, 0);
    let plain = dir.path().join("plain");
    put_bytes(&plain, "docs/readme.txt", 4, 0);
    put_bytes(&plain, "almost.dds.bak", 4, 0);
    std::fs::create_dir_all(plain.join("folder.dds")).unwrap();

    assert!(dir_contains_dds(&nested));
    assert!(!dir_contains_dds(&plain));
    assert!(!dir_contains_dds(&dir.path().join("missing")));
    assert!(!dir_contains_dds(
        &nested.join("SK_Res").join("missing.dds")
    ));
}

#[test]
fn a_drop_counts_when_any_path_holds_a_texture() {
    let dir = temp("yp_tex_check_drop_");
    let texture = put_bytes(dir.path(), "loose/Skin.DDS", 4, 0);
    let readme = put_bytes(dir.path(), "loose/readme.txt", 4, 0);
    let pack = dir.path().join("pack");
    put_bytes(&pack, "2B/a.dds", 4, 0);
    let empty = dir.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let missing = dir.path().join("missing.dds");
    let staging = staging();

    assert!(drop_contains_dds(&staging, std::slice::from_ref(&texture)));
    assert!(drop_contains_dds(&staging, std::slice::from_ref(&pack)));
    assert!(drop_contains_dds(
        &staging,
        &[readme.clone(), empty.clone(), pack]
    ));
    assert!(!drop_contains_dds(&staging, &[readme, empty, missing]));
    assert!(!drop_contains_dds(&staging, &[]));
}

#[test]
fn archives_are_checked_through_their_listing() {
    let dir = temp("yp_tex_check_zip_");
    let broken = put_bytes(dir.path(), "broken.7z", 8, 9);
    assert!(!drop_contains_dds(
        &staging(),
        std::slice::from_ref(&broken)
    ));
    for extension in ["zip", "7z", "rar"] {
        let with_textures = archive_of(
            dir.path(),
            &format!("HD Pack.{extension}"),
            &[
                ("SK_Res/inject/textures/04ad8a8f.DDS", &[7; 16]),
                ("Read me.txt", &[1; 4]),
            ],
        );
        let without_textures = archive_of(
            dir.path(),
            &format!("Mod.{extension}"),
            &[("data/pl/pl000d.dat", &[2; 8])],
        );

        assert!(
            drop_contains_dds(&staging(), std::slice::from_ref(&with_textures)),
            "{extension}"
        );
        assert!(
            !drop_contains_dds(&staging(), std::slice::from_ref(&without_textures)),
            "{extension}"
        );
        assert!(
            drop_contains_dds(
                &staging(),
                &[broken.clone(), without_textures, with_textures]
            ),
            "{extension}"
        );
    }
}
