use encoding_rs::Encoding;

pub const BACKSLASH_IN_NAME: char = '\u{f05c}';

const REPLACEMENT: char = '_';
const RESERVED_DEVICES: [&str; 6] = ["CON", "PRN", "AUX", "NUL", "COM", "LPT"];
const FIRST_NUMBERED_DEVICE: usize = 4;
const FIRST_PRINTABLE: u32 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backslash {
    Separator,
    NameCharacter,
}

pub fn oem_encoding(code_page: u32) -> Option<&'static Encoding> {
    Some(match code_page {
        866 => encoding_rs::IBM866,
        874 => encoding_rs::WINDOWS_874,
        932 => encoding_rs::SHIFT_JIS,
        936 => encoding_rs::GBK,
        949 => encoding_rs::EUC_KR,
        950 => encoding_rs::BIG5,
        1250 => encoding_rs::WINDOWS_1250,
        1251 => encoding_rs::WINDOWS_1251,
        1252 => encoding_rs::WINDOWS_1252,
        1253 => encoding_rs::WINDOWS_1253,
        1254 => encoding_rs::WINDOWS_1254,
        1255 => encoding_rs::WINDOWS_1255,
        1256 => encoding_rs::WINDOWS_1256,
        1257 => encoding_rs::WINDOWS_1257,
        1258 => encoding_rs::WINDOWS_1258,
        _ => return None,
    })
}

pub(crate) fn decode_legacy(raw: &[u8], oem: Option<&'static Encoding>) -> Option<String> {
    oem?.decode_without_bom_handling_and_without_replacement(raw)
        .map(|text| text.into_owned())
}

fn is_reserved_device(name: &str) -> bool {
    RESERVED_DEVICES.iter().enumerate().any(|(index, device)| {
        let Some(head) = name.get(..device.len()) else {
            return false;
        };
        if !head.eq_ignore_ascii_case(device) {
            return false;
        }
        let mut rest = name[device.len()..].chars();
        let mut next = rest.next();
        if index >= FIRST_NUMBERED_DEVICE {
            match next {
                Some(digit) if digit.is_ascii_digit() => next = rest.next(),
                _ => return false,
            }
        }
        loop {
            match next {
                None | Some('.') => return true,
                Some(' ') => next = rest.next(),
                Some(_) => return false,
            }
        }
    })
}

fn replace_invalid(part: &str) -> String {
    let mut characters: Vec<char> = part
        .chars()
        .map(|character| match character {
            ':' | '*' | '?' | '<' | '>' | '|' | '"' => REPLACEMENT,
            '\\' => BACKSLASH_IN_NAME,
            other if (other as u32) < FIRST_PRINTABLE => REPLACEMENT,
            other => other,
        })
        .collect();
    for character in characters.iter_mut().rev() {
        if *character != '.' && *character != ' ' {
            break;
        }
        *character = REPLACEMENT;
    }
    characters.into_iter().collect()
}

pub fn entry_parts(name: &str, is_dir: bool, backslash: Backslash) -> Vec<String> {
    let raw: Vec<&str> = match backslash {
        Backslash::Separator => name.split(['/', '\\']).collect(),
        Backslash::NameCharacter => name.split('/').collect(),
    };
    let last = raw.len().saturating_sub(1);
    let mut parts = Vec::with_capacity(raw.len());
    for (index, part) in raw.into_iter().enumerate() {
        let cleaned = if part == "." || part == ".." {
            String::new()
        } else {
            replace_invalid(part)
        };
        if cleaned.is_empty() {
            if !is_dir && index == last {
                parts.push(REPLACEMENT.to_string());
            }
        } else if is_reserved_device(&cleaned) {
            parts.push(format!("{REPLACEMENT}{cleaned}"));
        } else {
            parts.push(cleaned);
        }
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str) -> String {
        entry_parts(name, false, Backslash::Separator).join("/")
    }

    fn dir(name: &str) -> String {
        entry_parts(name, true, Backslash::Separator).join("/")
    }

    #[test]
    fn plain_names_pass_through() {
        assert_eq!(file("mod/data/pl/pl0000.dat"), "mod/data/pl/pl0000.dat");
        assert_eq!(file("mod\\info.txt"), "mod/info.txt");
        assert_eq!(dir("mod/data/"), "mod/data");
        assert_eq!(file("unicode/\u{30c6}\u{30b9}\u{30c8}.txt"), "unicode/\u{30c6}\u{30b9}\u{30c8}.txt");
        assert_eq!(file("a b/.hidden"), "a b/.hidden");
    }

    #[test]
    fn parent_and_current_components_are_dropped_not_resolved() {
        assert_eq!(file("../evil.txt"), "evil.txt");
        assert_eq!(file("sub/../../evil.txt"), "sub/evil.txt");
        assert_eq!(file("..\\evil.txt"), "evil.txt");
        assert_eq!(file("./a/./b.txt"), "a/b.txt");
        assert_eq!(dir("../.."), "");
        assert_eq!(dir("a/../b/"), "a/b");
    }

    #[test]
    fn absolute_and_drive_paths_stay_relative() {
        assert_eq!(file("/Users/x/evil.txt"), "Users/x/evil.txt");
        assert_eq!(file("\\\\server\\share\\evil.txt"), "server/share/evil.txt");
        assert_eq!(file("C:/Users/x/evil.txt"), "C_/Users/x/evil.txt");
        assert_eq!(file("C:\\Users\\x\\evil.txt"), "C_/Users/x/evil.txt");
        assert_eq!(file("C:evil.txt"), "C_evil.txt");
        assert_eq!(file("//a///b.txt"), "a/b.txt");
    }

    #[test]
    fn invalid_characters_become_underscores() {
        assert_eq!(file("col:on.txt"), "col_on.txt");
        assert_eq!(file("quest?ion.txt"), "quest_ion.txt");
        assert_eq!(file("star*pipe|.txt"), "star_pipe_.txt");
        assert_eq!(file("a<b>c\"d.txt"), "a_b_c_d.txt");
        assert_eq!(file("tab\there\u{1}.txt"), "tab_here_.txt");
        assert_eq!(file("nul\0byte"), "nul_byte");
    }

    #[test]
    fn trailing_dots_and_spaces_become_underscores() {
        assert_eq!(file("trailing space /b.txt"), "trailing space_/b.txt");
        assert_eq!(file("trailing dot./c.txt"), "trailing dot_/c.txt");
        assert_eq!(file("two dots../d.txt"), "two dots__/d.txt");
        assert_eq!(file("file ends dot."), "file ends dot_");
        assert_eq!(file("file ends space "), "file ends space_");
        assert_eq!(file("mixed. ."), "mixed___");
        assert_eq!(file("..."), "___");
        assert_eq!(file(" "), "_");
        assert_eq!(file("in.side dots"), "in.side dots");
    }

    #[test]
    fn reserved_device_names_get_a_prefix() {
        assert_eq!(file("aux.txt"), "_aux.txt");
        assert_eq!(file("sub/NUL"), "sub/_NUL");
        assert_eq!(file("con"), "_con");
        assert_eq!(file("PRN .log"), "_PRN .log");
        assert_eq!(file("com1.dat"), "_com1.dat");
        assert_eq!(file("LPT9"), "_LPT9");
        assert_eq!(file("COM0"), "_COM0");
        assert_eq!(file("aux."), "aux_");
        assert_eq!(dir("nul/x"), "_nul/x");
        for fine in ["console.txt", "com.txt", "com10", "lpt", "auxiliary", "nulls.bin", "com1x", "\u{e9}t\u{e9}"] {
            assert_eq!(file(fine), fine);
        }
    }

    #[test]
    fn a_file_keeps_a_name_even_when_nothing_is_left() {
        assert_eq!(file(""), "_");
        assert_eq!(file(".."), "_");
        assert_eq!(file("a/.."), "a/_");
        assert_eq!(file("a/"), "a/_");
        assert_eq!(file("/"), "_");
        assert_eq!(dir(""), "");
        assert_eq!(dir("/"), "");
        assert_eq!(dir("a/"), "a");
    }

    #[test]
    fn a_literal_backslash_gets_the_private_use_stand_in() {
        let parts = entry_parts("..\\evil.txt", false, Backslash::NameCharacter);
        assert_eq!(parts, vec![format!("..{BACKSLASH_IN_NAME}evil.txt")]);
        let nested = entry_parts("dir/a\\b.txt", false, Backslash::NameCharacter);
        assert_eq!(nested, vec!["dir".to_owned(), format!("a{BACKSLASH_IN_NAME}b.txt")]);
    }

    #[test]
    fn code_pages_map_to_encodings() {
        assert_eq!(oem_encoding(932), Some(encoding_rs::SHIFT_JIS));
        assert_eq!(oem_encoding(936), Some(encoding_rs::GBK));
        assert_eq!(oem_encoding(949), Some(encoding_rs::EUC_KR));
        assert_eq!(oem_encoding(950), Some(encoding_rs::BIG5));
        assert_eq!(oem_encoding(866), Some(encoding_rs::IBM866));
        assert_eq!(oem_encoding(874), Some(encoding_rs::WINDOWS_874));
        assert_eq!(oem_encoding(1252), Some(encoding_rs::WINDOWS_1252));
        for unknown in [0, 437, 850, 852, 65001] {
            assert_eq!(oem_encoding(unknown), None, "{unknown}");
        }
    }

    #[test]
    fn legacy_names_decode_with_the_code_page_or_not_at_all() {
        let shift_jis = oem_encoding(932);
        assert_eq!(
            decode_legacy(b"sjis/\x83\x65\x83\x58\x83\x67.txt", shift_jis).as_deref(),
            Some("sjis/\u{30c6}\u{30b9}\u{30c8}.txt")
        );
        assert_eq!(
            decode_legacy(b"\x95\x5c/\x83\x5c.txt", shift_jis).as_deref(),
            Some("\u{8868}/\u{30bd}.txt")
        );
        assert_eq!(decode_legacy(b"bad\x83", shift_jis), None);
        assert_eq!(decode_legacy(b"\x8f\xe0\xe2\xa5\xe2.txt", oem_encoding(866)).as_deref(), Some("\u{41f}\u{440}\u{442}\u{435}\u{442}.txt"));
        assert_eq!(decode_legacy(b"anything", None), None);
    }
}
