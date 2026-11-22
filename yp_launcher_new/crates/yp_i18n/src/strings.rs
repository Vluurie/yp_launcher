use std::fmt;

use crate::locale::Locale;
use crate::plural::ArgValue;

include!(concat!(env!("OUT_DIR"), "/strings.rs"));

impl fmt::Debug for Strings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Strings")
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl MsgKey {
    pub fn text(self, strings: &Strings) -> &'static str {
        strings.get(self)
    }
}

fn text_arg<'a>(args: &[(&str, ArgValue<'a>)], name: &str) -> Option<&'a str> {
    args.iter().find_map(|(arg, value)| match value {
        ArgValue::Text(text) if *arg == name => Some(*text),
        _ => None,
    })
}

fn int_arg(args: &[(&str, ArgValue<'_>)], name: &str) -> Option<i64> {
    args.iter().find_map(|(arg, value)| match value {
        ArgValue::Int(number) if *arg == name => Some(*number),
        _ => None,
    })
}
