use futures::StreamExt;
use futures::channel::mpsc;
use gpui::{Context, EventEmitter, SharedString, Subscription, Task};
use yp_core::CancelToken;
use yp_cutscenes::{CutsceneMod, ScanDirs, ScanProgress, ScanResult};
use yp_state::prelude::*;
use yp_state::{BusyHandle, DetectionEvent, SessionEvent, Stores};

const CUTSCENES_PREFIX: &str = "cutscenes";
const MODS_PREFIX: &str = "mods";

pub fn disabled_rel_path(cutscene: &CutsceneMod) -> String {
    match &cutscene.bundled_with_mod_id {
        Some(mod_id) => format!(
            "{MODS_PREFIX}/{mod_id}/{CUTSCENES_PREFIX}/{}",
            cutscene.name
        ),
        None => format!("{CUTSCENES_PREFIX}/{}", cutscene.name),
    }
}

pub fn merge_targets(mods: &[CutsceneMod]) -> Vec<String> {
    mods.iter()
        .filter(|cutscene| !cutscene.is_bundled())
        .map(|cutscene| cutscene.name.clone())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    H264,
    Mpeg2,
}

pub fn codec_of(cutscene: &CutsceneMod) -> Option<Codec> {
    if cutscene.has_h264() {
        Some(Codec::H264)
    } else if cutscene.usm_count() > 0 {
        Some(Codec::Mpeg2)
    } else {
        None
    }
}

pub fn resolution_of(cutscene: &CutsceneMod) -> Option<(u32, u32)> {
    (cutscene.max_width() > 0).then(|| (cutscene.max_width(), cutscene.max_height()))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    pub text: SharedString,
    pub percent: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutscenesEvent {
    Scanned,
    ScanFailed,
}

pub struct CutscenesModel {
    stores: Stores,
    scan: ScanResult,
    scanned_once: bool,
    loading: bool,
    scan_progress: Option<ScanProgress>,
    activity: Option<Activity>,
    busy: Option<BusyHandle>,
    cancel: CancelToken,
    scan_task: Option<Task<()>>,
    progress_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<CutscenesEvent> for CutscenesModel {}

impl CutscenesModel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let subscriptions = vec![
            cx.subscribe(&stores.session, |this, _, event: &SessionEvent, cx| {
                if matches!(event, SessionEvent::DirectoryChanged { .. }) {
                    this.forget(cx);
                    this.reload(cx);
                }
            }),
            cx.subscribe(&stores.detection, |this, _, event: &DetectionEvent, cx| {
                if matches!(event, DetectionEvent::Invalidated { .. }) {
                    this.reload(cx);
                }
            }),
        ];
        let mut model = Self {
            stores,
            scan: ScanResult::default(),
            scanned_once: false,
            loading: false,
            scan_progress: None,
            activity: None,
            busy: None,
            cancel: CancelToken::new(),
            scan_task: None,
            progress_task: None,
            _subscriptions: subscriptions,
        };
        model.reload(cx);
        model
    }

    pub fn mods(&self) -> &[CutsceneMod] {
        &self.scan.mods
    }

    pub fn direct_overrides(&self) -> &[String] {
        &self.scan.direct_overrides
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn has_scanned(&self) -> bool {
        self.scanned_once
    }

    pub fn scan_progress(&self) -> Option<ScanProgress> {
        self.scan_progress
    }

    pub fn activity(&self) -> Option<&Activity> {
        self.activity.as_ref()
    }

    pub fn is_installing(&self) -> bool {
        self.activity.is_some()
    }

    pub fn set_activity(
        &mut self,
        text: impl Into<SharedString>,
        percent: Option<f32>,
        cx: &mut Context<Self>,
    ) {
        if self.busy.is_none() {
            let tracker = self.stores.busy.clone();
            self.busy =
                Some(tracker.update(cx, |tracker, cx| tracker.enter(BusyScope::General, cx)));
        }
        self.activity = Some(Activity {
            text: text.into(),
            percent,
        });
        cx.notify();
    }

    pub fn clear_activity(&mut self, cx: &mut Context<Self>) {
        self.activity = None;
        self.busy = None;
        cx.notify();
    }

    fn forget(&mut self, cx: &mut Context<Self>) {
        self.scan = ScanResult::default();
        self.scanned_once = false;
        cx.notify();
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(game) = self.stores.session.read(cx).game() else {
            return;
        };
        self.cancel.cancel();
        let cancel = CancelToken::new();
        self.cancel = cancel.clone();
        if self.scan.mods.is_empty() && self.scan.direct_overrides.is_empty() {
            self.loading = true;
        }
        self.scan_progress = None;
        cx.notify();

        let (sender, mut updates) = mpsc::unbounded::<ScanProgress>();
        self.progress_task = Some(cx.spawn(async move |this, cx| {
            while let Some(progress) = updates.next().await {
                let updated = this.update(cx, |this, cx| {
                    this.scan_progress = Some(progress);
                    cx.notify();
                });
                if updated.is_err() {
                    break;
                }
            }
        }));
        let scanning = cx.background_executor().spawn(async move {
            let observe = move |progress: &ScanProgress| {
                let _ = sender.unbounded_send(*progress);
            };
            yp_cutscenes::scan(&ScanDirs::for_game(&game), &observe, &cancel)
        });
        self.scan_task = Some(cx.spawn(async move |this, cx| {
            let result = scanning.await;
            this.update(cx, |this, cx| {
                this.loading = false;
                this.scan_progress = None;
                this.progress_task = None;
                match result {
                    Ok(scan) => {
                        this.scan = scan;
                        this.scanned_once = true;
                        let config = this.stores.config.clone();
                        config.update(cx, |config, cx| config.auto_detect_cutscenes(cx));
                        cx.emit(CutscenesEvent::Scanned);
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        log::warn!("cutscene scan failed: {error}");
                        this.scanned_once = true;
                        cx.emit(CutscenesEvent::ScanFailed);
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cutscene(name: &str, bundled: Option<&str>) -> CutsceneMod {
        CutsceneMod {
            name: name.to_owned(),
            path: PathBuf::from(name),
            files: Vec::new(),
            bundled_with_mod_id: bundled.map(str::to_owned),
        }
    }

    #[test]
    fn disabled_paths_point_into_the_owning_mod_for_bundled_cutscenes() {
        assert_eq!(disabled_rel_path(&cutscene("HD", None)), "cutscenes/HD");
        assert_eq!(
            disabled_rel_path(&cutscene("Intro", Some("kimono"))),
            "mods/kimono/cutscenes/Intro"
        );
    }

    #[test]
    fn only_standalone_cutscene_mods_can_be_merged_into() {
        let mods = [
            cutscene("HD", None),
            cutscene("Intro", Some("kimono")),
            cutscene("Upscale", None),
        ];
        assert_eq!(merge_targets(&mods), ["HD", "Upscale"]);
    }

    #[test]
    fn empty_cutscene_mods_have_no_codec_or_resolution() {
        let empty = cutscene("HD", None);
        assert_eq!(codec_of(&empty), None);
        assert_eq!(resolution_of(&empty), None);
    }
}
