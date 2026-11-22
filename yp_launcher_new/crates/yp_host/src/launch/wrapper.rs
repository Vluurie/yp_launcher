use yp_core::{Os, Result, SettingKey, SettingsStore};

use super::command::LaunchCommand;

pub const LAUNCH_WRAPPER: SettingKey<String> = SettingKey::new("launch_wrapper", String::new);

pub fn read(settings: &SettingsStore) -> String {
    settings.get(&LAUNCH_WRAPPER).trim().to_owned()
}

pub fn save(settings: &SettingsStore, wrapper: &str) -> Result<()> {
    settings.set(&LAUNCH_WRAPPER, &wrapper.trim().to_owned())
}

#[derive(Default)]
struct Tokenizer {
    tokens: Vec<String>,
    buffer: String,
    in_single: bool,
    in_double: bool,
    has_token: bool,
}

impl Tokenizer {
    fn feed(&mut self, c: char) {
        match c {
            '\'' if !self.in_double => {
                self.in_single = !self.in_single;
                self.has_token = true;
            }
            '"' if !self.in_single => {
                self.in_double = !self.in_double;
                self.has_token = true;
            }
            ' ' if !self.in_single && !self.in_double => self.flush(),
            other => {
                self.buffer.push(other);
                self.has_token = true;
            }
        }
    }

    fn flush(&mut self) {
        if self.has_token {
            self.tokens.push(std::mem::take(&mut self.buffer));
            self.has_token = false;
        }
    }
}

pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokenizer = Tokenizer::default();
    input.chars().for_each(|c| tokenizer.feed(c));
    tokenizer.flush();
    tokenizer.tokens
}

pub fn apply_wrapper(base: LaunchCommand, wrapper: &str) -> LaunchCommand {
    let trimmed = wrapper.trim();
    if trimmed.is_empty() {
        return base;
    }
    let mut tokens = tokenize(trimmed).into_iter();
    let Some(command) = tokens.next() else {
        return base;
    };
    let mut args: Vec<String> = tokens.collect();
    args.push(base.command);
    args.extend(base.args);
    LaunchCommand {
        command,
        args,
        cwd: base.cwd,
        env: base.env,
        label: base.label,
    }
}

pub fn wrap(os: Os, base: LaunchCommand, wrapper: &str) -> LaunchCommand {
    if os != Os::Linux {
        return base;
    }
    apply_wrapper(base, wrapper)
}
