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

  Object? rawFrom(Map<String, dynamic> values) {
    Map<String, dynamic>? current = values;
    if (section != null) {
      for (final part in section!.split('.')) {
        final next = current?[part];
        if (next is! Map<String, dynamic>) return null;
        current = next;
      }
    }
    return current?[key];
  }
}

extension ConfigFieldBoolRead on ConfigField<bool> {
  bool valueIn(Map<String, dynamic> values) {
    final raw = rawFrom(values);
    return raw is bool ? raw : defaultValue;
  }
}

extension ConfigFieldIntRead on ConfigField<int> {
  int valueIn(Map<String, dynamic> values) {
    final raw = rawFrom(values);
    if (raw is int) return raw;
    if (raw is num) return raw.toInt();
    return defaultValue;
  }
}

extension ConfigFieldDoubleRead on ConfigField<double> {
  double valueIn(Map<String, dynamic> values) {
    final raw = rawFrom(values);
    return raw is num ? raw.toDouble() : defaultValue;
  }
}

extension ConfigFieldStringRead on ConfigField<String> {
  String valueIn(Map<String, dynamic> values) {
    final raw = rawFrom(values);
    return raw is String ? raw : defaultValue;
  }
}

extension ConfigFieldListRead on ConfigField<List<dynamic>> {
  List<dynamic> valueIn(Map<String, dynamic> values) {
    final raw = rawFrom(values);
    return raw is List ? raw : defaultValue;
  }
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

  static final contentEffectAreas = ConfigField<bool>(
    key: 'content_effect_areas',
    defaultValue: true,
    label: (l) => l.labelContentEffectAreas,
    tooltip: (l) => l.tooltipContentEffectAreas,
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

  static final contentBuddyRubySelector = ConfigField<bool>(
    key: 'content_buddy_ruby_selector',
    defaultValue: true,
    label: (l) => l.labelContentBuddyRubySelector,
    tooltip: (l) => l.tooltipContentBuddyRubySelector,
  );

  static final outfitSwapVisualEffects = ConfigField<bool>(
    key: 'outfit_swap_visual_effects',
    defaultValue: true,
    label: (l) => l.labelOutfitSwapVisualEffects,
    tooltip: (l) => l.tooltipOutfitSwapVisualEffects,
  );

  static final experimentalDefaultOutfits = ConfigField<bool>(
    key: 'experimental_default_outfits',
    defaultValue: false,
    restartRequired: true,
    label: (l) => l.labelExperimentalDefaultOutfits,
    tooltip: (l) => l.tooltipExperimentalDefaultOutfits,
  );

  static final disableReShadeLoading = ConfigField<bool>(
    key: 'disable_reshade_loading',
    defaultValue: false,
    label: (l) => l.labelDisableReShadeLoading,
    tooltip: (l) => l.tooltipDisableReShadeLoading,
  );

  static final disable3dmigotoLoading = ConfigField<bool>(
    key: 'disable_3dmigoto_loading',
    defaultValue: false,
    label: (l) => l.labelDisable3dmigotoLoading,
    tooltip: (l) => l.tooltipDisable3dmigotoLoading,
  );

  static final disableGameModsLoading = ConfigField<bool>(
    key: 'disable_game_mods_loading',
    defaultValue: false,
    label: (l) => l.labelDisableGameModsLoading,
    tooltip: (l) => l.tooltipDisableGameModsLoading,
  );

  static final disableTextureInjection = ConfigField<bool>(
    key: 'disable_texture_injection',
    defaultValue: false,
    label: (l) => l.labelDisableTextureInjection,
    tooltip: (l) => l.tooltipDisableTextureInjection,
  );

  static final hideSubtitleOverlay = ConfigField<bool>(
    key: 'hide_subtitle_overlay',
    defaultValue: false,
    section: 'cutscene',
    label: (l) => l.labelHideSubtitleOverlay,
    tooltip: (l) => l.tooltipHideSubtitleOverlay,
  );

  static final keepSubtitleText = ConfigField<bool>(
    key: 'keep_subtitle_text',
    defaultValue: false,
    section: 'cutscene',
    label: (l) => l.labelKeepSubtitleText,
    tooltip: (l) => l.tooltipKeepSubtitleText,
  );

  static final hideSubtitleInEvents = ConfigField<bool>(
    key: 'hide_subtitle_in_events',
    defaultValue: false,
    section: 'cutscene',
    label: (l) => l.labelHideSubtitleInEvents,
    tooltip: (l) => l.tooltipHideSubtitleInEvents,
  );

  static final solidLetterboxBars = ConfigField<bool>(
    key: 'solid_letterbox_bars',
    defaultValue: false,
    section: 'cutscene',
    label: (l) => l.labelSolidLetterboxBars,
    tooltip: (l) => l.tooltipSolidLetterboxBars,
  );

  static final disableSplashScreen = ConfigField<bool>(
    key: 'disable_splash_screen',
    defaultValue: false,
    label: (l) => l.labelDisableSplashScreen,
    tooltip: (l) => l.tooltipDisableSplashScreen,
  );

  static final skipStartupLogos = ConfigField<bool>(
    key: 'skip_startup_logos',
    defaultValue: false,
    label: (l) => l.labelSkipStartupLogos,
    tooltip: (l) => l.tooltipSkipStartupLogos,
    restartRequired: true,
  );

  static final disableDebugHotkeys = ConfigField<bool>(
    key: 'disable_debug_hotkeys',
    defaultValue: true,
    label: (l) => l.labelDisableDebugHotkeys,
    tooltip: (l) => l.tooltipDisableDebugHotkeys,
  );

  static final disableInputFeatures = ConfigField<bool>(
    key: 'disable_input_features',
    defaultValue: false,
    restartRequired: true,
    label: (l) => l.labelDisableInputFeatures,
    tooltip: (l) => l.tooltipDisableInputFeatures,
  );

  static final fixCameraAcceleration = ConfigField<bool>(
    key: 'fix_camera_acceleration',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelFixCameraAcceleration,
    tooltip: (l) => l.tooltipFixCameraAcceleration,
    restartRequired: true,
  );

  static final sensitivity = ConfigField<double>(
    key: 'sensitivity',
    defaultValue: 2.0,
    section: 'mouse',
    label: (l) => l.labelSensitivity,
    tooltip: (l) => l.tooltipSensitivity,
    min: 0.25,
    max: 10.0,
    step: 0.25,
    restartRequired: true,
  );

  static final thirdPersonMode = ConfigField<bool>(
    key: 'third_person_mode',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelThirdPersonMode,
    tooltip: (l) => l.tooltipThirdPersonMode,
  );

  static final thirdPersonCharFollow = ConfigField<bool>(
    key: 'third_person_char_follow',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelThirdPersonCharFollow,
    tooltip: (l) => l.tooltipThirdPersonCharFollow,
  );

  static final thirdPersonSmoothing = ConfigField<double>(
    key: 'third_person_smoothing',
    defaultValue: 0.0,
    section: 'mouse',
    label: (l) => l.labelThirdPersonSmoothing,
    tooltip: (l) => l.tooltipThirdPersonSmoothing,
    min: 0.0,
    max: 1.0,
    step: 0.05,
  );

  static final thirdPersonSensitivityX = ConfigField<double>(
    key: 'third_person_sensitivity_x',
    defaultValue: 1.0,
    section: 'mouse',
    label: (l) => l.labelThirdPersonSensX,
    tooltip: (l) => l.tooltipThirdPersonSensX,
    min: -5.0,
    max: 5.0,
    step: 0.05,
  );

  static final thirdPersonSensitivityY = ConfigField<double>(
    key: 'third_person_sensitivity_y',
    defaultValue: 1.0,
    section: 'mouse',
    label: (l) => l.labelThirdPersonSensY,
    tooltip: (l) => l.tooltipThirdPersonSensY,
    min: -5.0,
    max: 5.0,
    step: 0.05,
  );

  static final aimMode = ConfigField<bool>(
    key: 'aim_mode',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelAimMode,
    tooltip: (l) => l.tooltipAimMode,
  );

  static final aimCrosshair = ConfigField<bool>(
    key: 'aim_crosshair',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelAimCrosshair,
    tooltip: (l) => l.tooltipAimCrosshair,
  );

  static final aimCrosshairAlways = ConfigField<bool>(
    key: 'aim_crosshair_always',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelAimCrosshairAlways,
    tooltip: (l) => l.tooltipAimCrosshairAlways,
  );

  static final aimSensitivity = ConfigField<double>(
    key: 'aim_sensitivity',
    defaultValue: 0.001,
    section: 'mouse',
    label: (l) => l.labelAimSensitivity,
    tooltip: (l) => l.tooltipAimSensitivity,
    min: 0.0002,
    max: 0.01,
    step: 0.0001,
  );

  static final aimSensitivityX = ConfigField<double>(
    key: 'aim_sensitivity_x',
    defaultValue: 1.0,
    section: 'mouse',
    label: (l) => l.labelAimSensX,
    tooltip: (l) => l.tooltipAimSensX,
    min: -3.0,
    max: 3.0,
    step: 0.05,
  );

  static final aimSensitivityY = ConfigField<double>(
    key: 'aim_sensitivity_y',
    defaultValue: 1.0,
    section: 'mouse',
    label: (l) => l.labelAimSensY,
    tooltip: (l) => l.tooltipAimSensY,
    min: -3.0,
    max: 3.0,
    step: 0.05,
  );

  static final aimOutputMultiplier = ConfigField<double>(
    key: 'aim_output_multiplier',
    defaultValue: 15.0,
    section: 'mouse',
    label: (l) => l.labelAimOutputMultiplier,
    tooltip: (l) => l.tooltipAimOutputMultiplier,
    min: 5.0,
    max: 30.0,
    step: 0.5,
  );

  static final movementDisableTapEvade = ConfigField<bool>(
    key: 'movement_disable_tap_evade',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelDisableTapEvade,
    tooltip: (l) => l.tooltipDisableTapEvade,
  );

  static final miscDisablePodPet = ConfigField<bool>(
    key: 'misc_disable_pod_pet',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelDisablePodPet,
    tooltip: (l) => l.tooltipDisablePodPet,
    restartRequired: true,
  );

  static final miscCustomCursorMenu = ConfigField<String>(
    key: 'misc_custom_cursor_menu',
    defaultValue: '',
    section: 'mouse',
    label: (l) => l.labelCustomCursorMenu,
    tooltip: (l) => l.tooltipCustomCursorMenu,
    restartRequired: true,
  );

  static final miscCustomCursorHacking = ConfigField<String>(
    key: 'misc_custom_cursor_hacking',
    defaultValue: '',
    section: 'mouse',
    label: (l) => l.labelCustomCursorHacking,
    tooltip: (l) => l.tooltipCustomCursorHacking,
    restartRequired: true,
  );

  static final miscDisableDefaultCursor = ConfigField<bool>(
    key: 'misc_disable_default_cursor',
    defaultValue: false,
    section: 'mouse',
    label: (l) => l.labelDisableDefaultCursor,
    tooltip: (l) => l.tooltipDisableDefaultCursor,
    restartRequired: true,
  );

  static final miscOpenDebugMenu = ConfigField<int>(
    key: 'misc_open_debug_menu',
    defaultValue: 0,
    section: 'mouse',
    label: (l) => l.labelDebugMenuKey,
    tooltip: (l) => l.tooltipDebugMenuKey,
    restartRequired: true,
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

class NamsBindingFields {
  static ConfigField<String> _binding(
    String key,
    String Function(AppLocalizations l10n) label,
    String Function(AppLocalizations l10n) tooltip,
  ) {
    return ConfigField<String>(
      key: key,
      defaultValue: '',
      section: 'mouse.bindings',
      label: label,
      tooltip: tooltip,
    );
  }

  static final standardMoveForward = _binding(
    'standard_move_forward',
    (l) => l.labelBindMoveForward,
    (l) => l.tooltipBindMoveForward,
  );

  static final standardMoveBackward = _binding(
    'standard_move_backward',
    (l) => l.labelBindMoveBackward,
    (l) => l.tooltipBindMoveBackward,
  );

  static final standardMoveLeft = _binding(
    'standard_move_left',
    (l) => l.labelBindMoveLeft,
    (l) => l.tooltipBindMoveLeft,
  );

  static final standardMoveRight = _binding(
    'standard_move_right',
    (l) => l.labelBindMoveRight,
    (l) => l.tooltipBindMoveRight,
  );

  static final standardJump = _binding(
    'standard_jump',
    (l) => l.labelBindJump,
    (l) => l.tooltipBindJump,
  );

  static final standardWalk = _binding(
    'standard_walk',
    (l) => l.labelBindWalk,
    (l) => l.tooltipBindWalk,
  );

  static final standardAutoRun = _binding(
    'standard_auto_run',
    (l) => l.labelBindAutoRun,
    (l) => l.tooltipBindAutoRun,
  );

  static final standardLightAttack = _binding(
    'standard_light_attack',
    (l) => l.labelBindLightAttack,
    (l) => l.tooltipBindLightAttack,
  );

  static final standardHeavyAttack = _binding(
    'standard_heavy_attack',
    (l) => l.labelBindHeavyAttack,
    (l) => l.tooltipBindHeavyAttack,
  );

  static final standardFire = _binding(
    'standard_fire',
    (l) => l.labelBindFire,
    (l) => l.tooltipBindFire,
  );

  static final standardProgram = _binding(
    'standard_program',
    (l) => l.labelBindProgram,
    (l) => l.tooltipBindProgram,
  );

  static final standardLockOn = _binding(
    'standard_lock_on',
    (l) => l.labelBindLockOn,
    (l) => l.tooltipBindLockOn,
  );

  static final standardUse = _binding(
    'standard_use',
    (l) => l.labelBindUse,
    (l) => l.tooltipBindUse,
  );

  static final standardSelfDestruct = _binding(
    'standard_self_destruct',
    (l) => l.labelBindSelfDestruct,
    (l) => l.tooltipBindSelfDestruct,
  );

  static final standardLight = _binding(
    'standard_light',
    (l) => l.labelBindLight,
    (l) => l.tooltipBindLight,
  );

  static final standardResetCamera = _binding(
    'standard_reset_camera',
    (l) => l.labelBindResetCamera,
    (l) => l.tooltipBindResetCamera,
  );

  static final standardSwitchWeapon = _binding(
    'standard_switch_weapon',
    (l) => l.labelBindSwitchWeapon,
    (l) => l.tooltipBindSwitchWeapon,
  );

  static final standardNextProgram = _binding(
    'standard_next_program',
    (l) => l.labelBindNextProgram,
    (l) => l.tooltipBindNextProgram,
  );

  static final standardPreviousProgram = _binding(
    'standard_previous_program',
    (l) => l.labelBindPreviousProgram,
    (l) => l.tooltipBindPreviousProgram,
  );

  static final standardMenuUp = _binding(
    'standard_menu_up',
    (l) => l.labelBindMenuUp,
    (l) => l.tooltipBindMenuUp,
  );

  static final standardMenuDown = _binding(
    'standard_menu_down',
    (l) => l.labelBindMenuDown,
    (l) => l.tooltipBindMenuDown,
  );

  static final standardMenuLeft = _binding(
    'standard_menu_left',
    (l) => l.labelBindMenuLeft,
    (l) => l.tooltipBindMenuLeft,
  );

  static final standardMenuRight = _binding(
    'standard_menu_right',
    (l) => l.labelBindMenuRight,
    (l) => l.tooltipBindMenuRight,
  );

  static final standardMenuOpen = _binding(
    'standard_menu_open',
    (l) => l.labelBindMenuOpen,
    (l) => l.tooltipBindMenuOpen,
  );

  static final standardMenuBack = _binding(
    'standard_menu_back',
    (l) => l.labelBindMenuBack,
    (l) => l.tooltipBindMenuBack,
  );

  static final standardMenuEnter = _binding(
    'standard_menu_enter',
    (l) => l.labelBindMenuEnter,
    (l) => l.tooltipBindMenuEnter,
  );

  static final standardShortcutMenu = _binding(
    'standard_shortcut_menu',
    (l) => l.labelBindShortcutMenu,
    (l) => l.tooltipBindShortcutMenu,
  );

  static final nonStandardEvade = _binding(
    'non_standard_evade',
    (l) => l.labelBindEvade,
    (l) => l.tooltipBindEvade,
  );

  static final nonStandardAutoFire = _binding(
    'non_standard_auto_fire',
    (l) => l.labelBindAutoFire,
    (l) => l.tooltipBindAutoFire,
  );

  static final nonStandardNextItem = _binding(
    'non_standard_next_item',
    (l) => l.labelBindNextItem,
    (l) => l.tooltipBindNextItem,
  );

  static final nonStandardPreviousItem = _binding(
    'non_standard_previous_item',
    (l) => l.labelBindPreviousItem,
    (l) => l.tooltipBindPreviousItem,
  );

  static final nonStandardUseItem = _binding(
    'non_standard_use_item',
    (l) => l.labelBindUseItem,
    (l) => l.tooltipBindUseItem,
  );

  static final thirdPersonModeToggle = _binding(
    'third_person_mode_toggle',
    (l) => l.labelBindThirdPersonToggle,
    (l) => l.tooltipBindThirdPersonToggle,
  );

  static final aimModeToggle = _binding(
    'aim_mode_toggle',
    (l) => l.labelBindAimToggle,
    (l) => l.tooltipBindAimToggle,
  );

  static final List<ConfigField<String>> all = [
    standardMoveForward,
    standardMoveBackward,
    standardMoveLeft,
    standardMoveRight,
    standardJump,
    standardWalk,
    standardAutoRun,
    standardLightAttack,
    standardHeavyAttack,
    standardFire,
    standardProgram,
    standardLockOn,
    standardUse,
    standardSelfDestruct,
    standardLight,
    standardResetCamera,
    standardSwitchWeapon,
    standardNextProgram,
    standardPreviousProgram,
    standardMenuUp,
    standardMenuDown,
    standardMenuLeft,
    standardMenuRight,
    standardMenuOpen,
    standardMenuBack,
    standardMenuEnter,
    standardShortcutMenu,
    nonStandardEvade,
    nonStandardAutoFire,
    nonStandardNextItem,
    nonStandardPreviousItem,
    nonStandardUseItem,
    thirdPersonModeToggle,
    aimModeToggle,
  ];
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
    allowedValues: const [0.0, 0.75, 1.0, 10.0],
  );

  static final layoutSpawnRange = ConfigField<double>(
    key: 'layout_spawn_range',
    defaultValue: 100.0,
    label: (l) => l.labelLayoutSpawnRange,
    tooltip: (l) => l.tooltipLayoutSpawnRange,
    min: 25.0,
    max: 400.0,
    step: 25.0,
  );

  static final layoutSpawnEverything = ConfigField<bool>(
    key: 'layout_spawn_everything',
    defaultValue: false,
    label: (l) => l.labelLayoutSpawnEverything,
    tooltip: (l) => l.tooltipLayoutSpawnEverything,
  );

  static final disableCameraCulling = ConfigField<bool>(
    key: 'disable_camera_culling',
    defaultValue: false,
    label: (l) => l.labelDisableCameraCulling,
    tooltip: (l) => l.tooltipDisableCameraCulling,
  );

  static const bloom2017ReferenceHeight = 900;

  static const bloom2017Preset = <String, dynamic>{
    'bloom_reference_height': bloom2017ReferenceHeight,
  };

  static final disableManualCulling = ConfigField<bool>(
    key: 'disable_manual_culling',
    defaultValue: false,
    label: (l) => l.labelDisableManualCulling,
    tooltip: (l) => l.tooltipDisableManualCulling,
  );

  static final aoMultiplierWidth = ConfigField<double>(
    key: 'ao_multiplier_width',
    defaultValue: 1.0,
    label: (l) => l.labelAoWidth,
    tooltip: (l) => l.tooltipAoWidth,
    min: 0.1,
    max: 2.0,
    step: 0.05,
  );

  static final aoMultiplierHeight = ConfigField<double>(
    key: 'ao_multiplier_height',
    defaultValue: 1.0,
    label: (l) => l.labelAoHeight,
    tooltip: (l) => l.tooltipAoHeight,
    min: 0.1,
    max: 2.0,
    step: 0.05,
  );

  static final disableVignette = ConfigField<bool>(
    key: 'disable_vignette',
    defaultValue: false,
    label: (l) => l.labelDisableVignette,
    tooltip: (l) => l.tooltipDisableVignette,
  );

  static final bloomReferenceHeight = ConfigField<int>(
    key: 'bloom_reference_height',
    defaultValue: 0,
    label: (l) => l.labelBloomReferenceHeight,
    tooltip: (l) => l.tooltipBloomReferenceHeight,
    min: 0,
    max: 4320,
    step: 20,
  );

  static final bloomKernelReferenceHeight = ConfigField<int>(
    key: 'bloom_kernel_reference_height',
    defaultValue: 0,
    label: (l) => l.labelBloomKernelReferenceHeight,
    tooltip: (l) => l.tooltipBloomKernelReferenceHeight,
    min: 0,
    max: 4320,
    step: 20,
  );

  static const bloomReferenceMinimum = 360;

  static final bloomExtraBlur = ConfigField<double>(
    key: 'bloom_extra_blur',
    defaultValue: 0.0,
    label: (l) => l.labelBloomExtraBlur,
    tooltip: (l) => l.tooltipBloomExtraBlur,
    min: 0.0,
    max: 4.0,
    step: 0.1,
  );

  static final bloomDropCoarseLevels = ConfigField<int>(
    key: 'bloom_drop_coarse_levels',
    defaultValue: 0,
    label: (l) => l.labelBloomDropCoarseLevels,
    tooltip: (l) => l.tooltipBloomDropCoarseLevels,
    allowedValues: const [0, 1, 2],
  );

  static final highGridsEnabled = ConfigField<bool>(
    key: 'enabled',
    defaultValue: false,
    section: 'high_grids',
    label: (l) => l.labelHighGridsEnabled,
    tooltip: (l) => l.tooltipHighGridsEnabled,
    restartRequired: true,
  );

  static final highGridsRings = ConfigField<int>(
    key: 'rings',
    defaultValue: 1,
    section: 'high_grids',
    label: (l) => l.labelHighGridsRings,
    tooltip: (l) => l.tooltipHighGridsRings,
    allowedValues: const [1, 2, 3, 4],
    restartRequired: true,
  );

  static final highGridsRoomRings = ConfigField<List<dynamic>>(
    key: 'room_rings',
    defaultValue: const [],
    section: 'high_grids',
    label: (l) => l.labelHighGridsRoomRings,
    tooltip: (l) => l.tooltipHighGridsRoomRings,
    restartRequired: true,
  );

  static final highGridsBlockedInRoom = ConfigField<List<dynamic>>(
    key: 'blocked_in_room',
    defaultValue: const [],
    section: 'high_grids',
    label: (l) => l.labelHighGridsBlockedInRoom,
    tooltip: (l) => l.tooltipHighGridsBlockedInRoom,
    restartRequired: true,
  );

  static final highGridsBlockedFromGrid = ConfigField<List<dynamic>>(
    key: 'blocked_from_grid',
    defaultValue: const [],
    section: 'high_grids',
    label: (l) => l.labelHighGridsBlockedFromGrid,
    tooltip: (l) => l.tooltipHighGridsBlockedFromGrid,
    restartRequired: true,
  );

  static const highGridsCellCounts = <int, int>{1: 7, 2: 19, 3: 37, 4: 61};

  static const highGridsRooms = <int, String>{
    0x100: 'Ruined City',
    0x110: 'Resistance Camp',
    0x120: 'Factory',
    0x130: 'Amusement Park',
    0x140: "Pascal's Village",
    0x150: 'Desert',
    0x160: 'Forest',
    0x170: 'Flooded City',
    0x200: 'City Ruins (Route B)',
    0x520: 'Underground',
  };

  static final shadowResolution = ConfigField<int>(
    key: 'shadow_resolution',
    defaultValue: 2048,
    label: (l) => l.labelShadowResolution,
    tooltip: (l) => l.tooltipShadowResolution,
    allowedValues: const [512, 1024, 2048, 4096, 8192, 16384],
  );

  static final shadowDistanceMultiplier = ConfigField<double>(
    key: 'shadow_distance_multiplier',
    defaultValue: 1.0,
    label: (l) => l.labelDistanceMultiplier,
    tooltip: (l) => l.tooltipDistanceMultiplier,
    min: 1.0,
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
  );

  static final shadowModelForceAll = ConfigField<bool>(
    key: 'shadow_model_force_all',
    defaultValue: false,
    label: (l) => l.labelForceAllShadowModels,
    tooltip: (l) => l.tooltipForceAllShadowModels,
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

  static final shadowCascades = ConfigField<int>(
    key: 'shadow_cascades',
    defaultValue: 4,
    label: (l) => l.labelShadowCascades,
    tooltip: (l) => l.tooltipShadowCascades,
    allowedValues: const [4, 8],
    restartRequired: true,
  );

  static final shadowCascadeRange = ConfigField<double>(
    key: 'shadow_cascade_range',
    defaultValue: 4.0,
    label: (l) => l.labelShadowCascadeRange,
    tooltip: (l) => l.tooltipShadowCascadeRange,
    min: 1.05,
    max: 8.0,
    step: 0.05,
    restartRequired: true,
  );

  static final shadowBlurScale = ConfigField<List<dynamic>>(
    key: 'shadow_blur_scale',
    defaultValue: const [1.0, 1.0, 1.0, 1.0],
    label: (l) => l.labelShadowBlurScale,
    tooltip: (l) => l.tooltipShadowBlurScale,
    min: 0.0,
    max: 2.0,
    step: 0.05,
  );

  static final disableHdr = ConfigField<bool>(
    key: 'disable_hdr',
    defaultValue: false,
    label: (l) => l.labelDisableHdr,
    tooltip: (l) => l.tooltipDisableHdr,
  );

  static final renderScale = ConfigField<double>(
    key: 'render_scale',
    defaultValue: 1.0,
    label: (l) => l.labelRenderScale,
    tooltip: (l) => l.tooltipRenderScale,
    min: 0.5,
    max: 4.0,
    step: 0.25,
  );

  static final fxaa = ConfigField<bool>(
    key: 'fxaa',
    defaultValue: false,
    label: (l) => l.labelFxaa,
    tooltip: (l) => l.tooltipFxaa,
  );

  static final msaaPrepassFix = ConfigField<bool>(
    key: 'msaa_prepass_fix',
    defaultValue: true,
    label: (l) => l.labelMsaaPrepassFix,
    tooltip: (l) => l.tooltipMsaaPrepassFix,
  );

  static final msaaShadowMaskFix = ConfigField<bool>(
    key: 'msaa_shadow_mask_fix',
    defaultValue: true,
    label: (l) => l.labelMsaaShadowMaskFix,
    tooltip: (l) => l.tooltipMsaaShadowMaskFix,
  );

  static final msaaPerPixel = ConfigField<bool>(
    key: 'msaa_per_pixel',
    defaultValue: false,
    label: (l) => l.labelMsaaPerPixel,
    tooltip: (l) => l.tooltipMsaaPerPixel,
  );

  static final msaaPerPixelTerrain = ConfigField<bool>(
    key: 'msaa_per_pixel_terrain',
    defaultValue: false,
    label: (l) => l.labelMsaaPerPixelTerrain,
    tooltip: (l) => l.tooltipMsaaPerPixelTerrain,
  );

  static final constantBufferDedup = ConfigField<bool>(
    key: 'constant_buffer_dedup',
    defaultValue: false,
    label: (l) => l.labelConstantBufferDedup,
    tooltip: (l) => l.tooltipConstantBufferDedup,
  );

  static final constantBufferUploadOnBind = ConfigField<bool>(
    key: 'constant_buffer_upload_on_bind',
    defaultValue: false,
    label: (l) => l.labelConstantBufferUploadOnBind,
    tooltip: (l) => l.tooltipConstantBufferUploadOnBind,
  );

  static final decreaseStutterDuringGridLoading = ConfigField<bool>(
    key: 'decrease_stutter_during_grid_loading',
    defaultValue: false,
    label: (l) => l.labelDecreaseStutterDuringGridLoading,
    tooltip: (l) => l.tooltipDecreaseStutterDuringGridLoading,
  );

  static final aoFadeFix = ConfigField<bool>(
    key: 'ao_fade_fix',
    defaultValue: true,
    label: (l) => l.labelAoFadeFix,
    tooltip: (l) => l.tooltipAoFadeFix,
  );

  static final highGridsHiddenMeshes = ConfigField<List<dynamic>>(
    key: 'hidden_meshes',
    defaultValue: const [],
    section: 'high_grids',
    label: (l) => l.labelHighGridsHiddenMeshes,
    tooltip: (l) => l.tooltipHighGridsHiddenMeshes,
    restartRequired: true,
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

  static final fpsCapInHacking = ConfigField<bool>(
    key: 'fps_cap_in_hacking',
    defaultValue: true,
    label: (l) => l.labelFpsCapInHacking,
    tooltip: (l) => l.tooltipFpsCapInHacking,
  );

  static final fpsCapInEvents = ConfigField<bool>(
    key: 'fps_cap_in_events',
    defaultValue: true,
    label: (l) => l.labelFpsCapInEvents,
    tooltip: (l) => l.tooltipFpsCapInEvents,
  );

  static final fpsLimit = ConfigField<int>(
    key: 'fps_limit',
    defaultValue: 0,
    label: (l) => l.labelFpsLimit,
    tooltip: (l) => l.tooltipFpsLimit,
    min: 0,
    max: 1000,
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

  static final hotReload = ConfigField<bool>(
    key: 'hot_reload',
    defaultValue: true,
    label: (l) => l.labelTextureHotReload,
    tooltip: (l) => l.tooltipTextureHotReload,
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

  static final warpSave1 = ConfigField<String>(
    key: 'warp_save_1',
    defaultValue: 'Shift+C',
    section: 'yorha_protocol',
    label: (l) => l.labelWarpSave1,
    tooltip: (l) => l.tooltipWarpSave1,
  );

  static final warpGoto1 = ConfigField<String>(
    key: 'warp_goto_1',
    defaultValue: 'Z',
    section: 'yorha_protocol',
    label: (l) => l.labelWarpGoto1,
    tooltip: (l) => l.tooltipWarpGoto1,
  );

  static final warpSave2 = ConfigField<String>(
    key: 'warp_save_2',
    defaultValue: 'Shift+V',
    section: 'yorha_protocol',
    label: (l) => l.labelWarpSave2,
    tooltip: (l) => l.tooltipWarpSave2,
  );

  static final warpGoto2 = ConfigField<String>(
    key: 'warp_goto_2',
    defaultValue: 'X',
    section: 'yorha_protocol',
    label: (l) => l.labelWarpGoto2,
    tooltip: (l) => l.tooltipWarpGoto2,
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

  static final impeller = ConfigField<bool>(
    key: 'impeller',
    defaultValue: false,
    restartRequired: true,
    label: (l) => l.labelImpeller,
    tooltip: (l) => l.tooltipImpeller,
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
