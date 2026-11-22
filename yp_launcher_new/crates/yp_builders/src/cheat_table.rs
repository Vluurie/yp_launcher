use std::ops::Range;
use std::path::{Path, PathBuf};

use quick_xml::Reader;
use quick_xml::events::Event;

pub const OLD_MODULE: &str = "NieRAutomata.exe";
pub const NEW_MODULE: &str = "game.bin";
pub const OUTPUT_SUFFIX: &str = "-NAMS";
pub const DEFAULT_EXTENSION: &str = ".CT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEncoding {
    Utf8,
    Latin1,
}

impl TextEncoding {
    pub fn detect(bytes: &[u8]) -> Self {
        if std::str::from_utf8(bytes).is_ok() {
            TextEncoding::Utf8
        } else {
            TextEncoding::Latin1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheatTableOutcome {
    NothingToReplace,
    Converted {
        replacements: usize,
        output: PathBuf,
        encoding: TextEncoding,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CheatTableError {
    #[error("cheat table not found")]
    NotFound,
    #[error("cheat table unreadable")]
    Unreadable(#[source] std::io::Error),
    #[error("cheat table is not well-formed XML near byte {position}")]
    Malformed { position: u64 },
    #[error("converted cheat table could not be written")]
    WriteFailed(#[source] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rewritten {
    pub bytes: Vec<u8>,
    pub replacements: usize,
}

pub fn output_path_for(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let extension = input
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_else(|| DEFAULT_EXTENSION.to_owned());
    input.with_file_name(format!("{stem}{OUTPUT_SUFFIX}{extension}"))
}

pub fn convert(input: &Path) -> Result<CheatTableOutcome, CheatTableError> {
    if !input.is_file() {
        return Err(CheatTableError::NotFound);
    }
    let bytes = std::fs::read(input).map_err(CheatTableError::Unreadable)?;
    let rewritten = rewrite_module_name(&bytes)?;
    if rewritten.replacements == 0 {
        return Ok(CheatTableOutcome::NothingToReplace);
    }
    let output = output_path_for(input);
    std::fs::write(&output, &rewritten.bytes).map_err(CheatTableError::WriteFailed)?;
    Ok(CheatTableOutcome::Converted {
        replacements: rewritten.replacements,
        output,
        encoding: TextEncoding::detect(&bytes),
    })
}

pub fn rewrite_module_name(input: &[u8]) -> Result<Rewritten, CheatTableError> {
    // quick-xml 0.42 expects UTF-8.
    // For ISO-8859-1 input, create a same-length parsing view
    // while preserving the original bytes for output.
    let parsing_bytes: Vec<u8> = if input
        .windows(b"encoding=\"ISO-8859-1\"".len())
        .any(|window| window.eq_ignore_ascii_case(b"encoding=\"ISO-8859-1\""))
    {
        input
            .iter()
            .map(|&byte| if byte >= 0x80 { b'X' } else { byte })
            .collect()
    } else {
        input.to_vec()
    };

    let mut reader = Reader::from_reader(parsing_bytes.as_slice());
    let config = reader.config_mut();
    config.check_end_names = false;
    config.allow_unmatched_ends = true;
    config.allow_dangling_amp = true;

    let mut spans = Vec::new();
    loop {
        let start = position(reader.buffer_position());
        let event = reader
            .read_event()
            .map_err(|_| CheatTableError::Malformed {
                position: reader.error_position(),
            })?;
        let end = position(reader.buffer_position());
        match event {
            Event::Eof => break,
            Event::Text(_) | Event::CData(_) | Event::Comment(_) => {
                collect_needles(input, start..end, &mut spans)
            }
            Event::Start(_) | Event::Empty(_) => {
                collect_attribute_values(input, start..end, &mut spans);
            }
            _ => {}
        }
    }

    let mut bytes = Vec::with_capacity(input.len());
    let mut cursor = 0;
    for span in &spans {
        bytes.extend_from_slice(&input[cursor..span.start]);
        bytes.extend_from_slice(NEW_MODULE.as_bytes());
        cursor = span.end;
    }
    bytes.extend_from_slice(&input[cursor..]);
    Ok(Rewritten {
        bytes,
        replacements: spans.len(),
    })
}

fn collect_attribute_values(
    input: &[u8],
    range: Range<usize>,
    spans: &mut Vec<Range<usize>>,
) {
    let end = range.end.min(input.len());
    let mut index = range.start.saturating_add(1);

    // Skip the XML element name.
    while index < end
        && !input[index].is_ascii_whitespace()
        && input[index] != b'>'
        && input[index] != b'/'
    {
        index += 1;
    }

    while index < end {
        // Skip whitespace and the self-closing slash.
        while index < end
            && (input[index].is_ascii_whitespace()
                || input[index] == b'/')
        {
            index += 1;
        }

        if index >= end || input[index] == b'>' {
            break;
        }

        // Skip the attribute name.
        while index < end
            && !input[index].is_ascii_whitespace()
            && input[index] != b'='
            && input[index] != b'>'
        {
            index += 1;
        }

        while index < end && input[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= end || input[index] != b'=' {
            continue;
        }

        index += 1;

        while index < end && input[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= end {
            break;
        }

        let quote = input[index];

        if quote != b'"' && quote != b'\'' {
            // Ignore unquoted values.
            continue;
        }

        index += 1;
        let value_start = index;

        while index < end && input[index] != quote {
            index += 1;
        }

        collect_needles(input, value_start..index, spans);

        if index < end {
            index += 1;
        }
    }
}

fn position(offset: u64) -> usize {
    usize::try_from(offset).unwrap_or(usize::MAX)
}


fn collect_needles(input: &[u8], range: Range<usize>, spans: &mut Vec<Range<usize>>) {
    let needle = OLD_MODULE.as_bytes();
    let end = range.end.min(input.len());
    let mut index = range.start;
    while index + needle.len() <= end {
        if input[index..index + needle.len()].eq_ignore_ascii_case(needle) {
            spans.push(index..index + needle.len());
            index += needle.len();
        } else {
            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_names_keep_the_extension_or_default_to_ct() {
        assert_eq!(
            output_path_for(Path::new("dir/table.CT")),
            Path::new("dir").join("table-NAMS.CT")
        );
        assert_eq!(
            output_path_for(Path::new("dir/table")),
            Path::new("dir").join("table-NAMS.CT")
        );
    }

    #[test]
    fn tags_and_attribute_names_are_never_touched() {
        let xml = b"<NieRAutomata.exe a=\"x\">nierautomata.EXE+10</NieRAutomata.exe>";
        let out = rewrite_module_name(xml).unwrap();
        assert_eq!(out.replacements, 1);
        assert_eq!(
            out.bytes,
            b"<NieRAutomata.exe a=\"x\">game.bin+10</NieRAutomata.exe>"
        );
    }

    #[test]
    fn broken_xml_is_reported() {
        assert!(matches!(
            rewrite_module_name(b"<a><b x=\"1></a>"),
            Err(CheatTableError::Malformed { .. })
        ));
    }
}
