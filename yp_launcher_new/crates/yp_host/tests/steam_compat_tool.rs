mod support;

use support::FakeSteamTree;
use yp_core::wine_paths::{posix, NIER_STEAM_APP_ID};
use yp_host::vdf::parse;
use yp_host::wine::compat_tool::{find_steam_configured_proton, official_proton_tool_name, steam_compat_tool_name};

fn steam_config(mapping: &[(&str, &str)]) -> String {
    let entries: String = mapping
        .iter()
        .map(|(app, name)| {
            format!(
                "\t\t\t\t\t\"{app}\"\n\t\t\t\t\t{{\n\t\t\t\t\t\t\"name\"\t\t\"{name}\"\n\t\t\t\t\t\t\"config\"\t\t\"\"\n\t\t\t\t\t\t\"priority\"\t\t\"250\"\n\t\t\t\t\t}}\n"
            )
        })
        .collect();
    format!(
        "\"InstallConfigStore\"\n{{\n\t\"Software\"\n\t{{\n\t\t\"valve\"\n\t\t{{\n\t\t\t\"Steam\"\n\t\t\t{{\n\t\t\t\t\"CompatToolMapping\"\n\t\t\t\t{{\n{entries}\t\t\t\t}}\n\t\t\t}}\n\t\t}}\n\t}}\n}}\n"
    )
}

fn ge_manifest(internal_name: &str) -> String {
    format!(
        "\"compatibilitytools\"\n{{\n  \"compat_tools\"\n  {{\n    \"{internal_name}\" // Internal name of this tool\n    {{\n      // - The tool can be placed as a subdirectory in compatibilitytools.d, in which case this\n      //   should be '.'\n      \"install_path\" \".\"\n\n      \"display_name\" \"{internal_name}\"\n      \"from_oslist\"  \"windows\"\n      \"to_oslist\"    \"linux\"\n    }}\n  }}\n}}\n"
    )
}

#[test]
fn maps_valve_folder_names_onto_the_names_steam_stores() {
    for (folder, expected) in [
        ("Proton - Experimental", "proton_experimental"),
        ("Proton Hotfix", "proton_hotfix"),
        ("Proton 9.0 (Beta)", "proton_9"),
        ("Proton 10.0", "proton_10"),
        ("Proton 6.3", "proton_63"),
        ("Proton 5.13", "proton_513"),
    ] {
        assert_eq!(official_proton_tool_name(folder).as_deref(), Some(expected), "{folder}");
    }
}

#[test]
fn ignores_runtimes_and_other_apps() {
    assert_eq!(official_proton_tool_name("Proton EasyAntiCheat Runtime"), None);
    assert_eq!(official_proton_tool_name("Steam Linux Runtime 3.0 (sniper)"), None);
    assert_eq!(official_proton_tool_name("NieRAutomata"), None);
}

#[test]
fn reads_the_per_game_and_the_default_entry() {
    let config = parse(&steam_config(&[("0", "proton_experimental"), (NIER_STEAM_APP_ID, "proton_9")])).unwrap();
    assert_eq!(steam_compat_tool_name(&config, NIER_STEAM_APP_ID).as_deref(), Some("proton_9"));
    assert_eq!(steam_compat_tool_name(&config, "0").as_deref(), Some("proton_experimental"));
}

#[test]
fn tool_name_is_none_when_the_game_has_no_entry() {
    let config = parse(&steam_config(&[("0", "proton_experimental")])).unwrap();
    assert_eq!(steam_compat_tool_name(&config, NIER_STEAM_APP_ID), None);
}

#[test]
fn uses_the_proton_steam_runs_nier_with_not_the_newest_install() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    tree.add_steam_config(&steam_config(&[("0", "proton_experimental"), (NIER_STEAM_APP_ID, "proton_9")]));
    let nine = tree.add_official_proton("Proton 9.0 (Beta)");
    tree.add_official_proton("Proton - Experimental");
    assert_eq!(find_steam_configured_proton(&tree.native(), &root, &root), Some(nine));
}

#[test]
fn resolves_a_custom_tool_by_the_name_in_its_manifest() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    tree.add_steam_config(&steam_config(&[(NIER_STEAM_APP_ID, "GE-Proton9-20")]));
    let ge = tree.add_compat_tool("ge-folder", &ge_manifest("GE-Proton9-20"));
    assert_eq!(find_steam_configured_proton(&tree.native(), &root, &root), Some(posix::normalize(&ge)));
}

#[test]
fn falls_back_to_the_default_tool_when_the_game_tool_is_missing() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    tree.add_steam_config(&steam_config(&[("0", "proton_experimental"), (NIER_STEAM_APP_ID, "proton_8")]));
    let experimental = tree.add_official_proton("Proton - Experimental");
    assert_eq!(find_steam_configured_proton(&tree.native(), &root, &root), Some(experimental));
}

#[test]
fn is_none_without_a_steam_config() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    tree.add_official_proton("Proton 9.0");
    assert_eq!(find_steam_configured_proton(&tree.native(), &root, &root), None);
}

#[test]
fn honours_extra_compat_tool_paths_from_the_environment() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    tree.add_steam_config(&steam_config(&[(NIER_STEAM_APP_ID, "Custom-Tool")]));
    let scratch = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let extra = support::without_drive(&posix::join(&yp_core::wine_paths::to_posix(scratch.path()), "extra-tools"));
    let tool_dir = posix::join(&extra, "custom");
    support::write(&posix::join_all(&tool_dir, &["dist", "proton"]), "#!/bin/sh\n");
    support::write(
        &posix::join(&tool_dir, "compatibilitytool.vdf"),
        "\"compatibilitytools\" { \"compat_tools\" { \"Custom-Tool\" { \"install_path\" \"dist\" } } }",
    );
    let host = support::host_for(
        yp_core::Os::Linux,
        &[("HOME", &tree.home), ("STEAM_EXTRA_COMPAT_TOOLS_PATHS", &extra)],
        &tree.home,
    );
    let ctx = yp_host::wine::WineContext::isolated(host).with_steam_roots(vec![root.clone()]);
    assert_eq!(
        find_steam_configured_proton(&ctx, &root, &root),
        Some(posix::join_all(&tool_dir, &["dist", "proton"]))
    );
}
