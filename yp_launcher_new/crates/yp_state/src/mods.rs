use std::path::PathBuf;
use std::sync::Arc;

use futures::channel::mpsc;
use gpui::{App, Context, EventEmitter, Task};
use yp_core::watch::Recursive;
use yp_core::{CancelToken, GamePaths, Progress, ProgressSink};
use yp_host::session::BusyCounter;
use yp_mods::defaults::{DefaultEntry, DefaultKind};
use yp_mods::profiles::ModProfiles;
use yp_mods::staging::DropStaging;
use yp_mods::state::{
    DefaultModsData, DisabledModsData, ModLoadOrderData, ModNamesData, ModsData, ModsWorkspace,
};
use yp_mods::{
    InstallFailure, InstallRequest, InstallResult, InstalledMod, ModProfile, ModProfileState,
    ModsError, VariantInstallRequest,
};

use crate::Stores;
use crate::cell::{CancelOnDrop, Guarded};
use crate::env::AppEnv;
use crate::watch::{RELOAD_DEBOUNCE, Watch, touches};

pub type ProgressSender = mpsc::UnboundedSender<Progress>;
pub type ProgressReceiver = mpsc::UnboundedReceiver<Progress>;

pub fn progress_channel() -> (ProgressSender, ProgressReceiver) {
    mpsc::unbounded()
}

fn relay(progress: Option<ProgressSender>) -> impl Fn(Progress) + Send + Sync {
    move |update| {
        if let Some(progress) = &progress {
            let _ = progress.unbounded_send(update);
        }
    }
}

pub(crate) struct DataSlot<D> {
    game: Option<GamePaths>,
    cell: Guarded<D>,
    data: D,
    issued: u64,
    applied: u64,
    loading: BusyCounter,
    error: Option<Arc<ModsError>>,
    load_task: Option<Task<()>>,
}

impl<D: Clone + Default + PartialEq + Send + 'static> DataSlot<D> {
    fn new(cx: &App) -> Self {
        Self {
            game: None,
            cell: Guarded::new(D::default(), cx.background_executor()),
            data: D::default(),
            issued: 0,
            applied: 0,
            loading: BusyCounter::new(),
            error: None,
            load_task: None,
        }
    }

    fn issue(&mut self) -> u64 {
        self.issued += 1;
        self.issued
    }

    fn accept(&mut self, ticket: u64, data: D) -> bool {
        if ticket <= self.applied {
            return false;
        }
        self.applied = ticket;
        let differs = self.data != data;
        self.data = data;
        differs
    }
}

pub(crate) trait SlotStore: Sized + 'static {
    type Data: Clone + Default + PartialEq + Send + 'static;

    fn slot(&mut self) -> &mut DataSlot<Self::Data>;
    fn refresh(data: &mut Self::Data, game: &GamePaths) -> Result<(), ModsError>;
    fn changed(&mut self, cx: &mut Context<Self>);
    fn failed(&mut self, error: Arc<ModsError>, cx: &mut Context<Self>);
}

fn slot_run<S: SlotStore>(
    store: &mut S,
    cx: &mut Context<S>,
    marks_loading: bool,
    work: impl FnOnce(&mut S::Data, &GamePaths) -> Result<(), ModsError> + Send + 'static,
) -> Task<bool> {
    let slot = store.slot();
    let Some(game) = slot.game.clone() else {
        return Task::ready(false);
    };
    let ticket = slot.issue();
    let loading = marks_loading.then(|| slot.loading.enter());
    let job = slot.cell.spawn(move |data| {
        let result = work(data, &game);
        (result, data.clone())
    });
    if marks_loading {
        cx.notify();
    }
    cx.spawn(async move |this, cx| {
        let outcome = job.await;
        drop(loading);
        let Ok((result, data)) = outcome else {
            return false;
        };
        this.update(cx, |this, cx| {
            let differs = this.slot().accept(ticket, data);
            let succeeded = match result {
                Ok(()) => {
                    this.slot().error = None;
                    if differs {
                        this.changed(cx);
                    }
                    true
                }
                Err(error) => {
                    log::warn!("mod data update failed: {error}");
                    let error = Arc::new(error);
                    this.slot().error = Some(error.clone());
                    this.failed(error, cx);
                    false
                }
            };
            cx.notify();
            succeeded
        })
        .unwrap_or(false)
    })
}

fn slot_reload<S: SlotStore>(store: &mut S, cx: &mut Context<S>) -> Task<bool> {
    slot_run(store, cx, true, S::refresh)
}

fn slot_load<S: SlotStore>(store: &mut S, cx: &mut Context<S>) {
    let reload = slot_reload(store, cx);
    store.slot().load_task = Some(cx.foreground_executor().spawn(async move {
        reload.await;
    }));
}

fn slot_bind<S: SlotStore>(store: &mut S, game: Option<GamePaths>, cx: &mut Context<S>) {
    let slot = store.slot();
    if slot.game != game {
        slot.game = game;
        slot.load_task = None;
        slot.error = None;
        slot.data = S::Data::default();
        slot.applied = slot.issue();
        slot.cell.edit(|data| *data = S::Data::default(), |_| ());
        cx.notify();
    }
    slot_load(store, cx);
}

#[derive(Debug, Clone)]
pub enum ModsEvent {
    Changed,
    Installed { ids: Vec<String> },
    Uninstalled { id: String },
    ExternalChange(Arc<[PathBuf]>),
    Failed(Arc<ModsError>),
}

pub struct ModsStore {
    env: AppEnv,
    slot: DataSlot<ModsData>,
    installing: BusyCounter,
    reload_task: Option<Task<()>>,
    watch: Option<Watch>,
}

impl EventEmitter<ModsEvent> for ModsStore {}

impl SlotStore for ModsStore {
    type Data = ModsData;

    fn slot(&mut self) -> &mut DataSlot<ModsData> {
        &mut self.slot
    }

    fn refresh(data: &mut ModsData, game: &GamePaths) -> Result<(), ModsError> {
        data.reload(game)
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(ModsEvent::Changed);
    }

    fn failed(&mut self, error: Arc<ModsError>, cx: &mut Context<Self>) {
        cx.emit(ModsEvent::Failed(error));
    }
}

impl ModsStore {
    pub fn new(env: AppEnv, cx: &mut Context<Self>) -> Self {
        Self {
            env,
            slot: DataSlot::new(cx),
            installing: BusyCounter::new(),
            reload_task: None,
            watch: None,
        }
    }

    pub fn game(&self) -> Option<&GamePaths> {
        self.slot.game.as_ref()
    }

    pub fn data(&self) -> &ModsData {
        &self.slot.data
    }

    pub fn mods(&self) -> &[InstalledMod] {
        &self.slot.data.mods
    }

    pub fn find(&self, id: &str) -> Option<&InstalledMod> {
        self.slot.data.find(id)
    }

    pub fn is_loading(&self) -> bool {
        self.slot.loading.is_busy()
    }

    pub fn is_installing(&self) -> bool {
        self.installing.is_busy()
    }

    pub fn error(&self) -> Option<&Arc<ModsError>> {
        self.slot.error.as_ref()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        let rebound = self.slot.game != game;
        slot_bind(self, game, cx);
        if rebound {
            self.reload_task = None;
            self.watch = None;
            self.start_watching(cx);
        }
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        slot_load(self, cx);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) -> Task<bool> {
        slot_reload(self, cx)
    }

    pub fn install(
        &mut self,
        source: impl Into<PathBuf>,
        request: InstallRequest,
        progress: Option<ProgressSender>,
        cx: &mut Context<Self>,
    ) -> Task<InstallResult> {
        let Some(game) = self.slot.game.clone() else {
            return Task::ready(Err(InstallFailure::Cancelled));
        };
        let source = source.into();
        let env = self.env.clone();
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let installing = self.installing.enter();
        let ticket = self.slot.issue();
        cx.notify();
        let job = self.slot.cell.spawn(move |data| {
            let result = yp_mods::install::install(
                &env.staging(),
                &game,
                &source,
                &request,
                &relay(progress),
                &cancel,
            );
            if result.is_ok()
                && let Err(error) = data.reload(&game)
            {
                log::warn!("could not list the installed mods: {error}");
            }
            (result, data.clone())
        });
        cx.spawn(async move |this, cx| {
            let _guard = guard;
            let outcome = job.await;
            drop(installing);
            let Ok((result, data)) = outcome else {
                return Err(InstallFailure::Cancelled);
            };
            this.update(cx, |this, cx| {
                if this.slot.accept(ticket, data) {
                    cx.emit(ModsEvent::Changed);
                }
                if let Ok(installed) = &result {
                    cx.emit(ModsEvent::Installed {
                        ids: vec![installed.id.clone()],
                    });
                    this.rewatch(cx);
                }
                cx.notify();
            })
            .ok();
            result
        })
    }

    pub fn install_auto_named(
        &mut self,
        source: impl Into<PathBuf>,
        base_name: impl Into<String>,
        progress: Option<ProgressSender>,
        cx: &mut Context<Self>,
    ) -> Task<InstallResult> {
        let source = source.into();
        let base_name = base_name.into();
        self.install_with(progress, cx, move |staging, game, progress, cancel| {
            yp_mods::bulk::install_auto_named(staging, game, &source, &base_name, progress, cancel)
        })
    }

    pub fn install_archive(
        &mut self,
        archive: impl Into<PathBuf>,
        requested_name: impl Into<String>,
        progress: Option<ProgressSender>,
        cx: &mut Context<Self>,
    ) -> Task<InstallResult> {
        let archive = archive.into();
        let requested_name = requested_name.into();
        self.install_with(progress, cx, move |staging, game, progress, cancel| {
            yp_mods::bulk::install_archive(
                staging,
                game,
                &archive,
                &requested_name,
                progress,
                cancel,
            )
        })
    }

    fn install_with(
        &mut self,
        progress: Option<ProgressSender>,
        cx: &mut Context<Self>,
        work: impl FnOnce(&DropStaging, &GamePaths, &dyn ProgressSink, &CancelToken) -> InstallResult
        + Send
        + 'static,
    ) -> Task<InstallResult> {
        let Some(game) = self.slot.game.clone() else {
            return Task::ready(Err(InstallFailure::Cancelled));
        };
        let env = self.env.clone();
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let installing = self.installing.enter();
        let ticket = self.slot.issue();
        cx.notify();
        let job = self.slot.cell.spawn(move |data| {
            let result = work(&env.staging(), &game, &relay(progress), &cancel);
            if result.is_ok()
                && let Err(error) = data.reload(&game)
            {
                log::warn!("could not list the installed mods: {error}");
            }
            (result, data.clone())
        });
        cx.spawn(async move |this, cx| {
            let _guard = guard;
            let outcome = job.await;
            drop(installing);
            let Ok((result, data)) = outcome else {
                return Err(InstallFailure::Cancelled);
            };
            this.update(cx, |this, cx| {
                if this.slot.accept(ticket, data) {
                    cx.emit(ModsEvent::Changed);
                }
                if let Ok(installed) = &result {
                    cx.emit(ModsEvent::Installed {
                        ids: vec![installed.id.clone()],
                    });
                    this.rewatch(cx);
                }
                cx.notify();
            })
            .ok();
            result
        })
    }

    pub fn install_variants(
        &mut self,
        source: impl Into<PathBuf>,
        requests: Vec<VariantInstallRequest>,
        progress: Option<ProgressSender>,
        cx: &mut Context<Self>,
    ) -> Task<Vec<InstallResult>> {
        let Some(game) = self.slot.game.clone() else {
            return Task::ready(
                requests
                    .iter()
                    .map(|_| Err(InstallFailure::Cancelled))
                    .collect(),
            );
        };
        let source = source.into();
        let env = self.env.clone();
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let installing = self.installing.enter();
        let ticket = self.slot.issue();
        let count = requests.len();
        cx.notify();
        let job = self.slot.cell.spawn(move |data| {
            let results = yp_mods::install::install_variants(
                &env.staging(),
                &game,
                &source,
                &requests,
                &relay(progress),
                &cancel,
            );
            if results.iter().any(Result::is_ok)
                && let Err(error) = data.reload(&game)
            {
                log::warn!("could not list the installed mods: {error}");
            }
            (results, data.clone())
        });
        cx.spawn(async move |this, cx| {
            let _guard = guard;
            let outcome = job.await;
            drop(installing);
            let Ok((results, data)) = outcome else {
                return (0..count).map(|_| Err(InstallFailure::Cancelled)).collect();
            };
            this.update(cx, |this, cx| {
                if this.slot.accept(ticket, data) {
                    cx.emit(ModsEvent::Changed);
                }
                let ids: Vec<String> = results
                    .iter()
                    .filter_map(|result| result.as_ref().ok())
                    .map(|installed| installed.id.clone())
                    .collect();
                if !ids.is_empty() {
                    cx.emit(ModsEvent::Installed { ids });
                    this.rewatch(cx);
                }
                cx.notify();
            })
            .ok();
            results
        })
    }

    pub fn uninstall(
        &mut self,
        mod_id: &str,
        delete_bundled_textures: bool,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let id = mod_id.to_owned();
        let removed = id.clone();
        let uninstall = slot_run(self, cx, false, move |data, game| {
            yp_mods::uninstall::uninstall(game, &id, delete_bundled_textures)?;
            data.reload(game)
        });
        cx.spawn(async move |this, cx| {
            let succeeded = uninstall.await;
            if succeeded {
                this.update(cx, |_, cx| cx.emit(ModsEvent::Uninstalled { id: removed }))
                    .ok();
            }
            succeeded
        })
    }

    fn start_watching(&mut self, cx: &mut Context<Self>) {
        if !self.env.watch_files {
            return;
        }
        let Some(game) = &self.slot.game else {
            return;
        };
        let mut targets: Vec<(PathBuf, Recursive)> = Vec::new();
        for target in ModsWorkspace::watch_paths(game) {
            let watched = if target.recursive {
                Some((target.path, Recursive::YES))
            } else {
                target
                    .path
                    .parent()
                    .map(|dir| (dir.to_path_buf(), Recursive::NO))
            };
            if let Some(watched) = watched
                && !targets.contains(&watched)
            {
                targets.push(watched);
            }
        }
        self.watch = Some(Watch::start(targets, cx, |this, paths, cx| {
            this.on_external_change(&paths, cx)
        }));
    }

    pub(crate) fn rewatch(&mut self, cx: &mut Context<Self>) {
        self.watch = None;
        self.start_watching(cx);
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        cx.emit(ModsEvent::ExternalChange(paths.into()));
        let Some(game) = &self.slot.game else {
            return;
        };
        let mods_dir = game.mods_dir();
        if !paths.iter().any(|path| path.starts_with(&mods_dir)) {
            return;
        }
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DEBOUNCE).await;
            let reload = this.update(cx, |this, cx| {
                (!this.is_installing()).then(|| this.reload(cx))
            });
            if let Ok(Some(reload)) = reload {
                reload.await;
            }
        }));
    }
}

#[derive(Debug, Clone)]
pub enum ProfilesEvent {
    Changed,
    Switched { name: String },
    Created { name: String },
    Deleted { name: String },
    Renamed { from: String, to: String },
    Failed(Arc<ModsError>),
}

pub struct ModProfilesStore {
    slot: DataSlot<ModProfileState>,
}

impl EventEmitter<ProfilesEvent> for ModProfilesStore {}

impl SlotStore for ModProfilesStore {
    type Data = ModProfileState;

    fn slot(&mut self) -> &mut DataSlot<ModProfileState> {
        &mut self.slot
    }

    fn refresh(data: &mut ModProfileState, game: &GamePaths) -> Result<(), ModsError> {
        *data = yp_mods::profiles::load(game)?;
        Ok(())
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(ProfilesEvent::Changed);
    }

    fn failed(&mut self, error: Arc<ModsError>, cx: &mut Context<Self>) {
        cx.emit(ProfilesEvent::Failed(error));
    }
}

impl ModProfilesStore {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            slot: DataSlot::new(cx),
        }
    }

    pub fn state(&self) -> &ModProfileState {
        &self.slot.data
    }

    pub fn profiles(&self) -> &[ModProfile] {
        &self.slot.data.profiles
    }

    pub fn active_name(&self) -> &str {
        &self.slot.data.active_name
    }

    pub fn active(&self) -> Option<&ModProfile> {
        self.slot.data.active()
    }

    pub fn is_loading(&self) -> bool {
        self.slot.loading.is_busy()
    }

    pub fn error(&self) -> Option<&Arc<ModsError>> {
        self.slot.error.as_ref()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        slot_bind(self, game, cx);
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        slot_load(self, cx);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) -> Task<bool> {
        slot_reload(self, cx)
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let Some(game) = &self.slot.game else {
            return;
        };
        if touches(paths, &game.profiles_toml()) {
            self.load(cx);
        }
    }

    fn reload_dependents(cx: &mut App) -> Vec<Task<bool>> {
        let Some(stores) = cx.try_global::<Stores>().cloned() else {
            return Vec::new();
        };
        vec![
            stores.mods.update(cx, |store, cx| store.reload(cx)),
            stores
                .disabled_mods
                .update(cx, |store, cx| store.reload(cx)),
            stores.default_mods.update(cx, |store, cx| store.reload(cx)),
            stores.load_order.update(cx, |store, cx| store.reload(cx)),
        ]
    }

    fn change(
        &mut self,
        cx: &mut Context<Self>,
        reload_dependents: bool,
        event: ProfilesEvent,
        work: impl FnOnce(&ModProfiles<'_>) -> Result<ModProfileState, ModsError> + Send + 'static,
    ) -> Task<bool> {
        self.slot.error = None;
        let change = slot_run(self, cx, true, move |data, game| {
            *data = work(&ModProfiles::new(game))?;
            Ok(())
        });
        cx.spawn(async move |this, cx| {
            if !change.await {
                return false;
            }
            if reload_dependents {
                let reloads = cx.update(Self::reload_dependents);
                for reload in reloads {
                    reload.await;
                }
            }
            this.update(cx, |_, cx| cx.emit(event)).ok();
            true
        })
    }

    pub fn switch_to(&mut self, name: &str, cx: &mut Context<Self>) -> Task<bool> {
        if self.slot.data.active_name == name {
            return Task::ready(true);
        }
        let name = name.to_owned();
        let event = ProfilesEvent::Switched { name: name.clone() };
        self.change(cx, true, event, move |profiles| profiles.switch_to(&name))
    }

    pub fn create(&mut self, name: &str, cx: &mut Context<Self>) -> Task<bool> {
        let name = name.to_owned();
        let event = ProfilesEvent::Created { name: name.clone() };
        self.change(cx, true, event, move |profiles| profiles.create(&name))
    }

    pub fn delete(&mut self, name: &str, cx: &mut Context<Self>) -> Task<bool> {
        let name = name.to_owned();
        let event = ProfilesEvent::Deleted { name: name.clone() };
        self.change(cx, false, event, move |profiles| profiles.delete(&name))
    }

    pub fn rename(&mut self, old_name: &str, new_name: &str, cx: &mut Context<Self>) -> Task<bool> {
        let (from, to) = (old_name.to_owned(), new_name.to_owned());
        let event = ProfilesEvent::Renamed {
            from: from.clone(),
            to: to.clone(),
        };
        self.change(cx, false, event, move |profiles| {
            profiles.rename_profile(&from, &to)
        })
    }
}

#[derive(Debug, Clone)]
pub enum ModDataEvent {
    Changed,
    Failed(Arc<ModsError>),
}

pub struct DisabledModsStore {
    slot: DataSlot<DisabledModsData>,
}

impl EventEmitter<ModDataEvent> for DisabledModsStore {}

impl SlotStore for DisabledModsStore {
    type Data = DisabledModsData;

    fn slot(&mut self) -> &mut DataSlot<DisabledModsData> {
        &mut self.slot
    }

    fn refresh(data: &mut DisabledModsData, game: &GamePaths) -> Result<(), ModsError> {
        data.reload(game)
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Changed);
    }

    fn failed(&mut self, error: Arc<ModsError>, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Failed(error));
    }
}

impl DisabledModsStore {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            slot: DataSlot::new(cx),
        }
    }

    pub fn entries(&self) -> &[String] {
        &self.slot.data.entries
    }

    pub fn is_disabled(&self, rel_path: &str) -> bool {
        self.slot.data.is_disabled(rel_path)
    }

    pub fn is_loading(&self) -> bool {
        self.slot.loading.is_busy()
    }

    pub fn error(&self) -> Option<&Arc<ModsError>> {
        self.slot.error.as_ref()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        slot_bind(self, game, cx);
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        slot_load(self, cx);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) -> Task<bool> {
        slot_reload(self, cx)
    }

    pub fn set_disabled(&mut self, rel_path: &str, disabled: bool, cx: &mut Context<Self>) {
        let rel_path = rel_path.to_owned();
        slot_run(self, cx, false, move |data, game| {
            data.set_disabled(game, &rel_path, disabled)
        })
        .detach();
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let Some(game) = &self.slot.game else {
            return;
        };
        if touches(paths, &game.disabled_mods_toml()) {
            self.load(cx);
        }
    }
}

pub struct DefaultModsStore {
    slot: DataSlot<DefaultModsData>,
}

impl EventEmitter<ModDataEvent> for DefaultModsStore {}

impl SlotStore for DefaultModsStore {
    type Data = DefaultModsData;

    fn slot(&mut self) -> &mut DataSlot<DefaultModsData> {
        &mut self.slot
    }

    fn refresh(data: &mut DefaultModsData, game: &GamePaths) -> Result<(), ModsError> {
        data.reload(game)
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Changed);
    }

    fn failed(&mut self, error: Arc<ModsError>, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Failed(error));
    }
}

impl DefaultModsStore {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            slot: DataSlot::new(cx),
        }
    }

    pub fn entries(&self) -> &[DefaultEntry] {
        &self.slot.data.entries
    }

    pub fn enabled(&self) -> bool {
        self.slot.data.default_outfits_enabled
    }

    pub fn is_default(&self, rel_path: &str) -> bool {
        self.slot.data.is_default(rel_path)
    }

    pub fn kind_of(&self, rel_path: &str) -> Option<DefaultKind> {
        self.slot.data.kind_of(rel_path)
    }

    pub fn outfit_id_of(&self, rel_path: &str) -> Option<i64> {
        self.slot.data.outfit_id_of(rel_path)
    }

    pub fn would_replace(&self, rel_path: &str, kind: DefaultKind) -> Option<&DefaultEntry> {
        self.slot.data.would_replace(rel_path, kind)
    }

    pub fn entries_under(&self, rel_path: &str) -> Vec<&DefaultEntry> {
        self.slot.data.entries_under(rel_path)
    }

    pub fn is_loading(&self) -> bool {
        self.slot.loading.is_busy()
    }

    pub fn error(&self) -> Option<&Arc<ModsError>> {
        self.slot.error.as_ref()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        slot_bind(self, game, cx);
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        slot_load(self, cx);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) -> Task<bool> {
        slot_reload(self, cx)
    }

    pub fn set_default(
        &mut self,
        rel_path: &str,
        kind: Option<DefaultKind>,
        outfit_id: Option<i64>,
        cx: &mut Context<Self>,
    ) {
        let rel_path = rel_path.to_owned();
        slot_run(self, cx, false, move |data, game| {
            data.set_default(game, &rel_path, kind, outfit_id)
        })
        .detach();
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let Some(game) = &self.slot.game else {
            return;
        };
        if touches(paths, &game.default_mods_toml()) || touches(paths, &game.nams_toml()) {
            self.load(cx);
        }
    }
}

pub struct ModLoadOrderStore {
    slot: DataSlot<ModLoadOrderData>,
}

impl EventEmitter<ModDataEvent> for ModLoadOrderStore {}

impl SlotStore for ModLoadOrderStore {
    type Data = ModLoadOrderData;

    fn slot(&mut self) -> &mut DataSlot<ModLoadOrderData> {
        &mut self.slot
    }

    fn refresh(data: &mut ModLoadOrderData, game: &GamePaths) -> Result<(), ModsError> {
        data.reload(game)
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Changed);
    }

    fn failed(&mut self, error: Arc<ModsError>, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Failed(error));
    }
}

impl ModLoadOrderStore {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            slot: DataSlot::new(cx),
        }
    }

    pub fn order(&self) -> &[String] {
        &self.slot.data.order
    }

    pub fn index_of(&self, rel_path: &str) -> Option<usize> {
        self.slot.data.index_of(rel_path)
    }

    pub fn is_loading(&self) -> bool {
        self.slot.loading.is_busy()
    }

    pub fn error(&self) -> Option<&Arc<ModsError>> {
        self.slot.error.as_ref()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        slot_bind(self, game, cx);
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        slot_load(self, cx);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) -> Task<bool> {
        slot_reload(self, cx)
    }

    pub fn save(&mut self, order: Vec<String>, cx: &mut Context<Self>) {
        self.slot.data.order = order.clone();
        cx.notify();
        slot_run(self, cx, false, move |data, game| data.save(game, &order)).detach();
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let Some(game) = &self.slot.game else {
            return;
        };
        if touches(paths, &game.mod_load_order_toml()) {
            self.load(cx);
        }
    }
}

pub struct ModNamesStore {
    slot: DataSlot<ModNamesData>,
}

impl EventEmitter<ModDataEvent> for ModNamesStore {}

impl SlotStore for ModNamesStore {
    type Data = ModNamesData;

    fn slot(&mut self) -> &mut DataSlot<ModNamesData> {
        &mut self.slot
    }

    fn refresh(data: &mut ModNamesData, game: &GamePaths) -> Result<(), ModsError> {
        data.reload(game);
        Ok(())
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Changed);
    }

    fn failed(&mut self, error: Arc<ModsError>, cx: &mut Context<Self>) {
        cx.emit(ModDataEvent::Failed(error));
    }
}

impl ModNamesStore {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            slot: DataSlot::new(cx),
        }
    }

    pub fn custom_name_of(&self, mod_id: &str) -> Option<&str> {
        self.slot.data.custom_name_of(mod_id)
    }

    pub fn names(&self) -> impl Iterator<Item = (&str, &str)> {
        self.slot
            .data
            .names
            .iter()
            .map(|(id, name)| (id.as_str(), name.as_str()))
    }

    pub fn is_loading(&self) -> bool {
        self.slot.loading.is_busy()
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        slot_bind(self, game, cx);
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        slot_load(self, cx);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) -> Task<bool> {
        slot_reload(self, cx)
    }

    pub fn rename(&mut self, mod_id: &str, name: Option<&str>, cx: &mut Context<Self>) {
        let mod_id = mod_id.to_owned();
        let name = name.map(str::to_owned);
        slot_run(self, cx, false, move |data, game| {
            data.rename(game, &mod_id, name.as_deref());
            Ok(())
        })
        .detach();
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let Some(game) = &self.slot.game else {
            return;
        };
        if touches(paths, &game.mod_names_json()) {
            self.load(cx);
        }
    }
}
