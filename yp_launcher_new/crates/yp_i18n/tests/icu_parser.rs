use yp_i18n::{IcuErrorKind, Part, PluralCategory, Selector, icu};

#[test]
fn plain_text_keeps_apostrophes_and_symbols() {
    let message = icu::parse("Don't touch C:\\Games #1 'quoted'").unwrap();
    assert_eq!(
        message.plain_text().as_deref(),
        Some("Don't touch C:\\Games #1 'quoted'")
    );
    assert!(message.is_plain_text());
}

#[test]
fn empty_message_is_empty_text() {
    let message = icu::parse("").unwrap();
    assert!(message.parts.is_empty());
    assert_eq!(message.plain_text().as_deref(), Some(""));
}

#[test]
fn arguments_are_split_from_text() {
    let message = icu::parse("Missing: {files} ({ count })").unwrap();
    assert_eq!(
        message.parts,
        vec![
            Part::Text("Missing: ".into()),
            Part::Argument("files".into()),
            Part::Text(" (".into()),
            Part::Argument("count".into()),
            Part::Text(")".into()),
        ]
    );
    assert_eq!(
        message.arguments().into_iter().collect::<Vec<_>>(),
        vec!["count", "files"]
    );
}

#[test]
fn plurals_parse_cases_and_nested_arguments() {
    let message =
        icu::parse("{count} {count, plural, =0{none} one{one {name}} other{{count} items}}")
            .unwrap();
    let plurals = message.plurals();
    assert_eq!(plurals.len(), 1);
    let plural = plurals[0];
    assert_eq!(plural.argument, "count");
    let selectors: Vec<Selector> = plural.cases.iter().map(|case| case.selector).collect();
    assert_eq!(
        selectors,
        vec![
            Selector::Exact(0),
            Selector::Category(PluralCategory::One),
            Selector::Category(PluralCategory::Other)
        ]
    );
    assert_eq!(
        message.arguments().into_iter().collect::<Vec<_>>(),
        vec!["count", "name"]
    );
    assert_eq!(
        plural.explicit_numbers().into_iter().collect::<Vec<_>>(),
        vec![0, 1]
    );
}

#[test]
fn nested_plurals_are_collected() {
    let message = icu::parse("{a, plural, =1{x} other{{b, plural, =1{y} other{z}}}}").unwrap();
    let arguments: Vec<&str> = message
        .plurals()
        .iter()
        .map(|plural| plural.argument.as_str())
        .collect();
    assert_eq!(arguments, vec!["a", "b"]);
}

fn error_kind(source: &str) -> IcuErrorKind {
    icu::parse(source).unwrap_err().kind
}

#[test]
fn malformed_messages_are_rejected() {
    assert_eq!(error_kind("oops }"), IcuErrorKind::StrayClosingBrace);
    assert_eq!(error_kind("{count"), IcuErrorKind::UnexpectedEnd);
    assert_eq!(error_kind("{ }"), IcuErrorKind::InvalidArgumentName);
    assert_eq!(error_kind("{1abc}"), IcuErrorKind::InvalidArgumentName);
    assert_eq!(
        error_kind("{g, select, male{he} other{they}}"),
        IcuErrorKind::UnsupportedArgumentType("select".into())
    );
    assert_eq!(
        error_kind("{n, plural, =1{one}}"),
        IcuErrorKind::MissingOther
    );
    assert_eq!(
        error_kind("{n, plural, =1{a} one{b} other{c}}"),
        IcuErrorKind::DuplicateSelector("one".into())
    );
    assert_eq!(
        error_kind("{n, plural, other{a} other{b}}"),
        IcuErrorKind::DuplicateSelector("other".into())
    );
    assert_eq!(
        error_kind("{n, plural, lots{a} other{b}}"),
        IcuErrorKind::InvalidSelector("lots".into())
    );
    assert_eq!(
        error_kind("{n, plural, =1{a} other{b}"),
        IcuErrorKind::UnexpectedEnd
    );
    assert!(matches!(
        error_kind("{n plural}"),
        IcuErrorKind::UnexpectedChar { found: 'p', .. }
    ));
}

#[test]
fn errors_carry_byte_offsets() {
    let err = icu::parse("abc }").unwrap_err();
    assert_eq!(err.offset, 4);
    assert!(err.to_string().contains("byte 4"));
}
