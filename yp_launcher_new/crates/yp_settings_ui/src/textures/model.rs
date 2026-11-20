use std::path::PathBuf;
use std::sync::mpsc as reply;
use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc;
use gpui::{AsyncWindowContext, EventEmitter, Subscription, Task, WeakEntity};
use indexmap::IndexMap;
use yp_config::fields::texture_injection;
use yp_core::{CancelToken, GamePaths, Progress};
use yp_host::nams_cli::{self, NamsTexturePack};
use yp_i18n::{Msg, MsgKey, Strings};
use yp_mods::ModVariant;
use yp_mods::textures::pack_install::{self, MergeTarget, TexturePackPrompts};
use yp_mods::textures::{
    TextureCopyProgress, TextureListing, TextureRoots, bundled_origin_by_pack, delete_texture,
    drop_contains_dds, load_textures,
};
use yp_state::extraction_error_key;
use yp_state::prelude::*;
use yp_state::{
    BusyHandle, ConfigEvent, DetectionEvent, Notice, NoticeKind, NoticeSeverity, SessionEvent,
    Stores,
};
use yp_ui::prelude::*;
use yp_ui::{
    ConfirmOptions, ConfirmTone, IconName, PickItem, PickOptions, PromptOptions, confirm,
    pick_many, pick_one, prompt_text,
};

use crate::errors::core_error_text;

pub const TEXTURES_REL_PREFIX: &str = "inject/textures/";
const NAMS_LIST_TIMEOUT: Duration = Duration::from_secs(15);
const FORBIDDEN_NAME_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const BYTES_PER_MB: u64 = 1024 * 1024;

pub fn pack_rel_path(folder: &str) -> String {
    format!("{TEXTURES_REL_PREFIX}{folder}")
}

pub fn sanitize_pack_name(name: &str) -> Option<String> {
    let cleaned: String = name
        .trim()
        .chars()
        .map(|letter| {
            if FORBIDDEN_NAME_CHARS.contains(&letter) {
                '_'
            } else {
                letter
            }
        })
        .collect();
    (!cleaned.is_empty()).then_some(cleaned)
}

pub fn string_list(values: &toml::Table, key: &str) -> Vec<String> {
    values
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

pub fn names_value(names: &[String]) -> toml::Value {
    toml::Value::Array(names.iter().cloned().map(toml::Value::String).collect())
}

pub fn visible_order(load_order: &[String], disabled_packs: &[String]) -> Vec<String> {
    load_order
        .iter()
        .filter(|name| !disabled_packs.contains(name))
        .cloned()
        .collect()
}

pub fn reordered(order: &[String], from: usize, to: usize) -> Option<Vec<String>> {
    let mut next = order.to_vec();
    yp_ui::reorder(&mut next, from, to).then_some(next)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Highest,
    Lowest,
}

pub fn priority_of(index: usize, count: usize) -> Option<Priority> {
    if count <= 1 {
        None
    } else if index == 0 {
        Some(Priority::Highest)
    } else if index + 1 == count {
        Some(Priority::Lowest)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackRow {
    pub name: String,
    pub exists: bool,
    pub priority: Option<Priority>,
    pub conflicts: Vec<String>,
    pub bundled_with: Option<String>,
    pub disabled: bool,
}

pub fn pack_rows(
    order: &[String],
    listing: &TextureListing,
    bundled: &IndexMap<String, String>,
    is_disabled: impl Fn(&str) -> bool,
) -> Vec<PackRow> {
    order
        .iter()
        .enumerate()
        .map(|(index, name)| PackRow {
            name: name.clone(),
            exists: listing.nams_folders.contains(name),
            priority: priority_of(index, order.len()),
            conflicts: listing.conflicts.get(name).cloned().unwrap_or_default(),
            bundled_with: bundled.get(name).cloned(),
            disabled: is_disabled(&pack_rel_path(name)),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallPhase {
    Analyzing,
    Extracting {
        fraction: Option<f32>,
        file: Option<String>,
    },
    Copying(TextureCopyProgress),
}

impl InstallPhase {
    pub fn text(&self, strings: &'static Strings) -> String {
        match self {
            InstallPhase::Analyzing => strings.texture_drop_analyzing().to_owned(),
            InstallPhase::Extracting { fraction, file } => {
                let percent = fraction.map(|fraction| (fraction * 100.).round() as i64);
                match (percent, file.as_deref().filter(|file| !file.is_empty())) {
                    (Some(percent), Some(file)) => {
                        strings.extracting_archive_percent_file(percent, file)
                    }
                    (Some(percent), None) => strings.extracting_archive_percent(percent),
                    _ => strings.textures_progress_extracting().to_owned(),
                }
            }
            InstallPhase::Copying(progress) if progress.total > 0 => strings
                .textures_install_progress(
                    i64::try_from(progress.files).unwrap_or(i64::MAX),
                    i64::try_from(progress.total_files).unwrap_or(i64::MAX),
                    i64::try_from(progress.copied / BYTES_PER_MB).unwrap_or(i64::MAX),
                    i64::try_from(progress.total / BYTES_PER_MB).unwrap_or(i64::MAX),
                ),
            InstallPhase::Copying(_) => strings.textures_progress_copying().to_owned(),
        }
    }

    pub fn percent(&self) -> Option<f32> {
        match self {
            InstallPhase::Analyzing => None,
            InstallPhase::Extracting { fraction, .. } => {
                fraction.filter(|fraction| *fraction > 0.).map(|fraction| fraction * 100.)
            }
            InstallPhase::Copying(progress) => {
                let percent = progress.percent() as f32 * 100.;
                (percent > 0.).then_some(percent)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    NoDds,
    Cancelled,
    NothingFound,
    Installed(usize),
    Failed(String),
}

impl InstallOutcome {
    pub fn from_result(result: yp_core::Result<Option<usize>>, strings: &'static Strings) -> Self {
        match result {
            Ok(None) => InstallOutcome::Cancelled,
            Ok(Some(0)) => InstallOutcome::NothingFound,
            Ok(Some(count)) => InstallOutcome::Installed(count),
            Err(error) if error.is_cancelled() => InstallOutcome::Cancelled,
            Err(error) => InstallOutcome::Failed(core_error_text(&error, strings)),
        }
    }

    pub fn notice(&self) -> Option<(Msg, NoticeSeverity)> {
        match self {
            InstallOutcome::NoDds => Some((Msg::TextureDropNoDds, NoticeSeverity::Error)),
            InstallOutcome::Cancelled => None,
            InstallOutcome::NothingFound => {
                Some((Msg::NoTextureFilesFound, NoticeSeverity::Warning))
            }
            InstallOutcome::Installed(count) => Some((
                Msg::InstalledTextureCount {
                    count: i64::try_from(*count).unwrap_or(i64::MAX),
                },
                NoticeSeverity::Success,
            )),
            InstallOutcome::Failed(error) => Some((
                Msg::InstallationFailed {
                    error: error.clone(),
                },
                NoticeSeverity::Error,
            )),
        }
    }

    pub fn changed_files(&self) -> bool {
        matches!(
            self,
            InstallOutcome::Installed(_) | InstallOutcome::NothingFound | InstallOutcome::Failed(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeChoice {
    Into(String),
    NewPack,
}

pub fn merge_target(choice: Option<MergeChoice>) -> MergeTarget {
    match choice {
        None => MergeTarget::Cancel,
        Some(MergeChoice::NewPack) => MergeTarget::KeepNames,
        Some(MergeChoice::Into(folder)) => MergeTarget::Into(folder),
    }
}

pub enum PromptRequest {
    ChoosePacks {
        packs: Vec<ModVariant>,
        reply: reply::Sender<Option<Vec<ModVariant>>>,
    },
    PackName {
        fallback: String,
        reply: reply::Sender<Option<String>>,
    },
    MergeTarget {
        reply: reply::Sender<MergeTarget>,
    },
    ExtractFailed(PathBuf, MsgKey),
    ExtractProgress(Progress),
    CopyStarted,
    CopyProgress(TextureCopyProgress),
}

pub struct ChannelPrompts {
    requests: mpsc::UnboundedSender<PromptRequest>,
    ask_merge: bool,
}

impl ChannelPrompts {
    pub fn new(requests: mpsc::UnboundedSender<PromptRequest>, ask_merge: bool) -> Self {
        Self {
            requests,
            ask_merge,
        }
    }

    fn ask<T>(&self, build: impl FnOnce(reply::Sender<T>) -> PromptRequest) -> Option<T> {
        let (sender, receiver) = reply::channel();
        self.requests.unbounded_send(build(sender)).ok()?;
        receiver.recv().ok()
    }

    fn tell(&self, request: PromptRequest) {
        let _ = self.requests.unbounded_send(request);
    }
}

impl TexturePackPrompts for ChannelPrompts {
    fn choose_packs(&mut self, packs: &[ModVariant]) -> Option<Vec<ModVariant>> {
        let packs = packs.to_vec();
        self.ask(|reply| PromptRequest::ChoosePacks { packs, reply })
            .flatten()
    }

    fn pack_name(&mut self, fallback: &str) -> Option<String> {
        let fallback = fallback.to_owned();
        self.ask(|reply| PromptRequest::PackName { fallback, reply })
            .flatten()
    }

    fn merge_target(&mut self) -> MergeTarget {
        if !self.ask_merge {
            return MergeTarget::KeepNames;
        }
        self.ask(|reply| PromptRequest::MergeTarget { reply })
            .unwrap_or(MergeTarget::Cancel)
    }

    fn extract_failed(&mut self, archive: &std::path::Path, error: &yp_core::Error) {
        self.tell(PromptRequest::ExtractFailed(
            archive.to_path_buf(),
            extraction_error_key(error),
        ));
    }

    fn copy_started(&mut self) {
        self.tell(PromptRequest::CopyStarted);
    }

    fn copy_progress(&mut self, progress: &TextureCopyProgress) {
        self.tell(PromptRequest::CopyProgress(*progress));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexturesEvent {
    Reloaded,
    InstallFinished(InstallOutcome),
    Deleted(String),
}

pub struct TexturesModel {
    stores: Stores,
    env: AppEnv,
    game: Option<GamePaths>,
    listing: TextureListing,
    outfit_packs: Vec<NamsTexturePack>,
    loading: bool,
    install: Option<InstallPhase>,
    deleting: bool,
    notice_counter: u64,
    load_task: Option<Task<()>>,
    load_running: bool,
    reload_queued: bool,
    install_task: Option<Task<()>>,
    delete_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<TexturesEvent> for TexturesModel {}

impl TexturesModel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let env = cx.env().clone();
        let subscriptions = vec![
            cx.subscribe(&stores.session, |this, _, event: &SessionEvent, cx| {
                if let SessionEvent::DirectoryChanged { game, .. } = event {
                    this.bind_game(Some(game.clone()), cx);
                }
            }),
            cx.subscribe(&stores.config, |this, _, event: &ConfigEvent, cx| {
                if matches!(event, ConfigEvent::Loaded | ConfigEvent::ReloadedExternally) {
                    this.reload(cx);
                }
            }),
            cx.subscribe(&stores.detection, |this, _, event: &DetectionEvent, cx| {
                if matches!(event, DetectionEvent::Invalidated { .. }) && this.install.is_none() {
                    this.stores
                        .config
                        .update(cx, |store, cx| store.reload_if_clean(cx));
                    this.reload(cx);
                }
            }),
        ];
        let game = stores.session.read(cx).game();
        let mut this = Self {
            stores,
            env,
            game: None,
            listing: TextureListing::default(),
            outfit_packs: Vec::new(),
            loading: true,
            install: None,
            deleting: false,
            notice_counter: 0,
            load_task: None,
            load_running: false,
            reload_queued: false,
            install_task: None,
            delete_task: None,
            _subscriptions: subscriptions,
        };
        this.bind_game(game, cx);
        this
    }

    pub fn game(&self) -> Option<&GamePaths> {
        self.game.as_ref()
    }

    pub fn listing(&self) -> &TextureListing {
        &self.listing
    }

    pub fn outfit_packs(&self) -> &[NamsTexturePack] {
        &self.outfit_packs
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn install_phase(&self) -> Option<&InstallPhase> {
        self.install.as_ref()
    }

    pub fn is_installing(&self) -> bool {
        self.install.is_some()
    }

    pub fn is_deleting(&self) -> bool {
        self.deleting
    }

    pub fn has_any_textures(&self) -> bool {
        !(self.listing.nams.is_empty() && self.listing.sk.is_empty() && self.listing.wax.is_empty())
    }

    fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        if self.game != game {
            self.game = game;
            self.listing = TextureListing::default();
            self.outfit_packs.clear();
            self.loading = true;
        }
        self.reload(cx);
    }

    fn texture_values(&self, cx: &App) -> toml::Table {
        self.stores.config.read(cx).texture_injection().clone()
    }

    pub fn load_order(&self, cx: &App) -> Vec<String> {
        string_list(&self.texture_values(cx), texture_injection::LOAD_ORDER.key())
    }

    pub fn disabled_packs(&self, cx: &App) -> Vec<String> {
        string_list(
            &self.texture_values(cx),
            texture_injection::DISABLED_PACKS.key(),
        )
    }

    pub fn order(&self, cx: &App) -> Vec<String> {
        visible_order(&self.load_order(cx), &self.disabled_packs(cx))
    }

    pub fn rows(&self, cx: &App) -> Vec<PackRow> {
        let bundled = bundled_origin_by_pack(self.stores.mods.read(cx).mods());
        let disabled = self.stores.disabled_mods.read(cx);
        pack_rows(&self.order(cx), &self.listing, &bundled, |rel| {
            disabled.is_disabled(rel)
        })
    }

    fn save_load_order(&self, order: &[String], cx: &mut App) {
        let value = names_value(order);
        self.stores.config.update(cx, |store, cx| {
            store.update_texture_injection_silent(texture_injection::LOAD_ORDER.key(), value, cx);
            store.save(cx);
        });
    }

    pub fn set_value(&self, key: &'static str, value: toml::Value, cx: &mut App) {
        self.stores.config.update(cx, |store, cx| {
            store.update_texture_injection_silent(key, value, cx);
            store.save(cx);
        });
    }

    pub fn reorder(&mut self, from: usize, to: usize, cx: &mut Context<Self>) {
        let Some(order) = reordered(&self.order(cx), from, to) else {
            return;
        };
        self.save_load_order(&order, cx);
        cx.notify();
    }

    pub fn set_pack_enabled(&mut self, folder: &str, enabled: bool, cx: &mut Context<Self>) {
        let rel = pack_rel_path(folder);
        self.stores
            .disabled_mods
            .update(cx, |store, cx| store.set_disabled(&rel, !enabled, cx));
        cx.notify();
    }

    fn push_notice(&mut self, msg: Msg, severity: NoticeSeverity, cx: &mut Context<Self>) {
        self.notice_counter += 1;
        let notice = Notice::new(
            format!("texture_{}", self.notice_counter),
            msg,
            NoticeKind::Textures,
            severity,
        );
        self.stores
            .notifications
            .update(cx, |store, cx| store.push(notice, cx));
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(game) = self.game.clone() else {
            self.loading = false;
            cx.notify();
            return;
        };
        if self.load_running {
            self.reload_queued = true;
            return;
        }
        self.load_running = true;
        let disabled = self.disabled_packs(cx);
        let runner = self.env.runner.clone();
        let paths = self.env.paths.clone();
        let loaded_for = game.clone();
        let job = cx.background_spawn(async move {
            let listing = load_textures(&TextureRoots::for_game(&game), &disabled);
            let packs =
                nams_cli::textures_list(&*runner, &paths, game.root(), Some(NAMS_LIST_TIMEOUT));
            (listing, packs.unwrap_or_default())
        });
        self.load_task = Some(cx.spawn(async move |this, cx| {
            let (listing, packs) = job.await;
            this.update(cx, |this, cx| {
                this.load_running = false;
                let queued = std::mem::take(&mut this.reload_queued);
                if this.game.as_ref() == Some(&loaded_for) {
                    this.apply_listing(listing, packs, cx);
                }
                if queued || this.game.as_ref() != Some(&loaded_for) {
                    this.reload(cx);
                }
            })
            .ok();
        }));
    }

    fn apply_listing(
        &mut self,
        listing: TextureListing,
        packs: Vec<NamsTexturePack>,
        cx: &mut Context<Self>,
    ) {
        self.listing = listing;
        self.outfit_packs = packs
            .into_iter()
            .filter(|pack| pack.outfit_conditional)
            .collect();
        self.loading = false;
        let folders = self.listing.nams_folders.clone();
        self.stores
            .config
            .update(cx, |store, cx| store.reconcile_texture_load_order(folders, cx));
        cx.emit(TexturesEvent::Reloaded);
        cx.notify();
    }

    pub fn install(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        if self.install.is_some() || paths.is_empty() {
            return;
        }
        let Some(game) = self.game.clone() else {
            return;
        };
        self.install = Some(InstallPhase::Analyzing);
        cx.notify();
        let busy: BusyHandle = self
            .stores
            .busy
            .update(cx, |busy, cx| busy.enter(BusyScope::Textures, cx));
        let env = self.env.clone();
        let folders = self.listing.nams_folders.clone();
        self.install_task = Some(cx.spawn_in(window, async move |this, cx| {
            let outcome = run_install(&this, env, game, paths, folders, cx).await;
            drop(busy);
            this.update(cx, |this, cx| this.finish_install(outcome, cx))
                .ok();
        }));
    }

    fn set_phase(&mut self, phase: InstallPhase, cx: &mut Context<Self>) {
        if self.install.is_some() {
            self.install = Some(phase);
            cx.notify();
        }
    }

    fn finish_install(&mut self, outcome: InstallOutcome, cx: &mut Context<Self>) {
        self.install = None;
        if let Some((msg, severity)) = outcome.notice() {
            self.push_notice(msg, severity, cx);
        }
        if outcome.changed_files() {
            self.reload(cx);
            self.stores
                .detection
                .update(cx, |store, cx| store.invalidate(cx));
        }
        cx.emit(TexturesEvent::InstallFinished(outcome));
        cx.notify();
    }

    pub fn delete(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.deleting {
            return;
        }
        let strings = cx.t();
        let answer = confirm(
            ConfirmOptions::new(
                strings.texture_delete_confirm_title(),
                strings.texture_delete_confirm_body(&name),
            )
            .confirm_label(strings.texture_delete_label())
            .cancel_label(strings.cancel_button())
            .tone(ConfirmTone::Danger),
            window,
            cx,
        );
        self.delete_task = Some(cx.spawn(async move |this, cx| {
            if answer.await {
                this.update(cx, |this, cx| this.delete_confirmed(name, cx))
                    .ok();
            }
        }));
    }

    pub fn delete_confirmed(&mut self, name: String, cx: &mut Context<Self>) {
        let Some(game) = self.game.clone() else {
            return;
        };
        self.deleting = true;
        cx.notify();
        let busy = self
            .stores
            .busy
            .update(cx, |busy, cx| busy.enter(BusyScope::General, cx));
        let target = game.inject_textures_dir().join(&name);
        let job = cx.background_spawn(async move { delete_texture(&target) });
        self.delete_task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            drop(busy);
            this.update(cx, |this, cx| this.finish_delete(name, result, cx))
                .ok();
        }));
    }

    fn finish_delete(
        &mut self,
        name: String,
        result: yp_core::Result<bool>,
        cx: &mut Context<Self>,
    ) {
        self.deleting = false;
        match result {
            Ok(true) => {
                let mut order = self.load_order(cx);
                order.retain(|entry| *entry != name);
                self.save_load_order(&order, cx);
                self.push_notice(
                    Msg::RemovedItem { name: name.clone() },
                    NoticeSeverity::Info,
                    cx,
                );
                self.reload(cx);
                self.stores
                    .detection
                    .update(cx, |store, cx| store.invalidate(cx));
                cx.emit(TexturesEvent::Deleted(name));
            }
            Ok(false) => {}
            Err(error) => {
                let text = core_error_text(&error, cx.t());
                self.push_notice(
                    Msg::InstallationFailed { error: text },
                    NoticeSeverity::Error,
                    cx,
                );
            }
        }
        cx.notify();
    }
}

async fn run_install(
    this: &WeakEntity<TexturesModel>,
    env: AppEnv,
    game: GamePaths,
    paths: Vec<PathBuf>,
    folders: Vec<String>,
    cx: &mut AsyncWindowContext,
) -> InstallOutcome {
    let check_env = env.clone();
    let check_paths = paths.clone();
    let (staging, has_dds) = cx
        .background_executor()
        .spawn(async move {
            let staging = check_env.staging();
            let has_dds = drop_contains_dds(&staging, &check_paths);
            (staging, has_dds)
        })
        .await;
    if !has_dds {
        return InstallOutcome::NoDds;
    }
    let started = this.update(cx, |this, cx| {
        this.set_phase(
            InstallPhase::Extracting {
                fraction: None,
                file: None,
            },
            cx,
        );
        this.push_notice(Msg::InstallingTextures, NoticeSeverity::Accent, cx);
    });
    if started.is_err() {
        return InstallOutcome::Cancelled;
    }
    let (requests, mut inbox) = mpsc::unbounded();
    let progress_requests = requests.clone();
    let texture_dir = game.inject_textures_dir();
    let ask_merge = !folders.is_empty();
    let cancel = CancelToken::new();
    let job_cancel = cancel.clone();
    let job = cx.background_executor().spawn(async move {
        let progress = move |update: Progress| {
            let _ = progress_requests.unbounded_send(PromptRequest::ExtractProgress(update));
        };
        let mut prompts = ChannelPrompts::new(requests, ask_merge);
        pack_install::run(
            &staging,
            &paths,
            &texture_dir,
            &mut prompts,
            &progress,
            &job_cancel,
        )
    });
    while let Some(request) = inbox.next().await {
        if answer(request, this, &folders, cx).await.is_err() {
            cancel.cancel();
        }
    }
    let strings = cx.update(|_, cx| cx.t()).ok();
    let result = job.await;
    match strings {
        Some(strings) => InstallOutcome::from_result(result, strings),
        None => InstallOutcome::Cancelled,
    }
}

async fn answer(
    request: PromptRequest,
    this: &WeakEntity<TexturesModel>,
    folders: &[String],
    cx: &mut AsyncWindowContext,
) -> Result<(), ()> {
    match request {
        PromptRequest::ChoosePacks { packs, reply } => {
            let picked = cx
                .update(|window, cx| {
                    let strings = cx.t();
                    let items = packs
                        .iter()
                        .cloned()
                        .map(|pack| {
                            PickItem::new(pack.label.clone(), pack)
                                .icon(IconName::Texture)
                                .selected(true)
                        })
                        .collect();
                    pick_many(
                        PickOptions::new(strings.mod_variant_dialog_title(), items)
                            .body(strings.mod_variant_dialog_subtitle())
                            .confirm_label(strings.mod_variant_install())
                            .cancel_label(strings.cancel_button()),
                        window,
                        cx,
                    )
                })
                .map_err(|_| ())?
                .await;
            let _ = reply.send(picked);
        }
        PromptRequest::PackName { fallback, reply } => {
            let named = cx
                .update(|window, cx| {
                    let strings = cx.t();
                    prompt_text(
                        PromptOptions::new(strings.textures_pack_name_title())
                            .initial(fallback.clone())
                            .placeholder(strings.outfit_name_hint())
                            .confirm_label(strings.button_install())
                            .cancel_label(strings.button_cancel()),
                        window,
                        cx,
                    )
                })
                .map_err(|_| ())?
                .await;
            let _ = reply.send(named.as_deref().and_then(sanitize_pack_name));
        }
        PromptRequest::MergeTarget { reply } => {
            let choice = cx
                .update(|window, cx| {
                    let strings = cx.t();
                    let mut items: Vec<PickItem<MergeChoice>> = folders
                        .iter()
                        .map(|folder| {
                            PickItem::new(
                                strings.texture_merge_add_to(folder),
                                MergeChoice::Into(folder.clone()),
                            )
                            .icon(IconName::MergeType)
                        })
                        .collect();
                    items.push(
                        PickItem::new(strings.texture_merge_new_mod(), MergeChoice::NewPack)
                            .icon(IconName::Add),
                    );
                    pick_one(
                        PickOptions::new(strings.texture_merge_title(), items)
                            .body(strings.texture_merge_description())
                            .cancel_label(strings.cancel_button()),
                        window,
                        cx,
                    )
                })
                .map_err(|_| ())?
                .await;
            let _ = reply.send(merge_target(choice));
        }
        PromptRequest::ExtractFailed(_, reason) => {
            this.update(cx, |this, cx| {
                this.push_notice(Msg::from(reason), NoticeSeverity::Error, cx);
            })
            .map_err(|_| ())?;
        }
        PromptRequest::ExtractProgress(progress) => {
            this.update(cx, |this, cx| {
                this.set_phase(
                    InstallPhase::Extracting {
                        fraction: progress.fraction,
                        file: progress.current,
                    },
                    cx,
                );
            })
            .map_err(|_| ())?;
        }
        PromptRequest::CopyStarted => {
            this.update(cx, |this, cx| {
                this.set_phase(InstallPhase::Copying(TextureCopyProgress::default()), cx);
            })
            .map_err(|_| ())?;
        }
        PromptRequest::CopyProgress(progress) => {
            this.update(cx, |this, cx| {
                this.set_phase(InstallPhase::Copying(progress), cx);
            })
            .map_err(|_| ())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use futures::executor::block_on;
    use yp_i18n::Locale;

    use super::*;

    fn names(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn en() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    #[test]
    fn pack_names_lose_characters_windows_refuses() {
        assert_eq!(sanitize_pack_name("  My: Pack?  ").as_deref(), Some("My_ Pack_"));
        assert_eq!(sanitize_pack_name("a/b\\c").as_deref(), Some("a_b_c"));
        assert_eq!(sanitize_pack_name("   "), None);
        assert_eq!(sanitize_pack_name(""), None);
    }

    #[test]
    fn disabled_packs_leave_the_visible_order() {
        let order = names(&["a", "b", "c"]);
        assert_eq!(visible_order(&order, &names(&["b"])), names(&["a", "c"]));
        assert_eq!(visible_order(&order, &[]), order);
    }

    #[test]
    fn reordering_moves_one_pack_to_its_new_slot() {
        let order = names(&["a", "b", "c"]);
        assert_eq!(reordered(&order, 0, 2), Some(names(&["b", "c", "a"])));
        assert_eq!(reordered(&order, 2, 0), Some(names(&["c", "a", "b"])));
        assert_eq!(reordered(&order, 1, 1), None);
        assert_eq!(reordered(&order, 7, 0), None);
    }

    #[test]
    fn only_the_ends_of_a_list_carry_a_priority() {
        assert_eq!(priority_of(0, 1), None);
        assert_eq!(priority_of(0, 3), Some(Priority::Highest));
        assert_eq!(priority_of(1, 3), None);
        assert_eq!(priority_of(2, 3), Some(Priority::Lowest));
    }

    #[test]
    fn rows_combine_listing_conflicts_origin_and_toggle_state() {
        let mut listing = TextureListing {
            nams_folders: names(&["a", "b"]),
            ..TextureListing::default()
        };
        listing.conflicts.insert("a".into(), names(&["b"]));
        let mut bundled = IndexMap::new();
        bundled.insert("b".to_owned(), "some_mod".to_owned());
        let rows = pack_rows(&names(&["a", "ghost", "b"]), &listing, &bundled, |rel| {
            rel == "inject/textures/b"
        });
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].priority, Some(Priority::Highest));
        assert_eq!(rows[0].conflicts, names(&["b"]));
        assert!(rows[0].exists && !rows[0].disabled);
        assert!(!rows[1].exists);
        assert_eq!(rows[2].bundled_with.as_deref(), Some("some_mod"));
        assert!(rows[2].disabled);
        assert_eq!(rows[2].priority, Some(Priority::Lowest));
    }

    #[test]
    fn string_lists_skip_values_that_are_not_text() {
        let values: toml::Table = "load_order = [\"a\", 3, \"b\"]\nother = 1\n".parse().unwrap();
        assert_eq!(string_list(&values, "load_order"), names(&["a", "b"]));
        assert!(string_list(&values, "other").is_empty());
        assert!(string_list(&values, "missing").is_empty());
        assert_eq!(
            names_value(&names(&["x"])),
            toml::Value::Array(vec![toml::Value::String("x".into())])
        );
    }

    #[test]
    fn progress_text_follows_the_install_phase() {
        let strings = en();
        assert_eq!(
            InstallPhase::Analyzing.text(strings),
            strings.texture_drop_analyzing()
        );
        let extracting = InstallPhase::Extracting {
            fraction: Some(0.42),
            file: Some("a.dds".into()),
        };
        assert_eq!(
            extracting.text(strings),
            strings.extracting_archive_percent_file(42, "a.dds")
        );
        assert_eq!(extracting.percent(), Some(42.));
        let idle = InstallPhase::Extracting {
            fraction: None,
            file: None,
        };
        assert_eq!(idle.text(strings), strings.textures_progress_extracting());
        assert_eq!(idle.percent(), None);
        let copying = InstallPhase::Copying(TextureCopyProgress {
            copied: 5 * BYTES_PER_MB,
            total: 20 * BYTES_PER_MB,
            files: 1,
            total_files: 4,
        });
        assert_eq!(
            copying.text(strings),
            strings.textures_install_progress(1, 4, 5, 20)
        );
        assert_eq!(copying.percent(), Some(25.));
        assert_eq!(
            InstallPhase::Copying(TextureCopyProgress::default()).text(strings),
            strings.textures_progress_copying()
        );
    }

    #[test]
    fn install_results_decide_what_the_user_is_told() {
        let strings = en();
        assert_eq!(
            InstallOutcome::from_result(Ok(None), strings),
            InstallOutcome::Cancelled
        );
        assert_eq!(
            InstallOutcome::from_result(Ok(Some(0)), strings),
            InstallOutcome::NothingFound
        );
        assert_eq!(
            InstallOutcome::from_result(Ok(Some(7)), strings),
            InstallOutcome::Installed(7)
        );
        assert_eq!(
            InstallOutcome::from_result(Err(yp_core::Error::Cancelled), strings),
            InstallOutcome::Cancelled
        );
        let failed = InstallOutcome::from_result(
            Err(yp_core::Error::invalid("archive", "empty")),
            strings,
        );
        assert!(matches!(failed, InstallOutcome::Failed(_)));

        assert!(InstallOutcome::Cancelled.notice().is_none());
        assert!(!InstallOutcome::Cancelled.changed_files());
        assert!(!InstallOutcome::NoDds.changed_files());
        assert_eq!(
            InstallOutcome::NoDds.notice(),
            Some((Msg::TextureDropNoDds, NoticeSeverity::Error))
        );
        assert_eq!(
            InstallOutcome::Installed(3).notice(),
            Some((
                Msg::InstalledTextureCount { count: 3 },
                NoticeSeverity::Success
            ))
        );
        assert_eq!(
            InstallOutcome::NothingFound.notice(),
            Some((Msg::NoTextureFilesFound, NoticeSeverity::Warning))
        );
        assert!(InstallOutcome::Installed(1).changed_files());
    }

    #[test]
    fn merge_answers_map_to_the_install_decision() {
        assert_eq!(merge_target(None), MergeTarget::Cancel);
        assert_eq!(
            merge_target(Some(MergeChoice::NewPack)),
            MergeTarget::KeepNames
        );
        assert_eq!(
            merge_target(Some(MergeChoice::Into("Pack".into()))),
            MergeTarget::Into("Pack".into())
        );
    }

    #[test]
    fn prompts_cross_the_thread_boundary_and_wait_for_their_answer() {
        let (requests, mut inbox) = mpsc::unbounded();
        let worker = std::thread::spawn(move || {
            let mut prompts = ChannelPrompts::new(requests, true);
            let name = prompts.pack_name("fallback");
            let merge = prompts.merge_target();
            let packs = prompts.choose_packs(&[]);
            prompts.extract_failed(
                Path::new("broken.zip"),
                &yp_core::Error::archive(Path::new("broken.zip"), yp_core::ArchiveFailure::PasswordRequired),
            );
            prompts.copy_started();
            prompts.copy_progress(&TextureCopyProgress {
                copied: 1,
                total: 2,
                files: 0,
                total_files: 1,
            });
            (name, merge, packs)
        });
        let mut seen = Vec::new();
        while let Some(request) = block_on(inbox.next()) {
            match request {
                PromptRequest::PackName { fallback, reply } => {
                    seen.push("name");
                    reply.send(Some(format!("{fallback}!"))).unwrap();
                }
                PromptRequest::MergeTarget { reply } => {
                    seen.push("merge");
                    reply.send(MergeTarget::Into("Existing".into())).unwrap();
                }
                PromptRequest::ChoosePacks { reply, .. } => {
                    seen.push("packs");
                    drop(reply);
                }
                PromptRequest::ExtractFailed(path, reason) => {
                    assert_eq!(path, Path::new("broken.zip"));
                    assert_eq!(reason, MsgKey::ArchivePasswordProtected);
                    seen.push("failed");
                }
                PromptRequest::CopyStarted => seen.push("copy"),
                PromptRequest::CopyProgress(progress) => {
                    assert_eq!(progress.total, 2);
                    seen.push("progress");
                }
                PromptRequest::ExtractProgress(_) => seen.push("extract"),
            }
        }
        let (name, merge, packs) = worker.join().unwrap();
        assert_eq!(name.as_deref(), Some("fallback!"));
        assert_eq!(merge, MergeTarget::Into("Existing".into()));
        assert_eq!(packs, None);
        assert_eq!(seen, ["name", "merge", "packs", "failed", "copy", "progress"]);
    }

    #[test]
    fn merge_is_not_asked_when_nothing_is_installed_yet() {
        let (requests, mut inbox) = mpsc::unbounded();
        let mut prompts = ChannelPrompts::new(requests, false);
        assert_eq!(prompts.merge_target(), MergeTarget::KeepNames);
        drop(prompts);
        assert!(block_on(inbox.next()).is_none());
    }

    #[test]
    fn a_closed_window_cancels_pending_questions() {
        let (requests, inbox) = mpsc::unbounded();
        drop(inbox);
        let mut prompts = ChannelPrompts::new(requests, true);
        assert_eq!(prompts.merge_target(), MergeTarget::Cancel);
        assert_eq!(prompts.pack_name("x"), None);
        assert_eq!(prompts.choose_packs(&[]), None);
    }
}
