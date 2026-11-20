use yp_i18n::MsgKey;

use super::{AnyField, ConfigField, FieldMeta};

pub static CUSTOM_ITEMS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_items", MsgKey::LabelContentCustomItems)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomItems)
        .restart_required(),
    || true,
);

pub static CUSTOM_WEAPONS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_weapons", MsgKey::LabelContentCustomWeapons)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomWeapons)
        .restart_required(),
    || true,
);

pub static WEAPON_PATCHES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("weapon_patches", MsgKey::LabelContentWeaponPatches)
        .section("content")
        .tooltip(MsgKey::TooltipContentWeaponPatches)
        .restart_required(),
    || true,
);

pub static CUSTOM_OUTFITS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_outfits", MsgKey::LabelContentCustomOutfits)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomOutfits)
        .restart_required(),
    || true,
);

pub static CUSTOM_HAIRSPRAYS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_hairsprays", MsgKey::LabelContentCustomHairsprays)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomHairsprays)
        .restart_required(),
    || true,
);

pub static ACCESSORIES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("accessories", MsgKey::LabelContentAccessories)
        .section("content")
        .tooltip(MsgKey::TooltipContentAccessories)
        .restart_required(),
    || true,
);

pub static PLAYER_MESH_SWAPS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("player_mesh_swaps", MsgKey::LabelContentPlayerMeshSwaps)
        .section("content")
        .tooltip(MsgKey::TooltipContentPlayerMeshSwaps)
        .restart_required(),
    || true,
);

pub static MESH_VISIBILITY: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("mesh_visibility", MsgKey::LabelContentMeshVisibility)
        .section("content")
        .tooltip(MsgKey::TooltipContentMeshVisibility)
        .restart_required(),
    || true,
);

pub static SHOP_ITEMS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("shop_items", MsgKey::LabelContentShopItems)
        .section("content")
        .tooltip(MsgKey::TooltipContentShopItems)
        .restart_required(),
    || true,
);

pub static ITEM_SHORTCUT_MENU: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("item_shortcut_menu", MsgKey::LabelContentItemShortcutMenu)
        .section("content")
        .tooltip(MsgKey::TooltipContentItemShortcutMenu)
        .restart_required(),
    || true,
);

pub static EFFECT_RULES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("effect_rules", MsgKey::LabelContentEffectRules)
        .section("content")
        .tooltip(MsgKey::TooltipContentEffectRules)
        .restart_required(),
    || true,
);

pub static OUTFIT_SELECTOR_NPC: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("outfit_selector_npc", MsgKey::LabelContentOutfitSelectorNpc)
        .section("content")
        .tooltip(MsgKey::TooltipContentOutfitSelectorNpc)
        .restart_required(),
    || true,
);

pub static BUDDY_OUTFIT_MENU: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("buddy_outfit_menu", MsgKey::LabelContentBuddyOutfitMenu)
        .section("content")
        .tooltip(MsgKey::TooltipContentBuddyOutfitMenu)
        .restart_required(),
    || true,
);

pub static CUSTOM_QUESTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_quests", MsgKey::LabelContentCustomQuests)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomQuests)
        .restart_required(),
    || true,
);

pub static CUSTOM_MAILS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_mails", MsgKey::LabelContentCustomMails)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomMails)
        .restart_required(),
    || true,
);

pub static CUSTOM_VOICE_LINES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_voice_lines", MsgKey::LabelContentCustomVoiceLines)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomVoiceLines)
        .restart_required(),
    || true,
);

pub static CUSTOM_MAPS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_maps", MsgKey::LabelContentCustomMaps)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomMaps)
        .restart_required(),
    || true,
);

pub static MAP_LAYOUTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("map_layouts", MsgKey::LabelContentMapLayouts)
        .section("content")
        .tooltip(MsgKey::TooltipContentMapLayouts)
        .restart_required(),
    || true,
);

pub static NOVELS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("novels", MsgKey::LabelContentNovels)
        .section("content")
        .tooltip(MsgKey::TooltipContentNovels)
        .restart_required(),
    || true,
);

pub static ARCHIVES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("archives", MsgKey::LabelContentArchives)
        .section("content")
        .tooltip(MsgKey::TooltipContentArchives)
        .restart_required(),
    || true,
);

pub static CUTSCENE_REPLACEMENTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "cutscene_replacements",
        MsgKey::LabelContentCutsceneReplacements,
    )
    .section("content")
    .tooltip(MsgKey::TooltipContentCutsceneReplacements)
    .restart_required(),
    || true,
);

pub static CAMERA_SETTINGS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("camera_settings", MsgKey::LabelContentCameraSettings)
        .section("content")
        .tooltip(MsgKey::TooltipContentCameraSettings)
        .restart_required(),
    || true,
);

pub static SCRIPT_OVERLAYS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("script_overlays", MsgKey::LabelContentScriptOverlays)
        .section("content")
        .tooltip(MsgKey::TooltipContentScriptOverlays)
        .restart_required(),
    || true,
);

pub static SCRIPT_GROUPS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("script_groups", MsgKey::LabelContentScriptGroups)
        .section("content")
        .tooltip(MsgKey::TooltipContentScriptGroups)
        .restart_required(),
    || true,
);

pub static RUBY_PATCHES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("ruby_patches", MsgKey::LabelContentRubyPatches)
        .section("content")
        .tooltip(MsgKey::TooltipContentRubyPatches)
        .restart_required(),
    || true,
);

pub static CUSTOM_MUSIC: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_music", MsgKey::LabelContentCustomMusic)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomMusic)
        .restart_required(),
    || true,
);

pub static AMBIENT_SOUNDS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("ambient_sounds", MsgKey::LabelContentAmbientSounds)
        .section("content")
        .tooltip(MsgKey::TooltipContentAmbientSounds)
        .restart_required(),
    || true,
);

pub static CUSTOM_SOUNDS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_sounds", MsgKey::LabelContentCustomSounds)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomSounds)
        .restart_required(),
    || true,
);

pub static EFFECT_AREAS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("effect_areas", MsgKey::LabelContentEffectAreas)
        .section("content")
        .tooltip(MsgKey::TooltipContentEffectAreas)
        .restart_required(),
    || true,
);

pub static EFFECT_TYPE_MUTE: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("effect_type_mute", MsgKey::LabelContentEffectTypeMute)
        .section("content")
        .tooltip(MsgKey::TooltipContentEffectTypeMute)
        .restart_required(),
    || true,
);

pub static GLOBAL_EFFECTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("global_effects", MsgKey::LabelContentGlobalEffects)
        .section("content")
        .tooltip(MsgKey::TooltipContentGlobalEffects)
        .restart_required(),
    || true,
);

pub static LIGHTING_FILTERS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("lighting_filters", MsgKey::LabelContentLightingFilters)
        .section("content")
        .tooltip(MsgKey::TooltipContentLightingFilters)
        .restart_required(),
    || true,
);

pub static CLOTH_PHYSICS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("cloth_physics", MsgKey::LabelContentClothPhysics)
        .section("content")
        .tooltip(MsgKey::TooltipContentClothPhysics)
        .restart_required(),
    || true,
);

pub static CUSTOM_TEXTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("custom_texts", MsgKey::LabelContentCustomTexts)
        .section("content")
        .tooltip(MsgKey::TooltipContentCustomTexts)
        .restart_required(),
    || true,
);

pub static UI_OVERRIDES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("ui_overrides", MsgKey::LabelContentUiOverrides)
        .section("content")
        .tooltip(MsgKey::TooltipContentUiOverrides)
        .restart_required(),
    || true,
);

pub const SECTION: &str = "content";

#[derive(Debug, Clone, Copy)]
pub struct ContentGroup {
    pub title: MsgKey,
    pub fields: &'static [&'static ConfigField<bool>],
}

pub static GROUPS: [ContentGroup; 5] = [
    ContentGroup {
        title: MsgKey::ContentGroupItems,
        fields: &[
            &CUSTOM_ITEMS,
            &CUSTOM_WEAPONS,
            &WEAPON_PATCHES,
            &CUSTOM_OUTFITS,
            &CUSTOM_HAIRSPRAYS,
            &ACCESSORIES,
            &PLAYER_MESH_SWAPS,
            &MESH_VISIBILITY,
            &SHOP_ITEMS,
            &ITEM_SHORTCUT_MENU,
            &EFFECT_RULES,
            &OUTFIT_SELECTOR_NPC,
            &BUDDY_OUTFIT_MENU,
        ],
    },
    ContentGroup {
        title: MsgKey::ContentGroupStory,
        fields: &[
            &CUSTOM_QUESTS,
            &CUSTOM_MAILS,
            &CUSTOM_VOICE_LINES,
            &CUSTOM_MAPS,
            &MAP_LAYOUTS,
            &NOVELS,
            &ARCHIVES,
            &CUTSCENE_REPLACEMENTS,
            &CAMERA_SETTINGS,
        ],
    },
    ContentGroup {
        title: MsgKey::ContentGroupScripts,
        fields: &[&SCRIPT_OVERLAYS, &SCRIPT_GROUPS, &RUBY_PATCHES],
    },
    ContentGroup {
        title: MsgKey::ContentGroupAudio,
        fields: &[&CUSTOM_MUSIC, &AMBIENT_SOUNDS, &CUSTOM_SOUNDS],
    },
    ContentGroup {
        title: MsgKey::ContentGroupVisuals,
        fields: &[
            &EFFECT_AREAS,
            &EFFECT_TYPE_MUTE,
            &GLOBAL_EFFECTS,
            &LIGHTING_FILTERS,
            &CLOTH_PHYSICS,
            &CUSTOM_TEXTS,
            &UI_OVERRIDES,
        ],
    },
];

pub fn fields() -> impl Iterator<Item = &'static ConfigField<bool>> {
    GROUPS.iter().flat_map(|group| group.fields.iter().copied())
}

pub fn all() -> impl Iterator<Item = &'static dyn AnyField> {
    fields().map(|field| field as &'static dyn AnyField)
}
