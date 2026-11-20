use std::fmt;

use crate::index::DocPage;

pub const EXCERPT_CHARS_BEFORE: usize = 60;
pub const EXCERPT_CHARS_AFTER: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Excerpt {
    pub text: String,
    pub clipped_start: bool,
    pub clipped_end: bool,
}

impl fmt::Display for Excerpt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = if self.clipped_start { "..." } else { "" };
        let suffix = if self.clipped_end { "..." } else { "" };
        write!(f, "{prefix}{}{suffix}", self.text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocSearchHit {
    pub page: DocPage,
    pub excerpt: Option<Excerpt>,
}

struct Lowered {
    text: String,
    char_index: Vec<usize>,
}

fn lowered_with_char_index(body: &str) -> Lowered {
    let mut text = String::with_capacity(body.len());
    let mut char_index = Vec::with_capacity(body.len());
    for (index, character) in body.chars().enumerate() {
        for lower in character.to_lowercase() {
            for _ in 0..lower.len_utf8() {
                char_index.push(index);
            }
            text.push(lower);
        }
    }
    Lowered { text, char_index }
}

pub fn find_case_insensitive(body: &str, needle_lower: &str) -> Option<usize> {
    let lowered = lowered_with_char_index(body);
    let byte = lowered.text.find(needle_lower)?;
    lowered.char_index.get(byte).copied()
}

fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_was_space = false;
    for character in text.chars() {
        if matches!(character, ' ' | '\n' | '\r' | '\t') {
            if !last_was_space {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(character);
            last_was_space = false;
        }
    }
    out.trim().to_owned()
}

pub fn excerpt_at(body: &str, char_index: usize, char_len: usize) -> Excerpt {
    let total = body.chars().count();
    let start = char_index.saturating_sub(EXCERPT_CHARS_BEFORE).min(total);
    let end = (char_index + char_len + EXCERPT_CHARS_AFTER).min(total);
    let snippet: String = body
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect();
    Excerpt {
        text: collapse_whitespace(&snippet),
        clipped_start: start > 0,
        clipped_end: end < total,
    }
}

pub fn search_pages(pages: &[DocPage], bodies: &[String], query: &str) -> Vec<DocSearchHit> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut title_hits = Vec::new();
    let mut body_hits = Vec::new();
    for (page, body) in pages.iter().zip(bodies) {
        if page.title.to_lowercase().contains(&needle) {
            title_hits.push(DocSearchHit {
                page: page.clone(),
                excerpt: None,
            });
            continue;
        }
        let Some(index) = find_case_insensitive(body, &needle) else {
            continue;
        };
        body_hits.push(DocSearchHit {
            page: page.clone(),
            excerpt: Some(excerpt_at(body, index, needle.chars().count())),
        });
    }
    title_hits.extend(body_hits);
    title_hits
}
