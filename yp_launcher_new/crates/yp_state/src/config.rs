use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use gpui::AppContext as _;
use gpui::{Context, EventEmitter, Subscription, Task};
use indexmap::IndexMap;
use serde_json::Value as Json;
use yp_config::cli::{self, NamsCli};
use yp_config::{Change, ConfigError, ResetTarget, Schema, SetReport, Setting, Values, values_equal};
use yp_core::watch::Recursive;
use yp_core::{CancelToken, GamePaths};
use yp_host::nams_cli::HostNamsCli;
use yp_ui::I18n;

use crate::cell::CancelOnDrop;
use crate::env::AppEnv;
use crate::watch::{RELOAD_DEBOUNCE, Watch};

pub const WRITE_DELAY: Duration = Duration::from_millis(150);
pub const H264_ID: &str = "nams.cutscene.enable_h264";
pub const TEXTURE_LOAD_ORDER_ID: &str = "texture_injection.load_order";
pub const DEFAULT_OUTFITS_ID: &str = "nams.experimental_default_outfits";
pub const OVERLAY_LANGUAGE_ID: &str = "plugins.yorha_protocol.overlayLanguage";
const INTERNAL_DIR: &str = "_internal";
const CACHE_DIR: &str = "cache";

pub type CliFactory = Arc<dyn Fn(&GamePaths) -> Arc<dyn NamsCli> + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigOp {
    Load,
    Write,
    CutsceneFlags,
    ExternalReload,
}

#[derive(Debug, Clone)]
pub struct ConfigFailure {
    pub op: ConfigOp,
    pub error: Arc<ConfigError>,
}

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Loaded,
    Written(Arc<SetReport>),
    ReloadedExternally,
    CutsceneFlagsApplied(Vec<Change>),
    Failed(ConfigFailure),
}

pub struct ConfigStore {
    factory: CliFactory,
    watch_files: bool,
    game: Option<GamePaths>,
    cli: Option<Arc<dyn NamsCli>>,
    lang: String,
    schema: Arc<Schema>,
    values: Values,
    pending: IndexMap<String, Json>,
    in_flight: Option<IndexMap<String, Json>>,
    generation: u64,
    migrated: bool,
    loading: bool,
    loaded: bool,
    failure: Option<ConfigFailure>,
    load_task: Option<Task<()>>,
    delay_task: Option<Task<()>>,
    reload_task: Option<Task<()>>,
    change_task: Option<Task<()>>,
    cutscene_task: Option<Task<()>>,
    watch: Option<Watch>,
    _locale: Subscription,
}

impl EventEmitter<ConfigEvent> for ConfigStore {}

fn host_factory(env: &AppEnv) -> CliFactory {
    let platform = env.platform_adapter();
    let runner = env.runner.clone();
    let paths = env.paths.clone();
    Arc::new(move |game: &GamePaths| {
        Arc::new(HostNamsCli {
            platform: platform.clone(),
            runner: runner.clone(),
            paths: paths.clone(),
            game: game.clone(),
        }) as Arc<dyn NamsCli>
    })
}

impl ConfigStore {
    pub fn new(env: &AppEnv, cx: &mut Context<Self>) -> Self {
        cx.on_app_quit(|this, _| this.flush_on_quit()).detach();
        let locale = cx.observe_global::<I18n>(|this, cx| this.on_locale_changed(cx));
        Self {
            factory: host_factory(env),
            watch_files: env.watch_files,
            game: None,
            cli: None,
            lang: I18n::locale(cx).code().to_owned(),
            schema: Arc::new(Schema::default()),
            values: Values::default(),
            pending: IndexMap::new(),
            in_flight: None,
            generation: 0,
            migrated: false,
            loading: false,
            loaded: false,
            failure: None,
            load_task: None,
            delay_task: None,
            reload_task: None,
            change_task: None,
            cutscene_task: None,
            watch: None,
            _locale: locale,
        }
    }

    pub fn set_cli_factory(&mut self, factory: CliFactory, cx: &mut Context<Self>) {
        self.factory = factory;
        let game = self.game.take();
        self.bind_game(game, cx);
    }

    pub fn game(&self) -> Option<&GamePaths> {
        self.game.as_ref()
    }

    pub fn cli(&self) -> Option<Arc<dyn NamsCli>> {
        self.cli.clone()
    }

    pub fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }

    pub fn values(&self) -> &Values {
        &self.values
    }

    pub fn setting(&self, id: &str) -> Option<&Setting> {
        self.schema.setting(id)
    }

    pub fn value(&self, id: &str) -> Json {
        self.values
            .get(id)
            .cloned()
            .or_else(|| self.setting(id).map(|setting| setting.default.clone()))
            .unwrap_or(Json::Null)
    }

    pub fn bool(&self, id: &str) -> bool {
        self.value(id).as_bool().unwrap_or(false)
    }

    pub fn number(&self, id: &str) -> Option<f64> {
        self.value(id).as_f64()
    }

    pub fn text(&self, id: &str) -> String {
        self.value(id).as_str().unwrap_or_default().to_owned()
    }

    pub fn texts(&self, id: &str) -> Vec<String> {
        self.value(id)
            .as_array()
            .map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_owned)).collect())
            .unwrap_or_default()
    }

    pub fn is_changed(&self, id: &str) -> bool {
        self.setting(id)
            .is_some_and(|setting| !setting.is_default(&self.value(id)))
    }

    pub fn file_error(&self, file: &str) -> Option<&str> {
        self.values.file_error(file)
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn failure(&self) -> Option<&ConfigFailure> {
        self.failure.as_ref()
    }

    pub fn has_pending_writes(&self) -> bool {
        !self.pending.is_empty() || self.in_flight.is_some()
    }

    pub fn lang(&self) -> &str {
        &self.lang
    }

    pub(crate) fn bind_game(&mut self, game: Option<GamePaths>, cx: &mut Context<Self>) {
        if self.game == game && self.cli.is_some() == game.is_some() {
            self.load(cx);
            return;
        }
        self.flush(cx).detach();
        self.generation += 1;
        self.load_task = None;
        self.delay_task = None;
        self.reload_task = None;
        self.change_task = None;
        self.cutscene_task = None;
        self.watch = None;
        self.schema = Arc::new(Schema::default());
        self.values = Values::default();
        self.pending.clear();
        self.in_flight = None;
        self.migrated = false;
        self.loading = false;
        self.loaded = false;
        self.failure = None;
        self.cli = game.as_ref().map(|game| (self.factory)(game));
        self.game = game;
        cx.notify();
        self.load(cx);
    }

    fn fail(&mut self, op: ConfigOp, error: ConfigError, cx: &mut Context<Self>) {
        log::warn!("config {op:?} failed: {error}");
        let failure = ConfigFailure {
            op,
            error: Arc::new(error),
        };
        self.failure = Some(failure.clone());
        cx.emit(ConfigEvent::Failed(failure));
        cx.notify();
    }

    fn apply_values(&mut self, mut values: Values) {
        for (id, value) in self.in_flight.iter().flatten().chain(&self.pending) {
            values.values.insert(id.clone(), value.clone());
        }
        self.values = values;
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        let Some(cli) = self.cli.clone() else {
            return;
        };
        self.loading = true;
        self.failure = None;
        cx.notify();
        let lang = self.lang.clone();
        let generation = self.generation;
        let migrate = !std::mem::replace(&mut self.migrated, true);
        let work = cx.background_spawn(async move {
            if migrate && let Err(error) = cli::migrate(cli.as_ref(), false) {
                log::warn!("could not bring the config files up to date: {error}");
            }
            let schema = cli::schema(cli.as_ref(), Some(&lang))?;
            let values = cli::get(cli.as_ref())?;
            Ok::<_, ConfigError>((schema, values))
        });
        self.load_task = Some(cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.loading = false;
                match result {
                    Ok((schema, values)) => {
                        this.schema = Arc::new(schema);
                        this.apply_values(values);
                        this.loaded = true;
                        cx.emit(ConfigEvent::Loaded);
                        this.start_watching(cx);
                        this.auto_detect_cutscenes(cx);
                    }
                    Err(error) => this.fail(ConfigOp::Load, error, cx),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn on_locale_changed(&mut self, cx: &mut Context<Self>) {
        let lang = I18n::locale(cx).code().to_owned();
        if lang == self.lang {
            return;
        }
        self.lang = lang;
        let Some(cli) = self.cli.clone() else {
            return;
        };
        let lang = self.lang.clone();
        let generation = self.generation;
        let work = cx.background_spawn(async move { cli::schema(cli.as_ref(), Some(&lang)) });
        self.load_task = Some(cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match result {
                    Ok(schema) => this.schema = Arc::new(schema),
                    Err(error) => this.fail(ConfigOp::Load, error, cx),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn set(&mut self, id: &str, value: Json, cx: &mut Context<Self>) {
        if !self.stage(id, value) {
            return;
        }
        cx.notify();
        self.delay_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(WRITE_DELAY).await;
            this.update(cx, |this, cx| {
                this.delay_task = None;
                this.flush(cx).detach();
            })
            .ok();
        }));
    }

    pub fn set_now(&mut self, id: &str, value: Json, cx: &mut Context<Self>) {
        if self.stage(id, value) {
            cx.notify();
        }
        self.delay_task = None;
        self.flush(cx).detach();
    }

    pub fn set_many(&mut self, assignments: Vec<(String, Json)>, cx: &mut Context<Self>) {
        let mut staged = false;
        for (id, value) in assignments {
            staged |= self.stage(&id, value);
        }
        if staged {
            cx.notify();
        }
        self.delay_task = None;
        self.flush(cx).detach();
    }

    fn stage(&mut self, id: &str, value: Json) -> bool {
        if self.cli.is_none() {
            return false;
        }
        let unchanged = self.values.get(id).is_some_and(|current| values_equal(current, &value));
        if unchanged && !self.pending.contains_key(id) && !self.values.not_in_file.contains(id) {
            return false;
        }
        self.values.values.insert(id.to_owned(), value.clone());
        self.pending.insert(id.to_owned(), value);
        true
    }

    pub fn reset(&mut self, target: &ResetTarget, cx: &mut Context<Self>) {
        let ids: Vec<&Setting> = match target {
            ResetTarget::All => self.schema.settings.iter().collect(),
            ResetTarget::Category(category) => self
                .schema
                .settings
                .iter()
                .filter(|setting| yp_config::schema::is_in_category(&setting.category, category))
                .collect(),
            ResetTarget::Ids(ids) => self
                .schema
                .settings
                .iter()
                .filter(|setting| ids.contains(&setting.id))
                .collect(),
        };
        let assignments = ids
            .into_iter()
            .map(|setting| (setting.id.clone(), setting.default.clone()))
            .collect();
        self.set_many(assignments, cx);
    }

    pub fn flush(&mut self, cx: &mut Context<Self>) -> Task<()> {
        if self.in_flight.is_some() || self.pending.is_empty() {
            return Task::ready(());
        }
        let Some(cli) = self.cli.clone() else {
            self.pending.clear();
            return Task::ready(());
        };
        let writing = std::mem::take(&mut self.pending);
        let assignments: Vec<(String, Json)> = writing.clone().into_iter().collect();
        let written: Vec<String> = writing.keys().cloned().collect();
        self.in_flight = Some(writing);
        let generation = self.generation;
        let work = cx.background_spawn(async move { cli::set(cli.as_ref(), &assignments, false) });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.in_flight = None;
                match result {
                    Ok(report) => {
                        for id in written {
                            this.values.not_in_file.remove(&id);
                        }
                        cx.emit(ConfigEvent::Written(Arc::new(report)));
                        if !this.pending.is_empty() {
                            this.flush(cx).detach();
                        }
                    }
                    Err(error) => {
                        this.fail(ConfigOp::Write, error, cx);
                        this.reload_if_clean(cx);
                    }
                }
                cx.notify();
            })
            .ok();
        })
    }

    pub fn flush_pending(&mut self, cx: &mut Context<Self>) -> Task<()> {
        self.delay_task = None;
        self.flush(cx)
    }

    fn flush_on_quit(&mut self) -> impl Future<Output = ()> + use<> {
        let assignments: Vec<(String, Json)> = std::mem::take(&mut self.pending).into_iter().collect();
        let cli = self.cli.clone().filter(|_| !assignments.is_empty());
        let write = cli.map(|cli| {
            let (done, finished) = futures::channel::oneshot::channel();
            std::thread::spawn(move || {
                let _ = done.send(cli::set(cli.as_ref(), &assignments, false).is_ok());
            });
            finished
        });
        async move {
            if let Some(write) = write
                && write.await != Ok(true)
            {
                log::warn!("a config change was not written before quitting");
            }
        }
    }

    pub fn reload_if_clean(&mut self, cx: &mut Context<Self>) {
        if self.has_pending_writes() {
            return;
        }
        let Some(cli) = self.cli.clone() else {
            return;
        };
        let generation = self.generation;
        let work = cx.background_spawn(async move { cli::get(cli.as_ref()) });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match result {
                    Ok(values) => {
                        let changed = values.values != this.values.values || values.files != this.values.files;
                        this.apply_values(values);
                        if changed {
                            cx.emit(ConfigEvent::ReloadedExternally);
                        }
                    }
                    Err(error) => this.fail(ConfigOp::ExternalReload, error, cx),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn seed_overlay_language(&mut self, code: &str, cx: &mut Context<Self>) {
        let Some(setting) = self.setting(OVERLAY_LANGUAGE_ID) else {
            return;
        };
        if !self.values.not_in_file.contains(OVERLAY_LANGUAGE_ID)
            || !setting.choices.iter().any(|choice| choice.value.as_str() == Some(code))
        {
            return;
        }
        self.set_now(OVERLAY_LANGUAGE_ID, Json::String(code.to_owned()), cx);
    }

    pub fn reconcile_texture_load_order(&mut self, folders: Vec<String>, cx: &mut Context<Self>) {
        if !self.loaded {
            return;
        }
        let current = self.texts(TEXTURE_LOAD_ORDER_ID);
        let Some(order) = yp_mods::textures::reconciled_load_order(&current, &folders) else {
            return;
        };
        let value = Json::Array(order.into_iter().map(Json::String).collect());
        self.set_now(TEXTURE_LOAD_ORDER_ID, value, cx);
    }

    pub fn auto_detect_cutscenes(&mut self, cx: &mut Context<Self>) {
        let Some(game) = self.game.clone() else {
            return;
        };
        if self.setting(H264_ID).is_none() {
            return;
        }
        let cancel = CancelToken::new();
        let guard = CancelOnDrop(cancel.clone());
        let scan = cx.background_spawn(async move { yp_cutscenes::detect(&game, &cancel) });
        self.cutscene_task = Some(cx.spawn(async move |this, cx| {
            let _guard = guard;
            let detection = match scan.await {
                Ok(detection) => detection,
                Err(error) => {
                    if !error.is_cancelled() {
                        log::warn!("cutscene detection failed: {error}");
                    }
                    return;
                }
            };
            this.update(cx, |this, cx| {
                let wanted = Json::Bool(detection.needs_h264);
                let current = this.value(H264_ID);
                if values_equal(&current, &wanted) {
                    return;
                }
                this.set_now(H264_ID, wanted.clone(), cx);
                cx.emit(ConfigEvent::CutsceneFlagsApplied(vec![Change {
                    id: H264_ID.to_owned(),
                    old: current,
                    new: wanted,
                    ..Change::default()
                }]));
            })
            .ok();
        }));
    }

    fn watched_dirs(game: &GamePaths) -> Vec<PathBuf> {
        let nams = game.nams_dir();
        vec![nams.join(INTERNAL_DIR).join(CACHE_DIR), nams]
    }

    fn start_watching(&mut self, cx: &mut Context<Self>) {
        if !self.watch_files || self.watch.is_some() {
            return;
        }
        let Some(game) = &self.game else {
            return;
        };
        let targets = Self::watched_dirs(game)
            .into_iter()
            .filter(|dir| dir.is_dir())
            .map(|dir| (dir, Recursive::NO))
            .collect();
        self.watch = Some(Watch::start(targets, cx, |this, paths, cx| {
            this.on_external_change(&paths, cx)
        }));
    }

    fn is_config_file(&self, path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        self.schema.files.iter().any(|file| {
            Path::new(&file.file)
                .file_name()
                .and_then(|known| known.to_str())
                .is_some_and(|known| known.eq_ignore_ascii_case(name))
        })
    }

    pub fn on_external_change(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        if !paths.iter().any(|path| self.is_config_file(path)) {
            return;
        }
        self.change_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DEBOUNCE).await;
            this.update(cx, |this, cx| this.reload_if_clean(cx)).ok();
        }));
    }
}
