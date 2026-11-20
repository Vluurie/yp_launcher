use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::error::{IoOp, IoResultExt, Result};

fn digest_file<D: Digest>(path: &Path) -> Result<D> {
    let mut file = File::open(path).io(IoOp::Read, path)?;
    let mut hasher = D::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buffer).io(IoOp::Read, path)?;
        if read == 0 {
            return Ok(hasher);
        }
        hasher.update(&buffer[..read]);
    }
}

pub fn sha256_file(path: &Path) -> Result<[u8; 32]> {
    Ok(digest_file::<Sha256>(path)?.finalize().into())
}

pub fn sha1_file(path: &Path) -> Result<[u8; 20]> {
    Ok(digest_file::<Sha1>(path)?.finalize().into())
}

pub fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_digests() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, b"abc").unwrap();
        assert_eq!(
            to_hex(&sha256_file(&file).unwrap()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(to_hex(&sha1_file(&file).unwrap()), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(sha256_bytes(b"abc"), sha256_file(&file).unwrap());
    }
}
