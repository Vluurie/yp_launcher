use std::path::Path;

use yp_core::{IoOp, IoResultExt, Result};

pub fn delete_mod(mod_dir: &Path) -> Result<bool> {
    if !mod_dir.exists() {
        return Ok(false);
    }
    std::fs::remove_dir_all(mod_dir).io(IoOp::Remove, mod_dir)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletes_a_folder_once() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("HD").join("movie");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("ev0010.usm"), b"x").unwrap();
        assert!(delete_mod(&dir.path().join("HD")).unwrap());
        assert!(!dir.path().join("HD").exists());
        assert!(!delete_mod(&dir.path().join("HD")).unwrap());
    }
}
