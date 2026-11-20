use super::scanner::{eat_number, is_ident_part, is_ident_start};
use super::{Scanner, Token, TokenKind};

const LITERALS: [&str; 3] = ["true", "false", "null"];

fn followed_by_colon(rest: &str) -> bool {
    rest.trim_start().starts_with(':')
}

pub(super) fn tokenize(code: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(code);
    while let Some(next) = scanner.peek() {
        let start = scanner.pos();
        match next {
            '"' => {
                scanner.bump();
                scanner.eat_quoted('"', true, false);
                let kind = if followed_by_colon(scanner.rest()) {
                    TokenKind::Key
                } else {
                    TokenKind::String
                };
                scanner.emit(start, kind);
            }
            '/' if scanner.starts_with("//") => {
                scanner.eat_line();
                scanner.emit(start, TokenKind::Comment);
            }
            '-' | '0'..='9' => {
                scanner.bump();
                eat_number(&mut scanner);
                scanner.emit(start, TokenKind::Number);
            }
            '{' | '}' | '[' | ']' | ',' | ':' => scanner.emit_one(TokenKind::Punctuation),
            other if is_ident_start(other) => {
                scanner.eat_while(is_ident_part);
                let kind = if LITERALS.contains(&scanner.slice(start)) {
                    TokenKind::Literal
                } else {
                    TokenKind::Plain
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
    fn keys_values_and_literals_get_their_own_kinds() {
        let code = "{\n  \"name\": \"Pod\",\n  \"count\": -12.5e3,\n  \"on\": true,\n  \"none\": null,\n  \"list\": [1, \"two\"]\n}";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("\"name\"", TokenKind::Key)));
        assert!(found.contains(&("\"Pod\"", TokenKind::String)));
        assert!(found.contains(&("-12.5e3", TokenKind::Number)));
        assert!(found.contains(&("true", TokenKind::Literal)));
        assert!(found.contains(&("null", TokenKind::Literal)));
        assert!(found.contains(&("\"two\"", TokenKind::String)));
        assert!(found.contains(&("{", TokenKind::Punctuation)));
    }

    #[test]
    fn escaped_quotes_stay_inside_the_string() {
        let code = r#"{"a": "x \" y", "b": 1}"#;
        let tokens = tokenize(code);
        let found = kinds(code, &tokens);
        assert!(found.contains(&(r#""x \" y""#, TokenKind::String)));
        assert!(found.contains(&("\"b\"", TokenKind::Key)));
    }

    #[test]
    fn unterminated_input_is_still_covered() {
        assert_total(Language::Json, "{\"key\": [\"unterminated, 12, tru");
    }
}
