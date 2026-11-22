use std::path::{Path, PathBuf};

use yp_core::{Os, ProcessRunner, ProcessSpec, Result};

fn native_separators(target: &Path) -> PathBuf {
    target.components().collect()
}

pub fn reveal_spec(os: Os, target: &Path, is_file: bool) -> Option<ProcessSpec> {
    if target.as_os_str().is_empty() {
        return None;
    }
    let spec = match os {
        Os::Windows if is_file => ProcessSpec::new("explorer")
            .arg("/select,")
            .arg(native_separators(target).as_os_str()),
        Os::Windows => ProcessSpec::new("explorer").arg(native_separators(target).as_os_str()),
        Os::MacOs if is_file => ProcessSpec::new("open").arg("-R").arg(target.as_os_str()),
        Os::MacOs => ProcessSpec::new("open").arg(target.as_os_str()),
        Os::Linux => {
            let folder = if is_file { target.parent().unwrap_or(target) } else { target };
            ProcessSpec::new("xdg-open").arg(folder.as_os_str())
        }
    };
    Some(spec)
}

pub fn reveal(runner: &dyn ProcessRunner, os: Os, target: &Path, is_file: bool) -> Result<()> {
    let Some(spec) = reveal_spec(os, target, is_file) else {
        return Ok(());
    };
    runner.spawn(&spec).map(|_| ())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn explorer_gets_backslashes_for_the_paths_nams_prints() {
        let file = reveal_spec(Os::Windows, Path::new("F:/SteamLibrary/NieRAutomata/nams/nams.toml"), true).unwrap();
        assert_eq!(file.args_lossy(), ["/select,", r"F:\SteamLibrary\NieRAutomata\nams\nams.toml"]);
        let folder = reveal_spec(Os::Windows, Path::new("F:/SteamLibrary/NieRAutomata/nams"), false).unwrap();
        assert_eq!(folder.args_lossy(), [r"F:\SteamLibrary\NieRAutomata\nams"]);
    }
}
