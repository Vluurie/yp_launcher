use std::path::Path;

use yp_core::{Os, ProcessRunner, ProcessSpec, Result};

pub fn reveal_spec(os: Os, target: &Path, is_file: bool) -> Option<ProcessSpec> {
    if target.as_os_str().is_empty() {
        return None;
    }
    let spec = match os {
        Os::Windows if is_file => ProcessSpec::new("explorer").arg("/select,").arg(target.as_os_str()),
        Os::Windows => ProcessSpec::new("explorer").arg(target.as_os_str()),
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
