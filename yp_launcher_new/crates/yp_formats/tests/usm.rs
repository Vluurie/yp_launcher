use std::io::Cursor;
use std::path::PathBuf;

use yp_formats::usm::{
    self, ColumnStorage, ColumnType, PayloadKind, UtfTable, UtfValue, VideoCodec, VideoInfo,
};
use yp_formats::{FormatError, UsmFault};

#[derive(Clone)]
enum Val {
    U8(u8),
    I16(i16),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Str(&'static str),
    Data(&'static [u8]),
}

impl Val {
    fn type_code(&self) -> u8 {
        match self {
            Val::U8(_) => 0x0,
            Val::I16(_) => 0x3,
            Val::U16(_) => 0x2,
            Val::U32(_) => 0x4,
            Val::U64(_) => 0x6,
            Val::F32(_) => 0x8,
            Val::F64(_) => 0x9,
            Val::Str(_) => 0xA,
            Val::Data(_) => 0xB,
        }
    }
}

enum Col {
    Row(&'static str, Vec<Val>),
    Const(&'static str, Val),
    Zero(&'static str, u8),
    Raw(&'static str, u8),
}

struct Strings {
    bytes: Vec<u8>,
}

impl Strings {
    fn new() -> Self {
        Self {
            bytes: b"<NULL>\0".to_vec(),
        }
    }

    fn add(&mut self, value: &str) -> u32 {
        let offset = self.bytes.len() as u32;
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(0);
        offset
    }
}

fn encode(value: &Val, strings: &mut Strings, data: &mut Vec<u8>, out: &mut Vec<u8>) {
    match value {
        Val::U8(v) => out.push(*v),
        Val::I16(v) => out.extend_from_slice(&v.to_be_bytes()),
        Val::U16(v) => out.extend_from_slice(&v.to_be_bytes()),
        Val::U32(v) => out.extend_from_slice(&v.to_be_bytes()),
        Val::U64(v) => out.extend_from_slice(&v.to_be_bytes()),
        Val::F32(v) => out.extend_from_slice(&v.to_be_bytes()),
        Val::F64(v) => out.extend_from_slice(&v.to_be_bytes()),
        Val::Str(v) => out.extend_from_slice(&strings.add(v).to_be_bytes()),
        Val::Data(v) => {
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            out.extend_from_slice(&(v.len() as u32).to_be_bytes());
            data.extend_from_slice(v);
        }
    }
}

fn utf_table(table_name: &str, columns: &[Col], rows: usize) -> Vec<u8> {
    let mut strings = Strings::new();
    let mut data = Vec::new();
    let name_offset = strings.add(table_name);
    let mut descriptors = Vec::new();
    for column in columns {
        match column {
            Col::Row(name, values) => {
                descriptors.push(0x50 | values[0].type_code());
                descriptors.extend_from_slice(&strings.add(name).to_be_bytes());
            }
            Col::Const(name, value) => {
                descriptors.push(0x30 | value.type_code());
                descriptors.extend_from_slice(&strings.add(name).to_be_bytes());
                encode(value, &mut strings, &mut data, &mut descriptors);
            }
            Col::Zero(name, code) => {
                descriptors.push(0x10 | code);
                descriptors.extend_from_slice(&strings.add(name).to_be_bytes());
            }
            Col::Raw(name, flags) => {
                descriptors.push(*flags);
                descriptors.extend_from_slice(&strings.add(name).to_be_bytes());
            }
        }
    }
    let mut row_bytes = Vec::new();
    let mut row_width = 0usize;
    for row in 0..rows {
        let start = row_bytes.len();
        for column in columns {
            if let Col::Row(_, values) = column {
                encode(&values[row], &mut strings, &mut data, &mut row_bytes);
            }
        }
        row_width = row_bytes.len() - start;
    }
    let rows_offset = 24 + descriptors.len();
    let strings_offset = rows_offset + row_bytes.len();
    let data_offset = strings_offset + strings.bytes.len();
    let table_size = data_offset + data.len();

    let mut out = Vec::new();
    out.extend_from_slice(b"@UTF");
    out.extend_from_slice(&(table_size as u32).to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&(rows_offset as u16).to_be_bytes());
    out.extend_from_slice(&(strings_offset as u32).to_be_bytes());
    out.extend_from_slice(&(data_offset as u32).to_be_bytes());
    out.extend_from_slice(&name_offset.to_be_bytes());
    out.extend_from_slice(&(columns.len() as u16).to_be_bytes());
    out.extend_from_slice(&(row_width as u16).to_be_bytes());
    out.extend_from_slice(&(rows as u32).to_be_bytes());
    out.extend_from_slice(&descriptors);
    out.extend_from_slice(&row_bytes);
    out.extend_from_slice(&strings.bytes);
    out.extend_from_slice(&data);
    out
}

fn chunk(signature: &[u8; 4], kind: u8, payload: &[u8], padding: u16) -> Vec<u8> {
    let size = 0x18 + payload.len() + padding as usize;
    let mut out = Vec::new();
    out.extend_from_slice(signature);
    out.extend_from_slice(&(size as u32).to_be_bytes());
    out.push(0);
    out.push(0x18);
    out.extend_from_slice(&padding.to_be_bytes());
    out.push(0);
    out.extend_from_slice(&[0, 0]);
    out.push(kind);
    out.extend_from_slice(&0u32.to_be_bytes());
    out.extend_from_slice(&30u32.to_be_bytes());
    out.extend_from_slice(&[0; 8]);
    out.extend_from_slice(payload);
    out.extend(std::iter::repeat_n(0u8, padding as usize));
    out
}

fn crid() -> Vec<u8> {
    let table = utf_table(
        "CRIUSF_DIR_STREAM",
        &[
            Col::Const("fmtver", Val::U32(0x0100_0700)),
            Col::Row(
                "filename",
                vec![Val::Str("ev0010.usm"), Val::Str("ev0010.avi")],
            ),
            Col::Row("filesize", vec![Val::U32(68_472_800), Val::U32(1)]),
            Col::Row("stmid", vec![Val::U32(0), Val::U32(0x4053_4656)]),
            Col::Row("chno", vec![Val::I16(-1), Val::I16(0)]),
        ],
        2,
    );
    chunk(b"CRID", 1, &table, 0x40)
}

fn video_header(columns: &[Col]) -> Vec<u8> {
    chunk(
        b"@SFV",
        1,
        &utf_table("CRIUSF_VIDEOHDR_TABLE", columns, 1),
        0x20,
    )
}

fn basic_columns(width: u32, height: u32, codec: u32) -> Vec<Col> {
    vec![
        Col::Row("width", vec![Val::U32(width)]),
        Col::Row("height", vec![Val::U32(height)]),
        Col::Row("mat_width", vec![Val::U32(width)]),
        Col::Row("mat_height", vec![Val::U32(height)]),
        Col::Row("disp_width", vec![Val::U32(width)]),
        Col::Row("disp_height", vec![Val::U32(height)]),
        Col::Row("total_frames", vec![Val::U32(1200)]),
        Col::Row("framerate_n", vec![Val::U32(30000)]),
        Col::Row("framerate_d", vec![Val::U32(1001)]),
        Col::Row("mpeg_codec", vec![Val::U32(codec)]),
    ]
}

fn stream_chunk(signature: &[u8; 4]) -> Vec<u8> {
    chunk(signature, 0, &[0xAB; 64], 0)
}

fn file(chunks: &[Vec<u8>]) -> Vec<u8> {
    chunks.concat()
}

fn info(bytes: Vec<u8>) -> Result<VideoInfo, FormatError> {
    usm::read_video_info(Cursor::new(bytes))
}

fn fault(bytes: Vec<u8>) -> UsmFault {
    match info(bytes) {
        Err(FormatError::Usm(fault)) => fault,
        other => panic!("expected a USM fault, got {other:?}"),
    }
}

#[test]
fn reads_size_and_codec_from_per_row_columns() {
    let bytes = file(&[
        crid(),
        video_header(&basic_columns(1920, 1080, 1)),
        stream_chunk(b"@SFV"),
    ]);
    let info = info(bytes).unwrap();
    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);
    assert_eq!(info.codec, VideoCodec::SofdecPrime);
    assert!(info.is_vanilla_shape());
}

#[test]
fn takes_the_largest_of_coded_matrix_and_display_sizes() {
    let columns = vec![
        Col::Row("width", vec![Val::U32(1600)]),
        Col::Row("height", vec![Val::U32(900)]),
        Col::Row("mat_width", vec![Val::U32(1616)]),
        Col::Row("mat_height", vec![Val::U32(912)]),
        Col::Row("disp_width", vec![Val::U32(2560)]),
        Col::Row("disp_height", vec![Val::U32(904)]),
        Col::Row("mpeg_codec", vec![Val::U8(5)]),
    ];
    let info = info(file(&[crid(), video_header(&columns)])).unwrap();
    assert_eq!((info.width, info.height), (2560, 912));
    assert_eq!(info.codec, VideoCodec::H264);
    assert!(!info.is_vanilla_shape());
}

#[test]
fn constant_columns_come_from_the_descriptor() {
    let columns = vec![
        Col::Const("width", Val::U32(2560)),
        Col::Const("height", Val::U16(1440)),
        Col::Row("total_frames", vec![Val::U32(10)]),
        Col::Const("mpeg_codec", Val::U32(5)),
    ];
    let info = info(file(&[crid(), video_header(&columns)])).unwrap();
    assert_eq!(
        (info.width, info.height, info.codec),
        (2560, 1440, VideoCodec::H264)
    );
}

#[test]
fn wide_float_string_and_data_columns_are_skipped_correctly() {
    let columns = vec![
        Col::Row("stream_size", vec![Val::U64(u64::MAX)]),
        Col::Row("frame_rate", vec![Val::F64(29.97)]),
        Col::Row("gamma", vec![Val::F32(2.2)]),
        Col::Row("name", vec![Val::Str("movie")]),
        Col::Row("blob", vec![Val::Data(b"0123456789")]),
        Col::Const("const_blob", Val::Data(b"abc")),
        Col::Const("const_wide", Val::U64(7)),
        Col::Row("signed", vec![Val::I16(-5)]),
        Col::Row("width", vec![Val::U32(3840)]),
        Col::Row("height", vec![Val::U32(2160)]),
        Col::Row("mpeg_codec", vec![Val::U32(9)]),
    ];
    let info = info(file(&[crid(), video_header(&columns)])).unwrap();
    assert_eq!(
        (info.width, info.height, info.codec),
        (3840, 2160, VideoCodec::Vp9)
    );
}

#[test]
fn zero_storage_columns_read_as_zero_and_take_no_row_bytes() {
    let columns = vec![
        Col::Zero("alpha_type", 0x4),
        Col::Row("width", vec![Val::U32(1280)]),
        Col::Zero("mat_width", 0x4),
        Col::Row("height", vec![Val::U32(720)]),
        Col::Row("mpeg_codec", vec![Val::U32(5)]),
    ];
    let info = info(file(&[crid(), video_header(&columns)])).unwrap();
    assert_eq!((info.width, info.height), (1280, 720));
}

#[test]
fn missing_size_columns_read_as_zero() {
    let columns = vec![Col::Row("total_frames", vec![Val::U32(1)])];
    let info = info(file(&[crid(), video_header(&columns)])).unwrap();
    assert_eq!(
        (info.width, info.height, info.codec),
        (0, 0, VideoCodec::Other(0))
    );
}

#[test]
fn skips_audio_headers_and_stream_chunks_before_the_video_header() {
    let audio = chunk(
        b"@SFA",
        1,
        &utf_table(
            "CRIUSF_AUDIOHDR_TABLE",
            &[Col::Row("sampling_rate", vec![Val::U32(48000)])],
            1,
        ),
        0,
    );
    let metadata = chunk(
        b"@SFV",
        3,
        &utf_table(
            "VIDEO_SEEKINFO",
            &[Col::Row("ofs_byte", vec![Val::U64(0)])],
            1,
        ),
        0,
    );
    let bytes = file(&[
        crid(),
        audio,
        stream_chunk(b"@SFA"),
        metadata,
        stream_chunk(b"@SFV"),
        video_header(&basic_columns(1600, 904, 1)),
    ]);
    let info = info(bytes).unwrap();
    assert_eq!((info.width, info.height), (1600, 904));
    assert!(info.is_vanilla_shape());
}

#[test]
fn the_first_chunk_must_be_crid() {
    let bytes = file(&[video_header(&basic_columns(1920, 1080, 1)), crid()]);
    assert_eq!(fault(bytes), UsmFault::NotUsm);
    assert_eq!(fault(b"RIFF\0\0\0\0WAVEfmt ".repeat(4)), UsmFault::NotUsm);
}

#[test]
fn a_file_without_a_video_header_is_reported() {
    let bytes = file(&[crid(), stream_chunk(b"@SFA"), stream_chunk(b"@SFV")]);
    assert_eq!(fault(bytes), UsmFault::NoVideoHeader);
}

#[test]
fn an_empty_or_tiny_file_is_not_usm() {
    assert_eq!(fault(Vec::new()), UsmFault::NotUsm);
    assert_eq!(fault(b"CRID".to_vec()), UsmFault::Truncated);
}

#[test]
fn a_header_table_without_rows_is_reported() {
    let header = chunk(
        b"@SFV",
        1,
        &utf_table(
            "CRIUSF_VIDEOHDR_TABLE",
            &[Col::Const("width", Val::U32(1))],
            0,
        ),
        0,
    );
    assert_eq!(fault(file(&[crid(), header])), UsmFault::EmptyTable);
}

#[test]
fn truncation_inside_the_video_header_is_reported() {
    let mut bytes = file(&[crid(), video_header(&basic_columns(1920, 1080, 1))]);
    let crid_len = crid().len();
    bytes.truncate(crid_len + 0x30);
    assert_eq!(fault(bytes), UsmFault::Truncated);
}

#[test]
fn a_chunk_that_overruns_the_file_is_truncated() {
    let mut bytes = crid();
    bytes.extend_from_slice(b"@SFV");
    bytes.extend_from_slice(&0x7FFF_FFFFu32.to_be_bytes());
    bytes.extend_from_slice(&[0, 0x18, 0, 0, 0, 0, 0, 1]);
    assert_eq!(fault(bytes), UsmFault::Truncated);
}

#[test]
fn a_payload_offset_past_the_chunk_is_invalid() {
    let mut header = video_header(&basic_columns(1920, 1080, 1));
    header[9] = 0xFF;
    let size = 0x10u32;
    header[4..8].copy_from_slice(&size.to_be_bytes());
    header.truncate(8 + size as usize);
    assert_eq!(fault(file(&[crid(), header])), UsmFault::InvalidChunk);
}

#[test]
fn a_header_payload_without_utf_magic_is_invalid() {
    let header = chunk(b"@SFV", 1, b"@XYZ\0\0\0\x08\0\0\0\0\0\0\0\0", 0);
    assert_eq!(fault(file(&[crid(), header])), UsmFault::InvalidTable);
}

#[test]
fn unknown_column_types_are_reported() {
    let columns = vec![
        Col::Raw("mystery", 0x5C),
        Col::Row("width", vec![Val::U32(1)]),
    ];
    let header = chunk(
        b"@SFV",
        1,
        &utf_table("CRIUSF_VIDEOHDR_TABLE", &columns, 1),
        0,
    );
    assert_eq!(
        fault(file(&[crid(), header])),
        UsmFault::UnknownColumnType(0xC)
    );
}

#[test]
fn table_offsets_outside_the_table_are_invalid() {
    let mut table = utf_table("T", &basic_columns(1, 1, 1), 1);
    table[12..16].copy_from_slice(&0xFFFF_0000u32.to_be_bytes());
    assert!(matches!(
        UtfTable::parse(&table),
        Err(FormatError::Usm(UsmFault::InvalidTable))
    ));
}

#[test]
fn utf_tables_expose_names_rows_strings_and_data() {
    let table = utf_table(
        "CRIUSF_DIR_STREAM",
        &[
            Col::Const("fmtver", Val::U32(16_777_984)),
            Col::Row("filename", vec![Val::Str("a.usm"), Val::Str("a.avi")]),
            Col::Row("chno", vec![Val::I16(-1), Val::I16(0)]),
            Col::Row("blob", vec![Val::Data(b"xy"), Val::Data(b"")]),
            Col::Zero("minbuf", 0x4),
        ],
        2,
    );
    let parsed = UtfTable::parse(&table).unwrap();
    assert_eq!(parsed.name, "CRIUSF_DIR_STREAM");
    assert_eq!(parsed.rows.len(), 2);
    assert_eq!(parsed.columns.len(), 5);
    assert_eq!(parsed.columns[0].kind, ColumnType::U32);
    assert!(matches!(
        parsed.columns[0].storage,
        ColumnStorage::Constant(UtfValue::U32(16_777_984))
    ));
    assert_eq!(parsed.columns[1].storage, ColumnStorage::PerRow);
    assert_eq!(parsed.columns[4].storage, ColumnStorage::Zero);
    assert_eq!(
        parsed.value(1, "filename"),
        Some(&UtfValue::String("a.avi".to_string()))
    );
    assert_eq!(parsed.value(0, "chno"), Some(&UtfValue::I16(-1)));
    assert_eq!(
        parsed.value(0, "blob"),
        Some(&UtfValue::Data(b"xy".to_vec()))
    );
    assert_eq!(parsed.value(1, "fmtver"), Some(&UtfValue::U32(16_777_984)));
    assert_eq!(parsed.unsigned(1, "minbuf"), Some(0));
    assert_eq!(parsed.unsigned(0, "chno"), None);
    assert_eq!(parsed.value(2, "filename"), None);
    assert_eq!(parsed.value(0, "missing"), None);
}

#[test]
fn chunk_reader_walks_every_chunk() {
    let bytes = file(&[
        crid(),
        stream_chunk(b"@SFA"),
        video_header(&basic_columns(1, 1, 1)),
    ]);
    let mut reader = usm::ChunkReader::new(Cursor::new(bytes)).unwrap();
    let mut seen = Vec::new();
    while let Some(chunk) = reader.next_chunk().unwrap() {
        seen.push((chunk.signature, chunk.kind));
    }
    assert_eq!(
        seen,
        vec![
            (*b"CRID", PayloadKind::Header),
            (*b"@SFA", PayloadKind::Stream),
            (*b"@SFV", PayloadKind::Header),
        ]
    );
}

#[test]
fn codec_codes_round_trip() {
    for code in [0, 1, 5, 9, 42] {
        assert_eq!(VideoCodec::from_code(code).code(), code);
    }
    assert!(VideoCodec::H264.is_h264());
    assert!(!VideoCodec::SofdecPrime.is_h264());
}

#[test]
fn vanilla_shape_needs_sofdec_at_a_shipped_size() {
    let shape = |width, height, codec| {
        VideoInfo {
            width,
            height,
            codec,
        }
        .is_vanilla_shape()
    };
    assert!(shape(1600, 904, VideoCodec::SofdecPrime));
    assert!(shape(1920, 1080, VideoCodec::SofdecPrime));
    assert!(!shape(1920, 1080, VideoCodec::H264));
    assert!(!shape(2560, 1440, VideoCodec::SofdecPrime));
}

#[test]
fn probes_files_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("EV0010.USM");
    std::fs::write(
        &good,
        file(&[crid(), video_header(&basic_columns(2560, 1440, 5))]),
    )
    .unwrap();
    let bad = dir.path().join("broken.usm");
    std::fs::write(&bad, b"nope").unwrap();
    assert!(usm::is_usm_path(&good));
    assert!(!usm::is_usm_path(&dir.path().join("a.usm.txt")));
    assert_eq!(
        usm::probe(&good).unwrap(),
        VideoInfo {
            width: 2560,
            height: 1440,
            codec: VideoCodec::H264
        }
    );
    assert_eq!(usm::probe_quiet(&bad), None);
    assert_eq!(usm::probe_quiet(&dir.path().join("missing.usm")), None);
    assert!(matches!(
        usm::probe(&dir.path().join("missing.usm")),
        Err(FormatError::Io { .. })
    ));
}

fn env_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
}

fn usm_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| usm::is_usm_path(path))
        .collect();
    files.sort();
    files
}

#[test]
#[ignore = "needs YP_VANILLA_MOVIE_DIR pointing at data/movie"]
fn vanilla_movies_have_the_vanilla_shape() {
    let dir = env_dir("YP_VANILLA_MOVIE_DIR").expect("YP_VANILLA_MOVIE_DIR");
    let files = usm_files(&dir);
    assert!(!files.is_empty());
    for path in files {
        let info = usm::probe(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert!(info.is_vanilla_shape(), "{}: {info:?}", path.display());
    }
}

#[test]
#[ignore = "needs YP_CUTSCENE_MOD_MOVIE_DIR pointing at an installed cutscene mod movie folder"]
fn modded_movies_parse() {
    let dir = env_dir("YP_CUTSCENE_MOD_MOVIE_DIR").expect("YP_CUTSCENE_MOD_MOVIE_DIR");
    let files = usm_files(&dir);
    assert!(!files.is_empty());
    for path in files {
        let info = usm::probe(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert!(
            info.width > 0 && info.height > 0,
            "{}: {info:?}",
            path.display()
        );
        assert!(!info.is_vanilla_shape(), "{}: {info:?}", path.display());
    }
}
