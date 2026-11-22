use serde_json::Value as Json;
use yp_config::Setting;
use yp_config::schema::is_in_category;
use yp_i18n::Strings;
use yp_state::{ConfigStore, StateExt};
use yp_ui::prelude::*;
use yp_ui::{
    Button, ButtonStyle, Callout, IconName, ImageComparison, Label, LabelSize, PreviewImage, TintColor, Tooltip,
    asset_image,
};

use crate::page::{muted_text, note};

pub type Text = fn(&Strings) -> &'static str;

#[derive(Debug, Clone, Copy)]
pub enum When {
    On(&'static str),
    Off(&'static str),
    Empty(&'static str),
    Check(fn(&ConfigStore) -> bool),
    Both(&'static When, &'static When),
}

impl When {
    pub fn holds(self, store: &ConfigStore) -> bool {
        match self {
            When::On(id) => store.setting(id).is_none() || store.bool(id),
            When::Off(id) => store.setting(id).is_none() || !store.bool(id),
            When::Empty(id) => store.setting(id).is_none() || store.text(id).is_empty(),
            When::Check(check) => check(store),
            When::Both(first, second) => first.holds(store) && second.holds(store),
        }
    }
}

pub struct Shown {
    pub id: &'static str,
    pub when: When,
}

pub struct FileGate {
    pub file: &'static str,
    pub switch: &'static str,
    pub when: When,
    pub independent: &'static [&'static str],
}

impl FileGate {
    fn hides(&self, setting: &Setting, store: &ConfigStore) -> bool {
        self.file == setting.file
            && self.switch != setting.id
            && !self
                .independent
                .iter()
                .any(|category| is_in_category(&setting.category, category))
            && !self.when.holds(store)
    }
}

pub struct Lock {
    pub id: &'static str,
    pub when: When,
    pub reason: Text,
}

#[derive(Clone, Copy)]
pub enum Extra {
    Comparison {
        key: &'static str,
        before: &'static str,
        after: &'static str,
        before_label: Text,
        after_label: Text,
    },
    Preview {
        key: &'static str,
        image: &'static str,
        label: Text,
    },
    Note {
        when: When,
        text: Text,
    },
    Warning {
        key: &'static str,
        when: When,
        text: Text,
    },
    Apply {
        key: &'static str,
        id: &'static str,
        value: i64,
        label: Text,
        tooltip: Text,
    },
}

pub struct Attached {
    pub id: &'static str,
    pub extra: Extra,
}

#[derive(Clone, Copy)]
pub enum Intro {
    Muted(Text),
    HighGridsWip,
    Accent {
        key: &'static str,
        text: Text,
    },
}

pub const MANAGED_TEXTURE_IDS: [&str; 2] = [
    "texture_injection.load_order",
    "texture_injection.disabled_packs",
];

pub const CONTENT_GRID_PREFIX: &str = "nams.content.";

pub const SHOWN: &[Shown] = &[
    Shown {
        id: "lodmod.layout_spawn_range",
        when: When::Off("lodmod.layout_spawn_everything"),
    },
    Shown {
        id: "lodmod.msaa_per_pixel_terrain",
        when: When::On("lodmod.msaa_per_pixel"),
    },
    Shown {
        id: "lodmod.high_grids.rings",
        when: When::On("lodmod.high_grids.enabled"),
    },
    Shown {
        id: "lodmod.high_grids.room_rings",
        when: When::On("lodmod.high_grids.enabled"),
    },
    Shown {
        id: "lodmod.high_grids.blocked_in_room",
        when: When::On("lodmod.high_grids.enabled"),
    },
    Shown {
        id: "lodmod.high_grids.blocked_from_grid",
        when: When::On("lodmod.high_grids.enabled"),
    },
    Shown {
        id: "lodmod.high_grids.hidden_meshes",
        when: When::On("lodmod.high_grids.enabled"),
    },
    Shown {
        id: "lodmod.d3d12.ao_radius",
        when: When::On(RTAO_ID),
    },
    Shown {
        id: "lodmod.d3d12.ao_strength",
        when: When::On(RTAO_ID),
    },
    Shown {
        id: "lodmod.d3d12.ao_samples",
        when: When::On(RTAO_ID),
    },
];

pub const FILE_GATES: &[FileGate] = &[
    FileGate {
        file: "lodmod.toml",
        switch: "lodmod.enabled",
        when: When::On("lodmod.enabled"),
        independent: &["graphics/framerate", "graphics/backend", "graphics/d3d12"],
    },
    FileGate {
        file: "naiom.toml",
        switch: "nams.disable_input_features",
        when: When::Off("nams.disable_input_features"),
        independent: &[],
    },
];

pub const LOCKS: &[Lock] = &[
    Lock {
        id: "plugins.yorha_protocol.cheatsConfig.damageMultiplier",
        when: When::Off("plugins.yorha_protocol.firstPlaythrough"),
        reason: Strings::cheats_need_first_playthrough_off,
    },
    Lock {
        id: "plugins.yorha_protocol.cheatsConfig.syncEnemyLevels",
        when: When::Off("plugins.yorha_protocol.firstPlaythrough"),
        reason: Strings::cheats_need_first_playthrough_off,
    },
    Lock {
        id: "plugins.yorha_protocol.cheatsConfig.infiniteHealth",
        when: When::Off("plugins.yorha_protocol.firstPlaythrough"),
        reason: Strings::cheats_need_first_playthrough_off,
    },
    Lock {
        id: "plugins.yorha_protocol.cheatsConfig.infiniteJump",
        when: When::Off("plugins.yorha_protocol.firstPlaythrough"),
        reason: Strings::cheats_need_first_playthrough_off,
    },
    Lock {
        id: "plugins.yorha_protocol.cheatsConfig.noPodCooldown",
        when: When::Off("plugins.yorha_protocol.firstPlaythrough"),
        reason: Strings::cheats_need_first_playthrough_off,
    },
    Lock {
        id: "plugins.yorha_protocol.cheatsConfig.infiniteAirDash",
        when: When::Off("plugins.yorha_protocol.firstPlaythrough"),
        reason: Strings::cheats_need_first_playthrough_off,
    },
    Lock {
        id: "naiom.sensitivity",
        when: When::On("naiom.fix_camera_acceleration"),
        reason: Strings::naiom_needs_camera_fix,
    },
    Lock {
        id: "naiom.third_person_char_follow",
        when: When::On("naiom.third_person_mode"),
        reason: Strings::naiom_needs_third_person,
    },
    Lock {
        id: "naiom.third_person_smoothing",
        when: When::On("naiom.third_person_mode"),
        reason: Strings::naiom_needs_third_person,
    },
    Lock {
        id: "naiom.third_person_sensitivity_x",
        when: When::On("naiom.third_person_mode"),
        reason: Strings::naiom_needs_third_person,
    },
    Lock {
        id: "naiom.third_person_sensitivity_y",
        when: When::On("naiom.third_person_mode"),
        reason: Strings::naiom_needs_third_person,
    },
    Lock {
        id: "naiom.aim_crosshair",
        when: When::On("naiom.aim_mode"),
        reason: Strings::naiom_needs_aim_mode,
    },
    Lock {
        id: "naiom.aim_crosshair_always",
        when: When::On("naiom.aim_mode"),
        reason: Strings::naiom_needs_aim_mode,
    },
    Lock {
        id: "naiom.aim_crosshair_always",
        when: When::On("naiom.aim_crosshair"),
        reason: Strings::naiom_needs_crosshair,
    },
    Lock {
        id: "naiom.aim_sensitivity",
        when: When::On("naiom.aim_mode"),
        reason: Strings::naiom_needs_aim_mode,
    },
    Lock {
        id: "naiom.aim_sensitivity",
        when: When::Off("naiom.aim_crosshair"),
        reason: Strings::naiom_crosshair_overrides,
    },
    Lock {
        id: "naiom.aim_output_multiplier",
        when: When::On("naiom.aim_mode"),
        reason: Strings::naiom_needs_aim_mode,
    },
    Lock {
        id: "naiom.aim_output_multiplier",
        when: When::Off("naiom.aim_crosshair"),
        reason: Strings::naiom_crosshair_overrides,
    },
    Lock {
        id: "naiom.aim_sensitivity_x",
        when: When::On("naiom.aim_mode"),
        reason: Strings::naiom_needs_aim_mode,
    },
    Lock {
        id: "naiom.aim_sensitivity_y",
        when: When::On("naiom.aim_mode"),
        reason: Strings::naiom_needs_aim_mode,
    },
];

const fn comparison(
    id: &'static str,
    key: &'static str,
    before: &'static str,
    after: &'static str,
    before_label: Text,
    after_label: Text,
) -> Attached {
    Attached {
        id,
        extra: Extra::Comparison {
            key,
            before,
            after,
            before_label,
            after_label,
        },
    }
}

pub const AFTER: &[Attached] = &[
    Attached {
        id: GRAPHICS_BACKEND_ID,
        extra: Extra::Warning {
            key: "nams-d3d12-experimental",
            when: When::Check(d3d12_active),
            text: Strings::graphics_backend_d3d12_warning,
        },
    },
    comparison(
        "lodmod.enabled",
        "lodmod-cmp-master",
        "lod_mod_off.webp",
        "lod_mod_on.webp",
        Strings::comparison_vanilla,
        Strings::comparison_default_enabled,
    ),
    comparison(
        "lodmod.disable_vignette",
        "lodmod-cmp-vignette",
        "vignett_on.webp",
        "vignett_off.webp",
        Strings::comparison_vignette_on,
        Strings::comparison_vignette_off,
    ),
    comparison(
        "lodmod.fxaa",
        "lodmod-cmp-fxaa",
        "fxaa_off.webp",
        "fxaa_on.webp",
        Strings::comparison_off,
        Strings::comparison_on,
    ),
    comparison(
        "lodmod.msaa_shadow_mask_fix",
        "lodmod-cmp-msaa-mask",
        "msaa8_vanilla.webp",
        "msaa8_shadow_mask_fix.webp",
        Strings::comparison_off,
        Strings::comparison_on,
    ),
    comparison(
        "lodmod.msaa_per_pixel",
        "lodmod-cmp-msaa-per-pixel",
        "msaa8_both_fixes.webp",
        "msaa8_per_pixel.webp",
        Strings::comparison_off,
        Strings::comparison_on,
    ),
    Attached {
        id: "lodmod.bloom_reference_height",
        extra: Extra::Apply {
            key: "lodmod-bloom-2017",
            id: "lodmod.bloom_reference_height",
            value: 900,
            label: Strings::bloom2017_button,
            tooltip: Strings::bloom2017_tooltip,
        },
    },
    comparison(
        "lodmod.bloom_reference_height",
        "lodmod-cmp-bloom",
        "bloom_2021.webp",
        "bloom_2017.webp",
        Strings::comparison_bloom2021,
        Strings::comparison_bloom2017,
    ),
    comparison(
        "lodmod.gi_workgroup_size",
        "lodmod-cmp-gi",
        "16_cube.webp",
        "128_cube.webp",
        Strings::comparison_cubemaps16,
        Strings::comparison_cubemaps128,
    ),
    comparison(
        "lodmod.shadow_resolution",
        "lodmod-cmp-shadow-res",
        "shadow_resolution_2148.webp",
        "shadow_resolution_8192.webp",
        Strings::comparison2048,
        Strings::comparison8192,
    ),
    comparison(
        "lodmod.shadow_distance_multiplier",
        "lodmod-cmp-shadow-distance",
        "shadow_distance_default.webp",
        "shadow_distance_multiplier_2.0.webp",
        Strings::comparison_default,
        Strings::comparison20x,
    ),
    comparison(
        "lodmod.shadow_distance_pss",
        "lodmod-cmp-shadow-pss",
        "shadow_dist_filter_off.webp",
        "pss_distance_shadow_-5.0.webp",
        Strings::comparison_default,
        Strings::comparison_pss_minus5,
    ),
    comparison(
        "lodmod.shadow_filter_strength_bias",
        "lodmod-cmp-shadow-bias",
        "shadow_dist_filter_off.webp",
        "pss_distance_shadow_-5.0.webp",
        Strings::comparison_default,
        Strings::comparison_bias_minus5,
    ),
    comparison(
        "lodmod.shadow_filter_strength_minimum",
        "lodmod-cmp-shadow-filter",
        "shadow_filter_min_off.webp",
        "shadow_filter_min_3.0.webp",
        Strings::comparison_off,
        Strings::comparison30,
    ),
    comparison(
        "lodmod.shadow_model_force_all",
        "lodmod-cmp-shadow-models",
        "hd_shadow_models_off.webp",
        "hq_shadow_models_on.webp",
        Strings::comparison_off,
        Strings::comparison_hq_force_all,
    ),
    comparison(
        "lodmod.high_grids.rings",
        "lodmod-cmp-high-grids",
        "Vanilla_High_grids_7.webp",
        "YP_High_grids_61.webp",
        Strings::comparison_high_grids7,
        Strings::comparison_high_grids61,
    ),
    Attached {
        id: "naiom.aim_crosshair",
        extra: Extra::Preview {
            key: "naiom-preview-crosshair",
            image: "images/config/crosshair.webp",
            label: Strings::naiom_crosshair_preview_label,
        },
    },
    Attached {
        id: "naiom.aim_crosshair_always",
        extra: Extra::Note {
            when: When::On("naiom.aim_crosshair"),
            text: Strings::naiom_crosshair_note,
        },
    },
    Attached {
        id: "naiom.movement_disable_tap_evade",
        extra: Extra::Warning {
            key: "naiom-tap-evade-warning",
            when: When::Both(
                &When::On("naiom.movement_disable_tap_evade"),
                &When::Empty("naiom.bindings.non_standard_evade"),
            ),
            text: Strings::naiom_tap_evade_warning,
        },
    },
];

pub const INTROS: &[(&str, Intro)] = &[
    ("graphics/performance", Intro::Muted(Strings::performance_card_note)),
    ("graphics/high_grids", Intro::HighGridsWip),
    ("content", Intro::Muted(Strings::content_features_description)),
    (
        "input/keyboard",
        Intro::Accent {
            key: "naiom-bindings-intro",
            text: Strings::naiom_bindings_intro,
        },
    ),
];

pub const ZERO_LABELS: &[(&str, Text)] = &[
    ("lodmod.bloom_reference_height", Strings::bloom_reference_vanilla),
    ("lodmod.bloom_kernel_reference_height", Strings::bloom_reference_vanilla),
];

pub type OptionLabel = fn(&Strings, &Json) -> Option<String>;
pub type ItemLabel = fn(&Strings, i64) -> String;

const OPTION_TOLERANCE: f64 = 1e-6;

fn is_option(value: f64, option: f64) -> bool {
    (value - option).abs() < OPTION_TOLERANCE
}

fn lod_multiplier_label(strings: &Strings, value: &Json) -> Option<String> {
    let value = value.as_f64()?;
    let label = if is_option(value, 0.0) {
        strings.lod_multiplier_option_quality()
    } else if is_option(value, 0.75) {
        strings.lod_multiplier_option_performance()
    } else if is_option(value, 1.0) {
        strings.lod_multiplier_option_vanilla()
    } else if is_option(value, 10.0) {
        strings.lod_multiplier_option_far()
    } else {
        return None;
    };
    Some(label.to_owned())
}

fn bloom_drop_label(strings: &Strings, value: &Json) -> Option<String> {
    let value = value.as_f64()?;
    let label = if is_option(value, 0.0) {
        strings.bloom_drop_levels_off()
    } else if is_option(value, 1.0) {
        strings.bloom_drop_levels_widest()
    } else if is_option(value, 2.0) {
        strings.bloom_drop_levels_two_widest()
    } else {
        return None;
    };
    Some(label.to_owned())
}

pub fn high_grids_cells(rings: i64) -> i64 {
    3 * rings * (rings + 1) + 1
}

fn high_grids_rings_label(strings: &Strings, value: &Json) -> Option<String> {
    let rings = value.as_f64()?.round() as i64;
    (rings >= 1).then(|| strings.high_grids_cell_count(rings, high_grids_cells(rings)))
}

fn vram_budget_label(strings: &Strings, value: &Json) -> Option<String> {
    is_option(value.as_f64()?, 0.0).then(|| strings.texture_auto_recommended().to_owned())
}

const GRAPHICS_BACKEND_ID: &str = "nams.graphics_backend";
const D3D12_SWITCH_ID: &str = "lodmod.d3d12_backend";
const D3D11_BACKEND: &str = "d3d11";
const D3D12_BACKEND: &str = "d3d12";

pub struct Override {
    pub id: &'static str,
    pub by: &'static str,
    pub value: fn(&Json) -> Option<Json>,
}

fn backend_of_switch(switch: &Json) -> Option<Json> {
    switch
        .as_bool()
        .map(|on| Json::from(if on { D3D12_BACKEND } else { D3D11_BACKEND }))
}

pub const OVERRIDES: &[Override] = &[Override {
    id: GRAPHICS_BACKEND_ID,
    by: D3D12_SWITCH_ID,
    value: backend_of_switch,
}];

pub fn shown_value(id: &str, store: &ConfigStore) -> Json {
    OVERRIDES
        .iter()
        .filter(|rule| rule.id == id)
        .find_map(|rule| (rule.value)(&store.value(rule.by)))
        .unwrap_or_else(|| store.value(id))
}

pub fn overrides_to_clear(id: &str, store: &ConfigStore) -> Vec<&'static str> {
    OVERRIDES
        .iter()
        .filter(|rule| rule.id == id && !store.value(rule.by).is_null())
        .map(|rule| rule.by)
        .collect()
}

fn is_override(setting: &Setting) -> bool {
    OVERRIDES.iter().any(|rule| rule.by == setting.id)
}

pub fn d3d12_active(store: &ConfigStore) -> bool {
    match store.value(D3D12_SWITCH_ID).as_bool() {
        Some(on) => on,
        None => store.text(GRAPHICS_BACKEND_ID) == D3D12_BACKEND,
    }
}

const RTAO_ID: &str = "lodmod.d3d12.ambient_occlusion";

pub const CATEGORY_SHOWN: &[(&str, When)] = &[("graphics/d3d12", When::Check(d3d12_active))];

pub const ONLY_SHOWN: &[(&str, &[&str])] = &[(
    "graphics/d3d12",
    &[
        RTAO_ID,
        "lodmod.d3d12.ao_radius",
        "lodmod.d3d12.ao_strength",
        "lodmod.d3d12.ao_samples",
    ],
)];

fn graphics_backend_label(strings: &Strings, value: &Json) -> Option<String> {
    (value.as_str() == Some(D3D12_BACKEND)).then(|| strings.graphics_backend_d3d12_experimental().to_owned())
}

pub const OPTIONS: &[(&str, &[f64])] = &[(
    "texture_injection.vram_budget_mb",
    &[0., 1024., 2048., 4096., 6144., 8192., 12288., 16384.],
)];

pub const OPTION_LABELS: &[(&str, OptionLabel)] = &[
    ("lodmod.lod_multiplier", lod_multiplier_label),
    ("lodmod.bloom_drop_coarse_levels", bloom_drop_label),
    ("lodmod.high_grids.rings", high_grids_rings_label),
    ("texture_injection.vram_budget_mb", vram_budget_label),
    (GRAPHICS_BACKEND_ID, graphics_backend_label),
];

pub const ITEM_LABELS: &[(&str, ItemLabel)] = &[("lodmod.shadow_blur_scale", Strings::shadow_blur_cascade)];

pub fn options(id: &str) -> Option<&'static [f64]> {
    OPTIONS
        .iter()
        .find(|(candidate, _)| *candidate == id)
        .map(|(_, options)| *options)
}

pub fn option_label(id: &str) -> Option<OptionLabel> {
    OPTION_LABELS
        .iter()
        .find(|(candidate, _)| *candidate == id)
        .map(|(_, label)| *label)
}

pub fn item_label(id: &str) -> Option<ItemLabel> {
    ITEM_LABELS
        .iter()
        .find(|(candidate, _)| *candidate == id)
        .map(|(_, label)| *label)
}

pub const LIST_LABELS: &[(&str, Text, Text)] = &[(
    "lodmod.shadow_force_all_filters",
    Strings::shadow_force_all_filters_add,
    Strings::shadow_force_all_filters_empty,
)];

pub const COLUMN_LABELS: &[(&str, Text)] = &[
    ("room", Strings::grid_rule_room),
    ("rings", Strings::grid_rule_rings),
    ("grid", Strings::grid_rule_grid),
    ("from", Strings::grid_rule_from_grid),
    ("mesh", Strings::grid_rule_mesh),
    ("from_rings", Strings::grid_rule_from_rings),
];

pub fn is_shown(setting: &Setting, store: &ConfigStore) -> bool {
    let gated = FILE_GATES.iter().any(|gate| gate.hides(setting, store));
    !gated
        && !is_override(setting)
        && CATEGORY_SHOWN
            .iter()
            .filter(|(category, _)| is_in_category(&setting.category, category))
            .all(|(_, when)| when.holds(store))
        && ONLY_SHOWN
            .iter()
            .filter(|(category, _)| setting.category == *category)
            .all(|(_, ids)| ids.contains(&setting.id.as_str()))
        && SHOWN
            .iter()
            .filter(|rule| rule.id == setting.id)
            .all(|rule| rule.when.holds(store))
}

pub fn lock(setting: &Setting, store: &ConfigStore) -> Option<Text> {
    LOCKS
        .iter()
        .filter(|lock| lock.id == setting.id)
        .find(|lock| !lock.when.holds(store))
        .map(|lock| lock.reason)
}

pub fn zero_label(id: &str) -> Option<Text> {
    ZERO_LABELS
        .iter()
        .find(|(candidate, _)| *candidate == id)
        .map(|(_, text)| *text)
}

pub fn list_labels(id: &str) -> Option<(Text, Text)> {
    LIST_LABELS
        .iter()
        .find(|(candidate, _, _)| *candidate == id)
        .map(|(_, add, empty)| (*add, *empty))
}

pub fn column_label(key: &str) -> Option<Text> {
    COLUMN_LABELS
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, text)| *text)
}

pub fn intro(category: &str) -> Option<Intro> {
    INTROS
        .iter()
        .find(|(candidate, _)| *candidate == category)
        .map(|(_, intro)| *intro)
}

pub fn attached(id: &str) -> impl Iterator<Item = Extra> + '_ {
    AFTER
        .iter()
        .filter(move |attached| attached.id == id)
        .map(|attached| attached.extra)
}

pub fn render_extra(extra: Extra, store: &ConfigStore, window: &Window, cx: &App) -> Option<AnyElement> {
    let strings = cx.t();
    match extra {
        Extra::Comparison {
            key,
            before,
            after,
            before_label,
            after_label,
        } => Some(
            ImageComparison::new(
                SharedString::new_static(key),
                asset_image(format!("images/config/{before}")),
                asset_image(format!("images/config/{after}")),
                before_label(strings),
                after_label(strings),
            )
            .into_any_element(),
        ),
        Extra::Preview { key, image, label } => Some(
            PreviewImage::new(SharedString::new_static(key), asset_image(image), label(strings))
                .into_any_element(),
        ),
        Extra::Note { when, text } => when
            .holds(store)
            .then(|| note(text(strings), IconName::InfoOutline, window, cx).into_any_element()),
        Extra::Warning { key, when, text } => when.holds(store).then(|| {
            Callout::new()
                .id(SharedString::new_static(key))
                .severity(Severity::Warning)
                .icon(IconName::WarningAmberRounded)
                .description(text(strings))
                .into_any_element()
        }),
        Extra::Apply {
            key,
            id,
            value,
            label,
            tooltip,
        } => {
            store.setting(id)?;
            let sizes = window.sizes(cx);
            let active = store.number(id).is_some_and(|current| is_option(current, value as f64));
            let config = cx.stores().config.clone();
            Some(
                h_flex()
                    .pt(sizes.spacing_sm)
                    .px(sizes.padding_sm)
                    .child(
                        Button::new(key, label(strings))
                            .style(ButtonStyle::Tinted(if active { TintColor::Muted } else { TintColor::Accent }))
                            .tooltip(Tooltip::text(tooltip(strings)))
                            .on_click(move |_, _, cx| {
                                config.update(cx, |store, cx| store.set(id, Json::from(value), cx));
                            }),
                    )
                    .into_any_element(),
            )
        }
    }
}

pub fn after(setting: &Setting, store: &ConfigStore, window: &Window, cx: &App) -> Vec<AnyElement> {
    attached(&setting.id)
        .filter_map(|extra| render_extra(extra, store, window, cx))
        .collect()
}

pub fn render_intro(intro: Intro, window: &Window, cx: &App) -> AnyElement {
    let strings = cx.t();
    let sizes = window.sizes(cx);
    match intro {
        Intro::Muted(text) => muted_text(text(strings), false, window, cx).into_any_element(),
        Intro::HighGridsWip => div()
            .w_full()
            .pb(sizes.padding_sm)
            .child(
                Callout::new()
                    .id("lodmod-high-grids-wip")
                    .severity(Severity::Warning)
                    .icon(IconName::ConstructionRounded)
                    .title(strings.high_grids_wip_title())
                    .description(strings.high_grids_wip_body())
                    .child(
                        Label::new(strings.high_grids_wip_contribute())
                            .size(LabelSize::XSmall)
                            .color(Color::Secondary)
                            .line_height(1.35),
                    ),
            )
            .into_any_element(),
        Intro::Accent { key, text } => div()
            .w_full()
            .pb(sizes.padding_sm)
            .child(
                Callout::new()
                    .id(SharedString::new_static(key))
                    .severity(Severity::Accent)
                    .icon(IconName::KeyboardOutlined)
                    .description(text(strings)),
            )
            .into_any_element(),
    }
}
