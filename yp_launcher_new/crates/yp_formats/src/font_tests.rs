use super::*;
use std::env;

fn put(bytes: &mut Vec<u8>, at: usize, value: &[u8]) {
    if bytes.len() < at + value.len() {
        bytes.resize(at + value.len(), 0);
    }
    bytes[at..at + value.len()].copy_from_slice(value);
}

fn message_file() -> Vec<u8> {
    let mut bytes = vec![0; 40];
    let symbols = 64;
    for (i, (font, ch)) in [(36_u16, 'A'), (36, 'V'), (2, 'T')].iter().enumerate() {
        let at = symbols + i * SYMBOL;
        put(&mut bytes, at, &font.to_le_bytes());
        put(&mut bytes, at + 2, &(*ch as u16).to_le_bytes());
        put(&mut bytes, at + 4, &(i as u32).to_le_bytes());
    }
    let glyphs = 128;
    for i in 0..3 {
        let at = glyphs + i * GLYPH;
        put(&mut bytes, at, &7_u32.to_le_bytes());
        put(&mut bytes, at + 4, &(0.25 * i as f32).to_le_bytes());
        put(&mut bytes, at + 8, &0.5_f32.to_le_bytes());
        put(&mut bytes, at + 20, &3_f32.to_le_bytes());
        put(&mut bytes, at + 24, &2_f32.to_le_bytes());
    }
    let fonts = 256;
    put(&mut bytes, fonts, &36_u32.to_le_bytes());
    put(&mut bytes, fonts + 12, &(-7_f32).to_le_bytes());
    let messages = 300;
    let paragraphs = 340;
    let lines = 400;
    let letters = 480;
    for m in 0..2_u32 {
        let at = messages + m as usize * MESSAGE;
        put(
            &mut bytes,
            at,
            &(paragraphs as u32 + m * PARAGRAPH as u32).to_le_bytes(),
        );
        put(&mut bytes, at + 4, &1_u32.to_le_bytes());
        let paragraph = paragraphs + m as usize * PARAGRAPH;
        put(
            &mut bytes,
            paragraph,
            &(lines as u32 + m * LINE as u32).to_le_bytes(),
        );
        put(&mut bytes, paragraph + 4, &1_u32.to_le_bytes());
        put(&mut bytes, paragraph + 16, &36_u32.to_le_bytes());
    }
    let first = [(0_u16, 0_i16), (1, -4), (TERMINATOR + 1, 36), (1, 2)];
    let second = [(0_u16, -9_i16)];
    for (line, run) in [&first[..], &second[..]].into_iter().enumerate() {
        let at = lines + line * LINE;
        let start = letters + line * 32;
        put(&mut bytes, at, &(start as u32).to_le_bytes());
        put(
            &mut bytes,
            at + 8,
            &(run.len() as u32 * 2 + 1).to_le_bytes(),
        );
        for (k, (code, kerning)) in run.iter().enumerate() {
            put(&mut bytes, start + k * LETTER, &code.to_le_bytes());
            put(&mut bytes, start + k * LETTER + 2, &kerning.to_le_bytes());
        }
    }
    for (i, value) in [messages, 2, symbols, 3, glyphs, 3, fonts, 1, 560, 0]
        .into_iter()
        .enumerate()
    {
        put(&mut bytes, i * 4, &(value as u32).to_le_bytes());
    }
    bytes.resize(560, 0);
    bytes
}

#[test]
fn message_tables_give_symbols_spacing_and_the_kerning_their_lines_use() {
    let mcd = Mcd::parse(&message_file()).unwrap();
    let chars: Vec<(u16, u16)> = mcd.symbols.iter().map(|s| (s.font, s.code)).collect();
    assert_eq!(chars, [(36, 'A' as u16), (36, 'V' as u16), (2, 'T' as u16)]);
    assert_eq!(mcd.glyphs[2].origin, [0.5, 0.5]);
    assert_eq!(mcd.glyphs[2].size, [3., 2.]);
    assert_eq!(mcd.spacing, [(36, -7.)]);
    assert_eq!(
        mcd.kerning,
        [
            ([36, 'A' as u16, 'V' as u16], -4),
            ([36, 'V' as u16, 'A' as u16], -9),
        ]
    );
}

#[test]
fn truncated_message_tables_are_rejected() {
    let bytes = message_file();
    for end in [0, 39, 100, 290] {
        assert!(Mcd::parse(&bytes[..end]).is_none(), "{end}");
    }
}

#[test]
fn face_tables_read_glyph_rows_and_kerning_pairs() {
    let mut ftb = vec![0; FTB_TABLE_AT + 4];
    put(&mut ftb, FTB_COUNT_AT, &2_u16.to_le_bytes());
    put(&mut ftb, FTB_TABLE_AT, &0x90_u32.to_le_bytes());
    for (i, row) in [[65_u16, 0, 10, 12, 4, 8], [0x3042, 3, 20, 22, 0, 0]]
        .iter()
        .enumerate()
    {
        for (n, word) in row.iter().enumerate() {
            put(&mut ftb, 0x90 + i * FTB_ENTRY + n * 2, &word.to_le_bytes());
        }
    }
    assert_eq!(
        parse_ftb(&ftb).unwrap(),
        [[65, 0, 10, 12, 4, 8], [0x3042, 3, 20, 22, 0, 0]]
    );
    assert!(parse_ftb(&ftb[..ftb.len() - 1]).is_none());

    let mut ktb = 2_u16.to_le_bytes().to_vec();
    for (left, right, value) in [(65_u16, 86_u16, -3_i16), (84, 111, -5)] {
        ktb.extend(left.to_le_bytes());
        ktb.extend(right.to_le_bytes());
        ktb.extend(value.to_le_bytes());
    }
    assert_eq!(parse_ktb(&ktb), [(65, 86, -3), (84, 111, -5)]);
    assert_eq!(parse_ktb(&ktb[..ktb.len() - 1]), [(65, 86, -3)]);
}

#[test]
fn crops_past_the_page_edge_are_padded_with_empty_pixels() {
    let page = Page {
        width: 3,
        height: 2,
        alpha: vec![1, 2, 3, 4, 5, 6],
    };
    assert_eq!(page.crop(1, 0, 3, 3).unwrap(), [2, 3, 0, 5, 6, 0, 0, 0, 0]);
    assert!(page.crop(3, 0, 1, 1).is_none());
    assert!(page.crop(0, 0, 0, 1).is_none());
}

fn cell(width: u32, height: u32, fill: u8) -> Cell {
    Cell {
        width,
        height,
        coverage: vec![fill; (width * height) as usize],
        advance: width as f32 - 2.,
    }
}

fn sample() -> GameFont {
    let mut sheet = Sheet::default();
    sheet.cells.insert((36, 'A' as u16), cell(5, 7, 200));
    sheet.cells.insert((36, 'V' as u16), cell(4, 9, 90));
    sheet.cells.insert((FACE | 1, 'B' as u16), cell(6, 3, 30));
    sheet.kerning.insert([36, 'A' as u16, 'V' as u16], -3);
    sheet.pack(0xfeed).unwrap()
}

#[test]
fn packed_glyphs_keep_their_pixels_without_overlapping() {
    let font = sample();
    assert_eq!(font.width, ATLAS_WIDTH);
    assert_eq!(font.glyphs.len(), 3);
    let fill = |glyph: &Glyph| {
        let [x, y, w, h] = glyph.rect.map(u32::from);
        (y..y + h)
            .flat_map(|row| (x..x + w).map(move |col| (row, col)))
            .map(|(row, col)| font.coverage[(row * font.width + col) as usize])
            .collect::<Vec<_>>()
    };
    for (glyph, value) in font.glyphs.iter().zip([200, 90, 30]) {
        assert!(fill(glyph).iter().all(|v| *v == value), "{glyph:?}");
    }
    assert_eq!(font.glyphs[0].advance, 3.);
    assert_eq!(font.kerning, [([36, 'A' as u16, 'V' as u16], -3)]);
}

#[test]
fn the_cache_round_trips_and_refuses_damaged_files() {
    let font = sample();
    let bytes = font.encode().unwrap();
    assert_eq!(GameFont::decode(&bytes).unwrap(), font);
    let png = CACHE_HEADER + 3 * GLYPH_RECORD + KERN_RECORD;
    for end in [0, 3, 19, 40, png, png + 40] {
        assert!(GameFont::decode(&bytes[..end]).is_err(), "{end}");
    }
    let mut stale = bytes.clone();
    stale[4] = 0;
    assert!(GameFont::decode(&stale).is_err());
    let mut outside = bytes;
    outside[CACHE_HEADER + 4] = 0xff;
    outside[CACHE_HEADER + 5] = 0xff;
    assert!(GameFont::decode(&outside).is_err());
}

#[test]
fn common_ranges_cover_latin_kana_and_fullwidth_but_not_ideographs() {
    for ch in ['A', 'é', 'Ж', '—', '→', 'あ', 'ア', 'Ａ'] {
        assert!(in_common_ranges(ch), "{ch}");
    }
    for ch in ['\n', '\u{7f}', '设', '한'] {
        assert!(!in_common_ranges(ch), "{ch}");
    }
}

#[test]
#[ignore = "needs YP_NIER_DIR pointing at a NieR:Automata install"]
fn the_installed_game_gives_every_face_and_the_title_capitals() {
    let root = PathBuf::from(env::var_os("YP_NIER_DIR").expect("YP_NIER_DIR"));
    let font = build(&root, in_common_ranges).unwrap();
    let has = |font_id: u16, ch: char| {
        font.glyphs
            .iter()
            .any(|glyph| glyph.font == font_id && glyph.code == ch as u16)
    };
    for ch in 'A'..='Z' {
        assert!(has(2, ch), "title {ch}");
        assert!(has(36, ch), "menu {ch}");
    }
    for (_, id) in FACES {
        assert!(has(FACE | id, 'a'), "face {id}");
    }
    assert!(font.kerning.iter().any(|(key, _)| key[0] == 36));
    assert!(font.kerning.iter().any(|(key, _)| key[0] == FACE | 1));
    assert!(font.coverage.contains(&255));
    assert_eq!(source_stamp(&root).unwrap(), font.stamp);
    assert_eq!(GameFont::decode(&font.encode().unwrap()).unwrap(), font);
}
