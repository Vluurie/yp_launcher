use std::path::Path;

use yp_thirdparty::DllSummary;
use yp_ui::Strings;

pub fn dll_label(summary: &DllSummary, strings: &Strings) -> String {
    let size = summary.size_bytes.map(yp_core::fs::size_label);
    match (&summary.version, size) {
        (Some(version), Some(size)) => strings.mods_ui_dot_join(version, &size),
        (Some(version), None) => version.clone(),
        (None, Some(size)) => size,
        (None, None) => summary.file_name.clone(),
    }
}

pub fn seconds_label(milliseconds: u64) -> String {
    if milliseconds.is_multiple_of(1000) {
        (milliseconds / 1000).to_string()
    } else {
        format!("{:.1}", milliseconds as f64 / 1000.0)
    }
}

pub fn file_stem_of(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_ui::Locale;

    #[test]
    fn dll_labels_prefer_version_then_size_then_name() {
        let strings = Strings::for_locale(Locale::En);
        let mut summary = DllSummary {
            file_name: "dxgi.dll".to_owned(),
            version: Some("6.3.0".to_owned()),
            size_bytes: Some(3 * 1024 * 1024),
        };
        assert_eq!(dll_label(&summary, strings), "6.3.0 \u{b7} 3.0 MB");
        summary.version = None;
        assert_eq!(dll_label(&summary, strings), "3.0 MB");
        summary.size_bytes = None;
        assert_eq!(dll_label(&summary, strings), "dxgi.dll");
    }

    #[test]
    fn timeouts_show_whole_seconds_without_a_fraction() {
        assert_eq!(seconds_label(10_000), "10");
        assert_eq!(seconds_label(2_500), "2.5");
    }
}
