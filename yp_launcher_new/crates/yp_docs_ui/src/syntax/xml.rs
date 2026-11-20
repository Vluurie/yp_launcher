use super::{Scanner, Token, TokenKind};

fn is_name(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-' | ':' | '.')
}

fn tag(scanner: &mut Scanner<'_>) {
    let start = scanner.pos();
    scanner.bump();
    if scanner.peek() == Some('/') {
        scanner.bump();
    }
    scanner.emit(start, TokenKind::Punctuation);
    let name = scanner.pos();
    scanner.eat_while(is_name);
    scanner.emit(name, TokenKind::Tag);
    while let Some(next) = scanner.peek() {
        let start = scanner.pos();
        match next {
            '>' => {
                scanner.emit_one(TokenKind::Punctuation);
                return;
            }
            '<' => return,
            '/' | '=' => scanner.emit_one(TokenKind::Punctuation),
            '"' | '\'' => {
                scanner.bump();
                scanner.eat_quoted(next, false, true);
                scanner.emit(start, TokenKind::String);
            }
            other if other.is_whitespace() => {
                scanner.eat_while(char::is_whitespace);
                scanner.emit(start, TokenKind::Plain);
            }
            other if is_name(other) => {
                scanner.eat_while(is_name);
                scanner.emit(start, TokenKind::Attribute);
            }
            _ => scanner.emit_one(TokenKind::Plain),
        }
    }
}

fn entity(scanner: &mut Scanner<'_>) {
    let start = scanner.pos();
    scanner.bump();
    let name = scanner.eat_while(|next| next.is_ascii_alphanumeric() || next == '#');
    if name > 0 && scanner.peek() == Some(';') {
        scanner.bump();
        scanner.emit(start, TokenKind::Symbol);
    } else {
        scanner.emit(start, TokenKind::Plain);
    }
}

pub(super) fn tokenize(code: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(code);
    while let Some(next) = scanner.peek() {
        let start = scanner.pos();
        if scanner.eat("<!--") {
            scanner.eat_until("-->");
            scanner.emit(start, TokenKind::Comment);
        } else if scanner.eat("<![CDATA[") {
            scanner.eat_until("]]>");
            scanner.emit(start, TokenKind::String);
        } else if scanner.eat("<?") {
            scanner.eat_until("?>");
            scanner.emit(start, TokenKind::Meta);
        } else if scanner.eat("<!") {
            scanner.eat_until(">");
            scanner.emit(start, TokenKind::Meta);
        } else if next == '<' {
            tag(&mut scanner);
        } else if next == '&' {
            entity(&mut scanner);
        } else {
            scanner.eat_while(|next| next != '<' && next != '&');
            scanner.emit(start, TokenKind::Plain);
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
    fn tags_attributes_and_text_are_told_apart() {
        let code = "<?xml version=\"1.0\"?>\n<!-- area -->\n<action>\n\t<code str=\"app::area::BoxArea\">0x18cffd98</code>\n\t<empty/>\n\t<name>a &amp; b</name>\n</action>";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("<?xml version=\"1.0\"?>", TokenKind::Meta)));
        assert!(found.contains(&("<!-- area -->", TokenKind::Comment)));
        assert!(found.contains(&("action", TokenKind::Tag)));
        assert!(found.contains(&("code", TokenKind::Tag)));
        assert!(found.contains(&("str", TokenKind::Attribute)));
        assert!(found.contains(&("\"app::area::BoxArea\"", TokenKind::String)));
        assert!(found.contains(&("0x18cffd98", TokenKind::Plain)));
        assert!(found.contains(&("empty", TokenKind::Tag)));
        assert!(found.contains(&("&amp;", TokenKind::Symbol)));
        assert!(found.contains(&("</", TokenKind::Punctuation)));
    }

    #[test]
    fn cdata_doctype_and_stray_ampersands() {
        let code = "<!DOCTYPE x><a><![CDATA[1 < 2]]></a> AT&T & co";
        let tokens = tokenize(code);
        assert_covers(code, &tokens);
        let found = kinds(code, &tokens);
        assert!(found.contains(&("<!DOCTYPE x>", TokenKind::Meta)));
        assert!(found.contains(&("<![CDATA[1 < 2]]>", TokenKind::String)));
        assert!(!found.iter().any(|(_, kind)| *kind == TokenKind::Symbol));
    }

    #[test]
    fn broken_markup_is_still_covered() {
        assert_total(Language::Xml, "<a b='1\n<c d=\"x\" e></c>< /> &#x; <!-- open");
    }
}
