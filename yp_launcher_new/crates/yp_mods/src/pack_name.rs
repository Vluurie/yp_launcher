use crate::text::{is_line_terminator, is_regex_whitespace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopedPackName<'a> {
    pub base: &'a str,
    pub mod_id: &'a str,
    pub profile: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PulledOutPackName<'a> {
    pub base: &'a str,
    pub mod_id: &'a str,
    pub profile: &'a str,
    pub ordinal: Option<&'a str>,
}

struct Suffix<'a> {
    before_paren: &'a str,
    inner: &'a str,
}

fn split_paren_suffix(name: &str) -> Option<Suffix<'_>> {
    let without_close = name.strip_suffix(')')?;
    let open = without_close.rfind('(')?;
    let inner = &without_close[open + 1..];
    if inner.contains(')') {
        return None;
    }
    Some(Suffix {
        before_paren: &without_close[..open],
        inner,
    })
}

fn split_spaced_suffix(name: &str) -> Option<(&str, &str)> {
    let suffix = split_paren_suffix(name)?;
    let base = suffix.before_paren.strip_suffix(' ')?;
    if base.chars().any(is_line_terminator) {
        return None;
    }
    Some((base, suffix.inner))
}

fn split_scope(inner: &str) -> Option<(&str, &str)> {
    let (mod_id, profile) = inner.split_once('@')?;
    if mod_id.is_empty() || profile.is_empty() {
        return None;
    }
    Some((mod_id, profile))
}

fn trim_regex_whitespace_end(name: &str) -> &str {
    name.trim_end_matches(is_regex_whitespace)
}

pub fn parse_pulled_out(name: &str) -> Option<PulledOutPackName<'_>> {
    let (base, inner) = split_spaced_suffix(name)?;
    let (mod_id, rest) = split_scope(inner)?;
    let (profile, ordinal) = match split_ordinal(rest) {
        Some((profile, digits)) if !profile.is_empty() => (profile, Some(digits)),
        _ => (rest, None),
    };
    Some(PulledOutPackName {
        base,
        mod_id,
        profile,
        ordinal,
    })
}

fn split_ordinal(rest: &str) -> Option<(&str, &str)> {
    let digits_start = rest
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .last()
        .map(|(index, _)| index)?;
    let profile = rest[..digits_start].strip_suffix(' ')?;
    Some((profile, &rest[digits_start..]))
}

pub fn parse_scoped(name: &str) -> Option<ScopedPackName<'_>> {
    let (base, inner) = split_spaced_suffix(trim_regex_whitespace_end(name))?;
    let (mod_id, profile) = split_scope(inner)?;
    Some(ScopedPackName {
        base,
        mod_id,
        profile,
    })
}

pub fn is_profile_scoped(name: &str) -> bool {
    split_paren_suffix(trim_regex_whitespace_end(name))
        .and_then(|suffix| split_scope(suffix.inner))
        .is_some()
}

fn parse_legacy(name: &str) -> Option<(&str, &str)> {
    let (base, inner) = split_spaced_suffix(trim_regex_whitespace_end(name))?;
    if inner.is_empty() || inner.contains('@') {
        return None;
    }
    Some((base, inner))
}

pub fn scoped(base: &str, mod_id: &str, profile: &str) -> String {
    format!("{base} ({mod_id}@{profile})")
}

pub fn upgrade(pack: &str, profile: &str, mod_id: &str) -> String {
    if is_profile_scoped(pack) {
        return pack.to_owned();
    }
    if let Some((base, legacy_id)) = parse_legacy(pack) {
        return scoped(base, legacy_id, profile);
    }
    scoped(pack, mod_id, profile)
}

pub fn swap_profile(pack: &str, old_profile: &str, new_profile: &str) -> String {
    match parse_scoped(pack) {
        Some(parsed) if parsed.profile == old_profile => {
            scoped(parsed.base, parsed.mod_id, new_profile)
        }
        _ => pack.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pulled_out_names_with_and_without_ordinal() {
        assert_eq!(
            parse_pulled_out("HD Pack (kimono@default)"),
            Some(PulledOutPackName {
                base: "HD Pack",
                mod_id: "kimono",
                profile: "default",
                ordinal: None
            })
        );
        assert_eq!(
            parse_pulled_out("HD Pack (kimono@beta 2)"),
            Some(PulledOutPackName {
                base: "HD Pack",
                mod_id: "kimono",
                profile: "beta",
                ordinal: Some("2")
            })
        );
        assert_eq!(
            parse_pulled_out("p (m@a 1 2)").map(|p| (p.profile, p.ordinal)),
            Some(("a 1", Some("2")))
        );
        assert_eq!(
            parse_pulled_out("p (m@ 12)").map(|p| (p.profile, p.ordinal)),
            Some((" 12", None))
        );
        assert_eq!(
            parse_pulled_out("p (m@x@y)").map(|p| (p.mod_id, p.profile)),
            Some(("m", "x@y"))
        );
        assert_eq!(parse_pulled_out(" (m@p)").map(|p| p.base), Some(""));
        assert_eq!(
            parse_pulled_out("(a) b (m@p)").map(|p| p.base),
            Some("(a) b")
        );
    }

    #[test]
    fn rejects_malformed_pulled_out_names() {
        assert_eq!(parse_pulled_out("HD Pack"), None);
        assert_eq!(parse_pulled_out("HD Pack(m@p)"), None);
        assert_eq!(parse_pulled_out("HD Pack (@p)"), None);
        assert_eq!(parse_pulled_out("HD Pack (m@)"), None);
        assert_eq!(parse_pulled_out("HD Pack (mp)"), None);
        assert_eq!(parse_pulled_out("HD Pack (m@p) "), None);
        assert_eq!(parse_pulled_out("a)b (m@p)x)"), None);
        assert_eq!(parse_pulled_out("line\nbreak (m@p)"), None);
    }

    #[test]
    fn upgrades_plain_and_legacy_names() {
        assert_eq!(
            upgrade("HD Pack", "default", "kimono"),
            "HD Pack (kimono@default)"
        );
        assert_eq!(
            upgrade("HD Pack (kimono)", "beta", "other"),
            "HD Pack (kimono@beta)"
        );
        assert_eq!(
            upgrade("HD Pack (kimono)  ", "beta", "other"),
            "HD Pack (kimono@beta)"
        );
        assert_eq!(
            upgrade("HD Pack (kimono@beta)", "x", "y"),
            "HD Pack (kimono@beta)"
        );
        assert_eq!(
            upgrade("HD Pack(kimono@beta) ", "x", "y"),
            "HD Pack(kimono@beta) "
        );
        assert_eq!(upgrade("Pack(a)", "p", "m"), "Pack(a) (m@p)");
    }

    #[test]
    fn swaps_only_the_matching_profile() {
        assert_eq!(swap_profile("Pack (m@old)", "old", "new"), "Pack (m@new)");
        assert_eq!(
            swap_profile("Pack (m@other)", "old", "new"),
            "Pack (m@other)"
        );
        assert_eq!(swap_profile("Pack", "old", "new"), "Pack");
        assert_eq!(swap_profile("Pack (m@old) ", "old", "new"), "Pack (m@new)");
    }
}
