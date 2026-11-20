use yp_builders::cheat_table::{self, output_path_for};
use yp_builders::{CheatTableError, CheatTableOutcome, TextEncoding};

const UTF8_TABLE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<CheatTable CheatEngineTableVersion="45">
  <CheatEntries>
    <CheatEntry>
      <ID>0</ID>
      <Description>"Höhe — NieRAutomata.exe offset"</Description>
      <VariableType>4 Bytes</VariableType>
      <Address>"NieRAutomata.exe"+1234AB</Address>
      <Module name="nierautomata.EXE"/>
    </CheatEntry>
    <CheatEntry>
      <AssemblerScript><![CDATA[aobscanmodule(INJECT,NieRAutomata.exe,8B 45 08)
alloc(newmem,$1000,"NieRAutomata.exe"+5A3C1)]]></AssemblerScript>
      <LuaScript>local a = 1 &lt; 2 -- NIERAUTOMATA.EXE</LuaScript>
    </CheatEntry>
  </CheatEntries>
  <!-- made for NieRAutomata.exe v1.0 -->
</CheatTable>
"#;

#[test]
fn every_module_reference_is_rewritten_and_nothing_else_changes() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("NieR.CT");
    std::fs::write(&input, UTF8_TABLE).unwrap();

    let outcome = cheat_table::convert(&input).unwrap();

    let CheatTableOutcome::Converted { replacements, output, encoding } = outcome else {
        panic!("expected a conversion");
    };
    assert_eq!(replacements, 7);
    assert_eq!(encoding, TextEncoding::Utf8);
    assert_eq!(output, dir.path().join("NieR-NAMS.CT"));
    let written = std::fs::read_to_string(&output).unwrap();
    assert_eq!(written, UTF8_TABLE.replace("NieRAutomata.exe", "game.bin").replace("nierautomata.EXE", "game.bin").replace("NIERAUTOMATA.EXE", "game.bin"));
    assert!(written.contains("<Address>\"game.bin\"+1234AB</Address>"));
    assert!(written.contains("Höhe"));
    assert!(written.contains("&lt;"));
    assert_eq!(std::fs::read_to_string(&input).unwrap(), UTF8_TABLE, "the original table is left alone");
}

#[test]
fn a_latin1_table_keeps_its_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("legacy.ct");
    let mut bytes = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>\n<CheatTable><Description>H\xF6he</Description>".to_vec();
    bytes.extend_from_slice(b"<Address>NieRAutomata.exe+10</Address></CheatTable>\n");
    std::fs::write(&input, &bytes).unwrap();

    let outcome = cheat_table::convert(&input).unwrap();

    let CheatTableOutcome::Converted { replacements, output, encoding } = outcome else {
        panic!("expected a conversion");
    };
    assert_eq!(replacements, 1);
    assert_eq!(encoding, TextEncoding::Latin1);
    assert_eq!(output, dir.path().join("legacy-NAMS.ct"));
    let mut expected = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>\n<CheatTable><Description>H\xF6he</Description>".to_vec();
    expected.extend_from_slice(b"<Address>game.bin+10</Address></CheatTable>\n");
    assert_eq!(std::fs::read(&output).unwrap(), expected);
}

#[test]
fn a_table_without_the_module_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("other.CT");
    std::fs::write(&input, "<CheatTable><Address>other.exe+1</Address></CheatTable>").unwrap();

    assert_eq!(cheat_table::convert(&input).unwrap(), CheatTableOutcome::NothingToReplace);
    assert!(!output_path_for(&input).exists());
}

#[test]
fn a_missing_table_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(cheat_table::convert(&dir.path().join("nope.CT")), Err(CheatTableError::NotFound)));
}

#[test]
fn a_table_without_extension_gets_ct() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("table");
    std::fs::write(&input, "<a>NieRAutomata.exe</a>").unwrap();

    let CheatTableOutcome::Converted { output, .. } = cheat_table::convert(&input).unwrap() else {
        panic!("expected a conversion");
    };
    assert_eq!(output, dir.path().join("table-NAMS.CT"));
    assert_eq!(std::fs::read_to_string(output).unwrap(), "<a>game.bin</a>");
}
