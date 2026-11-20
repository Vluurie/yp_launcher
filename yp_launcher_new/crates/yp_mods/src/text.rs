use std::cmp::Ordering;

pub(crate) fn is_regex_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{9}'..='\u{d}'
            | ' '
            | '\u{a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    )
}

pub(crate) fn is_trim_whitespace(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

pub(crate) fn dart_trim(text: &str) -> &str {
    text.trim_matches(is_trim_whitespace)
}

pub(crate) fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

pub(crate) fn utf16_cmp(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

pub(crate) fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

pub(crate) fn normalize_rel_path(rel: &str) -> String {
    let slashed = rel.replace('\\', "/");
    dart_trim(&slashed).trim_matches('/').to_owned()
}

pub(crate) fn file_extension(name: &str) -> &str {
    match name.rfind('.') {
        Some(dot) if dot > 0 && name != ".." => &name[dot..],
        _ => "",
    }
}

pub(crate) fn has_extension_in(name: &str, extensions: &[&str]) -> bool {
    extensions.contains(&file_extension(name).to_lowercase().as_str())
}

pub(crate) fn strip_ascii_suffix<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    let split = text.len().checked_sub(suffix.len())?;
    if !text.is_char_boundary(split) {
        return None;
    }
    let (head, tail) = text.split_at(split);
    tail.eq_ignore_ascii_case(suffix).then_some(head)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_like_dart() {
        assert_eq!(dart_trim("\u{feff} a \u{85}"), "a");
        assert_eq!(normalize_rel_path(" \\mods\\a/ "), "mods/a");
        assert_eq!(normalize_rel_path("//x//"), "x");
    }

    #[test]
    fn compares_by_utf16_units() {
        assert_eq!(utf16_cmp("a", "b"), Ordering::Less);
        assert_eq!(utf16_cmp("\u{ffff}", "\u{10000}"), Ordering::Greater);
        assert_eq!(utf16_len("\u{10000}"), 2);
    }

    #[test]
    fn file_extensions_follow_the_dart_path_package() {
        assert_eq!(file_extension("a.dds"), ".dds");
        assert_eq!(file_extension(".dds"), "");
        assert_eq!(file_extension("noext"), "");
        assert_eq!(file_extension("a.b.cpk"), ".cpk");
        assert_eq!(file_extension(".."), "");
        assert!(has_extension_in("Mod.ZIP", &[".zip", ".7z"]));
        assert!(!has_extension_in(".zip", &[".zip"]));
        assert!(!has_extension_in("zip", &[".zip"]));
    }

    #[test]
    fn strips_suffix_case_insensitively() {
        assert_eq!(strip_ascii_suffix("PL0000.DAT", ".dat"), Some("PL0000"));
        assert_eq!(strip_ascii_suffix("a.dtt", ".dat"), None);
        assert_eq!(strip_ascii_suffix("é", ".dat"), None);
    }
}
