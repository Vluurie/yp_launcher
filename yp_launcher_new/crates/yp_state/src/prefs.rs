use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use futures::StreamExt;
use futures::channel::{mpsc, oneshot};
use gpui::AppContext as _;
use gpui::{BackgroundExecutor, Global};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use yp_core::{HostEnv, SettingKey, SettingsStore};
use yp_i18n::Locale;
use yp_ui::{GlobalTheme, I18n, ThemeId};

pub use yp_core::window_geometry::WINDOW;
pub use yp_health::DETECTED_NOTIFICATIONS_DIR;
pub use yp_host::gpu::PREFER_DEDICATED_GPU;
pub use yp_host::launch::wrapper::LAUNCH_WRAPPER;
pub use yp_host::launch::NAMS_DLL_PATH;
pub use yp_host::session::NIER_DIRECTORY;
pub use yp_host::win10_compat::WIN10_SETUP_DONE;
pub use yp_i18n::APP_LOCALE;

pub const APP_THEME: SettingKey<String> = SettingKey::new("app_theme", String::new);
pub const ONBOARDING_COMPLETE: SettingKey<bool> = SettingKey::new("onboarding_complete", || false);
pub const MINIMIZE_ON_LAUNCH: SettingKey<bool> = SettingKey::new("minimize_on_launch", || false);
pub const OUTFIT_GUIDE_SHOWN: SettingKey<bool> = SettingKey::new("outfit_guide_shown", || false);
pub const SIDEBAR_SECTION_COLLAPSED: SettingKey<BTreeMap<String, bool>> =
    SettingKey::new("sidebar_section_collapsed", BTreeMap::new);

type Write = Box<dyn FnOnce(&SettingsStore) + Send>;

fn null() -> Value {
    Value::Null
}

struct Unsaved {
    serial: u64,
    value: Option<Value>,
}

pub struct Prefs {
    settings: Arc<SettingsStore>,
    unsaved: RefCell<HashMap<&'static str, Unsaved>>,
    serial: Cell<u64>,
    writes: mpsc::UnboundedSender<Write>,
    saved: mpsc::UnboundedSender<(&'static str, u64)>,
    settled: RefCell<mpsc::UnboundedReceiver<(&'static str, u64)>>,
}

impl Global for Prefs {}

impl Prefs {
    pub fn new(settings: Arc<SettingsStore>, executor: &BackgroundExecutor) -> Self {
        let (writes, mut pending) = mpsc::unbounded::<Write>();
        let (saved, settled) = mpsc::unbounded();
        let store = settings.clone();
        executor
            .spawn(async move {
                while let Some(write) = pending.next().await {
                    write(&store);
                }
            })
            .detach();
        Self {
            settings,
            unsaved: RefCell::new(HashMap::new()),
            serial: Cell::new(0),
            writes,
            saved,
            settled: RefCell::new(settled),
        }
    }

    pub fn settings(&self) -> &Arc<SettingsStore> {
        &self.settings
    }

    fn forget_saved(&self) {
        let mut settled = self.settled.borrow_mut();
        let mut unsaved = self.unsaved.borrow_mut();
        while let Ok((name, serial)) = settled.try_recv() {
            if unsaved
                .get(name)
                .is_some_and(|entry| entry.serial == serial)
            {
                unsaved.remove(name);
            }
        }
    }

    fn remember(&self, name: &'static str, value: Option<Value>) -> u64 {
        let serial = self.serial.get() + 1;
        self.serial.set(serial);
        self.unsaved
            .borrow_mut()
            .insert(name, Unsaved { serial, value });
        serial
    }

    pub fn get<T: DeserializeOwned>(&self, key: &SettingKey<T>) -> T {
        self.forget_saved();
        match self.unsaved.borrow().get(key.name()) {
            Some(Unsaved {
                value: Some(value), ..
            }) => serde_json::from_value(value.clone()).unwrap_or_else(|_| key.default_value()),
            Some(Unsaved { value: None, .. }) => key.default_value(),
            None => self.settings.get(key),
        }
    }

    pub fn set<T: Serialize>(&self, key: &SettingKey<T>, value: &T) {
        let _ = self.set_with_result(key, value);
    }

    pub fn set_with_result<T: Serialize>(
        &self,
        key: &SettingKey<T>,
        value: &T,
    ) -> oneshot::Receiver<Result<(), String>> {
        let (done, result) = oneshot::channel();
        let name = key.name();
        let json = match serde_json::to_value(value) {
            Ok(json) => json,
            Err(err) => {
                log::warn!("could not encode setting {name}: {err}");
                let _ = done.send(Err(err.to_string()));
                return result;
            }
        };
        self.forget_saved();
        let serial = self.remember(name, Some(json.clone()));
        let saved = self.saved.clone();
        self.enqueue(Box::new(move |settings| {
            let result = settings
                .set(&SettingKey::<Value>::new(name, null), &json)
                .map_err(|error| error.to_string());
            if let Err(err) = &result {
                log::warn!("could not save setting {name}: {err}");
            }
            let _ = saved.unbounded_send((name, serial));
            let _ = done.send(result);
        }));
        result
    }

    pub fn update<T: Serialize + DeserializeOwned>(
        &self,
        key: &SettingKey<T>,
        change: impl FnOnce(&mut T),
    ) {
        let mut value = self.get(key);
        change(&mut value);
        self.set(key, &value);
    }

    pub fn remove<T>(&self, key: &SettingKey<T>) {
        let name = key.name();
        let serial = self.remember(name, None);
        let saved = self.saved.clone();
        self.enqueue(Box::new(move |settings| {
            if let Err(err) = settings.remove(&SettingKey::<Value>::new(name, null)) {
                log::warn!("could not remove setting {name}: {err}");
            }
            let _ = saved.unbounded_send((name, serial));
        }));
    }

    pub fn flush(&self) -> oneshot::Receiver<()> {
        let (done, flushed) = oneshot::channel();
        self.enqueue(Box::new(move |_| {
            let _ = done.send(());
        }));
        flushed
    }

    pub fn saved_locale(&self) -> Option<Locale> {
        Some(self.get(&APP_LOCALE))
            .filter(|code| !code.is_empty())
            .and_then(|code| Locale::from_code(&code))
    }

    pub fn locale(&self, host: &HostEnv) -> Locale {
        let saved = Some(self.get(&APP_LOCALE)).filter(|code| !code.is_empty());
        yp_i18n::locale::resolve_locale(saved.as_deref(), host.system_language().as_deref())
    }

    pub fn set_locale(&self, locale: Locale) {
        self.set(&APP_LOCALE, &locale.code().to_owned());
    }

    pub fn saved_theme(&self) -> Option<String> {
        Some(self.get(&APP_THEME)).filter(|name| !name.is_empty())
    }

    pub fn set_theme(&self, name: &str) {
        self.set(&APP_THEME, &name.to_owned());
    }

    pub fn onboarding_complete(&self) -> bool {
        self.get(&ONBOARDING_COMPLETE)
    }

    pub fn set_onboarding_complete(&self, complete: bool) {
        self.set(&ONBOARDING_COMPLETE, &complete);
    }

    fn enqueue(&self, write: Write) {
        if self.writes.unbounded_send(write).is_err() {
            log::warn!("settings writer is gone");
        }
    }
}

pub(crate) fn save_setting<T: Serialize>(
    settings: &Arc<SettingsStore>,
    key: &SettingKey<T>,
    value: &T,
    cx: &gpui::App,
) {
    if let Some(prefs) = cx.try_global::<Prefs>() {
        prefs.set(key, value);
        return;
    }
    let name = key.name();
    let Ok(json) = serde_json::to_value(value) else {
        return;
    };
    let settings = settings.clone();
    cx.background_spawn(async move {
        if let Err(err) = settings.set(&SettingKey::<Value>::new(name, null), &json) {
            log::warn!("could not save setting {name}: {err}");
        }
    })
    .detach();
}

fn saved_theme_id(prefs: &Prefs) -> ThemeId {
    prefs
        .saved_theme()
        .and_then(|code| ThemeId::from_code(&code))
        .unwrap_or_default()
}

pub(crate) fn bind_ui(host: &HostEnv, cx: &mut gpui::App) {
    let prefs = cx.global::<Prefs>();
    let theme = saved_theme_id(prefs);
    let locale = prefs.locale(host);
    if cx.try_global::<GlobalTheme>().map(|global| global.0.id) != Some(theme) {
        yp_ui::theme::set_theme(theme, cx);
    }
    I18n::set(locale, cx);

    cx.observe_global::<GlobalTheme>(|cx| {
        let active = cx.global::<GlobalTheme>().0.id;
        let prefs = cx.global::<Prefs>();
        if saved_theme_id(prefs) != active {
            prefs.set_theme(active.code());
        }
    })
    .detach();

    let host = host.clone();
    cx.observe_global::<I18n>(move |cx| {
        let active = I18n::locale(cx);
        let prefs = cx.global::<Prefs>();
        if prefs.locale(&host) != active {
            prefs.set_locale(active);
        }
    })
    .detach();
}
