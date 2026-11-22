use yp_core::ArchiveFailure;
use yp_i18n::MsgKey;

pub fn archive_failure_key(failure: ArchiveFailure) -> MsgKey {
    match failure {
        ArchiveFailure::PasswordRequired => MsgKey::ArchivePasswordProtected,
        ArchiveFailure::Corrupt => MsgKey::ArchiveDamaged,
        ArchiveFailure::UnsupportedMethod => MsgKey::ArchiveUnsupportedMethod,
        ArchiveFailure::MissingVolume => MsgKey::ArchivePartMissing,
        ArchiveFailure::NoFiles => MsgKey::ArchiveHoldsNoFiles,
        ArchiveFailure::DiskFull => MsgKey::ExtractFailedDiskFull,
        ArchiveFailure::UnsupportedFormat => MsgKey::FailedToExtractArchive,
    }
}

pub fn extraction_error_key(error: &yp_core::Error) -> MsgKey {
    match error.archive_failure() {
        Some(failure) => archive_failure_key(failure),
        None if error.is_disk_full() => MsgKey::ExtractFailedDiskFull,
        None => MsgKey::FailedToExtractArchive,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use yp_i18n::{Locale, Strings};

    use super::*;

    const FAILURES: [ArchiveFailure; 7] = [
        ArchiveFailure::UnsupportedFormat,
        ArchiveFailure::UnsupportedMethod,
        ArchiveFailure::PasswordRequired,
        ArchiveFailure::Corrupt,
        ArchiveFailure::MissingVolume,
        ArchiveFailure::DiskFull,
        ArchiveFailure::NoFiles,
    ];

    #[test]
    fn every_archive_failure_has_its_own_text_in_every_language() {
        for locale in Locale::ALL {
            let strings = Strings::for_locale(locale);
            let texts: BTreeSet<&str> = FAILURES
                .into_iter()
                .map(|failure| archive_failure_key(failure).text(strings))
                .collect();
            assert_eq!(texts.len(), FAILURES.len(), "{}", locale.code());
            assert!(texts.iter().all(|text| !text.trim().is_empty()));
        }
    }

    #[test]
    fn the_new_texts_are_translated() {
        let english = Strings::for_locale(Locale::En);
        for key in [
            MsgKey::ArchivePasswordProtected,
            MsgKey::ArchiveDamaged,
            MsgKey::ArchiveUnsupportedMethod,
            MsgKey::ArchivePartMissing,
            MsgKey::ArchiveHoldsNoFiles,
        ] {
            for locale in [Locale::De, Locale::Th, Locale::Zh] {
                assert_ne!(key.text(Strings::for_locale(locale)), key.text(english), "{key:?} {}", locale.code());
            }
        }
    }

    #[test]
    fn errors_map_through_their_archive_failure() {
        let archive = Path::new("pack.rar");
        assert_eq!(
            extraction_error_key(&yp_core::Error::archive(archive, ArchiveFailure::PasswordRequired)),
            MsgKey::ArchivePasswordProtected
        );
        assert_eq!(
            extraction_error_key(&yp_core::Error::archive(archive, ArchiveFailure::DiskFull)),
            MsgKey::ExtractFailedDiskFull
        );
        let full = yp_core::Error::Io {
            op: yp_core::IoOp::Write,
            path: PathBuf::from("x"),
            source: std::io::ErrorKind::StorageFull.into(),
        };
        assert_eq!(extraction_error_key(&full), MsgKey::ExtractFailedDiskFull);
        let denied = yp_core::Error::Io {
            op: yp_core::IoOp::Create,
            path: PathBuf::from("x"),
            source: std::io::ErrorKind::PermissionDenied.into(),
        };
        assert_eq!(extraction_error_key(&denied), MsgKey::FailedToExtractArchive);
        assert_eq!(extraction_error_key(&yp_core::Error::Cancelled), MsgKey::FailedToExtractArchive);
    }
}
