#[derive(Clone, Copy, PartialEq, Eq)]
enum AnsiState {
    Text,
    Escape,
    Csi,
    Osc,
    OscEscape,
}

pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut state = AnsiState::Text;
    for c in input.chars() {
        state = match state {
            AnsiState::Text => match c {
                '\u{1b}' => AnsiState::Escape,
                '\u{9b}' => AnsiState::Csi,
                _ => {
                    out.push(c);
                    AnsiState::Text
                }
            },
            AnsiState::Escape => match c {
                '[' => AnsiState::Csi,
                ']' => AnsiState::Osc,
                _ => AnsiState::Text,
            },
            AnsiState::Csi => {
                if ('\u{40}'..='\u{7e}').contains(&c) {
                    AnsiState::Text
                } else {
                    AnsiState::Csi
                }
            }
            AnsiState::Osc => match c {
                '\u{7}' => AnsiState::Text,
                '\u{1b}' => AnsiState::OscEscape,
                _ => AnsiState::Osc,
            },
            AnsiState::OscEscape => match c {
                '\\' => AnsiState::Text,
                _ => AnsiState::Osc,
            },
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_sgr_and_cursor_sequences() {
        assert_eq!(strip_ansi("\u{1b}[31mred\u{1b}[0m plain"), "red plain");
        assert_eq!(strip_ansi("\u{1b}[2K\u{1b}[1Gline"), "line");
        assert_eq!(strip_ansi("\u{1b}]0;title\u{7}after"), "after");
        assert_eq!(strip_ansi("\u{1b}]8;;http://x\u{1b}\\link"), "link");
        assert_eq!(strip_ansi("no escapes äö"), "no escapes äö");
    }
}
