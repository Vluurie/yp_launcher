use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

pub fn markdown_options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
}

pub fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                lines.push(&text[start..index]);
                index += 1;
                start = index;
            }
            b'\r' => {
                lines.push(&text[start..index]);
                index += if bytes.get(index + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                start = index;
            }
            _ => index += 1,
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

pub fn image_sources(markdown: &str) -> Vec<String> {
    Parser::new_ext(markdown, markdown_options())
        .filter_map(|event| match event {
            Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url.to_string()),
            _ => None,
        })
        .collect()
}

pub(crate) fn html_spans(markdown: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut block: Option<Range<usize>> = None;
    for (event, range) in Parser::new_ext(markdown, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::HtmlBlock) => block = Some(range),
            Event::End(TagEnd::HtmlBlock) => {
                if let Some(range) = block.take() {
                    spans.push(range);
                }
            }
            Event::InlineHtml(_) if block.is_none() => spans.push(range),
            _ => {}
        }
    }
    spans
}
