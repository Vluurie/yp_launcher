use yp_core::text::strip_ansi;

#[test]
fn strips_parameterised_and_private_mode_sequences() {
    assert_eq!(strip_ansi("\u{1b}[38;5;208morange\u{1b}[39m"), "orange");
    assert_eq!(strip_ansi("\u{1b}[?25lhidden cursor\u{1b}[?25h"), "hidden cursor");
    assert_eq!(strip_ansi("\u{9b}1mcsi8\u{9b}0m"), "csi8");
}

#[test]
fn drops_two_character_escapes_and_unterminated_tails() {
    assert_eq!(strip_ansi("a\u{1b}7b\u{1b}8c"), "abc");
    assert_eq!(strip_ansi("text\u{1b}"), "text");
    assert_eq!(strip_ansi("text\u{1b}[31"), "text");
    assert_eq!(strip_ansi("\u{1b}]0;never closed"), "");
}

#[test]
fn keeps_line_structure_and_unicode() {
    assert_eq!(
        strip_ansi("\u{1b}[2m2026-09-10T21:30:45Z\u{1b}[0m \u{1b}[32m INFO\u{1b}[0m nams: ä\r\nnext"),
        "2026-09-10T21:30:45Z  INFO nams: ä\r\nnext"
    );
}
