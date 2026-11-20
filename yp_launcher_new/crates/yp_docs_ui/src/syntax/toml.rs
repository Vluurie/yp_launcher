use super::scanner::{is_ident_part, is_ident_start};
use super::{Scanner, Token, TokenKind};

const INLINE_TABLE: char = '{';
const ARRAY: char = '[';

fn is_bare_key(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '-'
}

fn is_number_part(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '.' | ':' | '+' | '-' | '_')
}

fn eat_string(scanner: &mut Scanner<'_>, quote: char) {
    let triple = if quote == '"' { "\"\"\"" } else { "'''" };
    if scanner.eat(triple) {
        scanner.eat_until(triple);
    } else {
        scanner.bump();
        scanner.eat_quoted(quote, quote == '"', false);
    }
}

fn eat_header(scanner: &mut Scanner<'_>) {
    scanner.eat_while(|next| next == '[');
    loop {
        match scanner.peek() {
            None | Some('\n') => return,
            Some(']') => {
                scanner.eat_while(|next| next == ']');
                return;
            }
            Some('"') => {
                scanner.bump();
                scanner.eat_quoted('"', true, false);
            }
            Some('\'') => {
                scanner.bump();
                scanner.eat_quoted('\'', false, false);
            }
            Some(_) => {
                scanner.bump();
            }
        }
    }
}

pub(super) fn tokenize(code: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(code);
    let mut containers: Vec<char> = Vec::new();
    let mut expect_key = true;
    while let Some(next) = scanner.peek() {
        let start = scanner.pos();
        match next {
            '\n' => {
                scanner.emit_one(TokenKind::Plain);
                if containers.is_empty() {
                    expect_key = true;
                }
            }
            other if other.is_whitespace() => {
                scanner.eat_while(|next| next.is_whitespace() && next != '\n');
                scanner.emit(start, TokenKind::Plain);
            }
            '#' => {
                scanner.eat_line();
                scanner.emit(start, TokenKind::Comment);
            }
            '[' if expect_key && containers.is_empty() => {
                eat_header(&mut scanner);
                scanner.emit(start, TokenKind::Section);
                expect_key = false;
            }
            '"' | '\'' => {
                eat_string(&mut scanner, next);
                let kind = if expect_key {
                    TokenKind::Key
                } else {
                    TokenKind::String
                };
                scanner.emit(start, kind);
            }
            '=' => {
                scanner.emit_one(TokenKind::Punctuation);
                expect_key = false;
            }
            '.' if expect_key => scanner.emit_one(TokenKind::Punctuation),
            '{' => {
                scanner.emit_one(TokenKind::Punctuation);
                containers.push(INLINE_TABLE);
                expect_key = true;
            }
            '[' => {
                scanner.emit_one(TokenKind::Punctuation);
                containers.push(ARRAY);
                expect_key = false;
            }
            '}' | ']' => {
                scanner.emit_one(TokenKind::Punctuation);
                containers.pop();
                expect_key = false;
            }
            ',' => {
                scanner.emit_one(TokenKind::Punctuation);
                expect_key = containers.last() == Some(&INLINE_TABLE);
            }
            other if expect_key && is_bare_key(other) => {
                scanner.eat_while(is_bare_key);
                scanner.emit(start, TokenKind::Key);
            }
            '+' | '-' | '0'..='9' => {
                scanner.bump();
                scanner.eat_while(is_number_part);
                scanner.emit(start, TokenKind::Number);
            }
            other if is_ident_start(other) => {
                scanner.eat_while(is_ident_part);
                let kind = match scanner.slice(start) {
                    "true" | "false" => TokenKind::Literal,
                    "inf" | "nan" => TokenKind::Number,
                    _ => TokenKind::Plain,
                };
                scanner.emit(start, kind);
            }
            _ => scanner.emit_one(TokenKind::Plain),
        }
    }
    scanner.finish()
}

#[cfg(test)]
mod tests {
    use super::super::testing::{assert_covers, assert_total, kinds};
    use super::super::{Language, TokenKind};
    use super::*;

    #[test]
    fn headers_keys_and_values_are_told_apart() {
        let code = "# manifest\nid = \"my_mod\"\nversion = 1.5\nobj_id = 0x30003\nenabled = true\n\n[text.name]\n_us = 'Pebble'\n\n[[shop_item]]\nstate = 3 # late\n";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("# manifest", TokenKind::Comment)));
        assert!(found.contains(&("id", TokenKind::Key)));
        assert!(found.contains(&("\"my_mod\"", TokenKind::String)));
        assert!(found.contains(&("1.5", TokenKind::Number)));
        assert!(found.contains(&("0x30003", TokenKind::Number)));
        assert!(found.contains(&("true", TokenKind::Literal)));
        assert!(found.contains(&("[text.name]", TokenKind::Section)));
        assert!(found.contains(&("_us", TokenKind::Key)));
        assert!(found.contains(&("'Pebble'", TokenKind::String)));
        assert!(found.contains(&("[[shop_item]]", TokenKind::Section)));
        assert!(found.contains(&("# late", TokenKind::Comment)));
    }

    #[test]
    fn arrays_and_inline_tables_keep_keys_and_values_straight() {
        let code = "meshes = [\"Body\", \"Hair\"]\nlevels = [\n  { min_damage = 100, max_damage = 140 },\n]\nwhen = { player = \"2B\" }\n";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("meshes", TokenKind::Key)));
        assert!(found.contains(&("\"Body\"", TokenKind::String)));
        assert!(found.contains(&("\"Hair\"", TokenKind::String)));
        assert!(found.contains(&("min_damage", TokenKind::Key)));
        assert!(found.contains(&("max_damage", TokenKind::Key)));
        assert!(found.contains(&("140", TokenKind::Number)));
        assert!(found.contains(&("player", TokenKind::Key)));
        assert!(found.contains(&("\"2B\"", TokenKind::String)));
        assert!(!found.iter().any(|(text, kind)| *text == "[" && *kind == TokenKind::Section));
    }

    #[test]
    fn quoted_and_dotted_keys_and_multi_line_strings() {
        let code = "\"quoted key\" = 1\na.b = 2\nhelp = \"\"\"\nline one\nline two\"\"\"\nnext = -3\n";
        let tokens = tokenize(code);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("\"quoted key\"", TokenKind::Key)));
        assert!(found.contains(&("a", TokenKind::Key)));
        assert!(found.contains(&("b", TokenKind::Key)));
        assert!(found.contains(&("\"\"\"\nline one\nline two\"\"\"", TokenKind::String)));
        assert!(found.contains(&("next", TokenKind::Key)));
        assert!(found.contains(&("-3", TokenKind::Number)));
    }

    #[test]
    fn truncated_documents_are_still_covered() {
        assert_total(Language::Toml, "[a\nb = \"x\nc = [1, { d = '");
    }
}
