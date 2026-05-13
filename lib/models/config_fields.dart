import 'package:yp_launcher/l10n/app_localizations.dart';

class ConfigField<T> {
  final String key;
  final T defaultValue;
  final String? section;
  final String Function(AppLocalizations l10n) label;
  final String Function(AppLocalizations l10n)? tooltip;
  final num? min;
  final num? max;
  final num? step;
  final List<num>? allowedValues;
  final bool restartRequired;

  const ConfigField({
    required this.key,
    required this.defaultValue,
    this.section,
    required this.label,
    this.tooltip,
    this.min,
    this.max,
    this.step,
    this.allowedValues,
    this.restartRequired = false,
  });
}

class NamsFields {
  static final validateModelData = ConfigField<bool>(
    key: 'validate_model_data',
    defaultValue: false,
    label: (l) => l.labelValidateModelData,
    tooltip: (l) => l.tooltipValidateModelDataSettings,
  );

  static final validateScripts = ConfigField<bool>(
    key: 'validate_scripts',
    defaultValue: false,
    label: (l) => l.labelValidateScripts,
    tooltip: (l) => l.tooltipValidateScripts,
  );

  static final loadingStallHints = ConfigField<bool>(
    key: 'loading_stall_hints',
    defaultValue: true,
    label: (l) => l.labelLoadingStallHints,
    tooltip: (l) => l.tooltipLoadingStallHints,
  );

  static final fixWindTimerBug = ConfigField<bool>(
    key: 'fix_wind_timer_bug',
    defaultValue: true,
    label: (l) => l.labelFixWindTimerBug,
    tooltip: (l) => l.tooltipFixWindTimerBug,
  );

  static final disablePluginLoading = ConfigField<bool>(
    key: 'disable_plugin_loading',
    defaultValue: false,
    label: (l) => l.labelDisablePluginLoading,
    tooltip: (l) => l.tooltipDisablePluginLoading,
  );

  static final disableContentFeatures = ConfigField<bool>(
    key: 'disable_content_features',
    defaultValue: false,
    label: (l) => l.labelDisableContentFeatures,
    tooltip: (l) => l.tooltipDisableContentFeatures,
  );

  static final contentItems = ConfigField<bool>(
    key: 'content_items',
    defaultValue: true,
    label: (l) => l.labelContentItems,
    tooltip: (l) => l.tooltipContentItems,
  );

  static final contentAccessories = ConfigField<bool>(
    key: 'content_accessories',
    defaultValue: true,
    label: (l) => l.labelContentAccessories,
    tooltip: (l) => l.tooltipContentAccessories,
  );

  static final contentAssembleMeshes = ConfigField<bool>(
    key: 'content_assemble_meshes',
    defaultValue: true,
    label: (l) => l.labelContentAssembleMeshes,
    tooltip: (l) => l.tooltipContentAssembleMeshes,
  );

  static final contentQuestIntegration = ConfigField<bool>(
    key: 'content_quest_integration',
    defaultValue: true,
    label: (l) => l.labelContentQuestIntegration,
    tooltip: (l) => l.tooltipContentQuestIntegration,
  );

  static final contentEffectsApplier = ConfigField<bool>(
    key: 'content_effects_applier',
    defaultValue: true,
    label: (l) => l.labelContentEffectsApplier,
    tooltip: (l) => l.tooltipContentEffectsApplier,
  );

  static final contentEquipTracker = ConfigField<bool>(
    key: 'content_equip_tracker',
    defaultValue: true,
    label: (l) => l.labelContentEquipTracker,
    tooltip: (l) => l.tooltipContentEquipTracker,
  );

  static final contentMcd = ConfigField<bool>(
    key: 'content_mcd',
    defaultValue: true,
    label: (l) => l.labelContentMcd,
    tooltip: (l) => l.tooltipContentMcd,
  );

  static final disableReShadeLoading = ConfigField<bool>(
    key: 'disable_reshade_loading',
    defaultValue: false,
    label: (l) => l.labelDisableReShadeLoading,
    tooltip: (l) => l.tooltipDisableReShadeLoading,
  );

  static final disableTextureInjection = ConfigField<bool>(
    key: 'disable_texture_injection',
    defaultValue: false,
    label: (l) => l.labelDisableTextureInjection,
    tooltip: (l) => l.tooltipDisableTextureInjection,
  );

  static final fixCameraAcceleration = ConfigField<bool>(
    key: 'fix_camera_acceleration',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelFixCameraAcceleration,
    tooltip: (l) => l.tooltipFixCameraAcceleration,
  );

  static final sensitivity = ConfigField<double>(
    key: 'sensitivity',
    defaultValue: 2.0,
    section: 'mouse',
    label: (l) => l.labelSensitivity,
    tooltip: (l) => l.tooltipSensitivity,
  );

  static final fixAimAcceleration = ConfigField<bool>(
    key: 'fix_aim_acceleration',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelFixAimAcceleration,
    tooltip: (l) => l.tooltipFixAimAcceleration,
  );

  static final aimSensitivity = ConfigField<double>(
    key: 'aim_sensitivity',
    defaultValue: 0.001,
    section: 'mouse',
    label: (l) => l.labelAimSensitivity,
    tooltip: (l) => l.tooltipAimSensitivity,
  );

  static final aimOutputMultiplier = ConfigField<double>(
    key: 'aim_output_multiplier',
    defaultValue: 15.0,
    section: 'mouse',
    label: (l) => l.labelAimOutputMultiplier,
    tooltip: (l) => l.tooltipAimOutputMultiplier,
  );

  static final disablePodPet = ConfigField<bool>(
    key: 'disable_pod_pet',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelDisablePodPet,
    tooltip: (l) => l.tooltipDisablePodPet,
  );

  static final debugMenuKey = ConfigField<int>(
    key: 'debug_menu_key',
    defaultValue: 0,
    section: 'mouse',
    label: (l) => l.labelDebugMenuKey,
    tooltip: (l) => l.tooltipDebugMenuKey,
  );

  static final evadeKey = ConfigField<int>(
    key: 'evade_key',
    defaultValue: 0,
    section: 'mouse',
    label: (l) => l.labelEvadeKey,
    tooltip: (l) => l.tooltipEvadeKey,
  );

  static final globalHeapExtra = ConfigField<int>(
    key: 'global_heap_extra',
    defaultValue: 0,
    section: 'heap',
    label: (l) => l.heapScriptEngine,
  );

  static final plFileHeapExtra = ConfigField<int>(
    key: 'pl_file_heap_extra',
    defaultValue: 0,
    section: 'heap',
    label: (l) => l.heapPlayerModels,
  );

  static final plVramHeapExtra = ConfigField<int>(
    key: 'pl_vram_heap_extra',
    defaultValue: 0,
    section: 'heap',
    label: (l) => l.heapPlayerTextures,
  );

  static final emBgFileHeapExtra = ConfigField<int>(
    key: 'em_bg_file_heap_extra',
    defaultValue: 0,
    section: 'heap',
    label: (l) => l.heapEnemyBgModels,
  );

  static final emBgVramHeapExtra = ConfigField<int>(
    key: 'em_bg_vram_heap_extra',
    defaultValue: 0,
    section: 'heap',
    label: (l) => l.heapEnemyBgTextures,
  );
}

class LodModFields {
  static final enabled = ConfigField<bool>(
    key: 'enabled',
    defaultValue: false,
    label: (l) => l.labelEnableLodMod,
    tooltip: (l) => l.tooltipEnableLodMod,
  );

  static final lodMultiplier = ConfigField<double>(
    key: 'lod_multiplier',
    defaultValue: 0.0,
    label: (l) => l.labelLodMultiplier,
    tooltip: (l) => l.tooltipLodMultiplier,
    min: 0.0,
    max: 10.0,
    step: 0.1,
    restartRequired: true,
  );

  static final disableManualCulling = ConfigField<bool>(
    key: 'disable_manual_culling',
    defaultValue: false,
    label: (l) => l.labelDisableManualCulling,
    tooltip: (l) => l.tooltipDisableManualCulling,
    restartRequired: true,
  );

  static final aoMultiplierWidth = ConfigField<double>(
    key: 'ao_multiplier_width',
    defaultValue: 1.0,
    label: (l) => l.labelAoWidth,
    tooltip: (l) => l.tooltipAoWidth,
    min: 0.1,
    max: 2.0,
    step: 0.05,
    restartRequired: true,
  );

  static final aoMultiplierHeight = ConfigField<double>(
    key: 'ao_multiplier_height',
    defaultValue: 1.0,
    label: (l) => l.labelAoHeight,
    tooltip: (l) => l.tooltipAoHeight,
    min: 0.1,
    max: 2.0,
    step: 0.05,
    restartRequired: true,
  );

  static final disableVignette = ConfigField<bool>(
    key: 'disable_vignette',
    defaultValue: false,
    label: (l) => l.labelDisableVignette,
    tooltip: (l) => l.tooltipDisableVignette,
  );

  static final shadowResolution = ConfigField<int>(
    key: 'shadow_resolution',
    defaultValue: 2048,
    label: (l) => l.labelShadowResolution,
    tooltip: (l) => l.tooltipShadowResolution,
    allowedValues: const [512, 1024, 2048, 4096, 8192],
    restartRequired: true,
  );

  static final shadowDistanceMultiplier = ConfigField<double>(
    key: 'shadow_distance_multiplier',
    defaultValue: 1.0,
    label: (l) => l.labelDistanceMultiplier,
    tooltip: (l) => l.tooltipDistanceMultiplier,
    min: 0.1,
    max: 10.0,
    step: 0.1,
  );

  static final shadowDistanceMinimum = ConfigField<double>(
    key: 'shadow_distance_minimum',
    defaultValue: 0.0,
    label: (l) => l.labelDistanceMinimum,
    tooltip: (l) => l.tooltipDistanceMinimum,
    min: 0.0,
    max: 500.0,
    step: 1.0,
  );

  static final shadowDistanceMaximum = ConfigField<double>(
    key: 'shadow_distance_maximum',
    defaultValue: 0.0,
    label: (l) => l.labelDistanceMaximum,
    tooltip: (l) => l.tooltipDistanceMaximum,
    min: 0.0,
    max: 2000.0,
    step: 5.0,
  );

  static final shadowDistancePss = ConfigField<double>(
    key: 'shadow_distance_pss',
    defaultValue: 0.0,
    label: (l) => l.labelDistancePss,
    tooltip: (l) => l.tooltipDistancePss,
    min: 0.0,
    max: 2000.0,
    step: 5.0,
  );

  static final shadowFilterStrengthBias = ConfigField<double>(
    key: 'shadow_filter_strength_bias',
    defaultValue: 0.0,
    label: (l) => l.labelFilterStrengthBias,
    tooltip: (l) => l.tooltipFilterStrengthBias,
    min: -5.0,
    max: 5.0,
    step: 0.1,
  );

  static final shadowFilterStrengthMinimum = ConfigField<double>(
    key: 'shadow_filter_strength_minimum',
    defaultValue: 0.0,
    label: (l) => l.labelFilterStrengthMin,
    tooltip: (l) => l.tooltipFilterStrengthMin,
    min: 0.0,
    max: 10.0,
    step: 0.1,
  );

  static final shadowFilterStrengthMaximum = ConfigField<double>(
    key: 'shadow_filter_strength_maximum',
    defaultValue: 0.0,
    label: (l) => l.labelFilterStrengthMax,
    tooltip: (l) => l.tooltipFilterStrengthMax,
    min: 0.0,
    max: 10.0,
    step: 0.1,
  );

  static final shadowModelHq = ConfigField<bool>(
    key: 'shadow_model_hq',
    defaultValue: false,
    label: (l) => l.labelHqShadowModels,
    tooltip: (l) => l.tooltipHqShadowModels,
    restartRequired: true,
  );

  static final shadowModelForceAll = ConfigField<bool>(
    key: 'shadow_model_force_all',
    defaultValue: false,
    label: (l) => l.labelForceAllShadowModels,
    tooltip: (l) => l.tooltipForceAllShadowModels,
    restartRequired: true,
  );

  static final giEnabled = ConfigField<bool>(
    key: 'gi_enabled',
    defaultValue: false,
    label: (l) => l.labelGiEnabled,
    tooltip: (l) => l.tooltipGiEnabled,
  );

  static final giWorkgroupSize = ConfigField<int>(
    key: 'gi_workgroup_size',
    defaultValue: 128,
    label: (l) => l.labelGiWorkgroupSize,
    tooltip: (l) => l.tooltipGiWorkgroupSize,
    allowedValues: const [16, 32, 64, 128],
  );

  static final giMinLightExtent = ConfigField<double>(
    key: 'gi_min_light_extent',
    defaultValue: 0.0,
    label: (l) => l.labelGiMinLightExtent,
    tooltip: (l) => l.tooltipGiMinLightExtent,
    min: 0.0,
    max: 1.0,
    step: 0.05,
  );

  static final fpsUncapInMenus = ConfigField<bool>(
    key: 'fps_uncap_in_menus',
    defaultValue: false,
    label: (l) => l.labelFpsUncapInMenus,
    tooltip: (l) => l.tooltipFpsUncapInMenus,
  );

  static final fpsUncapInGameplay = ConfigField<bool>(
    key: 'fps_uncap_in_gameplay',
    defaultValue: false,
    label: (l) => l.labelFpsUncapInGameplay,
    tooltip: (l) => l.tooltipFpsUncapInGameplay,
  );
}

class TextureInjectionFields {
  static final vramBudgetMb = ConfigField<int>(
    key: 'vram_budget_mb',
    defaultValue: 0,
    label: (l) => l.labelVramBudget,
    tooltip: (l) => l.tooltipVramBudget,
  );

  static final streamingEnabled = ConfigField<bool>(
    key: 'streaming_enabled',
    defaultValue: true,
    label: (l) => l.labelStreamingEnabled,
    tooltip: (l) => l.tooltipStreamingEnabled,
  );

  static final loadOnlyRelevant = ConfigField<bool>(
    key: 'load_only_relevant',
    defaultValue: false,
    label: (l) => l.labelLoadOnlyRelevant,
    tooltip: (l) => l.tooltipLoadOnlyRelevant,
  );

  static final loadOrder = ConfigField<List<String>>(
    key: 'load_order',
    defaultValue: const [],
    label: (l) => l.cardTextureConfig,
  );

  static final disabledPacks = ConfigField<List<String>>(
    key: 'disabled_packs',
    defaultValue: const [],
    label: (l) => l.cardTextureConfig,
  );
}

class YpKeybindFields {
  static final openWorkspace = ConfigField<String>(
    key: 'yorha_protocol',
    defaultValue: 'F1',
    section: 'main',
    label: (l) => l.labelOpenWorkspace,
    tooltip: (l) => l.tooltipOpenWorkspace,
  );

  static final freezeGame = ConfigField<String>(
    key: 'freeze_game',
    defaultValue: 'F5',
    section: 'yorha_protocol',
    label: (l) => l.labelFreezeGame,
    tooltip: (l) => l.tooltipFreezeGame,
  );

  static final maxSpeed = ConfigField<String>(
    key: 'max_speed',
    defaultValue: 'F6',
    section: 'yorha_protocol',
    label: (l) => l.labelMaxSpeed,
    tooltip: (l) => l.tooltipMaxSpeed,
  );

  static final freeCam = ConfigField<String>(
    key: 'free_cam',
    defaultValue: 'F7',
    section: 'yorha_protocol',
    label: (l) => l.labelFreeCam,
    tooltip: (l) => l.tooltipFreeCam,
  );

  static final phaseJump = ConfigField<String>(
    key: 'phase_jump',
    defaultValue: 'F8',
    section: 'yorha_protocol',
    label: (l) => l.labelPhaseJump,
    tooltip: (l) => l.tooltipPhaseJump,
  );

  static final toggleGameInput = ConfigField<String>(
    key: 'toggle_game_input',
    defaultValue: 'F9',
    section: 'yorha_protocol',
    label: (l) => l.labelToggleInput,
    tooltip: (l) => l.tooltipToggleInput,
  );

  static final advanceFrame = ConfigField<String>(
    key: 'advance_frame',
    defaultValue: 'F10',
    section: 'yorha_protocol',
    label: (l) => l.labelAdvanceFrame,
    tooltip: (l) => l.tooltipAdvanceFrame,
  );

  static final devMode = ConfigField<String>(
    key: 'dev_mode',
    defaultValue: 'Insert',
    section: 'yorha_protocol',
    label: (l) => l.labelDevMode,
    tooltip: (l) => l.tooltipDevMode,
  );
}

class YpWorkspaceFields {
  static final gameKeybindsGlobal = ConfigField<bool>(
    key: 'gameKeybindsGlobal',
    defaultValue: false,
    label: (l) => l.labelGlobalKeybinds,
    tooltip: (l) => l.tooltipGlobalKeybinds,
  );

  static final loadingSpeedupEnabled = ConfigField<bool>(
    key: 'loadingSpeedupEnabled',
    defaultValue: false,
    label: (l) => l.labelLoadingSpeedup,
    tooltip: (l) => l.tooltipLoadingSpeedup,
  );

  static final shadersEnabled = ConfigField<bool>(
    key: 'shadersEnabled',
    defaultValue: false,
    label: (l) => l.labelShaders,
    tooltip: (l) => l.tooltipShaders,
  );

  static final soundEnabled = ConfigField<bool>(
    key: 'soundEnabled',
    defaultValue: false,
    label: (l) => l.labelSound,
    tooltip: (l) => l.tooltipSound,
  );
}

class YpCheatsFields {
  static final damageMultiplier = ConfigField<double>(
    key: 'damageMultiplier',
    defaultValue: 1.0,
    label: (l) => l.labelDamageMultiplier,
    tooltip: (l) => l.tooltipDamageMultiplier,
  );

  static final syncEnemyLevels = ConfigField<bool>(
    key: 'syncEnemyLevels',
    defaultValue: false,
    label: (l) => l.labelSyncEnemyLevels,
    tooltip: (l) => l.tooltipSyncEnemyLevels,
  );

  static final infiniteHealth = ConfigField<bool>(
    key: 'infiniteHealth',
    defaultValue: false,
    label: (l) => l.labelInfiniteHealth,
    tooltip: (l) => l.tooltipInfiniteHealth,
  );

  static final infiniteJump = ConfigField<bool>(
    key: 'infiniteJump',
    defaultValue: false,
    label: (l) => l.labelInfiniteJump,
    tooltip: (l) => l.tooltipInfiniteJump,
  );

  static final noPodCooldown = ConfigField<bool>(
    key: 'noPodCooldown',
    defaultValue: false,
    label: (l) => l.labelNoPodCooldown,
    tooltip: (l) => l.tooltipNoPodCooldown,
  );

  static final infiniteAirDash = ConfigField<bool>(
    key: 'infiniteAirDash',
    defaultValue: false,
    label: (l) => l.labelInfiniteAirDash,
    tooltip: (l) => l.tooltipInfiniteAirDash,
  );
}

class YpRandomizerFields {
  static final autoStartOnStartup = ConfigField<bool>(
    key: 'autoStartOnStartup',
    defaultValue: false,
    label: (l) => l.labelAutoStart,
    tooltip: (l) => l.tooltipAutoStart,
  );

  static final randomizeGroundEnemies = ConfigField<bool>(
    key: 'randomizeGroundEnemies',
    defaultValue: false,
    label: (l) => l.labelGroundEnemies,
    tooltip: (l) => l.tooltipGroundEnemies,
  );

  static final randomizeFlyingEnemies = ConfigField<bool>(
    key: 'randomizeFlyingEnemies',
    defaultValue: false,
    label: (l) => l.labelFlyingEnemies,
    tooltip: (l) => l.tooltipFlyingEnemies,
  );

  static final allowBigEnemies = ConfigField<bool>(
    key: 'allowBigEnemies',
    defaultValue: false,
    label: (l) => l.labelAllowBigEnemies,
    tooltip: (l) => l.tooltipAllowBigEnemies,
  );

  static final includeDlcEnemies = ConfigField<bool>(
    key: 'includeDlcEnemies',
    defaultValue: false,
    label: (l) => l.labelIncludeDlcEnemies,
    tooltip: (l) => l.tooltipIncludeDlcEnemies,
  );
}
