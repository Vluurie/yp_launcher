use yp_host::vdf::{parse, path_entries};

const LINUX_VDF: &str = "\"libraryfolders\"
{
\t\"0\"
\t{
\t\t\"path\"\t\t\"/home/u/.local/share/Steam\"
\t\t\"apps\"
\t\t{
\t\t\t\"524220\"\t\t\"43881494773\"
\t\t}
\t}
\t\"1\"
\t{
\t\t\"path\"\t\t\"/mnt/games/SteamLibrary\"
\t}
}
";

#[test]
fn returns_every_path_entry_in_order() {
    assert_eq!(path_entries(LINUX_VDF), ["/home/u/.local/share/Steam", "/mnt/games/SteamLibrary"]);
}

#[test]
fn unescapes_windows_style_backslashes() {
    assert_eq!(
        path_entries(r#""path"		"C:\\Program Files (x86)\\Steam""#),
        [r"C:\Program Files (x86)\Steam"]
    );
}

#[test]
fn empty_content_yields_nothing() {
    assert!(path_entries("").is_empty());
}

#[test]
fn escapes_are_undone_exactly_once() {
    assert_eq!(path_entries(r#""path" "\\\\server\\share\\Lib""#), [r"\\server\share\Lib"]);
    assert_eq!(path_entries(r#""path" "C:\\a\"b""#), [r#"C:\a"b"#]);
}

#[test]
fn broken_content_yields_nothing_instead_of_partial_results() {
    assert!(path_entries("\"libraryfolders\" { \"0\" { \"path\" \"/a\" }").is_empty());
}

#[test]
fn lookups_are_case_insensitive_and_nested_search_is_depth_first() {
    let root = parse(LINUX_VDF).unwrap();
    let folders = root.block("LibraryFolders").unwrap();
    assert_eq!(folders.block("0").unwrap().string("PATH"), Some("/home/u/.local/share/Steam"));
    assert_eq!(root.find_block("apps").unwrap().string("524220"), Some("43881494773"));
    assert!(root.find_block("missing").is_none());
    assert_eq!(folders.string("0"), None);
}

#[test]
fn reads_appmanifest_with_conditionals_and_comments() {
    let manifest = "\"AppState\"\n{\n\t\"appid\"\t\t\"524220\" [$WIN32]\n\t// a comment\n\t\"buildid\"\t\"123\"\n\t\"InstalledDepots\"\n\t{\n\t\t\"580601\"\n\t\t{\n\t\t\t\"dlcappid\"\t\t\"580600\"\n\t\t}\n\t}\n}\n";
    let root = parse(manifest).unwrap();
    let state = root.block("appstate").unwrap();
    assert_eq!(state.string("appid"), Some("524220"));
    assert_eq!(state.string("BuildID"), Some("123"));
    assert_eq!(
        state.block("InstalledDepots").unwrap().block("580601").unwrap().string("dlcappid"),
        Some("580600")
    );
}
