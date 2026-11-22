use std::path::{Path, PathBuf};

use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*};

use super::*;
use crate::syntax::{Language, TokenKind};

pub(crate) fn docs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("docs")
}

pub(crate) fn doc_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![docs_dir()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("docs folder").flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn paragraph(block: &Block) -> &RichText {
    match block {
        Block::Paragraph(text) => text,
        other => panic!("expected a paragraph, got {other:?}"),
    }
}

fn span_text<'a>(rich: &'a RichText, span: &StyleSpan) -> &'a str {
    &rich.text[span.range.clone()]
}

#[test]
fn headings_get_outline_compatible_anchors() {
    let source = "# Title\n\n## Setup steps\n\ntext\n\n### Setup steps\n\n## Setup steps\n\n#### Deep\n\n## `code` & More!\n";
    let document = parse_document(source);
    let headings: Vec<(u8, &str, Option<&str>)> = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Heading {
                level,
                text,
                anchor,
            } => Some((*level, text.text.as_str(), anchor.as_deref())),
            _ => None,
        })
        .collect();
    assert_eq!(
        headings,
        vec![
            (1, "Title", None),
            (2, "Setup steps", Some("setup-steps")),
            (3, "Setup steps", Some("setup-steps-1")),
            (2, "Setup steps", Some("setup-steps-2")),
            (4, "Deep", None),
            (2, "code & More!", Some("code-more")),
        ]
    );
    let outline: Vec<String> = yp_docs::outline(source)
        .into_iter()
        .map(|heading| heading.anchor)
        .collect();
    assert_eq!(document.anchors(), outline);
}

#[test]
fn inline_styles_become_contiguous_spans() {
    let document = parse_document("plain **bold *both*** `code` ~~gone~~ end");
    let rich = paragraph(&document.blocks[0]);
    assert_eq!(rich.text, "plain bold both code gone end");
    let described: Vec<(&str, bool, bool, bool, bool)> = rich
        .spans
        .iter()
        .map(|span| {
            (
                span_text(rich, span),
                span.style.bold,
                span.style.italic,
                span.style.code,
                span.style.strikethrough,
            )
        })
        .collect();
    assert_eq!(
        described,
        vec![
            ("plain ", false, false, false, false),
            ("bold ", true, false, false, false),
            ("both", true, true, false, false),
            (" ", false, false, false, false),
            ("code", false, false, true, false),
            (" ", false, false, false, false),
            ("gone", false, false, false, true),
            (" end", false, false, false, false),
        ]
    );
    let mut cursor = 0;
    for span in &rich.spans {
        assert_eq!(span.range.start, cursor);
        cursor = span.range.end;
    }
    assert_eq!(cursor, rich.text.len());
}

#[test]
fn links_keep_their_target_and_cover_styled_text() {
    let document = parse_document(
        "See [the **item** page](03_item.md#fields), [top](#top) or <https://example.com/x>.",
    );
    let rich = paragraph(&document.blocks[0]);
    assert_eq!(
        rich.links,
        vec!["03_item.md#fields", "#top", "https://example.com/x"]
    );
    let linked: Vec<(&str, usize)> = rich
        .spans
        .iter()
        .filter_map(|span| span.link.map(|link| (span_text(rich, span), link)))
        .collect();
    assert_eq!(
        linked,
        vec![
            ("the ", 0),
            ("item", 0),
            (" page", 0),
            ("top", 1),
            ("https://example.com/x", 2),
        ]
    );
    let item = rich.text.find("item").expect("item");
    assert_eq!(rich.link_at(item), Some(0));
    assert_eq!(rich.link_at(0), None);
}

#[test]
fn soft_breaks_join_lines_and_hard_breaks_split_them() {
    let document = parse_document("one\ntwo  \nthree<br>four");
    assert_eq!(paragraph(&document.blocks[0]).text, "one two\nthree\nfour");
}

#[test]
fn tight_loose_nested_ordered_and_task_lists() {
    let source = "- a\n- b\n  1. one\n  2. two\n- c\n\n3. first\n\n   more\n4. second\n\n- [x] done\n- [ ] open\n";
    let document = parse_document(source);
    let lists: Vec<&List> = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::List(list) => Some(list),
            _ => None,
        })
        .collect();
    assert_eq!(lists.len(), 3);

    let bullets = lists[0];
    assert_eq!(bullets.start, None);
    assert_eq!(bullets.items.len(), 3);
    assert_eq!(paragraph(&bullets.items[0].blocks[0]).text, "a");
    assert_eq!(bullets.items[1].blocks.len(), 2);
    match &bullets.items[1].blocks[1] {
        Block::List(nested) => {
            assert_eq!(nested.start, Some(1));
            assert_eq!(nested.items.len(), 2);
            assert_eq!(paragraph(&nested.items[1].blocks[0]).text, "two");
        }
        other => panic!("expected a nested list, got {other:?}"),
    }

    let ordered = lists[1];
    assert_eq!(ordered.start, Some(3));
    assert_eq!(ordered.items[0].blocks.len(), 2);
    assert_eq!(paragraph(&ordered.items[0].blocks[1]).text, "more");

    let tasks = lists[2];
    assert_eq!(tasks.items[0].checked, Some(true));
    assert_eq!(tasks.items[1].checked, Some(false));
    assert_eq!(paragraph(&tasks.items[1].blocks[0]).text, "open");
}

#[test]
fn block_quotes_hold_their_own_blocks() {
    let document = parse_document("> **Note**\n> second line\n>\n> - item\n\nafter");
    match &document.blocks[0] {
        Block::Quote(children) => {
            assert_eq!(paragraph(&children[0]).text, "Note second line");
            assert!(matches!(children[1], Block::List(_)));
        }
        other => panic!("expected a quote, got {other:?}"),
    }
    assert_eq!(paragraph(&document.blocks[1]).text, "after");
}

#[test]
fn tables_keep_alignment_inline_code_and_ragged_rows() {
    let source = "| Key | Type | Notes |\n|:----|:----:|------:|\n| `id` | int | **required** |\n| name | text |\n";
    let document = parse_document(source);
    let table = match &document.blocks[0] {
        Block::Table(table) => table,
        other => panic!("expected a table, got {other:?}"),
    };
    assert_eq!(table.columns(), 3);
    assert_eq!(
        table.alignments,
        vec![CellAlign::Start, CellAlign::Center, CellAlign::End]
    );
    let head: Vec<&str> = table.head.iter().map(|cell| cell.text.as_str()).collect();
    assert_eq!(head, vec!["Key", "Type", "Notes"]);
    assert_eq!(table.rows.len(), 2);
    assert!(table.rows[0][0].spans[0].style.code);
    assert!(table.rows[0][2].spans[0].style.bold);
    assert_eq!(table.rows[1][1].text, "text");
    assert_eq!(table.alignment(7), CellAlign::Start);
}

#[test]
fn table_columns_get_room_for_their_longest_word_and_long_prose_is_capped() {
    let source = "| Field | Default | Meaning |\n|---|---|---|\n| `music_effects_on` | `true` | Picks the Wwise bus the track plays on, which is a channel of the mix. |\n| `a_very_long_identifier_that_cannot_wrap_anywhere_at_all_in_a_cell` | | x |\n";
    let document = parse_document(source);
    let Block::Table(table) = &document.blocks[0] else {
        panic!("expected a table");
    };
    assert_eq!(table.column_weights(), vec![48., 11., 36.]);
    let short = parse_document("| A | B |\n|---|---|\n| 1 | two words |\n");
    let Block::Table(short) = &short.blocks[0] else {
        panic!("expected a table");
    };
    assert_eq!(short.column_weights(), vec![10., 10.]);
    assert!(Table::default().column_weights().is_empty());
}

#[test]
fn code_blocks_keep_their_text_language_and_tokens() {
    let source = "```toml\n[item]\nid = 9999\n```\n\n```\nplain <b>\n```\n\n    indented\n\n```ruby\ndef a()\nend\n```\n";
    let document = parse_document(source);
    let blocks: Vec<(Option<Language>, &str, usize)> = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Code {
                language,
                code,
                tokens,
            } => Some((*language, code.as_str(), tokens.len())),
            _ => None,
        })
        .collect();
    assert_eq!(blocks.len(), 4);
    assert_eq!(blocks[0].0, Some(Language::Toml));
    assert_eq!(blocks[0].1, "[item]\nid = 9999");
    assert!(blocks[0].2 > 3);
    assert_eq!(blocks[1], (None, "plain <b>", 0));
    assert_eq!(blocks[2], (None, "indented", 0));
    assert_eq!(blocks[3].0, Some(Language::Ruby));
    match &document.blocks[0] {
        Block::Code { tokens, code, .. } => {
            assert_eq!(tokens[0].kind, TokenKind::Section);
            assert_eq!(&code[tokens[0].range.clone()], "[item]");
        }
        other => panic!("expected code, got {other:?}"),
    }
}

#[test]
fn images_become_figures_and_remote_ones_are_dropped() {
    let source = "Before ![The shop](../images/shop.png) after\n\n![](local.jpg)\n\n![remote](https://example.com/a.png)\n";
    let document = parse_document(source);
    assert_eq!(paragraph(&document.blocks[0]).text, "Before");
    assert_eq!(
        document.blocks[1],
        Block::Image {
            source: "../images/shop.png".into(),
            caption: Some("The shop".into()),
        }
    );
    assert_eq!(paragraph(&document.blocks[2]).text.trim(), "after");
    assert_eq!(
        document.blocks[3],
        Block::Image {
            source: "local.jpg".into(),
            caption: None,
        }
    );
    assert_eq!(document.blocks.len(), 4);
}

#[test]
fn markers_split_the_page_and_share_the_anchor_counter() {
    let source = "## Make it\n\nintro\n\n<!-- yp:item-builder -->\n\n## Make it\n\n<!-- yp:first-mod step=\"2\" -->\n\n<!-- a plain comment -->\n\nend\n";
    let document = parse_document(source);
    let tools: Vec<&str> = document
        .tools()
        .into_iter()
        .map(|marker| marker.id.as_str())
        .collect();
    assert_eq!(tools, vec!["item-builder", "first-mod"]);
    assert_eq!(document.tools()[1].args.get("step").map(String::as_str), Some("2"));
    assert_eq!(document.anchors(), vec!["make-it", "make-it-1"]);
    assert!(matches!(document.blocks[2], Block::Tool(_)));
    assert_eq!(paragraph(document.blocks.last().expect("last")).text, "end");
    assert!(!document.blocks.iter().any(|block| match block {
        Block::Paragraph(text) => text.text.contains("plain comment"),
        _ => false,
    }));
}

#[test]
fn rules_and_raw_html_are_kept_visible_or_dropped_sanely() {
    let document = parse_document("a\n\n---\n\n<details>\nraw\n</details>\n\nb <kbd>F5</kbd> c\n");
    assert!(matches!(document.blocks[1], Block::Rule));
    assert!(paragraph(&document.blocks[2]).text.contains("<details>"));
    assert!(paragraph(&document.blocks[2]).spans[0].style.code);
    assert_eq!(paragraph(&document.blocks[3]).text, "b <kbd>F5</kbd> c");
}

#[test]
fn empty_and_whitespace_documents_have_no_blocks() {
    assert!(parse_document("").is_empty());
    assert!(parse_document("  \n\n\t\n").is_empty());
    assert!(parse_document("<!-- only a comment -->").is_empty());
}

fn walk<'a>(blocks: &'a [Block], visit: &mut dyn FnMut(&'a Block)) {
    for block in blocks {
        visit(block);
        match block {
            Block::Quote(children) => walk(children, visit),
            Block::List(list) => {
                for item in &list.items {
                    walk(&item.blocks, visit);
                }
            }
            _ => {}
        }
    }
}

fn check_rich(rich: &RichText, file: &Path) {
    let mut cursor = 0;
    for span in &rich.spans {
        assert_eq!(span.range.start, cursor, "span gap in {}", file.display());
        assert!(rich.text.is_char_boundary(span.range.end));
        if let Some(link) = span.link {
            assert!(link < rich.links.len(), "dangling link in {}", file.display());
        }
        cursor = span.range.end;
    }
    assert_eq!(cursor, rich.text.len(), "spans stop early in {}", file.display());
}

#[test]
fn every_bundled_page_parses_with_consistent_spans_and_outline_anchors() {
    let files = doc_files();
    assert!(files.len() > 50, "bundled docs are missing");
    let mut constructs = std::collections::BTreeSet::new();
    for file in &files {
        let source = std::fs::read_to_string(file).expect("doc page");
        let document = parse_document(&source);
        assert!(!document.is_empty(), "{} rendered empty", file.display());
        let outline: Vec<String> = yp_docs::outline(&source)
            .into_iter()
            .map(|heading| heading.anchor)
            .collect();
        assert_eq!(
            document.anchors(),
            outline,
            "anchors differ from the outline in {}",
            file.display()
        );
        walk(&document.blocks, &mut |block| match block {
            Block::Heading { text, .. } => {
                constructs.insert("heading");
                check_rich(text, file);
            }
            Block::Paragraph(text) => {
                constructs.insert("paragraph");
                check_rich(text, file);
                if !text.links.is_empty() {
                    constructs.insert("link");
                }
            }
            Block::Code {
                code,
                tokens,
                language,
            } => {
                constructs.insert("code");
                if language.is_some() {
                    crate::syntax::testing::assert_covers(code, tokens);
                }
            }
            Block::List(_) => {
                constructs.insert("list");
            }
            Block::Quote(_) => {
                constructs.insert("quote");
            }
            Block::Table(table) => {
                constructs.insert("table");
                for cell in table.head.iter().chain(table.rows.iter().flatten()) {
                    check_rich(cell, file);
                }
            }
            Block::Image { .. } => {
                constructs.insert("image");
            }
            Block::Rule => {
                constructs.insert("rule");
            }
            Block::Tool(_) => {
                constructs.insert("tool");
            }
        });
    }
    for expected in [
        "heading",
        "paragraph",
        "link",
        "code",
        "list",
        "quote",
        "table",
        "image",
        "tool",
    ] {
        assert!(constructs.contains(expected), "no {expected} in the bundled docs");
    }
}

struct Page {
    document: Document,
    context: MarkdownContext,
}

impl Render for Page {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("page")
            .size_full()
            .overflow_y_scroll()
            .children(render_document(&self.document, &self.context, window, cx))
    }
}

fn test_context() -> MarkdownContext {
    MarkdownContext::new("test")
        .figures(|source| {
            Some(Figure {
                path: docs_dir().join("images").join(source),
                width: 640,
                height: 360,
            })
        })
        .tools(|marker, _, _| Some(div().child(marker.id.clone()).into_any_element()))
}

#[gpui::test]
fn every_bundled_page_renders_without_panicking(cx: &mut TestAppContext) {
    cx.update(yp_ui::init_for_tests);
    cx.update(crate::markdown::rich_text::init);
    let (page, cx) = cx.add_window_view(|_, _| Page {
        document: Document::default(),
        context: test_context(),
    });
    let mut rendered = 0;
    for file in doc_files() {
        let source = std::fs::read_to_string(&file).expect("doc page");
        page.update(cx, |page, cx| {
            page.document = parse_document(&source);
            page.context.anchors.borrow_mut().clear();
            cx.notify();
        });
        cx.run_until_parked();
        let expected = page.read_with(cx, |page, _| page.document.anchors().len());
        let tracked = page.read_with(cx, |page, _| page.context.anchors.borrow().len());
        assert_eq!(tracked, expected, "anchors not tracked in {}", file.display());
        rendered += 1;
    }
    assert!(rendered > 50);
}

#[gpui::test]
fn a_right_click_on_text_offers_select_all_and_copy(cx: &mut TestAppContext) {
    cx.update(yp_ui::init_for_tests);
    cx.update(crate::markdown::rich_text::init);
    let (page, cx) = cx.add_window_view(|_, _| Page {
        document: parse_document("## Alpha beta\n"),
        context: test_context(),
    });
    cx.run_until_parked();
    let heading = page
        .read_with(cx, |page, _| page.context.anchors.borrow().get("alpha-beta").copied())
        .expect("heading bounds");
    let inside = gpui::point(heading.left() + gpui::px(6.), heading.bottom() - gpui::px(6.));
    let right_click = |cx: &mut gpui::VisualTestContext| {
        cx.simulate_mouse_down(inside, gpui::MouseButton::Right, gpui::Modifiers::default());
        cx.simulate_mouse_up(inside, gpui::MouseButton::Right, gpui::Modifiers::default());
        cx.run_until_parked();
    };

    assert_eq!(heading.top(), gpui::px(0.));
    let clipboard = |cx: &mut gpui::VisualTestContext| {
        cx.read_from_clipboard().and_then(|item| item.text())
    };

    right_click(cx);
    cx.simulate_keystrokes("down enter");
    cx.run_until_parked();
    assert_eq!(clipboard(cx).as_deref(), Some("Alpha beta"));

    cx.write_to_clipboard(gpui::ClipboardItem::new_string("untouched".to_owned()));
    right_click(cx);
    cx.simulate_keystrokes("down down enter");
    cx.run_until_parked();
    assert_eq!(clipboard(cx).as_deref(), Some("untouched"));
    cx.simulate_keystrokes("secondary-c");
    assert_eq!(clipboard(cx).as_deref(), Some("Alpha beta"));
}

#[gpui::test]
fn both_themes_and_the_compact_style_render_every_construct(cx: &mut TestAppContext) {
    cx.update(yp_ui::init_for_tests);
    let source = "# One\n\n## Two\n\ntext **b** *i* `c` ~~s~~ [l](x.md) <https://a.b>\n\n- a\n  1. b\n- [x] c\n\n> quote\n\n| a | b |\n|---|:-:|\n| 1 | `2` |\n\n```toml\na = 1\n```\n\n```\nplain\n```\n\n![cap](x.png)\n\n---\n\n<!-- yp:first-mod -->\n\n###### six\n";
    let (page, cx) = cx.add_window_view(|_, _| Page {
        document: parse_document(source),
        context: test_context(),
    });
    for theme in yp_ui::ThemeId::ALL {
        for compact in [false, true] {
            cx.update(|_, cx| yp_ui::theme::set_theme(theme, cx));
            page.update(cx, |page, cx| {
                page.context.compact = compact;
                cx.notify();
            });
            cx.run_until_parked();
        }
    }
    let anchors = page.read_with(cx, |page, _| page.context.anchors.borrow().len());
    assert_eq!(anchors, 1);
}
