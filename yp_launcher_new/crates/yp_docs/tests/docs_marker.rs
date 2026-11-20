use yp_docs::DocsMarker;

#[test]
fn returns_none_when_there_is_no_marker() {
    assert_eq!(DocsMarker::first_in("# Title\n\nJust prose.\n"), None);
    assert_eq!(DocsMarker::first_in("<!-- just a comment -->\n"), None);
}

#[test]
fn reads_a_bare_marker() {
    let md = "before\n\n<!-- yp:item-builder -->\n\nafter";
    let marker = DocsMarker::first_in(md).unwrap();
    assert_eq!(marker.id, "item-builder");
    assert!(marker.args.is_empty());
    assert_eq!(md[..marker.start].trim(), "before");
    assert_eq!(md[marker.end..].trim(), "after");
}

#[test]
fn reads_arguments() {
    let md = "<!-- yp:figure src=\"assets/docs/images/a.jpg\" caption=\"Hello\" side=\"left\" -->";
    let marker = DocsMarker::first_in(md).unwrap();
    assert_eq!(marker.id, "figure");
    assert_eq!(marker.arg("src"), Some("assets/docs/images/a.jpg"));
    assert_eq!(marker.arg("caption"), Some("Hello"));
    assert_eq!(marker.arg("side"), Some("left"));
}

#[test]
fn keeps_spaces_and_punctuation_inside_a_quoted_value() {
    let md = "<!-- yp:figure caption=\"It is, in fact, fine.\" -->";
    assert_eq!(
        DocsMarker::first_in(md).unwrap().arg("caption"),
        Some("It is, in fact, fine.")
    );
}

#[test]
fn finds_the_first_of_several_markers() {
    let md = "a\n<!-- yp:first-mod -->\nb\n<!-- yp:figure src=\"x\" -->\nc";
    assert_eq!(DocsMarker::first_in(md).unwrap().id, "first-mod");
    let ids: Vec<String> = DocsMarker::all_in(md)
        .into_iter()
        .map(|marker| marker.id)
        .collect();
    assert_eq!(ids, vec!["first-mod", "figure"]);
}

#[test]
fn ignores_an_unterminated_marker() {
    assert_eq!(DocsMarker::first_in("<!-- yp:figure src=\"x\""), None);
}

#[test]
fn ignores_an_empty_marker() {
    assert_eq!(DocsMarker::first_in("<!-- yp: -->"), None);
}

#[test]
fn an_unquoted_argument_is_skipped_rather_than_misparsed() {
    let marker = DocsMarker::first_in("<!-- yp:figure src=x.jpg -->").unwrap();
    assert_eq!(marker.id, "figure");
    assert_eq!(marker.arg("src"), None);
}

#[test]
fn start_and_end_cover_the_whole_comment() {
    let md = "A<!-- yp:first-mod -->B";
    let marker = DocsMarker::first_in(md).unwrap();
    assert_eq!(&md[marker.start..marker.end], "<!-- yp:first-mod -->");
}

#[test]
fn a_marker_inside_a_code_block_is_documentation_not_a_marker() {
    let md = "```\n<!-- yp:first-mod -->\n```\n\n<!-- yp:item-builder -->\n";
    assert_eq!(DocsMarker::first_in(md).unwrap().id, "item-builder");
}

#[test]
fn a_marker_may_span_several_lines() {
    let md = "intro\n\n<!-- yp:figure src=\"a.jpg\"\n     caption=\"two lines\" -->\n\nrest";
    let marker = DocsMarker::first_in(md).unwrap();
    assert_eq!(marker.arg("caption"), Some("two lines"));
    assert_eq!(md[marker.end..].trim(), "rest");
}
