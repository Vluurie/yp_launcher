use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::bundled;
use crate::error::Result;
use crate::texture_config::prune_file_quiet;

pub fn uninstall(game: &GamePaths, mod_id: &str, delete_bundled_textures: bool) -> Result<()> {
    let dir = game.mods_dir().join(mod_id);
    if !dir.is_dir() {
        return Ok(());
    }
    let packs = if delete_bundled_textures {
        bundled::read_packs(&dir)
    } else {
        Vec::new()
    };
    std::fs::remove_dir_all(&dir).io(IoOp::Remove, &dir)?;
    if packs.is_empty() {
        return Ok(());
    }
    let inject_root = game.inject_textures_dir();
    for name in &packs {
        let pack = inject_root.join(name);
        if pack.is_dir() {
            let _ = std::fs::remove_dir_all(&pack);
        }
    }
    prune_file_quiet(&game.texture_injection_toml(), &packs);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_mod_and_its_bundled_packs() {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path());
        let mod_dir = game.mods_dir().join("kimono");
        std::fs::create_dir_all(&mod_dir).unwrap();
        std::fs::write(
            bundled::sidecar_path(&mod_dir),
            r#"{"packs":["HD (kimono@default)"]}"#,
        )
        .unwrap();
        let pack = game.inject_textures_dir().join("HD (kimono@default)");
        let other = game.inject_textures_dir().join("Other");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(
            game.texture_injection_toml(),
            "load_order = [\"Other\", \"HD (kimono@default)\"]\ndisabled_packs = [\"HD (kimono@default)\"]\n",
        )
        .unwrap();

        uninstall(&game, "kimono", true).unwrap();

        assert!(!mod_dir.exists());
        assert!(!pack.exists());
        assert!(other.is_dir());
        let toml = std::fs::read_to_string(game.texture_injection_toml()).unwrap();
        assert!(toml.contains("load_order = [\"Other\"]"));
        assert!(toml.contains("disabled_packs = []"));
    }

    #[test]
    fn keeps_packs_unless_asked() {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path());
        let mod_dir = game.mods_dir().join("kimono");
        std::fs::create_dir_all(&mod_dir).unwrap();
        std::fs::write(bundled::sidecar_path(&mod_dir), r#"{"packs":["P"]}"#).unwrap();
        let pack = game.inject_textures_dir().join("P");
        std::fs::create_dir_all(&pack).unwrap();
        uninstall(&game, "kimono", false).unwrap();
        uninstall(&game, "missing", true).unwrap();
        assert!(pack.is_dir());
    }
}
