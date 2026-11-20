use jiff::Timestamp;

use crate::probes::FileFacts;

const KIB: u64 = 1024;
const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * 1024 * 1024;
const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

pub const DEFAULT_LIST_CAP: usize = 25;

#[derive(Debug, Default)]
pub(crate) struct Lines {
    text: String,
}

impl Lines {
    pub(crate) fn line(&mut self, text: impl AsRef<str>) {
        self.text.push_str(text.as_ref());
        self.text.push('\n');
    }

    pub(crate) fn blank(&mut self) {
        self.text.push('\n');
    }

    pub(crate) fn finish(self) -> String {
        self.text
    }
}

pub fn time_label(stamp: Option<Timestamp>) -> String {
    match stamp {
        Some(stamp) => stamp.strftime(TIME_FORMAT).to_string(),
        None => "?".to_owned(),
    }
}

pub fn human_size(bytes: u64) -> String {
    if bytes >= GIB {
        format!("{:.2} GB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{} KB", (bytes as f64 / KIB as f64).round() as u64)
    } else {
        format!("{bytes} B")
    }
}

pub fn hex_label(value: Option<u32>) -> String {
    match value {
        Some(value) => format!("0x{value:08x}"),
        None => "?".to_owned(),
    }
}

pub fn file_line(facts: &FileFacts) -> String {
    let sha1 = facts
        .short_sha1
        .as_deref()
        .map(|digest| format!("  sha1 {digest}"))
        .unwrap_or_default();
    format!(
        "{}  {}  modified {}{sha1}",
        facts.name,
        human_size(facts.size),
        time_label(facts.modified)
    )
}

pub fn cap_list<S: AsRef<str>>(items: &[S], cap: usize) -> String {
    let shown: Vec<&str> = items.iter().take(cap).map(AsRef::as_ref).collect();
    if items.len() <= cap {
        shown.join(", ")
    } else {
        format!("{}, ... {} more", shown.join(", "), items.len() - cap)
    }
}

pub(crate) fn yes_no(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}

pub(crate) fn truncated(text: &str, max_chars: usize) -> Option<&str> {
    text.char_indices()
        .nth(max_chars)
        .map(|(index, _)| &text[..index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_use_the_report_units() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1536), "2 KB");
        assert_eq!(human_size(17_837_952), "17.0 MB");
        assert_eq!(human_size(3 * GIB + GIB / 2), "3.50 GB");
    }

    #[test]
    fn stamps_print_as_whole_utc_seconds() {
        let stamp: Timestamp = "2026-11-06T10:20:30.987654Z".parse().unwrap();
        assert_eq!(time_label(Some(stamp)), "2026-11-06T10:20:30Z");
        assert_eq!(time_label(None), "?");
        assert_eq!(hex_label(Some(0x60d3_a8d2)), "0x60d3a8d2");
        assert_eq!(hex_label(Some(0x12)), "0x00000012");
        assert_eq!(hex_label(None), "?");
    }

    #[test]
    fn long_lists_are_capped_with_a_remainder() {
        let items = ["a", "b", "c"];
        assert_eq!(cap_list(&items, 3), "a, b, c");
        assert_eq!(cap_list(&items, 2), "a, b, ... 1 more");
        assert_eq!(cap_list::<&str>(&[], 2), "");
    }

    #[test]
    fn truncation_counts_characters() {
        assert_eq!(truncated("äbc", 2), Some("äb"));
        assert_eq!(truncated("abc", 3), None);
    }
}
