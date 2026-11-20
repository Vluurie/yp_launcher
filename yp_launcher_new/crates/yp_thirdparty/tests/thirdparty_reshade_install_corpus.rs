mod common;

use common::{corpus_available, discover_corpus, extract, fresh_game, shader_corpus_root, temp};
use yp_core::Os;
use yp_thirdparty::{classify, tree, GraphicsRuntime, ReShadeRuntime, ThirdPartyKind};

fn has_absolute_path_value(ini: &str) -> bool {
    yp_core::ini::IniDoc::parse(ini)
        .entries()
        .iter()
        .any(|(_, _, value)| !value.is_empty() && yp_thirdparty::reshade::is_absolute(value.trim()))
}

#[test]
#[ignore = "needs the shader corpus (YP_SHADER_CORPUS_DIR)"]
fn reshade_install_from_real_corpus_mods_keeps_ini_paths_valid() {
    assert!(corpus_available(), "shader corpus missing ({})", shader_corpus_root().display());
    let reshade = ReShadeRuntime::for_os(Os::Windows);
    let mut failures = Vec::new();
    let mut installed = 0usize;

    for archive in discover_corpus(&shader_corpus_root()) {
        let tmp = temp("yp_tp_corpus_");
        let extract_dir = tmp.path().join("extract");
        if !extract(&archive.path, &extract_dir) {
            continue;
        }
        let classification = classify(&extract_dir);
        if !reshade.can_install(&classification) {
            assert!(!matches!(
                classification.kind,
                ThirdPartyKind::ReShadeWholeInstall | ThirdPartyKind::ReShadePreset
            ));
            continue;
        }

        let fresh = fresh_game("yp_tp_corpus_game_", "disable_reshade_loading = true\n");
        let result = reshade.install(&fresh.game, &classification).unwrap();
        installed += 1;
        let label = format!("{}/{}", archive.mod_id, archive.basename);
        if !result.is_ok() {
            failures.push(format!("{label}: install failed"));
            continue;
        }

        let dir = reshade.install_dir(&fresh.game);
        if !tree::is_within(&yp_thirdparty::paths::reshade(&fresh.game), &dir) {
            failures.push(format!("{label}: installed outside nams/thirdparty/reshade"));
        }
        if let Ok(ini) = std::fs::read_to_string(dir.join("ReShade.ini"))
            && has_absolute_path_value(&ini)
        {
            failures.push(format!("{label}: absolute machine path survived in ReShade.ini"));
        }
        if classification.has_shaders {
            if result.report().is_some_and(|report| report.shaders_missing) {
                failures.push(format!("{label}: pack shipped shaders but they are missing after install"));
            }
            if !dir.join("reshade-shaders").join("Shaders").is_dir() {
                failures.push(format!("{label}: shaders not relocated into the canonical folder"));
            }
        }
    }

    eprintln!("installed {installed} ReShade packs from the corpus");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
