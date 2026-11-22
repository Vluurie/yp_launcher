mod common;

use std::collections::BTreeSet;
use std::path::Path;

use common::{corpus_available, discover_corpus, extract, find_archive, fresh_game, nams_flag, shader_corpus_root, temp, write_bytes, FreshGame};
use yp_core::consts::{GAME_EXE_NAME, MIGOTO_DIR_NAME, NAMS_EXE_NAME, THIRD_PARTY_DIR_NAME};
use yp_core::ini::IniDoc;
use yp_thirdparty::{classify, tree, Classification, GraphicsRuntime, LoadingFlag, MigotoRuntime, ThirdPartyKind};

fn migoto() -> MigotoRuntime {
    MigotoRuntime::for_wine(false)
}

fn game() -> FreshGame {
    fresh_game("yp_migoto_", "disable_3dmigoto_loading = true\n")
}

fn loader_target(d3dx: &str) -> Option<String> {
    IniDoc::parse(d3dx).get(Some("Loader"), "target").map(str::to_owned)
}

fn classify_archive(mod_id_part: &str, into: &Path) -> Option<Classification> {
    let all = discover_corpus(&shader_corpus_root());
    let archive = find_archive(&all, mod_id_part)?;
    let out = into.join("x");
    extract(&archive.path, &out);
    Some(classify(&out))
}

fn shader_fix_names(dir: &Path) -> BTreeSet<String> {
    common::files_under(&dir.join("ShaderFixes"))
        .iter()
        .map(|file| tree::file_name(file))
        .collect()
}

#[test]
fn loader_target_defaults_to_nams_the_host_not_the_game_exe() {
    assert_eq!(migoto().loader_target(), NAMS_EXE_NAME);
    assert_ne!(migoto().loader_target(), GAME_EXE_NAME);
}

#[test]
fn files_differ_same_content_false_changed_byte_true_missing_true() {
    let dir = temp("yp_diff_");
    let a = dir.path().join("a.bin");
    let b = dir.path().join("b.bin");
    write_bytes(&a, &[1, 2, 3, 4]);
    write_bytes(&b, &[1, 2, 3, 4]);
    assert!(!tree::files_differ(&a, &b));

    write_bytes(&b, &[1, 2, 3, 5]);
    assert!(tree::files_differ(&a, &b), "same length, different content");

    write_bytes(&b, &[1, 2, 3, 4, 5]);
    assert!(tree::files_differ(&a, &b), "different length");

    assert!(tree::files_differ(&a, &dir.path().join("nope.bin")), "missing counterpart counts as differing");
}

#[test]
fn a_synthetic_pack_installs_with_nams_as_loader_target() {
    let fresh = game();
    let drop = temp("yp_migoto_pack_");
    write_bytes(&drop.path().join("pack").join("d3d11.dll"), b"MZ HackerDevice");
    common::write(&drop.path().join("pack").join("d3dx.ini"), "[Loader]\n;target = NieRAutomata.exe\nmodule = d3d11.dll\n");
    common::write(&drop.path().join("pack").join("ShaderFixes").join("abc-ps_replace.txt"), "fix");

    let c = classify(drop.path()).named("Pack");
    assert_eq!(c.kind, ThirdPartyKind::Migoto);
    assert!(migoto().install(&fresh.game, &c).unwrap().is_ok());

    let dest = migoto().install_dir(&fresh.game);
    assert!(dest.join("3dmigoto.dll").is_file());
    assert!(!dest.join("d3d11.dll").exists());
    let d3dx = std::fs::read_to_string(dest.join("d3dx.ini")).unwrap();
    assert_eq!(loader_target(&d3dx).as_deref(), Some(NAMS_EXE_NAME));
    assert_eq!(IniDoc::parse(&d3dx).get(Some("System"), "hook"), Some("recommended"));
    let status = migoto().status(&fresh.game);
    let info = status.migoto_info.unwrap();
    assert!(info.loader_target_ok);
    assert_eq!(info.shader_fix_names, ["Pack"]);
    assert_eq!(info.shader_fix_count, 1);
    assert_eq!(nams_flag(&fresh.game, LoadingFlag::Migoto.key()), Some(false));
}

#[test]
#[ignore = "needs the shader corpus (YP_SHADER_CORPUS_DIR)"]
fn bande_desinee_without_a_loader_section_gets_target_nams() {
    assert!(corpus_available());
    let src = temp("yp_src_");
    let fresh = game();
    let Some(c) = classify_archive("Bande_Desinee", src.path()) else {
        return;
    };
    assert_eq!(c.kind, ThirdPartyKind::Migoto);
    assert!(migoto().install(&fresh.game, &c).unwrap().is_ok());

    let dest = migoto().install_dir(&fresh.game);
    assert!(dest.join("3dmigoto.dll").is_file());
    assert!(!dest.join("d3d11.dll").exists(), "the loader dll is renamed so NAMS owns d3d11.dll");
    assert!(dest.join("ShaderFixes").is_dir());
    assert!(
        tree::is_within(&fresh.game.nams_dir().join(THIRD_PARTY_DIR_NAME).join(MIGOTO_DIR_NAME), &dest),
        "the 3DMigoto mod folder lives in <game>/nams/thirdparty, which a launcher update does not replace"
    );

    let d3dx = std::fs::read_to_string(dest.join("d3dx.ini")).unwrap();
    assert_eq!(loader_target(&d3dx).as_deref(), Some(NAMS_EXE_NAME), "target added even when [Loader] was absent");
    assert_eq!(nams_flag(&fresh.game, LoadingFlag::Migoto.key()), Some(false));
}

#[test]
#[ignore = "needs the shader corpus (YP_SHADER_CORPUS_DIR)"]
fn bloom_fix_with_a_commented_target_gets_target_nams() {
    assert!(corpus_available());
    let src = temp("yp_src_");
    let fresh = game();
    let Some(c) = classify_archive("Bloom_Fix", src.path()) else {
        return;
    };
    assert_eq!(c.kind, ThirdPartyKind::Migoto);
    migoto().install(&fresh.game, &c).unwrap();

    let d3dx = std::fs::read_to_string(migoto().install_dir(&fresh.game).join("d3dx.ini")).unwrap();
    assert_eq!(loader_target(&d3dx).as_deref(), Some(NAMS_EXE_NAME));
}

#[test]
#[ignore = "needs the shader corpus (YP_SHADER_CORPUS_DIR)"]
fn two_migoto_mods_stack_shader_fixes_merge_not_overwrite() {
    assert!(corpus_available());
    let src1 = temp("yp_s1_");
    let src2 = temp("yp_s2_");
    let fresh = game();
    let (Some(c1), Some(c2)) = (classify_archive("Bande_Desinee", src1.path()), classify_archive("Bloom_Fix", src2.path())) else {
        return;
    };

    migoto().install(&fresh.game, &c1).unwrap();
    let dest = migoto().install_dir(&fresh.game);
    let after_first = shader_fix_names(&dest);

    migoto().install(&fresh.game, &c2).unwrap();
    let after_second = shader_fix_names(&dest);

    assert!(after_second.is_superset(&after_first), "first mod ShaderFixes still present after second install");
    assert!(after_second.len() >= after_first.len(), "merge, not overwrite");
}

#[test]
#[ignore = "needs the shader corpus (YP_SHADER_CORPUS_DIR)"]
fn would_update_is_none_when_nothing_is_installed_yet() {
    assert!(corpus_available());
    let src = temp("yp_src_");
    let fresh = game();
    let Some(c) = classify_archive("Bande_Desinee", src.path()) else {
        return;
    };
    assert_eq!(migoto().would_update(&fresh.game, &c), None, "no installed dll to compare against");
}

#[test]
#[ignore = "needs the shader corpus (YP_SHADER_CORPUS_DIR)"]
fn shader_fixes_get_the_dropped_mod_name_shown_in_status() {
    assert!(corpus_available());
    let src = temp("yp_src_");
    let fresh = game();
    let Some(c) = classify_archive("Bande_Desinee", src.path()) else {
        return;
    };
    migoto().install(&fresh.game, &c.named("Bande Desinee")).unwrap();

    let names = migoto().status(&fresh.game).migoto_info.unwrap().shader_fix_names;
    assert!(names.iter().any(|name| name == "Bande Desinee"), "hash-named ShaderFixes surface under the mod name");
    assert!(!names.iter().any(|name| name.contains("-ps_replace")), "raw hash filenames are hidden once named");
}
