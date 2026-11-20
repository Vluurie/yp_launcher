use std::io::Cursor;
use std::path::PathBuf;

use yp_formats::FormatError;
use yp_formats::dat::{DatArchive, name_crc};
use yp_formats::files;
use yp_formats::texture::{self, DDS_HEADER_SIZE};

const TEXTURE_NAME: &str = "item_thumb_1234";

fn fixture(name: &str) -> PathBuf {
    yp_core::testing::fixtures::path(&format!("yp_formats/{name}"))
}

fn read_fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixture(name)).unwrap()
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[test]
fn matches_the_output_of_the_shipped_flutter_build() {
    let source = read_fixture("misctex_source.png");
    let texture = texture::create_misctex(&source, TEXTURE_NAME, 64, 32).unwrap();
    assert_eq!(texture.dat, read_fixture("misctex_item_thumb_1234.dat"));
    assert_eq!(texture.dtt, read_fixture("misctex_item_thumb_1234.dtt"));
    assert_eq!((texture.width, texture.height), (64, 32));
}

#[test]
fn golden_archives_have_the_expected_layout() {
    let dat = read_fixture("misctex_item_thumb_1234.dat");
    let mut archive = DatArchive::from_reader(Cursor::new(dat)).unwrap();
    let names: Vec<String> = archive
        .entries()
        .iter()
        .map(|entry| entry.name.clone())
        .collect();
    assert_eq!(
        names,
        ["misctex_item_thumb_1234.wta", "misctex_item_thumb_1234.uvd"]
    );
    let wta = archive
        .read_named("misctex_item_thumb_1234.wta")
        .unwrap()
        .unwrap();
    let uvd = archive
        .read_named("misctex_item_thumb_1234.uvd")
        .unwrap()
        .unwrap();

    let crc = name_crc(TEXTURE_NAME);
    assert_eq!(&wta[0..4], b"WTB\0");
    assert_eq!(u32_at(&wta, 8), 1);
    assert_eq!(u32_at(&wta, 0x40), (DDS_HEADER_SIZE + 64 * 32) as u32);
    assert_eq!(u32_at(&wta, 0x60), 0x2200_0022);
    assert_eq!(u32_at(&wta, 0x80), crc);
    assert_eq!(u32_at(&wta, 0xA0), 77);

    assert_eq!(
        &uvd[0x10..0x10 + TEXTURE_NAME.len()],
        TEXTURE_NAME.as_bytes()
    );
    assert_eq!(u32_at(&uvd, 0x30), crc);
    assert_eq!(u32_at(&uvd, 0x74), name_crc("item_thumb_1234_temp"));
    assert_eq!(u32_at(&uvd, 0x78), crc);
    assert_eq!(f32_at(&uvd, 0x84), 64.0);
    assert_eq!(f32_at(&uvd, 0x88), 32.0);
    assert_eq!(f32_at(&uvd, 0x8C), 1.0 / 64.0);

    let dtt = read_fixture("misctex_item_thumb_1234.dtt");
    let mut textures = DatArchive::from_reader(Cursor::new(dtt)).unwrap();
    assert_eq!(textures.entries().len(), 1);
    let wtp = textures
        .read_named("misctex_item_thumb_1234.wtp")
        .unwrap()
        .unwrap();
    assert_eq!(&wtp[0..4], b"DDS ");
    assert_eq!(&wtp[84..88], b"DXT5");
    assert_eq!(u32_at(&wtp, 12), 32);
    assert_eq!(u32_at(&wtp, 16), 64);
}

#[test]
fn thumbnail_files_are_written_next_to_each_other() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("mod").join("data").join("misctex");
    let result =
        files::create_item_thumbnail(&fixture("misctex_source.png"), &out, TEXTURE_NAME, 64, 32)
            .unwrap();
    assert_eq!(result.dat_path, out.join("misctex_item_thumb_1234.dat"));
    assert_eq!(result.dtt_path, out.join("misctex_item_thumb_1234.dtt"));
    assert_eq!((result.width, result.height), (64, 32));
    assert_eq!(result.dat_bytes, 512);
    assert_eq!(result.dtt_bytes, 2320);
    assert_eq!(
        std::fs::read(&result.dat_path).unwrap(),
        read_fixture("misctex_item_thumb_1234.dat")
    );
    assert_eq!(
        std::fs::read(&result.dtt_path).unwrap(),
        read_fixture("misctex_item_thumb_1234.dtt")
    );
    assert_eq!(
        files::misctex_paths(&out, TEXTURE_NAME),
        (result.dat_path, result.dtt_path)
    );
}

#[test]
fn thumbnail_errors_are_typed() {
    let dir = tempfile::tempdir().unwrap();
    let missing =
        files::create_item_thumbnail(&dir.path().join("none.png"), dir.path(), "x", 64, 32)
            .unwrap_err();
    assert_eq!(missing.code(), 1300);
    let bad_size =
        files::create_item_thumbnail(&fixture("misctex_source.png"), dir.path(), "x", 62, 32)
            .unwrap_err();
    assert!(matches!(
        bad_size,
        FormatError::ImageSizeNotMultipleOfFour {
            width: 62,
            height: 32
        }
    ));
    let blocker = dir.path().join("file");
    std::fs::write(&blocker, b"x").unwrap();
    let create = files::create_item_thumbnail(
        &fixture("misctex_source.png"),
        &blocker.join("sub"),
        "x",
        64,
        32,
    )
    .unwrap_err();
    assert_eq!(create.code(), 1302);
}

fn rgba_of(png: &[u8]) -> Vec<u8> {
    image::load_from_memory(png).unwrap().to_rgba8().into_raw()
}

#[test]
fn preview_matches_the_shipped_decoder() {
    let preview = files::read_misctex_preview(&fixture("misctex_item_thumb_1234.dtt")).unwrap();
    assert_eq!((preview.width, preview.height), (64, 32));
    assert_eq!(
        rgba_of(&preview.png),
        rgba_of(&read_fixture("misctex_item_thumb_1234_preview.png"))
    );
}

#[test]
fn preview_of_an_archive_without_wtp_is_typed() {
    let error = files::read_misctex_preview(&fixture("misctex_item_thumb_1234.dat")).unwrap_err();
    assert!(matches!(error, FormatError::TextureNoWtp));
    assert!(error.is_texture_unreadable());
    let not_dat = files::read_misctex_preview(&fixture("misctex_source.png")).unwrap_err();
    assert!(matches!(not_dat, FormatError::Dat(_)));
}

#[test]
#[ignore = "needs YP_MISCTEX_DTT pointing at a misctex .dtt from a real mod"]
fn previews_a_real_misctex() {
    let path = PathBuf::from(std::env::var_os("YP_MISCTEX_DTT").expect("YP_MISCTEX_DTT"));
    let preview = files::read_misctex_preview(&path).unwrap();
    assert!(preview.width > 0 && preview.height > 0);
    assert_eq!(&preview.png[1..4], b"PNG");
}
