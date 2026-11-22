use std::fmt;

use crate::icu::{Message, Part, Plural, PluralCategory};
use crate::locale::Locale;

pub fn category(locale: Locale, count: i64) -> PluralCategory {
    match locale {
        Locale::En | Locale::De => {
            if count == 1 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
        Locale::Zh | Locale::Th => PluralCategory::Other,
    }
}

pub fn choose(plural: &Plural, locale: Locale, count: i64) -> &Message {
    if let Some(message) = plural.explicit(count) {
        return message;
    }
    let category = category(locale, count);
    let explicit_category = matches!(
        category,
        PluralCategory::Zero | PluralCategory::One | PluralCategory::Two
    );
    if !explicit_category && let Some(message) = plural.category(category) {
        return message;
    }
    plural.other()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgValue<'a> {
    Text(&'a str),
    Int(i64),
}

impl fmt::Display for ArgValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgValue::Text(text) => f.write_str(text),
            ArgValue::Int(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    MissingArgument(String),
    PluralNeedsInt(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::MissingArgument(name) => write!(f, "no value for argument {name}"),
            RenderError::PluralNeedsInt(name) => write!(f, "plural argument {name} needs an int"),
        }
    }
}

impl std::error::Error for RenderError {}

pub fn render(
    message: &Message,
    locale: Locale,
    args: &[(&str, ArgValue<'_>)],
) -> Result<String, RenderError> {
    let mut out = String::new();
    render_into(&mut out, message, locale, args)?;
    Ok(out)
}

fn lookup<'a>(args: &[(&str, ArgValue<'a>)], name: &str) -> Result<ArgValue<'a>, RenderError> {
    args.iter()
        .find(|(arg, _)| *arg == name)
        .map(|(_, value)| *value)
        .ok_or_else(|| RenderError::MissingArgument(name.to_owned()))
}

fn render_into(
    out: &mut String,
    message: &Message,
    locale: Locale,
    args: &[(&str, ArgValue<'_>)],
) -> Result<(), RenderError> {
    for part in &message.parts {
        match part {
            Part::Text(text) => out.push_str(text),
            Part::Argument(name) => out.push_str(&lookup(args, name)?.to_string()),
            Part::Plural(plural) => {
                let ArgValue::Int(count) = lookup(args, &plural.argument)? else {
                    return Err(RenderError::PluralNeedsInt(plural.argument.clone()));
                };
                render_into(out, choose(plural, locale, count), locale, args)?;
            }
        }
    }
    Ok(())
}
