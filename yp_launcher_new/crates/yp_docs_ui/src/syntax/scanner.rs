use super::{Token, TokenKind};

pub(crate) struct Scanner<'a> {
    text: &'a str,
    pos: usize,
    tokens: Vec<Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            pos: 0,
            tokens: Vec::new(),
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn rest(&self) -> &'a str {
        &self.text[self.pos..]
    }

    pub fn slice(&self, start: usize) -> &'a str {
        &self.text[start..self.pos]
    }

    pub fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    pub fn peek_second(&self) -> Option<char> {
        self.rest().chars().nth(1)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.rest().starts_with(prefix)
    }

    pub fn previous(&self) -> Option<char> {
        self.text[..self.pos].chars().next_back()
    }

    pub fn bump(&mut self) -> Option<char> {
        let next = self.peek()?;
        self.pos += next.len_utf8();
        Some(next)
    }

    pub fn skip(&mut self, bytes: usize) {
        let mut target = (self.pos + bytes).min(self.text.len());
        while !self.text.is_char_boundary(target) {
            target += 1;
        }
        self.pos = target;
    }

    pub fn eat(&mut self, prefix: &str) -> bool {
        if self.starts_with(prefix) {
            self.pos += prefix.len();
            true
        } else {
            false
        }
    }

    pub fn eat_while(&mut self, mut accept: impl FnMut(char) -> bool) -> usize {
        let start = self.pos;
        while let Some(next) = self.peek() {
            if !accept(next) {
                break;
            }
            self.pos += next.len_utf8();
        }
        self.pos - start
    }

    pub fn eat_until(&mut self, terminator: &str) -> bool {
        match self.rest().find(terminator) {
            Some(offset) => {
                self.pos += offset + terminator.len();
                true
            }
            None => {
                self.pos = self.text.len();
                false
            }
        }
    }

    pub fn eat_line(&mut self) {
        self.eat_while(|next| next != '\n');
    }

    pub fn eat_quoted(&mut self, quote: char, escapes: bool, multi_line: bool) {
        while let Some(next) = self.peek() {
            if next == quote {
                self.bump();
                return;
            }
            if next == '\n' && !multi_line {
                return;
            }
            self.bump();
            if next == '\\' && escapes {
                self.bump();
            }
        }
    }

    pub fn emit(&mut self, start: usize, kind: TokenKind) {
        if start >= self.pos {
            return;
        }
        match self.tokens.last_mut() {
            Some(last) if last.kind == kind && last.range.end == start => {
                last.range.end = self.pos;
            }
            _ => self.tokens.push(Token {
                range: start..self.pos,
                kind,
            }),
        }
    }

    pub fn emit_one(&mut self, kind: TokenKind) {
        let start = self.pos;
        self.bump();
        self.emit(start, kind);
    }

    pub fn finish(mut self) -> Vec<Token> {
        let covered = self.tokens.last().map_or(0, |token| token.range.end);
        if covered < self.text.len() {
            self.pos = self.text.len();
            self.emit(covered, TokenKind::Plain);
        }
        self.tokens
    }
}

pub(crate) fn is_ident_start(character: char) -> bool {
    character.is_alphabetic() || character == '_'
}

pub(crate) fn is_ident_part(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

pub(crate) fn eat_number(scanner: &mut Scanner<'_>) {
    if scanner.starts_with("0x") || scanner.starts_with("0X") {
        scanner.skip(2);
        scanner.eat_while(|next| next.is_ascii_hexdigit() || next == '_');
        return;
    }
    scanner.eat_while(|next| next.is_ascii_digit() || next == '_');
    if scanner.peek() == Some('.') && scanner.peek_second().is_some_and(|next| next.is_ascii_digit())
    {
        scanner.bump();
        scanner.eat_while(|next| next.is_ascii_digit() || next == '_');
    }
    if matches!(scanner.peek(), Some('e' | 'E')) {
        let mut ahead = scanner.rest().chars().skip(1);
        let exponent = match ahead.next() {
            Some('+' | '-') => ahead.next(),
            other => other,
        };
        if exponent.is_some_and(|next| next.is_ascii_digit()) {
            scanner.bump();
            if matches!(scanner.peek(), Some('+' | '-')) {
                scanner.bump();
            }
            scanner.eat_while(|next| next.is_ascii_digit());
        }
    }
}
