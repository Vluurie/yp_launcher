use std::collections::BTreeMap;
use std::ops::Range;

use crate::markdown::html_spans;

const COMMENT_OPEN: &str = "<!--";
const MARKER_PREFIX: &str = " yp:";
const COMMENT_CLOSE: &str = "-->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsMarker {
    pub id: String,
    pub args: BTreeMap<String, String>,
    pub start: usize,
    pub end: usize,
}

enum Candidate {
    NotAMarker,
    Invalid,
    Marker(DocsMarker),
}

fn comment_close(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let close = COMMENT_CLOSE.as_bytes();
    (0..bytes.len().saturating_sub(close.len() - 1))
        .find(|&index| bytes[index..].starts_with(close))
}

fn candidate(markdown: &str, span: Range<usize>) -> Candidate {
    let text = &markdown[span.clone()];
    let indent = text.len() - text.trim_start().len();
    let comment = &text[indent..];
    let Some(body) = comment.strip_prefix(COMMENT_OPEN) else {
        return Candidate::NotAMarker;
    };
    let Some(after_prefix) = body.strip_prefix(MARKER_PREFIX) else {
        return Candidate::NotAMarker;
    };
    let Some(close) = comment_close(after_prefix) else {
        return Candidate::Invalid;
    };
    let inner = after_prefix[..close].trim();
    if inner.is_empty() {
        return Candidate::Invalid;
    }
    let (id, rest) = inner.split_once(' ').unwrap_or((inner, ""));
    let start = span.start + indent;
    let end = start + COMMENT_OPEN.len() + MARKER_PREFIX.len() + close + COMMENT_CLOSE.len();
    Candidate::Marker(DocsMarker {
        id: id.to_owned(),
        args: parse_args(rest),
        start,
        end,
    })
}

fn parse_args(text: &str) -> BTreeMap<String, String> {
    let mut args = BTreeMap::new();
    let mut rest = text;
    while let Some((key, after_equals)) = rest.split_once('=') {
        let key = key.trim();
        if key.is_empty() {
            break;
        }
        let Some(quoted) = after_equals.strip_prefix('"') else {
            break;
        };
        let Some((value, remainder)) = quoted.split_once('"') else {
            break;
        };
        args.insert(key.to_owned(), value.to_owned());
        rest = remainder;
    }
    args
}

impl DocsMarker {
    pub fn first_in(markdown: &str) -> Option<DocsMarker> {
        for span in html_spans(markdown) {
            match candidate(markdown, span) {
                Candidate::NotAMarker => continue,
                Candidate::Invalid => return None,
                Candidate::Marker(marker) => return Some(marker),
            }
        }
        None
    }

    pub fn all_in(markdown: &str) -> Vec<DocsMarker> {
        html_spans(markdown)
            .into_iter()
            .filter_map(|span| match candidate(markdown, span) {
                Candidate::Marker(marker) => Some(marker),
                Candidate::NotAMarker | Candidate::Invalid => None,
            })
            .collect()
    }

    pub fn arg(&self, key: &str) -> Option<&str> {
        self.args.get(key).map(String::as_str)
    }
}
