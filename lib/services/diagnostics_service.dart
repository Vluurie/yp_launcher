import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/cutscene_detection_service.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/diagnostics_probes.dart';
import 'package:yp_launcher/services/disabled_mods_service.dart';
import 'package:yp_launcher/services/gpu_preference_service.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/launch_record_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/nams_cli_service.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/thirdparty/game_mods_runtime.dart';
import 'package:yp_launcher/services/thirdparty/graphic_mods_runtime.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/services/wine/wine_runtime.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_isolates.dart';

enum FindingLevel { error, warning, info }

class DiagnosticFinding {
  final FindingLevel level;
  final String message;

  const DiagnosticFinding(this.level, this.message);

  String get tag => switch (level) {
    FindingLevel.error => '[X]',
    FindingLevel.warning => '[!]',
    FindingLevel.info => '[i]',
  };
}

class GameIdentity {
  final String exeVariant;
  final bool hasDlc;
  final String dlcSource;
  final int? exeSize;
  final String? exeSha1;
  final int? exeTimeDateStamp;
  final ExeBuildStatus buildStatus;
  final String? buildLabel;
  final String? stampBuildLabel;
  final SteamAppManifest? steamManifest;
  final Map<String, String> buildInfo;
  final bool compatExePresent;
  final int? data100Size;
  final FileFacts? gameBin;
  final int? gameBinTimeDateStamp;

  const GameIdentity({
    this.exeVariant = 'unknown',
    this.hasDlc = false,
    this.dlcSource = 'size check',
    this.exeSize,
    this.exeSha1,
    this.exeTimeDateStamp,
    this.buildStatus = ExeBuildStatus.missing,
    this.buildLabel,
    this.stampBuildLabel,
    this.steamManifest,
    this.buildInfo = const {},
    this.compatExePresent = false,
    this.data100Size,
    this.gameBin,
    this.gameBinTimeDateStamp,
  });

  bool get exeVariantSupported => buildStatus == ExeBuildStatus.known;

  bool get gameBinMatchesExe =>
      gameBinTimeDateStamp != null && gameBinTimeDateStamp == exeTimeDateStamp;

  String? get gameBinBuildLabel =>
      GameDetection.buildOfTimeDateStamp(gameBinTimeDateStamp)?.label;

  String get buildDisplay => switch (buildStatus) {
    ExeBuildStatus.known => buildLabel ?? '?',
    ExeBuildStatus.missing => 'missing',
    ExeBuildStatus.legacyWindows7 => 'Windows 7/8 legacy build',
    ExeBuildStatus.modified =>
      'modified copy of ${stampBuildLabel ?? '?'}'
          '${exeVariant == ExeVariant.wolfLimitBreak.name ? ' (Wolf Limit Break)' : ''}',
    ExeBuildStatus.newer => 'newer than every build NAMS knows',
    ExeBuildStatus.unknown => 'unknown build',
  };
}

class VanillaDirFinding {
  final String path;
  final int sizeBytes;
  final String bucket;

  const VanillaDirFinding({
    required this.path,
    required this.sizeBytes,
    required this.bucket,
  });
}

class ThirdPartyReport {
  final ThirdPartyRuntimeStatus reshade;
  final ThirdPartyRuntimeStatus migoto;
  final ThirdPartyRuntimeStatus gameMods;
  final ThirdPartyRuntimeStatus graphicMods;

  const ThirdPartyReport({
    this.reshade = const ThirdPartyRuntimeStatus(),
    this.migoto = const ThirdPartyRuntimeStatus(),
    this.gameMods = const ThirdPartyRuntimeStatus(),
    this.graphicMods = const ThirdPartyRuntimeStatus(),
  });
}

class NamsHealth {
  final List<String> missingFiles;
  final bool namsExePresent;
  final String? launchCommandPreview;
  final FileFacts? namsExe;
  final List<FileFacts> plugins;
  final String? namsVersion;
  final String? nierInstallError;
  final bool? steamRunning;

  const NamsHealth({
    this.missingFiles = const [],
    this.namsExePresent = false,
    this.launchCommandPreview,
    this.namsExe,
    this.plugins = const [],
    this.namsVersion,
    this.nierInstallError,
    this.steamRunning,
  });
}

class CompatLayerReport {
  final String source;
  final String label;
  final String binary;
  final String? prefix;
  final bool prefixExists;
  final String? toolVersion;
  final String? prefixVersion;

  const CompatLayerReport({
    required this.source,
    required this.label,
    required this.binary,
    this.prefix,
    this.prefixExists = false,
    this.toolVersion,
    this.prefixVersion,
  });
}

class LogsReport {
  final TextExcerpt? namsLog;
  final int crashDumpCount;
  final TextExcerpt? newestCrashDump;
  final TextExcerpt? heartbeat;
  final List<FileFacts> capturedLaunchLogs;

  const LogsReport({
    this.namsLog,
    this.crashDumpCount = 0,
    this.newestCrashDump,
    this.heartbeat,
    this.capturedLaunchLogs = const [],
  });
}

class ConfigDelta {
  final String file;
  final String key;
  final String value;
  final String defaultValue;
  final String? note;

  const ConfigDelta({
    required this.file,
    required this.key,
    required this.value,
    required this.defaultValue,
    this.note,
  });
}

class ConfigReport {
  final List<ConfigDelta> deltas;
  final List<ConfigDelta> unknownKeys;
  final List<ConfigDelta> legacyKeys;
  final Map<String, String> parseErrors;
  final List<String> missingFiles;

  const ConfigReport({
    this.deltas = const [],
    this.unknownKeys = const [],
    this.legacyKeys = const [],
    this.parseErrors = const {},
    this.missingFiles = const [],
  });
}

const _legacyKeys = <({String file, String key, String note})>[
  (
    file: 'nams.toml',
    key: 'mouse.',
    note: 'moved to naiom.toml, NAMS ignores it once naiom.toml exists',
  ),
  (
    file: 'nams.toml',
    key: 'heap.',
    note: 'removed, the heaps grow on their own',
  ),
  (file: 'nams.toml', key: 'cutscene.hd_cutscenes', note: 'removed'),
  (
    file: 'nams.toml',
    key: 'content_',
    note: 'old content switch, only read while [content] is missing',
  ),
  (file: 'lodmod.toml', key: 'disable_fake_hdr', note: 'removed from NAMS'),
  (file: 'lodmod.toml', key: 'gi_min_light_extent', note: 'removed from NAMS'),
];

const _gameFileExtensions = <String>{
  '.dat',
  '.dtt',
  '.cpk',
  '.usm',
  '.wsp',
  '.wem',
  '.bnk',
  '.wtp',
  '.wta',
  '.wtb',
  '.wmb',
  '.bxm',
  '.mcd',
  '.tmd',
  '.smd',
  '.ktb',
  '.eff',
  '.evn',
  '.mot',
  '.dll',
  '.exe',
};

const _proxyDllNames = <String>{
  'd3d9.dll',
  'd3d10.dll',
  'd3d11.dll',
  'd3d12.dll',
  'dxgi.dll',
  'dinput8.dll',
  'xinput1_3.dll',
  'xinput1_4.dll',
  'xinput9_1_0.dll',
  'winmm.dll',
  'version.dll',
  'dsound.dll',
};

String? _legacyNote(String file, String key) {
  for (final l in _legacyKeys) {
    if (l.file != file) continue;
    final prefix = l.key.endsWith('.') || l.key.endsWith('_');
    if (prefix ? key.startsWith(l.key) : key == l.key) return l.note;
  }
  return null;
}

class DiagnosticsReport {
  final DateTime generatedAt;
  final String gameDir;
  final Map<String, String> systemInfo;
  final Map<String, String> launcherInfo;
  final List<InstalledMod> mods;
  final List<String> hiddenModFolders;
  final List<String> disabledModsEntries;
  final String? activeProfile;
  final List<String> defaultModsEntries;
  final List<CutsceneMod> cutsceneMods;
  final List<String> directOverrides;
  final CutsceneDetection cutsceneDetection;
  final List<String> namsTextures;
  final List<String> skResTextures;
  final List<String> waxTextures;
  final Map<String, int> dataDirContents;
  final List<String> waxModsRoot;
  final List<String> skResPacks;
  final List<String> reshadeContents;
  final List<String> gameRootExtras;
  final Map<String, String> configRaw;
  final GameIdentity gameIdentity;
  final List<VanillaDirFinding> vanillaDropped;
  final ThirdPartyReport thirdParty;
  final NamsHealth namsHealth;
  final List<NamsTexturePack> texturePacks;
  final bool texturePacksAvailable;
  final String? texturePacksNote;
  final List<LogEntry> recentLogIssues;
  final ConfigReport config;
  final bool gameRunning;
  final bool preferDedicatedGpu;
  final Map<String, MountFacts?> filesystems;
  final CompatLayerReport? compatLayer;
  final LaunchRecord? lastLaunch;
  final LogsReport logs;
  final Map<String, String> collectionErrors;
  final List<DiagnosticFinding> findings;

  const DiagnosticsReport({
    required this.generatedAt,
    required this.gameDir,
    required this.systemInfo,
    required this.launcherInfo,
    required this.mods,
    required this.disabledModsEntries,
    required this.cutsceneMods,
    required this.directOverrides,
    required this.cutsceneDetection,
    required this.namsTextures,
    required this.skResTextures,
    required this.waxTextures,
    required this.dataDirContents,
    required this.waxModsRoot,
    required this.skResPacks,
    required this.reshadeContents,
    required this.gameRootExtras,
    this.hiddenModFolders = const [],
    this.activeProfile,
    this.defaultModsEntries = const [],
    this.configRaw = const {},
    this.gameIdentity = const GameIdentity(),
    this.vanillaDropped = const [],
    this.thirdParty = const ThirdPartyReport(),
    this.namsHealth = const NamsHealth(),
    this.texturePacks = const [],
    this.texturePacksAvailable = false,
    this.texturePacksNote,
    this.recentLogIssues = const [],
    this.config = const ConfigReport(),
    this.gameRunning = false,
    this.preferDedicatedGpu = false,
    this.filesystems = const {},
    this.compatLayer,
    this.lastLaunch,
    this.logs = const LogsReport(),
    this.collectionErrors = const {},
    this.findings = const [],
  });

  List<ConfigDelta> get configDeltas => config.deltas;
}

const _configFiles = [
  'nams.toml',
  'naiom.toml',
  'lodmod.toml',
  'texture_injection.toml',
];

const _reportFormat = 2;
const _discordLimit = 1900;

class DiagnosticsService {
  DiagnosticsService._();

  @visibleForTesting
  static String? userNameForTest;

  static Future<DiagnosticsReport> collect(
    String gameDir, {
    String? launchCommandPreview,
  }) async {
    final errors = <String, String>{};
    Future<T> guard<T>(
      String section,
      T fallback,
      Future<T> Function() body,
    ) async {
      try {
        return await body();
      } catch (e) {
        errors[section] = _shortError(e);
        return fallback;
      }
    }

    final hasGame = gameDir.isNotEmpty;

    final mods = hasGame
        ? await guard(
            'mods',
            <InstalledMod>[],
            () => ModsService.listInstalled(gameDir),
          )
        : <InstalledMod>[];
    final hiddenModFolders = hasGame
        ? await guard(
            'hidden mod folders',
            <String>[],
            () async => _hiddenModFolders(gameDir),
          )
        : <String>[];
    final disabled = hasGame
        ? await guard(
            'disabled mods',
            <String>[],
            () => DisabledModsService.list(gameDir),
          )
        : <String>[];
    final cutsceneScan = hasGame
        ? await guard(
            'cutscene detection',
            const CutsceneDetection(),
            () => CutsceneDetectionService.scan(gameDir),
          )
        : const CutsceneDetection();
    final cutscenes = hasGame
        ? await guard(
            'cutscene mods',
            const ScanResult(mods: [], directOverrides: []),
            () => scanCutscenes(
              p.join(gameDir, 'nams', 'cutscenes'),
              p.join(gameDir, 'data', 'movie'),
              p.join(gameDir, 'nams', 'mods'),
            ),
          )
        : const ScanResult(mods: [], directOverrides: []);

    const emptyTextures = <String, dynamic>{
      'nams': <String>[],
      'sk': <String>[],
      'wax': <String>[],
    };
    final textureScan = hasGame
        ? await guard(
            'texture folders',
            emptyTextures,
            () => IsolateService.run<_TextureScanParams, Map<String, dynamic>>(
              _scanTexturesSync,
              _TextureScanParams(gameDir: gameDir),
            ),
          )
        : emptyTextures;

    const emptyExtras = _ExtraScan(
      dataDirContents: {},
      waxModsRoot: [],
      skResPacks: [],
      reshadeContents: [],
      gameRootExtras: [],
    );
    final extraScan = hasGame
        ? await guard(
            'game folder scan',
            emptyExtras,
            () => IsolateService.run(_scanExtrasSync, gameDir),
          )
        : emptyExtras;

    final configRaw = <String, String>{};
    if (hasGame) {
      for (final name in _configFiles) {
        configRaw[name] = await guard(
          name,
          '',
          () => TomlService.readTomlFile(p.join(gameDir, 'nams', name)),
        );
      }
    }
    final config = _collectConfig(configRaw, hasGame);

    final launcherDir = await guard(
      'launcher dir',
      '',
      LauncherSetupService.getLauncherDirectory,
    );

    final gameIdentity = await guard(
      'game identity',
      const GameIdentity(),
      () => _collectGameIdentity(gameDir, launcherDir),
    );
    final vanillaDropped = hasGame
        ? await guard(
            'vanilla data drops',
            <VanillaDirFinding>[],
            () => IsolateService.run(_scanVanillaSync, gameDir),
          )
        : <VanillaDirFinding>[];
    final thirdParty = await guard(
      'third-party',
      const ThirdPartyReport(),
      () => _collectThirdParty(gameDir),
    );
    final namsHealth = await guard(
      'NAMS health',
      NamsHealth(launchCommandPreview: launchCommandPreview),
      () => _collectNamsHealth(gameDir, launchCommandPreview),
    );

    List<NamsTexturePack>? texturePacks;
    String? texturePacksNote;
    if (!hasGame) {
      texturePacksNote = 'no game folder selected';
    } else if (!Platform.isWindows) {
      texturePacksNote =
          'NAMS texture query runs NAMS.exe natively (Windows only)';
    } else {
      texturePacks = await guard<List<NamsTexturePack>?>(
        'NAMS texture query',
        null,
        () => NamsCliService.texturesList(
          gameDir,
          timeout: const Duration(seconds: 20),
        ),
      );
      if (texturePacks == null) {
        texturePacksNote = namsHealth.namsExePresent
            ? 'NAMS texture query failed or timed out'
            : 'NAMS.exe missing';
      }
    }

    final recentLogIssues = await guard(
      'nams.log issues',
      const <LogEntry>[],
      _collectRecentIssues,
    );
    final gameRunning = await guard(
      'game running',
      false,
      () async => await PlatformAdapter.current.isGameRunning(),
    );
    final preferDedicatedGpu = await guard(
      'GPU preference',
      false,
      GpuPreferenceService.preferDedicatedGpu,
    );
    final gpu = GpuProbe.detect();
    final gpuEnv = preferDedicatedGpu
        ? GpuPreferenceService.linuxGpuEnv(gpu)
        : const <String, String>{};

    final compatLayer = hasGame && !Platform.isWindows
        ? await guard<CompatLayerReport?>(
            'compatibility layer',
            null,
            () async => _collectCompatLayer(gameDir),
          )
        : null;

    final filesystems = <String, MountFacts?>{};
    if (Platform.isLinux) {
      if (hasGame) {
        filesystems['game folder'] = await DiagnosticsProbes.mountOf(gameDir);
      }
      if (launcherDir.isNotEmpty) {
        filesystems['launcher folder'] = await DiagnosticsProbes.mountOf(
          launcherDir,
        );
      }
      final prefix = compatLayer?.prefix;
      if (prefix != null && Directory(prefix).existsSync()) {
        filesystems['wine prefix'] = await DiagnosticsProbes.mountOf(prefix);
      }
    }

    final lastLaunch = await guard<LaunchRecord?>(
      'last launch',
      null,
      LaunchRecordService.read,
    );
    final logs = await guard(
      'logs',
      const LogsReport(),
      () async => _collectLogs(),
    );

    final profile = hasGame
        ? await guard<({String? active, List<String> defaults})>('profiles', (
            active: null,
            defaults: const [],
          ), () async => _collectProfiles(gameDir))
        : (active: null, defaults: const <String>[]);

    final report = DiagnosticsReport(
      generatedAt: DateTime.now().toUtc(),
      gameDir: gameDir,
      systemInfo: {
        'OS': Platform.operatingSystem,
        'OS version': _osVersionLabel(),
        'Locale': Platform.localeName,
        'Dart version': Platform.version,
        'Game running': gameRunning ? 'yes' : 'no',
        'Steam running': switch (namsHealth.steamRunning) {
          true => 'yes',
          false => 'no',
          null => 'unknown',
        },
        'Prefer dedicated GPU': preferDedicatedGpu ? 'yes' : 'no',
        ...DiagnosticsProbes.linuxContext(),
        if (Platform.isLinux) ...{
          'NVIDIA driver': gpu.nvidiaDriverLoaded ? 'yes' : 'no',
          'DRM render nodes': '${gpu.renderNodes}',
          'Dedicated GPU env': gpuEnv.isEmpty
              ? 'none'
              : gpuEnv.entries.map((e) => '${e.key}=${e.value}').join(' '),
        },
      },
      launcherInfo: {
        'Launcher version': AppStrings.appVersion,
        'Launcher dir': launcherDir,
        'Game dir': gameDir.isEmpty ? '<not selected>' : gameDir,
      },
      mods: mods,
      hiddenModFolders: hiddenModFolders,
      disabledModsEntries: disabled,
      activeProfile: profile.active,
      defaultModsEntries: profile.defaults,
      cutsceneMods: cutscenes.mods,
      directOverrides: cutscenes.directOverrides,
      cutsceneDetection: cutsceneScan,
      namsTextures: List<String>.from(textureScan['nams'] as List? ?? []),
      skResTextures: List<String>.from(textureScan['sk'] as List? ?? []),
      waxTextures: List<String>.from(textureScan['wax'] as List? ?? []),
      dataDirContents: extraScan.dataDirContents,
      waxModsRoot: extraScan.waxModsRoot,
      skResPacks: extraScan.skResPacks,
      reshadeContents: extraScan.reshadeContents,
      gameRootExtras: extraScan.gameRootExtras,
      configRaw: configRaw,
      gameIdentity: gameIdentity,
      vanillaDropped: vanillaDropped,
      thirdParty: thirdParty,
      namsHealth: namsHealth,
      texturePacks: texturePacks ?? const [],
      texturePacksAvailable: texturePacks != null,
      texturePacksNote: texturePacksNote,
      recentLogIssues: recentLogIssues,
      config: config,
      gameRunning: gameRunning,
      preferDedicatedGpu: preferDedicatedGpu,
      filesystems: filesystems,
      compatLayer: compatLayer,
      lastLaunch: lastLaunch,
      logs: logs,
      collectionErrors: errors,
    );
    return _withFindings(report);
  }

  static String _shortError(Object e) {
    final text = '$e'.replaceFirst('Exception: ', '');
    final firstLine = text.split('\n').first;
    return firstLine.length > 200
        ? '${firstLine.substring(0, 200)}...'
        : firstLine;
  }

  static String _osVersionLabel() {
    final raw = Platform.operatingSystemVersion;
    if (!Platform.isWindows) return raw;
    final m = RegExp(r'(\d+)\.(\d+)\.(\d+)').firstMatch(raw);
    if (m == null) return raw;
    final build = int.tryParse(m.group(3)!) ?? 0;
    if (build >= 22000 && raw.contains('Windows 10')) {
      return raw.replaceFirst('Windows 10', 'Windows 11');
    }
    return raw;
  }

  static List<String> _hiddenModFolders(String gameDir) {
    final dir = Directory(ModsService.modsDir(gameDir));
    if (!dir.existsSync()) return const [];
    final out = <String>[];
    for (final e in dir.listSync(followLinks: false).whereType<Directory>()) {
      final name = p.basename(e.path);
      if (name.startsWith('_')) out.add(name);
    }
    out.sort();
    return out;
  }

  static Future<GameIdentity> _collectGameIdentity(
    String gameDir,
    String launcherDir,
  ) async {
    if (gameDir.isEmpty) return const GameIdentity();
    final variant = await GameDetection.detectExeVariant(gameDir);
    final probe = await GameDetection.probeExe(gameDir);
    final manifest = GameDetection.readSteamManifest(gameDir);
    final hasDlc = await GameDetection.hasDlc(gameDir);
    final data100 = File(p.join(gameDir, 'data', 'data100.cpk'));
    final buildInfoFile = File(p.join(gameDir, 'Latest_BuildInfo.properties'));
    final buildInfo = buildInfoFile.existsSync()
        ? DiagnosticsProbes.parseKeyValueLines(buildInfoFile.readAsStringSync())
        : const <String, String>{};
    final gameBinPath = launcherDir.isEmpty
        ? null
        : p.join(launcherDir, 'runtime', 'game.bin');
    return GameIdentity(
      exeVariant: variant.name,
      hasDlc: hasDlc,
      dlcSource: manifest != null ? 'Steam manifest' : 'size check',
      exeSize: probe.size,
      exeSha1: probe.sha1,
      exeTimeDateStamp: probe.timeDateStamp,
      buildStatus: probe.status,
      buildLabel: probe.build?.label,
      stampBuildLabel: probe.stampBuild?.label,
      steamManifest: manifest,
      buildInfo: buildInfo,
      compatExePresent: File(
        p.join(gameDir, 'NieRAutomataCompat.exe'),
      ).existsSync(),
      data100Size: data100.existsSync() ? data100.lengthSync() : null,
      gameBin: gameBinPath == null
          ? null
          : DiagnosticsProbes.facts(gameBinPath),
      gameBinTimeDateStamp: gameBinPath == null
          ? null
          : GameDetection.fileTimeDateStamp(gameBinPath),
    );
  }

  static Future<ThirdPartyReport> _collectThirdParty(String gameDir) async {
    if (gameDir.isEmpty) return const ThirdPartyReport();
    final reshade =
        await _quiet(() => const ReShadeRuntime().status(gameDir)) ??
        const ThirdPartyRuntimeStatus();
    final migoto =
        await _quiet(() => const MigotoRuntime().status(gameDir)) ??
        const ThirdPartyRuntimeStatus();
    final gameMods =
        await _quiet(() => const GameModsRuntime().status(gameDir)) ??
        const ThirdPartyRuntimeStatus();
    final graphic =
        await _quiet(() => const GraphicModsRuntime().status(gameDir)) ??
        const ThirdPartyRuntimeStatus();
    return ThirdPartyReport(
      reshade: reshade,
      migoto: migoto,
      gameMods: gameMods,
      graphicMods: graphic,
    );
  }

  static Future<T?> _quiet<T>(Future<T?> Function() f) async {
    try {
      return await f();
    } catch (_) {
      return null;
    }
  }

  static Future<NamsHealth> _collectNamsHealth(
    String gameDir,
    String? launchPreview,
  ) async {
    final missing = await LauncherSetupService.findMissingFiles();
    final paths = await LauncherSetupService.getLauncherPaths();
    final namsExe = paths['namsExe'];
    final files = await IsolateService.run(_runtimeFilesSync, (
      namsExe: namsExe ?? '',
      pluginsDir: paths['pluginsDir'] ?? '',
    ));
    NamsInfo? info;
    if (Platform.isWindows && gameDir.isNotEmpty && files.namsExe != null) {
      info = await NamsCliService.info(
        gameDir,
        timeout: const Duration(seconds: 10),
      );
    }
    return NamsHealth(
      missingFiles: missing,
      namsExePresent: files.namsExe != null,
      launchCommandPreview: launchPreview,
      namsExe: files.namsExe,
      plugins: files.plugins,
      namsVersion: info?.version,
      nierInstallError: info?.nierInstallError,
      steamRunning: await DiagnosticsProbes.isSteamRunning(),
    );
  }

  static CompatLayerReport? _collectCompatLayer(String gameDir) {
    final runtime = detectWineRuntime(
      gameExePath: p.join(gameDir, AppStrings.gameExeName),
    );
    if (runtime == null) return null;
    final prefix = runtime.prefix;
    String? toolVersion;
    String? prefixVersion;
    if (runtime.source == WineSource.proton) {
      toolVersion = DiagnosticsProbes.firstLine(
        p.join(p.dirname(runtime.binary), 'version'),
      );
      if (prefix != null && p.basename(prefix) == 'pfx') {
        prefixVersion = DiagnosticsProbes.firstLine(
          p.join(p.dirname(prefix), 'version'),
        );
      }
    }
    return CompatLayerReport(
      source: runtime.source.name,
      label: runtime.label,
      binary: runtime.binary,
      prefix: prefix,
      prefixExists: prefix != null && Directory(prefix).existsSync(),
      toolVersion: toolVersion,
      prefixVersion: prefixVersion,
    );
  }

  static LogsReport _collectLogs() {
    final dir = LogService.logsDirectory;
    final crashDumps = DiagnosticsProbes.newestFirst(
      dir,
      (n) => n.startsWith('crashdump_') && n.endsWith('.txt'),
    );
    final captured = DiagnosticsProbes.newestFirst(
      dir,
      (n) => n.startsWith('nams_') && n.endsWith('.log'),
    );
    return LogsReport(
      namsLog: DiagnosticsProbes.tail(p.join(dir, AppStrings.namsLogName)),
      crashDumpCount: crashDumps.length,
      newestCrashDump: crashDumps.isEmpty
          ? null
          : DiagnosticsProbes.head(crashDumps.first.path),
      heartbeat: DiagnosticsProbes.tail(
        p.join(dir, 'heartbeat.log'),
        maxLines: 20,
      ),
      capturedLaunchLogs: [
        for (final f in captured.take(3))
          if (DiagnosticsProbes.facts(f.path) case final facts?) facts,
      ],
    );
  }

  static ({String? active, List<String> defaults}) _collectProfiles(
    String gameDir,
  ) {
    String? active;
    final profiles = File(p.join(gameDir, 'nams', 'profiles.toml'));
    if (profiles.existsSync()) {
      final parsed = TomlService.parse(profiles.readAsStringSync());
      if (parsed['active'] is String) active = parsed['active'] as String;
    }
    final defaults = <String>[];
    final defaultMods = File(p.join(gameDir, 'nams', 'default_mods.toml'));
    if (defaultMods.existsSync()) {
      final parsed = TomlService.parse(defaultMods.readAsStringSync());
      for (final e in _flatten(parsed).entries) {
        defaults.add('${e.key} = ${_display(e.value)}');
      }
    }
    return (active: active, defaults: defaults);
  }

  static Future<List<LogEntry>> _collectRecentIssues() async {
    final entries = await LogService.readLog(AppStrings.namsLogName);
    final issues = entries
        .where((e) => e.level == 'WARN' || e.level == 'ERROR')
        .toList();
    return issues.length > 60 ? issues.sublist(issues.length - 60) : issues;
  }

  static ConfigReport _collectConfig(Map<String, String> raws, bool hasGame) {
    if (!hasGame) return const ConfigReport();
    final templates = NamsConfigService.shippedTemplates;
    final deltas = <ConfigDelta>[];
    final unknown = <ConfigDelta>[];
    final legacy = <ConfigDelta>[];
    final parseErrors = <String, String>{};
    final missing = <String>[];
    for (final file in _configFiles) {
      final raw = raws[file] ?? '';
      if (raw.trim().isEmpty) {
        missing.add(file);
        continue;
      }
      final parsed = TomlService.parseStrict(raw);
      if (!parsed.ok) {
        parseErrors[file] = parsed.error!;
        continue;
      }
      final defaults = _flatten(TomlService.parse(templates[file] ?? ''));
      for (final entry in _flatten(parsed.values).entries) {
        final hasDefault = defaults.containsKey(entry.key);
        final defaultValue = defaults[entry.key];
        if (hasDefault && _sameSetting(entry.value, defaultValue)) {
          continue;
        }
        final note = hasDefault ? null : _legacyNote(file, entry.key);
        final delta = ConfigDelta(
          file: file,
          key: entry.key,
          value: _display(entry.value),
          defaultValue: hasDefault ? _display(defaultValue) : '',
          note: note,
        );
        if (hasDefault) {
          deltas.add(delta);
        } else if (note != null) {
          legacy.add(delta);
        } else {
          unknown.add(delta);
        }
      }
    }
    return ConfigReport(
      deltas: deltas,
      unknownKeys: unknown,
      legacyKeys: legacy,
      parseErrors: parseErrors,
      missingFiles: missing,
    );
  }

  static bool _sameSetting(dynamic a, dynamic b) {
    if (a is num && b is num) {
      final scale = [
        1.0,
        a.abs().toDouble(),
        b.abs().toDouble(),
      ].reduce((x, y) => x > y ? x : y);
      return (a - b).abs() <= 1e-6 * scale;
    }
    return TomlService.sameValues(a, b);
  }

  static Map<String, dynamic> _flatten(Map<String, dynamic> map) {
    final out = <String, dynamic>{};
    void walk(String prefix, Map<dynamic, dynamic> node) {
      for (final e in node.entries) {
        final key = prefix.isEmpty ? '${e.key}' : '$prefix.${e.key}';
        if (e.value is Map) {
          walk(key, e.value as Map);
        } else {
          out[key] = e.value;
        }
      }
    }

    walk('', map);
    return out;
  }

  static String _display(dynamic value) {
    final text = _plain(value);
    return text.length > 160 ? '${text.substring(0, 160)}...' : text;
  }

  static String _plain(dynamic value) {
    if (value is String) return '"$value"';
    if (value is double && value.isFinite) {
      return double.parse(value.toStringAsPrecision(7)).toString();
    }
    if (value is List) return '[${value.map(_plain).join(', ')}]';
    if (value is Map) {
      return '{${value.entries.map((e) => '${e.key} = ${_plain(e.value)}').join(', ')}}';
    }
    return '$value';
  }

  static DiagnosticsReport _withFindings(DiagnosticsReport r) {
    return DiagnosticsReport(
      generatedAt: r.generatedAt,
      gameDir: r.gameDir,
      systemInfo: r.systemInfo,
      launcherInfo: r.launcherInfo,
      mods: r.mods,
      hiddenModFolders: r.hiddenModFolders,
      disabledModsEntries: r.disabledModsEntries,
      activeProfile: r.activeProfile,
      defaultModsEntries: r.defaultModsEntries,
      cutsceneMods: r.cutsceneMods,
      directOverrides: r.directOverrides,
      cutsceneDetection: r.cutsceneDetection,
      namsTextures: r.namsTextures,
      skResTextures: r.skResTextures,
      waxTextures: r.waxTextures,
      dataDirContents: r.dataDirContents,
      waxModsRoot: r.waxModsRoot,
      skResPacks: r.skResPacks,
      reshadeContents: r.reshadeContents,
      gameRootExtras: r.gameRootExtras,
      configRaw: r.configRaw,
      gameIdentity: r.gameIdentity,
      vanillaDropped: r.vanillaDropped,
      thirdParty: r.thirdParty,
      namsHealth: r.namsHealth,
      texturePacks: r.texturePacks,
      texturePacksAvailable: r.texturePacksAvailable,
      texturePacksNote: r.texturePacksNote,
      recentLogIssues: r.recentLogIssues,
      config: r.config,
      gameRunning: r.gameRunning,
      preferDedicatedGpu: r.preferDedicatedGpu,
      filesystems: r.filesystems,
      compatLayer: r.compatLayer,
      lastLaunch: r.lastLaunch,
      logs: r.logs,
      collectionErrors: r.collectionErrors,
      findings: buildFindings(r),
    );
  }

  static List<DiagnosticFinding> buildFindings(DiagnosticsReport r) {
    final out = <DiagnosticFinding>[];
    void add(FindingLevel level, String message) =>
        out.add(DiagnosticFinding(level, message));

    if (r.gameDir.isEmpty) {
      add(FindingLevel.warning, 'No game folder selected.');
    }

    final gi = r.gameIdentity;
    if (r.gameDir.isNotEmpty) {
      switch (gi.buildStatus) {
        case ExeBuildStatus.missing:
          add(
            FindingLevel.error,
            'NieRAutomata.exe not found in the game folder.',
          );
        case ExeBuildStatus.legacyWindows7:
          add(
            FindingLevel.error,
            'NieRAutomata.exe is the Windows 7/8 build. NAMS needs the Windows 10/11 '
            'Steam build: set Proton Experimental, delete the exe and verify game files.',
          );
        case ExeBuildStatus.modified:
          add(
            FindingLevel.error,
            'NieRAutomata.exe is a modified copy of ${gi.stampBuildLabel}'
            '${gi.exeVariant == ExeVariant.wolfLimitBreak.name ? ' (Wolf Limit Break)' : ''}. '
            'NAMS refuses it: verify game files in Steam.',
          );
        case ExeBuildStatus.newer:
          add(
            FindingLevel.error,
            'NieRAutomata.exe is a newer game update than this NAMS knows. '
            'Update the launcher.',
          );
        case ExeBuildStatus.unknown:
          add(
            FindingLevel.error,
            'NieRAutomata.exe is not a build NAMS knows '
            '(size ${gi.exeSize}, sha1 ${gi.exeSha1 ?? '?'}).',
          );
        case ExeBuildStatus.known:
          break;
      }
      final manifest = gi.steamManifest;
      if (manifest != null) {
        if (manifest.updatePending) {
          add(
            FindingLevel.warning,
            'Steam has a game update pending (build ${manifest.buildId} -> '
            '${manifest.targetBuildId}). Let Steam finish it.',
          );
        }
        if (manifest.betaKey != null) {
          add(
            FindingLevel.info,
            'Steam beta branch selected: ${manifest.betaKey}.',
          );
        }
        final steamBuild = _steamBuildIds[manifest.buildId];
        if (steamBuild != null &&
            gi.buildStatus == ExeBuildStatus.known &&
            gi.buildLabel != steamBuild) {
          add(
            FindingLevel.warning,
            'Steam reports build ${manifest.buildId} ($steamBuild) but '
            'NieRAutomata.exe is ${gi.buildLabel}: the exe was replaced by hand.',
          );
        }
      }
      if (gi.gameBin != null &&
          gi.exeTimeDateStamp != null &&
          !gi.gameBinMatchesExe) {
        add(
          FindingLevel.info,
          'runtime/game.bin was built from ${gi.gameBinBuildLabel ?? 'another exe'}; '
          'NAMS rebuilds it from the installed exe on the next launch.',
        );
      }
    }

    final health = r.namsHealth;
    if (health.missingFiles.isNotEmpty) {
      add(
        FindingLevel.error,
        'Launcher files missing: ${health.missingFiles.join(', ')} '
        '(antivirus quarantine?). Restore them or reinstall the launcher.',
      );
    }
    if (health.nierInstallError != null) {
      add(FindingLevel.warning, 'NAMS: ${health.nierInstallError}');
    }
    if (health.steamRunning == false) {
      add(
        FindingLevel.warning,
        'Steam is not running. NAMS needs the Steam client to start the game.',
      );
    }

    for (final e in r.config.parseErrors.entries) {
      add(FindingLevel.error, '${e.key} does not parse: ${e.value}');
    }
    if (r.config.unknownKeys.isNotEmpty) {
      final keys = r.config.unknownKeys
          .take(4)
          .map((d) => '${d.file}: ${d.key}')
          .join(', ');
      final more = r.config.unknownKeys.length > 4
          ? ' and ${r.config.unknownKeys.length - 4} more'
          : '';
      add(
        FindingLevel.info,
        'Unknown setting(s) NAMS does not read (typo?): $keys$more.',
      );
    }
    if (_deltaIs(r, 'nams.toml', NamsFields.disablePluginLoading.key, 'true')) {
      add(
        FindingLevel.warning,
        'Plugin loading is disabled (disable_plugin_loading): YoRHa Protocol and '
        'all plugins are off.',
      );
    }
    if (_deltaIs(
          r,
          'nams.toml',
          NamsFields.disableTextureInjection.key,
          'true',
        ) &&
        (r.namsTextures.isNotEmpty || r.texturePacks.isNotEmpty)) {
      add(
        FindingLevel.info,
        'Texture packs are installed but texture injection is disabled.',
      );
    }

    for (final e in r.filesystems.entries) {
      final fs = e.value;
      if (fs == null || !fs.risky) continue;
      add(
        e.key == 'game folder' ? FindingLevel.error : FindingLevel.warning,
        'The ${e.key} is on ${fs.fsType}. Under Proton, Play and Verify hang on '
        'NTFS/FAT drives: move it to a Linux filesystem (ext4, btrfs).',
      );
    }
    if (!Platform.isWindows && r.gameDir.isNotEmpty && r.compatLayer == null) {
      add(
        FindingLevel.error,
        'No Proton, Wine or CrossOver found to run NAMS.exe.',
      );
    }

    final last = r.lastLaunch;
    if (last != null && last.exitCode != null && last.exitCode != 0) {
      final secs = last.duration?.inSeconds;
      add(
        secs != null && secs < 60 ? FindingLevel.warning : FindingLevel.info,
        'Last launch ended with exit code ${last.exitCode}'
        '${secs != null ? ' after ${secs}s' : ''}.',
      );
    }
    final crash = r.logs.newestCrashDump;
    if (crash?.modified != null) {
      final since = last?.startedAt;
      final fromLastSession =
          since != null &&
          crash!.modified!.toUtc().isAfter(
            since.subtract(const Duration(minutes: 1)),
          );
      final recent = DateTime.now().difference(crash!.modified!).inDays < 7;
      if (fromLastSession || recent) {
        add(
          fromLastSession ? FindingLevel.warning : FindingLevel.info,
          'NAMS wrote a crash dump${fromLastSession ? ' in the last session' : ''}: '
          '${crash.name}.',
        );
      }
    }
    final heartbeat = r.logs.heartbeat;
    if (heartbeat?.modified != null &&
        last != null &&
        heartbeat!.modified!.toUtc().isAfter(last.startedAt)) {
      add(
        FindingLevel.warning,
        'The heartbeat watchdog reported a freeze in the last session (heartbeat.log).',
      );
    }

    final gameFiles = r.vanillaDropped
        .where(
          (f) =>
              _gameFileExtensions.contains(p.extension(f.path).toLowerCase()),
        )
        .toList();
    if (gameFiles.isNotEmpty) {
      add(
        FindingLevel.warning,
        '${gameFiles.length} game file(s) dropped loose into vanilla data/ '
        '(${_capList(gameFiles.map((f) => p.basename(f.path)).toList(), 4)}). '
        'NAMS mods belong in nams/mods.',
      );
    }

    final proxies = r.gameRootExtras
        .where((n) => _proxyDllNames.contains(n.toLowerCase()))
        .toList();
    final specialK =
        r.gameRootExtras.any((n) => n.toLowerCase() == 'sk_res/') &&
        proxies.any((n) => n.toLowerCase() == 'dinput8.dll');
    final others = specialK
        ? proxies.where((n) => n.toLowerCase() != 'dinput8.dll').toList()
        : proxies;
    if (specialK) {
      add(
        FindingLevel.info,
        'Special K (dinput8.dll in the game folder) does not run under NAMS. '
        'NAMS reads SK_Res texture packs itself.',
      );
    }
    if (others.isNotEmpty) {
      add(
        FindingLevel.warning,
        '${others.join(', ')} in the game folder ${others.length == 1 ? 'is' : 'are'} '
        'never loaded: NAMS runs the game from its own folder. Install DLL mods '
        'through Third Party, which puts them in thirdparty/graphic or thirdparty/game.',
      );
    }

    final gameDlls = r.thirdParty.gameMods.gameModsInfo?.mods ?? const [];
    final activeGameDlls = gameDlls.where((m) => !m.disabled).toList();
    if (r.thirdParty.gameMods.enabled &&
        activeGameDlls.isNotEmpty &&
        r.gameIdentity.buildLabel == 'Steam 2026-11') {
      final old = activeGameDlls.where(
        (m) =>
            (m.settings?.sizeOfImage ?? GameModSettings.steam2021SizeOfImage) ==
            GameModSettings.steam2021SizeOfImage,
      );
      if (old.isNotEmpty) {
        add(
          FindingLevel.warning,
          'Game DLL mods (${_capList(old.map((m) => m.fileName).toList(), 4)}) '
          'are told the game is the 2021 build, but Steam 2026-11 is installed. '
          'A mod made for 2021 reads the wrong memory there; check that it '
          'supports the update or turn it off in Third Party.',
        );
      }
    }

    if (r.texturePacks.isNotEmpty &&
        r.texturePacks.every((t) => packDisabledBy(r, t) != null)) {
      add(FindingLevel.info, 'Every texture pack is disabled.');
    }
    final missingRequires = r.mods.where((m) => m.requiresMissing.isNotEmpty);
    for (final m in missingRequires.take(5)) {
      add(
        FindingLevel.warning,
        'Mod ${m.id} needs missing mod(s): ${m.requiresMissing.join(', ')}.',
      );
    }
    if (r.hiddenModFolders.isNotEmpty) {
      add(
        FindingLevel.info,
        '${r.hiddenModFolders.length} mod folder(s) start with "_": hidden in the '
        'launcher, NAMS still loads them.',
      );
    }

    final migoto = r.thirdParty.migoto;
    if (migoto.installed && migoto.migotoInfo?.loaderTargetOk == false) {
      add(
        FindingLevel.warning,
        '3DMigoto d3dx.ini [Loader] target is not NAMS.exe.',
      );
    }
    final reshade = r.thirdParty.reshade;
    if (reshade.installed &&
        (reshade.reshadeInfo?.d3dCompilerMissing ?? false)) {
      add(
        FindingLevel.warning,
        'ReShade: d3dcompiler_47.dll missing, shaders will not compile under Wine.',
      );
    }

    for (final e in r.collectionErrors.entries) {
      add(FindingLevel.warning, 'Could not collect ${e.key}: ${e.value}');
    }

    out.sort((a, b) => a.level.index.compareTo(b.level.index));
    return out;
  }

  static String? packDisabledBy(DiagnosticsReport r, NamsTexturePack t) {
    if (!t.enabled) return 'texture_injection.toml';
    if (r.gameDir.isEmpty || r.disabledModsEntries.isEmpty) return null;
    final namsRoot = p.normalize(p.join(r.gameDir, 'nams'));
    final packPath = p.normalize(t.path);
    if (!p.isWithin(namsRoot, packPath)) return null;
    final rel = p.relative(packPath, from: namsRoot).replaceAll('\\', '/');
    final hit = r.disabledModsEntries.any(
      (d) => rel == d || rel.startsWith('$d/'),
    );
    return hit ? 'disabled_mods.toml' : null;
  }

  static const _steamBuildIds = <String, String>{'25901062': 'Steam 2026-11'};

  static bool _deltaIs(
    DiagnosticsReport r,
    String file,
    String key,
    String value,
  ) => r.config.deltas.any(
    (d) => d.file == file && d.key == key && d.value == value,
  );

  static int _countDisabledMods(DiagnosticsReport r) {
    final disabled = r.disabledModsEntries.toSet();
    var n = 0;
    for (final m in r.mods) {
      final rel = 'mods/${m.id}';
      if (disabled.any((p) => rel == p || rel.startsWith('$p/'))) n++;
    }
    return n;
  }

  static String _humanSize(int bytes) {
    if (bytes >= 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB';
    }
    if (bytes >= 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    if (bytes >= 1024) return '${(bytes / 1024).round()} KB';
    return '$bytes B';
  }

  static String _hex(int? v) =>
      v == null ? '?' : '0x${v.toRadixString(16).padLeft(8, '0')}';

  static String _time(DateTime? t) {
    if (t == null) return '?';
    final utc = t.toUtc().toIso8601String();
    final dot = utc.indexOf('.');
    return dot == -1 ? utc : '${utc.substring(0, dot)}Z';
  }

  static String _fileLine(FileFacts f) =>
      '${f.name}  ${_humanSize(f.size)}  modified ${_time(f.modified)}'
      '${f.shortSha1 != null ? '  sha1 ${f.shortSha1}' : ''}';

  static String _humanizeConflictKind(ModConflictKind k) {
    switch (k) {
      case ModConflictKind.overlappingDataFile:
        return 'overlapping data file';
      case ModConflictKind.outfitIdCollision:
        return 'outfit ID collision';
      case ModConflictKind.vanillaOutfitId:
        return 'vanilla outfit ID';
    }
  }

  static String redact(String input, DiagnosticsReport r) {
    if (input.isEmpty) return input;
    var out = input;

    final paths = <String, String>{};
    void addPath(String raw, String token) {
      if (raw.isEmpty) return;
      final variants = <String>{
        raw,
        raw.replaceAll('\\', '/'),
        raw.replaceAll('/', '\\'),
        if (raw.startsWith('/')) 'Z:${raw.replaceAll('/', '\\')}',
        if (raw.startsWith('/')) 'Z:$raw',
        if (raw.startsWith('/')) 'z:${raw.replaceAll('/', '\\')}',
      };
      for (final v in variants) {
        if (v.length >= 3) paths.putIfAbsent(v, () => token);
      }
    }

    final gameDir = r.launcherInfo['Game dir'] ?? '';
    if (gameDir.isNotEmpty && gameDir != '<not selected>') {
      addPath(gameDir, '<gameDir>');
    }
    addPath(r.launcherInfo['Launcher dir'] ?? '', '<launcherDir>');
    final env = Platform.environment;
    addPath(env['USERPROFILE'] ?? env['HOME'] ?? '', '<userHome>');

    final ordered = paths.keys.toList()
      ..sort((a, b) => b.length.compareTo(a.length));
    for (final from in ordered) {
      out = _replaceIgnoreCase(out, from, paths[from]!);
    }

    final username = userNameForTest ?? env['USERNAME'] ?? env['USER'] ?? '';
    if (username.length >= 2) {
      out = _replaceUserInPaths(out, username, '<user>');
    }
    return out;
  }

  static String _replaceIgnoreCase(String input, String from, String to) {
    final lowerInput = input.toLowerCase();
    final lowerFrom = from.toLowerCase();
    final out = StringBuffer();
    var start = 0;
    while (true) {
      final i = lowerInput.indexOf(lowerFrom, start);
      if (i == -1) break;
      out
        ..write(input.substring(start, i))
        ..write(to);
      start = i + from.length;
    }
    out.write(input.substring(start));
    return out.toString();
  }

  static bool _isWordChar(int c) =>
      (c >= 0x30 && c <= 0x39) ||
      (c >= 0x41 && c <= 0x5A) ||
      (c >= 0x61 && c <= 0x7A) ||
      c == 0x5F ||
      c == 0x2D ||
      c == 0x2E;

  static bool _isSeparator(String c) => c == '/' || c == '\\';

  static bool _followsHomeRoot(String lowerInput, int i) {
    if (i < 2 || !_isSeparator(lowerInput[i - 1])) return false;
    var j = i - 2;
    while (j >= 0 && !_isSeparator(lowerInput[j])) {
      j--;
    }
    final segment = lowerInput.substring(j + 1, i - 1);
    return segment == 'users' || segment == 'home';
  }

  static String _replaceUserInPaths(String input, String user, String to) {
    final lowerInput = input.toLowerCase();
    final lowerUser = user.toLowerCase();
    final out = StringBuffer();
    var start = 0;
    var search = 0;
    while (true) {
      final i = lowerInput.indexOf(lowerUser, search);
      if (i == -1) break;
      final end = i + user.length;
      final before = _followsHomeRoot(lowerInput, i);
      final after = end >= input.length || !_isWordChar(input.codeUnitAt(end));
      if (before && after) {
        out
          ..write(input.substring(start, i))
          ..write(to);
        start = end;
      }
      search = end;
    }
    out.write(input.substring(start));
    return out.toString();
  }

  static String formatSummary(DiagnosticsReport r) {
    final b = StringBuffer();
    b.writeln('YP Launcher Diagnostics (format $_reportFormat)');
    b.writeln('Generated: ${_time(r.generatedAt)}');
    b.writeln(
      'Launcher: ${r.launcherInfo['Launcher version'] ?? '?'}   '
      'OS: ${r.systemInfo['OS']} ${r.systemInfo['OS version']}',
    );
    if (r.systemInfo['Distro'] != null) {
      b.writeln(
        'Distro: ${r.systemInfo['Distro']}   '
        'Session: ${r.systemInfo['Session']}   '
        'AppImage: ${r.systemInfo['AppImage']}   '
        'Flatpak: ${r.systemInfo['Flatpak']}',
      );
    }
    final gi = r.gameIdentity;
    final manifest = gi.steamManifest;
    b.writeln(
      'Game: ${gi.buildDisplay}'
      '${manifest?.buildId != null ? '   Steam build ${manifest!.buildId}' : ''}'
      '   DLC=${gi.hasDlc}   running=${r.gameRunning}',
    );
    b.writeln(
      'NAMS: exe=${r.namsHealth.namsExePresent}'
      '${r.namsHealth.namsVersion != null ? ' v${r.namsHealth.namsVersion}' : ''}'
      '   missing=${r.namsHealth.missingFiles.length}'
      '   Steam running=${r.systemInfo['Steam running'] ?? '?'}',
    );
    final compat = r.compatLayer;
    if (compat != null) {
      b.writeln(
        'Compat: ${compat.label}'
        '${compat.toolVersion != null ? ' (${compat.toolVersion})' : ''}',
      );
    }
    for (final e in r.filesystems.entries) {
      if (e.value != null) b.writeln('FS ${e.key}: ${e.value!.fsType}');
    }
    final last = r.lastLaunch;
    if (last != null) {
      b.writeln(
        'Last launch: ${_time(last.startedAt)}   '
        'exit=${last.exitCode?.toString() ?? (last.stillRunning ? 'running/unknown' : '?')}'
        '${last.duration != null ? '   ${last.duration!.inSeconds}s' : ''}',
      );
    }
    b.writeln(
      'Mods: ${r.mods.length}   disabled prefixes: ${r.disabledModsEntries.length}   '
      'texture packs: ${r.texturePacksAvailable ? '${r.texturePacks.where((t) => packDisabledBy(r, t) == null).length}/${r.texturePacks.length} on' : r.namsTextures.length}   '
      'cutscene mods: ${r.cutsceneMods.length}   '
      'non-default settings: ${r.config.deltas.length}',
    );
    final rs = r.thirdParty.reshade;
    final mi = r.thirdParty.migoto;
    final gm = r.thirdParty.gameMods;
    final tp = [
      if (rs.installed) 'ReShade${rs.enabled ? '' : ' (off)'}',
      if (mi.installed) '3DMigoto${mi.enabled ? '' : ' (off)'}',
      if (gm.installed) 'game DLLs ${gm.gameModsInfo?.enabledCount ?? 0}',
      if (r.thirdParty.graphicMods.installed)
        'graphic DLLs ${r.thirdParty.graphicMods.graphicModsInfo?.enabledCount ?? 0}',
    ];
    if (tp.isNotEmpty) b.writeln('Third-party: ${tp.join(', ')}');

    b.writeln('');
    if (r.findings.isEmpty) {
      b.writeln('Findings: none');
    } else {
      b.writeln('Findings:');
      for (final f in r.findings) {
        b.writeln('${f.tag} ${f.message}');
      }
    }

    var text = redact(b.toString(), r);
    if (text.length > _discordLimit) {
      const more = '\n... (see the full report)';
      text = '${text.substring(0, _discordLimit - more.length)}$more';
    }
    return text;
  }

  static String formatFullReport(DiagnosticsReport r) {
    final b = StringBuffer();
    void section(String title) {
      b.writeln('');
      b.writeln('=' * 72);
      b.writeln(title);
      b.writeln('=' * 72);
    }

    b.writeln('YP Launcher Diagnostics Report');
    b.writeln('Format: $_reportFormat');
    b.writeln('Generated: ${_time(r.generatedAt)}');

    section('Findings (${r.findings.length})');
    if (r.findings.isEmpty) {
      b.writeln('(none)');
    } else {
      for (final f in r.findings) {
        b.writeln('${f.tag} ${f.message}');
      }
    }

    section('System');
    r.systemInfo.forEach((k, v) => b.writeln('$k: $v'));
    if (r.filesystems.isNotEmpty) {
      b.writeln('');
      b.writeln('Filesystems:');
      r.filesystems.forEach((k, v) {
        b.writeln(
          v == null
              ? '  $k: unknown'
              : '  $k: ${v.fsType} (mount ${v.target}${v.options.isEmpty ? '' : ', ${v.options}'})',
        );
      });
    }

    section('Launcher');
    r.launcherInfo.forEach((k, v) {
      if (k == 'Launcher dir' || k == 'Game dir') return;
      b.writeln('$k: $v');
    });

    final gi = r.gameIdentity;
    section('Game identity');
    b.writeln(
      'build: ${gi.buildDisplay} '
      '(${gi.exeVariantSupported ? 'supported' : 'UNSUPPORTED'})',
    );
    b.writeln('exe variant: ${gi.exeVariant}');
    if (gi.exeSize != null) {
      b.writeln(
        'exe: ${_humanSize(gi.exeSize!)} (${gi.exeSize} bytes)  '
        'sha1 ${gi.exeSha1 ?? '?'}  PE stamp ${_hex(gi.exeTimeDateStamp)}',
      );
    }
    b.writeln(
      'known builds: '
      '${GameDetection.knownBuilds.map((k) => '${k.label} (${k.exeSize} bytes)').join(', ')}',
    );
    if (gi.compatExePresent) b.writeln('NieRAutomataCompat.exe: present');
    if (gi.buildInfo.isNotEmpty) {
      b.writeln(
        'Latest_BuildInfo.properties: '
        '${gi.buildInfo.entries.map((e) => '${e.key}=${e.value}').join('  ')}',
      );
    }
    final manifest = gi.steamManifest;
    if (manifest != null) {
      b.writeln(
        'Steam manifest: build ${manifest.buildId ?? '?'}'
        '${manifest.updatePending ? ' (update pending -> ${manifest.targetBuildId})' : ''}'
        '  state ${manifest.stateFlags ?? '?'}'
        '  updated ${_time(manifest.lastUpdatedAt)}'
        '${manifest.betaKey != null ? '  beta ${manifest.betaKey}' : ''}',
      );
      b.writeln(
        'Steam DLC: installed [${manifest.installedDlc.join(', ')}]'
        '  disabled [${manifest.disabledDlc.join(', ')}]',
      );
    } else if (r.gameDir.isNotEmpty) {
      b.writeln('Steam manifest: not found (game outside steamapps/common?)');
    }
    b.writeln(
      'DLC (3C3C Concert & Costume): ${gi.hasDlc} (${gi.dlcSource})'
      '${gi.data100Size != null ? '  data100.cpk ${gi.data100Size} bytes' : ''}',
    );
    if (gi.gameBin != null) {
      b.writeln(
        'runtime/game.bin: ${_fileLine(gi.gameBin!)}  '
        'PE stamp ${_hex(gi.gameBinTimeDateStamp)}'
        ' (${gi.gameBinBuildLabel ?? 'unknown'}, '
        '${gi.gameBinMatchesExe ? 'matches exe' : 'rebuilt on next launch'})',
      );
    } else {
      b.writeln('runtime/game.bin: not built yet');
    }

    section('NAMS runtime health');
    final health = r.namsHealth;
    b.writeln(
      'NAMS.exe: ${health.namsExe != null ? _fileLine(health.namsExe!) : 'missing'}',
    );
    b.writeln(
      'NAMS version: ${health.namsVersion ?? (Platform.isWindows ? 'unavailable' : 'not queried (Windows only)')}',
    );
    if (health.nierInstallError != null) {
      b.writeln('NAMS install lookup: ${health.nierInstallError}');
    }
    b.writeln('Plugins (${health.plugins.length}):');
    for (final f in health.plugins) {
      b.writeln('  - ${_fileLine(f)}');
    }
    b.writeln('Game running: ${r.gameRunning}');
    b.writeln('Steam running: ${r.systemInfo['Steam running'] ?? '?'}');
    if (health.missingFiles.isEmpty) {
      b.writeln('Missing runtime files: none');
    } else {
      b.writeln(
        '!! Missing runtime files (AV/quarantine?): '
        '${health.missingFiles.join(', ')}',
      );
    }
    if (health.launchCommandPreview != null) {
      b.writeln('');
      b.writeln('Launch command:');
      b.writeln(health.launchCommandPreview!);
    }

    final compat = r.compatLayer;
    if (!Platform.isWindows) {
      section('Compatibility layer');
      if (compat == null) {
        b.writeln(r.gameDir.isEmpty ? '(no game folder)' : '(none found)');
      } else {
        b.writeln('source: ${compat.source}  label: ${compat.label}');
        b.writeln('binary: ${compat.binary}');
        if (compat.toolVersion != null) {
          b.writeln('tool version: ${compat.toolVersion}');
        }
        b.writeln(
          'prefix: ${compat.prefix ?? '?'} '
          '(${compat.prefixExists ? 'exists' : 'not created yet'})',
        );
        if (compat.prefixVersion != null) {
          b.writeln('prefix created by: ${compat.prefixVersion}');
        }
      }
    }

    section('Last launch');
    final last = r.lastLaunch;
    if (last == null) {
      b.writeln('(no launch recorded)');
    } else {
      b.writeln('started: ${_time(last.startedAt)}  via: ${last.label}');
      if (last.stillRunning) {
        b.writeln(
          'ended: not recorded (still running, or the launcher closed first)',
        );
      } else {
        b.writeln(
          'ended: ${_time(last.endedAt)}  exit code: ${last.exitCode}  '
          'duration: ${last.duration?.inSeconds}s',
        );
      }
      final tail = last.outputTail.trim();
      if (tail.isNotEmpty) {
        b.writeln('output tail:');
        final lines = tail.split('\n');
        for (final l in lines.skip(lines.length > 40 ? lines.length - 40 : 0)) {
          b.writeln('  ${l.trimRight()}');
        }
      }
    }

    section('Logs');
    final logs = r.logs;
    final namsLog = logs.namsLog;
    if (namsLog == null) {
      b.writeln('nams.log: missing');
    } else {
      b.writeln(
        'nams.log: ${_humanSize(namsLog.size)}  modified ${_time(namsLog.modified)}  '
        '(last ${namsLog.lines.length} lines)',
      );
      for (final l in namsLog.lines) {
        b.writeln('  $l');
      }
    }
    b.writeln('');
    b.writeln('crash dumps: ${logs.crashDumpCount}');
    final dump = logs.newestCrashDump;
    if (dump != null) {
      b.writeln('newest: ${dump.name}  modified ${_time(dump.modified)}');
      for (final l in dump.lines) {
        b.writeln('  $l');
      }
    }
    final heartbeat = logs.heartbeat;
    if (heartbeat != null) {
      b.writeln('');
      b.writeln('heartbeat.log: modified ${_time(heartbeat.modified)}');
      for (final l in heartbeat.lines) {
        b.writeln('  $l');
      }
    }
    if (logs.capturedLaunchLogs.isNotEmpty) {
      b.writeln('');
      b.writeln('captured early-exit logs:');
      for (final f in logs.capturedLaunchLogs) {
        b.writeln('  - ${_fileLine(f)}');
      }
    }

    section('Recent NAMS warnings/errors (${r.recentLogIssues.length})');
    if (r.recentLogIssues.isEmpty) {
      b.writeln('(none in nams.log)');
    } else {
      final grouped = <String, ({LogEntry last, int count})>{};
      for (final e in r.recentLogIssues) {
        final key = '${e.level} ${e.module}: ${e.message}';
        final prev = grouped.remove(key);
        grouped[key] = (last: e, count: (prev?.count ?? 0) + 1);
      }
      for (final g in grouped.entries) {
        b.writeln(
          '  ${g.value.last.timestamp} ${g.key}'
          '${g.value.count > 1 ? '  (x${g.value.count})' : ''}',
        );
      }
    }

    section(
      'Vanilla data/ drops (user-added files, ${r.vanillaDropped.length})',
    );
    if (r.vanillaDropped.isEmpty) {
      b.writeln('(none, vanilla data/ is clean)');
    } else {
      for (final f in r.vanillaDropped) {
        b.writeln('  ${f.path}  ${_humanSize(f.sizeBytes)}  [${f.bucket}]');
      }
      if (r.vanillaDropped.length >= _vanillaDropCap) {
        b.writeln('  ... (capped at $_vanillaDropCap)');
      }
    }
    if (r.dataDirContents.isNotEmpty) {
      b.writeln('');
      b.writeln('Non-vanilla folders in data/:');
      final entries = r.dataDirContents.entries.toList()
        ..sort((a, b) => b.value.compareTo(a.value));
      for (final e in entries) {
        b.writeln('  ${e.key}/  (${e.value} files)');
      }
    }

    section('ReShade');
    _writeReShade(b, r.thirdParty.reshade);

    section('3DMigoto');
    _writeMigoto(b, r.thirdParty.migoto);

    section('Third-party game & graphic DLLs');
    final gm = r.thirdParty.gameMods;
    b.writeln(
      'thirdparty/game: ${gm.installed ? '' : '(none)'}'
      '${gm.installed ? 'loading ${gm.enabled ? 'enabled' : 'DISABLED'}' : ''}',
    );
    for (final m in gm.gameModsInfo?.mods ?? const <GameModEntry>[]) {
      final s = m.settings;
      b.writeln(
        '  - ${m.fileName}  ${m.sizeLabel ?? ''}${m.disabled ? '  [disabled]' : ''}'
        '${s == null ? '' : '  module=${s.fakeModuleHandle ? 'game.bin' : 'NAMS.exe'}'
                  '  exePath=${s.fakeExePath ? 'NieRAutomata.exe' : 'NAMS.exe'}'
                  '  sizeOfImage=${s.sizeOfImage}${s.reportPatches ? '  reportPatches' : ''}${s.waitForPatchPhase ? '  waitForPatchPhase=${s.patchPhaseTimeoutMs}ms' : ''}'}',
      );
    }
    final gx = r.thirdParty.graphicMods;
    b.writeln(
      'thirdparty/graphic: ${gx.installed ? '' : '(none)'}'
      '${gx.installed ? 'loading ${gx.enabled ? 'enabled' : 'DISABLED'}' : ''}',
    );
    for (final m in gx.graphicModsInfo?.mods ?? const <GameModEntry>[]) {
      b.writeln(
        '  - ${m.fileName}  ${m.sizeLabel ?? ''}${m.disabled ? '  [disabled]' : ''}',
      );
    }

    section(
      'Texture packs (${r.texturePacksAvailable ? r.texturePacks.length : r.namsTextures.length})',
    );
    if (!r.texturePacksAvailable) {
      b.writeln('NAMS query: ${r.texturePacksNote ?? 'unavailable'}');
    } else if (r.texturePacks.isEmpty) {
      b.writeln('(none)');
    } else {
      for (final t in r.texturePacks) {
        final off = packDisabledBy(r, t);
        b.writeln(
          '- ${t.name}  [${t.source}]  '
          '${off == null ? 'enabled' : 'disabled ($off)'}  ${t.ddsCount} dds'
          '${t.mod != null ? '  mod=${t.mod}' : ''}'
          '${t.character != null ? '  char=${t.character}' : ''}'
          '${t.outfitConditional ? '  outfit-conditional' : ''}'
          '${t.loadOrderIndex != null ? '  order=${t.loadOrderIndex}' : ''}',
        );
      }
    }
    b.writeln('');
    b.writeln(
      'nams/inject/textures/ (${r.namsTextures.length}): ${_capList(r.namsTextures)}',
    );
    if (r.skResTextures.isNotEmpty) {
      b.writeln(
        'SK_Res/inject/textures/ (${r.skResTextures.length}): ${_capList(r.skResTextures)}',
      );
    }
    if (r.skResPacks.isNotEmpty) {
      b.writeln('SK_Res packs (all exe folders): ${_capList(r.skResPacks)}');
    }
    if (r.waxTextures.isNotEmpty) {
      b.writeln(
        'WAX textures (${r.waxTextures.length}): ${_capList(r.waxTextures)}',
      );
    }

    section('Settings');
    final cfg = r.config;
    if (cfg.missingFiles.isNotEmpty) {
      b.writeln('missing or empty: ${cfg.missingFiles.join(', ')}');
    }
    for (final e in cfg.parseErrors.entries) {
      b.writeln('!! ${e.key} does not parse: ${e.value}');
    }
    b.writeln('');
    b.writeln('Non-default (${cfg.deltas.length}):');
    if (cfg.deltas.isEmpty) {
      b.writeln('  (all defaults)');
    }
    for (final d in cfg.deltas) {
      b.writeln(
        '  [${d.file}] ${d.key} = ${d.value}  (default ${d.defaultValue})',
      );
    }
    if (cfg.unknownKeys.isNotEmpty) {
      b.writeln('');
      b.writeln(
        'Unknown keys, NAMS does not read them (${cfg.unknownKeys.length}):',
      );
      _writeKeyGroups(b, cfg.unknownKeys);
    }
    if (cfg.legacyKeys.isNotEmpty) {
      b.writeln('');
      b.writeln('Old keys left in the files (${cfg.legacyKeys.length}):');
      _writeKeyGroups(b, cfg.legacyKeys);
    }

    section('Profiles');
    b.writeln('active profile: ${r.activeProfile ?? 'default'}');
    if (r.defaultModsEntries.isNotEmpty) {
      b.writeln('default_mods.toml:');
      for (final e in r.defaultModsEntries) {
        b.writeln('  $e');
      }
    }

    section('Installed mods (${r.mods.length})');
    final disabledSet = r.disabledModsEntries.toSet();
    final byKind = <ModKind, int>{};
    var withWarnings = 0;
    var withBundles = 0;
    for (final m in r.mods) {
      byKind[m.kind] = (byKind[m.kind] ?? 0) + 1;
      if (m.hasWarnings) withWarnings++;
      if (m.bundledTexturePacks.isNotEmpty || m.bundledCutscenes.isNotEmpty) {
        withBundles++;
      }
    }
    if (r.mods.isNotEmpty) {
      b.writeln(
        'Totals by kind: ${byKind.entries.map((e) => '${e.key.name}=${e.value}').join(', ')}',
      );
      b.writeln(
        'With warnings: $withWarnings   '
        'With bundled assets: $withBundles   '
        'Disabled prefixes affecting mods: ${_countDisabledMods(r)}',
      );
      b.writeln('');
    }
    if (r.hiddenModFolders.isNotEmpty) {
      b.writeln(
        'Hidden "_" folders (NAMS still loads them): ${r.hiddenModFolders.join(', ')}',
      );
      b.writeln('');
    }

    for (final m in r.mods) {
      final disabledByPrefix = disabledSet.any(
        (p) => 'mods/${m.id}' == p || 'mods/${m.id}'.startsWith('$p/'),
      );
      final statusTag = disabledByPrefix ? '[DISABLED]' : '[enabled]';
      final warnTag = m.hasWarnings ? '  !! warnings' : '';

      b.writeln(
        '- ${m.id}  $statusTag  [${m.kind.name}]'
        '${m.manifest?.version != null ? '  v${m.manifest!.version}' : ''}'
        '$warnTag',
      );
      final detailed =
          m.hasWarnings ||
          m.conflicts.isNotEmpty ||
          m.requiresMissing.isNotEmpty;
      if (!detailed) continue;
      if (m.displayName.isNotEmpty && m.displayName != m.id) {
        b.writeln('    name:     ${m.displayName}');
      }
      b.writeln('    installed: ${_time(m.installedAt)}');
      final mf = m.manifest;
      if (mf != null) {
        if (mf.author != null) b.writeln('    author:   ${mf.author}');
        if (mf.namsMinVersion != null) {
          b.writeln('    NAMS min version: ${mf.namsMinVersion}');
        }
        if (mf.requires.isNotEmpty) {
          b.writeln('    requires mods:    ${mf.requires.join(', ')}');
        }
        if (mf.requiresPlugins.isNotEmpty) {
          b.writeln('    requires plugins: ${mf.requiresPlugins.join(', ')}');
        }
        if (mf.replaces.isNotEmpty) {
          b.writeln('    replaces:         ${mf.replaces.join(', ')}');
        }
      }
      if (m.requiresMissing.isNotEmpty) {
        b.writeln('    !! missing requires: ${m.requiresMissing.join(', ')}');
      }
      if (m.native != null) {
        final n = m.native!;
        if (n.bundlesByKind.isNotEmpty || n.totalEntityFiles > 0) {
          final bundles = n.bundlesByKind.entries
              .map((e) => '${e.key}=${e.value}')
              .join(', ');
          b.writeln(
            '    native:   bundles=[$bundles]'
            '  entityFiles=${n.totalEntityFiles}',
          );
        }
      }
      if (m.data != null) {
        final d = m.data!;
        if (d.entries.isNotEmpty) {
          final entryNames = d.entries.map((e) => e.dirName).toList()..sort();
          b.writeln('    data dirs: ${entryNames.join(', ')}');
        }
        if (d.players.isNotEmpty) {
          b.writeln('    player models:');
          for (final p in d.players) {
            final lookup = playerModelLookup[p.fileName.split('.').first];
            b.writeln('      - ${p.fileName}  ->  ${lookup ?? p.label}');
          }
        }
      }
      if (m.conflicts.isNotEmpty) {
        b.writeln('    conflicts (${m.conflicts.length}):');
        for (final c in m.conflicts) {
          b.writeln(
            '      - vs ${c.otherModId} '
            '[${_humanizeConflictKind(c.kind)}] ${c.detail}',
          );
        }
      }
    }

    section('Disabled mod prefixes (${r.disabledModsEntries.length})');
    for (final e in r.disabledModsEntries) {
      b.writeln('- $e');
    }

    section('Cutscene mods (${r.cutsceneMods.length})');
    for (final c in r.cutsceneMods) {
      final origin = c.bundledWithModId == null
          ? 'standalone'
          : 'bundled with ${c.bundledWithModId}';
      b.writeln(
        '- ${c.name}  [$origin]  ${c.usmCount} USM'
        '${c.hasH264 ? '  H264' : '  MPEG-2'}'
        '${c.maxWidth > 0 ? '  ${c.maxWidth}x${c.maxHeight}' : ''}'
        '${c.missingOriginals.isNotEmpty ? '  missing originals: ${c.missingOriginals.length}' : ''}',
      );
    }
    if (r.directOverrides.isNotEmpty) {
      b.writeln('');
      b.writeln(
        'Custom files in data/movie/ (${r.directOverrides.length}): '
        '${_capList(r.directOverrides)}',
      );
    }
    b.writeln('');
    b.writeln(
      'cutscene detection: '
      'hasHd=${r.cutsceneDetection.hasHdCutscenes}, '
      'needsH264=${r.cutsceneDetection.needsH264}, '
      'filesScanned=${r.cutsceneDetection.filesScanned}, '
      'largest=${r.cutsceneDetection.largestWidth}x${r.cutsceneDetection.largestHeight}',
    );

    section('Other folders');
    b.writeln(
      'wax/mods/: ${r.waxModsRoot.isEmpty ? '(none)' : _capList(r.waxModsRoot)}',
    );
    b.writeln(
      'reshade/: ${r.reshadeContents.isEmpty ? '(none)' : _capList(r.reshadeContents)}',
    );
    b.writeln(
      'game root extras: '
      '${r.gameRootExtras.isEmpty ? '(none, clean install root)' : _capList(r.gameRootExtras, 40)}',
    );

    for (final e in r.config.parseErrors.keys) {
      section('nams/$e (does not parse)');
      b.writeln(r.configRaw[e] ?? '');
    }

    return redact(b.toString(), r);
  }

  static void _writeKeyGroups(StringBuffer b, List<ConfigDelta> keys) {
    final groups = <String, List<ConfigDelta>>{};
    for (final d in keys) {
      final dot = d.key.indexOf('.');
      final section = dot == -1 ? '' : d.key.substring(0, dot);
      groups.putIfAbsent('${d.file}|$section|${d.note ?? ''}', () => []).add(d);
    }
    for (final group in groups.values) {
      final first = group.first;
      final dot = first.key.indexOf('.');
      final note = _noteSuffix(first.note ?? '');
      if (dot != -1 && group.length > 4) {
        b.writeln(
          '  [${first.file}] [${first.key.substring(0, dot)}] '
          '${group.length} keys$note',
        );
        continue;
      }
      for (final d in group) {
        b.writeln('  [${d.file}] ${d.key} = ${d.value}$note');
      }
    }
  }

  static String _noteSuffix(String note) => note.isEmpty ? '' : '  ($note)';

  static String _capList(List<String> items, [int cap = 25]) {
    if (items.length <= cap) return items.join(', ');
    return '${items.take(cap).join(', ')}, ... ${items.length - cap} more';
  }

  static void _writeReShade(StringBuffer b, ThirdPartyRuntimeStatus s) {
    if (!s.installed) {
      b.writeln('(not installed)');
      return;
    }
    b.writeln(
      'installed: true   enabled: ${s.enabled}   '
      'shadersMissing: ${s.shadersMissing}',
    );
    final info = s.reshadeInfo;
    if (info != null) {
      if (info.version != null) b.writeln('version: ${info.version}');
      if (info.dllName != null) {
        b.writeln('dll: ${info.dllName}   addonBuild: ${info.isAddonBuild}');
      }
      b.writeln('presets (${info.presets.length}): ${info.presets.join(', ')}');
      b.writeln(
        'shader repos (${info.shaderRepos.length}): '
        '${info.shaderRepos.join(', ')}   effects: ${info.shaderCount}',
      );
      if (info.addons.isNotEmpty) {
        b.writeln('addons (${info.addons.length}): ${info.addons.join(', ')}');
      }
      if (info.d3dCompilerMissing) {
        b.writeln(
          '!! d3dcompiler_47.dll missing (Wine shaders will not compile)',
        );
      }
      final c = info.config;
      b.writeln(
        'config: performanceMode=${c.performanceMode} '
        'showFps=${c.showFps} showClock=${c.showClock}'
        '${c.activePreset != null ? ' activePreset=${c.activePreset}' : ''}',
      );
    }
  }

  static void _writeMigoto(StringBuffer b, ThirdPartyRuntimeStatus s) {
    if (!s.installed) {
      b.writeln('(not installed)');
      return;
    }
    b.writeln(
      'installed: true   enabled: ${s.enabled}   '
      'hasShaderFixes: ${s.hasShaderFixes}',
    );
    final info = s.migotoInfo;
    if (info != null) {
      b.writeln('files: ${info.files.join(', ')}');
      b.writeln(
        'loader target: ${info.loaderTarget ?? '?'} '
        '(ok: ${info.loaderTargetOk})',
      );
      b.writeln('shader fixes: ${info.shaderFixCount}');
      if (info.shaderFixNames.isNotEmpty) {
        b.writeln('  from: ${info.shaderFixNames.join(', ')}');
      }
      final c = info.config;
      b.writeln(
        'config: hunting=${c.hunting.name} marking=${c.markingMode.name} '
        'cacheShaders=${c.cacheShaders} verboseOverlay=${c.verboseOverlay}',
      );
    }
  }

  static Future<String> writeFullReport(DiagnosticsReport r) async {
    final dir = Directory(LogService.logsDirectory);
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }
    final stamp = r.generatedAt
        .toIso8601String()
        .replaceAll(':', '-')
        .replaceAll('.', '-');
    final out = p.join(dir.path, 'diagnostics_$stamp.txt');
    final file = File(out);
    await file.writeAsString(formatFullReport(r));
    return out;
  }
}

class _TextureScanParams {
  final String gameDir;
  const _TextureScanParams({required this.gameDir});
}

Map<String, dynamic> _scanTexturesSync(_TextureScanParams params) {
  final nams = <String>[];
  final sk = <String>[];
  final wax = <String>[];

  final namsDir = Directory(
    p.join(params.gameDir, 'nams', 'inject', 'textures'),
  );
  if (namsDir.existsSync()) {
    for (final entity in namsDir.listSync()) {
      if (entity is Directory) {
        nams.add(p.basename(entity.path));
      }
    }
  }

  final skDir = Directory(
    p.join(params.gameDir, 'SK_Res', 'inject', 'textures', 'NieRAutomata.exe'),
  );
  if (skDir.existsSync()) {
    for (final entity in skDir.listSync()) {
      if (entity is Directory) {
        sk.add(p.basename(entity.path));
      }
    }
  }

  final waxDir = Directory(p.join(params.gameDir, 'wax', 'mods'));
  if (waxDir.existsSync()) {
    for (final modDir in waxDir.listSync().whereType<Directory>()) {
      final tex = Directory(p.join(modDir.path, 'textures'));
      if (tex.existsSync()) {
        wax.add(p.basename(modDir.path));
      }
    }
  }

  return {'nams': nams, 'sk': sk, 'wax': wax};
}

({FileFacts? namsExe, List<FileFacts> plugins}) _runtimeFilesSync(
  ({String namsExe, String pluginsDir}) params,
) {
  return (
    namsExe: params.namsExe.isEmpty
        ? null
        : DiagnosticsProbes.facts(params.namsExe, hash: true),
    plugins: params.pluginsDir.isEmpty
        ? const <FileFacts>[]
        : DiagnosticsProbes.filesIn(
            params.pluginsDir,
            where: (n) => n.endsWith('.dll'),
            hash: true,
          ),
  );
}

class _ExtraScan {
  final Map<String, int> dataDirContents;
  final List<String> waxModsRoot;
  final List<String> skResPacks;
  final List<String> reshadeContents;
  final List<String> gameRootExtras;
  const _ExtraScan({
    required this.dataDirContents,
    required this.waxModsRoot,
    required this.skResPacks,
    required this.reshadeContents,
    required this.gameRootExtras,
  });
}

const _vanillaDataDirs = <String>{'movie', 'enlighten', 'sound', 'movie_logo'};

const _vanillaGameRootEntries = <String>{
  'data',
  'wallpaper',
  'redist',
  'logs',
  'nams',
  'nierautomata.exe',
  'nierautomatacompat.exe',
  'latest_buildinfo.properties',
  'steam_api64.dll',
  'installscript.vdf',
  'steam_appid.txt',
};

_ExtraScan _scanExtrasSync(String gameDir) {
  final dataCounts = <String, int>{};
  final dataDir = Directory(p.join(gameDir, 'data'));
  if (dataDir.existsSync()) {
    for (final entity in dataDir.listSync().whereType<Directory>()) {
      final name = p.basename(entity.path);
      if (_vanillaDataDirs.contains(name.toLowerCase())) continue;
      var count = 0;
      try {
        count = entity
            .listSync(recursive: true, followLinks: false)
            .whereType<File>()
            .length;
      } catch (_) {}
      dataCounts[name] = count;
    }
  }

  final waxRoot = <String>[];
  final waxModsDir = Directory(p.join(gameDir, 'wax', 'mods'));
  if (waxModsDir.existsSync()) {
    for (final entity in waxModsDir.listSync().whereType<Directory>()) {
      waxRoot.add(p.basename(entity.path));
    }
  }

  final skResPacks = <String>[];
  final skBase = Directory(p.join(gameDir, 'SK_Res', 'inject', 'textures'));
  if (skBase.existsSync()) {
    for (final exeDir in skBase.listSync().whereType<Directory>()) {
      for (final pack in exeDir.listSync().whereType<Directory>()) {
        skResPacks.add(p.basename(pack.path));
      }
    }
  }

  final reshade = <String>[];
  final reshadeDir = Directory(p.join(gameDir, 'reshade'));
  if (reshadeDir.existsSync()) {
    for (final entity in reshadeDir.listSync()) {
      reshade.add(p.basename(entity.path));
    }
  }

  final rootExtras = <String>[];
  final root = Directory(gameDir);
  if (root.existsSync()) {
    for (final entity in root.listSync()) {
      final name = p.basename(entity.path);
      if (_vanillaGameRootEntries.contains(name.toLowerCase())) continue;
      final tag = entity is Directory ? '$name/' : name;
      rootExtras.add(tag);
    }
    rootExtras.sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
  }

  return _ExtraScan(
    dataDirContents: dataCounts,
    waxModsRoot: waxRoot,
    skResPacks: skResPacks,
    reshadeContents: reshade,
    gameRootExtras: rootExtras,
  );
}

const _vanillaDataRootFiles = <String>{
  'data000.cpk',
  'data001.cpk',
  'data002.cpk',
  'data003.cpk',
  'data004.cpk',
  'data005.cpk',
  'data006.cpk',
  'data007.cpk',
  'data008.cpk',
  'data009.cpk',
  'data010.cpk',
  'data011.cpk',
  'data012.cpk',
  'data013.cpk',
  'data014.cpk',
  'data015.cpk',
  'data016.cpk',
  'data017.cpk',
  'data018.cpk',
  'data019.cpk',
  'data100.cpk',
  'data101.cpk',
  'data102.cpk',
  'data103.cpk',
  'data105.cpk',
  'data107.cpk',
  'data108.cpk',
};

const _vanillaDropCap = 200;

List<VanillaDirFinding> _scanVanillaSync(String gameDir) {
  final findings = <VanillaDirFinding>[];
  final dataDir = Directory(p.join(gameDir, 'data'));
  if (!dataDir.existsSync()) return findings;

  for (final entity in dataDir.listSync(followLinks: false)) {
    if (entity is! File) continue;
    final name = p.basename(entity.path);
    if (_vanillaDataRootFiles.contains(name.toLowerCase())) continue;
    findings.add(
      VanillaDirFinding(
        path: 'data/$name',
        sizeBytes: _lengthQuiet(entity),
        bucket: 'data-root-loose',
      ),
    );
    if (findings.length >= _vanillaDropCap) break;
  }

  findings.sort((a, b) => a.path.toLowerCase().compareTo(b.path.toLowerCase()));
  return findings;
}

int _lengthQuiet(File f) {
  try {
    return f.lengthSync();
  } catch (_) {
    return 0;
  }
}
