use std::path::PathBuf;

use futures::StreamExt;
use futures::channel::mpsc;
use gpui::{AsyncWindowContext, Entity, Task};
use yp_core::{CancelToken, Progress};
use yp_cutscenes::install::validate_mod_name;
use yp_cutscenes::{
    CopyProgress, CutsceneMod, InstallContext, InstallOutcome, MAX_MOD_NAME_LENGTH, NameChoice,
    NameRejection, PrepareFailure,
};
use yp_state::extraction_error_key;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    ConfirmOptions, ConfirmTone, Msg, PickItem, PickOptions, PromptOptions, Strings, confirm,
    pick_one, prompt_text,
};

use crate::cutscenes::model::{CutscenesModel, merge_targets};
use crate::notify::brief_toast;

fn max_name_length() -> i64 {
    i64::try_from(MAX_MOD_NAME_LENGTH).unwrap_or(i64::MAX)
}

pub fn prepare_failure_notice(failure: &PrepareFailure) -> Option<(Msg, Severity)> {
    if failure.is_cancelled() {
        return None;
    }
    Some(match failure {
        PrepareFailure::NoUsm => (Msg::CutsceneDropNoUsm, Severity::Error),
        PrepareFailure::Extract {
            disk_full: true, ..
        } => (Msg::ExtractFailedDiskFull, Severity::Error),
        PrepareFailure::Extract { error, .. } | PrepareFailure::Failed(error) => {
            (Msg::from(extraction_error_key(error)), Severity::Error)
        }
        PrepareFailure::NoMovieFolder => (Msg::NoMovieFolderFound, Severity::Warning),
    })
}

pub fn name_rejection_notice(rejection: NameRejection) -> Option<Msg> {
    match rejection {
        NameRejection::Empty => None,
        NameRejection::TooLong { max } => Some(Msg::CutsceneNameTooLong {
            max: i64::try_from(max).unwrap_or(i64::MAX),
        }),
        NameRejection::Invalid => Some(Msg::ModsUiCutsceneNameInvalid),
    }
}

pub fn install_outcome_notice(outcome: &InstallOutcome) -> (Msg, Severity) {
    match outcome {
        InstallOutcome::Installed { name, files, .. } => (
            Msg::InstalledCutsceneMod {
                name: name.clone(),
                count: i64::try_from(*files).unwrap_or(i64::MAX),
            },
            Severity::Success,
        ),
        InstallOutcome::NoUsmFiles => (Msg::NoUsmFilesFound, Severity::Error),
    }
}

pub fn install_error_notice(error: &yp_core::Error) -> Option<Msg> {
    if error.is_cancelled() {
        None
    } else if yp_cutscenes::install::is_disk_full(error) {
        Some(Msg::ExtractFailedDiskFull)
    } else {
        Some(Msg::ModsUiCutsceneWriteFailed)
    }
}

pub fn copy_message(progress: &CopyProgress, strings: &Strings) -> String {
    let current = i64::try_from(progress.file_index).unwrap_or(i64::MAX);
    let total = i64::try_from(progress.file_count).unwrap_or(i64::MAX);
    if progress.megabytes_done() == 0 {
        strings.cutscene_copying_file(current, total, &progress.file_name)
    } else {
        strings.cutscene_copying_file_bytes(
            current,
            total,
            &progress.file_name,
            &progress.megabytes_done().to_string(),
            &progress.megabytes_total().to_string(),
        )
    }
}

fn extract_message(progress: &Progress, strings: &Strings) -> Option<(String, f32)> {
    let fraction = progress.fraction?;
    let percent = (fraction * 100.0).round() as i64;
    let text = match progress.current.as_deref().filter(|file| !file.is_empty()) {
        Some(file) => strings.extracting_archive_percent_file(percent, file),
        None => strings.extracting_archive_percent(percent),
    };
    Some((text, fraction * 100.0))
}

fn name_validation_error(text: &str, strings: &'static Strings) -> Result<(), SharedString> {
    match validate_mod_name(text) {
        Ok(_) | Err(NameRejection::Empty) => Ok(()),
        Err(NameRejection::TooLong { max }) => Err(strings
            .cutscene_name_too_long(i64::try_from(max).unwrap_or(i64::MAX))
            .into()),
        Err(NameRejection::Invalid) => Err(SharedString::new_static(
            strings.mods_ui_cutscene_name_invalid(),
        )),
    }
}

struct Session {
    model: Entity<CutscenesModel>,
}

impl Session {
    fn activity(
        &self,
        text: impl Into<SharedString>,
        percent: Option<f32>,
        cx: &mut AsyncWindowContext,
    ) {
        let text = text.into();
        self.model
            .update(cx, |model, cx| model.set_activity(text, percent, cx));
    }

    fn done(&self, cx: &mut AsyncWindowContext) {
        self.model
            .update(cx, |model, cx| model.clear_activity(cx));
    }

    fn notify(&self, msg: Msg, severity: Severity, cx: &mut AsyncWindowContext) {
        cx.update(|_, cx| brief_toast(cx, "cutscenes", msg, severity))
            .ok();
    }

    fn refresh(&self, cx: &mut AsyncWindowContext) {
        cx.update(|_, cx| {
            let detection = cx.stores().detection.clone();
            detection.update(cx, |detection, cx| detection.invalidate(cx));
        })
        .ok();
    }

    async fn choose_name(
        &self,
        suggested: String,
        cx: &mut AsyncWindowContext,
    ) -> Option<NameChoice> {
        let targets = self
            .model
            .read_with(cx, |model, _| merge_targets(model.mods()));
        if !targets.is_empty() {
            let picking = cx
                .update(|window, cx| {
                    let strings = cx.t();
                    let mut items: Vec<PickItem<Option<String>>> = targets
                        .into_iter()
                        .map(|name| {
                            PickItem::new(strings.cutscene_merge_add_to(&name), Some(name))
                                .icon(IconName::MergeType)
                        })
                        .collect();
                    items.push(
                        PickItem::new(strings.cutscene_merge_new_mod(), None).icon(IconName::Add),
                    );
                    pick_one(
                        PickOptions::new(strings.cutscene_merge_title(), items)
                            .body(strings.cutscene_merge_description())
                            .cancel_label(strings.cancel_button()),
                        window,
                        cx,
                    )
                })
                .ok()?;
            if let Some(existing) = picking.await? {
                return Some(NameChoice::MergeInto(existing));
            }
        }
        let asking = cx
            .update(|window, cx| {
                let strings = cx.t();
                prompt_text(
                    PromptOptions::new(strings.cutscene_naming_title())
                        .label(strings.cutscene_naming_hint(max_name_length()))
                        .initial(suggested)
                        .max_len(MAX_MOD_NAME_LENGTH)
                        .confirm_label(strings.button_install())
                        .validate(move |text| name_validation_error(text, strings)),
                    window,
                    cx,
                )
            })
            .ok()?;
        asking.await.map(NameChoice::New)
    }

    async fn install(&self, source: PathBuf, cx: &mut AsyncWindowContext) {
        let Ok((strings, env, game)) = cx.update(|_, cx| {
            (
                cx.t(),
                cx.env().clone(),
                cx.stores().session.read(cx).game(),
            )
        }) else {
            return;
        };
        let Some(game) = game else {
            return;
        };
        self.activity(strings.cutscene_drop_analyzing(), None, cx);

        let (sender, mut updates) = mpsc::unbounded::<Progress>();
        let model = self.model.downgrade();
        let pump: Task<()> = cx.spawn(async move |cx| {
            let mut announced = false;
            while let Some(progress) = updates.next().await {
                cx.update(|_, cx| {
                    let strings = cx.t();
                    let Some((text, percent)) = extract_message(&progress, strings) else {
                        return;
                    };
                    if !announced {
                        announced = true;
                        brief_toast(cx, "cutscenes", Msg::ExtractingArchive, Severity::Accent);
                    }
                    model
                        .update(cx, |model, cx| model.set_activity(text, Some(percent), cx))
                        .ok();
                })
                .ok();
            }
        });
        let prepare_env = env.clone();
        let prepare_game = game.clone();
        let prepare_source = source.clone();
        let prepared = cx
            .background_executor()
            .spawn(async move {
                let context = InstallContext {
                    game: &prepare_game,
                    host: &prepare_env.host,
                    extractor: &prepare_env.extractor,
                };
                let relay = move |progress: Progress| {
                    let _ = sender.unbounded_send(progress);
                };
                yp_cutscenes::prepare(&prepare_source, &context, &relay, &CancelToken::new())
            })
            .await;
        drop(pump);
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(failure) => {
                self.done(cx);
                if let Some((msg, severity)) = prepare_failure_notice(&failure) {
                    self.notify(msg, severity, cx);
                }
                return;
            }
        };

        self.activity(strings.cutscene_scanning_archive(), None, cx);
        let Some(choice) = self.choose_name(prepared.suggested_name(), cx).await else {
            self.done(cx);
            return;
        };
        let name = match choice.resolve() {
            Ok(name) => name,
            Err(rejection) => {
                self.done(cx);
                if let Some(msg) = name_rejection_notice(rejection) {
                    self.notify(msg, Severity::Warning, cx);
                }
                return;
            }
        };

        self.activity(strings.preparing_install(), None, cx);
        let (sender, mut updates) = mpsc::unbounded::<CopyProgress>();
        let model = self.model.downgrade();
        let pump: Task<()> = cx.spawn(async move |cx| {
            while let Some(progress) = updates.next().await {
                cx.update(|_, cx| {
                    let text = copy_message(&progress, cx.t());
                    let percent = progress.fraction() * 100.0;
                    model
                        .update(cx, |model, cx| model.set_activity(text, Some(percent), cx))
                        .ok();
                })
                .ok();
            }
        });
        let cutscenes_dir = game.cutscenes_dir();
        let outcome = cx
            .background_executor()
            .spawn(async move {
                let observe = move |progress: &CopyProgress| {
                    let _ = sender.unbounded_send(progress.clone());
                };
                yp_cutscenes::install(
                    &prepared,
                    &cutscenes_dir,
                    &name,
                    &observe,
                    &CancelToken::new(),
                )
            })
            .await;
        drop(pump);
        self.done(cx);

        match outcome {
            Ok(outcome) => {
                let installed = matches!(outcome, InstallOutcome::Installed { .. });
                let (msg, severity) = install_outcome_notice(&outcome);
                self.notify(msg, severity, cx);
                if installed {
                    self.refresh(cx);
                }
            }
            Err(error) => {
                if let Some(msg) = install_error_notice(&error) {
                    self.notify(msg, Severity::Error, cx);
                }
            }
        }
    }

    async fn delete(&self, cutscene: CutsceneMod, cx: &mut AsyncWindowContext) {
        let Ok(confirming) = cx.update(|window, cx| {
            let strings = cx.t();
            confirm(
                ConfirmOptions::new(
                    strings.remove_cutscene_mod_title(),
                    strings.delete_cutscene_confirm(&cutscene.name),
                )
                .confirm_label(strings.button_delete())
                .cancel_label(strings.button_cancel())
                .tone(ConfirmTone::Danger),
                window,
                cx,
            )
        }) else {
            return;
        };
        if !confirming.await {
            return;
        }
        if let Ok(strings) = cx.update(|_, cx| cx.t()) {
            self.activity(strings.busy_deleting_cutscene(), None, cx);
        }
        let path = cutscene.path.clone();
        let deleted = cx
            .background_executor()
            .spawn(async move { yp_cutscenes::delete_mod(&path) })
            .await;
        self.done(cx);
        match deleted {
            Ok(_) => {
                self.notify(
                    Msg::RemovedItem {
                        name: cutscene.name,
                    },
                    Severity::Info,
                    cx,
                );
                self.refresh(cx);
            }
            Err(error) => {
                log::warn!("could not delete {}: {error}", cutscene.path.display());
                self.notify(Msg::ModsUiCutsceneWriteFailed, Severity::Error, cx);
            }
        }
    }
}

pub fn install_from_path(
    model: Entity<CutscenesModel>,
    source: PathBuf,
    window: &mut Window,
    cx: &mut App,
) -> Task<()> {
    window.spawn(cx, async move |cx| {
        Session { model }.install(source, cx).await;
    })
}

pub fn delete_cutscene(
    model: Entity<CutscenesModel>,
    cutscene: CutsceneMod,
    window: &mut Window,
    cx: &mut App,
) -> Task<()> {
    window.spawn(cx, async move |cx| {
        Session { model }.delete(cutscene, cx).await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use yp_ui::Locale;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    fn io_error() -> yp_core::Error {
        yp_core::Error::Io {
            op: yp_core::IoOp::Copy,
            path: PathBuf::from("x"),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        }
    }

    #[test]
    fn every_prepare_failure_maps_to_its_own_message() {
        assert_eq!(
            prepare_failure_notice(&PrepareFailure::NoUsm),
            Some((Msg::CutsceneDropNoUsm, Severity::Error))
        );
        for (failure, msg) in [
            (yp_core::ArchiveFailure::PasswordRequired, Msg::ArchivePasswordProtected),
            (yp_core::ArchiveFailure::Corrupt, Msg::ArchiveDamaged),
            (yp_core::ArchiveFailure::UnsupportedMethod, Msg::ArchiveUnsupportedMethod),
            (yp_core::ArchiveFailure::MissingVolume, Msg::ArchivePartMissing),
            (yp_core::ArchiveFailure::NoFiles, Msg::ArchiveHoldsNoFiles),
            (yp_core::ArchiveFailure::DiskFull, Msg::ExtractFailedDiskFull),
        ] {
            let error = yp_core::Error::archive(std::path::Path::new("pack.7z"), failure);
            assert_eq!(
                prepare_failure_notice(&PrepareFailure::Extract {
                    disk_full: error.is_disk_full(),
                    error
                }),
                Some((msg, Severity::Error))
            );
        }
        assert_eq!(
            prepare_failure_notice(&PrepareFailure::NoMovieFolder),
            Some((Msg::NoMovieFolderFound, Severity::Warning))
        );
        assert_eq!(
            prepare_failure_notice(&PrepareFailure::Extract {
                disk_full: true,
                error: io_error()
            }),
            Some((Msg::ExtractFailedDiskFull, Severity::Error))
        );
        assert_eq!(
            prepare_failure_notice(&PrepareFailure::Extract {
                disk_full: false,
                error: io_error()
            }),
            Some((Msg::FailedToExtractArchive, Severity::Error))
        );
        assert_eq!(
            prepare_failure_notice(&PrepareFailure::Failed(yp_core::Error::Cancelled)),
            None
        );
    }

    #[test]
    fn name_rejections_stay_silent_only_for_an_empty_name() {
        assert_eq!(name_rejection_notice(NameRejection::Empty), None);
        assert_eq!(
            name_rejection_notice(NameRejection::TooLong { max: 27 }),
            Some(Msg::CutsceneNameTooLong { max: 27 })
        );
        assert_eq!(
            name_rejection_notice(NameRejection::Invalid),
            Some(Msg::ModsUiCutsceneNameInvalid)
        );
        assert!(name_validation_error("", strings()).is_ok());
        assert!(name_validation_error("HD Cutscenes", strings()).is_ok());
        assert!(name_validation_error("a/b", strings()).is_err());
        assert!(name_validation_error(&"x".repeat(28), strings()).is_err());
    }

    #[test]
    fn install_outcomes_report_the_file_count_or_the_missing_files() {
        let (msg, severity) = install_outcome_notice(&InstallOutcome::Installed {
            name: "HD".to_owned(),
            target: PathBuf::from("HD"),
            files: 150,
        });
        assert_eq!(severity, Severity::Success);
        assert_eq!(
            msg,
            Msg::InstalledCutsceneMod {
                name: "HD".to_owned(),
                count: 150
            }
        );
        assert_eq!(
            install_outcome_notice(&InstallOutcome::NoUsmFiles),
            (Msg::NoUsmFilesFound, Severity::Error)
        );
        assert_eq!(install_error_notice(&yp_core::Error::Cancelled), None);
        assert_eq!(
            install_error_notice(&io_error()),
            Some(Msg::ModsUiCutsceneWriteFailed)
        );
    }

    #[test]
    fn copy_progress_adds_megabytes_once_some_were_copied() {
        let strings = strings();
        let mut progress = CopyProgress {
            file_index: 2,
            file_count: 5,
            file_name: "ev0010.usm".to_owned(),
            bytes_done: 0,
            bytes_total: 300 * 1024 * 1024,
        };
        assert_eq!(
            copy_message(&progress, strings),
            strings.cutscene_copying_file(2, 5, "ev0010.usm")
        );
        progress.bytes_done = 120 * 1024 * 1024;
        assert_eq!(
            copy_message(&progress, strings),
            strings.cutscene_copying_file_bytes(2, 5, "ev0010.usm", "120", "300")
        );
    }
}
