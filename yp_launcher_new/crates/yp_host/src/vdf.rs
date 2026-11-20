#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdfValue {
    String(String),
    Object(VdfObject),
}

impl VdfValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            VdfValue::String(text) => Some(text),
            VdfValue::Object(_) => None,
        }
    }

    pub fn as_object(&self) -> Option<&VdfObject> {
        match self {
            VdfValue::Object(object) => Some(object),
            VdfValue::String(_) => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VdfObject {
    entries: Vec<(String, VdfValue)>,
}

impl VdfObject {
    pub fn entries(&self) -> &[(String, VdfValue)] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&VdfValue> {
        let wanted = key.to_lowercase();
        self.entries
            .iter()
            .find(|(name, _)| name.to_lowercase() == wanted)
            .map(|(_, value)| value)
    }

    pub fn block(&self, key: &str) -> Option<&VdfObject> {
        self.get(key).and_then(VdfValue::as_object)
    }

    pub fn string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(VdfValue::as_str)
    }

    pub fn find_block(&self, key: &str) -> Option<&VdfObject> {
        self.block(key).or_else(|| {
            self.entries
                .iter()
                .filter_map(|(_, value)| value.as_object())
                .find_map(|child| child.find_block(key))
        })
    }

    pub fn blocks(&self) -> impl Iterator<Item = (&str, &VdfObject)> {
        self.entries
            .iter()
            .filter_map(|(key, value)| value.as_object().map(|object| (key.as_str(), object)))
    }

    pub fn strings_named(&self, key: &str) -> Vec<&str> {
        let wanted = key.to_lowercase();
        let mut out = Vec::new();
        self.collect_strings(&wanted, &mut out);
        out
    }

    fn collect_strings<'a>(&'a self, wanted: &str, out: &mut Vec<&'a str>) {
        for (name, value) in &self.entries {
            match value {
                VdfValue::Object(child) => child.collect_strings(wanted, out),
                VdfValue::String(text) if name.to_lowercase() == wanted => out.push(text),
                VdfValue::String(_) => {}
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VdfError {
    #[error("unterminated string on line {line}")]
    UnterminatedString { line: usize },
    #[error("unexpected '}}' on line {line}")]
    UnexpectedClose { line: usize },
    #[error("block opened on line {line} is never closed")]
    UnclosedBlock { line: usize },
    #[error("key {key:?} on line {line} has no value")]
    MissingValue { key: String, line: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text { text: String, line: usize },
    Open { line: usize },
    Close { line: usize },
    Condition,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            chars: text.strip_prefix('\u{feff}').unwrap_or(text).chars().peekable(),
            line: 1,
        }
    }

    fn bump(&mut self) -> Option<char> {
        let next = self.chars.next();
        if next == Some('\n') {
            self.line += 1;
        }
        next
    }

    fn skip_line(&mut self) {
        while let Some(c) = self.chars.peek().copied() {
            if c == '\n' {
                return;
            }
            self.bump();
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, VdfError> {
        loop {
            let Some(c) = self.chars.peek().copied() else {
                return Ok(None);
            };
            if c.is_whitespace() {
                self.bump();
                continue;
            }
            let line = self.line;
            return match c {
                '{' => {
                    self.bump();
                    Ok(Some(Token::Open { line }))
                }
                '}' => {
                    self.bump();
                    Ok(Some(Token::Close { line }))
                }
                '"' => {
                    self.bump();
                    self.quoted(line).map(Some)
                }
                '[' => {
                    self.condition();
                    Ok(Some(Token::Condition))
                }
                '/' => {
                    self.bump();
                    if self.chars.peek() == Some(&'/') {
                        self.skip_line();
                        continue;
                    }
                    let mut text = String::from('/');
                    self.unquoted_into(&mut text);
                    Ok(Some(Token::Text { text, line }))
                }
                _ => {
                    let mut text = String::new();
                    self.unquoted_into(&mut text);
                    Ok(Some(Token::Text { text, line }))
                }
            };
        }
    }

    fn quoted(&mut self, line: usize) -> Result<Token, VdfError> {
        let mut text = String::new();
        loop {
            match self.bump() {
                None => return Err(VdfError::UnterminatedString { line }),
                Some('"') => return Ok(Token::Text { text, line }),
                Some('\\') => match self.chars.peek().copied() {
                    Some(escaped @ ('\\' | '"')) => {
                        self.bump();
                        text.push(escaped);
                    }
                    Some('n') => {
                        self.bump();
                        text.push('\n');
                    }
                    Some('t') => {
                        self.bump();
                        text.push('\t');
                    }
                    _ => text.push('\\'),
                },
                Some(other) => text.push(other),
            }
        }
    }

    fn unquoted_into(&mut self, text: &mut String) {
        while let Some(c) = self.chars.peek().copied() {
            if c.is_whitespace() || matches!(c, '{' | '}' | '"') {
                return;
            }
            text.push(c);
            self.bump();
        }
    }

    fn condition(&mut self) {
        while let Some(c) = self.bump() {
            if c == ']' || c == '\n' {
                return;
            }
        }
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Token>,
}

impl<'a> Parser<'a> {
    fn next(&mut self) -> Result<Option<Token>, VdfError> {
        match self.peeked.take() {
            Some(token) => Ok(Some(token)),
            None => self.lexer.next_token(),
        }
    }

    fn next_significant(&mut self) -> Result<Option<Token>, VdfError> {
        loop {
            match self.next()? {
                Some(Token::Condition) => continue,
                other => return Ok(other),
            }
        }
    }

    fn skip_conditions(&mut self) -> Result<(), VdfError> {
        loop {
            match self.next()? {
                Some(Token::Condition) => continue,
                other => {
                    self.peeked = other;
                    return Ok(());
                }
            }
        }
    }

    fn object(&mut self, opened_at: Option<usize>) -> Result<VdfObject, VdfError> {
        let mut object = VdfObject::default();
        loop {
            let Some(token) = self.next_significant()? else {
                return match opened_at {
                    Some(line) => Err(VdfError::UnclosedBlock { line }),
                    None => Ok(object),
                };
            };
            let (key, line) = match token {
                Token::Close { line } => {
                    return match opened_at {
                        Some(_) => Ok(object),
                        None => Err(VdfError::UnexpectedClose { line }),
                    };
                }
                Token::Open { line } => {
                    let nested = self.object(Some(line))?;
                    object.entries.push((String::new(), VdfValue::Object(nested)));
                    continue;
                }
                Token::Text { text, line, .. } => (text, line),
                Token::Condition => continue,
            };
            if is_directive(&key) {
                self.next_significant()?;
                continue;
            }
            match self.next_significant()? {
                Some(Token::Open { line }) => {
                    let nested = self.object(Some(line))?;
                    self.skip_conditions()?;
                    object.entries.push((key, VdfValue::Object(nested)));
                }
                Some(Token::Text { text, .. }) => {
                    self.skip_conditions()?;
                    object.entries.push((key, VdfValue::String(text)));
                }
                Some(Token::Close { .. }) | None => return Err(VdfError::MissingValue { key, line }),
                Some(Token::Condition) => {}
            }
        }
    }
}

fn is_directive(key: &str) -> bool {
    matches!(key.to_lowercase().as_str(), "#base" | "#include")
}

pub fn parse(text: &str) -> Result<VdfObject, VdfError> {
    Parser {
        lexer: Lexer::new(text),
        peeked: None,
    }
    .object(None)
}

pub fn path_entries(text: &str) -> Vec<String> {
    parse(text)
        .map(|root| root.strings_named("path").into_iter().map(str::to_owned).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_blocks_with_escapes_comments_and_conditions() {
        let root = parse(
            "\u{feff}#base \"other.vdf\"\n\"Root\" // comment\n{\n  \"Key\" \"a\\\\b\\\"c\" [$WIN32]\n  unquoted value\n  \"Child\" [!$X360] { \"n\" \"1\" }\n}\n",
        )
        .unwrap();
        let block = root.block("root").unwrap();
        assert_eq!(block.string("KEY"), Some("a\\b\"c"));
        assert_eq!(block.string("unquoted"), Some("value"));
        assert_eq!(block.block("child").unwrap().string("n"), Some("1"));
        assert_eq!(root.find_block("child").unwrap().string("N"), Some("1"));
    }

    #[test]
    fn duplicate_keys_are_kept_and_lookup_returns_the_first() {
        let root = parse("\"a\" \"1\"\n\"A\" \"2\"\n").unwrap();
        assert_eq!(root.entries().len(), 2);
        assert_eq!(root.string("a"), Some("1"));
    }

    #[test]
    fn reports_structural_errors() {
        assert_eq!(parse("\"a\" \"b"), Err(VdfError::UnterminatedString { line: 1 }));
        assert_eq!(parse("\"a\"\n{\n"), Err(VdfError::UnclosedBlock { line: 2 }));
        assert_eq!(parse("}"), Err(VdfError::UnexpectedClose { line: 1 }));
        assert!(matches!(parse("\"a\""), Err(VdfError::MissingValue { .. })));
    }

    #[test]
    fn unknown_escapes_keep_the_backslash() {
        let root = parse(r#""p" "C:\Games\x""#).unwrap();
        assert_eq!(root.string("p"), Some(r"C:\Games\x"));
    }
}
