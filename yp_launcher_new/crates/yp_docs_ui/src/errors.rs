use yp_core::IoOp;
use yp_formats::{FileOp, FormatError};
use yp_i18n::Strings;

const UNKNOWN_CODE: i64 = 1000;

pub fn format_error_text(error: &FormatError, strings: &Strings) -> String {
    match error {
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
        FormatError::WeaponStemUnreadable { .. } => strings.err_weapon_stem_unreadable().to_owned(),
        FormatError::WeaponStemLengthMismatch { .. } => {
            strings.err_weapon_stem_length_mismatch().to_owned()
        }
        FormatError::WeaponSameStem { .. } => strings.err_weapon_same_stem().to_owned(),
        FormatError::Io { op, path, source } => {
            let detail = format!("{}: {source}", path.display());
            match op {
                FileOp::Read => strings.err_file_read_failed(&detail),
                FileOp::Write => strings.err_file_write_failed(&detail),
                FileOp::CreateDir => strings.err_directory_create_failed(&detail),
            }
        }
        other if other.is_texture_unreadable() => strings.err_texture_unreadable().to_owned(),
        other => strings.err_unknown_code(i64::from(other.code()), &other.to_string()),
    }
}

pub fn core_error_text(error: &yp_core::Error, strings: &Strings) -> String {
    match error {
        yp_core::Error::Io { op, path, source } => {
            let detail = format!("{}: {source}", path.display());
            match op {
                IoOp::Read | IoOp::List | IoOp::Metadata => strings.err_file_read_failed(&detail),
                IoOp::Create => strings.err_directory_create_failed(&detail),
                IoOp::Write | IoOp::Remove | IoOp::Rename | IoOp::Copy => {
                    strings.err_file_write_failed(&detail)
                }
            }
        }
        other => strings.err_unknown_code(UNKNOWN_CODE, &other.to_string()),
    }
}

pub fn builder_error_text(error: &yp_builders::Error, strings: &Strings) -> String {
    match error {
        yp_builders::Error::Core(error) => core_error_text(error, strings),
        yp_builders::Error::Format(error) => format_error_text(error, strings),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use yp_formats::{DatFault, WeaponClass};
    use yp_i18n::Locale;

    use super::*;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    #[test]
    fn weapon_and_image_failures_use_their_own_sentences() {
        let strings = strings();
        assert_eq!(
            format_error_text(
                &FormatError::WeaponNothingToRename {
                    stem: "wp0003".into()
                },
                strings
            ),
            strings.err_weapon_nothing_to_convert()
        );
        assert_eq!(
            format_error_text(
                &FormatError::WeaponClassMismatch {
                    from: WeaponClass::Spear,
                    to: WeaponClass::SmallSword
                },
                strings
            ),
            strings.err_weapon_class_mismatch()
        );
        assert_eq!(
            format_error_text(
                &FormatError::ImageUnreadable {
                    detail: "bad".into()
                },
                strings
            ),
            strings.err_image_unreadable()
        );
        assert_eq!(
            format_error_text(
                &FormatError::ImageSizeNotMultipleOfFour {
                    width: 5,
                    height: 4
                },
                strings
            ),
            strings.err_image_size_not_multiple_of_four()
        );
    }

    #[test]
    fn every_texture_fault_reads_as_an_unreadable_icon() {
        let strings = strings();
        for error in [
            FormatError::TextureNoWtp,
            FormatError::TextureNotDds,
            FormatError::TextureBadSize {
                width: 0,
                height: 0,
            },
            FormatError::Dat(DatFault::UnsupportedMagic),
        ] {
            assert_eq!(
                format_error_text(&error, strings),
                strings.err_texture_unreadable()
            );
        }
    }

    #[test]
    fn io_failures_name_the_path_and_unknown_ones_keep_their_code() {
        let strings = strings();
        let read = FormatError::io(
            FileOp::Read,
            Path::new("icons/a.dtt"),
            std::io::ErrorKind::NotFound.into(),
        );
        let text = format_error_text(&read, strings);
        assert!(text.contains("a.dtt"));
        assert!(text.starts_with(&strings.err_file_read_failed("")[..10]));

        let unknown = FormatError::ArchiveTooManyEntries { count: 70000 };
        let text = format_error_text(&unknown, strings);
        assert!(text.contains("1207"));
        assert!(text.contains("70000"));
    }

    #[test]
    fn core_io_failures_map_to_read_write_and_create() {
        let strings = strings();
        let error = |op| yp_core::Error::Io {
            op,
            path: "mods/x".into(),
            source: std::io::ErrorKind::PermissionDenied.into(),
        };
        assert!(core_error_text(&error(IoOp::Create), strings).contains("mods"));
        assert_ne!(
            core_error_text(&error(IoOp::Create), strings),
            core_error_text(&error(IoOp::Write), strings)
        );
        assert_ne!(
            core_error_text(&error(IoOp::Read), strings),
            core_error_text(&error(IoOp::Write), strings)
        );
        assert!(core_error_text(&yp_core::Error::Cancelled, strings).contains("1000"));
    }
}
