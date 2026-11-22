mod json;
mod ruby;
mod scanner;
mod toml;
mod xml;

use std::ops::Range;

use gpui::Hsla;
use yp_ui::ThemeColors;

pub(crate) use scanner::Scanner;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Plain,
    Comment,
    String,
    Number,
    Keyword,
    Literal,
    Symbol,
    Section,
    Key,
    Tag,
    Attribute,
    Variable,
    Builtin,
    Meta,
    Punctuation,
}

impl TokenKind {
    pub fn color(self, colors: &ThemeColors) -> Hsla {
        match self {
            TokenKind::Plain | TokenKind::Punctuation => colors.text_secondary,
            TokenKind::Comment | TokenKind::Meta => colors.text_muted,
            TokenKind::String => colors.success,
            TokenKind::Number | TokenKind::Key | TokenKind::Attribute | TokenKind::Builtin => {
                colors.accent_primary
            }
            TokenKind::Keyword | TokenKind::Literal | TokenKind::Symbol => colors.error,
            TokenKind::Section | TokenKind::Tag => colors.warning,
            TokenKind::Variable => colors.text_primary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub range: Range<usize>,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Ruby,
    Toml,
    Xml,
    Json,
}

impl Language {
    pub const ALL: [Language; 4] = [
        Language::Ruby,
        Language::Toml,
        Language::Xml,
        Language::Json,
    ];

    pub fn from_fence(info: &str) -> Option<Language> {
        let tag = info
            .split([' ', '\t', ','])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        match tag.as_str() {
            "ruby" | "rb" => Some(Language::Ruby),
            "toml" | "ini" => Some(Language::Toml),
            "xml" | "html" => Some(Language::Xml),
            "json" => Some(Language::Json),
            _ => None,
        }
    }
}

pub fn tokenize(language: Language, code: &str) -> Vec<Token> {
    match language {
        Language::Ruby => ruby::tokenize(code),
        Language::Toml => toml::tokenize(code),
        Language::Xml => xml::tokenize(code),
        Language::Json => json::tokenize(code),
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;

    pub fn kinds<'a>(code: &'a str, tokens: &[Token]) -> Vec<(&'a str, TokenKind)> {
        tokens
            .iter()
            .filter(|token| !code[token.range.clone()].trim().is_empty())
            .map(|token| (&code[token.range.clone()], token.kind))
            .collect()
    }

    pub fn assert_covers(code: &str, tokens: &[Token]) {
        let mut cursor = 0;
        for token in tokens {
            assert_eq!(token.range.start, cursor, "gap before {token:?} in {code:?}");
            assert!(token.range.end > token.range.start, "empty token in {code:?}");
            assert!(code.is_char_boundary(token.range.end));
            cursor = token.range.end;
        }
        assert_eq!(cursor, code.len(), "tokens stop early in {code:?}");
    }

    pub fn assert_total(language: Language, code: &str) {
        let boundaries: Vec<usize> = code
            .char_indices()
            .map(|(index, _)| index)
            .chain([code.len()])
            .collect();
        for end in boundaries {
            let prefix = &code[..end];
            assert_covers(prefix, &tokenize(language, prefix));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fence_tags_map_to_the_languages_the_docs_use() {
        assert_eq!(Language::from_fence("ruby"), Some(Language::Ruby));
        assert_eq!(Language::from_fence("RB"), Some(Language::Ruby));
        assert_eq!(Language::from_fence("toml title=x"), Some(Language::Toml));
        assert_eq!(Language::from_fence("ini"), Some(Language::Toml));
        assert_eq!(Language::from_fence("html"), Some(Language::Xml));
        assert_eq!(Language::from_fence("json"), Some(Language::Json));
        assert_eq!(Language::from_fence(""), None);
        assert_eq!(Language::from_fence("text"), None);
    }

    #[test]
    fn every_language_survives_garbage_and_truncated_input() {
        let samples = [
            "",
            "\"",
            "'''",
            "<",
            "<!--",
            "<a b=\"",
            "[[",
            "{\"a\":",
            "#",
            ":",
            "@",
            "=begin",
            "\\",
            "\u{feff}é文\t\r\n\"é",
            "a = \"\"\"x",
            "0x",
            "1e",
            "</",
            "<?",
            "&",
            "]]>",
        ];
        for language in Language::ALL {
            for sample in samples {
                testing::assert_total(language, sample);
            }
        }
    }
}
