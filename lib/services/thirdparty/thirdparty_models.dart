enum ThirdPartyRuntime { reshade, migoto, gameMods, graphicMods }

enum ThirdPartyKind {
  reshadeWholeInstall,
  reshadePreset,
  migoto,
  gameMod,
  graphicMod,
  lodmod,
  textures,
  gameData,
  specialK,
  skShaderEdit,
  brokenLegacy,
  multiBundle,
  unknown,
}

class ThirdPartyClassification {
  final ThirdPartyKind kind;
  final String sourceRoot;
  final String? sourceName;
  final List<String> innerArchives;
  final Map<String, dynamic>? lodmodValues;
  final int lodmodCount;
  final bool hasReShadeDll;
  final bool isAddonBuild;
  final bool hasShaders;
  final int presetCount;
  final List<String> gameModDlls;
  final List<String> graphicModDlls;

  const ThirdPartyClassification({
    required this.kind,
    required this.sourceRoot,
    this.sourceName,
    this.innerArchives = const [],
    this.lodmodValues,
    this.lodmodCount = 0,
    this.hasReShadeDll = false,
    this.isAddonBuild = false,
    this.hasShaders = false,
    this.presetCount = 0,
    this.gameModDlls = const [],
    this.graphicModDlls = const [],
  });

  ThirdPartyClassification withSourceName(String? name) =>
      ThirdPartyClassification(
        kind: kind,
        sourceRoot: sourceRoot,
        sourceName: name,
        innerArchives: innerArchives,
        lodmodValues: lodmodValues,
        lodmodCount: lodmodCount,
        hasReShadeDll: hasReShadeDll,
        isAddonBuild: isAddonBuild,
        hasShaders: hasShaders,
        presetCount: presetCount,
        gameModDlls: gameModDlls,
        graphicModDlls: graphicModDlls,
      );
}

class GameModSettings {
  static const steam2021SizeOfImage = 26177536;
  static const steam2026SizeOfImage = 25944064;
  static const defaultPatchTimeoutMs = 10000;

  final bool fakeModuleHandle;
  final bool fakeExePath;
  final int sizeOfImage;
  final bool disabled;
  final bool reportPatches;
  final bool waitForPatchPhase;
  final int patchPhaseTimeoutMs;

  const GameModSettings({
    this.fakeModuleHandle = true,
    this.fakeExePath = true,
    this.sizeOfImage = steam2021SizeOfImage,
    this.disabled = false,
    this.reportPatches = false,
    this.waitForPatchPhase = false,
    this.patchPhaseTimeoutMs = defaultPatchTimeoutMs,
  });

  factory GameModSettings.fromToml(Map<dynamic, dynamic> table) {
    const d = GameModSettings();
    bool flag(String key, bool fallback) =>
        table[key] is bool ? table[key] as bool : fallback;
    final size = table['size_of_image'];
    final timeout = table['patch_phase_timeout_ms'];
    return GameModSettings(
      fakeModuleHandle: flag('fake_module_handle', d.fakeModuleHandle),
      fakeExePath: flag('fake_exe_path', d.fakeExePath),
      sizeOfImage: size is int && size >= 0 ? size : d.sizeOfImage,
      disabled: flag('disabled', d.disabled),
      reportPatches: flag('report_patches', d.reportPatches),
      waitForPatchPhase: flag('wait_for_patch_phase', d.waitForPatchPhase),
      patchPhaseTimeoutMs: timeout is int && timeout >= 0
          ? timeout
          : d.patchPhaseTimeoutMs,
    );
  }

  Map<String, dynamic> toToml() => {
    'fake_module_handle': fakeModuleHandle,
    'fake_exe_path': fakeExePath,
    'size_of_image': sizeOfImage,
    'disabled': disabled,
    'report_patches': reportPatches,
    'wait_for_patch_phase': waitForPatchPhase,
    'patch_phase_timeout_ms': patchPhaseTimeoutMs,
  };

  GameModSettings copyWith({
    bool? fakeModuleHandle,
    bool? fakeExePath,
    int? sizeOfImage,
    bool? disabled,
    bool? reportPatches,
    bool? waitForPatchPhase,
    int? patchPhaseTimeoutMs,
  }) => GameModSettings(
    fakeModuleHandle: fakeModuleHandle ?? this.fakeModuleHandle,
    fakeExePath: fakeExePath ?? this.fakeExePath,
    sizeOfImage: sizeOfImage ?? this.sizeOfImage,
    disabled: disabled ?? this.disabled,
    reportPatches: reportPatches ?? this.reportPatches,
    waitForPatchPhase: waitForPatchPhase ?? this.waitForPatchPhase,
    patchPhaseTimeoutMs: patchPhaseTimeoutMs ?? this.patchPhaseTimeoutMs,
  );
}

class GameModEntry {
  final String fileName;
  final String? sizeLabel;
  final bool disabled;
  final GameModSettings? settings;

  const GameModEntry({
    required this.fileName,
    this.sizeLabel,
    this.disabled = false,
    this.settings,
  });
}

class GameModsInfo {
  final List<GameModEntry> mods;
  final bool hasConfig;

  const GameModsInfo({this.mods = const [], this.hasConfig = false});

  int get enabledCount => mods.where((m) => !m.disabled).length;
}

class ReShadeInfo {
  final List<String> presets;
  final List<String> shaderRepos;
  final int shaderCount;
  final List<String> addons;
  final bool isAddonBuild;
  final String? version;
  final String? dllName;
  final bool d3dCompilerMissing;
  final ReShadeConfig config;

  const ReShadeInfo({
    this.presets = const [],
    this.shaderRepos = const [],
    this.shaderCount = 0,
    this.addons = const [],
    this.isAddonBuild = false,
    this.version,
    this.dllName,
    this.d3dCompilerMissing = false,
    this.config = const ReShadeConfig(),
  });
}

class ReShadeConfig {
  final bool performanceMode;
  final bool skipLoadingCheck;
  final String? activePreset;
  final String overlayKey;
  final String effectToggleKey;
  final String screenshotKey;
  final String? screenshotPath;
  final int screenshotFormat;
  final bool showFps;
  final bool showClock;

  const ReShadeConfig({
    this.performanceMode = false,
    this.skipLoadingCheck = false,
    this.activePreset,
    this.overlayKey = '',
    this.effectToggleKey = '',
    this.screenshotKey = '',
    this.screenshotPath,
    this.screenshotFormat = 1,
    this.showFps = false,
    this.showClock = false,
  });

  ReShadeConfig copyWith({
    bool? performanceMode,
    bool? skipLoadingCheck,
    String? activePreset,
    String? overlayKey,
    String? effectToggleKey,
    String? screenshotKey,
    String? screenshotPath,
    int? screenshotFormat,
    bool? showFps,
    bool? showClock,
  }) {
    return ReShadeConfig(
      performanceMode: performanceMode ?? this.performanceMode,
      skipLoadingCheck: skipLoadingCheck ?? this.skipLoadingCheck,
      activePreset: activePreset ?? this.activePreset,
      overlayKey: overlayKey ?? this.overlayKey,
      effectToggleKey: effectToggleKey ?? this.effectToggleKey,
      screenshotKey: screenshotKey ?? this.screenshotKey,
      screenshotPath: screenshotPath ?? this.screenshotPath,
      screenshotFormat: screenshotFormat ?? this.screenshotFormat,
      showFps: showFps ?? this.showFps,
      showClock: showClock ?? this.showClock,
    );
  }
}

class MigotoInfo {
  final List<String> files;
  final int shaderFixCount;
  final List<String> shaderFixNames;
  final String? loaderTarget;
  final bool loaderTargetOk;
  final MigotoConfig config;

  const MigotoInfo({
    this.files = const [],
    this.shaderFixCount = 0,
    this.shaderFixNames = const [],
    this.loaderTarget,
    this.loaderTargetOk = false,
    this.config = const MigotoConfig(),
  });
}

enum MigotoHunting { off, on, noMarking }

enum MigotoMarking { skip, original, pink, mono }

class MigotoConfig {
  final MigotoHunting hunting;
  final MigotoMarking markingMode;
  final bool verboseOverlay;
  final bool cacheShaders;
  final bool checkForegroundWindow;
  final String reloadFixesKey;
  final String wipeCacheKey;
  final String toggleHuntKey;

  const MigotoConfig({
    this.hunting = MigotoHunting.off,
    this.markingMode = MigotoMarking.skip,
    this.verboseOverlay = false,
    this.cacheShaders = true,
    this.checkForegroundWindow = false,
    this.reloadFixesKey = '',
    this.wipeCacheKey = '',
    this.toggleHuntKey = '',
  });

  MigotoConfig copyWith({
    MigotoHunting? hunting,
    MigotoMarking? markingMode,
    bool? verboseOverlay,
    bool? cacheShaders,
    bool? checkForegroundWindow,
    String? reloadFixesKey,
    String? wipeCacheKey,
    String? toggleHuntKey,
  }) {
    return MigotoConfig(
      hunting: hunting ?? this.hunting,
      markingMode: markingMode ?? this.markingMode,
      verboseOverlay: verboseOverlay ?? this.verboseOverlay,
      cacheShaders: cacheShaders ?? this.cacheShaders,
      checkForegroundWindow:
          checkForegroundWindow ?? this.checkForegroundWindow,
      reloadFixesKey: reloadFixesKey ?? this.reloadFixesKey,
      wipeCacheKey: wipeCacheKey ?? this.wipeCacheKey,
      toggleHuntKey: toggleHuntKey ?? this.toggleHuntKey,
    );
  }
}

class ThirdPartyRuntimeStatus {
  final bool installed;
  final bool enabled;
  final bool shadersMissing;
  final int presetCount;
  final bool hasShaderFixes;
  final ReShadeInfo? reshadeInfo;
  final MigotoInfo? migotoInfo;
  final GameModsInfo? gameModsInfo;
  final GameModsInfo? graphicModsInfo;

  const ThirdPartyRuntimeStatus({
    this.installed = false,
    this.enabled = false,
    this.shadersMissing = false,
    this.presetCount = 0,
    this.hasShaderFixes = false,
    this.reshadeInfo,
    this.migotoInfo,
    this.gameModsInfo,
    this.graphicModsInfo,
  });
}

class ThirdPartyUpdateInfo {
  final ThirdPartyRuntime runtime;
  final String? installedLabel;
  final String? incomingLabel;

  const ThirdPartyUpdateInfo({
    required this.runtime,
    this.installedLabel,
    this.incomingLabel,
  });
}

class ThirdPartyInstallResult {
  final bool ok;
  final ThirdPartyRuntime? runtime;
  final bool shadersMissing;
  final int presetCount;
  final String? errorKey;

  const ThirdPartyInstallResult({
    required this.ok,
    this.runtime,
    this.shadersMissing = false,
    this.presetCount = 0,
    this.errorKey,
  });
}
