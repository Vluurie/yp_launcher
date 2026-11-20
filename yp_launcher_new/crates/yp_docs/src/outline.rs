use std::collections::HashMap;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::anchor::anchor_for;
use crate::markdown::{markdown_options, split_lines};

const FENCE: &str = "```";
const MIN_LEVEL: usize = 2;
const MAX_LEVEL: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocHeading {
    pub level: u8,
    pub text: String,
    pub anchor: String,
}

pub fn heading_plain_text(heading_line: &str) -> String {
    let mut text = String::new();
    let mut inside_heading = false;
    for event in Parser::new_ext(heading_line, markdown_options()) {
        match event {
            Event::Start(Tag::Heading { .. }) => inside_heading = true,
            Event::End(TagEnd::Heading(_)) => inside_heading = false,
            Event::Text(chunk) | Event::Code(chunk) | Event::InlineHtml(chunk)
                if inside_heading =>
            {
                text.push_str(&chunk);
            }
            Event::SoftBreak | Event::HardBreak if inside_heading => text.push(' '),
            _ => {}
        }
    }
    text.trim().to_owned()
}

fn heading_level(trimmed: &str) -> Option<usize> {
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    let followed_by_space = trimmed[level..].starts_with(' ');
    (MIN_LEVEL..=MAX_LEVEL)
        .contains(&level)
        .then_some(level)
        .filter(|_| followed_by_space)
}

pub fn outline(markdown: &str) -> Vec<DocHeading> {
    let mut headings = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut in_fence = false;

    for line in split_lines(markdown) {
        let trimmed = line.trim();
        if trimmed.starts_with(FENCE) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let Some(level) = heading_level(trimmed) else {
            continue;
        };
        let text = heading_plain_text(&format!(
            "{} {}",
            "#".repeat(level),
            trimmed[level + 1..].trim()
        ));
        if text.is_empty() {
            continue;
        }
        let base = anchor_for(&text);
        let count = seen
            .entry(base.clone())
            .and_modify(|count| *count += 1)
            .or_insert(0);
        let anchor = if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        };
        headings.push(DocHeading {
            level: u8::try_from(level).unwrap_or(u8::MAX),
            text,
            anchor,
        });
    }
    headings
}
