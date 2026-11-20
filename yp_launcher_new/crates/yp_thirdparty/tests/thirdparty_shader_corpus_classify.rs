mod common;

use std::collections::BTreeMap;

use common::{corpus_available, discover_corpus, extract, shader_corpus_root, temp};
use yp_thirdparty::{classify, ThirdPartyKind};

#[test]
#[ignore = "needs the shader corpus (YP_SHADER_CORPUS_DIR)"]
fn most_real_shader_archives_classify_as_an_installable_runtime() {
    assert!(corpus_available(), "shader corpus missing ({})", shader_corpus_root().display());
    let archives = discover_corpus(&shader_corpus_root());
    let tmp = temp("yp_shader_classify_");
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut supported = 0usize;
    let mut routed = 0usize;
    let mut unsupported = Vec::new();

    for archive in &archives {
        let out = tmp.path().join(archive.unique_id());
        if !extract(&archive.path, &out) {
            *by_kind.entry(format!("{:?}", ThirdPartyKind::Unknown)).or_default() += 1;
            unsupported.push(format!("{}/{}  (extract failed)", archive.mod_id, archive.basename));
            yp_core::fs::remove_dir_quiet(&out);
            continue;
        }
        let kind = std::panic::catch_unwind(|| classify(&out).kind);
        match kind {
            Ok(kind) => {
                *by_kind.entry(format!("{kind:?}")).or_default() += 1;
                if kind.is_installable() {
                    supported += 1;
                } else if kind.belongs_to_other_tab() {
                    routed += 1;
                } else {
                    unsupported.push(format!("{}/{}  -> {kind:?}", archive.mod_id, archive.basename));
                }
            }
            Err(_) => unsupported.push(format!("{}/{}  (classify panicked)", archive.mod_id, archive.basename)),
        }
        yp_core::fs::remove_dir_quiet(&out);
    }

    let total = archives.len();
    let mut report = format!(
        "Shader corpus classification over {total} archives:\n  installable: {supported}\n  routed to another tab: {routed}\n  UNSUPPORTED: {}\n",
        unsupported.len()
    );
    for (kind, count) in &by_kind {
        report.push_str(&format!("    {kind:<20} {count}\n"));
    }
    for line in &unsupported {
        report.push_str(&format!("    {line}\n"));
    }
    eprintln!("{report}");

    assert!(total > 0, "no shader archives discovered");
    assert!(
        !unsupported.iter().any(|line| line.contains("classify panicked")),
        "classify() must never panic on a real archive\n{report}"
    );
    let handled_ratio = (supported + routed) as f64 / total as f64;
    assert!(handled_ratio >= 0.75, "at least 75% of shader archives should be handled\n{report}");
}
