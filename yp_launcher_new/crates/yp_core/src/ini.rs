use std::fmt;

const BOM: char = '\u{feff}';
const LF: &str = "\n";
const CRLF: &str = "\r\n";
const NO_NEWLINE: &str = "";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniSyntax {
    pub separators: Vec<char>,
    pub comment_prefixes: Vec<char>,
}

impl IniSyntax {
    pub fn standard() -> Self {
        Self {
            separators: vec!['='],
            comment_prefixes: vec![';', '#'],
        }
    }

    pub fn whitespace_separated() -> Self {
        Self {
            separators: vec![' ', '\t'],
            comment_prefixes: vec![';', '#'],
        }
    }

    pub fn is_comment(&self, trimmed: &str) -> bool {
        trimmed.chars().next().is_some_and(|first| self.comment_prefixes.contains(&first))
    }

    pub fn strip_inline_comment<'a>(&self, value: &'a str) -> &'a str {
        match value.find(|c: char| self.comment_prefixes.contains(&c)) {
            Some(index) => value[..index].trim(),
            None => value.trim(),
        }
    }

    fn split_entry<'a>(&self, trimmed: &'a str) -> Option<(&'a str, &'a str)> {
        let index = trimmed.find(|c: char| self.separators.contains(&c))?;
        let separator_len = trimmed[index..].chars().next().map(char::len_utf8).unwrap_or(1);
        let key = trimmed[..index].trim();
        if key.is_empty() {
            return None;
        }
        Some((key, trimmed[index + separator_len..].trim()))
    }

    fn render_entry(&self, key: &str, value: &str) -> String {
        match self.separators.first() {
            Some(separator) if separator.is_whitespace() => format!("{key}{separator}{value}"),
            Some(separator) => format!("{key} {separator} {value}"),
            None => format!("{key} = {value}"),
        }
    }

    fn section_name<'a>(&self, trimmed: &'a str) -> Option<&'a str> {
        let inner = trimmed.strip_prefix('[')?;
        let close = inner.find(']')?;
        let rest = inner[close + 1..].trim();
        if !rest.is_empty() && !self.is_comment(rest) {
            return None;
        }
        Some(inner[..close].trim())
    }
}

impl Default for IniSyntax {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IniLine {
    Section { name: String, raw: String },
    Entry { key: String, value: String, raw: String },
    Comment(String),
    Blank(String),
    Unparsed(String),
}

impl IniLine {
    pub fn raw(&self) -> &str {
        match self {
            IniLine::Section { raw, .. } | IniLine::Entry { raw, .. } => raw,
            IniLine::Comment(raw) | IniLine::Blank(raw) | IniLine::Unparsed(raw) => raw,
        }
    }

    fn section(&self) -> Option<&str> {
        match self {
            IniLine::Section { name, .. } => Some(name),
            _ => None,
        }
    }

    fn entry(&self) -> Option<(&str, &str)> {
        match self {
            IniLine::Entry { key, value, .. } => Some((key, value)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniDoc {
    lines: Vec<IniLine>,
    syntax: IniSyntax,
    newline: &'static str,
    trailing_newline: bool,
    endings: Vec<&'static str>,
    bom: bool,
}

impl IniDoc {
    pub fn parse(text: &str) -> Self {
        Self::parse_with(text, IniSyntax::standard())
    }

    pub fn parse_with(text: &str, syntax: IniSyntax) -> Self {
        let bom = text.starts_with(BOM);
        let body = text.strip_prefix(BOM).unwrap_or(text);
        let mut lines = Vec::new();
        let mut endings = Vec::new();
        let mut rest = body;
        let mut trailing_newline = false;
        while !rest.is_empty() {
            let (line, ending, next) = match rest.find('\n') {
                Some(index) => {
                    let line = &rest[..index];
                    match line.strip_suffix('\r') {
                        Some(stripped) => (stripped, CRLF, &rest[index + 1..]),
                        None => (line, LF, &rest[index + 1..]),
                    }
                }
                None => (rest, NO_NEWLINE, ""),
            };
            lines.push(classify(line, &syntax));
            endings.push(ending);
            trailing_newline = ending != NO_NEWLINE;
            rest = next;
        }
        let newline = endings.iter().copied().find(|ending| *ending != NO_NEWLINE).unwrap_or(LF);
        Self {
            lines,
            syntax,
            newline,
            trailing_newline,
            endings,
            bom,
        }
    }

    pub fn lines(&self) -> &[IniLine] {
        &self.lines
    }

    pub fn syntax(&self) -> &IniSyntax {
        &self.syntax
    }

    pub fn newline(&self) -> &'static str {
        self.newline
    }

    pub fn get(&self, section: Option<&str>, key: &str) -> Option<&str> {
        let mut current: Option<&str> = None;
        for line in &self.lines {
            if let Some(name) = line.section() {
                current = Some(name);
                continue;
            }
            if let Some((entry_key, value)) = line.entry()
                && section_matches(current, section)
                && entry_key.eq_ignore_ascii_case(key)
            {
                return Some(value);
            }
        }
        None
    }

    pub fn get_uncommented(&self, section: Option<&str>, key: &str) -> Option<&str> {
        self.get(section, key).map(|value| self.syntax.strip_inline_comment(value))
    }

    pub fn set(&mut self, section: Option<&str>, key: &str, value: &str) {
        let line = IniLine::Entry {
            key: key.to_owned(),
            value: value.to_owned(),
            raw: self.syntax.render_entry(key, value),
        };
        let Some(section) = section else {
            self.set_global(key, line);
            return;
        };
        if self.lines.iter().all(|line| matches!(line, IniLine::Blank(_))) {
            self.lines.clear();
            self.endings.clear();
            self.push_line(section_line(section));
            self.push_line(line);
            self.trailing_newline = true;
            return;
        }
        let Some(header) = self.section_index(section) else {
            self.append_section(section, line);
            return;
        };
        let end = self.section_end(header);
        let existing = (header + 1..end).find(|&index| {
            self.lines[index].entry().is_some_and(|(entry_key, _)| entry_key.eq_ignore_ascii_case(key))
        });
        match existing {
            Some(index) => self.lines[index] = line,
            None => self.insert_line(header + 1, line),
        }
    }

    pub fn remove(&mut self, section: Option<&str>, key: &str) -> bool {
        let mut current: Option<String> = None;
        let found = self.lines.iter().position(|line| {
            if let Some(name) = line.section() {
                current = Some(name.to_owned());
                return false;
            }
            line.entry().is_some_and(|(entry_key, _)| {
                section_matches(current.as_deref(), section) && entry_key.eq_ignore_ascii_case(key)
            })
        });
        let Some(index) = found else {
            return false;
        };
        self.lines.remove(index);
        let ending = self.endings.remove(index);
        if index == self.lines.len() && ending == NO_NEWLINE {
            self.trailing_newline = false;
            if let Some(last) = self.endings.last_mut() {
                *last = NO_NEWLINE;
            }
        }
        true
    }

    pub fn entries(&self) -> Vec<(Option<&str>, &str, &str)> {
        let mut current: Option<&str> = None;
        let mut out = Vec::new();
        for line in &self.lines {
            if let Some(name) = line.section() {
                current = Some(name);
            } else if let Some((key, value)) = line.entry() {
                out.push((current, key, value));
            }
        }
        out
    }

    pub fn sections(&self) -> Vec<&str> {
        self.lines.iter().filter_map(IniLine::section).collect()
    }

    fn section_index(&self, section: &str) -> Option<usize> {
        self.lines
            .iter()
            .position(|line| line.section().is_some_and(|name| name.eq_ignore_ascii_case(section)))
    }

    fn section_end(&self, header: usize) -> usize {
        (header + 1..self.lines.len())
            .find(|&index| self.lines[index].section().is_some())
            .unwrap_or(self.lines.len())
    }

    fn set_global(&mut self, key: &str, line: IniLine) {
        let end = (0..self.lines.len())
            .find(|&index| self.lines[index].section().is_some())
            .unwrap_or(self.lines.len());
        let existing =
            (0..end).find(|&index| self.lines[index].entry().is_some_and(|(entry_key, _)| entry_key.eq_ignore_ascii_case(key)));
        if let Some(index) = existing {
            self.lines[index] = line;
            return;
        }
        let after_last_entry = (0..end).rev().find(|&index| self.lines[index].entry().is_some()).map(|index| index + 1);
        self.insert_line(after_last_entry.unwrap_or(0), line);
    }

    fn append_section(&mut self, section: &str, line: IniLine) {
        while self.lines.last().is_some_and(|last| last.raw().is_empty()) {
            self.lines.pop();
            self.endings.pop();
        }
        if let Some(last) = self.endings.last_mut() {
            *last = self.newline;
        }
        self.push_line(IniLine::Blank(String::new()));
        self.push_line(section_line(section));
        self.push_line(line);
        self.trailing_newline = true;
    }

    fn push_line(&mut self, line: IniLine) {
        if let Some(last) = self.endings.last_mut()
            && *last == NO_NEWLINE
        {
            *last = self.newline;
        }
        self.lines.push(line);
        self.endings.push(self.newline);
    }

    fn insert_line(&mut self, index: usize, line: IniLine) {
        if index >= self.lines.len() {
            let had_trailing = self.trailing_newline;
            self.push_line(line);
            if !had_trailing && let Some(last) = self.endings.last_mut() {
                *last = NO_NEWLINE;
            }
            return;
        }
        self.lines.insert(index, line);
        self.endings.insert(index, self.newline);
    }
}

fn section_matches(current: Option<&str>, wanted: Option<&str>) -> bool {
    match (current, wanted) {
        (None, None) => true,
        (Some(current), Some(wanted)) => current.eq_ignore_ascii_case(wanted),
        _ => false,
    }
}

fn section_line(section: &str) -> IniLine {
    IniLine::Section {
        name: section.to_owned(),
        raw: format!("[{section}]"),
    }
}

fn classify(line: &str, syntax: &IniSyntax) -> IniLine {
    let raw = line.to_owned();
    let trimmed = line.trim().trim_start_matches(BOM);
    if trimmed.is_empty() {
        return IniLine::Blank(raw);
    }
    if syntax.is_comment(trimmed) {
        return IniLine::Comment(raw);
    }
    if trimmed.starts_with('[') {
        return match syntax.section_name(trimmed) {
            Some(name) => IniLine::Section {
                name: name.to_owned(),
                raw,
            },
            None => IniLine::Unparsed(raw),
        };
    }
    match syntax.split_entry(trimmed) {
        Some((key, value)) => IniLine::Entry {
            key: key.to_owned(),
            value: value.to_owned(),
            raw,
        },
        None => IniLine::Unparsed(raw),
    }
}

impl fmt::Display for IniDoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bom {
            write!(f, "{BOM}")?;
        }
        let last = self.lines.len().saturating_sub(1);
        for (index, (line, ending)) in self.lines.iter().zip(&self.endings).enumerate() {
            f.write_str(line.raw())?;
            let ending = if index == last && !self.trailing_newline {
                NO_NEWLINE
            } else if *ending == NO_NEWLINE {
                self.newline
            } else {
                ending
            };
            f.write_str(ending)?;
        }
        Ok(())
    }
}
