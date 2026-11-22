use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsDll {
    Other,
    ReShade,
    Migoto,
}

const RESHADE_MARKERS: [&[u8]; 2] = [b"reshade.me", b"crosire"];

const MIGOTO_MARKERS: [&[u8]; 3] = [b"HackerDevice", b"HackerContext", b"3DMigoto"];

pub fn identify_bytes(bytes: &[u8]) -> GraphicsDll {
    if any_marker(bytes, &MIGOTO_MARKERS) {
        GraphicsDll::Migoto
    } else if any_marker(bytes, &RESHADE_MARKERS) {
        GraphicsDll::ReShade
    } else {
        GraphicsDll::Other
    }
}

pub fn identify_file(path: &Path) -> GraphicsDll {
    match std::fs::read(path) {
        Ok(bytes) => identify_bytes(&bytes),
        Err(_) => GraphicsDll::Other,
    }
}

pub fn is_reshade(bytes: &[u8]) -> bool {
    identify_bytes(bytes) == GraphicsDll::ReShade
}

pub fn is_migoto(bytes: &[u8]) -> bool {
    identify_bytes(bytes) == GraphicsDll::Migoto
}

fn any_marker(haystack: &[u8], markers: &[&[u8]]) -> bool {
    markers.iter().any(|needle| contains_bytes(haystack, needle))
}

pub(crate) fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && needle.len() <= haystack.len() && haystack.windows(needle.len()).any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_identify_runtimes() {
        assert_eq!(identify_bytes(b"MZ.. reshade.me .."), GraphicsDll::ReShade);
        assert_eq!(identify_bytes(b"MZ crosire"), GraphicsDll::ReShade);
        assert_eq!(identify_bytes(b"MZ HackerDevice"), GraphicsDll::Migoto);
        assert_eq!(identify_bytes(b"MZ HackerContext reshade.me"), GraphicsDll::Migoto);
        assert_eq!(identify_bytes(b"MZ 3DMigoto"), GraphicsDll::Migoto);
        assert_eq!(identify_bytes(b"MZ plain"), GraphicsDll::Other);
        assert_eq!(identify_bytes(b""), GraphicsDll::Other);
        assert!(is_reshade(b"crosire"));
        assert!(is_migoto(b"3DMigoto"));
    }

    #[test]
    fn missing_file_is_not_a_graphics_dll() {
        assert_eq!(identify_file(Path::new("definitely/not/here.dll")), GraphicsDll::Other);
    }
}
