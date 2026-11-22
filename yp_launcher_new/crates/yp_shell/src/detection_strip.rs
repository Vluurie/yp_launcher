use gpui::{App, CursorStyle, IntoElement, RenderOnce, SharedString, Window};
use yp_host::game_probe::ExeVariant;
use yp_i18n::Strings;
use yp_state::DetectionSnapshot;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{ButtonSize, ButtonStyle, IconSize, Sizes, TintColor, Tooltip};

const CODEC_H264: &str = "H264";
const CODEC_MPEG2: &str = "MPEG-2";
const PART_SEPARATOR: &str = ", ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionKind {
    Exe,
    Dlc,
    LodMod,
    ReShade,
    Textures,
    Mods,
    Cutscene,
}

impl DetectionKind {
    pub const ALL: [DetectionKind; 7] = [
        DetectionKind::Exe,
        DetectionKind::Dlc,
        DetectionKind::LodMod,
        DetectionKind::ReShade,
        DetectionKind::Textures,
        DetectionKind::Mods,
        DetectionKind::Cutscene,
    ];

    pub fn icon(self) -> IconName {
        match self {
            DetectionKind::Exe => IconName::Memory,
            DetectionKind::Dlc => IconName::VideogameAssetOutlined,
            DetectionKind::LodMod => IconName::ImageOutlined,
            DetectionKind::ReShade => IconName::AutoFixHigh,
            DetectionKind::Textures => IconName::Texture,
            DetectionKind::Mods => IconName::ExtensionOutlined,
            DetectionKind::Cutscene => IconName::MovieCreationOutlined,
        }
    }

    fn element_id(self) -> &'static str {
        match self {
            DetectionKind::Exe => "shell-detection-exe",
            DetectionKind::Dlc => "shell-detection-dlc",
            DetectionKind::LodMod => "shell-detection-lodmod",
            DetectionKind::ReShade => "shell-detection-reshade",
            DetectionKind::Textures => "shell-detection-textures",
            DetectionKind::Mods => "shell-detection-mods",
            DetectionKind::Cutscene => "shell-detection-cutscene",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionState {
    Detected,
    Absent,
    Warning,
}

impl DetectionState {
    pub fn tint(self) -> TintColor {
        match self {
            DetectionState::Detected => TintColor::Success,
            DetectionState::Absent => TintColor::Muted,
            DetectionState::Warning => TintColor::Warning,
        }
    }

    pub fn badge(self) -> IconName {
        match self {
            DetectionState::Detected => IconName::CheckCircle,
            DetectionState::Absent => IconName::RemoveCircleOutline,
            DetectionState::Warning => IconName::WarningAmberRounded,
        }
    }

    fn detected(found: bool) -> Self {
        if found {
            DetectionState::Detected
        } else {
            DetectionState::Absent
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionChip {
    pub kind: DetectionKind,
    pub state: DetectionState,
    pub label: SharedString,
    pub detail: Option<SharedString>,
}

fn fixed(text: &'static str) -> SharedString {
    SharedString::new_static(text)
}

pub fn chip_for(
    kind: DetectionKind,
    snapshot: &DetectionSnapshot,
    strings: &Strings,
) -> DetectionChip {
    let (state, label, detail) = match kind {
        DetectionKind::Exe => match snapshot.exe_variant {
            ExeVariant::WolfLimitBreak => (
                DetectionState::Warning,
                fixed(strings.detection_exe_wolf_limit_break()),
                Some(fixed(strings.detection_exe_wolf_limit_break_tooltip())),
            ),
            ExeVariant::LegacyWindows7 => (
                DetectionState::Warning,
                fixed(strings.detection_exe_legacy_win7()),
                Some(fixed(strings.detection_exe_legacy_win7_tooltip())),
            ),
            ExeVariant::Original => (
                DetectionState::Detected,
                fixed(strings.detection_exe_original()),
                None,
            ),
            ExeVariant::Missing => (
                DetectionState::Absent,
                fixed(strings.detection_exe_missing()),
                None,
            ),
            ExeVariant::Unknown => (
                DetectionState::Absent,
                fixed(strings.detection_exe_unrecognised()),
                Some(fixed(strings.detection_exe_unrecognised_tooltip())),
            ),
        },
        DetectionKind::Dlc => {
            if snapshot.dlc {
                (
                    DetectionState::Detected,
                    fixed(strings.detection_dlc_present()),
                    Some(fixed(strings.detection_dlc_present_tooltip())),
                )
            } else {
                (
                    DetectionState::Absent,
                    fixed(strings.detection_dlc_not_detected()),
                    Some(fixed(strings.detection_dlc_not_detected_tooltip())),
                )
            }
        }
        DetectionKind::LodMod => (
            DetectionState::detected(snapshot.lodmod_enabled),
            fixed(if snapshot.lodmod_enabled {
                strings.chip_lod_mod_on()
            } else {
                strings.chip_lod_mod_off()
            }),
            None,
        ),
        DetectionKind::ReShade => (
            DetectionState::detected(snapshot.reshade_detected()),
            fixed(strings.chip_re_shade()),
            None,
        ),
        DetectionKind::Textures => {
            if snapshot.has_textures() {
                let mut parts = Vec::new();
                if snapshot.textures.active_packs > 0 {
                    parts.push(strings.chip_injected_count(snapshot.textures.active_packs as i64));
                }
                if snapshot.textures.has_sk_res {
                    parts.push(strings.chip_sk_res().to_owned());
                }
                (
                    DetectionState::Detected,
                    strings
                        .chip_textures_count(&parts.join(PART_SEPARATOR))
                        .into(),
                    None,
                )
            } else {
                (
                    DetectionState::Absent,
                    fixed(strings.chip_no_textures()),
                    None,
                )
            }
        }
        DetectionKind::Mods => {
            if snapshot.has_mods() {
                (
                    DetectionState::Detected,
                    strings.chip_mods_count(snapshot.mods as i64).into(),
                    None,
                )
            } else {
                (DetectionState::Absent, fixed(strings.chip_no_mods()), None)
            }
        }
        DetectionKind::Cutscene => {
            if snapshot.has_hd_cutscenes() {
                let codec = if snapshot.cutscenes.needs_h264 {
                    CODEC_H264
                } else {
                    CODEC_MPEG2
                };
                (
                    DetectionState::Detected,
                    strings
                        .chip_cutscene_mod(
                            i64::from(snapshot.cutscenes.largest_width),
                            i64::from(snapshot.cutscenes.largest_height),
                            codec,
                        )
                        .into(),
                    None,
                )
            } else {
                (
                    DetectionState::Absent,
                    fixed(strings.chip_no_cutscene_mod()),
                    None,
                )
            }
        }
    };
    DetectionChip {
        kind,
        state,
        label,
        detail,
    }
}

pub fn chips(snapshot: &DetectionSnapshot, strings: &Strings) -> Vec<DetectionChip> {
    DetectionKind::ALL
        .into_iter()
        .map(|kind| chip_for(kind, snapshot, strings))
        .collect()
}

fn render_chip(chip: DetectionChip, window: &Window, cx: &App) -> impl IntoElement {
    let theme = cx.theme().clone();
    let sizes = window.sizes(cx);
    let tint = chip.state.tint();
    let tooltip = match chip.detail {
        Some(detail) => Tooltip::new(chip.label).meta(detail),
        None => Tooltip::new(chip.label),
    };
    div()
        .relative()
        .flex_none()
        .child(
            IconButton::new(chip.kind.element_id(), chip.kind.icon())
                .style(ButtonStyle::Tinted(tint))
                .size(ButtonSize::Medium)
                .icon_size(IconSize::Medium)
                .cursor_style(CursorStyle::Arrow)
                .tooltip(tooltip.build()),
        )
        .child(
            div()
                .absolute()
                .right(sizes.spacing_xs)
                .bottom(sizes.spacing_xs)
                .rounded(Sizes::INDICATOR_SIZE)
                .bg(theme.colors.background_card)
                .child(
                    Icon::new(chip.state.badge())
                        .size(IconSize::XSmall)
                        .color(tint.color()),
                ),
        )
}

#[derive(Default)]
pub struct DetectionStrip;

yp_ui::impl_into_element!(DetectionStrip);

impl DetectionStrip {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for DetectionStrip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let stores = cx.stores();
        let selected = stores.session.read(cx).is_directory_selected();
        let chips = stores
            .detection
            .read(cx)
            .snapshot()
            .filter(|_| selected)
            .map(|snapshot| chips(snapshot, strings))
            .unwrap_or_default();
        if chips.is_empty() {
            return div().into_any_element();
        }
        let mut strip = v_flex()
            .flex_none()
            .gap(sizes.spacing_md)
            .px(sizes.chip_padding_h)
            .py(sizes.chip_padding_h)
            .rounded(sizes.border_radius)
            .bg(theme.colors.background_card.opacity(theme.alpha.glass))
            .border_1()
            .border_color(theme.colors.border_light);
        for chip in chips {
            strip = strip.child(render_chip(chip, window, cx));
        }
        strip.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_cutscenes::CutsceneDetection;
    use yp_i18n::Locale;
    use yp_state::TextureChip;
    use yp_thirdparty::ReShadeStatus;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    fn empty() -> DetectionSnapshot {
        DetectionSnapshot {
            dlc: false,
            exe_variant: ExeVariant::Missing,
            lodmod_enabled: false,
            reshade: ReShadeStatus::NotFound,
            textures: TextureChip::default(),
            mods: 0,
            cutscenes: CutsceneDetection::default(),
        }
    }

    #[test]
    fn nothing_installed_reads_as_absent_everywhere() {
        let chips = chips(&empty(), strings());
        assert_eq!(chips.len(), 7);
        assert!(
            chips
                .iter()
                .all(|chip| chip.state == DetectionState::Absent)
        );
        assert_eq!(chips[0].label.as_ref(), strings().detection_exe_missing());
        assert_eq!(chips[5].label.as_ref(), strings().chip_no_mods());
    }

    #[test]
    fn unusual_executables_warn_with_an_explanation() {
        let mut snapshot = empty();
        snapshot.exe_variant = ExeVariant::LegacyWindows7;
        let chip = chip_for(DetectionKind::Exe, &snapshot, strings());
        assert_eq!(chip.state, DetectionState::Warning);
        assert!(chip.detail.is_some());
        snapshot.exe_variant = ExeVariant::Original;
        let chip = chip_for(DetectionKind::Exe, &snapshot, strings());
        assert_eq!(chip.state, DetectionState::Detected);
        assert_eq!(chip.detail, None);
    }

    #[test]
    fn texture_chip_lists_injected_packs_and_sk_res() {
        let mut snapshot = empty();
        snapshot.textures = TextureChip {
            active_packs: 3,
            has_sk_res: true,
        };
        let chip = chip_for(DetectionKind::Textures, &snapshot, strings());
        assert_eq!(chip.state, DetectionState::Detected);
        assert_eq!(chip.label.as_ref(), "Textures (3 injected, SK_Res)");
    }

    #[test]
    fn cutscene_chip_names_resolution_and_codec() {
        let mut snapshot = empty();
        snapshot.cutscenes = CutsceneDetection {
            has_hd_cutscenes: true,
            needs_h264: true,
            files_scanned: 2,
            largest_width: 3840,
            largest_height: 2160,
        };
        let chip = chip_for(DetectionKind::Cutscene, &snapshot, strings());
        assert_eq!(chip.label.as_ref(), "Cutscene Mod (3840x2160 H264)");
        snapshot.mods = 1;
        let chip = chip_for(DetectionKind::Mods, &snapshot, strings());
        assert_eq!(chip.label.as_ref(), "1 mod");
    }
}
