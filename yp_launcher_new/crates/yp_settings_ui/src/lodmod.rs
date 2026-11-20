use std::rc::Rc;

use gpui::{ScrollHandle, Subscription};
use yp_config::ConfigFile;
use yp_config::fields::lodmod;
use yp_i18n::Strings;
use yp_state::ConfigStore;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    Button, ButtonStyle, Callout, Card, CardPadding, ConfigErrorBanner, ConfirmOptions, ConfirmTone,
    IconName, ImageComparison, Label, LabelSize, PageHeader, SettingSlider, TintColor, Tooltip,
    TwoColumn, asset_image, confirm,
};

use crate::errors::{config_failure_text, toml_error_text};
use crate::fields::{
    FieldRef, FieldRow, Marker, ValueSink, bool_of, field_id, is_changed, label_of, number_of,
    snap_to_step, text_of, tooltip_of, value_in, values_equal,
};
use crate::page::{card, column, failure_banner, muted_text, reveal, subheading, tab_page};
use crate::rules::{NameListEditor, RuleColumn, RulesEditor};

const SCOPE: &str = "lodmod";
const CASCADE_COUNT: usize = 4;
const CASCADE_FALLBACK: f64 = 1.0;

pub const PERFORMANCE_FIELDS: [FieldRef; 3] = [
    &lodmod::CONSTANT_BUFFER_DEDUP,
    &lodmod::CONSTANT_BUFFER_UPLOAD_ON_BIND,
    &lodmod::DECREASE_STUTTER_DURING_GRID_LOADING,
];

pub const AMBIENT_OCCLUSION_FIELDS: [FieldRef; 2] =
    [&lodmod::AO_FULL_RESOLUTION, &lodmod::AO_FADE_FIX];

pub const POST_PROCESSING_FIELDS: [FieldRef; 5] = [
    &lodmod::DISABLE_VIGNETTE,
    &lodmod::FXAA,
    &lodmod::MSAA_PREPASS_FIX,
    &lodmod::MSAA_SHADOW_MASK_FIX,
    &lodmod::DISABLE_HDR,
];

pub const BLOOM_FIELDS: [FieldRef; 4] = [
    &lodmod::BLOOM_REFERENCE_HEIGHT,
    &lodmod::BLOOM_KERNEL_REFERENCE_HEIGHT,
    &lodmod::BLOOM_EXTRA_BLUR,
    &lodmod::BLOOM_DROP_COARSE_LEVELS,
];

pub const GLOBAL_ILLUMINATION_FIELDS: [FieldRef; 2] =
    [&lodmod::GI_ENABLED, &lodmod::GI_WORKGROUP_SIZE];

pub const SHADOW_FIELDS: [FieldRef; 12] = [
    &lodmod::SHADOW_RESOLUTION,
    &lodmod::SHADOW_DISTANCE_MULTIPLIER,
    &lodmod::SHADOW_DISTANCE_MINIMUM,
    &lodmod::SHADOW_DISTANCE_MAXIMUM,
    &lodmod::SHADOW_DISTANCE_PSS,
    &lodmod::SHADOW_FILTER_STRENGTH_BIAS,
    &lodmod::SHADOW_FILTER_STRENGTH_MINIMUM,
    &lodmod::SHADOW_FILTER_STRENGTH_MAXIMUM,
    &lodmod::SHADOW_MODEL_HQ,
    &lodmod::SHADOW_MODEL_FORCE_ALL,
    &lodmod::SHADOW_FORCE_ALL_FILTERS,
    &lodmod::SHADOW_MODEL_FORCE_OBJECT_PARAM,
];

pub fn level_of_detail_fields(values: &toml::Table) -> Vec<FieldRef> {
    let mut fields: Vec<FieldRef> = vec![
        &lodmod::LOD_MULTIPLIER,
        &lodmod::DISABLE_MANUAL_CULLING,
        &lodmod::DISABLE_CAMERA_CULLING,
    ];
    if !lodmod::LAYOUT_SPAWN_EVERYTHING.value_in(values) {
        fields.push(&lodmod::LAYOUT_SPAWN_RANGE);
    }
    fields.push(&lodmod::LAYOUT_SPAWN_EVERYTHING);
    fields
}

pub fn experimental_fields(values: &toml::Table) -> Vec<FieldRef> {
    let mut fields: Vec<FieldRef> = vec![&lodmod::MSAA_PER_PIXEL];
    if lodmod::MSAA_PER_PIXEL.value_in(values) {
        fields.push(&lodmod::MSAA_PER_PIXEL_TERRAIN);
    }
    fields.push(&lodmod::HIGH_GRIDS_ENABLED);
    if lodmod::HIGH_GRIDS_ENABLED.value_in(values) {
        fields.extend([
            &lodmod::HIGH_GRIDS_RINGS as FieldRef,
            &lodmod::HIGH_GRIDS_ROOM_RINGS,
            &lodmod::HIGH_GRIDS_BLOCKED_IN_ROOM,
            &lodmod::HIGH_GRIDS_BLOCKED_FROM_GRID,
            &lodmod::HIGH_GRIDS_HIDDEN_MESHES,
        ]);
    }
    fields.extend([
        &lodmod::FPS_UNCAP_IN_MENUS as FieldRef,
        &lodmod::FPS_UNCAP_IN_GAMEPLAY,
        &lodmod::FPS_CAP_IN_HACKING,
        &lodmod::FPS_CAP_IN_EVENTS,
        &lodmod::FPS_LIMIT,
        &lodmod::RENDER_SCALE,
        &lodmod::SHADOW_CASCADES,
    ]);
    if lodmod::SHADOW_CASCADES.value_in(values) == 8 {
        fields.push(&lodmod::SHADOW_CASCADE_RANGE);
    }
    fields
}

pub fn bloom_reference_snap(next: f64, current: f64) -> f64 {
    let rounded = next.round();
    let floor = lodmod::BLOOM_REFERENCE_MINIMUM as f64;
    if rounded == 0. {
        0.
    } else if rounded < floor {
        if current == 0. { floor } else { 0. }
    } else {
        rounded
    }
}

pub fn cascade_values(values: &toml::Table) -> [f64; CASCADE_COUNT] {
    let raw = lodmod::SHADOW_BLUR_SCALE.value_in(values);
    let mut cascades = [CASCADE_FALLBACK; CASCADE_COUNT];
    for (index, slot) in cascades.iter_mut().enumerate() {
        *slot = match raw.get(index) {
            Some(toml::Value::String(text)) => text.trim().parse().unwrap_or(CASCADE_FALLBACK),
            Some(value) => number_of(value).unwrap_or(CASCADE_FALLBACK),
            None => CASCADE_FALLBACK,
        };
    }
    cascades
}

pub fn cascade_defaults() -> [f64; CASCADE_COUNT] {
    cascade_values(&toml::Table::new())
}

pub fn with_cascade(cascades: [f64; CASCADE_COUNT], index: usize, value: f64) -> toml::Value {
    let mut next = cascades;
    if let Some(slot) = next.get_mut(index) {
        *slot = value;
    }
    toml::Value::Array(next.into_iter().map(toml::Value::Float).collect())
}

pub fn changed_labels(
    fields: &[FieldRef],
    values: &toml::Table,
    strings: &'static Strings,
) -> Vec<SharedString> {
    fields
        .iter()
        .filter(|field| is_changed(**field, &value_in(**field, values)))
        .map(|field| label_of(*field, strings))
        .collect()
}

fn cascade_label(index: usize, strings: &'static Strings) -> SharedString {
    let number = i64::try_from(index + 1).unwrap_or(i64::MAX);
    SharedString::from(strings.lodmod_ui_cascade_label(
        &label_of(&lodmod::SHADOW_BLUR_SCALE, strings),
        &strings.shadow_blur_cascade(number),
    ))
}

pub fn shadow_changed_labels(values: &toml::Table, strings: &'static Strings) -> Vec<SharedString> {
    let mut labels = changed_labels(&SHADOW_FIELDS, values, strings);
    let current = cascade_values(values);
    let defaults = cascade_defaults();
    for index in 0..CASCADE_COUNT {
        if !values_equal(
            &toml::Value::Float(current[index]),
            &toml::Value::Float(defaults[index]),
        ) {
            labels.push(cascade_label(index, strings));
        }
    }
    labels
}

pub struct LodModTab {
    config: Entity<ConfigStore>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl LodModTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx.stores().config.clone();
        let subscriptions = vec![cx.observe(&config, |_, _, cx| cx.notify())];
        Self {
            config,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    fn sink(&self, field: FieldRef) -> ValueSink {
        let config = self.config.clone();
        let meta = *field.meta();
        Rc::new(move |value, _, cx| {
            config.update(cx, |store, cx| {
                store.update_lodmod_live(meta.key, value, meta.section, cx)
            });
        })
    }

    fn row(&self, field: FieldRef, values: &toml::Table) -> FieldRow {
        FieldRow::toml(SCOPE, field, values)
            .marker(Marker::Always)
            .sink(self.sink(field))
    }

    fn slider(&self, field: FieldRef, values: &toml::Table, decimals: usize) -> FieldRow {
        self.row(field, values).decimals(decimals)
    }

    fn bloom_reference(
        &self,
        field: &'static yp_config::ConfigField<i64>,
        values: &toml::Table,
        strings: &'static Strings,
    ) -> FieldRow {
        let current = field.value_in(values);
        let label = if current == 0 {
            SharedString::new_static(strings.bloom_reference_vanilla())
        } else {
            SharedString::from(current.to_string())
        };
        self.row(field, values)
            .decimals(0)
            .value_label(label)
            .snap(bloom_reference_snap)
    }

    fn rules(
        &self,
        field: FieldRef,
        values: &toml::Table,
        columns: Vec<RuleColumn>,
        strings: &'static Strings,
    ) -> RulesEditor {
        let rows = value_in(field, values)
            .as_array()
            .cloned()
            .unwrap_or_default();
        RulesEditor::new(field_id(SCOPE, field), label_of(field, strings), rows)
            .columns(columns)
            .description(tooltip_of(field, strings))
            .apply_marker(Marker::Always.kind(field.meta().restart_required))
            .sink(self.sink(field))
    }

    pub fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        let answer = confirm(
            ConfirmOptions::new(
                strings.lod_mod_reset_confirm_title(),
                strings.lod_mod_reset_confirm_body(),
            )
            .confirm_label(strings.lod_mod_reset_confirm_action())
            .cancel_label(strings.cancel_button())
            .tone(ConfirmTone::Danger),
            window,
            cx,
        );
        let config = self.config.clone();
        cx.spawn(async move |_, cx| {
            if answer.await {
                config
                    .update(cx, |store, cx| store.reset_lodmod_defaults(cx));
            }
        })
        .detach();
    }

    fn comparison(
        id: &'static str,
        before: &'static str,
        after: &'static str,
        before_label: &'static str,
        after_label: &'static str,
    ) -> ImageComparison {
        ImageComparison::new(
            id,
            asset_image(format!("images/config/{before}")),
            asset_image(format!("images/config/{after}")),
            before_label,
            after_label,
        )
    }

    fn intro(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        Card::new()
            .padding(CardPadding::Compact)
            .child(
                Label::new(strings.lod_mod_description())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .line_height(1.4),
            )
            .child(
                v_flex().w_full().pt(sizes.spacing_xs).pb(sizes.spacing_md).child(
                    Label::new(strings.comparison_resolution_note())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .italic(),
                ),
            )
            .child(
                h_flex().child(
                    Button::new("lodmod-reset", strings.lod_mod_reset_button())
                        .style(ButtonStyle::Tinted(TintColor::Warning))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.confirm_reset(window, cx)
                        })),
                ),
            )
            .into_any_element()
    }

    fn performance_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        card("lodmod-performance", strings.card_performance(), cx)
            .changed(changed_labels(&PERFORMANCE_FIELDS, values, strings))
            .child(muted_text(strings.performance_card_note(), false, window, cx))
            .children(
                PERFORMANCE_FIELDS
                    .iter()
                    .map(|field| self.row(*field, values)),
            )
            .into_any_element()
    }

    fn level_of_detail_card(&self, values: &toml::Table, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let spawn_everything = lodmod::LAYOUT_SPAWN_EVERYTHING.value_in(values);
        card("lodmod-lod", strings.card_level_of_detail(), cx)
            .changed(changed_labels(
                &level_of_detail_fields(values),
                values,
                strings,
            ))
            .child(
                self.row(&lodmod::LOD_MULTIPLIER, values)
                    .option_label(move |option| {
                        let label = if option == 0. {
                            strings.lod_multiplier_option_quality()
                        } else if option == 0.75 {
                            strings.lod_multiplier_option_performance()
                        } else if option == 1. {
                            strings.lod_multiplier_option_vanilla()
                        } else if option == 10. {
                            strings.lod_multiplier_option_far()
                        } else {
                            return None;
                        };
                        Some(SharedString::new_static(label))
                    }),
            )
            .child(self.row(&lodmod::DISABLE_MANUAL_CULLING, values))
            .child(self.row(&lodmod::DISABLE_CAMERA_CULLING, values))
            .when(!spawn_everything, |this| {
                this.child(self.slider(&lodmod::LAYOUT_SPAWN_RANGE, values, 0))
            })
            .child(self.row(&lodmod::LAYOUT_SPAWN_EVERYTHING, values))
            .into_any_element()
    }

    fn ambient_occlusion_card(&self, values: &toml::Table, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        card("lodmod-ao", strings.card_ambient_occlusion(), cx)
            .changed(changed_labels(&AMBIENT_OCCLUSION_FIELDS, values, strings))
            .children(
                AMBIENT_OCCLUSION_FIELDS
                    .iter()
                    .map(|field| self.row(*field, values)),
            )
            .into_any_element()
    }

    fn post_processing_card(&self, values: &toml::Table, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        card("lodmod-post", strings.card_post_processing(), cx)
            .changed(changed_labels(&POST_PROCESSING_FIELDS, values, strings))
            .child(self.row(&lodmod::DISABLE_VIGNETTE, values))
            .child(Self::comparison(
                "lodmod-cmp-vignette",
                "vignett_on.webp",
                "vignett_off.webp",
                strings.comparison_vignette_on(),
                strings.comparison_vignette_off(),
            ))
            .child(self.row(&lodmod::FXAA, values))
            .child(Self::comparison(
                "lodmod-cmp-fxaa",
                "fxaa_off.webp",
                "fxaa_on.webp",
                strings.comparison_off(),
                strings.comparison_on(),
            ))
            .child(self.row(&lodmod::MSAA_PREPASS_FIX, values))
            .child(self.row(&lodmod::MSAA_SHADOW_MASK_FIX, values))
            .child(Self::comparison(
                "lodmod-cmp-msaa-mask",
                "msaa8_vanilla.webp",
                "msaa8_shadow_mask_fix.webp",
                strings.comparison_off(),
                strings.comparison_on(),
            ))
            .child(self.row(&lodmod::DISABLE_HDR, values))
            .into_any_element()
    }

    fn bloom_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let preset_active = lodmod::BLOOM_REFERENCE_HEIGHT.value_in(values)
            == lodmod::BLOOM_2017_REFERENCE_HEIGHT;
        let config = self.config.clone();
        card("lodmod-bloom", strings.card_bloom(), cx)
            .changed(changed_labels(&BLOOM_FIELDS, values, strings))
            .child(self.bloom_reference(&lodmod::BLOOM_REFERENCE_HEIGHT, values, strings))
            .child(
                h_flex().pt(sizes.spacing_sm).px(sizes.padding_sm).child(
                    Button::new("lodmod-bloom-2017", strings.bloom2017_button())
                        .style(ButtonStyle::Tinted(if preset_active {
                            TintColor::Muted
                        } else {
                            TintColor::Accent
                        }))
                        .tooltip(Tooltip::text(strings.bloom2017_tooltip()))
                        .on_click(move |_, _, cx| {
                            config.update(cx, |store, cx| {
                                store.apply_lodmod_bloom_2017_preset(cx)
                            });
                        }),
                ),
            )
            .child(Self::comparison(
                "lodmod-cmp-bloom",
                "bloom_2021.webp",
                "bloom_2017.webp",
                strings.comparison_bloom2021(),
                strings.comparison_bloom2017(),
            ))
            .child(self.bloom_reference(
                &lodmod::BLOOM_KERNEL_REFERENCE_HEIGHT,
                values,
                strings,
            ))
            .child(self.slider(&lodmod::BLOOM_EXTRA_BLUR, values, 1))
            .child(
                self.row(&lodmod::BLOOM_DROP_COARSE_LEVELS, values)
                    .option_label(move |option| {
                        let label = if option == 0. {
                            strings.bloom_drop_levels_off()
                        } else if option == 1. {
                            strings.bloom_drop_levels_widest()
                        } else if option == 2. {
                            strings.bloom_drop_levels_two_widest()
                        } else {
                            return None;
                        };
                        Some(SharedString::new_static(label))
                    }),
            )
            .into_any_element()
    }

    fn global_illumination_card(&self, values: &toml::Table, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        card("lodmod-gi", strings.card_global_illumination(), cx)
            .changed(changed_labels(&GLOBAL_ILLUMINATION_FIELDS, values, strings))
            .child(self.row(&lodmod::GI_ENABLED, values))
            .child(self.row(&lodmod::GI_WORKGROUP_SIZE, values))
            .child(Self::comparison(
                "lodmod-cmp-gi",
                "16_cube.webp",
                "128_cube.webp",
                strings.comparison_cubemaps16(),
                strings.comparison_cubemaps128(),
            ))
            .into_any_element()
    }

    fn cascade_sliders(&self, values: &toml::Table, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let strings = cx.t();
        let field = &lodmod::SHADOW_BLUR_SCALE;
        let meta = field.meta;
        let (min, max, step) = (
            meta.min.unwrap_or(0.),
            meta.max.unwrap_or(2.),
            meta.step.unwrap_or(0.05),
        );
        let cascades = cascade_values(values);
        let apply = Marker::Always.kind(meta.restart_required);
        (0..CASCADE_COUNT)
            .map(|index| {
                let sink = self.sink(field);
                let description = (index == 0).then(|| tooltip_of(field, strings)).flatten();
                let id = ElementId::from((field_id(SCOPE, field), index.to_string()));
                SettingSlider::new(id, cascade_label(index, strings), cascades[index] as f32)
                    .min(min as f32)
                    .max(max as f32)
                    .step(step as f32)
                    .decimals(2)
                    .apply_marker(apply)
                    .when_some(description, |this, description| {
                        this.description(description)
                    })
                    .on_change(move |value, window, cx| {
                        let snapped = snap_to_step(f64::from(*value), min, max, step);
                        sink(with_cascade(cascades, index, snapped), window, cx)
                    })
                    .into_any_element()
            })
            .collect()
    }

    fn shadows_card(&self, values: &toml::Table, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let filters = &lodmod::SHADOW_FORCE_ALL_FILTERS;
        let names: Vec<String> = value_in(filters, values)
            .as_array()
            .map(|items| items.iter().map(text_of).collect())
            .unwrap_or_default();
        card("lodmod-shadows", strings.card_shadows(), cx)
            .changed(shadow_changed_labels(values, strings))
            .child(self.row(&lodmod::SHADOW_RESOLUTION, values))
            .child(Self::comparison(
                "lodmod-cmp-shadow-res",
                "shadow_resolution_2148.webp",
                "shadow_resolution_8192.webp",
                strings.comparison2048(),
                strings.comparison8192(),
            ))
            .child(self.slider(&lodmod::SHADOW_DISTANCE_MULTIPLIER, values, 1))
            .child(Self::comparison(
                "lodmod-cmp-shadow-distance",
                "shadow_distance_default.webp",
                "shadow_distance_multiplier_2.0.webp",
                strings.comparison_default(),
                strings.comparison20x(),
            ))
            .child(self.slider(&lodmod::SHADOW_DISTANCE_MINIMUM, values, 0))
            .child(self.slider(&lodmod::SHADOW_DISTANCE_MAXIMUM, values, 0))
            .child(self.slider(&lodmod::SHADOW_DISTANCE_PSS, values, 0))
            .child(Self::comparison(
                "lodmod-cmp-shadow-pss",
                "shadow_dist_filter_off.webp",
                "pss_distance_shadow_-5.0.webp",
                strings.comparison_default(),
                strings.comparison_pss_minus5(),
            ))
            .child(self.slider(&lodmod::SHADOW_FILTER_STRENGTH_BIAS, values, 1))
            .child(Self::comparison(
                "lodmod-cmp-shadow-bias",
                "shadow_dist_filter_off.webp",
                "pss_distance_shadow_-5.0.webp",
                strings.comparison_default(),
                strings.comparison_bias_minus5(),
            ))
            .child(self.slider(&lodmod::SHADOW_FILTER_STRENGTH_MINIMUM, values, 1))
            .child(Self::comparison(
                "lodmod-cmp-shadow-filter",
                "shadow_filter_min_off.webp",
                "shadow_filter_min_3.0.webp",
                strings.comparison_off(),
                strings.comparison30(),
            ))
            .child(self.slider(&lodmod::SHADOW_FILTER_STRENGTH_MAXIMUM, values, 1))
            .child(self.row(&lodmod::SHADOW_MODEL_HQ, values))
            .child(self.row(&lodmod::SHADOW_MODEL_FORCE_ALL, values))
            .child(Self::comparison(
                "lodmod-cmp-shadow-models",
                "hd_shadow_models_off.webp",
                "hq_shadow_models_on.webp",
                strings.comparison_off(),
                strings.comparison_hq_force_all(),
            ))
            .child(
                NameListEditor::new(
                    field_id(SCOPE, filters),
                    label_of(filters, strings),
                    names,
                )
                .description(tooltip_of(filters, strings))
                .apply_marker(Marker::Always.kind(filters.meta.restart_required))
                .add_label(strings.shadow_force_all_filters_add())
                .empty_label(strings.shadow_force_all_filters_empty())
                .sink(self.sink(filters)),
            )
            .child(self.row(&lodmod::SHADOW_MODEL_FORCE_OBJECT_PARAM, values))
            .children(self.cascade_sliders(values, cx))
            .into_any_element()
    }

    fn high_grids_note(window: &mut Window, cx: &mut App) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        div()
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
            .into_any_element()
    }

    fn experimental_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let per_pixel = lodmod::MSAA_PER_PIXEL.value_in(values);
        let high_grids = lodmod::HIGH_GRIDS_ENABLED.value_in(values);
        let eight_cascades = lodmod::SHADOW_CASCADES.value_in(values) == 8;
        let mut left: Vec<AnyElement> = vec![
            subheading(strings.subheading_msaa_performance(), window, cx).into_any_element(),
            self.row(&lodmod::MSAA_PER_PIXEL, values).into_any_element(),
            Self::comparison(
                "lodmod-cmp-msaa-per-pixel",
                "msaa8_both_fixes.webp",
                "msaa8_per_pixel.webp",
                strings.comparison_off(),
                strings.comparison_on(),
            )
            .into_any_element(),
        ];
        if per_pixel {
            left.push(
                self.row(&lodmod::MSAA_PER_PIXEL_TERRAIN, values)
                    .into_any_element(),
            );
        }
        left.push(subheading(strings.subheading_high_grids(), window, cx).into_any_element());
        left.push(Self::high_grids_note(window, cx));
        left.push(
            self.row(&lodmod::HIGH_GRIDS_ENABLED, values)
                .into_any_element(),
        );
        if high_grids {
            left.push(
                self.row(&lodmod::HIGH_GRIDS_RINGS, values)
                    .option_label(move |option| {
                        let rings = option.round() as i64;
                        let cells = lodmod::high_grids_cell_count(rings).unwrap_or(7);
                        Some(SharedString::from(
                            strings.high_grids_cell_count(rings, cells),
                        ))
                    })
                    .into_any_element(),
            );
            left.push(
                Self::comparison(
                    "lodmod-cmp-high-grids",
                    "Vanilla_High_grids_7.webp",
                    "YP_High_grids_61.webp",
                    strings.comparison_high_grids7(),
                    strings.comparison_high_grids61(),
                )
                .into_any_element(),
            );
            left.push(
                self.rules(
                    &lodmod::HIGH_GRIDS_ROOM_RINGS,
                    values,
                    vec![
                        RuleColumn::room("room", strings.grid_rule_room()),
                        RuleColumn::choice(
                            "rings",
                            strings.grid_rule_rings(),
                            vec![1, 2, 3, 4, 5],
                            1,
                        ),
                    ],
                    strings,
                )
                .into_any_element(),
            );
            left.push(
                self.rules(
                    &lodmod::HIGH_GRIDS_BLOCKED_IN_ROOM,
                    values,
                    vec![
                        RuleColumn::room("room", strings.grid_rule_room()),
                        RuleColumn::hex("grid", strings.grid_rule_grid(), 0x1121),
                    ],
                    strings,
                )
                .into_any_element(),
            );
            left.push(
                self.rules(
                    &lodmod::HIGH_GRIDS_BLOCKED_FROM_GRID,
                    values,
                    vec![
                        RuleColumn::hex("from", strings.grid_rule_from_grid(), 0x1517),
                        RuleColumn::hex("grid", strings.grid_rule_grid(), 0x1121),
                    ],
                    strings,
                )
                .into_any_element(),
            );
            left.push(
                self.rules(
                    &lodmod::HIGH_GRIDS_HIDDEN_MESHES,
                    values,
                    vec![
                        RuleColumn::text("mesh", strings.grid_rule_mesh()),
                        RuleColumn::choice(
                            "from_rings",
                            strings.grid_rule_from_rings(),
                            vec![2, 3, 4, 5],
                            2,
                        ),
                        RuleColumn::hex("grid", strings.grid_rule_grid(), 0),
                    ],
                    strings,
                )
                .into_any_element(),
            );
        }
        let mut right: Vec<AnyElement> = vec![
            subheading(strings.subheading_frame_rate(), window, cx).into_any_element(),
            self.row(&lodmod::FPS_UNCAP_IN_MENUS, values)
                .into_any_element(),
            self.row(&lodmod::FPS_UNCAP_IN_GAMEPLAY, values)
                .into_any_element(),
            self.row(&lodmod::FPS_CAP_IN_HACKING, values)
                .into_any_element(),
            self.row(&lodmod::FPS_CAP_IN_EVENTS, values)
                .into_any_element(),
            self.row(&lodmod::FPS_LIMIT, values).into_any_element(),
            subheading(strings.subheading_rendering(), window, cx).into_any_element(),
            self.slider(&lodmod::RENDER_SCALE, values, 2)
                .into_any_element(),
            self.row(&lodmod::SHADOW_CASCADES, values)
                .into_any_element(),
            Self::comparison(
                "lodmod-cmp-cascades",
                "4_cascades_8k.webp",
                "8_cascades_8k.webp",
                strings.comparison_cascades4(),
                strings.comparison_cascades8(),
            )
            .into_any_element(),
        ];
        if eight_cascades {
            right.push(
                self.slider(&lodmod::SHADOW_CASCADE_RANGE, values, 2)
                    .into_any_element(),
            );
        }
        card("lodmod-experimental", strings.card_experimental(), cx)
            .changed(changed_labels(&experimental_fields(values), values, strings))
            .child(muted_text(
                strings.experimental_warning_body(),
                false,
                window,
                cx,
            ))
            .child(
                TwoColumn::new(v_flex().w_full().children(left), v_flex().w_full().children(right))
                    .id("lodmod-experimental-columns"),
            )
            .into_any_element()
    }
}

impl Render for LodModTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let store = self.config.read(cx);
        let loading = store.is_loading();
        let dirty = store.is_dirty();
        let values = store.lodmod().clone();
        let error = store
            .error(ConfigFile::LodMod)
            .map(|error| toml_error_text(error, strings));
        let failure = store
            .failure()
            .map(|failure| config_failure_text(failure, strings));
        let path = store.path(ConfigFile::LodMod);
        let save_store = self.config.clone();
        let discard_store = self.config.clone();
        let header = PageHeader::new(strings.header_lod_mod())
            .dirty(dirty)
            .save_discard(
                move |_, cx| save_store.update(cx, |store, cx| store.save(cx)),
                move |_, cx| discard_store.update(cx, |store, cx| store.discard(cx)),
            );
        let header = match path {
            Some(path) => header.info(strings.tooltip_edits_lodmod_toml(), move |_, cx| {
                reveal(path.clone(), true, cx)
            }),
            None => header.info_tooltip(strings.tooltip_edits_lodmod_toml()),
        };
        let content = if loading {
            div().into_any_element()
        } else {
            let enabled = bool_of(&value_in(&lodmod::ENABLED, &values));
            let mut sections: Vec<AnyElement> = Vec::new();
            if let Some(error) = error {
                sections.push(
                    ConfigErrorBanner::new(ConfigFile::LodMod.file_name(), error)
                        .into_any_element(),
                );
            }
            if let Some(failure) = failure {
                sections.push(failure_banner("lodmod-failure", failure).into_any_element());
            }
            sections.push(self.intro(window, cx));
            sections.push(
                TwoColumn::new(
                    self.row(&lodmod::ENABLED, &values),
                    Self::comparison(
                        "lodmod-cmp-master",
                        "lod_mod_off.webp",
                        "lod_mod_on.webp",
                        strings.comparison_vanilla(),
                        strings.comparison_default_enabled(),
                    ),
                )
                .id("lodmod-master")
                .into_any_element(),
            );
            if enabled {
                let left = column(
                    [
                        self.performance_card(&values, window, cx),
                        self.level_of_detail_card(&values, cx),
                        self.ambient_occlusion_card(&values, cx),
                        self.post_processing_card(&values, cx),
                    ],
                    window,
                    cx,
                );
                let right = column(
                    [
                        self.bloom_card(&values, window, cx),
                        self.global_illumination_card(&values, cx),
                        self.shadows_card(&values, cx),
                    ],
                    window,
                    cx,
                );
                sections.push(
                    TwoColumn::new(left, right)
                        .id("lodmod-columns")
                        .into_any_element(),
                );
                sections.push(self.experimental_card(&values, window, cx));
            }
            column(sections, window, cx).into_any_element()
        };
        tab_page("lodmod-scroll", header, &self.scroll, loading, content, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use yp_i18n::Locale;

    use super::*;

    fn table(text: &str) -> toml::Table {
        text.parse::<toml::Table>().unwrap()
    }

    #[test]
    fn bloom_reference_skips_the_range_below_the_minimum() {
        assert_eq!(bloom_reference_snap(0., 900.), 0.);
        assert_eq!(bloom_reference_snap(20., 0.), 360.);
        assert_eq!(bloom_reference_snap(340., 0.), 360.);
        assert_eq!(bloom_reference_snap(340., 360.), 0.);
        assert_eq!(bloom_reference_snap(900.4, 360.), 900.);
        assert_eq!(bloom_reference_snap(360., 0.), 360.);
    }

    #[test]
    fn cascade_values_fill_missing_entries_with_one() {
        assert_eq!(cascade_defaults(), [1.0; 4]);
        let values = table("shadow_blur_scale = [0.5, 2, \"1.25\"]\n");
        assert_eq!(cascade_values(&values), [0.5, 2.0, 1.25, 1.0]);
        assert_eq!(
            with_cascade([1.0; 4], 2, 0.25),
            toml::Value::Array(vec![
                toml::Value::Float(1.0),
                toml::Value::Float(1.0),
                toml::Value::Float(0.25),
                toml::Value::Float(1.0),
            ])
        );
    }

    #[test]
    fn conditional_rows_only_count_while_they_are_shown() {
        let strings = Strings::for_locale(Locale::En);
        let hidden = table("layout_spawn_everything = true\nlayout_spawn_range = 400.0\n");
        let fields = level_of_detail_fields(&hidden);
        assert!(!fields
            .iter()
            .any(|field| field.meta().key == lodmod::LAYOUT_SPAWN_RANGE.key()));
        let shown = table("layout_spawn_range = 400.0\nlayout_spawn_everything = false\n");
        let fields = level_of_detail_fields(&shown);
        let labels = changed_labels(&fields, &shown, strings);
        assert!(labels.contains(&label_of(&lodmod::LAYOUT_SPAWN_RANGE, strings)));

        let plain = experimental_fields(&toml::Table::new());
        let with_grids = experimental_fields(&table("[high_grids]\nenabled = true\n"));
        assert_eq!(with_grids.len(), plain.len() + 5);
        let cascades = experimental_fields(&table("shadow_cascades = 8\n"));
        assert_eq!(cascades.len(), plain.len() + 1);
    }

    #[test]
    fn every_lodmod_field_is_reachable_on_the_page() {
        let all_on = table(
            "msaa_per_pixel = true\nshadow_cascades = 8\nlayout_spawn_everything = false\n[high_grids]\nenabled = true\n",
        );
        let mut keys: Vec<&str> = vec![lodmod::ENABLED.key(), lodmod::SHADOW_BLUR_SCALE.key()];
        for group in [
            PERFORMANCE_FIELDS.as_slice(),
            &AMBIENT_OCCLUSION_FIELDS,
            &POST_PROCESSING_FIELDS,
            &BLOOM_FIELDS,
            &GLOBAL_ILLUMINATION_FIELDS,
            &SHADOW_FIELDS,
            &level_of_detail_fields(&all_on),
            &experimental_fields(&all_on),
        ] {
            keys.extend(group.iter().map(|field| field.meta().key));
        }
        for field in lodmod::ALL {
            assert!(
                keys.contains(&field.meta().key),
                "{} is missing from the LodMod page",
                field.meta().key
            );
        }
        assert_eq!(keys.len(), lodmod::ALL.len());
    }

    #[test]
    fn shadow_card_counts_each_changed_cascade() {
        let strings = Strings::for_locale(Locale::En);
        let values = table("shadow_blur_scale = [1.0, 0.5, 1.0, 1.5]\nshadow_resolution = 8192\n");
        let labels = shadow_changed_labels(&values, strings);
        let cascades = labels
            .iter()
            .filter(|label| label.contains(strings.shadow_blur_cascade(2).as_str()) || label.contains(strings.shadow_blur_cascade(4).as_str()))
            .count();
        assert_eq!(cascades, 2);
    }
}
