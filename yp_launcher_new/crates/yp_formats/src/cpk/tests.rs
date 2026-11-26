use super::{layla, read_member};
use std::fs;
use tempfile::tempdir;

enum Value<'a> {
    Number(u64),
    Text(&'a str),
}

fn string(pool: &mut Vec<u8>, text: &str) -> [u8; 4] {
    let offset = (pool.len() as u32).to_be_bytes();
    pool.extend_from_slice(text.as_bytes());
    pool.push(0);
    offset
}

fn packet(magic: &[u8; 4], fields: &[(&str, Value<'_>, bool)]) -> Vec<u8> {
    let mut table = vec![0u8; 24];
    let mut strings = vec![0];
    let mut row = Vec::new();
    for (name, value, constant) in fields {
        let (kind, bytes) = match value {
            Value::Number(n) => (6, n.to_be_bytes().to_vec()),
            Value::Text(s) => (10, string(&mut strings, s).to_vec()),
        };
        table.push((if *constant { 0x30 } else { 0x50 }) | kind);
        table.extend_from_slice(&string(&mut strings, name));
        if *constant {
            table.extend_from_slice(&bytes)
        } else {
            row.extend_from_slice(&bytes)
        }
    }
    let rows = table.len() as u16;
    let pool = u32::from(rows) + row.len() as u32;
    table[2..4].copy_from_slice(&rows.to_be_bytes());
    table[4..8].copy_from_slice(&pool.to_be_bytes());
    table[8..12].copy_from_slice(&(pool + strings.len() as u32).to_be_bytes());
    table[16..18].copy_from_slice(&(fields.len() as u16).to_be_bytes());
    table[18..20].copy_from_slice(&(row.len() as u16).to_be_bytes());
    table[20..24].copy_from_slice(&1u32.to_be_bytes());
    table.extend(row);
    table.extend(strings);
    let mut chunk = magic.to_vec();
    chunk.extend([0; 4]);
    chunk.extend((8 + table.len() as u64).to_le_bytes());
    chunk.extend(b"@UTF");
    chunk.extend((table.len() as u32).to_be_bytes());
    chunk.extend(table);
    chunk
}

fn package(stored: &[u8], expanded: usize) -> Vec<u8> {
    let mut bytes = packet(
        b"CPK ",
        &[
            ("TocOffset", Value::Number(1024), true),
            ("ContentOffset", Value::Number(2048), false),
        ],
    );
    bytes.resize(1024, 0);
    bytes.extend(packet(
        b"TOC ",
        &[
            ("DirName", Value::Text("pl"), true),
            ("FileName", Value::Text("pl020f.dat"), false),
            ("FileOffset", Value::Number(1024), false),
            ("FileSize", Value::Number(stored.len() as u64), false),
            ("ExtractSize", Value::Number(expanded as u64), false),
        ],
    ));
    bytes.resize(2048, 0);
    bytes.extend_from_slice(stored);
    bytes
}

fn compressed() -> (Vec<u8>, Vec<u8>) {
    let mut bits = Vec::new();
    for (value, width) in [(67, 9), (66, 9), (65, 9), (1, 1), (0, 13), (0, 2)] {
        bits.extend((0..width).rev().map(|i| ((value >> i) & 1) as u8));
    }
    bits.resize(bits.len().next_multiple_of(8), 0);
    let stream: Vec<_> = bits
        .chunks_exact(8)
        .rev()
        .map(|b| b.iter().fold(0, |n, v| (n << 1) | v))
        .collect();
    let mut packed = b"CRILAYLA".to_vec();
    packed.extend(6u32.to_le_bytes());
    packed.extend((stream.len() as u32).to_le_bytes());
    packed.extend(stream);
    let mut expected: Vec<_> = (0..=255).collect();
    packed.extend_from_slice(&expected);
    expected.extend(b"ABCABC");
    (packed, expected)
}

#[test]
fn reads_only_the_requested_member_in_stored_and_compressed_packages() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("motion.cpk");
    let (packed, expected) = compressed();
    for payload in [&expected, &packed] {
        fs::write(&file, package(payload, expected.len())).unwrap();
        assert_eq!(
            read_member(&file, "PL", "pl020F.dat", 1024).as_deref(),
            Some(expected.as_slice())
        );
        assert!(read_member(&file, "em", "pl020f.dat", 1024).is_none());
        assert!(read_member(&file, "pl", "pl000f.dat", 1024).is_none());
        assert!(read_member(&file, "pl", "pl020f.dat", 128).is_none());
    }
}

#[test]
fn rejects_truncated_tables_payloads_and_invalid_compression() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("bad.cpk");
    let (packed, expected) = compressed();
    let bytes = package(&packed, expected.len());
    for end in [0, 15, 32, 1029, 2047, bytes.len() - 1] {
        fs::write(&file, &bytes[..end]).unwrap();
        assert!(read_member(&file, "pl", "pl020f.dat", 1024).is_none());
    }
    let mut broken = packed.clone();
    let last_byte = 15 + u32::from_le_bytes(packed[12..16].try_into().unwrap()) as usize;
    broken[last_byte] |= 128;
    assert!(layla::decompress(&broken, expected.len()).is_none());
    assert!(layla::decompress(&packed, 255).is_none());
}
