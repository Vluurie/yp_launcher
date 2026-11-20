use yp_core::ini::{IniDoc, IniLine, IniSyntax};

const D3DX: &str = "; 3DMigoto config\r\n[Loader]\r\ntarget = NieRAutomata.exe\r\nmodule = d3d11.dll\r\n\r\n[Hunting]\r\nhunting = 0\r\nmarking_mode = skip ; inline\r\n[Logging]\r\n# hash comment\r\nverbose_overlay=1\r\n";

#[test]
fn round_trip_is_byte_identical() {
    for text in [
        D3DX,
        "a=1\nb = 2",
        "",
        "\n\n",
        "\u{feff}[General]\nKey=Value\n",
        "[x]\r\nmixed=1\nendings=2\r\n",
        "  [ spaced ]  ; c\n  indented = yes  \n",
        "garbage line\n[broken\nk=v\n",
    ] {
        assert_eq!(IniDoc::parse(text).to_string(), text, "{text:?}");
    }
}

#[test]
fn classifies_lines() {
    let doc = IniDoc::parse("; c\n\n[S]\nk = v\nnot an entry\n[bad\n");
    let kinds: Vec<&str> = doc
        .lines()
        .iter()
        .map(|line| match line {
            IniLine::Section { .. } => "section",
            IniLine::Entry { .. } => "entry",
            IniLine::Comment(_) => "comment",
            IniLine::Blank(_) => "blank",
            IniLine::Unparsed(_) => "unparsed",
        })
        .collect();
    assert_eq!(kinds, ["comment", "blank", "section", "entry", "unparsed", "unparsed"]);
}

#[test]
fn get_is_case_insensitive_and_scoped_to_the_section() {
    let doc = IniDoc::parse(D3DX);
    assert_eq!(doc.get(Some("loader"), "TARGET"), Some("NieRAutomata.exe"));
    assert_eq!(doc.get(Some("Hunting"), "marking_mode"), Some("skip ; inline"));
    assert_eq!(doc.get_uncommented(Some("Hunting"), "marking_mode"), Some("skip"));
    assert_eq!(doc.get(Some("Logging"), "verbose_overlay"), Some("1"));
    assert_eq!(doc.get(Some("Hunting"), "target"), None);
    assert_eq!(doc.get(None, "target"), None);
    assert_eq!(doc.sections(), ["Loader", "Hunting", "Logging"]);
    assert_eq!(doc.entries().len(), 5);
    assert_eq!(doc.entries()[0], (Some("Loader"), "target", "NieRAutomata.exe"));
}

#[test]
fn set_replaces_an_existing_key_in_place() {
    let mut doc = IniDoc::parse(D3DX);
    doc.set(Some("hunting"), "Hunting", "2");
    let text = doc.to_string();
    assert!(text.contains("[Hunting]\r\nHunting = 2\r\nmarking_mode"));
    assert_eq!(text.matches("unting").count(), 2);
}

#[test]
fn set_inserts_a_missing_key_right_after_the_header() {
    let mut doc = IniDoc::parse("[Hunting]\nhunting = 0\n[Other]\nx = 1\n");
    doc.set(Some("Hunting"), "reload_fixes", "VK_F10");
    assert_eq!(doc.to_string(), "[Hunting]\nreload_fixes = VK_F10\nhunting = 0\n[Other]\nx = 1\n");
}

#[test]
fn set_appends_a_missing_section_after_trimming_blank_lines() {
    let mut doc = IniDoc::parse("[A]\nx = 1\n\n\n");
    doc.set(Some("System"), "allow_check_interfaces", "1");
    assert_eq!(doc.to_string(), "[A]\nx = 1\n\n[System]\nallow_check_interfaces = 1\n");

    let mut no_newline = IniDoc::parse("[A]\nx = 1");
    no_newline.set(Some("B"), "y", "2");
    assert_eq!(no_newline.to_string(), "[A]\nx = 1\n\n[B]\ny = 2\n");
}

#[test]
fn set_on_empty_content_creates_the_section() {
    for empty in ["", "  \n\n"] {
        let mut doc = IniDoc::parse(empty);
        doc.set(Some("GENERAL"), "PresetPath", ".\\a.ini");
        assert_eq!(doc.to_string(), "[GENERAL]\nPresetPath = .\\a.ini\n");
    }
}

#[test]
fn set_keeps_crlf_style_for_new_lines() {
    let mut doc = IniDoc::parse(D3DX);
    doc.set(Some("Device"), "check_foreground_window", "0");
    doc.set(Some("Loader"), "require_admin", "false");
    let text = doc.to_string();
    assert!(text.ends_with("\r\n\r\n[Device]\r\ncheck_foreground_window = 0\r\n"));
    assert!(text.contains("[Loader]\r\nrequire_admin = false\r\ntarget"));
    assert!(!text.replace("\r\n", "").contains('\n'));
}

#[test]
fn set_and_remove_global_entries() {
    let mut doc = IniDoc::parse("a = 1\n[S]\nk = v\n");
    doc.set(None, "b", "2");
    doc.set(None, "a", "3");
    assert_eq!(doc.to_string(), "a = 3\nb = 2\n[S]\nk = v\n");
    assert!(doc.remove(None, "a"));
    assert!(!doc.remove(None, "k"));
    assert!(doc.remove(Some("s"), "K"));
    assert_eq!(doc.to_string(), "b = 2\n[S]\n");
}

#[test]
fn remove_last_line_without_newline_keeps_shape() {
    let mut doc = IniDoc::parse("[S]\na = 1\nb = 2");
    assert!(doc.remove(Some("S"), "b"));
    assert_eq!(doc.to_string(), "[S]\na = 1");
}

#[test]
fn whitespace_separated_syntax_reads_naiom_style_lines() {
    let doc = IniDoc::parse_with(
        "; NAIOM\nStandardJump\tSPACE\nAimSensitivityX 1.5 ; fast\nlonely\n",
        IniSyntax::whitespace_separated(),
    );
    assert_eq!(doc.get(None, "standardjump"), Some("SPACE"));
    assert_eq!(doc.get_uncommented(None, "AimSensitivityX"), Some("1.5"));
    assert!(matches!(doc.lines()[3], IniLine::Unparsed(_)));
    let mut edited = doc.clone();
    edited.set(Some("Mouse"), "Key", "Value");
    assert!(edited.to_string().ends_with("[Mouse]\nKey Value\n"));
}
