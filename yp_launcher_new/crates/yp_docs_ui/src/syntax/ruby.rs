use super::scanner::{eat_number, is_ident_part, is_ident_start};
use super::{Scanner, Token, TokenKind};

const KEYWORDS: [&str; 37] = [
    "alias", "and", "begin", "BEGIN", "break", "case", "class", "def", "defined?", "do", "else",
    "elsif", "end", "END", "ensure", "for", "if", "in", "module", "next", "not", "or", "redo",
    "rescue", "retry", "return", "self", "super", "then", "undef", "unless", "until", "when",
    "while", "yield", "__FILE__", "__LINE__",
];

const LITERALS: [&str; 3] = ["true", "false", "nil"];

const BUILTINS: [&str; 18] = [
    "puts",
    "print",
    "p",
    "require",
    "require_relative",
    "include",
    "extend",
    "attr_accessor",
    "attr_reader",
    "attr_writer",
    "raise",
    "lambda",
    "proc",
    "format",
    "sprintf",
    "rand",
    "sleep",
    "loop",
];

const BLOCK_COMMENT_OPEN: &str = "=begin";
const BLOCK_COMMENT_CLOSE: &str = "\n=end";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Naming {
    None,
    Definition,
}

fn at_line_start(scanner: &Scanner<'_>) -> bool {
    matches!(scanner.previous(), None | Some('\n'))
}

fn eat_identifier(scanner: &mut Scanner<'_>) {
    scanner.eat_while(is_ident_part);
    if matches!(scanner.peek(), Some('?' | '!')) && scanner.peek_second() != Some('=') {
        scanner.bump();
    }
}

fn symbol(scanner: &mut Scanner<'_>) {
    let start = scanner.pos();
    match scanner.peek_second() {
        Some(':') => {
            scanner.skip(2);
            scanner.emit(start, TokenKind::Plain);
        }
        Some(next) if is_ident_start(next) => {
            scanner.bump();
            eat_identifier(scanner);
            scanner.emit(start, TokenKind::Symbol);
        }
        Some(quote @ ('"' | '\'')) => {
            scanner.skip(2);
            scanner.eat_quoted(quote, true, false);
            scanner.emit(start, TokenKind::Symbol);
        }
        _ => scanner.emit_one(TokenKind::Plain),
    }
}

fn word(scanner: &mut Scanner<'_>, naming: &mut Naming) {
    let start = scanner.pos();
    let after_dot = scanner.previous() == Some('.');
    eat_identifier(scanner);
    let text = scanner.slice(start);
    if scanner.peek() == Some(':') && scanner.peek_second() != Some(':') && !after_dot {
        scanner.bump();
        scanner.emit(start, TokenKind::Symbol);
        return;
    }
    let kind = if LITERALS.contains(&text) {
        TokenKind::Literal
    } else if KEYWORDS.contains(&text) && !after_dot {
        if matches!(text, "def" | "class" | "module") {
            *naming = Naming::Definition;
            scanner.emit(start, TokenKind::Keyword);
            return;
        }
        TokenKind::Keyword
    } else if *naming == Naming::Definition {
        TokenKind::Section
    } else if BUILTINS.contains(&text) && !after_dot {
        TokenKind::Builtin
    } else {
        TokenKind::Plain
    };
    if text != "self" {
        *naming = Naming::None;
    }
    scanner.emit(start, kind);
}

pub(super) fn tokenize(code: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(code);
    let mut naming = Naming::None;
    while let Some(next) = scanner.peek() {
        let start = scanner.pos();
        match next {
            '#' => {
                scanner.eat_line();
                scanner.emit(start, TokenKind::Comment);
            }
            '=' if at_line_start(&scanner) && scanner.starts_with(BLOCK_COMMENT_OPEN) => {
                scanner.eat_until(BLOCK_COMMENT_CLOSE);
                scanner.eat_line();
                scanner.emit(start, TokenKind::Comment);
            }
            '"' | '\'' | '`' => {
                scanner.bump();
                scanner.eat_quoted(next, true, true);
                scanner.emit(start, TokenKind::String);
                naming = Naming::None;
            }
            ':' => symbol(&mut scanner),
            '@' => {
                scanner.eat_while(|next| next == '@');
                scanner.eat_while(is_ident_part);
                scanner.emit(start, TokenKind::Variable);
            }
            '$' => {
                scanner.bump();
                if scanner.eat_while(is_ident_part) == 0 {
                    scanner.bump();
                }
                scanner.emit(start, TokenKind::Variable);
            }
            '0'..='9' => {
                eat_number(&mut scanner);
                scanner.emit(start, TokenKind::Number);
            }
            '\n' | ';' | '(' => {
                naming = Naming::None;
                scanner.emit_one(TokenKind::Plain);
            }
            other if is_ident_start(other) => word(&mut scanner, &mut naming),
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
    fn definitions_calls_and_literals_are_told_apart() {
        let code = "# talk\ndef famima_ON_TALK_a1()\n    set_scene_state(\"q005/RUN_1\")\n    quest_mount(20737)\n    if @rate > 0.5 and language() == 4\n        mess_ex([ :FOR_2B, :FOR_9S ], nil, true)\n    end\nend\n";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("# talk", TokenKind::Comment)));
        assert!(found.contains(&("def", TokenKind::Keyword)));
        assert!(found.contains(&("famima_ON_TALK_a1", TokenKind::Section)));
        assert!(found.contains(&("\"q005/RUN_1\"", TokenKind::String)));
        assert!(found.contains(&("20737", TokenKind::Number)));
        assert!(found.contains(&("if", TokenKind::Keyword)));
        assert!(found.contains(&("@rate", TokenKind::Variable)));
        assert!(found.contains(&("0.5", TokenKind::Number)));
        assert!(found.contains(&("and", TokenKind::Keyword)));
        assert!(found.contains(&(":FOR_2B", TokenKind::Symbol)));
        assert!(found.contains(&("nil", TokenKind::Literal)));
        assert!(found.contains(&("true", TokenKind::Literal)));
        assert!(found.contains(&("end", TokenKind::Keyword)));
        assert!(found.iter().any(|(text, kind)| text.contains("set_scene_state")
            && *kind == TokenKind::Plain));
    }

    #[test]
    fn classes_methods_on_self_and_hash_keys() {
        let code = "class Door < Base\n  def self.open?(speed: 1)\n    puts \"it's #{speed}\"\n    obj.class\n  end\nend\n$debug = Foo::BAR\n";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("class", TokenKind::Keyword)));
        assert!(found.contains(&("Door", TokenKind::Section)));
        assert!(found.contains(&("self", TokenKind::Keyword)));
        assert!(found.contains(&("open?", TokenKind::Section)));
        assert!(found.contains(&("speed:", TokenKind::Symbol)));
        assert!(found.contains(&("puts", TokenKind::Builtin)));
        assert!(found.contains(&("\"it's #{speed}\"", TokenKind::String)));
        assert!(found.contains(&("$debug", TokenKind::Variable)));
        assert!(!found.iter().any(|(text, kind)| *text == "Base" && *kind == TokenKind::Section));
        assert_eq!(
            found
                .iter()
                .filter(|(text, kind)| *text == "class" && *kind == TokenKind::Keyword)
                .count(),
            1
        );
    }

    #[test]
    fn block_comments_and_non_ascii_strings() {
        let code = "=begin\nnotes: \"x\"\n=end\nnovel_ex(\"風呂へ行く\", [ :CLEAR ])\n";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("=begin\nnotes: \"x\"\n=end", TokenKind::Comment)));
        assert!(found.contains(&("\"風呂へ行く\"", TokenKind::String)));
        assert!(found.contains(&(":CLEAR", TokenKind::Symbol)));
    }

    #[test]
    fn truncated_scripts_are_still_covered() {
        assert_total(Language::Ruby, "def x(a = \"un\nif :\"sym @@v $ 0x1f 1.5e+3 :: a ? b : c");
    }
}
