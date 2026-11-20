use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    pub const ALL: [PluralCategory; 6] = [
        PluralCategory::Zero,
        PluralCategory::One,
        PluralCategory::Two,
        PluralCategory::Few,
        PluralCategory::Many,
        PluralCategory::Other,
    ];

    pub fn keyword(self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }

    pub fn from_keyword(keyword: &str) -> Option<PluralCategory> {
        PluralCategory::ALL
            .into_iter()
            .find(|category| category.keyword() == keyword)
    }

    pub fn variant_name(self) -> &'static str {
        match self {
            PluralCategory::Zero => "Zero",
            PluralCategory::One => "One",
            PluralCategory::Two => "Two",
            PluralCategory::Few => "Few",
            PluralCategory::Many => "Many",
            PluralCategory::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Selector {
    Exact(u32),
    Category(PluralCategory),
}

impl Selector {
    pub fn explicit_number(self) -> Option<u32> {
        match self {
            Selector::Exact(value) => Some(value),
            Selector::Category(PluralCategory::Zero) => Some(0),
            Selector::Category(PluralCategory::One) => Some(1),
            Selector::Category(PluralCategory::Two) => Some(2),
            Selector::Category(_) => None,
        }
    }

    pub fn is_other(self) -> bool {
        self == Selector::Category(PluralCategory::Other)
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selector::Exact(value) => write!(f, "={value}"),
            Selector::Category(category) => f.write_str(category.keyword()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluralCase {
    pub selector: Selector,
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plural {
    pub argument: String,
    pub cases: Vec<PluralCase>,
}

impl Plural {
    pub fn other(&self) -> &Message {
        self.cases
            .iter()
            .find(|case| case.selector.is_other())
            .map(|case| &case.message)
            .unwrap_or(&EMPTY_MESSAGE)
    }

    pub fn explicit(&self, value: i64) -> Option<&Message> {
        self.cases
            .iter()
            .find(|case| case.selector.explicit_number().map(i64::from) == Some(value))
            .map(|case| &case.message)
    }

    pub fn category(&self, category: PluralCategory) -> Option<&Message> {
        self.cases
            .iter()
            .find(|case| case.selector == Selector::Category(category))
            .map(|case| &case.message)
    }

    pub fn explicit_numbers(&self) -> BTreeSet<u32> {
        self.cases
            .iter()
            .filter_map(|case| case.selector.explicit_number())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Text(String),
    Argument(String),
    Plural(Plural),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Message {
    pub parts: Vec<Part>,
}

static EMPTY_MESSAGE: Message = Message { parts: Vec::new() };

impl Message {
    pub fn is_plain_text(&self) -> bool {
        self.parts.iter().all(|part| matches!(part, Part::Text(_)))
    }

    pub fn plain_text(&self) -> Option<String> {
        let mut text = String::new();
        for part in &self.parts {
            match part {
                Part::Text(chunk) => text.push_str(chunk),
                Part::Argument(_) | Part::Plural(_) => return None,
            }
        }
        Some(text)
    }

    pub fn arguments(&self) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        self.collect_arguments(&mut names);
        names
    }

    fn collect_arguments(&self, names: &mut BTreeSet<String>) {
        for part in &self.parts {
            match part {
                Part::Text(_) => {}
                Part::Argument(name) => {
                    names.insert(name.clone());
                }
                Part::Plural(plural) => {
                    names.insert(plural.argument.clone());
                    for case in &plural.cases {
                        case.message.collect_arguments(names);
                    }
                }
            }
        }
    }

    pub fn plurals(&self) -> Vec<&Plural> {
        let mut found = Vec::new();
        self.collect_plurals(&mut found);
        found
    }

    fn collect_plurals<'a>(&'a self, found: &mut Vec<&'a Plural>) {
        for part in &self.parts {
            if let Part::Plural(plural) = part {
                found.push(plural);
                for case in &plural.cases {
                    case.message.collect_plurals(found);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IcuErrorKind {
    UnexpectedEnd,
    UnexpectedChar { found: char, expected: &'static str },
    StrayClosingBrace,
    InvalidArgumentName,
    UnsupportedArgumentType(String),
    InvalidSelector(String),
    DuplicateSelector(String),
    MissingOther,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcuError {
    pub offset: usize,
    pub kind: IcuErrorKind,
}

impl fmt::Display for IcuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            IcuErrorKind::UnexpectedEnd => {
                write!(f, "unexpected end of message at byte {}", self.offset)
            }
            IcuErrorKind::UnexpectedChar { found, expected } => {
                write!(
                    f,
                    "unexpected {found:?} at byte {}, expected {expected}",
                    self.offset
                )
            }
            IcuErrorKind::StrayClosingBrace => write!(f, "unmatched '}}' at byte {}", self.offset),
            IcuErrorKind::InvalidArgumentName => {
                write!(f, "invalid argument name at byte {}", self.offset)
            }
            IcuErrorKind::UnsupportedArgumentType(kind) => {
                write!(
                    f,
                    "unsupported argument type {kind:?} at byte {}",
                    self.offset
                )
            }
            IcuErrorKind::InvalidSelector(selector) => {
                write!(
                    f,
                    "invalid plural selector {selector:?} at byte {}",
                    self.offset
                )
            }
            IcuErrorKind::DuplicateSelector(selector) => {
                write!(
                    f,
                    "duplicate plural selector {selector:?} at byte {}",
                    self.offset
                )
            }
            IcuErrorKind::MissingOther => {
                write!(f, "plural without an 'other' case at byte {}", self.offset)
            }
        }
    }
}

impl std::error::Error for IcuError {}

pub fn parse(source: &str) -> Result<Message, IcuError> {
    let mut parser = Parser {
        source,
        position: 0,
    };
    let message = parser.message(Nesting::TopLevel)?;
    Ok(message)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nesting {
    TopLevel,
    PluralCase,
}

struct Parser<'a> {
    source: &'a str,
    position: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let next = self.peek()?;
        self.position += next.len_utf8();
        Some(next)
    }

    fn error(&self, kind: IcuErrorKind) -> IcuError {
        IcuError {
            offset: self.position,
            kind,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn expect(&mut self, wanted: char, expected: &'static str) -> Result<(), IcuError> {
        match self.peek() {
            Some(found) if found == wanted => {
                self.bump();
                Ok(())
            }
            Some(found) => Err(self.error(IcuErrorKind::UnexpectedChar { found, expected })),
            None => Err(self.error(IcuErrorKind::UnexpectedEnd)),
        }
    }

    fn message(&mut self, nesting: Nesting) -> Result<Message, IcuError> {
        let mut parts = Vec::new();
        let mut text = String::new();
        loop {
            match self.peek() {
                None => {
                    if nesting == Nesting::PluralCase {
                        return Err(self.error(IcuErrorKind::UnexpectedEnd));
                    }
                    break;
                }
                Some('}') => {
                    if nesting == Nesting::TopLevel {
                        return Err(self.error(IcuErrorKind::StrayClosingBrace));
                    }
                    break;
                }
                Some('{') => {
                    if !text.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut text)));
                    }
                    parts.push(self.argument()?);
                }
                Some(other) => {
                    text.push(other);
                    self.bump();
                }
            }
        }
        if !text.is_empty() {
            parts.push(Part::Text(text));
        }
        Ok(Message { parts })
    }

    fn identifier(&mut self) -> Option<String> {
        let start = self.position;
        let first = self.peek()?;
        if !(first.is_ascii_alphabetic() || first == '_') {
            return None;
        }
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.bump();
        }
        Some(self.source[start..self.position].to_owned())
    }

    fn argument(&mut self) -> Result<Part, IcuError> {
        self.expect('{', "'{'")?;
        self.skip_whitespace();
        let name = self
            .identifier()
            .ok_or_else(|| self.error(IcuErrorKind::InvalidArgumentName))?;
        self.skip_whitespace();
        match self.peek() {
            Some('}') => {
                self.bump();
                Ok(Part::Argument(name))
            }
            Some(',') => {
                self.bump();
                self.skip_whitespace();
                let type_start = self.position;
                let kind = self
                    .identifier()
                    .ok_or_else(|| self.error(IcuErrorKind::InvalidArgumentName))?;
                if kind != "plural" {
                    return Err(IcuError {
                        offset: type_start,
                        kind: IcuErrorKind::UnsupportedArgumentType(kind),
                    });
                }
                self.skip_whitespace();
                self.expect(',', "',' after plural")?;
                let opening = self.position;
                let cases = self.plural_cases()?;
                if !cases.iter().any(|case| case.selector.is_other()) {
                    return Err(IcuError {
                        offset: opening,
                        kind: IcuErrorKind::MissingOther,
                    });
                }
                Ok(Part::Plural(Plural {
                    argument: name,
                    cases,
                }))
            }
            Some(found) => Err(self.error(IcuErrorKind::UnexpectedChar {
                found,
                expected: "'}' or ','",
            })),
            None => Err(self.error(IcuErrorKind::UnexpectedEnd)),
        }
    }

    fn selector(&mut self) -> Result<Selector, IcuError> {
        let start = self.position;
        if self.peek() == Some('=') {
            self.bump();
            let digits_start = self.position;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.bump();
            }
            let digits = &self.source[digits_start..self.position];
            return digits
                .parse::<u32>()
                .map(Selector::Exact)
                .map_err(|_| IcuError {
                    offset: start,
                    kind: IcuErrorKind::InvalidSelector(
                        self.source[start..self.position].to_owned(),
                    ),
                });
        }
        let word = self.identifier().unwrap_or_default();
        PluralCategory::from_keyword(&word)
            .map(Selector::Category)
            .ok_or(IcuError {
                offset: start,
                kind: IcuErrorKind::InvalidSelector(word),
            })
    }

    fn plural_cases(&mut self) -> Result<Vec<PluralCase>, IcuError> {
        let mut cases: Vec<PluralCase> = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err(self.error(IcuErrorKind::UnexpectedEnd)),
                Some('}') => {
                    self.bump();
                    return Ok(cases);
                }
                Some(_) => {
                    let start = self.position;
                    let selector = self.selector()?;
                    let clashes = cases.iter().any(|case| {
                        case.selector == selector
                            || (case.selector.explicit_number().is_some()
                                && case.selector.explicit_number() == selector.explicit_number())
                    });
                    if clashes {
                        return Err(IcuError {
                            offset: start,
                            kind: IcuErrorKind::DuplicateSelector(selector.to_string()),
                        });
                    }
                    self.skip_whitespace();
                    self.expect('{', "'{' opening a plural case")?;
                    let message = self.message(Nesting::PluralCase)?;
                    self.expect('}', "'}' closing a plural case")?;
                    cases.push(PluralCase { selector, message });
                }
            }
        }
    }
}
