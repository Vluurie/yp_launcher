use yp_i18n::MsgKey;

use super::{AnyField, ConfigField, FieldMeta};

pub static ENABLED: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("enabled", MsgKey::LabelEnableLodMod).tooltip(MsgKey::TooltipEnableLodMod),
    || false,
);

pub static LOD_MULTIPLIER: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("lod_multiplier", MsgKey::LabelLodMultiplier)
        .tooltip(MsgKey::TooltipLodMultiplier)
        .allowed(&[0.0, 0.75, 1.0, 10.0]),
    || 0.0,
);

pub static LAYOUT_SPAWN_RANGE: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("layout_spawn_range", MsgKey::LabelLayoutSpawnRange)
        .tooltip(MsgKey::TooltipLayoutSpawnRange)
        .range(25.0, 400.0)
        .step(25.0),
    || 100.0,
);

pub static LAYOUT_SPAWN_EVERYTHING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "layout_spawn_everything",
        MsgKey::LabelLayoutSpawnEverything,
    )
    .tooltip(MsgKey::TooltipLayoutSpawnEverything),
    || false,
);

pub static DISABLE_CAMERA_CULLING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_camera_culling", MsgKey::LabelDisableCameraCulling)
        .tooltip(MsgKey::TooltipDisableCameraCulling),
    || false,
);

pub static DISABLE_MANUAL_CULLING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_manual_culling", MsgKey::LabelDisableManualCulling)
        .tooltip(MsgKey::TooltipDisableManualCulling),
    || false,
);

pub static AO_FULL_RESOLUTION: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("ao_full_resolution", MsgKey::LabelAoFullResolution)
        .tooltip(MsgKey::TooltipAoFullResolution),
    || false,
);

pub static DISABLE_VIGNETTE: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_vignette", MsgKey::LabelDisableVignette)
        .tooltip(MsgKey::TooltipDisableVignette),
    || false,
);

pub static BLOOM_REFERENCE_HEIGHT: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("bloom_reference_height", MsgKey::LabelBloomReferenceHeight)
        .tooltip(MsgKey::TooltipBloomReferenceHeight)
        .range(0.0, 4320.0)
        .step(20.0),
    || 0,
);

pub static BLOOM_KERNEL_REFERENCE_HEIGHT: ConfigField<i64> = ConfigField::new(
    FieldMeta::new(
        "bloom_kernel_reference_height",
        MsgKey::LabelBloomKernelReferenceHeight,
    )
    .tooltip(MsgKey::TooltipBloomKernelReferenceHeight)
    .range(0.0, 4320.0)
    .step(20.0),
    || 0,
);

pub static BLOOM_EXTRA_BLUR: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("bloom_extra_blur", MsgKey::LabelBloomExtraBlur)
        .tooltip(MsgKey::TooltipBloomExtraBlur)
        .range(0.0, 4.0)
        .step(0.1),
    || 0.0,
);

pub static BLOOM_DROP_COARSE_LEVELS: ConfigField<i64> = ConfigField::new(
    FieldMeta::new(
        "bloom_drop_coarse_levels",
        MsgKey::LabelBloomDropCoarseLevels,
    )
    .tooltip(MsgKey::TooltipBloomDropCoarseLevels)
    .allowed(&[0.0, 1.0, 2.0]),
    || 0,
);

pub static HIGH_GRIDS_ENABLED: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("enabled", MsgKey::LabelHighGridsEnabled)
        .section("high_grids")
        .tooltip(MsgKey::TooltipHighGridsEnabled)
        .restart_required(),
    || false,
);

pub static HIGH_GRIDS_RINGS: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("rings", MsgKey::LabelHighGridsRings)
        .section("high_grids")
        .tooltip(MsgKey::TooltipHighGridsRings)
        .allowed(&[1.0, 2.0, 3.0, 4.0])
        .restart_required(),
    || 1,
);

pub static HIGH_GRIDS_ROOM_RINGS: ConfigField<Vec<toml::Value>> = ConfigField::new(
    FieldMeta::new("room_rings", MsgKey::LabelHighGridsRoomRings)
        .section("high_grids")
        .tooltip(MsgKey::TooltipHighGridsRoomRings)
        .restart_required(),
    Vec::new,
);

pub static HIGH_GRIDS_BLOCKED_IN_ROOM: ConfigField<Vec<toml::Value>> = ConfigField::new(
    FieldMeta::new("blocked_in_room", MsgKey::LabelHighGridsBlockedInRoom)
        .section("high_grids")
        .tooltip(MsgKey::TooltipHighGridsBlockedInRoom)
        .restart_required(),
    Vec::new,
);

pub static HIGH_GRIDS_BLOCKED_FROM_GRID: ConfigField<Vec<toml::Value>> = ConfigField::new(
    FieldMeta::new("blocked_from_grid", MsgKey::LabelHighGridsBlockedFromGrid)
        .section("high_grids")
        .tooltip(MsgKey::TooltipHighGridsBlockedFromGrid)
        .restart_required(),
    Vec::new,
);

pub static SHADOW_RESOLUTION: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("shadow_resolution", MsgKey::LabelShadowResolution)
        .tooltip(MsgKey::TooltipShadowResolution)
        .allowed(&[512.0, 1024.0, 2048.0, 4096.0, 8192.0]),
    || 2048,
);

pub static SHADOW_DISTANCE_MULTIPLIER: ConfigField<f64> = ConfigField::new(
    FieldMeta::new(
        "shadow_distance_multiplier",
        MsgKey::LabelDistanceMultiplier,
    )
    .tooltip(MsgKey::TooltipDistanceMultiplier)
    .range(1.0, 3.0)
    .step(0.1),
    || 1.0,
);

pub static SHADOW_DISTANCE_MINIMUM: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("shadow_distance_minimum", MsgKey::LabelDistanceMinimum)
        .tooltip(MsgKey::TooltipDistanceMinimum)
        .range(0.0, 500.0)
        .step(1.0),
    || 0.0,
);

pub static SHADOW_DISTANCE_MAXIMUM: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("shadow_distance_maximum", MsgKey::LabelDistanceMaximum)
        .tooltip(MsgKey::TooltipDistanceMaximum)
        .range(0.0, 2000.0)
        .step(5.0),
    || 0.0,
);

pub static SHADOW_DISTANCE_PSS: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("shadow_distance_pss", MsgKey::LabelDistancePss)
        .tooltip(MsgKey::TooltipDistancePss)
        .range(0.0, 2000.0)
        .step(5.0),
    || 0.0,
);

pub static SHADOW_FILTER_STRENGTH_BIAS: ConfigField<f64> = ConfigField::new(
    FieldMeta::new(
        "shadow_filter_strength_bias",
        MsgKey::LabelFilterStrengthBias,
    )
    .tooltip(MsgKey::TooltipFilterStrengthBias)
    .range(-5.0, 5.0)
    .step(0.1),
    || 0.0,
);

pub static SHADOW_FILTER_STRENGTH_MINIMUM: ConfigField<f64> = ConfigField::new(
    FieldMeta::new(
        "shadow_filter_strength_minimum",
        MsgKey::LabelFilterStrengthMin,
    )
    .tooltip(MsgKey::TooltipFilterStrengthMin)
    .range(0.0, 10.0)
    .step(0.1),
    || 0.0,
);

pub static SHADOW_FILTER_STRENGTH_MAXIMUM: ConfigField<f64> = ConfigField::new(
    FieldMeta::new(
        "shadow_filter_strength_maximum",
        MsgKey::LabelFilterStrengthMax,
    )
    .tooltip(MsgKey::TooltipFilterStrengthMax)
    .range(0.0, 10.0)
    .step(0.1),
    || 0.0,
);

pub static SHADOW_MODEL_HQ: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("shadow_model_hq", MsgKey::LabelHqShadowModels)
        .tooltip(MsgKey::TooltipHqShadowModels),
    || false,
);

pub static SHADOW_MODEL_FORCE_ALL: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("shadow_model_force_all", MsgKey::LabelForceAllShadowModels)
        .tooltip(MsgKey::TooltipForceAllShadowModels),
    || false,
);

pub static SHADOW_FORCE_ALL_FILTERS: ConfigField<Vec<toml::Value>> = ConfigField::new(
    FieldMeta::new(
        "shadow_force_all_filters",
        MsgKey::LabelShadowForceAllFilters,
    )
    .tooltip(MsgKey::TooltipShadowForceAllFilters),
    default_shadow_force_all_filters,
);

pub static SHADOW_MODEL_FORCE_OBJECT_PARAM: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("shadow_model_force_object_param", MsgKey::LabelGrassShadows)
        .tooltip(MsgKey::TooltipGrassShadows),
    || false,
);

pub static GI_ENABLED: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("gi_enabled", MsgKey::LabelGiEnabled).tooltip(MsgKey::TooltipGiEnabled),
    || false,
);

pub static GI_WORKGROUP_SIZE: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("gi_workgroup_size", MsgKey::LabelGiWorkgroupSize)
        .tooltip(MsgKey::TooltipGiWorkgroupSize)
        .allowed(&[16.0, 32.0, 64.0, 128.0]),
    || 128,
);

pub static SHADOW_CASCADES: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("shadow_cascades", MsgKey::LabelShadowCascades)
        .tooltip(MsgKey::TooltipShadowCascades)
        .allowed(&[4.0, 8.0])
        .restart_required(),
    || 4,
);

pub static SHADOW_CASCADE_RANGE: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("shadow_cascade_range", MsgKey::LabelShadowCascadeRange)
        .tooltip(MsgKey::TooltipShadowCascadeRange)
        .range(1.05, 8.0)
        .step(0.05)
        .restart_required(),
    || 4.0,
);

pub static SHADOW_BLUR_SCALE: ConfigField<Vec<toml::Value>> = ConfigField::new(
    FieldMeta::new("shadow_blur_scale", MsgKey::LabelShadowBlurScale)
        .tooltip(MsgKey::TooltipShadowBlurScale)
        .range(0.0, 2.0)
        .step(0.05),
    default_shadow_blur_scale,
);

pub static DISABLE_HDR: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_hdr", MsgKey::LabelDisableHdr).tooltip(MsgKey::TooltipDisableHdr),
    || false,
);

pub static RENDER_SCALE: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("render_scale", MsgKey::LabelRenderScale)
        .tooltip(MsgKey::TooltipRenderScale)
        .range(0.5, 4.0)
        .step(0.25),
    || 1.0,
);

pub static FXAA: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("fxaa", MsgKey::LabelFxaa).tooltip(MsgKey::TooltipFxaa),
    || false,
);

pub static MSAA_PREPASS_FIX: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("msaa_prepass_fix", MsgKey::LabelMsaaPrepassFix)
        .tooltip(MsgKey::TooltipMsaaPrepassFix),
    || true,
);

pub static MSAA_SHADOW_MASK_FIX: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("msaa_shadow_mask_fix", MsgKey::LabelMsaaShadowMaskFix)
        .tooltip(MsgKey::TooltipMsaaShadowMaskFix),
    || true,
);

pub static MSAA_PER_PIXEL: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("msaa_per_pixel", MsgKey::LabelMsaaPerPixel)
        .tooltip(MsgKey::TooltipMsaaPerPixel),
    || false,
);

pub static MSAA_PER_PIXEL_TERRAIN: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("msaa_per_pixel_terrain", MsgKey::LabelMsaaPerPixelTerrain)
        .tooltip(MsgKey::TooltipMsaaPerPixelTerrain),
    || false,
);

pub static CONSTANT_BUFFER_DEDUP: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("constant_buffer_dedup", MsgKey::LabelConstantBufferDedup)
        .tooltip(MsgKey::TooltipConstantBufferDedup),
    || false,
);

pub static CONSTANT_BUFFER_UPLOAD_ON_BIND: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "constant_buffer_upload_on_bind",
        MsgKey::LabelConstantBufferUploadOnBind,
    )
    .tooltip(MsgKey::TooltipConstantBufferUploadOnBind),
    || false,
);

pub static DECREASE_STUTTER_DURING_GRID_LOADING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "decrease_stutter_during_grid_loading",
        MsgKey::LabelDecreaseStutterDuringGridLoading,
    )
    .tooltip(MsgKey::TooltipDecreaseStutterDuringGridLoading),
    || false,
);

pub static AO_FADE_FIX: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("ao_fade_fix", MsgKey::LabelAoFadeFix).tooltip(MsgKey::TooltipAoFadeFix),
    || true,
);

pub static HIGH_GRIDS_HIDDEN_MESHES: ConfigField<Vec<toml::Value>> = ConfigField::new(
    FieldMeta::new("hidden_meshes", MsgKey::LabelHighGridsHiddenMeshes)
        .section("high_grids")
        .tooltip(MsgKey::TooltipHighGridsHiddenMeshes)
        .restart_required(),
    default_hidden_meshes,
);

pub static FPS_UNCAP_IN_MENUS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("fps_uncap_in_menus", MsgKey::LabelFpsUncapInMenus)
        .tooltip(MsgKey::TooltipFpsUncapInMenus),
    || false,
);

pub static FPS_UNCAP_IN_GAMEPLAY: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("fps_uncap_in_gameplay", MsgKey::LabelFpsUncapInGameplay)
        .tooltip(MsgKey::TooltipFpsUncapInGameplay),
    || false,
);

pub static FPS_CAP_IN_HACKING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("fps_cap_in_hacking", MsgKey::LabelFpsCapInHacking)
        .tooltip(MsgKey::TooltipFpsCapInHacking),
    || true,
);

pub static FPS_CAP_IN_EVENTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("fps_cap_in_events", MsgKey::LabelFpsCapInEvents)
        .tooltip(MsgKey::TooltipFpsCapInEvents),
    || true,
);

pub static FPS_LIMIT: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("fps_limit", MsgKey::LabelFpsLimit)
        .tooltip(MsgKey::TooltipFpsLimit)
        .range(0.0, 1000.0),
    || 0,
);

pub const HIGH_GRIDS_SECTION: &str = "high_grids";

pub const BLOOM_2017_REFERENCE_HEIGHT: i64 = 900;

pub const BLOOM_REFERENCE_MINIMUM: i64 = 360;

pub fn bloom_2017_preset() -> toml::Table {
    toml::Table::from_iter([(
        BLOOM_REFERENCE_HEIGHT.key().to_owned(),
        toml::Value::Integer(BLOOM_2017_REFERENCE_HEIGHT),
    )])
}

pub const HIGH_GRIDS_CELL_COUNTS: [(i64, i64); 4] = [(1, 7), (2, 19), (3, 37), (4, 61)];

pub const HIGH_GRIDS_ROOMS: [(u32, &str); 10] = [
    (0x100, "Ruined City"),
    (0x110, "Resistance Camp"),
    (0x120, "Factory"),
    (0x130, "Amusement Park"),
    (0x140, "Pascal's Village"),
    (0x150, "Desert"),
    (0x160, "Forest"),
    (0x170, "Flooded City"),
    (0x200, "City Ruins (Route B)"),
    (0x520, "Underground"),
];

pub fn high_grids_cell_count(rings: i64) -> Option<i64> {
    HIGH_GRIDS_CELL_COUNTS
        .iter()
        .find(|(ring_count, _)| *ring_count == rings)
        .map(|(_, cells)| *cells)
}

fn default_shadow_force_all_filters() -> Vec<toml::Value> {
    ["ENKEI", "landscape", "GROUND_g11118jimen"]
        .into_iter()
        .map(|name| toml::Value::String(name.to_owned()))
        .collect()
}

fn default_shadow_blur_scale() -> Vec<toml::Value> {
    vec![toml::Value::Float(1.0); 4]
}

fn hidden_mesh(mesh: &str, from_rings: i64, grid: i64) -> toml::Value {
    toml::Value::Table(toml::Table::from_iter([
        ("mesh".to_owned(), toml::Value::String(mesh.to_owned())),
        ("from_rings".to_owned(), toml::Value::Integer(from_rings)),
        ("grid".to_owned(), toml::Value::Integer(grid)),
    ]))
}

fn default_hidden_meshes() -> Vec<toml::Value> {
    vec![
        hidden_mesh("nolowmap_enkei", 2, 0x1121),
        hidden_mesh("nolowmap_seaplane", 2, 0x0922),
        hidden_mesh("nolowmap_g11517_bigtree_05", 2, 0x1517),
    ]
}

pub static ALL: [&dyn AnyField; 51] = [
    &ENABLED,
    &LOD_MULTIPLIER,
    &LAYOUT_SPAWN_RANGE,
    &LAYOUT_SPAWN_EVERYTHING,
    &DISABLE_CAMERA_CULLING,
    &AO_FULL_RESOLUTION,
    &BLOOM_REFERENCE_HEIGHT,
    &BLOOM_KERNEL_REFERENCE_HEIGHT,
    &BLOOM_EXTRA_BLUR,
    &BLOOM_DROP_COARSE_LEVELS,
    &SHADOW_RESOLUTION,
    &SHADOW_DISTANCE_MINIMUM,
    &SHADOW_DISTANCE_MAXIMUM,
    &SHADOW_DISTANCE_MULTIPLIER,
    &SHADOW_DISTANCE_PSS,
    &SHADOW_FILTER_STRENGTH_BIAS,
    &SHADOW_FILTER_STRENGTH_MINIMUM,
    &SHADOW_FILTER_STRENGTH_MAXIMUM,
    &SHADOW_CASCADES,
    &SHADOW_CASCADE_RANGE,
    &SHADOW_BLUR_SCALE,
    &SHADOW_MODEL_HQ,
    &SHADOW_MODEL_FORCE_ALL,
    &SHADOW_FORCE_ALL_FILTERS,
    &SHADOW_MODEL_FORCE_OBJECT_PARAM,
    &DISABLE_MANUAL_CULLING,
    &GI_ENABLED,
    &GI_WORKGROUP_SIZE,
    &FPS_UNCAP_IN_MENUS,
    &FPS_UNCAP_IN_GAMEPLAY,
    &FPS_CAP_IN_HACKING,
    &FPS_CAP_IN_EVENTS,
    &FPS_LIMIT,
    &DISABLE_VIGNETTE,
    &DISABLE_HDR,
    &RENDER_SCALE,
    &FXAA,
    &MSAA_PREPASS_FIX,
    &MSAA_SHADOW_MASK_FIX,
    &MSAA_PER_PIXEL,
    &MSAA_PER_PIXEL_TERRAIN,
    &AO_FADE_FIX,
    &CONSTANT_BUFFER_DEDUP,
    &CONSTANT_BUFFER_UPLOAD_ON_BIND,
    &DECREASE_STUTTER_DURING_GRID_LOADING,
    &HIGH_GRIDS_ENABLED,
    &HIGH_GRIDS_RINGS,
    &HIGH_GRIDS_ROOM_RINGS,
    &HIGH_GRIDS_BLOCKED_IN_ROOM,
    &HIGH_GRIDS_BLOCKED_FROM_GRID,
    &HIGH_GRIDS_HIDDEN_MESHES,
];
