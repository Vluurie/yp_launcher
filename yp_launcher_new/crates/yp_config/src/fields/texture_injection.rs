use yp_i18n::MsgKey;

use super::{AnyField, ConfigField, FieldMeta};

pub static VRAM_BUDGET_MB: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("vram_budget_mb", MsgKey::LabelVramBudget).tooltip(MsgKey::TooltipVramBudget),
    || 0,
);

pub static STREAMING_ENABLED: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("streaming_enabled", MsgKey::LabelStreamingEnabled)
        .tooltip(MsgKey::TooltipStreamingEnabled),
    || true,
);

pub static LOAD_ONLY_RELEVANT: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("load_only_relevant", MsgKey::LabelLoadOnlyRelevant)
        .tooltip(MsgKey::TooltipLoadOnlyRelevant),
    || false,
);

pub static PIN_PRIORITY_TEXTURES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("pin_priority_textures", MsgKey::LabelPinPriorityTextures)
        .tooltip(MsgKey::TooltipPinPriorityTextures)
        .restart_required(),
    || true,
);

pub static LOAD_ORDER: ConfigField<Vec<String>> = ConfigField::new(
    FieldMeta::new("load_order", MsgKey::CardTextureConfig),
    Vec::new,
);

pub static HOT_RELOAD: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("hot_reload", MsgKey::LabelTextureHotReload)
        .tooltip(MsgKey::TooltipTextureHotReload),
    || true,
);

pub static DISABLED_PACKS: ConfigField<Vec<String>> = ConfigField::new(
    FieldMeta::new("disabled_packs", MsgKey::CardTextureConfig),
    Vec::new,
);

pub static ALL: [&dyn AnyField; 7] = [
    &VRAM_BUDGET_MB,
    &STREAMING_ENABLED,
    &LOAD_ONLY_RELEVANT,
    &PIN_PRIORITY_TEXTURES,
    &LOAD_ORDER,
    &HOT_RELOAD,
    &DISABLED_PACKS,
];
