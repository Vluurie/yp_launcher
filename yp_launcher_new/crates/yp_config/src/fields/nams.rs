use yp_i18n::MsgKey;

use super::{AnyField, ConfigField, FieldMeta};

pub static VALIDATE_MODEL_DATA: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("validate_model_data", MsgKey::LabelValidateModelData)
        .tooltip(MsgKey::TooltipValidateModelDataSettings),
    || false,
);

pub static VALIDATE_SCRIPTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("validate_scripts", MsgKey::LabelValidateScripts)
        .tooltip(MsgKey::TooltipValidateScripts),
    || false,
);

pub static LOADING_STALL_HINTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("loading_stall_hints", MsgKey::LabelLoadingStallHints)
        .tooltip(MsgKey::TooltipLoadingStallHints),
    || true,
);

pub static FIX_WIND_TIMER_BUG: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("fix_wind_timer_bug", MsgKey::LabelFixWindTimerBug)
        .tooltip(MsgKey::TooltipFixWindTimerBug),
    || true,
);

pub static DISABLE_PLUGIN_LOADING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_plugin_loading", MsgKey::LabelDisablePluginLoading)
        .tooltip(MsgKey::TooltipDisablePluginLoading),
    || false,
);

pub static DISABLE_MAIN_OVERLAY: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_main_overlay", MsgKey::LabelDisableMainOverlay)
        .tooltip(MsgKey::TooltipDisableMainOverlay),
    || false,
);

pub static DISABLE_CONTENT_FEATURES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "disable_content_features",
        MsgKey::LabelDisableContentFeatures,
    )
    .tooltip(MsgKey::TooltipDisableContentFeatures),
    || false,
);

pub static OUTFIT_SWAP_VISUAL_EFFECTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "outfit_swap_visual_effects",
        MsgKey::LabelOutfitSwapVisualEffects,
    )
    .tooltip(MsgKey::TooltipOutfitSwapVisualEffects),
    || true,
);

pub static EXPERIMENTAL_DEFAULT_OUTFITS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "experimental_default_outfits",
        MsgKey::LabelExperimentalDefaultOutfits,
    )
    .tooltip(MsgKey::TooltipExperimentalDefaultOutfits)
    .restart_required(),
    || true,
);

pub static DISABLE_RESHADE_LOADING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "disable_reshade_loading",
        MsgKey::LabelDisableReShadeLoading,
    )
    .tooltip(MsgKey::TooltipDisableReShadeLoading),
    || false,
);

pub static DISABLE_3DMIGOTO_LOADING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "disable_3dmigoto_loading",
        MsgKey::LabelDisable3dmigotoLoading,
    )
    .tooltip(MsgKey::TooltipDisable3dmigotoLoading),
    || false,
);

pub static DISABLE_GAME_MODS_LOADING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "disable_game_mods_loading",
        MsgKey::LabelDisableGameModsLoading,
    )
    .tooltip(MsgKey::TooltipDisableGameModsLoading),
    || false,
);

pub static DISABLE_GRAPHIC_MODS_LOADING: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "disable_graphic_mods_loading",
        MsgKey::LabelDisableGraphicModsLoading,
    )
    .tooltip(MsgKey::TooltipDisableGraphicModsLoading)
    .restart_required(),
    || false,
);

pub static DISABLE_TEXTURE_INJECTION: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "disable_texture_injection",
        MsgKey::LabelDisableTextureInjection,
    )
    .tooltip(MsgKey::TooltipDisableTextureInjection),
    || false,
);

pub static HIDE_SUBTITLE_OVERLAY: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("hide_subtitle_overlay", MsgKey::LabelHideSubtitleOverlay)
        .section("cutscene")
        .tooltip(MsgKey::TooltipHideSubtitleOverlay),
    || false,
);

pub static KEEP_SUBTITLE_TEXT: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("keep_subtitle_text", MsgKey::LabelKeepSubtitleText)
        .section("cutscene")
        .tooltip(MsgKey::TooltipKeepSubtitleText),
    || false,
);

pub static HIDE_SUBTITLE_IN_EVENTS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("hide_subtitle_in_events", MsgKey::LabelHideSubtitleInEvents)
        .section("cutscene")
        .tooltip(MsgKey::TooltipHideSubtitleInEvents),
    || false,
);

pub static SOLID_LETTERBOX_BARS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("solid_letterbox_bars", MsgKey::LabelSolidLetterboxBars)
        .section("cutscene")
        .tooltip(MsgKey::TooltipSolidLetterboxBars),
    || false,
);

pub static DISABLE_SPLASH_SCREEN: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_splash_screen", MsgKey::LabelDisableSplashScreen)
        .tooltip(MsgKey::TooltipDisableSplashScreen),
    || false,
);

pub static SKIP_STARTUP_LOGOS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("skip_startup_logos", MsgKey::LabelSkipStartupLogos)
        .tooltip(MsgKey::TooltipSkipStartupLogos)
        .restart_required(),
    || false,
);

pub static DISABLE_DEBUG_HOTKEYS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_debug_hotkeys", MsgKey::LabelDisableDebugHotkeys)
        .tooltip(MsgKey::TooltipDisableDebugHotkeys),
    || true,
);

pub static INGAME_SETTINGS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("ingame_settings", MsgKey::LabelIngameSettings)
        .tooltip(MsgKey::TooltipIngameSettings)
        .restart_required(),
    || true,
);

pub static DISABLE_INPUT_FEATURES: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("disable_input_features", MsgKey::LabelDisableInputFeatures)
        .tooltip(MsgKey::TooltipDisableInputFeatures)
        .restart_required(),
    || false,
);

pub static FIX_CAMERA_ACCELERATION: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "fix_camera_acceleration",
        MsgKey::LabelFixCameraAcceleration,
    )
    .section("mouse")
    .tooltip(MsgKey::TooltipFixCameraAcceleration),
    || false,
);

pub static SENSITIVITY: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("sensitivity", MsgKey::LabelSensitivity)
        .section("mouse")
        .tooltip(MsgKey::TooltipSensitivity)
        .range(0.25, 10.0)
        .step(0.25),
    || 2.0,
);

pub static THIRD_PERSON_MODE: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("third_person_mode", MsgKey::LabelThirdPersonMode)
        .section("mouse")
        .tooltip(MsgKey::TooltipThirdPersonMode),
    || false,
);

pub static THIRD_PERSON_CHAR_FOLLOW: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "third_person_char_follow",
        MsgKey::LabelThirdPersonCharFollow,
    )
    .section("mouse")
    .tooltip(MsgKey::TooltipThirdPersonCharFollow),
    || false,
);

pub static THIRD_PERSON_SMOOTHING: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("third_person_smoothing", MsgKey::LabelThirdPersonSmoothing)
        .section("mouse")
        .tooltip(MsgKey::TooltipThirdPersonSmoothing)
        .range(0.0, 1.0)
        .step(0.05),
    || 0.0,
);

pub static THIRD_PERSON_SENSITIVITY_X: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("third_person_sensitivity_x", MsgKey::LabelThirdPersonSensX)
        .section("mouse")
        .tooltip(MsgKey::TooltipThirdPersonSensX)
        .range(-100.0, 100.0)
        .step(0.1),
    || 1.0,
);

pub static THIRD_PERSON_SENSITIVITY_Y: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("third_person_sensitivity_y", MsgKey::LabelThirdPersonSensY)
        .section("mouse")
        .tooltip(MsgKey::TooltipThirdPersonSensY)
        .range(-100.0, 100.0)
        .step(0.1),
    || 1.0,
);

pub static AIM_MODE: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("aim_mode", MsgKey::LabelAimMode)
        .section("mouse")
        .tooltip(MsgKey::TooltipAimMode),
    || false,
);

pub static AIM_CROSSHAIR: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("aim_crosshair", MsgKey::LabelAimCrosshair)
        .section("mouse")
        .tooltip(MsgKey::TooltipAimCrosshair),
    || false,
);

pub static AIM_CROSSHAIR_ALWAYS: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("aim_crosshair_always", MsgKey::LabelAimCrosshairAlways)
        .section("mouse")
        .tooltip(MsgKey::TooltipAimCrosshairAlways),
    || false,
);

pub static AIM_SENSITIVITY: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("aim_sensitivity", MsgKey::LabelAimSensitivity)
        .section("mouse")
        .tooltip(MsgKey::TooltipAimSensitivity)
        .range(0.0002, 0.01)
        .step(0.0001),
    || 0.001,
);

pub static AIM_SENSITIVITY_X: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("aim_sensitivity_x", MsgKey::LabelAimSensX)
        .section("mouse")
        .tooltip(MsgKey::TooltipAimSensX)
        .range(-3.0, 3.0)
        .step(0.05),
    || 1.0,
);

pub static AIM_SENSITIVITY_Y: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("aim_sensitivity_y", MsgKey::LabelAimSensY)
        .section("mouse")
        .tooltip(MsgKey::TooltipAimSensY)
        .range(-3.0, 3.0)
        .step(0.05),
    || 1.0,
);

pub static AIM_OUTPUT_MULTIPLIER: ConfigField<f64> = ConfigField::new(
    FieldMeta::new("aim_output_multiplier", MsgKey::LabelAimOutputMultiplier)
        .section("mouse")
        .tooltip(MsgKey::TooltipAimOutputMultiplier)
        .range(5.0, 30.0)
        .step(0.5),
    || 15.0,
);

pub static MOVEMENT_DISABLE_TAP_EVADE: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("movement_disable_tap_evade", MsgKey::LabelDisableTapEvade)
        .section("mouse")
        .tooltip(MsgKey::TooltipDisableTapEvade),
    || false,
);

pub static MISC_DISABLE_POD_PET: ConfigField<bool> = ConfigField::new(
    FieldMeta::new("misc_disable_pod_pet", MsgKey::LabelDisablePodPet)
        .section("mouse")
        .tooltip(MsgKey::TooltipDisablePodPet)
        .restart_required(),
    || false,
);

pub static MISC_CUSTOM_CURSOR_MENU: ConfigField<String> = ConfigField::new(
    FieldMeta::new("misc_custom_cursor_menu", MsgKey::LabelCustomCursorMenu)
        .section("mouse")
        .tooltip(MsgKey::TooltipCustomCursorMenu)
        .restart_required(),
    String::new,
);

pub static MISC_CUSTOM_CURSOR_HACKING: ConfigField<String> = ConfigField::new(
    FieldMeta::new(
        "misc_custom_cursor_hacking",
        MsgKey::LabelCustomCursorHacking,
    )
    .section("mouse")
    .tooltip(MsgKey::TooltipCustomCursorHacking)
    .restart_required(),
    String::new,
);

pub static MISC_DISABLE_DEFAULT_CURSOR: ConfigField<bool> = ConfigField::new(
    FieldMeta::new(
        "misc_disable_default_cursor",
        MsgKey::LabelDisableDefaultCursor,
    )
    .section("mouse")
    .tooltip(MsgKey::TooltipDisableDefaultCursor)
    .restart_required(),
    || false,
);

pub static MISC_OPEN_DEBUG_MENU: ConfigField<i64> = ConfigField::new(
    FieldMeta::new("misc_open_debug_menu", MsgKey::LabelDebugMenuKey)
        .section("mouse")
        .tooltip(MsgKey::TooltipDebugMenuKey)
        .restart_required(),
    || 0,
);

pub static ALL: [&dyn AnyField; 43] = [
    &VALIDATE_MODEL_DATA,
    &VALIDATE_SCRIPTS,
    &LOADING_STALL_HINTS,
    &FIX_WIND_TIMER_BUG,
    &DISABLE_PLUGIN_LOADING,
    &DISABLE_MAIN_OVERLAY,
    &DISABLE_CONTENT_FEATURES,
    &OUTFIT_SWAP_VISUAL_EFFECTS,
    &EXPERIMENTAL_DEFAULT_OUTFITS,
    &DISABLE_RESHADE_LOADING,
    &DISABLE_3DMIGOTO_LOADING,
    &DISABLE_GAME_MODS_LOADING,
    &DISABLE_GRAPHIC_MODS_LOADING,
    &DISABLE_TEXTURE_INJECTION,
    &HIDE_SUBTITLE_OVERLAY,
    &KEEP_SUBTITLE_TEXT,
    &HIDE_SUBTITLE_IN_EVENTS,
    &SOLID_LETTERBOX_BARS,
    &DISABLE_SPLASH_SCREEN,
    &SKIP_STARTUP_LOGOS,
    &DISABLE_DEBUG_HOTKEYS,
    &INGAME_SETTINGS,
    &DISABLE_INPUT_FEATURES,
    &FIX_CAMERA_ACCELERATION,
    &SENSITIVITY,
    &THIRD_PERSON_MODE,
    &THIRD_PERSON_CHAR_FOLLOW,
    &THIRD_PERSON_SMOOTHING,
    &THIRD_PERSON_SENSITIVITY_X,
    &THIRD_PERSON_SENSITIVITY_Y,
    &AIM_MODE,
    &AIM_CROSSHAIR,
    &AIM_CROSSHAIR_ALWAYS,
    &AIM_SENSITIVITY,
    &AIM_SENSITIVITY_X,
    &AIM_SENSITIVITY_Y,
    &AIM_OUTPUT_MULTIPLIER,
    &MOVEMENT_DISABLE_TAP_EVADE,
    &MISC_DISABLE_POD_PET,
    &MISC_CUSTOM_CURSOR_MENU,
    &MISC_CUSTOM_CURSOR_HACKING,
    &MISC_DISABLE_DEFAULT_CURSOR,
    &MISC_OPEN_DEBUG_MENU,
];

pub static MOUSE: [&dyn AnyField; 20] = [
    &FIX_CAMERA_ACCELERATION,
    &SENSITIVITY,
    &THIRD_PERSON_MODE,
    &THIRD_PERSON_CHAR_FOLLOW,
    &THIRD_PERSON_SMOOTHING,
    &THIRD_PERSON_SENSITIVITY_X,
    &THIRD_PERSON_SENSITIVITY_Y,
    &AIM_MODE,
    &AIM_CROSSHAIR,
    &AIM_CROSSHAIR_ALWAYS,
    &AIM_SENSITIVITY,
    &AIM_SENSITIVITY_X,
    &AIM_SENSITIVITY_Y,
    &AIM_OUTPUT_MULTIPLIER,
    &MOVEMENT_DISABLE_TAP_EVADE,
    &MISC_DISABLE_POD_PET,
    &MISC_CUSTOM_CURSOR_MENU,
    &MISC_CUSTOM_CURSOR_HACKING,
    &MISC_DISABLE_DEFAULT_CURSOR,
    &MISC_OPEN_DEBUG_MENU,
];
