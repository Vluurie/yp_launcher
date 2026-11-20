use std::sync::Arc;

use gpui::{
    AnyElement, App, ClipboardItem, Context, DismissEvent, EventEmitter, FocusHandle, Focusable,
    IntoElement, Render, ScrollHandle, SharedString, Task, Window,
};
use jiff::Timestamp;
use jiff::tz::TimeZone;
use yp_health::diagnostics::full_report::mod_kind_name;
use yp_health::diagnostics::human_size;
use yp_health::diagnostics::model::{LAUNCHER_VERSION_KEY, LOCALE_KEY, OS_KEY, OS_VERSION_KEY};
use yp_health::{DiagnosticFinding, DiagnosticsReport, FindingLevel};
use yp_i18n::Msg;
use yp_mods::ModKind;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, Chip, ChipTone, CountBadge, IconSize, LabelSize, Modal, ModalView,
    ModalWidth,
    ScrollableElement, SelectableText, Spinner, TintColor, TwoColumn, WindowExt,
};

use crate::metrics;
use crate::notify;
use crate::parts;

const UNKNOWN: &str = "?";
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const LIST_SEPARATOR: &str = ", ";
const VANILLA_DROP_LIMIT: usize = 12;
const CONFIG_DELTA_LIMIT: usize = 14;
const DATA_DIR_LIMIT: usize = 8;
const ROOT_EXTRA_LIMIT: usize = 20;

pub fn local_time(stamp: Timestamp) -> String {
    stamp
        .to_zoned_datetime(TimeZone::system())
        .strftime(TIME_FORMAT)
        .to_string()
}
pub fn mod_kind_counts(kinds: impl IntoIterator<Item = ModKind>) -> Vec<(ModKind, usize)> {
    let mut counts: Vec<(ModKind, usize)> = Vec::new();
    for kind in kinds {
        match counts.iter_mut().find(|(known, _)| *known == kind) {
            Some((_, count)) => *count += 1,
            None => counts.push((kind, 1)),
        }
    }
    counts
}

pub fn largest_first(entries: impl IntoIterator<Item = (String, usize)>) -> Vec<(String, usize)> {
    let mut sorted: Vec<(String, usize)> = entries.into_iter().collect();
    sorted.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    sorted
}

fn finding_color(level: FindingLevel) -> Color {
    match level {
        FindingLevel::Error => Color::Error,
        FindingLevel::Warning => Color::Warning,
        FindingLevel::Info => Color::Accent,
    }
}

fn finding_icon(level: FindingLevel) -> IconName {
    match level {
        FindingLevel::Error => IconName::ErrorOutline,
        FindingLevel::Warning => IconName::WarningAmberRounded,
        FindingLevel::Info => IconName::InfoOutline,
    }
}

pub struct DiagnosticsLoading {
    focus_handle: FocusHandle,
    _collect: Task<()>,
}

impl DiagnosticsLoading {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let collecting = cx.stores().session.read(cx).collect_diagnostics(cx);
        let collect = cx.spawn_in(window, async move |this, cx| {
            let result = collecting.await;
            this.update_in(cx, |_, window, cx| {
                cx.emit(DismissEvent);
                match result {
                    Ok(report) => {
                        window.open_modal(cx, |_, cx| DiagnosticsDialog::new(report, cx));
                    }
                    Err(err) => notify::error(
                        "diag_fail",
                        Msg::ShellDiagnosticsFailed {
                            reason: err.to_string(),
                        },
                        cx,
                    ),
                }
            })
            .ok();
        });
        Self {
            focus_handle: cx.focus_handle(),
            _collect: collect,
        }
    }
}

impl EventEmitter<DismissEvent> for DiagnosticsLoading {}

impl Focusable for DiagnosticsLoading {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for DiagnosticsLoading {
    fn dismiss_on_scrim_click(&self) -> bool {
        false
    }
}

impl Render for DiagnosticsLoading {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        div().track_focus(&self.focus_handle).child(
            Modal::new("shell-diagnostics-loading", None)
                .width(ModalWidth::Small)
                .section(
                    h_flex()
                        .w_full()
                        .justify_center()
                        .gap(sizes.spacing_lg)
                        .child(Spinner::new().size(IconSize::Large))
                        .child(
                            Label::new(strings.diagnostics_collecting())
                                .size(LabelSize::Default)
                                .color(Color::Default),
                        ),
                ),
        )
    }
}

pub fn open(window: &mut Window, cx: &mut App) {
    if window.has_modal(cx) {
        return;
    }
    window.open_modal(cx, DiagnosticsLoading::new);
}

pub struct DiagnosticsDialog {
    report: Arc<DiagnosticsReport>,
    summary: SharedString,
    saving: bool,
    focus_handle: FocusHandle,
    scroll: ScrollHandle,
    save_task: Option<Task<()>>,
}

impl DiagnosticsDialog {
    pub fn new(report: DiagnosticsReport, cx: &mut Context<Self>) -> Self {
        let summary = yp_health::format_summary(&report).into();
        Self {
            report: Arc::new(report),
            summary,
            saving: false,
            focus_handle: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            save_task: None,
        }
    }

    pub fn report(&self) -> &DiagnosticsReport {
        &self.report
    }

    fn copy_summary(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.summary.to_string()));
        notify::success("diag_copied", Msg::DiagnosticsCopied, cx);
    }

    fn save_full_report(&mut self, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        self.saving = true;
        cx.notify();
        let report = self.report.clone();
        let paths = cx.env().paths.clone();
        let write =
            cx.background_spawn(async move { yp_health::write_full_report(&report, &paths) });
        self.save_task = Some(cx.spawn(async move |this, cx| {
            let written = write.await;
            this.update(cx, |this, cx| {
                this.saving = false;
                match written {
                    Ok(path) => notify::success(
                        "diag_saved",
                        Msg::DiagnosticsSavedAt {
                            path: path.to_string_lossy().into_owned(),
                        },
                        cx,
                    ),
                    Err(err) => notify::error(
                        "diag_fail",
                        Msg::ShellDiagnosticsFailed {
                            reason: err.to_string(),
                        },
                        cx,
                    ),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn cards(&self, window: &Window, cx: &App) -> (AnyElement, Vec<AnyElement>) {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let report = &self.report;
        let chip = |label: SharedString, tone: ChipTone| Chip::new(label).tone(tone);
        let flag = |label: SharedString, on: bool| {
            Chip::new(label)
                .icon(if on {
                    IconName::CheckCircle
                } else {
                    IconName::RemoveCircleOutline
                })
                .tone(if on {
                    ChipTone::Success
                } else {
                    ChipTone::Muted
                })
        };
        let chips = || h_flex().w_full().flex_wrap().gap(sizes.spacing_md);
        let badge = |count: usize| {
            CountBadge::new(count)
                .tone(ChipTone::Accent)
                .into_any_element()
        };
        let mut next_row = 0usize;
        let mut kv = |key: SharedString, value: SharedString| {
            next_row += 1;
            v_flex()
                .w_full()
                .child(
                    Label::new(key)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .semibold(),
                )
                .when(!value.is_empty(), |this| {
                    this.child(
                        SelectableText::new(("shell-diag-value", next_row), value)
                            .mono()
                            .size(LabelSize::XSmall)
                            .color(Color::Secondary),
                    )
                })
        };
        let body = || v_flex().w_full().gap(sizes.spacing_md);
        let more = |hidden: usize| parts::muted_line(strings.diagnostics_more_items(hidden as i64));

        let worst = report.findings.first().map(|finding| finding.level);
        let mut findings = body();
        if report.findings.is_empty() {
            findings = findings.child(parts::muted_line(strings.diagnostics_no_findings()));
        }
        for (index, finding) in report.findings.iter().enumerate() {
            findings = findings.child(finding_row(index, finding, window, cx));
        }
        let findings_card = parts::section_card(
            IconName::FactCheckOutlined,
            strings.diagnostics_section_findings(),
            worst.map_or(Color::Success, finding_color),
            Some(badge(report.findings.len())),
            window,
            cx,
        )
        .child(findings)
        .into_any_element();

        let mut cards: Vec<AnyElement> = Vec::new();

        let os = format!(
            "{} {}",
            report.system_value(OS_KEY).unwrap_or(UNKNOWN),
            report.system_value(OS_VERSION_KEY).unwrap_or_default()
        );
        cards.push(
            parts::section_card(
                IconName::ComputerOutlined,
                strings.log_section_system(),
                Color::Accent,
                None,
                window,
                cx,
            )
            .child(
                body()
                    .child(kv(strings.log_detail_os().into(), os.trim().to_owned().into()))
                    .child(kv(
                        strings.log_detail_locale().into(),
                        report
                            .system_value(LOCALE_KEY)
                            .unwrap_or(UNKNOWN)
                            .to_owned()
                            .into(),
                    ))
                    .when(report.prefer_dedicated_gpu, |this| {
                        this.child(chips().child(chip(
                            strings.shell_diag_dedicated_gpu().into(),
                            ChipTone::Accent,
                        )))
                    }),
            )
            .into_any_element(),
        );

        let identity = &report.game_identity;
        let supported = identity.exe_variant_supported();
        cards.push(
            parts::section_card(
                IconName::VideogameAssetOutlined,
                strings.diagnostics_section_game_identity(),
                if supported {
                    Color::Accent
                } else {
                    Color::Warning
                },
                None,
                window,
                cx,
            )
            .child(
                body()
                    .child(kv(
                        strings
                            .diagnostics_exe_variant(&identity.build_display())
                            .into(),
                        if supported {
                            SharedString::default()
                        } else {
                            strings.diagnostics_exe_unsupported().into()
                        },
                    ))
                    .child(
                        chips()
                            .child(flag(
                                strings.diagnostics_dlc_present().into(),
                                identity.has_dlc,
                            ))
                            .when_some(identity.exe_size, |this, size| {
                                this.child(chip(human_size(size).into(), ChipTone::Muted))
                            })
                            .when_some(
                                identity
                                    .steam_manifest
                                    .as_ref()
                                    .and_then(|manifest| manifest.build_id.clone()),
                                |this, build| {
                                    this.child(chip(
                                        strings.shell_diag_steam_build(&build).into(),
                                        ChipTone::Muted,
                                    ))
                                },
                            )
                            .when(identity.game_bin.is_some(), |this| {
                                this.child(flag(
                                    strings.shell_diag_game_bin().into(),
                                    identity.game_bin_matches_exe(),
                                ))
                            }),
                    ),
            )
            .into_any_element(),
        );

        let health = &report.nams_health;
        let nams_label = match &health.nams_version {
            Some(version) => format!("{} {version}", strings.diagnostics_nams_present()),
            None => strings.diagnostics_nams_present().to_owned(),
        };
        let mut health_chips = chips().child(flag(nams_label.into(), health.nams_exe_present));
        if let Some(running) = health.steam_running {
            health_chips = health_chips.child(flag(strings.shell_diag_steam().into(), running));
        }
        for (role, mount) in &report.filesystems {
            if let Some(mount) = mount {
                health_chips = health_chips.child(chip(
                    format!("{}: {}", role.label(), mount.fs_type).into(),
                    if mount.risky() {
                        ChipTone::Error
                    } else {
                        ChipTone::Muted
                    },
                ));
            }
        }
        if !health.missing_files.is_empty() {
            health_chips = health_chips.child(chip(
                strings
                    .diagnostics_missing_files(health.missing_files.len() as i64)
                    .into(),
                ChipTone::Error,
            ));
        }
        cards.push(
            parts::section_card(
                IconName::HealthAndSafetyOutlined,
                strings.diagnostics_section_nams_health(),
                if health.missing_files.is_empty() {
                    Color::Accent
                } else {
                    Color::Error
                },
                None,
                window,
                cx,
            )
            .child(health_chips)
            .into_any_element(),
        );

        let kinds = mod_kind_counts(report.mods.iter().map(|installed| installed.kind));
        let mut mods = body();
        if kinds.is_empty() {
            mods = mods.child(parts::muted_line(strings.log_no_mods_installed()));
        } else {
            let mut row = chips();
            for (kind, count) in kinds {
                row = row.child(chip(
                    format!("{}: {count}", mod_kind_name(kind)).into(),
                    ChipTone::Accent,
                ));
            }
            mods = mods.child(row);
        }
        if !report.disabled_mods_entries.is_empty() {
            mods = mods.child(parts::muted_line(
                strings.shell_diag_disabled_prefixes(report.disabled_mods_entries.len() as i64),
            ));
        }
        cards.push(
            parts::section_card(
                IconName::ExtensionOutlined,
                strings.log_section_mods_nams(),
                Color::Accent,
                Some(badge(report.mods.len())),
                window,
                cx,
            )
            .child(mods)
            .into_any_element(),
        );

        let reshade = &report.third_party.reshade;
        if reshade.installed {
            let info = reshade.reshade_info.as_ref();
            let shaders_missing =
                reshade.shaders_missing || info.is_some_and(|info| info.d3d_compiler_missing);
            cards.push(
                parts::section_card(
                    IconName::AutoAwesome,
                    strings.diagnostics_section_reshade(),
                    Color::Accent,
                    Some(badge(info.map_or(0, |info| info.presets.len()))),
                    window,
                    cx,
                )
                .child(
                    chips()
                        .child(flag(
                            strings.diagnostics_enabled().into(),
                            reshade.enabled,
                        ))
                        .when_some(
                            info.and_then(|info| info.version.clone()),
                            |this, version| this.child(chip(version.into(), ChipTone::Accent)),
                        )
                        .child(chip(
                            strings
                                .shell_diag_presets(reshade.preset_count as i64)
                                .into(),
                            ChipTone::Accent,
                        ))
                        .child(chip(
                            strings
                                .shell_diag_shaders(
                                    info.map_or(0, |info| info.shader_count) as i64
                                )
                                .into(),
                            ChipTone::Accent,
                        ))
                        .when(shaders_missing, |this| {
                            this.child(chip(
                                strings.diagnostics_shaders_missing().into(),
                                ChipTone::Warning,
                            ))
                        }),
                )
                .into_any_element(),
            );
        }

        let migoto = &report.third_party.migoto;
        if migoto.installed {
            let info = migoto.migoto_info.as_ref();
            let fixes = info
                .map(|info| info.shader_fix_names.join(LIST_SEPARATOR))
                .filter(|names| !names.is_empty());
            cards.push(
                parts::section_card(
                    IconName::BlurOn,
                    strings.diagnostics_section_migoto(),
                    Color::Accent,
                    Some(badge(info.map_or(0, |info| info.shader_fix_count))),
                    window,
                    cx,
                )
                .child(
                    body()
                        .child(
                            chips()
                                .child(flag(
                                    strings.diagnostics_enabled().into(),
                                    migoto.enabled,
                                ))
                                .child(flag(
                                    strings.shell_diag_loader().into(),
                                    info.is_some_and(|info| info.loader_target_ok),
                                )),
                        )
                        .when_some(fixes, |this, fixes| this.child(parts::muted_line(fixes))),
                )
                .into_any_element(),
            );
        }

        let texture_body = if report.texture_packs_available {
            chips()
                .child(chip(
                    strings
                        .shell_diag_nams_list(report.texture_packs.len() as i64)
                        .into(),
                    ChipTone::Accent,
                ))
                .child(chip(
                    strings
                        .shell_diag_nams_inject(report.nams_textures.len() as i64)
                        .into(),
                    ChipTone::Muted,
                ))
                .when(!report.sk_res_textures.is_empty(), |this| {
                    this.child(chip(
                        strings
                            .shell_diag_sk_res(report.sk_res_textures.len() as i64)
                            .into(),
                        ChipTone::Warning,
                    ))
                })
                .when(!report.wax_textures.is_empty(), |this| {
                    this.child(chip(
                        strings
                            .shell_diag_wax(report.wax_textures.len() as i64)
                            .into(),
                        ChipTone::Warning,
                    ))
                })
                .into_any_element()
        } else {
            parts::muted_line(strings.diagnostics_texture_packs_unavailable()).into_any_element()
        };
        cards.push(
            parts::section_card(
                IconName::Texture,
                strings.diagnostics_section_texture_packs(),
                Color::Accent,
                report
                    .texture_packs_available
                    .then(|| badge(report.texture_packs.len())),
                window,
                cx,
            )
            .child(texture_body)
            .into_any_element(),
        );

        cards.push(
            parts::section_card(
                IconName::MovieCreationOutlined,
                strings.log_section_cutscenes(),
                Color::Accent,
                Some(badge(report.cutscene_mods.len())),
                window,
                cx,
            )
            .child(
                chips()
                    .child(flag(
                        strings.shell_diag_hd().into(),
                        report.cutscene_detection.has_hd_cutscenes,
                    ))
                    .child(flag(
                        strings.shell_diag_h264().into(),
                        report.cutscene_detection.needs_h264,
                    ))
                    .when(!report.direct_overrides.is_empty(), |this| {
                        this.child(chip(
                            strings
                                .shell_diag_movie_overrides(report.direct_overrides.len() as i64)
                                .into(),
                            ChipTone::Warning,
                        ))
                    }),
            )
            .into_any_element(),
        );

        if !report.vanilla_dropped.is_empty() {
            let mut list = body();
            for dropped in report.vanilla_dropped.iter().take(VANILLA_DROP_LIMIT) {
                list = list.child(kv(
                    dropped.path.clone().into(),
                    human_size(dropped.size_bytes).into(),
                ));
            }
            if report.vanilla_dropped.len() > VANILLA_DROP_LIMIT {
                list = list.child(more(report.vanilla_dropped.len() - VANILLA_DROP_LIMIT));
            }
            cards.push(
                parts::section_card(
                    IconName::FolderZipOutlined,
                    strings.diagnostics_section_vanilla_drops(),
                    Color::Warning,
                    Some(badge(report.vanilla_dropped.len())),
                    window,
                    cx,
                )
                .child(list)
                .into_any_element(),
            );
        }

        let deltas = report.config_deltas();
        if !deltas.is_empty() {
            let mut list = body();
            for delta in deltas.iter().take(CONFIG_DELTA_LIMIT) {
                list = list.child(kv(delta.key.clone().into(), delta.value.clone().into()));
            }
            if deltas.len() > CONFIG_DELTA_LIMIT {
                list = list.child(more(deltas.len() - CONFIG_DELTA_LIMIT));
            }
            cards.push(
                parts::section_card(
                    IconName::Tune,
                    strings.diagnostics_section_non_default(),
                    Color::Accent,
                    Some(badge(deltas.len())),
                    window,
                    cx,
                )
                .child(list)
                .into_any_element(),
            );
        }

        if !report.data_dir_contents.is_empty() {
            let sorted = largest_first(
                report
                    .data_dir_contents
                    .iter()
                    .map(|(name, count)| (name.clone(), *count)),
            );
            let mut list = body();
            for (name, count) in sorted.iter().take(DATA_DIR_LIMIT) {
                list = list.child(kv(
                    strings.shell_diag_dir_entry(name).into(),
                    strings.diagnostics_file_count(*count as i64).into(),
                ));
            }
            if sorted.len() > DATA_DIR_LIMIT {
                list = list.child(more(sorted.len() - DATA_DIR_LIMIT));
            }
            cards.push(
                parts::section_card(
                    IconName::WarningAmberRounded,
                    strings.diagnostics_section_data_overlay(),
                    Color::Warning,
                    None,
                    window,
                    cx,
                )
                .child(list)
                .into_any_element(),
            );
        }

        if !report.wax_mods_root.is_empty()
            || !report.sk_res_packs.is_empty()
            || !report.reshade_contents.is_empty()
        {
            let mut list = body();
            let groups: [(&'static str, &[String]); 3] = [
                (strings.shell_diag_wax_mods(), &report.wax_mods_root),
                (strings.shell_diag_sk_res_packs(), &report.sk_res_packs),
                (strings.shell_diag_reshade_dir(), &report.reshade_contents),
            ];
            for (label, items) in groups {
                if !items.is_empty() {
                    list = list.child(kv(label.into(), items.join(LIST_SEPARATOR).into()));
                }
            }
            cards.push(
                parts::section_card(
                    IconName::FolderOutlined,
                    strings.diagnostics_section_external_legacy(),
                    Color::Accent,
                    None,
                    window,
                    cx,
                )
                .child(list)
                .into_any_element(),
            );
        }

        if !report.game_root_extras.is_empty() {
            let mut list = body();
            for name in report.game_root_extras.iter().take(ROOT_EXTRA_LIMIT) {
                list = list.child(parts::muted_line(strings.shell_diag_bullet(name)));
            }
            if report.game_root_extras.len() > ROOT_EXTRA_LIMIT {
                list = list.child(more(report.game_root_extras.len() - ROOT_EXTRA_LIMIT));
            }
            cards.push(
                parts::section_card(
                    IconName::TravelExplore,
                    strings.diagnostics_section_game_root_extras(),
                    Color::Accent,
                    None,
                    window,
                    cx,
                )
                .child(list)
                .into_any_element(),
            );
        }

        (findings_card, cards)
    }
}

fn finding_row(
    index: usize,
    finding: &DiagnosticFinding,
    window: &Window,
    cx: &App,
) -> impl IntoElement + use<> {
    let sizes = window.sizes(cx);
    h_flex()
        .w_full()
        .items_start()
        .gap(sizes.spacing_md)
        .child(
            Icon::new(finding_icon(finding.level))
                .size(IconSize::Small)
                .color(finding_color(finding.level)),
        )
        .child(
            div().flex_1().min_w_0().child(
                SelectableText::new(("shell-diag-finding", index), finding.message.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Secondary),
            ),
        )
}

impl EventEmitter<DismissEvent> for DiagnosticsDialog {}

impl Focusable for DiagnosticsDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for DiagnosticsDialog {}

impl Render for DiagnosticsDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let version = self
            .report
            .launcher_value(LAUNCHER_VERSION_KEY)
            .unwrap_or(UNKNOWN)
            .to_owned();
        let subtitle =
            strings.shell_diag_header_line(&version, &local_time(self.report.generated_at));
        let (findings, cards) = self.cards(window, cx);
        let gap = sizes.spacing_lg;
        let mut left = v_flex().w_full().gap(gap);
        let mut right = v_flex().w_full().gap(gap);
        for (index, card) in cards.into_iter().enumerate() {
            if index % 2 == 0 {
                left = left.child(card);
            } else {
                right = right.child(card);
            }
        }

        let header = h_flex()
            .w_full()
            .flex_none()
            .gap(sizes.spacing_lg)
            .px(sizes.padding_lg)
            .py(sizes.padding_md)
            .rounded_t(sizes.border_radius)
            .bg(theme.colors.surface_medium)
            .border_b_1()
            .border_color(theme.colors.border_light)
            .child(parts::header_tile(
                IconName::MedicalInformationOutlined,
                Color::Accent,
                cx,
            ))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        Label::new(strings.diagnostics_title())
                            .size(LabelSize::Large)
                            .bold()
                            .uppercase(),
                    )
                    .child(
                        Label::new(subtitle)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
            );

        let body = div()
            .id("shell-diagnostics-scroll")
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .px(sizes.padding_lg)
            .py(sizes.padding_md)
            .child(
                v_flex()
                    .w_full()
                    .max_w(metrics::DIAGNOSTICS_MAX_WIDTH)
                    .gap(gap)
                    .child(findings)
                    .child(
                        TwoColumn::new(left, right)
                            .id("shell-diagnostics-columns")
                            .breakpoint(metrics::DIAGNOSTICS_TWO_COLUMN_MIN_WIDTH),
                    ),
            )
            .overflow_y_scrollbar(&self.scroll);

        let footer = h_flex()
            .w_full()
            .flex_none()
            .gap(sizes.spacing_lg)
            .px(sizes.padding_lg)
            .py(sizes.padding_sm)
            .border_t_1()
            .border_color(theme.colors.border_light)
            .child(
                Button::new("shell-diagnostics-copy", strings.diagnostics_copy())
                    .style(ButtonStyle::Tinted(TintColor::Accent))
                    .size(ButtonSize::Medium)
                    .start_icon(IconName::ContentCopy)
                    .on_click(cx.listener(|this, _, _, cx| this.copy_summary(cx))),
            )
            .child(
                Button::new("shell-diagnostics-save", strings.diagnostics_save_full())
                    .style(ButtonStyle::Filled)
                    .size(ButtonSize::Medium)
                    .start_icon(IconName::Save)
                    .loading(self.saving)
                    .on_click(cx.listener(|this, _, _, cx| this.save_full_report(cx))),
            )
            .child(div().flex_1())
            .child(
                Button::new("shell-diagnostics-close", strings.diagnostics_close())
                    .style(ButtonStyle::Subtle)
                    .size(ButtonSize::Medium)
                    .color(Color::Muted)
                    .on_click(|_, window, cx| {
                        window.close_modal(cx);
                    }),
            );

        div().track_focus(&self.focus_handle).child(
            Modal::new("shell-diagnostics", None)
                .width(ModalWidth::Full)
                .padded(false)
                .fill_height()
                .section(
                    v_flex()
                        .size_full()
                        .min_h_0()
                        .child(header)
                        .child(body)
                        .child(footer),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_kinds_are_counted_in_first_seen_order() {
        let counts = mod_kind_counts([
            ModKind::Data,
            ModKind::Unknown,
            ModKind::Data,
            ModKind::Native,
            ModKind::Data,
        ]);
        assert_eq!(
            counts,
            vec![
                (ModKind::Data, 3),
                (ModKind::Unknown, 1),
                (ModKind::Native, 1)
            ]
        );
        assert!(mod_kind_counts([]).is_empty());
    }

    #[test]
    fn data_folders_are_listed_largest_first() {
        let sorted = largest_first([
            ("em".to_owned(), 2),
            ("pl".to_owned(), 40),
            ("wp".to_owned(), 7),
        ]);
        let names: Vec<&str> = sorted.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, vec!["pl", "wp", "em"]);
    }

    #[test]
    fn report_time_is_formatted_as_date_and_time() {
        let stamp: Timestamp = "2026-11-06T10:20:30Z".parse().unwrap();
        let text = local_time(stamp);
        assert_eq!(text.len(), "2026-11-06 10:20:30".len());
        assert!(text.starts_with("2026-11-0"));
    }
}
