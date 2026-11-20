use crate::text::dart_trim;

const LAST_BMP_CODE_POINT: u32 = 0xFFFF;

fn without_trailing_number_groups(text: &str) -> &str {
    let bytes = text.as_bytes();
    let mut end = bytes.len();
    loop {
        let digits_start = bytes[..end]
            .iter()
            .rposition(|byte| !byte.is_ascii_digit())
            .map_or(0, |index| index + 1);
        if digits_start == end || digits_start == 0 {
            break;
        }
        let separator = digits_start - 1;
        if !matches!(bytes[separator], b'-' | b'_') {
            break;
        }
        end = separator;
    }
    &text[..end]
}

fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) if u32::from(first) <= LAST_BMP_CODE_POINT => {
            first.to_uppercase().chain(chars).collect()
        }
        _ => word.to_owned(),
    }
}

pub fn prettify_mod_id(suggested: &str) -> String {
    let stem = without_trailing_number_groups(suggested);
    let mut spaced = String::with_capacity(stem.len());
    let mut in_underscores = false;
    for c in stem.chars() {
        if c == '_' {
            if !in_underscores {
                spaced.push(' ');
            }
            in_underscores = true;
        } else {
            in_underscores = false;
            spaced.push(c);
        }
    }
    let trimmed = dart_trim(&spaced);
    if trimmed.is_empty() {
        return suggested.to_owned();
    }
    trimmed
        .split(' ')
        .map(capitalized)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_number_groups() {
        assert_eq!(
            without_trailing_number_groups("kimono_wax_0_0_3"),
            "kimono_wax"
        );
        assert_eq!(
            without_trailing_number_groups("nier_cpk-709-1-2"),
            "nier_cpk"
        );
        assert_eq!(without_trailing_number_groups("mixed-1_2-3"), "mixed");
        assert_eq!(without_trailing_number_groups("a1_2"), "a1");
        assert_eq!(without_trailing_number_groups("a_1b_2"), "a_1b");
        assert_eq!(without_trailing_number_groups("outfit_v2"), "outfit_v2");
        assert_eq!(without_trailing_number_groups("weapon_12x"), "weapon_12x");
        assert_eq!(without_trailing_number_groups("foo__3"), "foo_");
        assert_eq!(without_trailing_number_groups("foo-3_"), "foo-3_");
        assert_eq!(without_trailing_number_groups("123"), "123");
        assert_eq!(without_trailing_number_groups("_123"), "");
        assert_eq!(without_trailing_number_groups(""), "");
        assert_eq!(without_trailing_number_groups("é_1"), "é");
    }

    #[test]
    fn prettifies_suggested_ids() {
        assert_eq!(prettify_mod_id("kimono_wax_0_0_3"), "Kimono Wax");
        assert_eq!(prettify_mod_id("nier_cpk-709-1-2"), "Nier Cpk");
        assert_eq!(prettify_mod_id("2p_classic"), "2p Classic");
        assert_eq!(prettify_mod_id("mock-machine"), "Mock-machine");
        assert_eq!(prettify_mod_id("a__b"), "A B");
        assert_eq!(prettify_mod_id("foo__3"), "Foo");
        assert_eq!(prettify_mod_id("_lead_and_trail_"), "Lead And Trail");
        assert_eq!(prettify_mod_id("already Pretty"), "Already Pretty");
        assert_eq!(prettify_mod_id("two  spaces"), "Two  Spaces");
        assert_eq!(prettify_mod_id("éclair_1"), "Éclair");
        assert_eq!(prettify_mod_id("straße"), "Straße");
        assert_eq!(prettify_mod_id("ßeta"), "SSeta");
    }

    #[test]
    fn keeps_the_suggestion_when_nothing_is_left() {
        assert_eq!(prettify_mod_id("_123"), "_123");
        assert_eq!(prettify_mod_id("___"), "___");
        assert_eq!(prettify_mod_id(""), "");
        assert_eq!(prettify_mod_id("123"), "123");
    }
}
