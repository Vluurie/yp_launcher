use super::model::{DiagnosticsReport, GAME_DIR_KEY, GAME_DIR_NOT_SELECTED, LAUNCHER_DIR_KEY};

pub const GAME_DIR_TOKEN: &str = "<gameDir>";
pub const LAUNCHER_DIR_TOKEN: &str = "<launcherDir>";
pub const USER_HOME_TOKEN: &str = "<userHome>";
pub const USER_TOKEN: &str = "<user>";
pub const MIN_PATH_CHARS: usize = 3;
pub const MIN_USER_CHARS: usize = 2;
pub const HOME_ROOT_SEGMENTS: [&str; 2] = ["users", "home"];
pub const WINE_HOST_DRIVES: [&str; 2] = ["Z:", "z:"];

fn folded(c: char) -> char {
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        _ => c,
    }
}

fn folded_chars(text: &str) -> Vec<char> {
    text.chars().map(folded).collect()
}

fn is_separator(c: char) -> bool {
    c == '/' || c == '\\'
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}

fn path_variants(raw: &str) -> Vec<String> {
    let backslashed = raw.replace('/', "\\");
    let mut variants = vec![raw.to_owned(), raw.replace('\\', "/"), backslashed.clone()];
    if raw.starts_with('/') {
        let [upper, lower] = WINE_HOST_DRIVES;
        variants.push(format!("{upper}{backslashed}"));
        variants.push(format!("{upper}{raw}"));
        variants.push(format!("{lower}{backslashed}"));
    }
    variants
}

fn follows_home_root(text: &[char], index: usize) -> bool {
    if index < 2 || !is_separator(text[index - 1]) {
        return false;
    }
    let before = &text[..index - 1];
    let start = before
        .iter()
        .rposition(|c| is_separator(*c))
        .map_or(0, |separator| separator + 1);
    let segment = &before[start..];
    HOME_ROOT_SEGMENTS
        .iter()
        .any(|root| segment.iter().copied().eq(root.chars()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathRule {
    pattern: Vec<char>,
    token: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Redactor {
    paths: Vec<PathRule>,
    user: Vec<char>,
}

impl Redactor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_report(report: &DiagnosticsReport) -> Self {
        let typed_game_dir = report
            .game_dir
            .as_ref()
            .map(|dir| dir.to_string_lossy().into_owned())
            .unwrap_or_default();
        let listed_game_dir = report
            .launcher_value(GAME_DIR_KEY)
            .filter(|dir| *dir != GAME_DIR_NOT_SELECTED)
            .unwrap_or_default();
        Self::new()
            .with_path(&typed_game_dir, GAME_DIR_TOKEN)
            .with_path(listed_game_dir, GAME_DIR_TOKEN)
            .with_path(
                report.launcher_value(LAUNCHER_DIR_KEY).unwrap_or_default(),
                LAUNCHER_DIR_TOKEN,
            )
            .with_path(&report.host.user_home, USER_HOME_TOKEN)
            .with_user_name(&report.host.user_name)
    }

    pub fn with_path(mut self, raw: &str, token: &'static str) -> Self {
        if raw.is_empty() {
            return self;
        }
        for variant in path_variants(raw) {
            let pattern = folded_chars(&variant);
            if pattern.len() >= MIN_PATH_CHARS
                && !self.paths.iter().any(|rule| rule.pattern == pattern)
            {
                self.paths.push(PathRule { pattern, token });
            }
        }
        self.paths
            .sort_by_key(|rule| std::cmp::Reverse(rule.pattern.len()));
        self
    }

    pub fn with_user_name(mut self, name: &str) -> Self {
        let user = folded_chars(name);
        self.user = if user.len() >= MIN_USER_CHARS {
            user
        } else {
            Vec::new()
        };
        self
    }

    pub fn redact(&self, input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }
        let mut chars: Vec<char> = input.chars().collect();
        for rule in &self.paths {
            chars = replace_ignoring_case(&chars, &rule.pattern, rule.token);
        }
        if !self.user.is_empty() {
            chars = replace_user_in_paths(&chars, &self.user, USER_TOKEN);
        }
        chars.into_iter().collect()
    }
}

fn replace_ignoring_case(input: &[char], pattern: &[char], token: &str) -> Vec<char> {
    let lowered: Vec<char> = input.iter().copied().map(folded).collect();
    let mut out = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if lowered[index..].starts_with(pattern) {
            out.extend(token.chars());
            index += pattern.len();
        } else {
            out.push(input[index]);
            index += 1;
        }
    }
    out
}

fn replace_user_in_paths(input: &[char], user: &[char], token: &str) -> Vec<char> {
    let lowered: Vec<char> = input.iter().copied().map(folded).collect();
    let mut out = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if !lowered[index..].starts_with(user) {
            out.push(input[index]);
            index += 1;
            continue;
        }
        let end = index + user.len();
        let ends_segment = input.get(end).is_none_or(|next| !is_word_char(*next));
        if follows_home_root(&lowered, index) && ends_segment {
            out.extend(token.chars());
        } else {
            out.extend_from_slice(&input[index..end]);
        }
        index = end;
    }
    out
}

pub fn redact(input: &str, report: &DiagnosticsReport) -> String {
    Redactor::for_report(report).redact(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_paths_gain_wine_drive_variants() {
        let variants = path_variants("/home/a/Games");
        assert!(variants.contains(&"Z:\\home\\a\\Games".to_owned()));
        assert!(variants.contains(&"Z:/home/a/Games".to_owned()));
        assert!(variants.contains(&"z:\\home\\a\\Games".to_owned()));
        assert_eq!(path_variants("C:\\Games").len(), 3);
    }

    #[test]
    fn home_root_is_the_segment_right_before_the_user() {
        let text = folded_chars("c:\\users\\me\\x and /home/me and /opt/me and users/me");
        let position = |needle: &str| {
            let needle = folded_chars(needle);
            (0..text.len())
                .find(|start| text[*start..].starts_with(&needle))
                .unwrap()
                + needle.len()
                - 2
        };
        assert!(follows_home_root(&text, position("users\\me")));
        assert!(follows_home_root(&text, position("/home/me")));
        assert!(!follows_home_root(&text, position("/opt/me")));
        assert!(!follows_home_root(&text, 0));
    }

    #[test]
    fn longest_path_wins_when_one_contains_the_other() {
        let redactor = Redactor::new()
            .with_path("C:\\Users\\me", USER_HOME_TOKEN)
            .with_path("C:\\Users\\me\\Games\\NieR", GAME_DIR_TOKEN);
        assert_eq!(
            redactor.redact("C:\\Users\\me\\Games\\NieR\\data and C:\\Users\\me\\Desktop"),
            "<gameDir>\\data and <userHome>\\Desktop"
        );
    }
}
