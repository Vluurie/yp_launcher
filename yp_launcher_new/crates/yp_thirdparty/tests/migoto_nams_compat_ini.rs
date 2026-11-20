use yp_core::ini::IniDoc;
use yp_thirdparty::migoto::{patch_loader_target, patch_nams_compat};

fn system_key(ini: &str, key: &str) -> Option<String> {
    IniDoc::parse(ini).get(Some("System"), key).map(str::to_owned)
}

fn active_key_count(ini: &str, key: &str) -> usize {
    IniDoc::parse(ini).entries().iter().filter(|(_, entry_key, _)| *entry_key == key).count()
}

#[test]
fn an_empty_d3dx_ini_gets_the_full_nams_compat_set() {
    let out = patch_nams_compat("");

    assert_eq!(system_key(&out, "hook").as_deref(), Some("recommended"));
    assert_eq!(system_key(&out, "load_library_redirect").as_deref(), Some("1"));
    assert_eq!(system_key(&out, "allow_check_interface").as_deref(), Some("1"));
    assert_eq!(system_key(&out, "allow_create_device").as_deref(), Some("1"));
    assert_eq!(system_key(&out, "allow_platform_update").as_deref(), Some("1"));
}

#[test]
fn values_a_mod_shipped_are_overwritten_not_appended() {
    let shipped = "\n[System]\nhook = 0\nload_library_redirect = 2\nallow_create_device = 0\n";
    let out = patch_nams_compat(shipped);

    assert_eq!(system_key(&out, "hook").as_deref(), Some("recommended"));
    assert_eq!(system_key(&out, "load_library_redirect").as_deref(), Some("1"));
    assert_eq!(system_key(&out, "allow_create_device").as_deref(), Some("1"));
    assert_eq!(active_key_count(&out, "hook"), 1);
    assert_eq!(active_key_count(&out, "load_library_redirect"), 1);
}

#[test]
fn other_sections_a_mod_ships_are_left_alone() {
    let shipped = "\n[System]\nhook = 0\n\n[Rendering]\noverride_directory = ShaderFixes\n\n[Hunting]\nhunting = 0\n";
    let out = patch_nams_compat(shipped);
    let doc = IniDoc::parse(&out);

    assert_eq!(doc.get(Some("Rendering"), "override_directory"), Some("ShaderFixes"));
    assert_eq!(doc.get(Some("Hunting"), "hunting"), Some("0"));
    assert_eq!(system_key(&out, "hook").as_deref(), Some("recommended"));
}

#[test]
fn the_loader_target_is_nams_not_the_game() {
    let out = patch_loader_target("", yp_core::consts::NAMS_EXE_NAME);
    assert_eq!(out, "[Loader]\ntarget = NAMS.exe\n");

    let without_loader = patch_loader_target("[System]\nhook = 0\n", "NAMS.exe");
    let doc = IniDoc::parse(&without_loader);
    assert_eq!(doc.sections()[0], "Loader");
    assert_eq!(doc.get(Some("Loader"), "target"), Some("NAMS.exe"));
    assert_eq!(doc.get(Some("System"), "hook"), Some("0"));

    let game_target = patch_loader_target("[Loader]\ntarget = NieRAutomata.exe\nmodule = d3d11.dll\n", "NAMS.exe");
    let doc = IniDoc::parse(&game_target);
    assert_eq!(doc.get(Some("loader"), "TARGET"), Some("NAMS.exe"));
    assert_eq!(active_key_count(&game_target, "target"), 1);

    let commented = patch_loader_target("[Loader]\n;target = NieRAutomata.exe\nmodule = d3d11.dll\n", "NAMS.exe");
    assert_eq!(IniDoc::parse(&commented).get(Some("Loader"), "target"), Some("NAMS.exe"));
    assert_eq!(active_key_count(&commented, "target"), 1);
}
