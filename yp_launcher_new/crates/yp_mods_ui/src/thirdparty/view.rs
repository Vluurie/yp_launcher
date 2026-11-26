use std::collections::HashSet;
use std::path::PathBuf;

use gpui::{PathPromptOptions, ScrollHandle, Subscription};
use yp_state::prelude::*;
use yp_state::{BusyHandle, NavigationEvent, SessionEvent, Stores, ThirdPartyEvent};
use yp_thirdparty::{
    GameModEntry, GameModSettings, MigotoConfig, MigotoHunting, MigotoMarking, ReShadeConfig,
    RuntimePresence, RuntimeStatus, ThirdPartyRuntime,
};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, Callout, Card, ChipSize, ChipTone, ConfirmOptions, DropProgress,
    DropZone, ListItemSpacing, Msg, PageHeader, ScrollableElement, SelectItem, SettingSelect,
    SettingSwitch, TintColor, Tooltip, TwoColumn, confirm,
};

use crate::layout::shrinkable;
use crate::notify::{reveal, toast};
use crate::thirdparty::flow::{
    DropNext, dll_row_label, drop_next, image_size_label, image_size_options, install_notice,
    lodmod_import_notice, patch_timeout_label, patch_timeout_options, runtime_name,
    stage_failure_notice, status_line, status_text, update_body,
};

const THIRD_PARTY_DOC_PAGE: &str = yp_core::consts::THIRD_PARTY_DOC_PATH;

pub struct ThirdPartyView {
    stores: Stores,
    expanded_game_mods: HashSet<String>,
    activity: Option<SharedString>,
    busy: Option<BusyHandle>,
    pending_import: Option<ThirdPartyRuntime>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl ThirdPartyView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let subscriptions = vec![
            cx.observe(&stores.thirdparty, |_, _, cx| cx.notify()),
            cx.subscribe(
                &stores.thirdparty,
                |this, _, event: &ThirdPartyEvent, cx| match event {
                    ThirdPartyEvent::Changed => this.announce_import(cx),
                    ThirdPartyEvent::Failed(_) => {
                        this.pending_import = None;
                        if this.activity.is_none() {
                            toast(
                                cx,
                                "thirdparty",
                                Msg::ModsUiThirdPartyFailed,
                                Severity::Error,
                            );
                        }
                    }
                    ThirdPartyEvent::Notice(_) => {}
                },
            ),
            cx.subscribe(
                &stores.navigation,
                |this, _, event: &NavigationEvent, cx| {
                    if matches!(
                        event,
                        NavigationEvent::TabChanged {
                            to: TabId::ThirdParty,
                            ..
                        }
                    ) {
                        this.refresh(cx);
                    }
                },
            ),
            cx.subscribe(&stores.session, |this, _, event: &SessionEvent, cx| {
                if matches!(event, SessionEvent::DirectoryChanged { .. }) {
                    this.refresh(cx);
                }
                cx.notify();
            }),
        ];
        let mut view = Self {
            stores,
            expanded_game_mods: HashSet::new(),
            activity: None,
            busy: None,
            pending_import: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.stores
            .thirdparty
            .update(cx, |store, cx| store.refresh(cx));
    }

    pub fn is_expanded(&self, key: &str) -> bool {
        self.expanded_game_mods.contains(key)
    }

    pub fn toggle_expanded(&mut self, key: &str, cx: &mut Context<Self>) {
        if !self.expanded_game_mods.remove(key) {
            self.expanded_game_mods.insert(key.to_owned());
        }
        cx.notify();
    }

    pub fn is_installing(&self) -> bool {
        self.activity.is_some()
    }

    fn announce_import(&mut self, cx: &mut Context<Self>) {
        let Some(which) = self.pending_import else {
            return;
        };
        if self.stores.thirdparty.read(cx).presence(which) == RuntimePresence::Installed {
            self.pending_import = None;
            toast(cx, "thirdparty", Msg::ThirdPartyImported, Severity::Success);
        }
    }

    pub fn import_from_game_root(&mut self, which: ThirdPartyRuntime, cx: &mut Context<Self>) {
        self.pending_import = Some(which);
        self.stores
            .thirdparty
            .update(cx, |store, cx| store.import_from_game_root(which, cx));
    }

    fn set_activity(&mut self, text: Option<SharedString>, cx: &mut Context<Self>) {
        match &text {
            Some(_) if self.busy.is_none() => {
                let tracker = self.stores.busy.clone();
                self.busy =
                    Some(tracker.update(cx, |tracker, cx| tracker.enter(BusyScope::General, cx)));
            }
            None => self.busy = None,
            _ => {}
        }
        self.activity = text;
        cx.notify();
    }

    pub fn install_paths(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if paths.is_empty() || self.is_installing() {
            return;
        }
        let stores = self.stores.clone();
        self.set_activity(Some(cx.t().preparing_install().into()), cx);
        cx.spawn_in(window, async move |this, cx| {
            for source in paths {
                let step = |text: &'static str, cx: &mut gpui::AsyncWindowContext| {
                    this.update(cx, |this, cx| this.set_activity(Some(text.into()), cx))
                        .ok();
                };
                let Ok(strings) = cx.update(|_, cx| cx.t()) else {
                    break;
                };
                step(strings.preparing_install(), cx);
                let Ok(staging) = cx.update(|_, cx| stores.thirdparty.read(cx).stage(source, cx))
                else {
                    break;
                };
                let staged = match staging.await {
                    Ok(staged) => staged,
                    Err(failure) => {
                        if let Some((msg, severity)) = stage_failure_notice(&failure) {
                            cx.update(|_, cx| toast(cx, "thirdparty", msg, severity))
                                .ok();
                        }
                        continue;
                    }
                };
                let (staged, classification) = cx
                    .background_executor()
                    .spawn(async move {
                        let classification = staged.classify();
                        (staged, classification)
                    })
                    .await;
                match drop_next(&classification) {
                    DropNext::Tell(msg, severity) => {
                        cx.update(|_, cx| toast(cx, "thirdparty", msg, severity))
                            .ok();
                    }
                    DropNext::OfferLodModImport(ini) => {
                        let Ok(asking) = cx.update(|window, cx| {
                            let strings = cx.t();
                            confirm(
                                ConfirmOptions::new(
                                    strings.mods_ui_lodmod_title(),
                                    strings.third_party_lod_mod_prompt(),
                                )
                                .confirm_label(strings.button_yes())
                                .cancel_label(strings.button_no()),
                                window,
                                cx,
                            )
                        }) else {
                            break;
                        };
                        if asking.await {
                            let cli = cx
                                .update(|_, cx| stores.config.read(cx).cli())
                                .ok()
                                .flatten();
                            let imported = cx
                                .background_executor()
                                .spawn(async move {
                                    let cli = cli?;
                                    let content =
                                        yp_core::fs::read_text_opt(&ini).ok()?.unwrap_or_default();
                                    let values = yp_config::legacy::parse_lodmod_ini(&content);
                                    yp_config::legacy::import_lodmod_ini(cli.as_ref(), &values).ok()
                                })
                                .await;
                            let (msg, severity) = lodmod_import_notice(imported);
                            cx.update(|_, cx| {
                                stores
                                    .config
                                    .update(cx, |config, cx| config.reload_if_clean(cx));
                                toast(cx, "thirdparty", msg, severity);
                            })
                            .ok();
                        }
                    }
                    DropNext::Install => {
                        let Ok(checking) = cx.update(|_, cx| {
                            stores
                                .thirdparty
                                .read(cx)
                                .would_update(classification.clone(), cx)
                        }) else {
                            break;
                        };
                        if let Some(update) = checking.await {
                            let Ok(asking) = cx.update(|window, cx| {
                                let strings = cx.t();
                                let name = runtime_name(update.runtime, strings);
                                confirm(
                                    ConfirmOptions::new(
                                        strings.third_party_update_title(name),
                                        update_body(&update, strings),
                                    )
                                    .confirm_label(strings.third_party_update_replace())
                                    .cancel_label(strings.third_party_update_keep()),
                                    window,
                                    cx,
                                )
                            }) else {
                                break;
                            };
                            if !asking.await {
                                cx.update(|_, cx| {
                                    toast(
                                        cx,
                                        "thirdparty",
                                        Msg::ThirdPartyUpdateSkipped,
                                        Severity::Info,
                                    )
                                })
                                .ok();
                                drop(staged);
                                continue;
                            }
                        }
                        step(strings.mods_ui_installing(), cx);
                        let Ok(installing) = cx.update(|_, cx| {
                            stores
                                .thirdparty
                                .update(cx, |store, cx| store.install(classification, cx))
                        }) else {
                            break;
                        };
                        let (msg, severity) = install_notice(&installing.await);
                        cx.update(|_, cx| toast(cx, "thirdparty", msg, severity))
                            .ok();
                    }
                }
                drop(staged);
            }
            this.update(cx, |this, cx| this.set_activity(None, cx)).ok();
        })
        .detach();
    }

    fn browse(&mut self, files: bool, window: &mut Window, cx: &mut Context<Self>) {
        let picking = cx.prompt_for_paths(PathPromptOptions {
            files,
            directories: !files,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = picking.await else {
                return;
            };
            this.update_in(cx, |this, window, cx| this.install_paths(paths, window, cx))
                .ok();
        })
        .detach();
    }

    fn open_install_dir(&mut self, which: ThirdPartyRuntime, cx: &mut Context<Self>) {
        let finding = self
            .stores
            .thirdparty
            .read(cx)
            .existing_install_dir(which, cx);
        cx.spawn(async move |_, cx| {
            if let Some(dir) = finding.await {
                cx.update(|cx| reveal(dir, false, cx));
            }
        })
        .detach();
    }

    fn open_shaders_dir(&mut self, cx: &mut Context<Self>) {
        let finding = self.stores.thirdparty.read(cx).existing_shaders_dir(cx);
        cx.spawn(async move |_, cx| {
            if let Some(dir) = finding.await {
                cx.update(|cx| reveal(dir, false, cx));
            }
        })
        .detach();
    }

    fn apply_reshade(&mut self, config: ReShadeConfig, cx: &mut Context<Self>) {
        self.stores
            .thirdparty
            .update(cx, |store, cx| store.apply_reshade_config(config, cx));
    }

    fn apply_migoto(&mut self, config: MigotoConfig, cx: &mut Context<Self>) {
        self.stores
            .thirdparty
            .update(cx, |store, cx| store.apply_migoto_config(config, cx));
    }

    fn apply_game_mod(
        &mut self,
        file_name: &str,
        settings: GameModSettings,
        cx: &mut Context<Self>,
    ) {
        self.stores.thirdparty.update(cx, |store, cx| {
            store.set_game_mod_settings(file_name, settings, cx);
        });
    }

    fn header_controls(
        &self,
        which: ThirdPartyRuntime,
        status: &RuntimeStatus,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let strings = cx.t();
        let enabled = status.enabled;
        let index = which as usize;
        h_flex()
            .gap(sizes.spacing_md)
            .child(
                Label::new(if enabled {
                    strings.third_party_status_active()
                } else {
                    strings.third_party_status_inactive()
                })
                .size(LabelSize::XSmall)
                .color(if enabled {
                    Color::Success
                } else {
                    Color::Muted
                })
                .semibold(),
            )
            .child(
                div()
                    .id(("thirdparty-enable-tip", index))
                    .tooltip(Tooltip::text(strings.third_party_enable()))
                    .child(Switch::new(("thirdparty-enable", index), enabled).on_click(
                        cx.listener(move |this, _, _, cx| {
                            this.stores.thirdparty.update(cx, |store, cx| {
                                store.set_enabled(which, !enabled, cx);
                            });
                        }),
                    )),
            )
            .child(
                IconButton::new(("thirdparty-open", index), IconName::FolderOpen)
                    .size(ButtonSize::Compact)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text(strings.third_party_open_folder()))
                    .controller_support(false)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_install_dir(which, cx);
                    })),
            )
            .child(
                IconButton::new(("thirdparty-remove", index), IconName::DeleteOutline)
                    .size(ButtonSize::Compact)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Error)
                    .tooltip(Tooltip::text(strings.third_party_remove()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.stores
                            .thirdparty
                            .update(cx, |store, cx| store.remove(which, cx));
                    })),
            )
    }

    fn status_row(
        &self,
        which: ThirdPartyRuntime,
        presence: RuntimePresence,
        status: &RuntimeStatus,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let strings = cx.t();
        let line = status_line(presence);
        let color = match line.severity {
            Some(Severity::Success) => Color::Success,
            Some(Severity::Warning) => Color::Warning,
            _ => Color::Muted,
        };
        h_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(Icon::new(line.icon).size(IconSize::Small).color(color))
            .child(
                div().flex_1().min_w_0().child(
                    Label::new(status_text(presence, status, strings))
                        .size(LabelSize::Small)
                        .color(color),
                ),
            )
            .when(line.can_import, |this| {
                this.child(
                    Button::new(
                        ("thirdparty-import", which as usize),
                        strings.third_party_import(),
                    )
                    .style(ButtonStyle::Tinted(TintColor::Accent))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.import_from_game_root(which, cx);
                    })),
                )
            })
    }

    fn render_reshade(
        &self,
        status: &RuntimeStatus,
        presence: RuntimePresence,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> Card {
        let strings = cx.t();
        let which = ThirdPartyRuntime::ReShade;
        let card = Card::new().title(strings.third_party_re_shade_header());
        if !status.installed {
            return card.child(
                v_flex()
                    .w_full()
                    .items_start()
                    .gap(sizes.spacing_lg)
                    .child(self.status_row(which, presence, status, sizes, cx))
                    .child(howto(strings.third_party_re_shade_howto()))
                    .child(
                        Button::new("thirdparty-get-reshade", strings.third_party_get_re_shade())
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .end_icon(IconName::OpenInNew)
                            .on_click(|_, _, cx| cx.open_url(yp_core::consts::RESHADE_ME_URL)),
                    ),
            );
        }
        let info = status.reshade_info.clone().unwrap_or_default();
        let config = info.config.clone();
        let version = match (&info.version, info.is_addon_build) {
            (Some(version), true) => {
                Some(strings.mods_ui_dot_join(version, strings.third_party_addon_build()))
            }
            (Some(version), false) => Some(version.clone()),
            (None, true) if info.dll_name.is_some() => {
                Some(strings.third_party_addon_build().to_owned())
            }
            _ => None,
        };
        let performance = config.clone();
        let fps = config.clone();
        let clock = config.clone();
        let shader_count = i64::try_from(info.shader_count).unwrap_or(i64::MAX);
        card.header_end(self.header_controls(which, status, sizes, cx))
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .children(
                        version.map(|version| {
                            key_value(strings.third_party_version(), version, strings)
                        }),
                    )
                    .child(chip_section(
                        "reshade-presets",
                        strings.third_party_presets().to_owned(),
                        &info.presets,
                        strings.third_party_none_presets(),
                        sizes,
                    ))
                    .child(chip_section(
                        "reshade-shaders",
                        strings.mods_ui_dot_join(
                            strings.third_party_shader_repos(),
                            &strings.third_party_shader_count(shader_count),
                        ),
                        &info.shader_repos,
                        strings.third_party_none_shaders(),
                        sizes,
                    ))
                    .when(!info.addons.is_empty(), |this| {
                        this.child(chip_section(
                            "reshade-addons",
                            strings.third_party_addons().to_owned(),
                            &info.addons,
                            "",
                            sizes,
                        ))
                    })
                    .child(
                        config_group(strings, sizes)
                            .children(config.active_preset.clone().map(|preset| {
                                key_value(
                                    strings.third_party_re_shade_active_preset(),
                                    preset,
                                    strings,
                                )
                            }))
                            .child(
                                SettingSwitch::new(
                                    "reshade-performance",
                                    strings.third_party_re_shade_performance_mode(),
                                    config.performance_mode,
                                )
                                .description(strings.third_party_re_shade_performance_mode_hint())
                                .on_change(cx.listener(
                                    move |this, value: &bool, _, cx| {
                                        this.apply_reshade(
                                            ReShadeConfig {
                                                performance_mode: *value,
                                                ..performance.clone()
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            )
                            .child(
                                SettingSwitch::new(
                                    "reshade-fps",
                                    strings.third_party_re_shade_show_fps(),
                                    config.show_fps,
                                )
                                .description(strings.third_party_re_shade_show_fps_hint())
                                .on_change(cx.listener(
                                    move |this, value: &bool, _, cx| {
                                        this.apply_reshade(
                                            ReShadeConfig {
                                                show_fps: *value,
                                                ..fps.clone()
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            )
                            .child(
                                SettingSwitch::new(
                                    "reshade-clock",
                                    strings.third_party_re_shade_show_clock(),
                                    config.show_clock,
                                )
                                .description(strings.third_party_re_shade_show_clock_hint())
                                .on_change(cx.listener(
                                    move |this, value: &bool, _, cx| {
                                        this.apply_reshade(
                                            ReShadeConfig {
                                                show_clock: *value,
                                                ..clock.clone()
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            ),
                    )
                    .when(status.shaders_missing, |this| {
                        this.child(
                            Callout::new()
                                .id("reshade-shaders-missing")
                                .severity(Severity::Warning)
                                .description(strings.third_party_shaders_missing())
                                .actions_slot(
                                    Button::new(
                                        "reshade-open-shaders",
                                        strings.third_party_open_folder(),
                                    )
                                    .controller_support(false)
                                    .style(ButtonStyle::Tinted(TintColor::Warning))
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.open_shaders_dir(cx);
                                        },
                                    )),
                                ),
                        )
                    })
                    .when(info.d3d_compiler_missing, |this| {
                        this.child(
                            Callout::new()
                                .id("reshade-compiler-missing")
                                .severity(Severity::Warning)
                                .description(strings.third_party_d3d_compiler_missing())
                                .actions_slot(
                                    Button::new(
                                        "reshade-open-install",
                                        strings.third_party_open_folder(),
                                    )
                                    .controller_support(false)
                                    .style(ButtonStyle::Tinted(TintColor::Warning))
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| {
                                            this.open_install_dir(which, cx);
                                        },
                                    )),
                                ),
                        )
                    }),
            )
    }

    fn render_migoto(
        &self,
        status: &RuntimeStatus,
        presence: RuntimePresence,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> Card {
        let strings = cx.t();
        let which = ThirdPartyRuntime::Migoto;
        let card = Card::new().title(strings.third_party_migoto_header());
        if !status.installed {
            return card.child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(self.status_row(which, presence, status, sizes, cx))
                    .child(howto(strings.third_party_migoto_howto())),
            );
        }
        let info = status.migoto_info.clone().unwrap_or_default();
        let config = info.config.clone();
        let fix_count = i64::try_from(info.shader_fix_count).unwrap_or(i64::MAX);
        let hunting_label = |hunting: MigotoHunting| match hunting {
            MigotoHunting::Off => strings.third_party_migoto_hunting_off(),
            MigotoHunting::On => strings.third_party_migoto_hunting_on(),
            MigotoHunting::NoMarking => strings.third_party_migoto_hunting_no_marking(),
        };
        let marking_label = |marking: MigotoMarking| match marking {
            MigotoMarking::Skip => strings.third_party_migoto_marking_skip(),
            MigotoMarking::Original => strings.third_party_migoto_marking_original(),
            MigotoMarking::Pink => strings.third_party_migoto_marking_pink(),
            MigotoMarking::Mono => strings.third_party_migoto_marking_mono(),
        };
        let hunting_config = config.clone();
        let marking_config = config.clone();
        let cache_config = config.clone();
        let verbose_config = config.clone();
        let foreground_config = config.clone();
        card.header_end(self.header_controls(which, status, sizes, cx))
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .when(!info.loader_target_ok, |this| {
                        this.child(
                            h_flex()
                                .w_full()
                                .gap(sizes.spacing_md)
                                .child(
                                    Icon::new(IconName::ErrorOutline)
                                        .size(IconSize::Small)
                                        .color(Color::Error),
                                )
                                .child(
                                    Label::new(
                                        strings.mods_ui_label_value(
                                            strings.third_party_loader_target(),
                                            info.loader_target
                                                .as_deref()
                                                .unwrap_or(strings.mods_ui_none()),
                                        ),
                                    )
                                    .size(LabelSize::XSmall)
                                    .color(Color::Error),
                                ),
                        )
                    })
                    .child(chip_section(
                        "migoto-fixes",
                        strings.mods_ui_dot_join(
                            strings.third_party_shader_fixes(),
                            &strings.third_party_shader_fix_count(fix_count),
                        ),
                        &info.shader_fix_names,
                        "",
                        sizes,
                    ))
                    .child(
                        config_group(strings, sizes)
                            .child(
                                SettingSelect::<MigotoHunting>::new(
                                    "migoto-hunting",
                                    strings.third_party_migoto_hunting(),
                                    hunting_label(config.hunting),
                                )
                                .items(
                                    [
                                        MigotoHunting::Off,
                                        MigotoHunting::On,
                                        MigotoHunting::NoMarking,
                                    ]
                                    .map(|hunting| {
                                        SelectItem::new(hunting_label(hunting), hunting)
                                    }),
                                )
                                .description(strings.third_party_migoto_hunting_hint())
                                .on_select(cx.listener(
                                    move |this, hunting: &MigotoHunting, _, cx| {
                                        this.apply_migoto(
                                            MigotoConfig {
                                                hunting: *hunting,
                                                ..hunting_config.clone()
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            )
                            .when(config.hunting != MigotoHunting::Off, |this| {
                                this.child(
                                    SettingSelect::<MigotoMarking>::new(
                                        "migoto-marking",
                                        strings.third_party_migoto_marking_mode(),
                                        marking_label(config.marking_mode),
                                    )
                                    .items(
                                        [
                                            MigotoMarking::Skip,
                                            MigotoMarking::Original,
                                            MigotoMarking::Pink,
                                            MigotoMarking::Mono,
                                        ]
                                        .map(|marking| {
                                            SelectItem::new(marking_label(marking), marking)
                                        }),
                                    )
                                    .description(strings.third_party_migoto_marking_mode_hint())
                                    .on_select(cx.listener(
                                        move |this, marking: &MigotoMarking, _, cx| {
                                            this.apply_migoto(
                                                MigotoConfig {
                                                    marking_mode: *marking,
                                                    ..marking_config.clone()
                                                },
                                                cx,
                                            );
                                        },
                                    )),
                                )
                            })
                            .child(
                                SettingSwitch::new(
                                    "migoto-cache",
                                    strings.third_party_migoto_cache_shaders(),
                                    config.cache_shaders,
                                )
                                .description(strings.third_party_migoto_cache_shaders_hint())
                                .on_change(cx.listener(
                                    move |this, value: &bool, _, cx| {
                                        this.apply_migoto(
                                            MigotoConfig {
                                                cache_shaders: *value,
                                                ..cache_config.clone()
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            )
                            .child(
                                SettingSwitch::new(
                                    "migoto-verbose",
                                    strings.third_party_migoto_verbose_overlay(),
                                    config.verbose_overlay,
                                )
                                .description(strings.third_party_migoto_verbose_overlay_hint())
                                .on_change(cx.listener(
                                    move |this, value: &bool, _, cx| {
                                        this.apply_migoto(
                                            MigotoConfig {
                                                verbose_overlay: *value,
                                                ..verbose_config.clone()
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            )
                            .child(
                                SettingSwitch::new(
                                    "migoto-foreground",
                                    strings.third_party_migoto_check_foreground_window(),
                                    config.check_foreground_window,
                                )
                                .description(
                                    strings.third_party_migoto_check_foreground_window_hint(),
                                )
                                .on_change(cx.listener(
                                    move |this, value: &bool, _, cx| {
                                        this.apply_migoto(
                                            MigotoConfig {
                                                check_foreground_window: *value,
                                                ..foreground_config.clone()
                                            },
                                            cx,
                                        );
                                    },
                                )),
                            ),
                    )
                    .when(!info.files.is_empty(), |this| {
                        this.child(chip_section(
                            "migoto-files",
                            strings.third_party_files().to_owned(),
                            &info.files,
                            "",
                            sizes,
                        ))
                    }),
            )
    }

    fn render_game_mod_settings(
        &self,
        index: usize,
        entry: &GameModEntry,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let strings = cx.t();
        let settings = entry.settings.unwrap_or_default();
        let file_name = entry.file_name.clone();
        let apply = |change: fn(GameModSettings, bool) -> GameModSettings| {
            let file_name = file_name.clone();
            cx.listener(move |this, value: &bool, _, cx| {
                this.apply_game_mod(&file_name, change(settings, *value), cx);
            })
        };
        let timeout_file = file_name.clone();
        let size_file = file_name.clone();
        v_flex().w_full().pl(sizes.padding_xl).child(
            config_group(strings, sizes)
                .child(
                    SettingSwitch::new(
                        ("game-mod-fake-handle", index),
                        strings.third_party_game_mod_fake_handle(),
                        settings.fake_module_handle,
                    )
                    .description(strings.third_party_game_mod_fake_handle_hint())
                    .on_change(apply(|settings, value| GameModSettings {
                        fake_module_handle: value,
                        ..settings
                    })),
                )
                .child(
                    SettingSwitch::new(
                        ("game-mod-fake-exe", index),
                        strings.third_party_game_mod_fake_exe_path(),
                        settings.fake_exe_path,
                    )
                    .description(strings.third_party_game_mod_fake_exe_path_hint())
                    .on_change(apply(GameModSettings::with_fake_exe_path)),
                )
                .child(
                    SettingSwitch::new(
                        ("game-mod-wait-patch", index),
                        strings.third_party_game_mod_wait_patch(),
                        settings.wait_for_patch_phase,
                    )
                    .description(strings.third_party_game_mod_wait_patch_hint())
                    .on_change(apply(GameModSettings::with_wait_for_patch_phase)),
                )
                .when(settings.wait_for_patch_phase, |this| {
                    this.child(
                        SettingSelect::<u64>::new(
                            ("game-mod-timeout", index),
                            strings.third_party_game_mod_patch_timeout(),
                            patch_timeout_label(settings.patch_phase_timeout_ms, strings),
                        )
                        .items(
                            patch_timeout_options(settings.patch_phase_timeout_ms)
                                .into_iter()
                                .map(|timeout| {
                                    SelectItem::new(patch_timeout_label(timeout, strings), timeout)
                                }),
                        )
                        .description(strings.third_party_game_mod_patch_timeout_hint())
                        .on_select(cx.listener(
                            move |this, timeout: &u64, _, cx| {
                                this.apply_game_mod(
                                    &timeout_file,
                                    GameModSettings {
                                        patch_phase_timeout_ms: *timeout,
                                        ..settings
                                    },
                                    cx,
                                );
                            },
                        )),
                    )
                })
                .child(
                    SettingSelect::<u64>::new(
                        ("game-mod-image-size", index),
                        strings.third_party_game_mod_size_of_image(),
                        image_size_label(settings.size_of_image, strings),
                    )
                    .items(
                        image_size_options(settings.size_of_image)
                            .into_iter()
                            .map(|size| SelectItem::new(image_size_label(size, strings), size)),
                    )
                    .description(strings.third_party_game_mod_size_of_image_hint())
                    .on_select(cx.listener(move |this, size: &u64, _, cx| {
                        this.apply_game_mod(
                            &size_file,
                            GameModSettings {
                                size_of_image: *size,
                                ..settings
                            },
                            cx,
                        );
                    })),
                )
                .child(
                    SettingSwitch::new(
                        ("game-mod-report", index),
                        strings.third_party_game_mod_report_patches(),
                        settings.report_patches,
                    )
                    .description(strings.third_party_game_mod_report_patches_hint())
                    .on_change(apply(|settings, value| GameModSettings {
                        report_patches: value,
                        ..settings
                    })),
                ),
        )
    }

    fn render_dll_row(
        &self,
        which: ThirdPartyRuntime,
        index: usize,
        entry: &GameModEntry,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let is_game_mod = which == ThirdPartyRuntime::GameMods;
        let slot = which as usize * 1000 + index;
        let disabled = entry.disabled;
        let key = entry.key();
        let expanded = is_game_mod && self.is_expanded(&key);
        let label = Label::new(dll_row_label(&entry.label(), entry.size_bytes, strings))
            .size(LabelSize::Small)
            .truncate();
        let label = if disabled {
            label.color(Color::Muted).strikethrough()
        } else {
            label.color(Color::Secondary)
        };
        let toggle_file = entry.file_name.clone();
        let toggle_dir = entry.mod_dir.clone();
        let toggle = div()
            .id(("thirdparty-dll-toggle-tip", slot))
            .tooltip(Tooltip::text(strings.third_party_game_mods_toggle_hint()))
            .child(
                Switch::new(("thirdparty-dll-toggle", slot), !disabled).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.stores.thirdparty.update(cx, |store, cx| {
                            if is_game_mod {
                                store.set_game_mod_disabled(&toggle_file, !disabled, cx);
                            } else {
                                store.set_graphic_mod_disabled(&toggle_dir, !disabled, cx);
                            }
                        });
                    },
                )),
            );
        let settings_key = key.clone();
        let move_dir = entry.mod_dir.clone();
        let remove_dir = entry.mod_dir.clone();
        let actions = h_flex()
            .gap(sizes.spacing_sm)
            .when(disabled, |this| {
                this.child(
                    Label::new(strings.third_party_game_mods_disabled_chip())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .italic(),
                )
            })
            .when(is_game_mod, |this| {
                this.child(
                    IconButton::new(
                        ("thirdparty-dll-settings", slot),
                        if expanded {
                            IconName::ExpandLess
                        } else {
                            IconName::Tune
                        },
                    )
                    .size(ButtonSize::Compact)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text(strings.third_party_game_mod_settings()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_expanded(&settings_key, cx);
                    })),
                )
            })
            .child(
                IconButton::new(("thirdparty-dll-move", slot), IconName::SwapHoriz)
                    .size(ButtonSize::Compact)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text(if is_game_mod {
                        strings.third_party_move_to_graphic()
                    } else {
                        strings.third_party_move_to_game()
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.stores
                            .thirdparty
                            .update(cx, |store, cx| store.move_mod(&move_dir, which, cx));
                    })),
            )
            .child(
                IconButton::new(("thirdparty-dll-remove", slot), IconName::DeleteOutline)
                    .size(ButtonSize::Compact)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Error)
                    .tooltip(Tooltip::text(strings.third_party_game_mods_remove_one()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.stores.thirdparty.update(cx, |store, cx| {
                            if is_game_mod {
                                store.remove_game_mod(&remove_dir, cx);
                            } else {
                                store.remove_graphic_mod(&remove_dir, cx);
                            }
                        });
                    })),
            );
        let row = ListItem::new(("thirdparty-dll-row", slot))
            .spacing(ListItemSpacing::Dense)
            .rounded(true)
            .start_slot(toggle)
            .child(shrinkable(label))
            .end_slot(actions);
        v_flex()
            .w_full()
            .child(row)
            .when(expanded, |this| {
                this.child(self.render_game_mod_settings(index, entry, sizes, cx))
            })
            .into_any_element()
    }

    fn render_dll_card(
        &self,
        which: ThirdPartyRuntime,
        status: &RuntimeStatus,
        presence: RuntimePresence,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> Card {
        let strings = cx.t();
        let (title, howto_text, info) = match which {
            ThirdPartyRuntime::GraphicMods => (
                strings.third_party_graphic_mods_header(),
                strings.third_party_graphic_mods_howto(),
                status.graphic_mods_info.clone().unwrap_or_default(),
            ),
            _ => (
                strings.third_party_game_mods_header(),
                strings.third_party_game_mods_howto(),
                status.game_mods_info.clone().unwrap_or_default(),
            ),
        };
        let card = Card::new().title(title);
        if info.mods.is_empty() {
            return card.child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(self.status_row(which, presence, status, sizes, cx))
                    .child(howto(howto_text)),
            );
        }
        let rows: Vec<AnyElement> = info
            .mods
            .iter()
            .enumerate()
            .map(|(index, entry)| self.render_dll_row(which, index, entry, sizes, cx))
            .collect();
        card.header_end(self.header_controls(which, status, sizes, cx))
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(howto(howto_text))
                    .child(
                        Label::new(strings.third_party_game_mods_count(
                            i64::try_from(info.mods.len()).unwrap_or(i64::MAX),
                        ))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .bold(),
                    )
                    .children(rows),
            )
    }
}

fn howto(text: &'static str) -> impl IntoElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(Color::Secondary)
}

fn key_value(key: &'static str, value: String, strings: &yp_ui::Strings) -> impl IntoElement {
    Label::new(strings.mods_ui_label_value(key, &value))
        .size(LabelSize::XSmall)
        .color(Color::Secondary)
}

fn config_group(strings: &yp_ui::Strings, sizes: &Sizes) -> gpui::Div {
    v_flex().w_full().child(
        h_flex()
            .gap(sizes.spacing_md)
            .child(
                Label::new(strings.third_party_config_section())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .bold(),
            )
            .child(
                Icon::new(IconName::RestartAlt)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Label::new(strings.third_party_restart_required())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .italic(),
            ),
    )
}

fn chip_section(
    id: &'static str,
    title: String,
    items: &[String],
    empty_label: &'static str,
    sizes: &Sizes,
) -> impl IntoElement + use<> {
    let chips: Vec<AnyElement> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            Chip::new(item.clone())
                .id((id, index))
                .tone(ChipTone::Neutral)
                .size(ChipSize::Small)
                .into_any_element()
        })
        .collect();
    v_flex()
        .w_full()
        .gap(sizes.spacing_md)
        .child(
            Label::new(title)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .bold(),
        )
        .map(|this| {
            if chips.is_empty() {
                this.when(!empty_label.is_empty(), |this| {
                    this.child(
                        Label::new(empty_label)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .italic(),
                    )
                })
            } else {
                this.child(h_flex().flex_wrap().gap(sizes.spacing_md).children(chips))
            }
        })
}

impl Render for ThirdPartyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let view = cx.entity().downgrade();
        let (statuses, presence) = {
            let store = self.stores.thirdparty.read(cx);
            (
                store.statuses().clone(),
                ThirdPartyRuntime::ALL.map(|which| store.presence(which)),
            )
        };
        let presence_of = |which: ThirdPartyRuntime| presence[which as usize];

        let banner = Callout::new()
            .id("thirdparty-banner")
            .severity(Severity::Accent)
            .icon(IconName::InfoOutline)
            .title(strings.third_party_banner_title())
            .description(strings.third_party_banner())
            .child(
                h_flex().child(
                    Button::new("thirdparty-read-doc", strings.third_party_read_doc())
                        .style(ButtonStyle::Tinted(TintColor::Accent))
                        .on_click(|_, window, cx| {
                            window.dispatch_action(
                                Box::new(OpenDocs {
                                    page: Some(THIRD_PARTY_DOC_PAGE.to_owned()),
                                }),
                                cx,
                            );
                        }),
                ),
            );

        let drop_view = view.clone();
        let files_view = view.clone();
        let folder_view = view.clone();
        let drop_card = Card::new().title(strings.third_party_install_card()).child(
            DropZone::new("thirdparty-drop-zone")
                .title(strings.third_party_drop_here())
                .hint(strings.drop_mod_here_hint())
                .busy(self.activity.clone().map(DropProgress::new))
                .on_drop_paths(move |paths, window, cx| {
                    let paths = paths.to_vec();
                    drop_view
                        .update(cx, |this, cx| this.install_paths(paths, window, cx))
                        .ok();
                })
                .on_browse_files(move |window, cx| {
                    files_view
                        .update(cx, |this, cx| this.browse(true, window, cx))
                        .ok();
                })
                .on_browse_folder(move |window, cx| {
                    folder_view
                        .update(cx, |this, cx| this.browse(false, window, cx))
                        .ok();
                }),
        );

        let reshade = self.render_reshade(
            &statuses.reshade,
            presence_of(ThirdPartyRuntime::ReShade),
            &sizes,
            cx,
        );
        let migoto = self.render_migoto(
            &statuses.migoto,
            presence_of(ThirdPartyRuntime::Migoto),
            &sizes,
            cx,
        );
        let game_mods = self.render_dll_card(
            ThirdPartyRuntime::GameMods,
            &statuses.game_mods,
            presence_of(ThirdPartyRuntime::GameMods),
            &sizes,
            cx,
        );
        let graphic_mods = self.render_dll_card(
            ThirdPartyRuntime::GraphicMods,
            &statuses.graphic_mods,
            presence_of(ThirdPartyRuntime::GraphicMods),
            &sizes,
            cx,
        );

        let left = v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(drop_card)
            .child(reshade);
        let right = v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(migoto)
            .child(game_mods)
            .child(graphic_mods);

        v_flex()
            .size_full()
            .child(
                PageHeader::new(strings.third_party_title())
                    .info_tooltip(strings.third_party_subtitle()),
            )
            .child(
                div()
                    .id("thirdparty-scroll")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(sizes.padding_md)
                    .p(sizes.content_padding)
                    .map(|el| {
                        let panels = TwoColumn::new(left, right).id("thirdparty-columns");
                        if cx.theme().id == ThemeId::NierNew {
                            el.child(panels).child(banner)
                        } else {
                            el.child(banner).child(panels)
                        }
                    })
                    .overflow_y_scrollbar(&self.scroll),
            )
    }
}
