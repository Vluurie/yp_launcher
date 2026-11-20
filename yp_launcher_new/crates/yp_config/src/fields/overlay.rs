use super::AnyField;

pub mod keybinds {
    use yp_i18n::MsgKey;

    use crate::fields::{ConfigField, FieldMeta};

    pub static OPEN_WORKSPACE: ConfigField<String> = ConfigField::new(
        FieldMeta::new("yorha_protocol", MsgKey::LabelOpenWorkspace)
            .section("main")
            .tooltip(MsgKey::TooltipOpenWorkspace),
        || String::from("F1"),
    );

    pub static FREEZE_GAME: ConfigField<String> = ConfigField::new(
        FieldMeta::new("freeze_game", MsgKey::LabelFreezeGame)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipFreezeGame),
        || String::from("F5"),
    );

    pub static MAX_SPEED: ConfigField<String> = ConfigField::new(
        FieldMeta::new("max_speed", MsgKey::LabelMaxSpeed)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipMaxSpeed),
        || String::from("F6"),
    );

    pub static FREE_CAM: ConfigField<String> = ConfigField::new(
        FieldMeta::new("free_cam", MsgKey::LabelFreeCam)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipFreeCam),
        || String::from("F7"),
    );

    pub static PHASE_JUMP: ConfigField<String> = ConfigField::new(
        FieldMeta::new("phase_jump", MsgKey::LabelPhaseJump)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipPhaseJump),
        || String::from("F8"),
    );

    pub static TOGGLE_GAME_INPUT: ConfigField<String> = ConfigField::new(
        FieldMeta::new("toggle_game_input", MsgKey::LabelToggleInput)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipToggleInput),
        || String::from("F9"),
    );

    pub static ADVANCE_FRAME: ConfigField<String> = ConfigField::new(
        FieldMeta::new("advance_frame", MsgKey::LabelAdvanceFrame)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipAdvanceFrame),
        || String::from("F10"),
    );

    pub static DEV_MODE: ConfigField<String> = ConfigField::new(
        FieldMeta::new("dev_mode", MsgKey::LabelDevMode)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipDevMode),
        || String::from("Insert"),
    );

    pub static WARP_SAVE_1: ConfigField<String> = ConfigField::new(
        FieldMeta::new("warp_save_1", MsgKey::LabelWarpSave1)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipWarpSave1),
        || String::from("Shift+C"),
    );

    pub static WARP_GOTO_1: ConfigField<String> = ConfigField::new(
        FieldMeta::new("warp_goto_1", MsgKey::LabelWarpGoto1)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipWarpGoto1),
        || String::from("Z"),
    );

    pub static WARP_SAVE_2: ConfigField<String> = ConfigField::new(
        FieldMeta::new("warp_save_2", MsgKey::LabelWarpSave2)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipWarpSave2),
        || String::from("Shift+V"),
    );

    pub static WARP_GOTO_2: ConfigField<String> = ConfigField::new(
        FieldMeta::new("warp_goto_2", MsgKey::LabelWarpGoto2)
            .section("yorha_protocol")
            .tooltip(MsgKey::TooltipWarpGoto2),
        || String::from("X"),
    );

    pub static ALL: [&ConfigField<String>; 12] = [
        &OPEN_WORKSPACE,
        &FREEZE_GAME,
        &MAX_SPEED,
        &FREE_CAM,
        &PHASE_JUMP,
        &TOGGLE_GAME_INPUT,
        &ADVANCE_FRAME,
        &DEV_MODE,
        &WARP_SAVE_1,
        &WARP_GOTO_1,
        &WARP_SAVE_2,
        &WARP_GOTO_2,
    ];
}

pub mod workspace {
    use yp_i18n::MsgKey;

    use crate::fields::{AnyField, ConfigField, FieldMeta};

    pub static GAME_KEYBINDS_GLOBAL: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("gameKeybindsGlobal", MsgKey::LabelGlobalKeybinds)
            .tooltip(MsgKey::TooltipGlobalKeybinds),
        || false,
    );

    pub static LOADING_SPEEDUP_ENABLED: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("loadingSpeedupEnabled", MsgKey::LabelLoadingSpeedup)
            .tooltip(MsgKey::TooltipLoadingSpeedup),
        || false,
    );

    pub static SHADERS_ENABLED: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("shadersEnabled", MsgKey::LabelShaders).tooltip(MsgKey::TooltipShaders),
        || true,
    );

    pub static SOUND_ENABLED: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("soundEnabled", MsgKey::LabelSound).tooltip(MsgKey::TooltipSound),
        || true,
    );

    pub static EDITOR_MCP_ENABLED: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("editorMcpEnabled", MsgKey::LabelEditorMcp)
            .tooltip(MsgKey::TooltipEditorMcp),
        || false,
    );

    pub static IMPELLER: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("impeller", MsgKey::LabelImpeller)
            .tooltip(MsgKey::TooltipImpeller)
            .restart_required(),
        || false,
    );

    pub static ALL: [&dyn AnyField; 6] = [
        &GAME_KEYBINDS_GLOBAL,
        &LOADING_SPEEDUP_ENABLED,
        &SHADERS_ENABLED,
        &SOUND_ENABLED,
        &EDITOR_MCP_ENABLED,
        &IMPELLER,
    ];
}

pub mod cheats {
    use yp_i18n::MsgKey;

    use crate::fields::{AnyField, ConfigField, FieldMeta};

    pub static DAMAGE_MULTIPLIER: ConfigField<f64> = ConfigField::new(
        FieldMeta::new("damageMultiplier", MsgKey::LabelDamageMultiplier)
            .tooltip(MsgKey::TooltipDamageMultiplier),
        || 1.0,
    );

    pub static SYNC_ENEMY_LEVELS: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("syncEnemyLevels", MsgKey::LabelSyncEnemyLevels)
            .tooltip(MsgKey::TooltipSyncEnemyLevels),
        || false,
    );

    pub static INFINITE_HEALTH: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("infiniteHealth", MsgKey::LabelInfiniteHealth)
            .tooltip(MsgKey::TooltipInfiniteHealth),
        || false,
    );

    pub static INFINITE_JUMP: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("infiniteJump", MsgKey::LabelInfiniteJump)
            .tooltip(MsgKey::TooltipInfiniteJump),
        || false,
    );

    pub static NO_POD_COOLDOWN: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("noPodCooldown", MsgKey::LabelNoPodCooldown)
            .tooltip(MsgKey::TooltipNoPodCooldown),
        || false,
    );

    pub static INFINITE_AIR_DASH: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("infiniteAirDash", MsgKey::LabelInfiniteAirDash)
            .tooltip(MsgKey::TooltipInfiniteAirDash),
        || false,
    );

    pub static ALL: [&dyn AnyField; 6] = [
        &DAMAGE_MULTIPLIER,
        &SYNC_ENEMY_LEVELS,
        &INFINITE_HEALTH,
        &INFINITE_JUMP,
        &NO_POD_COOLDOWN,
        &INFINITE_AIR_DASH,
    ];
}

pub mod randomizer {
    use yp_i18n::MsgKey;

    use crate::fields::{AnyField, ConfigField, FieldMeta};

    pub static AUTO_START_ON_STARTUP: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("autoStartOnStartup", MsgKey::LabelAutoStart)
            .tooltip(MsgKey::TooltipAutoStart),
        || false,
    );

    pub static RANDOMIZE_GROUND_ENEMIES: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("randomizeGroundEnemies", MsgKey::LabelGroundEnemies)
            .tooltip(MsgKey::TooltipGroundEnemies),
        || true,
    );

    pub static RANDOMIZE_FLYING_ENEMIES: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("randomizeFlyingEnemies", MsgKey::LabelFlyingEnemies)
            .tooltip(MsgKey::TooltipFlyingEnemies),
        || true,
    );

    pub static ALLOW_BIG_ENEMIES: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("allowBigEnemies", MsgKey::LabelAllowBigEnemies)
            .tooltip(MsgKey::TooltipAllowBigEnemies),
        || true,
    );

    pub static INCLUDE_DLC_ENEMIES: ConfigField<bool> = ConfigField::new(
        FieldMeta::new("includeDlcEnemies", MsgKey::LabelIncludeDlcEnemies)
            .tooltip(MsgKey::TooltipIncludeDlcEnemies),
        || true,
    );

    pub static ALL: [&dyn AnyField; 5] = [
        &AUTO_START_ON_STARTUP,
        &RANDOMIZE_GROUND_ENEMIES,
        &RANDOMIZE_FLYING_ENEMIES,
        &ALLOW_BIG_ENEMIES,
        &INCLUDE_DLC_ENEMIES,
    ];
}

pub fn all() -> Vec<&'static dyn AnyField> {
    let mut fields: Vec<&'static dyn AnyField> = Vec::new();
    fields.extend(
        keybinds::ALL
            .iter()
            .map(|field| *field as &'static dyn AnyField),
    );
    fields.extend(workspace::ALL.iter().copied());
    fields.extend(cheats::ALL.iter().copied());
    fields.extend(randomizer::ALL.iter().copied());
    fields
}
