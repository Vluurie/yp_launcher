use std::path::Path;

use yp_config::{ConfigError, ConfigFile};
use yp_core::toml_io::TomlError;
use yp_formats::{FileOp, FormatError};
use yp_i18n::Strings;
use yp_state::inspector::{ArchiveSide, InspectorError, ProbeWarning};
use yp_state::{ConfigFailure, ConfigOp, NamsSettingsFailure, NamsSettingsOp, archive_failure_key};

fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

pub fn toml_error_text(error: &TomlError, strings: &Strings) -> String {
    match (error.line, error.column) {
        (Some(line), Some(column)) => strings.settings_toml_error_at(
            i64::try_from(line).unwrap_or(i64::MAX),
            i64::try_from(column).unwrap_or(i64::MAX),
            &error.message,
        ),
        _ => error.message.clone(),
    }
}

pub fn core_error_text(error: &yp_core::Error, strings: &Strings) -> String {
    match error {
        yp_core::Error::Io { op, path, source } => {
            let detail = format!("{}: {source}", path.display());
            match op {
                yp_core::IoOp::Read | yp_core::IoOp::Metadata => {
                    strings.err_file_read_failed(&detail)
                }
                yp_core::IoOp::Create => strings.err_directory_create_failed(&detail),
                _ => strings.err_file_write_failed(&detail),
            }
        }
        yp_core::Error::TomlParse { path, message } => match path {
            Some(path) => strings.settings_config_not_toml(&file_label(path), message),
            None => message.clone(),
        },
        yp_core::Error::Archive { failure, .. } => archive_failure_key(*failure).text(strings).to_owned(),
        other => other.to_string(),
    }
}

pub fn config_error_text(error: &ConfigError, strings: &Strings) -> String {
    match error {
        ConfigError::Core(error) => core_error_text(error, strings),
        ConfigError::Toml { path, error } => {
            strings.settings_config_not_toml(&file_label(path), &toml_error_text(error, strings))
        }
        ConfigError::SettingsNotObject { path } => {
            strings.settings_settings_not_object(&file_label(path))
        }
    }
}

pub fn config_failure_text(failure: &ConfigFailure, strings: &Strings) -> String {
    let detail = config_error_text(&failure.error, strings);
    match failure.op {
        ConfigOp::Load | ConfigOp::Discard => strings.settings_config_load_failed(&detail),
        ConfigOp::Save | ConfigOp::WriteNow | ConfigOp::CutsceneFlags => {
            strings.settings_config_save_failed(&detail)
        }
        ConfigOp::Autosave(file) => {
            strings.settings_config_autosave_failed(file.file_name(), &detail)
        }
        ConfigOp::LodModPreset | ConfigOp::LodModReset => {
            strings.settings_config_autosave_failed(ConfigFile::LodMod.file_name(), &detail)
        }
        ConfigOp::ExternalReload => strings.settings_config_reload_failed(&detail),
    }
}

pub fn settings_failure_text(failure: &NamsSettingsFailure, strings: &Strings) -> String {
    let detail = config_error_text(&failure.error, strings);
    match failure.op {
        NamsSettingsOp::Load | NamsSettingsOp::Discard => {
            strings.settings_config_load_failed(&detail)
        }
        NamsSettingsOp::Save => strings.settings_config_save_failed(&detail),
        NamsSettingsOp::Reload => strings.settings_config_reload_failed(&detail),
    }
}

pub fn format_error_text(error: &FormatError, strings: &Strings) -> String {
    match error {
        FormatError::Io { op, path, source } => {
            let detail = format!("{}: {source}", path.display());
            match op {
                FileOp::Read => strings.err_file_read_failed(&detail),
                FileOp::Write => strings.err_file_write_failed(&detail),
                FileOp::CreateDir => strings.err_directory_create_failed(&detail),
            }
        }
        FormatError::ImageUnreadable { .. } => strings.err_image_unreadable().to_owned(),
        FormatError::ImageSizeOutOfRange { .. } => strings.err_image_size_out_of_range().to_owned(),
        FormatError::ImageSizeNotMultipleOfFour { .. } => {
            strings.err_image_size_not_multiple_of_four().to_owned()
        }
        FormatError::ArchiveEmpty => strings.err_archive_empty().to_owned(),
        FormatError::ArchiveExtensionTooLong { name } => {
            strings.err_archive_extension_too_long(name)
        }
        FormatError::WeaponNothingToRename { .. } => {
            strings.err_weapon_nothing_to_convert().to_owned()
        }
        FormatError::WeaponClassMismatch { .. } => strings.err_weapon_class_mismatch().to_owned(),
        FormatError::Model(fault) => strings.model_view_err_model_unreadable(&fault.to_string()),
        other if other.is_texture_unreadable() => {
            strings.model_view_err_archive_unreadable(&other.to_string())
        }
        other => strings.err_unknown_code(i64::from(other.code()), &other.to_string()),
    }
}

pub fn side_label(side: ArchiveSide) -> &'static str {
    match side {
        ArchiveSide::Dat => ".dat",
        ArchiveSide::Dtt => ".dtt",
    }
}

pub fn probe_warning_text(warning: &ProbeWarning, strings: &Strings) -> String {
    match warning {
        ProbeWarning::ArchiveUnreadable { path, error, .. } => strings
            .model_view_warn_file_unreadable(&file_label(path), &format_error_text(error, strings)),
        ProbeWarning::EntryUnreadable { entry, error, .. } => {
            strings.model_view_warn_file_unreadable(entry, &format_error_text(error, strings))
        }
    }
}

pub fn inspector_error_text(error: &InspectorError, strings: &Strings) -> String {
    match error {
        InspectorError::NoArchives => strings.model_view_err_no_archives().to_owned(),
        InspectorError::NoReadableArchive { warnings } => {
            let mut text = strings.model_view_err_no_readable_archive().to_owned();
            for warning in warnings {
                text.push('\n');
                text.push_str(&probe_warning_text(warning, strings));
            }
            text
        }
        InspectorError::MissingArchive(side) => {
            strings.model_view_err_missing_archive(side_label(*side))
        }
        InspectorError::EntryMissing { side, .. } => {
            strings.model_view_err_entry_missing(side_label(*side))
        }
        InspectorError::MeshMissing(id) => strings.model_view_err_mesh_missing(i64::from(*id)),
        InspectorError::LodMissing(id) => strings.model_view_err_lod_missing(i64::from(*id)),
        InspectorError::RenderTooSmall { width, height }
        | InspectorError::RenderTooLarge { width, height } => {
            strings.model_view_err_render_size(i64::from(*width), i64::from(*height))
        }
        InspectorError::Format(error) => format_error_text(error, strings),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use yp_formats::{DatFault, ModelFault};
    use yp_i18n::Locale;

    use super::*;

    fn en() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    #[test]
    fn archive_failures_use_the_translated_reason() {
        let locked = yp_core::Error::archive(Path::new("pack.rar"), yp_core::ArchiveFailure::PasswordRequired);
        assert_eq!(core_error_text(&locked, en()), en().archive_password_protected());
        let german = Strings::for_locale(Locale::De);
        assert_eq!(core_error_text(&locked, german), german.archive_password_protected());
    }

    #[test]
    fn toml_errors_carry_their_position_when_known() {
        let located = TomlError {
            message: "expected `=`".into(),
            line: Some(3),
            column: Some(7),
        };
        let text = toml_error_text(&located, en());
        assert!(text.contains('3') && text.contains('7') && text.contains("expected `=`"));
        let bare = TomlError {
            message: "broken".into(),
            line: None,
            column: None,
        };
        assert_eq!(toml_error_text(&bare, en()), "broken");
    }

    #[test]
    fn config_failures_name_the_operation_and_the_file() {
        let failure = ConfigFailure {
            op: ConfigOp::Autosave(ConfigFile::Naiom),
            error: Arc::new(ConfigError::SettingsNotObject {
                path: PathBuf::from("C:/game/nams/settings.json"),
            }),
        };
        let text = config_failure_text(&failure, en());
        assert!(text.contains("naiom.toml"));
        assert!(text.contains("settings.json"));
        let load = ConfigFailure {
            op: ConfigOp::Load,
            error: Arc::new(ConfigError::Toml {
                path: PathBuf::from("C:/game/nams/lodmod.toml"),
                error: TomlError {
                    message: "bad".into(),
                    line: Some(1),
                    column: Some(2),
                },
            }),
        };
        let text = config_failure_text(&load, en());
        assert!(text.contains("lodmod.toml") && text.contains("bad"));
    }

    #[test]
    fn every_inspector_error_maps_to_text() {
        let format = Arc::new(FormatError::Model(ModelFault::NotWmb3));
        let warnings = vec![
            ProbeWarning::ArchiveUnreadable {
                side: ArchiveSide::Dat,
                path: PathBuf::from("pl/pl000d.dat"),
                error: Arc::new(FormatError::Dat(DatFault::UnsupportedMagic)),
            },
            ProbeWarning::EntryUnreadable {
                side: ArchiveSide::Dtt,
                entry: "pl000d.wmb".into(),
                error: format.clone(),
            },
        ];
        let errors = [
            InspectorError::NoArchives,
            InspectorError::NoReadableArchive {
                warnings: warnings.clone(),
            },
            InspectorError::MissingArchive(ArchiveSide::Dtt),
            InspectorError::EntryMissing {
                side: ArchiveSide::Dat,
                index: 4,
            },
            InspectorError::MeshMissing(9),
            InspectorError::LodMissing(2),
            InspectorError::RenderTooSmall {
                width: 2,
                height: 2,
            },
            InspectorError::RenderTooLarge {
                width: 9000,
                height: 9000,
            },
            InspectorError::Format(format),
        ];
        for error in &errors {
            assert!(!inspector_error_text(error, en()).is_empty(), "{error:?}");
        }
        let unreadable = inspector_error_text(&errors[1], en());
        assert!(unreadable.contains("pl000d.dat") && unreadable.contains("pl000d.wmb"));
        assert!(inspector_error_text(&errors[2], en()).contains(".dtt"));
        for warning in &warnings {
            assert!(!probe_warning_text(warning, en()).is_empty());
        }
    }

    #[test]
    fn format_errors_reuse_the_flutter_messages_where_they_exist() {
        let strings = en();
        assert_eq!(
            format_error_text(&FormatError::ArchiveEmpty, strings),
            strings.err_archive_empty()
        );
        let io = FormatError::io(
            FileOp::Read,
            Path::new("a/b.dat"),
            std::io::Error::other("denied"),
        );
        assert!(format_error_text(&io, strings).contains("denied"));
        let unknown = FormatError::ArchiveTooManyEntries { count: 70000 };
        assert!(format_error_text(&unknown, strings).contains("1207"));
    }
}
