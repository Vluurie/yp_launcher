use std::collections::HashSet;
use std::path::PathBuf;

use futures::StreamExt;
use futures::channel::oneshot;
use gpui::{AsyncWindowContext, Task, WeakEntity};
use yp_core::{CancelToken, Progress};
use yp_mods::drop_gate::{self, DropGate};
use yp_mods::naming::prettify_mod_id;
use yp_mods::{
    DetectedDrop, DropRejection, DropStaging, InstallFailure, InstallRequest, InstallResult,
    InstalledMod, ModKind, variants,
};
use yp_state::prefs::OUTFIT_GUIDE_SHOWN;
use yp_state::prelude::*;
use yp_state::{
    BusyHandle, ModInstallMode, ModInstallReport, ModInstallRequest, ProgressReceiver, Stores,
    TexturePackOutcome, archive_failure_key, progress_channel,
};
use yp_ui::prelude::*;
use yp_ui::{Msg, Strings, WindowExt};

use crate::labels::{drop_rejection_reason, install_failure_reason};
use crate::mods::dialogs::{BusyDialog, NamingRequest, OutfitGuideDialog, prompt_mod_name};
use crate::mods::variant::pick_variants;
use crate::notify::toast;
use crate::text::{file_name_of, file_stem_of};

const OPEN_OVERLAY_KEY_ID: &str = "plugins.yorha_protocol.keybinds.main.yorha_protocol";

const PLAYER_DIR: &str = "pl";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectStep {
    Variants,
    Rejected(DropRejection),
    Single {
        needs_texture_choice: bool,
        manifest_name: Option<String>,
    },
}

pub fn detect_step(detected: &DetectedDrop) -> DetectStep {
    if detected.has_variants() {
        return DetectStep::Variants;
    }
    if detected.kind == ModKind::Unknown {
        return DetectStep::Rejected(detected.rejection.unwrap_or(DropRejection::UnknownDrop));
    }
    let manifest_name = detected
        .manifest
        .as_ref()
        .and_then(|manifest| manifest.id.as_deref())
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_owned);
    DetectStep::Single {
        needs_texture_choice: detected.has_texture_variants(),
        manifest_name,
    }
}

pub fn gate_notice(gate: DropGate) -> Option<Msg> {
    match gate {
        DropGate::Mod => None,
        DropGate::MisroutedCutscenes => Some(Msg::ModDropMisroutedCutscenes),
        DropGate::NotAMod => Some(Msg::ModDropNotAMod),
        DropGate::UnreadableArchive(failure) => Some(Msg::from(archive_failure_key(failure))),
    }
}

pub fn gate_rejection(gate: DropGate) -> DropRejection {
    match gate {
        DropGate::UnreadableArchive(failure) => DropRejection::ArchiveExtractFailed(failure),
        DropGate::Mod | DropGate::MisroutedCutscenes | DropGate::NotAMod => DropRejection::UnknownDrop,
    }
}

pub fn rejection_notice(rejection: DropRejection, strings: &Strings) -> (Msg, Severity) {
    match rejection {
        DropRejection::TextureOnly => (Msg::ModInstallReasonTextureOnly, Severity::Warning),
        other => (
            Msg::ModInstallFailed {
                reason: drop_rejection_reason(other, strings).to_owned(),
            },
            Severity::Error,
        ),
    }
}

pub fn failure_notice(failure: &InstallFailure, strings: &Strings) -> (Msg, Severity) {
    match failure {
        InstallFailure::Rejected(rejection) => rejection_notice(*rejection, strings),
        other => (
            Msg::ModInstallFailed {
                reason: install_failure_reason(other, strings),
            },
            Severity::Error,
        ),
    }
}

pub fn extract_message(progress: &Progress, strings: &Strings) -> Option<String> {
    let percent = (progress.fraction? * 100.0).round() as i64;
    Some(
        match progress.current.as_deref().filter(|file| !file.is_empty()) {
            Some(file) => strings.extracting_archive_percent_file(percent, file),
            None => strings.extracting_archive_percent(percent),
        },
    )
}

pub fn is_outfit_mod(installed: &InstalledMod) -> bool {
    installed.data.as_ref().is_some_and(|data| {
        !data.players.is_empty()
            || data
                .entries
                .iter()
                .any(|entry| entry.dir_name == PLAYER_DIR)
    })
}

pub fn first_new_outfit<'a>(
    before: &HashSet<String>,
    mods: &'a [InstalledMod],
) -> Option<&'a InstalledMod> {
    mods.iter()
        .find(|installed| !before.contains(&installed.id) && is_outfit_mod(installed))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariantSummary {
    pub installed: Vec<String>,
    pub failures: Vec<(usize, String)>,
}

pub fn summarize_variants(
    labels: &[String],
    results: &[InstallResult],
    strings: &Strings,
) -> VariantSummary {
    let mut summary = VariantSummary::default();
    for (index, result) in results.iter().enumerate() {
        match result {
            Ok(installed) => summary.installed.push(installed.id.clone()),
            Err(failure) => {
                let label = labels.get(index).map(String::as_str).unwrap_or_default();
                let reason = install_failure_reason(failure, strings);
                summary
                    .failures
                    .push((index, strings.mods_ui_label_value(label, &reason)));
            }
        }
    }
    summary
}

pub fn bulk_done_severity(installed: usize, total: usize) -> Severity {
    if installed == total {
        Severity::Success
    } else {
        Severity::Warning
    }
}

fn failure_from_core(error: yp_core::Error) -> InstallFailure {
    match error {
        yp_core::Error::Cancelled => InstallFailure::Cancelled,
        yp_core::Error::Io { op, path, source } => InstallFailure::Io {
            op,
            path,
            kind: source.kind(),
        },
        _ => InstallFailure::DataEmpty,
    }
}

fn count(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[derive(Default)]
pub(crate) struct Busy {
    dialog: Option<WeakEntity<BusyDialog>>,
    owner_alive: Option<oneshot::Sender<()>>,
    handle: Option<BusyHandle>,
}

impl Busy {
    pub(crate) fn show(&mut self, message: impl Into<SharedString>, cx: &mut AsyncWindowContext) {
        let message = message.into();
        let existing = self.dialog.as_ref().and_then(WeakEntity::upgrade);
        cx.update(|window, cx| match existing {
            Some(dialog) => dialog.update(cx, |dialog, cx| dialog.set_message(message, cx)),
            None => {
                let (owner, owner_alive) = oneshot::channel();
                let dialog =
                    window.open_modal(cx, |_, cx| BusyDialog::new(message, owner_alive, cx));
                self.dialog = dialog.map(|dialog| dialog.downgrade());
                self.owner_alive = Some(owner);
                let tracker = cx.stores().busy.clone();
                self.handle =
                    Some(tracker.update(cx, |tracker, cx| tracker.enter(BusyScope::General, cx)));
            }
        })
        .ok();
    }

    pub(crate) fn hide(&mut self, cx: &mut AsyncWindowContext) {
        self.owner_alive = None;
        self.handle = None;
        if let Some(dialog) = self.dialog.take().and_then(|dialog| dialog.upgrade()) {
            cx.update(|_, cx| dialog.update(cx, |dialog, cx| dialog.finish(cx)))
                .ok();
        }
    }

    fn pump(
        &self,
        mut updates: ProgressReceiver,
        format: impl Fn(&Progress, &Strings) -> Option<String> + 'static,
        cx: &AsyncWindowContext,
    ) -> Task<()> {
        let dialog = self.dialog.clone();
        cx.spawn(async move |cx| {
            while let Some(progress) = updates.next().await {
                let Some(dialog) = dialog.as_ref().and_then(WeakEntity::upgrade) else {
                    continue;
                };
                cx.update(|_, cx| {
                    if let Some(message) = format(&progress, cx.t()) {
                        dialog.update(cx, |dialog, cx| dialog.set_message(message, cx));
                    }
                })
                .ok();
            }
        })
    }
}

struct Flow {
    stores: Stores,
    env: AppEnv,
    busy: Busy,
    report: ModInstallReport,
    texture_packs: bool,
}

fn strings(cx: &mut AsyncWindowContext) -> Option<&'static Strings> {
    cx.update(|_, cx| cx.t()).ok()
}

fn notify(
    cx: &mut AsyncWindowContext,
    topic: &'static str,
    msg: impl Into<Msg>,
    severity: Severity,
) {
    let msg = msg.into();
    cx.update(|_, cx| toast(cx, topic, msg, severity)).ok();
}

impl Flow {
    fn new(stores: Stores, env: AppEnv) -> Self {
        Self {
            stores,
            env,
            busy: Busy::default(),
            report: ModInstallReport::default(),
            texture_packs: false,
        }
    }

    fn mod_ids(&self, cx: &mut AsyncWindowContext) -> HashSet<String> {
        let mods = self.stores.mods.clone();
        cx.update(|_, cx| {
            mods.read(cx)
                .mods()
                .iter()
                .map(|installed| installed.id.clone())
                .collect()
        })
        .unwrap_or_default()
    }

    fn fail(&mut self, source: PathBuf, failure: InstallFailure, cx: &mut AsyncWindowContext) {
        if let Some(strings) = strings(cx) {
            let (msg, severity) = failure_notice(&failure, strings);
            notify(cx, "mod_install_fail", msg, severity);
        }
        self.report.failed.push((source, failure));
    }

    async fn install_texture_pack(&mut self, source: PathBuf, cx: &mut AsyncWindowContext) {
        let sources = vec![source.clone()];
        let installing = cx
            .update(|window, cx| Registry::install_texture_pack(sources, window, cx))
            .ok()
            .flatten();
        let Some(installing) = installing else {
            self.fail(source, DropRejection::TextureOnly.into(), cx);
            return;
        };
        match installing.await {
            TexturePackOutcome::Installed => {
                let name = if yp_core::archive::is_archive(&source) {
                    file_stem_of(&source)
                } else {
                    file_name_of(&source)
                };
                self.report.texture_packs.push(name);
            }
            TexturePackOutcome::Cancelled => self.report.cancelled = true,
            TexturePackOutcome::Failed => self
                .report
                .failed
                .push((source, DropRejection::TextureOnly.into())),
        }
    }

    async fn guided(&mut self, sources: Vec<PathBuf>, cx: &mut AsyncWindowContext) {
        let split = cx
            .background_executor()
            .spawn(async move { yp_mods::loose::split_drop(&sources) })
            .await;
        if !split.loose_files.is_empty() {
            self.install_loose_files(split.loose_files, cx).await;
        }
        for source in split.rest {
            self.install_from_path(source, cx).await;
        }
    }

    async fn install_loose_files(&mut self, files: Vec<PathBuf>, cx: &mut AsyncWindowContext) {
        let Some(strings) = strings(cx) else {
            return;
        };
        self.busy.show(strings.mod_drop_analyzing(), cx);
        let env = self.env.clone();
        let staged_files = files.clone();
        let staged = cx
            .background_executor()
            .spawn(async move { yp_mods::loose::stage_files(&env.staging(), &staged_files) })
            .await;
        match staged {
            Ok(Some(staged)) => {
                self.install_from_path(staged.source().to_path_buf(), cx)
                    .await;
                drop(staged);
            }
            Ok(None) | Err(_) => {
                self.busy.hide(cx);
                notify(cx, "mod_drop_reject", Msg::ModDropNotAMod, Severity::Error);
                for file in files {
                    self.report
                        .failed
                        .push((file, DropRejection::UnknownDrop.into()));
                }
            }
        }
    }

    async fn install_from_path(&mut self, source: PathBuf, cx: &mut AsyncWindowContext) {
        let Some(strings) = strings(cx) else {
            return;
        };
        self.busy.show(strings.mod_drop_analyzing(), cx);
        let env = self.env.clone();
        let gate_source = source.clone();
        let gate = cx
            .background_executor()
            .spawn(async move { drop_gate::classify(&env.staging(), &gate_source) })
            .await;
        if let Some(msg) = gate_notice(gate) {
            self.busy.hide(cx);
            notify(cx, "mod_drop_reject", msg, Severity::Error);
            self.report
                .failed
                .push((source, gate_rejection(gate).into()));
            return;
        }

        self.busy.show(strings.mod_inspect_busy(), cx);
        let (sender, updates) = progress_channel();
        let pump = self.busy.pump(updates, extract_message, cx);
        let env = self.env.clone();
        let detect_source = source.clone();
        let detected = cx
            .background_executor()
            .spawn(async move {
                let relay = move |progress: Progress| {
                    let _ = sender.unbounded_send(progress);
                };
                yp_mods::detect::detect_drop(
                    &env.staging(),
                    &detect_source,
                    &relay,
                    &CancelToken::new(),
                )
            })
            .await;
        drop(pump);
        self.busy.hide(cx);
        let detected = match detected {
            Ok(detected) => detected,
            Err(error) => {
                self.fail(source, InstallFailure::extraction(error), cx);
                return;
            }
        };

        match detect_step(&detected) {
            DetectStep::Variants => self.install_variants(source, detected, cx).await,
            DetectStep::Rejected(DropRejection::TextureOnly) if self.texture_packs => {
                self.install_texture_pack(source, cx).await
            }
            DetectStep::Rejected(rejection) => self.fail(source, rejection.into(), cx),
            DetectStep::Single {
                needs_texture_choice,
                manifest_name,
            } => {
                self.install_single(source, detected, needs_texture_choice, manifest_name, cx)
                    .await
            }
        }
    }

    async fn install_single(
        &mut self,
        source: PathBuf,
        detected: DetectedDrop,
        needs_texture_choice: bool,
        manifest_name: Option<String>,
        cx: &mut AsyncWindowContext,
    ) {
        let Some(strings) = strings(cx) else {
            return;
        };
        let mut texture_sub_paths = Vec::new();
        if needs_texture_choice {
            let choices = variants::texture_set_choices(&detected);
            let picked = match cx.update(|window, cx| pick_variants(choices, window, cx)) {
                Ok(picking) => picking.await,
                Err(_) => None,
            };
            match picked {
                Some(chosen) if !chosen.is_empty() => {
                    texture_sub_paths = chosen.into_iter().map(|choice| choice.sub_path).collect();
                }
                _ => {
                    self.report.cancelled = true;
                    return;
                }
            }
        }

        let mut name = match manifest_name {
            Some(name) => name,
            None => {
                let request = NamingRequest::new(prettify_mod_id(&detected.suggested_id));
                match self.ask_name(request, cx).await {
                    Some(name) => name,
                    None => return,
                }
            }
        };

        let result = loop {
            self.busy.show(strings.mod_install_busy(), cx);
            let (sender, updates) = progress_channel();
            let pump = self.busy.pump(updates, extract_message, cx);
            let request = InstallRequest {
                requested_name: Some(name.clone()),
                variant_sub_path: None,
                texture_pack_sub_paths: texture_sub_paths.clone(),
            };
            let mods = self.stores.mods.clone();
            let install_source = source.clone();
            let installing = cx.update(|_, cx| {
                mods.update(cx, |mods, cx| {
                    mods.install(install_source, request, Some(sender), cx)
                })
            });
            let result = match installing {
                Ok(installing) => installing.await,
                Err(_) => Err(InstallFailure::Cancelled),
            };
            drop(pump);
            match result {
                Err(InstallFailure::Exists { id }) => {
                    self.busy.hide(cx);
                    let request = NamingRequest::new(name.clone()).exists(id);
                    match self.ask_name(request, cx).await {
                        Some(next) => name = next,
                        None => return,
                    }
                }
                other => break other,
            }
        };
        self.busy.hide(cx);

        match result {
            Ok(installed) => {
                notify(
                    cx,
                    "mod_installed",
                    Msg::ModInstalled {
                        id: installed.id.clone(),
                    },
                    Severity::Success,
                );
                if !installed.side_installed_texture_packs.is_empty() {
                    notify(
                        cx,
                        "mod_side_textures",
                        Msg::ModSideInstalledTextures {
                            names: installed.side_installed_texture_packs.join(", "),
                        },
                        Severity::Accent,
                    );
                }
                if !installed.unpaired_warnings.is_empty() {
                    notify(
                        cx,
                        "mod_unpaired",
                        Msg::ModLooseUnpairedWarning {
                            names: installed.unpaired_warnings.join(", "),
                        },
                        Severity::Warning,
                    );
                }
                self.report.installed.push(installed.id);
            }
            Err(failure) => self.fail(source, failure, cx),
        }
    }

    async fn ask_name(
        &mut self,
        request: NamingRequest,
        cx: &mut AsyncWindowContext,
    ) -> Option<String> {
        let asked = match cx.update(|window, cx| prompt_mod_name(request, window, cx)) {
            Ok(asking) => asking.await,
            Err(_) => None,
        };
        if asked.is_none() {
            self.report.cancelled = true;
        }
        asked
    }

    async fn install_variants(
        &mut self,
        source: PathBuf,
        detected: DetectedDrop,
        cx: &mut AsyncWindowContext,
    ) {
        let Some(strings) = strings(cx) else {
            return;
        };
        let offered = detected.variants.clone();
        let picked = match cx.update(|window, cx| pick_variants(offered, window, cx)) {
            Ok(picking) => picking.await,
            Err(_) => None,
        };
        let chosen = match picked {
            Some(chosen) if !chosen.is_empty() => chosen,
            _ => {
                self.report.cancelled = true;
                return;
            }
        };
        let request = NamingRequest::new(prettify_mod_id(&detected.suggested_id));
        let Some(base_name) = self.ask_name(request, cx).await else {
            return;
        };
        let requests = variants::install_requests(&base_name, &chosen, &detected);

        self.busy.show(strings.mod_install_busy(), cx);
        let (sender, updates) = progress_channel();
        let pump = self.busy.pump(updates, extract_message, cx);
        let mods = self.stores.mods.clone();
        let install_source = source.clone();
        let installing = cx.update(|_, cx| {
            mods.update(cx, |mods, cx| {
                mods.install_variants(install_source, requests, Some(sender), cx)
            })
        });
        let results = match installing {
            Ok(installing) => installing.await,
            Err(_) => Vec::new(),
        };
        drop(pump);
        self.busy.hide(cx);

        let labels: Vec<String> = chosen.iter().map(|variant| variant.label.clone()).collect();
        let summary = summarize_variants(&labels, &results, strings);
        if !summary.installed.is_empty() {
            notify(
                cx,
                "mod_variants",
                Msg::ModVariantInstalledToast {
                    count: count(summary.installed.len()),
                },
                Severity::Success,
            );
        }
        for (_, reason) in &summary.failures {
            notify(
                cx,
                "mod_variant_fail",
                Msg::ModInstallFailed {
                    reason: reason.clone(),
                },
                Severity::Error,
            );
        }
        self.report.installed.extend(summary.installed);
        for (index, result) in results.into_iter().enumerate() {
            if let Err(failure) = result {
                let sub_path = chosen
                    .get(index)
                    .map(|variant| source.join(&variant.sub_path))
                    .unwrap_or_else(|| source.clone());
                self.report.failed.push((sub_path, failure));
            }
        }
    }

    async fn bulk(&mut self, sources: Vec<PathBuf>, cx: &mut AsyncWindowContext) {
        let Some(strings) = strings(cx) else {
            return;
        };
        self.busy.show(strings.mod_bulk_install_scanning(), cx);
        let archives = cx
            .background_executor()
            .spawn(async move {
                let mut archives = Vec::new();
                for source in sources {
                    let found = yp_mods::bulk::find_archives(&source);
                    if found.is_empty() && DropStaging::is_archive(&source) {
                        archives.push(source);
                    } else {
                        archives.extend(found);
                    }
                }
                archives
            })
            .await;
        if archives.is_empty() {
            self.busy.hide(cx);
            notify(cx, "bulk_none", Msg::ModBulkInstallNone, Severity::Warning);
            return;
        }

        let total = archives.len();
        let mut failures = Vec::new();
        for (index, archive) in archives.iter().enumerate() {
            let base_name = file_stem_of(archive);
            self.busy.show(
                strings.mod_bulk_install_busy(count(index + 1), count(total), &base_name),
                cx,
            );
            let requested_name = prettify_mod_id(&base_name);
            let source = archive.clone();
            let mods = self.stores.mods.clone();
            let installing = cx.update(|_, cx| {
                mods.update(cx, |mods, cx| {
                    mods.install_archive(source, requested_name, None, cx)
                })
            });
            let result = match installing {
                Ok(installing) => installing.await,
                Err(_) => Err(InstallFailure::Cancelled),
            };
            match result {
                Ok(installed) => self.report.installed.push(installed.id),
                Err(failure) => {
                    let reason = install_failure_reason(&failure, strings);
                    failures.push(strings.mods_ui_label_value(&base_name, &reason));
                    self.report.failed.push((archive.clone(), failure));
                }
            }
        }
        self.busy.hide(cx);

        let installed = self.report.installed.len();
        notify(
            cx,
            "bulk_done",
            Msg::ModBulkInstallDone {
                installed: count(installed),
                total: count(total),
            },
            bulk_done_severity(installed, total),
        );
        for reason in failures {
            notify(
                cx,
                "bulk_fail",
                Msg::ModInstallFailed { reason },
                Severity::Error,
            );
        }
    }

    async fn loose_folder(&mut self, folder: PathBuf, cx: &mut AsyncWindowContext) {
        let Some(strings) = strings(cx) else {
            return;
        };
        self.busy.show(strings.mod_loose_install_scanning(), cx);
        let scan_folder = folder.clone();
        let total = cx
            .background_executor()
            .spawn(async move { yp_mods::loose::count_files(&scan_folder) })
            .await;
        if total == 0 {
            self.busy.hide(cx);
            notify(
                cx,
                "loose_none",
                Msg::ModLooseInstallNone,
                Severity::Warning,
            );
            return;
        }

        self.busy
            .show(strings.mod_loose_install_busy(count(total)), cx);
        let (sender, updates) = progress_channel();
        let pump = self.busy.pump(
            updates,
            move |progress, strings| {
                let done = progress.current.as_deref()?.parse::<i64>().ok()?;
                Some(strings.mod_loose_install_progress(done, count(total)))
            },
            cx,
        );
        let env = self.env.clone();
        let stage_folder = folder.clone();
        let staged = cx
            .background_executor()
            .spawn(async move {
                let relay = move |progress: Progress| {
                    let _ = sender.unbounded_send(progress);
                };
                yp_mods::loose::stage_folder(
                    &env.staging(),
                    &stage_folder,
                    &relay,
                    &CancelToken::new(),
                )
            })
            .await;
        drop(pump);
        let staged = match staged {
            Ok(Some(staged)) => staged,
            Ok(None) => {
                self.busy.hide(cx);
                self.fail(folder, InstallFailure::DataEmpty, cx);
                return;
            }
            Err(error) => {
                self.busy.hide(cx);
                self.fail(folder, failure_from_core(error), cx);
                return;
            }
        };

        self.busy.show(strings.mod_loose_install_finalizing(), cx);
        let base_name = prettify_mod_id(&file_name_of(&folder));
        let staged_root = staged.path().to_path_buf();
        let mods = self.stores.mods.clone();
        let installing = cx.update(|_, cx| {
            mods.update(cx, |mods, cx| {
                mods.install_auto_named(staged_root, base_name, None, cx)
            })
        });
        let result = match installing {
            Ok(installing) => installing.await,
            Err(_) => Err(InstallFailure::Cancelled),
        };
        drop(staged);
        self.busy.hide(cx);

        match result {
            Ok(installed) => {
                notify(
                    cx,
                    "loose_done",
                    Msg::ModLooseInstallDone {
                        count: count(total),
                        id: installed.id.clone(),
                    },
                    Severity::Success,
                );
                if !installed.unpaired_warnings.is_empty() {
                    notify(
                        cx,
                        "loose_unpaired",
                        Msg::ModLooseUnpairedWarning {
                            names: installed.unpaired_warnings.join(", "),
                        },
                        Severity::Warning,
                    );
                }
                self.report.installed.push(installed.id);
            }
            Err(failure) => self.fail(folder, failure, cx),
        }
    }

    fn offer_outfit_guide(&self, before: &HashSet<String>, cx: &mut AsyncWindowContext) {
        let stores = self.stores.clone();
        cx.update(|window, cx| {
            if cx.prefs().get(&OUTFIT_GUIDE_SHOWN) {
                return;
            }
            let Some(added) = first_new_outfit(before, stores.mods.read(cx).mods()) else {
                return;
            };
            let name = added.display_name.clone();
            let has_compat_config = added
                .data
                .as_ref()
                .is_some_and(|data| data.has_compat_config);
            let key = stores.config.read(cx).text(OPEN_OVERLAY_KEY_ID);
            cx.prefs().set(&OUTFIT_GUIDE_SHOWN, &true);
            window.open_modal(cx, |_, cx| {
                OutfitGuideDialog::new(name, key, has_compat_config, cx)
            });
        })
        .ok();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Job {
    Request(ModInstallRequest),
    LooseFolder(PathBuf),
}

fn start(job: Job, window: &mut Window, cx: &mut App) -> Task<ModInstallReport> {
    let stores = cx.stores().clone();
    let env = cx.env().clone();
    if stores.session.read(cx).game().is_none() {
        return Task::ready(ModInstallReport {
            cancelled: true,
            ..ModInstallReport::default()
        });
    }
    window.spawn(cx, async move |cx| {
        let mut flow = Flow::new(stores, env);
        let before = flow.mod_ids(cx);
        match job {
            Job::Request(request) => {
                flow.texture_packs = request.texture_packs;
                match request.mode {
                    ModInstallMode::Guided => flow.guided(request.sources, cx).await,
                    ModInstallMode::Bulk => flow.bulk(request.sources, cx).await,
                }
            }
            Job::LooseFolder(folder) => flow.loose_folder(folder, cx).await,
        }
        flow.busy.hide(cx);
        flow.offer_outfit_guide(&before, cx);
        flow.report
    })
}

pub fn run(
    request: ModInstallRequest,
    window: &mut Window,
    cx: &mut App,
) -> Task<ModInstallReport> {
    start(Job::Request(request), window, cx)
}

pub fn run_loose_folder(
    folder: PathBuf,
    window: &mut Window,
    cx: &mut App,
) -> Task<ModInstallReport> {
    start(Job::LooseFolder(folder), window, cx)
}

pub(crate) fn init(cx: &mut App) {
    cx.registry_mut().flows.set_mod_install(run);
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_mods::{DataCategory, DataDirEntry, DataSummary, Installed, ManifestInfo, ModVariant};
    use yp_mods::{PlayerModelEntry, TexturePack};
    use yp_ui::Locale;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    fn drop_of(kind: ModKind) -> DetectedDrop {
        let mut detected = DetectedDrop::rejected(
            PathBuf::from("drop"),
            "kimono_wax-12-3".to_owned(),
            DropRejection::UnknownDrop,
        );
        detected.kind = kind;
        detected.rejection = None;
        detected
    }

    fn variant(label: &str) -> ModVariant {
        ModVariant {
            sub_path: PathBuf::from(label),
            label: label.to_owned(),
            kind: ModKind::Data,
            texture_only: false,
            category: Some(DataCategory::Player),
        }
    }

    #[test]
    fn drops_with_variants_go_to_the_variant_picker_first() {
        let mut detected = drop_of(ModKind::Unknown);
        detected.variants = vec![variant("A"), variant("B")];
        assert_eq!(detect_step(&detected), DetectStep::Variants);
    }

    #[test]
    fn unknown_drops_are_rejected_with_their_reason() {
        let mut detected = drop_of(ModKind::Unknown);
        assert_eq!(
            detect_step(&detected),
            DetectStep::Rejected(DropRejection::UnknownDrop)
        );
        detected.rejection = Some(DropRejection::UnsupportedNasa);
        assert_eq!(
            detect_step(&detected),
            DetectStep::Rejected(DropRejection::UnsupportedNasa)
        );
    }

    #[test]
    fn single_mods_take_the_manifest_id_or_ask_for_a_name() {
        let mut detected = drop_of(ModKind::Native);
        assert_eq!(
            detect_step(&detected),
            DetectStep::Single {
                needs_texture_choice: false,
                manifest_name: None
            }
        );
        detected.manifest = Some(ManifestInfo {
            id: Some("  bunker_cam  ".to_owned()),
            ..ManifestInfo::default()
        });
        assert_eq!(
            detect_step(&detected),
            DetectStep::Single {
                needs_texture_choice: false,
                manifest_name: Some("bunker_cam".to_owned())
            }
        );
        detected.manifest = Some(ManifestInfo {
            id: Some("   ".to_owned()),
            ..ManifestInfo::default()
        });
        assert_eq!(
            detect_step(&detected),
            DetectStep::Single {
                needs_texture_choice: false,
                manifest_name: None
            }
        );
    }

    #[test]
    fn several_texture_sets_ask_which_ones_to_bundle() {
        let mut detected = drop_of(ModKind::Data);
        let pack = |label: &str| TexturePack {
            path: PathBuf::from("drop").join(label),
            label: label.to_owned(),
        };
        detected.texture_variants = vec![pack("2K"), pack("4K")];
        assert_eq!(
            detect_step(&detected),
            DetectStep::Single {
                needs_texture_choice: true,
                manifest_name: None
            }
        );
    }

    #[test]
    fn the_gate_names_the_right_tab_for_cutscene_drops() {
        assert_eq!(gate_notice(DropGate::Mod), None);
        assert_eq!(
            gate_notice(DropGate::MisroutedCutscenes),
            Some(Msg::ModDropMisroutedCutscenes)
        );
        assert_eq!(gate_notice(DropGate::NotAMod), Some(Msg::ModDropNotAMod));
        assert_eq!(gate_rejection(DropGate::NotAMod), DropRejection::UnknownDrop);
    }

    #[test]
    fn the_gate_says_why_an_archive_cannot_be_read() {
        use yp_core::ArchiveFailure;
        assert_eq!(
            gate_notice(DropGate::UnreadableArchive(ArchiveFailure::PasswordRequired)),
            Some(Msg::ArchivePasswordProtected)
        );
        assert_eq!(
            gate_notice(DropGate::UnreadableArchive(ArchiveFailure::Corrupt)),
            Some(Msg::ArchiveDamaged)
        );
        assert_eq!(
            gate_rejection(DropGate::UnreadableArchive(ArchiveFailure::Corrupt)),
            DropRejection::ArchiveExtractFailed(ArchiveFailure::Corrupt)
        );
        let (msg, severity) = rejection_notice(
            DropRejection::ArchiveExtractFailed(ArchiveFailure::PasswordRequired),
            strings(),
        );
        assert_eq!(severity, Severity::Error);
        assert_eq!(
            msg,
            Msg::ModInstallFailed {
                reason: strings().archive_password_protected().to_owned()
            }
        );
    }

    #[test]
    fn texture_only_drops_warn_and_other_rejections_fail() {
        let (msg, severity) = rejection_notice(DropRejection::TextureOnly, strings());
        assert_eq!(msg, Msg::ModInstallReasonTextureOnly);
        assert_eq!(severity, Severity::Warning);
        let (msg, severity) = rejection_notice(DropRejection::InvalidMixed, strings());
        assert_eq!(severity, Severity::Error);
        assert_eq!(
            msg,
            Msg::ModInstallFailed {
                reason: strings().mod_install_reason_invalid_mixed().to_owned()
            }
        );
        let (_, severity) = failure_notice(&InstallFailure::NativeEmpty, strings());
        assert_eq!(severity, Severity::Error);
    }

    #[test]
    fn extraction_progress_names_the_file_when_there_is_one() {
        let strings = strings();
        assert_eq!(extract_message(&Progress::message("x"), strings), None);
        assert_eq!(
            extract_message(&Progress::fraction(0.424), strings),
            Some(strings.extracting_archive_percent(42))
        );
        assert_eq!(
            extract_message(&Progress::step(1, 2, "pl000d.dat"), strings),
            Some(strings.extracting_archive_percent_file(50, "pl000d.dat"))
        );
    }

    fn outfit(id: &str) -> InstalledMod {
        let mut installed = InstalledMod::new(id, PathBuf::from(id), ModKind::Data);
        installed.data = Some(DataSummary {
            players: vec![PlayerModelEntry {
                file_name: "pl000d.dat".to_owned(),
                label: "2B".to_owned(),
            }],
            ..DataSummary::default()
        });
        installed
    }

    #[test]
    fn the_outfit_guide_is_offered_for_the_first_newly_added_outfit() {
        let mut weapon = InstalledMod::new("sword", PathBuf::from("sword"), ModKind::Data);
        weapon.data = Some(DataSummary {
            entries: vec![DataDirEntry {
                dir_name: "wp".to_owned(),
                category: DataCategory::Weapon,
                file_count: 2,
            }],
            ..DataSummary::default()
        });
        let mut loose_outfit = InstalledMod::new("loose", PathBuf::from("loose"), ModKind::Data);
        loose_outfit.data = Some(DataSummary {
            entries: vec![DataDirEntry {
                dir_name: "pl".to_owned(),
                category: DataCategory::Player,
                file_count: 2,
            }],
            ..DataSummary::default()
        });
        assert!(!is_outfit_mod(&weapon));
        assert!(is_outfit_mod(&loose_outfit));
        assert!(is_outfit_mod(&outfit("kimono")));

        let mods = vec![outfit("old"), weapon, outfit("kimono"), loose_outfit];
        let before: HashSet<String> = ["old".to_owned()].into_iter().collect();
        assert_eq!(
            first_new_outfit(&before, &mods).map(|installed| installed.id.as_str()),
            Some("kimono")
        );
        let all: HashSet<String> = mods.iter().map(|installed| installed.id.clone()).collect();
        assert_eq!(first_new_outfit(&all, &mods), None);
    }

    #[test]
    fn variant_results_are_split_into_installed_ids_and_labelled_failures() {
        let labels = vec!["Classic".to_owned(), "Soulcalibur".to_owned()];
        let results = vec![
            Ok(Installed {
                id: "two_p_classic".to_owned(),
                ..Installed::default()
            }),
            Err(InstallFailure::DataEmpty),
        ];
        let summary = summarize_variants(&labels, &results, strings());
        assert_eq!(summary.installed, vec!["two_p_classic".to_owned()]);
        assert_eq!(summary.failures.len(), 1);
        assert_eq!(summary.failures[0].0, 1);
        assert!(summary.failures[0].1.starts_with("Soulcalibur"));
        assert!(
            summary.failures[0]
                .1
                .contains(strings().mod_install_reason_data_empty())
        );
    }

    #[test]
    fn bulk_installs_only_report_success_when_every_archive_installed() {
        assert_eq!(bulk_done_severity(3, 3), Severity::Success);
        assert_eq!(bulk_done_severity(2, 3), Severity::Warning);
        assert_eq!(bulk_done_severity(0, 3), Severity::Warning);
    }
}
