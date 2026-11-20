use yp_mods::detect::is_texture_only_listing;

#[test]
fn a_pack_of_dds_files_with_a_readme_is_a_texture_pack() {
    assert!(is_texture_only_listing([
        "Read me.txt",
        "SK_Res",
        r"SK_Res\desktop.ini",
        r"SK_Res\inject",
        r"SK_Res\inject\textures",
        r"SK_Res\inject\textures\04ad8a8f.dds",
        r"SK_Res\inject\textures\0a2b11c3.dds",
    ]));
}

#[test]
fn dds_next_to_game_files_is_a_mod() {
    assert!(!is_texture_only_listing([
        "Kimono/data/pl/pl000d.dat",
        "Kimono/data/pl/pl000d.dtt",
        "Kimono/textures/pl000d_0.dds",
    ]));
}

#[test]
fn dds_next_to_a_cpk_is_a_mod() {
    assert!(!is_texture_only_listing(["data100.cpk", "textures/a.dds"]));
}

#[test]
fn dds_next_to_mod_toml_entities_or_wax_is_a_mod() {
    assert!(!is_texture_only_listing(["mod.toml", "textures/a.dds"]));
    assert!(!is_texture_only_listing([
        "Sword/entities/weapons/sword.toml",
        "Sword/textures/a.dds",
    ]));
    assert!(!is_texture_only_listing([
        "Dress/wax/mods/Dress/outfits/dress.json",
        "Dress/textures/a.dds",
    ]));
}

#[test]
fn an_archive_without_dds_is_not_a_texture_pack() {
    assert!(!is_texture_only_listing(["readme.txt"]));
    assert!(!is_texture_only_listing(Vec::<String>::new()));
}
