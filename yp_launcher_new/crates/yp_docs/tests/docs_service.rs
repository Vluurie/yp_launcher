use yp_docs::{DocPage, anchor_for, anchor_in, outline, pretty_name, resolve_link};

#[test]
fn anchor_for_slugifies_headings_the_way_markdown_anchors_do() {
    assert_eq!(anchor_for("Getting started"), "getting-started");
    assert_eq!(anchor_for("What you need first"), "what-you-need-first");
    assert_eq!(
        anchor_for("Step 1 - Make a folder"),
        "step-1---make-a-folder"
    );
}

#[test]
fn anchor_for_drops_punctuation_and_collapses_spacing() {
    assert_eq!(anchor_for("Why? Because!"), "why-because");
    assert_eq!(anchor_for("  spaced   out  "), "spaced-out");
}

#[test]
fn anchor_for_never_starts_with_a_dash() {
    assert_eq!(anchor_for("  leading"), "leading");
}

#[test]
fn anchor_for_keeps_non_ascii_letters() {
    assert_eq!(anchor_for("Über Mods"), "über-mods");
    assert_eq!(anchor_for("模组 文件夹"), "模组-文件夹");
}

#[test]
fn outline_collects_h2_and_h3_but_ignores_h1_and_fenced_code() {
    let markdown = "
# Title

## First

```
## not a heading
```

### Nested

#### Too deep

## Second
";
    let headings = outline(markdown);
    let texts: Vec<&str> = headings
        .iter()
        .map(|heading| heading.text.as_str())
        .collect();
    let levels: Vec<u8> = headings.iter().map(|heading| heading.level).collect();
    assert_eq!(texts, vec!["First", "Nested", "Second"]);
    assert_eq!(levels, vec![2, 3, 2]);
    assert_eq!(headings[0].anchor, "first");
}

#[test]
fn outline_ignores_a_hash_run_with_no_space_after_it() {
    let headings = outline("##NoSpace\n\n## Real\n");
    let texts: Vec<&str> = headings
        .iter()
        .map(|heading| heading.text.as_str())
        .collect();
    assert_eq!(texts, vec!["Real"]);
}

#[test]
fn outline_shows_inline_code_and_emphasis_as_plain_text() {
    let headings =
        outline("## How do mods in `thirdparty\\graphic\\` work?\n\n## A **bold** one\n");
    let texts: Vec<&str> = headings
        .iter()
        .map(|heading| heading.text.as_str())
        .collect();
    assert_eq!(
        texts,
        vec!["How do mods in thirdparty\\graphic\\ work?", "A bold one"]
    );
    assert_eq!(headings[0].anchor, "how-do-mods-in-thirdpartygraphic-work");
}

#[test]
fn outline_numbers_repeated_headings_so_anchors_stay_unique() {
    let markdown =
        "## Attribute 0x400\n\ntext\n\n## Attribute 0x400\n\nmore\n\n## Attribute 0x400\n";
    let anchors: Vec<String> = outline(markdown)
        .into_iter()
        .map(|heading| heading.anchor)
        .collect();
    assert_eq!(
        anchors,
        vec!["attribute-0x400", "attribute-0x400-1", "attribute-0x400-2"]
    );
}

#[test]
fn outline_anchors_of_a_real_page_are_unique() {
    let markdown =
        "## Step 2 - Create `mod.toml`\n\n## `mod.toml` fields\n\n### Notes\n\n### Notes\n";
    let anchors: Vec<String> = outline(markdown)
        .into_iter()
        .map(|heading| heading.anchor)
        .collect();
    let unique: std::collections::BTreeSet<&String> = anchors.iter().collect();
    assert_eq!(unique.len(), anchors.len());
}

#[test]
fn outline_handles_crlf_and_links() {
    let headings = outline("## See [the guide](01_first_mod.md)\r\n\r\n### Done\r");
    let texts: Vec<&str> = headings
        .iter()
        .map(|heading| heading.text.as_str())
        .collect();
    assert_eq!(texts, vec!["See the guide", "Done"]);
}

#[test]
fn pretty_name_strips_the_numeric_prefix_and_extension() {
    assert_eq!(pretty_name("03_item.md"), "Item");
    assert_eq!(
        pretty_name("16_wax_to_toml_migration.md"),
        "Wax to toml migration"
    );
}

#[test]
fn pretty_name_leaves_names_without_a_numeric_prefix_intact() {
    assert_eq!(pretty_name("AreaAction.md"), "AreaAction");
    assert_eq!(pretty_name("_signals.md"), "Signals");
    assert_eq!(pretty_name(".md"), ".md");
}

#[test]
fn anchor_in_returns_the_fragment_when_present() {
    assert_eq!(anchor_in("07_quest.md#rewards"), Some("rewards"));
    assert_eq!(anchor_in("#top"), Some("top"));
}

#[test]
fn anchor_in_returns_none_without_a_usable_fragment() {
    assert_eq!(anchor_in("07_quest.md"), None);
    assert_eq!(anchor_in("07_quest.md#"), None);
}

const FROM: &str = "assets/docs/scripting/03_getting_started.md";

#[test]
fn resolve_link_resolves_a_sibling_page() {
    assert_eq!(
        resolve_link(None, FROM, "01_hap.md").as_deref(),
        Some("assets/docs/scripting/01_hap.md")
    );
}

#[test]
fn resolve_link_resolves_a_parent_relative_page() {
    assert_eq!(
        resolve_link(None, FROM, "../native/07_quest.md").as_deref(),
        Some("assets/docs/native/07_quest.md")
    );
}

#[test]
fn resolve_link_drops_the_fragment_when_resolving() {
    assert_eq!(
        resolve_link(None, FROM, "../native/07_quest.md#rewards").as_deref(),
        Some("assets/docs/native/07_quest.md")
    );
}

#[test]
fn resolve_link_returns_none_for_external_links() {
    assert_eq!(resolve_link(None, FROM, "https://example.com"), None);
    assert_eq!(resolve_link(None, FROM, "#local"), None);
}

fn page(path: &str) -> DocPage {
    DocPage {
        path: path.to_owned(),
        category: path.split('/').nth(2).unwrap().to_owned(),
        title: String::new(),
        file_name: path.rsplit('/').next().unwrap().to_owned(),
    }
}

#[test]
fn resolve_link_falls_back_to_a_page_with_the_same_file_name() {
    let pages = [
        page("assets/docs/native/07_quest.md"),
        page("assets/docs/scripting/01_hap.md"),
    ];
    assert_eq!(
        resolve_link(Some(&pages), FROM, "../native/07_quest.md").as_deref(),
        Some("assets/docs/native/07_quest.md")
    );
    assert_eq!(
        resolve_link(Some(&pages), FROM, "../elsewhere/07_quest.md").as_deref(),
        Some("assets/docs/native/07_quest.md")
    );
    assert_eq!(resolve_link(Some(&pages), FROM, "missing.md"), None);
}
