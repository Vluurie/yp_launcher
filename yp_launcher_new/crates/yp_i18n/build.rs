use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde_json::{Map, Value};

#[allow(dead_code)]
#[path = "src/icu.rs"]
mod icu;

use icu::{Message, Part, Plural, PluralCategory};

struct LocaleSpec {
    code: &'static str,
    variant: &'static str,
    table: &'static str,
}

const LOCALES: [LocaleSpec; 4] = [
    LocaleSpec {
        code: "en",
        variant: "En",
        table: "EN",
    },
    LocaleSpec {
        code: "de",
        variant: "De",
        table: "DE",
    },
    LocaleSpec {
        code: "zh",
        variant: "Zh",
        table: "ZH",
    },
    LocaleSpec {
        code: "th",
        variant: "Th",
        table: "TH",
    },
];

const RESERVED_METHODS: [&str; 3] = ["for_locale", "locale", "get"];

const RUST_KEYWORDS: [&str; 51] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArgKind {
    Text,
    Int,
}

struct Placeholder {
    name: String,
    rust_name: String,
    kind: ArgKind,
}

struct Entry {
    key: String,
    method: String,
    variant: String,
    placeholders: Vec<Placeholder>,
    messages: Vec<Message>,
}

impl Entry {
    fn is_static(&self) -> bool {
        self.placeholders.is_empty()
    }

    fn placeholder(&self, name: &str) -> Option<&Placeholder> {
        self.placeholders
            .iter()
            .find(|placeholder| placeholder.name == name)
    }
}

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let arb_dir = manifest_dir
        .join("..")
        .join("..")
        .join("assets")
        .join("l10n");
    let extra_dir = arb_dir.join("gpui");
    println!("cargo:rerun-if-changed=src/icu.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", extra_dir.display());

    let mut documents = Vec::new();
    for locale in &LOCALES {
        let path = arb_dir.join(format!("app_{}.arb", locale.code));
        println!("cargo:rerun-if-changed={}", path.display());
        documents.push(read_arb(&path));
    }

    let mut problems = Vec::new();
    merge_extra_sets(&extra_dir, &mut documents, &mut problems);
    let entries = collect_entries(&documents, &mut problems);
    if !problems.is_empty() {
        panic!(
            "invalid ARB files in {}:
{}",
            arb_dir.display(),
            problems.join(
                "
"
            )
        );
    }

    let code = generate(&entries);
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(out_dir.join("strings.rs"), code).expect("write generated strings.rs");
}

fn read_arb(path: &Path) -> Map<String, Value> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => map,
        Ok(_) => panic!("{} is not a JSON object", path.display()),
        Err(err) => panic!("parse {}: {err}", path.display()),
    }
}

struct ArbPairs(Vec<(String, Value)>);

impl<'de> Deserialize<'de> for ArbPairs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PairsVisitor;

        impl<'de> Visitor<'de> for PairsVisitor {
            type Value = ArbPairs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an ARB object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<ArbPairs, A::Error> {
                let mut pairs = Vec::new();
                while let Some(pair) = access.next_entry::<String, Value>()? {
                    pairs.push(pair);
                }
                Ok(ArbPairs(pairs))
            }
        }

        deserializer.deserialize_map(PairsVisitor)
    }
}

fn read_arb_pairs(path: &Path) -> Vec<(String, Value)> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    match serde_json::from_str::<ArbPairs>(&text) {
        Ok(pairs) => pairs.0,
        Err(err) => panic!("parse {}: {err}", path.display()),
    }
}

fn extra_set_names(extra_dir: &Path, problems: &mut Vec<String>) -> BTreeSet<String> {
    let mut sets = BTreeSet::new();
    let Ok(listing) = std::fs::read_dir(extra_dir) else {
        return sets;
    };
    for entry in listing.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("arb") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let set = LOCALES.iter().find_map(|locale| {
            stem.strip_suffix(locale.code)
                .and_then(|rest| rest.strip_suffix('_'))
                .filter(|set| !set.is_empty())
        });
        match set {
            Some(set) => {
                sets.insert(set.to_owned());
            }
            None => problems.push(format!(
                "gpui/{stem}.arb: file name must be <crate>_<en|de|zh|th>.arb"
            )),
        }
    }
    sets
}

fn merge_extra_sets(
    extra_dir: &Path,
    documents: &mut [Map<String, Value>],
    problems: &mut Vec<String>,
) {
    for set in extra_set_names(extra_dir, problems) {
        let template_path = extra_dir.join(format!("{set}_{}.arb", LOCALES[0].code));
        if !template_path.is_file() {
            problems.push(format!("gpui/{set}: {set}_en.arb is missing"));
            continue;
        }
        for (locale, document) in LOCALES.iter().zip(documents.iter_mut()) {
            let path = extra_dir.join(format!("{set}_{}.arb", locale.code));
            let source = if path.is_file() {
                path
            } else {
                println!(
                    "cargo:warning={}: gpui/{set}_{}.arb is missing, using en",
                    locale.code, locale.code
                );
                template_path.clone()
            };
            println!("cargo:rerun-if-changed={}", source.display());
            for (key, value) in read_arb_pairs(&source) {
                if document.contains_key(&key) {
                    problems.push(format!("{}: gpui/{set}: duplicate key {key}", locale.code));
                    continue;
                }
                document.insert(key, value);
            }
        }
    }
}

fn collect_entries(documents: &[Map<String, Value>], problems: &mut Vec<String>) -> Vec<Entry> {
    let template = &documents[0];
    let mut entries = Vec::new();
    let mut methods = BTreeMap::new();
    let mut variants = BTreeMap::new();

    for (key, value) in template {
        if key.starts_with('@') {
            continue;
        }
        let Value::String(_) = value else {
            problems.push(format!("en: {key} is not a string"));
            continue;
        };
        let placeholders = read_placeholders(template, key, problems);
        let method = snake_case(key);
        let variant = pascal_case(key);
        if RESERVED_METHODS.contains(&method.as_str()) || RUST_KEYWORDS.contains(&method.as_str()) {
            problems.push(format!("{key}: method name {method} is reserved"));
        }
        if let Some(previous) = methods.insert(method.clone(), key.clone()) {
            problems.push(format!("{key} and {previous} both map to method {method}"));
        }
        if let Some(previous) = variants.insert(variant.clone(), key.clone()) {
            problems.push(format!(
                "{key} and {previous} both map to variant {variant}"
            ));
        }

        let mut entry = Entry {
            key: key.clone(),
            method,
            variant,
            placeholders,
            messages: Vec::new(),
        };
        for (locale, document) in LOCALES.iter().zip(documents) {
            let source = match document.get(key) {
                Some(Value::String(source)) => source.clone(),
                Some(_) => {
                    problems.push(format!("{}: {key} is not a string", locale.code));
                    continue;
                }
                None => {
                    println!(
                        "cargo:warning={}: missing translation for {key}, using en",
                        locale.code
                    );
                    match template.get(key) {
                        Some(Value::String(source)) => source.clone(),
                        _ => String::new(),
                    }
                }
            };
            match icu::parse(&source) {
                Ok(message) => {
                    validate_message(&entry, locale, &message, problems);
                    entry.messages.push(message);
                }
                Err(err) => problems.push(format!("{}: {key}: {err}", locale.code)),
            }
        }
        entries.push(entry);
    }

    for (locale, document) in LOCALES.iter().zip(documents).skip(1) {
        for key in document.keys() {
            if !key.starts_with('@') && !template.contains_key(key) {
                problems.push(format!("{}: {key} does not exist in en", locale.code));
            }
        }
    }
    entries
}

fn read_placeholders(
    template: &Map<String, Value>,
    key: &str,
    problems: &mut Vec<String>,
) -> Vec<Placeholder> {
    let Some(Value::Object(meta)) = template.get(&format!("@{key}")) else {
        return Vec::new();
    };
    let Some(Value::Object(declared)) = meta.get("placeholders") else {
        return Vec::new();
    };
    let mut placeholders = Vec::new();
    for (name, spec) in declared {
        let kind = match spec.get("type").and_then(Value::as_str) {
            Some("String") | None => ArgKind::Text,
            Some("int") => ArgKind::Int,
            Some(other) => {
                problems.push(format!(
                    "{key}: placeholder {name} has unsupported type {other}"
                ));
                continue;
            }
        };
        let rust_name = snake_case(name);
        if RUST_KEYWORDS.contains(&rust_name.as_str()) {
            problems.push(format!("{key}: placeholder {name} is a Rust keyword"));
        }
        placeholders.push(Placeholder {
            name: name.clone(),
            rust_name,
            kind,
        });
    }
    placeholders
}

fn validate_message(
    entry: &Entry,
    locale: &LocaleSpec,
    message: &Message,
    problems: &mut Vec<String>,
) {
    for name in message.arguments() {
        if entry.placeholder(&name).is_none() {
            problems.push(format!(
                "{}: {}: unknown placeholder {{{name}}}",
                locale.code, entry.key
            ));
        }
    }
    for plural in message.plurals() {
        if let Some(placeholder) = entry.placeholder(&plural.argument)
            && placeholder.kind != ArgKind::Int
        {
            problems.push(format!(
                "{}: {}: plural argument {} must be an int placeholder",
                locale.code, entry.key, plural.argument
            ));
        }
    }
}

fn snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 8);
    for (index, &current) in chars.iter().enumerate() {
        if current.is_ascii_uppercase() {
            let previous = index.checked_sub(1).map(|i| chars[i]);
            let next = chars.get(index + 1).copied();
            let boundary = match previous {
                None => false,
                Some(previous) => {
                    previous.is_ascii_lowercase()
                        || previous.is_ascii_digit()
                        || (previous.is_ascii_uppercase()
                            && next.is_some_and(|next| next.is_ascii_lowercase()))
                }
            };
            if boundary {
                out.push('_');
            }
            out.push(current.to_ascii_lowercase());
        } else {
            out.push(current);
        }
    }
    out
}

fn pascal_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn literal(text: &str) -> String {
    format!("{text:?}")
}

fn generate(entries: &[Entry]) -> String {
    let static_entries: Vec<&Entry> = entries.iter().filter(|entry| entry.is_static()).collect();
    let count = static_entries.len();
    let mut code = String::new();

    code.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\npub enum MsgKey {\n",
    );
    for entry in &static_entries {
        let _ = writeln!(code, "    {},", entry.variant);
    }
    code.push_str("}\n\n");

    let _ = writeln!(code, "static MSG_KEY_NAMES: [&str; {count}] = [");
    for entry in &static_entries {
        let _ = writeln!(code, "    {},", literal(&entry.key));
    }
    code.push_str("];\n\n");

    code.push_str("impl MsgKey {\n");
    let _ = writeln!(code, "    pub const COUNT: usize = {count};");
    code.push_str("    pub const ALL: &'static [MsgKey] = &[\n");
    for entry in &static_entries {
        let _ = writeln!(code, "        MsgKey::{},", entry.variant);
    }
    code.push_str("    ];\n\n");
    code.push_str(
        "    pub fn name(self) -> &'static str {\n        MSG_KEY_NAMES[self as usize]\n    }\n\n",
    );
    code.push_str("    pub fn from_name(name: &str) -> Option<MsgKey> {\n        match name {\n");
    for entry in &static_entries {
        let _ = writeln!(
            code,
            "            {} => Some(MsgKey::{}),",
            literal(&entry.key),
            entry.variant
        );
    }
    code.push_str("            _ => None,\n        }\n    }\n}\n\n");

    generate_msg(&mut code, entries);
    generate_strings(&mut code, entries, &static_entries);

    for (index, locale) in LOCALES.iter().enumerate() {
        let _ = writeln!(
            code,
            "static {}: Strings = Strings {{\n    locale: Locale::{},\n    texts: &{}_TEXTS,\n}};\n",
            locale.table, locale.variant, locale.table
        );
        let _ = writeln!(code, "static {}_TEXTS: [&str; {count}] = [", locale.table);
        for entry in &static_entries {
            let text = entry.messages[index].plain_text().unwrap_or_default();
            let _ = writeln!(code, "    {},", literal(&text));
        }
        code.push_str("];\n\n");
    }

    for (index, locale) in LOCALES.iter().enumerate() {
        let _ = writeln!(code, "mod {} {{", locale.code);
        for entry in entries.iter().filter(|entry| !entry.is_static()) {
            generate_locale_fn(&mut code, entry, &entry.messages[index], locale);
        }
        code.push_str("}\n\n");
    }

    code
}

fn generate_msg(code: &mut String, entries: &[Entry]) {
    code.push_str("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\npub enum Msg {\n");
    for entry in entries {
        if entry.is_static() {
            let _ = writeln!(code, "    {},", entry.variant);
        } else {
            let fields: Vec<String> = entry
                .placeholders
                .iter()
                .map(|placeholder| {
                    let ty = match placeholder.kind {
                        ArgKind::Text => "String",
                        ArgKind::Int => "i64",
                    };
                    format!("{}: {ty}", placeholder.rust_name)
                })
                .collect();
            let _ = writeln!(code, "    {} {{ {} }},", entry.variant, fields.join(", "));
        }
    }
    code.push_str("}\n\n");

    code.push_str("impl Msg {\n");
    code.push_str("    pub fn name(&self) -> &'static str {\n        match self {\n");
    for entry in entries {
        let pattern = if entry.is_static() {
            format!("Msg::{}", entry.variant)
        } else {
            format!("Msg::{} {{ .. }}", entry.variant)
        };
        let _ = writeln!(code, "            {pattern} => {},", literal(&entry.key));
    }
    code.push_str("        }\n    }\n\n");

    code.push_str(
        "    pub fn resolve(&self, strings: &Strings) -> std::borrow::Cow<'static, str> {\n        match self {\n",
    );
    for entry in entries {
        if entry.is_static() {
            let _ = writeln!(
                code,
                "            Msg::{} => std::borrow::Cow::Borrowed(strings.{}()),",
                entry.variant, entry.method
            );
        } else {
            let names: Vec<&str> = entry
                .placeholders
                .iter()
                .map(|p| p.rust_name.as_str())
                .collect();
            let args: Vec<String> = entry
                .placeholders
                .iter()
                .map(|placeholder| match placeholder.kind {
                    ArgKind::Text => placeholder.rust_name.clone(),
                    ArgKind::Int => format!("*{}", placeholder.rust_name),
                })
                .collect();
            let _ = writeln!(
                code,
                "            Msg::{} {{ {} }} => std::borrow::Cow::Owned(strings.{}({})),",
                entry.variant,
                names.join(", "),
                entry.method,
                args.join(", ")
            );
        }
    }
    code.push_str("        }\n    }\n\n");

    code.push_str(
        "    pub fn from_args(name: &str, args: &[(&str, crate::plural::ArgValue<'_>)]) -> Option<Msg> {\n        match name {\n",
    );
    for entry in entries {
        if entry.is_static() {
            let _ = writeln!(
                code,
                "            {} => Some(Msg::{}),",
                literal(&entry.key),
                entry.variant
            );
        } else {
            let fields: Vec<String> = entry
                .placeholders
                .iter()
                .map(|placeholder| match placeholder.kind {
                    ArgKind::Text => format!(
                        "{}: text_arg(args, {})?.to_owned()",
                        placeholder.rust_name,
                        literal(&placeholder.name)
                    ),
                    ArgKind::Int => format!(
                        "{}: int_arg(args, {})?",
                        placeholder.rust_name,
                        literal(&placeholder.name)
                    ),
                })
                .collect();
            let _ = writeln!(
                code,
                "            {} => Some(Msg::{} {{ {} }}),",
                literal(&entry.key),
                entry.variant,
                fields.join(", ")
            );
        }
    }
    code.push_str("            _ => None,\n        }\n    }\n}\n\n");

    code.push_str(
        "impl From<MsgKey> for Msg {\n    fn from(key: MsgKey) -> Msg {\n        match key {\n",
    );
    for entry in entries.iter().filter(|entry| entry.is_static()) {
        let _ = writeln!(code, "            MsgKey::{0} => Msg::{0},", entry.variant);
    }
    code.push_str("        }\n    }\n}\n\n");
}

fn signature(entry: &Entry, used: Option<&BTreeSet<String>>) -> String {
    entry
        .placeholders
        .iter()
        .map(|placeholder| {
            let unused = used.is_some_and(|used| !used.contains(&placeholder.name));
            let prefix = if unused { "_" } else { "" };
            let ty = match placeholder.kind {
                ArgKind::Text => "&str",
                ArgKind::Int => "i64",
            };
            format!("{prefix}{}: {ty}", placeholder.rust_name)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_strings(code: &mut String, entries: &[Entry], static_entries: &[&Entry]) {
    let count = static_entries.len();
    let _ = writeln!(
        code,
        "pub struct Strings {{\n    locale: Locale,\n    texts: &'static [&'static str; {count}],\n}}\n"
    );
    code.push_str("impl Strings {\n");
    code.push_str(
        "    pub fn for_locale(locale: Locale) -> &'static Strings {\n        match locale {\n",
    );
    for locale in &LOCALES {
        let _ = writeln!(
            code,
            "            Locale::{} => &{},",
            locale.variant, locale.table
        );
    }
    code.push_str("        }\n    }\n\n");
    code.push_str("    pub fn locale(&self) -> Locale {\n        self.locale\n    }\n\n");
    code.push_str("    pub fn get(&self, key: MsgKey) -> &'static str {\n        self.texts[key as usize]\n    }\n\n");

    for entry in entries {
        if entry.is_static() {
            let _ = writeln!(
                code,
                "    pub fn {}(&self) -> &'static str {{\n        self.texts[MsgKey::{} as usize]\n    }}\n",
                entry.method, entry.variant
            );
        } else {
            let args: Vec<&str> = entry
                .placeholders
                .iter()
                .map(|p| p.rust_name.as_str())
                .collect();
            let _ = writeln!(
                code,
                "    pub fn {}(&self, {}) -> String {{\n        match self.locale {{",
                entry.method,
                signature(entry, None)
            );
            for locale in &LOCALES {
                let _ = writeln!(
                    code,
                    "            Locale::{} => {}::{}({}),",
                    locale.variant,
                    locale.code,
                    entry.method,
                    args.join(", ")
                );
            }
            code.push_str("        }\n    }\n\n");
        }
    }
    code.push_str("}\n\n");
}

fn generate_locale_fn(code: &mut String, entry: &Entry, message: &Message, locale: &LocaleSpec) {
    let used = message.arguments();
    let mut counter = 0usize;
    let body = message_expr(entry, message, locale, &mut counter, 2);
    let _ = writeln!(
        code,
        "    pub(super) fn {}({}) -> String {{\n        {body}\n    }}\n",
        entry.method,
        signature(entry, Some(&used))
    );
}

fn indent(depth: usize) -> String {
    "    ".repeat(depth)
}

fn message_expr(
    entry: &Entry,
    message: &Message,
    locale: &LocaleSpec,
    counter: &mut usize,
    depth: usize,
) -> String {
    if let [Part::Plural(plural)] = message.parts.as_slice() {
        return plural_expr(entry, plural, locale, counter, depth);
    }
    let mut lets = Vec::new();
    let mut template = String::new();
    let mut has_arguments = false;
    for part in &message.parts {
        match part {
            Part::Text(text) => {
                for c in text.chars() {
                    match c {
                        '{' => template.push_str("{{"),
                        '}' => template.push_str("}}"),
                        other => template.push(other),
                    }
                }
            }
            Part::Argument(name) => {
                has_arguments = true;
                let rust_name = entry
                    .placeholder(name)
                    .map(|p| p.rust_name.as_str())
                    .unwrap_or(name);
                let _ = write!(template, "{{{rust_name}}}");
            }
            Part::Plural(plural) => {
                has_arguments = true;
                let binding = format!("plural_{counter}");
                *counter += 1;
                let value = plural_expr(entry, plural, locale, counter, depth + 1);
                lets.push(format!("let {binding} = {value};"));
                let _ = write!(template, "{{{binding}}}");
            }
        }
    }
    let formatted = if has_arguments {
        format!("format!({})", literal(&template))
    } else {
        let plain = message.plain_text().unwrap_or_default();
        format!("String::from({})", literal(&plain))
    };
    if lets.is_empty() {
        return formatted;
    }
    let inner = indent(depth + 1);
    let outer = indent(depth);
    let mut block = String::from("{\n");
    for binding in lets {
        let _ = writeln!(block, "{inner}{binding}");
    }
    let _ = write!(block, "{inner}{formatted}\n{outer}}}");
    block
}

fn plural_expr(
    entry: &Entry,
    plural: &Plural,
    locale: &LocaleSpec,
    counter: &mut usize,
    depth: usize,
) -> String {
    let argument = entry
        .placeholder(&plural.argument)
        .map(|p| p.rust_name.clone())
        .unwrap_or_else(|| plural.argument.clone());
    let mut explicit: Vec<(u32, &Message)> = plural
        .cases
        .iter()
        .filter_map(|case| case.selector.explicit_number().map(|n| (n, &case.message)))
        .collect();
    explicit.sort_by_key(|(n, _)| *n);
    let categories: Vec<(PluralCategory, &Message)> = plural
        .cases
        .iter()
        .filter_map(|case| match case.selector {
            icu::Selector::Category(category)
                if case.selector.explicit_number().is_none()
                    && category != PluralCategory::Other =>
            {
                Some((category, &case.message))
            }
            _ => None,
        })
        .collect();
    let other = plural.other();

    let arm_indent = indent(depth + 1);
    let close_indent = indent(depth);
    let fallback = if categories.is_empty() {
        message_expr(entry, other, locale, counter, depth + 1)
    } else {
        let inner_arm = indent(depth + 2);
        let mut nested = format!(
            "match crate::plural::category(crate::locale::Locale::{}, {argument}) {{\n",
            locale.variant
        );
        for (category, message) in &categories {
            let value = message_expr(entry, message, locale, counter, depth + 2);
            let _ = writeln!(
                nested,
                "{inner_arm}crate::icu::PluralCategory::{} => {value},",
                category.variant_name()
            );
        }
        let value = message_expr(entry, other, locale, counter, depth + 2);
        let _ = write!(nested, "{inner_arm}_ => {value},\n{arm_indent}}}");
        nested
    };
    if explicit.is_empty() {
        return fallback;
    }
    let mut out = format!("match {argument} {{\n");
    for (number, message) in explicit {
        let value = message_expr(entry, message, locale, counter, depth + 1);
        let _ = writeln!(out, "{arm_indent}{number} => {value},");
    }
    let _ = write!(out, "{arm_indent}_ => {fallback},\n{close_indent}}}");
    out
}
