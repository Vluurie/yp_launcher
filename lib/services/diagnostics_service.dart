import 'dart:io';
import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/cutscene_detection_service.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/disabled_mods_service.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/nams_cli_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_isolates.dart';

class GameIdentity {
  final String exeVariant;
  final bool exeVariantSupported;
  final bool hasDlc;
  final int? exeSize;
  final bool exeSizeMatchesWin10;

  const GameIdentity({
    this.exeVariant = 'unknown',
    this.exeVariantSupported = false,
    this.hasDlc = false,
    this.exeSize,
    this.exeSizeMatchesWin10 = false,
  });
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

  const ThirdPartyReport({
    this.reshade = const ThirdPartyRuntimeStatus(),
    this.migoto = const ThirdPartyRuntimeStatus(),
  });
}

class NamsHealth {
  final List<String> missingFiles;
  final bool namsExePresent;
  final String? launchCommandPreview;

  const NamsHealth({
    this.missingFiles = const [],
    this.namsExePresent = false,
    this.launchCommandPreview,
  });
}

class ConfigDelta {
  final String file;
  final String key;
  final String value;
  final String defaultValue;

  const ConfigDelta({
    required this.file,
    required this.key,
    required this.value,
    required this.defaultValue,
  });
}

class DiagnosticsReport {
  final DateTime generatedAt;
  final String gameDir;
  final Map<String, String> systemInfo;
  final Map<String, String> launcherInfo;
  final List<InstalledMod> mods;
  final List<String> disabledModsEntries;
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
  final String namsTomlRaw;
  final String lodmodTomlRaw;
  final String textureInjectionTomlRaw;
  final GameIdentity gameIdentity;
  final List<VanillaDirFinding> vanillaDropped;
  final ThirdPartyReport thirdParty;
  final NamsHealth namsHealth;
  final List<NamsTexturePack> texturePacks;
  final bool texturePacksAvailable;
  final List<LogEntry> recentLogIssues;
  final List<ConfigDelta> configDeltas;
  final bool gameRunning;
  final bool preferDedicatedGpu;

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
    required this.namsTomlRaw,
    required this.lodmodTomlRaw,
    required this.textureInjectionTomlRaw,
    this.gameIdentity = const GameIdentity(),
    this.vanillaDropped = const [],
    this.thirdParty = const ThirdPartyReport(),
    this.namsHealth = const NamsHealth(),
    this.texturePacks = const [],
    this.texturePacksAvailable = false,
    this.recentLogIssues = const [],
    this.configDeltas = const [],
    this.gameRunning = false,
    this.preferDedicatedGpu = false,
  });
}

class DiagnosticsService {
  DiagnosticsService._();

  static Future<DiagnosticsReport> collect(
    String gameDir, {
    String? launchCommandPreview,
  }) async {
    final mods = gameDir.isEmpty
        ? <InstalledMod>[]
        : await ModsService.listInstalled(gameDir);
    final disabled = gameDir.isEmpty
        ? <String>[]
        : await DisabledModsService.list(gameDir);
    final cutsceneScan = gameDir.isEmpty
        ? const CutsceneDetection()
        : await CutsceneDetectionService.scan(gameDir);

    final cutscenes = gameDir.isEmpty
        ? const ScanResult(mods: [], directOverrides: [])
        : await scanCutscenes(
            p.join(gameDir, 'nams', 'cutscenes'),
            p.join(gameDir, 'data', 'movie'),
            p.join(gameDir, 'nams', 'mods'),
          );

    final textureScan = gameDir.isEmpty
        ? <String, dynamic>{
            'nams': <String>[],
            'sk': <String>[],
            'wax': <String>[],
          }
        : await IsolateService.run<_TextureScanParams,
            Map<String, dynamic>>(
            _scanTexturesSync,
            _TextureScanParams(gameDir: gameDir),
          );

    final extraScan = gameDir.isEmpty
        ? const _ExtraScan(
            dataDirContents: {},
            waxModsRoot: [],
            skResPacks: [],
            reshadeContents: [],
            gameRootExtras: [],
          )
        : await IsolateService.run(_scanExtrasSync, gameDir);

    final namsToml = gameDir.isEmpty
        ? ''
        : await TomlService.readTomlFile(
            p.join(gameDir, 'nams', 'nams.toml'));
    final lodmodToml = gameDir.isEmpty
        ? ''
        : await TomlService.readTomlFile(
            p.join(gameDir, 'nams', 'lodmod.toml'));
    final textureInjToml = gameDir.isEmpty
        ? ''
        : await TomlService.readTomlFile(
            p.join(gameDir, 'nams', 'texture_injection.toml'));

    final launcherDir =
        await LauncherSetupService.getLauncherDirectory();

    final gameIdentity = await _collectGameIdentity(gameDir);
    final vanillaDropped = gameDir.isEmpty
        ? <VanillaDirFinding>[]
        : await IsolateService.run(_scanVanillaSync, gameDir);
    final thirdParty = await _collectThirdParty(gameDir);
    final namsHealth = await _collectNamsHealth(launchCommandPreview);
    final texturePacksResult =
        gameDir.isEmpty ? null : await _quiet(() => NamsCliService.texturesList(gameDir));
    final recentLogIssues = await _collectRecentIssues();
    final configDeltas = _collectConfigDeltas(namsToml, lodmodToml, textureInjToml);
    final gameRunning =
        await _quiet(() => PlatformAdapter.current.isGameRunning()) ?? false;
    final preferDedicatedGpu = await _collectGpuPref();

    return DiagnosticsReport(
      generatedAt: DateTime.now(),
      gameDir: gameDir,
      systemInfo: {
        'OS': Platform.operatingSystem,
        'OS version': _osVersionLabel(),
        'Locale': Platform.localeName,
        'Dart version': Platform.version,
        'Game running': gameRunning ? 'yes' : 'no',
        'Prefer dedicated GPU': preferDedicatedGpu ? 'yes' : 'no',
      },
      launcherInfo: {
        'Launcher version': AppStrings.appVersion,
        'Launcher dir': launcherDir,
        'Game dir': gameDir.isEmpty ? '<not selected>' : gameDir,
      },
      mods: mods,
      disabledModsEntries: disabled,
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
      namsTomlRaw: namsToml,
      lodmodTomlRaw: lodmodToml,
      textureInjectionTomlRaw: textureInjToml,
      gameIdentity: gameIdentity,
      vanillaDropped: vanillaDropped,
      thirdParty: thirdParty,
      namsHealth: namsHealth,
      texturePacks: texturePacksResult ?? const [],
      texturePacksAvailable: texturePacksResult != null,
      recentLogIssues: recentLogIssues,
      configDeltas: configDeltas,
      gameRunning: gameRunning,
      preferDedicatedGpu: preferDedicatedGpu,
    );
  }

  static Future<T?> _quiet<T>(Future<T?> Function() f) async {
    try {
      return await f();
    } catch (_) {
      return null;
    }
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

  static Future<GameIdentity> _collectGameIdentity(String gameDir) async {
    if (gameDir.isEmpty) return const GameIdentity();
    try {
      final variant = await GameDetection.detectExeVariant(gameDir);
      final hasDlc = await GameDetection.hasDlc(gameDir);
      final exeFile = File(p.join(gameDir, AppStrings.gameExeName));
      final exeSize = exeFile.existsSync() ? exeFile.lengthSync() : null;
      return GameIdentity(
        exeVariant: variant.name,
        exeVariantSupported: variant == ExeVariant.original ||
            variant == ExeVariant.wolfLimitBreak,
        hasDlc: hasDlc,
        exeSize: exeSize,
        exeSizeMatchesWin10: exeSize == GameDetection.windows10Size,
      );
    } catch (_) {
      return const GameIdentity();
    }
  }

  static Future<ThirdPartyReport> _collectThirdParty(String gameDir) async {
    if (gameDir.isEmpty) return const ThirdPartyReport();
    final reshade = await _quiet(() => const ReShadeRuntime().status(gameDir)) ??
        const ThirdPartyRuntimeStatus();
    final migoto = await _quiet(() => const MigotoRuntime().status(gameDir)) ??
        const ThirdPartyRuntimeStatus();
    return ThirdPartyReport(reshade: reshade, migoto: migoto);
  }

  static Future<NamsHealth> _collectNamsHealth(String? launchPreview) async {
    try {
      final missing = await LauncherSetupService.findMissingFiles();
      final paths = await LauncherSetupService.getLauncherPaths();
      final namsExe = paths['namsExe'];
      return NamsHealth(
        missingFiles: missing,
        namsExePresent: namsExe != null && File(namsExe).existsSync(),
        launchCommandPreview: launchPreview,
      );
    } catch (_) {
      return NamsHealth(launchCommandPreview: launchPreview);
    }
  }

  static Future<List<LogEntry>> _collectRecentIssues() async {
    try {
      final entries = await LogService.readLog(AppStrings.namsLogName);
      final issues = entries
          .where((e) => e.level == 'WARN' || e.level == 'ERROR')
          .toList();
      return issues.length > 25 ? issues.sublist(issues.length - 25) : issues;
    } catch (_) {
      return const [];
    }
  }

  static Future<bool> _collectGpuPref() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return prefs.getBool(AppStrings.prefKeyPreferDedicatedGpu) ?? false;
    } catch (_) {
      return false;
    }
  }

  static List<ConfigDelta> _collectConfigDeltas(
    String namsToml,
    String lodmodToml,
    String textureInjToml,
  ) {
    final deltas = <ConfigDelta>[];
    _diffFields(deltas, 'nams.toml', namsToml, _diagNamsFields);
    _diffFields(deltas, 'lodmod.toml', lodmodToml, _diagLodmodFields);
    _diffFields(
        deltas, 'texture_injection.toml', textureInjToml, _diagTextureFields);
    return deltas;
  }

  static void _diffFields(
    List<ConfigDelta> out,
    String file,
    String raw,
    List<ConfigField> fields,
  ) {
    if (raw.isEmpty) return;
    Map<String, dynamic> parsed;
    try {
      parsed = TomlService.parse(raw);
    } catch (_) {
      return;
    }
    for (final f in fields) {
      final actual = _lookup(parsed, f.section, f.key);
      if (actual == null) continue;
      if (actual.toString() == f.defaultValue.toString()) continue;
      out.add(ConfigDelta(
        file: file,
        key: f.section == null ? f.key : '${f.section}.${f.key}',
        value: actual.toString(),
        defaultValue: f.defaultValue.toString(),
      ));
    }
  }

  static dynamic _lookup(
      Map<String, dynamic> parsed, String? section, String key) {
    if (section == null) return parsed[key];
    final parts = section.split('.');
    dynamic node = parsed;
    for (final part in parts) {
      if (node is Map && node[part] is Map) {
        node = node[part];
      } else {
        return null;
      }
    }
    return node is Map ? node[key] : null;
  }

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

  /// Strips paths and OS usernames that could identify the user.
  /// Replacement order matters: longer/more-specific paths first so they
  /// don't get shadowed by a shorter match (e.g. `<user>` swallowing the
  /// rest of a game-dir prefix).
  static String redact(String input, DiagnosticsReport r) {
    if (input.isEmpty) return input;
    var out = input;

    final replacements = <_RedactPair>[];

    final gameDir = r.launcherInfo['Game dir'] ?? '';
    if (gameDir.isNotEmpty && gameDir != '<not selected>') {
      replacements.add(_RedactPair(gameDir, '<gameDir>'));
    }
    final launcherDir = r.launcherInfo['Launcher dir'] ?? '';
    if (launcherDir.isNotEmpty) {
      replacements.add(_RedactPair(launcherDir, '<launcherDir>'));
    }

    final env = Platform.environment;
    final userProfile = env['USERPROFILE'] ?? env['HOME'] ?? '';
    if (userProfile.isNotEmpty) {
      replacements.add(_RedactPair(userProfile, '<userHome>'));
    }
    final username = env['USERNAME'] ?? env['USER'] ?? '';
    if (username.isNotEmpty && username.length >= 2) {
      replacements.add(_RedactPair(username, '<user>'));
    }

    replacements.sort((a, b) => b.from.length.compareTo(a.from.length));

    for (final pair in replacements) {
      out = out.replaceAll(pair.from, pair.to);
      final lower = pair.from.toLowerCase();
      if (lower != pair.from) {
        out = out.replaceAll(lower, pair.to);
      }
    }
    return out;
  }

  /// Compact human-friendly summary suitable for a dialog / screenshot.
  static String formatSummary(DiagnosticsReport r) {
    final b = StringBuffer();
    b.writeln('YP Launcher Diagnostics');
    b.writeln('Generated: ${r.generatedAt.toIso8601String()}');
    b.writeln('Launcher: ${r.launcherInfo['Launcher version'] ?? '?'}   '
        'OS: ${r.systemInfo['OS']} ${r.systemInfo['OS version']}');
    final gi = r.gameIdentity;
    b.writeln('Game: exe=${gi.exeVariant} '
        'supported=${gi.exeVariantSupported}   DLC=${gi.hasDlc}   '
        'running=${r.gameRunning}');
    b.writeln('NAMS: exe=${r.namsHealth.namsExePresent}   '
        'missing=${r.namsHealth.missingFiles.length}   '
        'vanilla data/ drops=${r.vanillaDropped.length}');
    final rs = r.thirdParty.reshade;
    if (rs.installed) {
      b.writeln('ReShade: enabled=${rs.enabled}   '
          'presets=${rs.presetCount}   shaders=${rs.reshadeInfo?.shaderCount ?? 0}'
          '${rs.reshadeInfo?.version != null ? "   ${rs.reshadeInfo!.version}" : ""}');
    }
    final mi = r.thirdParty.migoto;
    if (mi.installed) {
      b.writeln('3DMigoto: enabled=${mi.enabled}   '
          'fixes=${mi.migotoInfo?.shaderFixCount ?? 0}   '
          'loaderOk=${mi.migotoInfo?.loaderTargetOk ?? false}');
    }
    b.writeln('Texture packs (NAMS): '
        '${r.texturePacksAvailable ? r.texturePacks.length.toString() : "unavailable"}   '
        'Non-default settings: ${r.configDeltas.length}   '
        'Recent issues: ${r.recentLogIssues.length}');
    b.writeln('');
    b.writeln(
        'Mods (NAMS): ${r.mods.length}   '
        'Disabled entries: ${r.disabledModsEntries.length}');
    final modKinds = <ModKind, int>{};
    for (final m in r.mods) {
      modKinds[m.kind] = (modKinds[m.kind] ?? 0) + 1;
    }
    if (modKinds.isNotEmpty) {
      b.writeln('  by kind: ' +
          modKinds.entries
              .map((e) => '${e.key.name}=${e.value}')
              .join(', '));
    }
    b.writeln('Cutscene mods: ${r.cutsceneMods.length}   '
        'HD: ${r.cutsceneDetection.hasHdCutscenes}   '
        'H264: ${r.cutsceneDetection.needsH264}   '
        'data/movie overrides: ${r.directOverrides.length}');
    b.writeln(
        'Textures (nams/inject): ${r.namsTextures.length}   '
        'SK_Res: ${r.skResTextures.length}   '
        'WAX: ${r.waxTextures.length}');
    if (r.dataDirContents.isNotEmpty) {
      b.writeln('');
      b.writeln('Vanilla data/ overlay (suspicious if non-empty):');
      final entries = r.dataDirContents.entries.toList()
        ..sort((a, b) => b.value.compareTo(a.value));
      for (final e in entries.take(8)) {
        b.writeln('  ${e.key}/  (${e.value} files)');
      }
      if (entries.length > 8) {
        b.writeln('  ... ${entries.length - 8} more');
      }
    }
    if (r.waxModsRoot.isNotEmpty) {
      b.writeln('');
      b.writeln('wax/mods/: ${r.waxModsRoot.join(", ")}');
    }
    if (r.skResPacks.isNotEmpty) {
      b.writeln('SK_Res packs: ${r.skResPacks.join(", ")}');
    }
    if (r.reshadeContents.isNotEmpty) {
      b.writeln('reshade/: ${r.reshadeContents.join(", ")}');
    }
    if (r.gameRootExtras.isNotEmpty) {
      b.writeln('');
      b.writeln('Game root extras (non-vanilla): ${r.gameRootExtras.join(", ")}');
    }
    return redact(b.toString(), r);
  }

  /// Verbose multi-section dump suitable for sharing with a maintainer.
  static String formatFullReport(DiagnosticsReport r) {
    final b = StringBuffer();
    void section(String title) {
      b.writeln('');
      b.writeln('=' * 72);
      b.writeln(title);
      b.writeln('=' * 72);
    }

    b.writeln('YP Launcher Diagnostics Report');
    b.writeln('Generated: ${r.generatedAt.toIso8601String()}');
    b.writeln('');

    section('System');
    r.systemInfo.forEach((k, v) => b.writeln('$k: $v'));

    section('Launcher');
    r.launcherInfo.forEach((k, v) {
      if (k == 'Launcher dir' || k == 'Game dir') return;
      b.writeln('$k: $v');
    });

    final gi = r.gameIdentity;
    section('Game identity');
    b.writeln('exe variant: ${gi.exeVariant} '
        '(${gi.exeVariantSupported ? "supported" : "UNSUPPORTED"})');
    if (gi.exeSize != null) {
      b.writeln('exe size: ${_humanSize(gi.exeSize!)} '
          '(win10/11 build match: ${gi.exeSizeMatchesWin10})');
    }
    b.writeln('DLC (3C3C Concert & Costume): ${gi.hasDlc}');
    if (gi.exeVariant == 'legacyWindows7') {
      b.writeln('!! legacy Windows 7/8 exe — NAMS cannot host this build.');
    }

    section('NAMS runtime health');
    b.writeln('NAMS.exe present: ${r.namsHealth.namsExePresent}');
    b.writeln('Game running: ${r.gameRunning}');
    if (r.namsHealth.missingFiles.isEmpty) {
      b.writeln('Missing runtime files: none');
    } else {
      b.writeln('!! Missing runtime files (AV/quarantine?): '
          '${r.namsHealth.missingFiles.join(", ")}');
    }
    if (r.namsHealth.launchCommandPreview != null) {
      b.writeln('');
      b.writeln('Launch command:');
      b.writeln(r.namsHealth.launchCommandPreview!);
    }

    section('Vanilla data/ drops (user-added files, ${r.vanillaDropped.length})');
    if (r.vanillaDropped.isEmpty) {
      b.writeln('(none — vanilla data/ is clean)');
    } else {
      for (final f in r.vanillaDropped) {
        b.writeln('  ${f.path}  ${_humanSize(f.sizeBytes)}  [${f.bucket}]');
      }
      if (r.vanillaDropped.length >= _vanillaDropCap) {
        b.writeln('  ... (capped at $_vanillaDropCap)');
      }
    }

    section('ReShade');
    _writeReShade(b, r.thirdParty.reshade);

    section('3DMigoto');
    _writeMigoto(b, r.thirdParty.migoto);

    section('Texture packs (NAMS, structured)');
    if (!r.texturePacksAvailable) {
      b.writeln('(NAMS texture query unavailable — NAMS.exe missing or failed)');
    } else if (r.texturePacks.isEmpty) {
      b.writeln('(none)');
    } else {
      for (final t in r.texturePacks) {
        b.writeln('- ${t.name}  [${t.source}]  '
            '${t.enabled ? "enabled" : "disabled"}  ${t.ddsCount} dds'
            '${t.mod != null ? "  mod=${t.mod}" : ""}'
            '${t.character != null ? "  char=${t.character}" : ""}'
            '${t.outfitConditional ? "  outfit-conditional" : ""}'
            '${t.loadOrderIndex != null ? "  order=${t.loadOrderIndex}" : ""}');
        b.writeln('    ${t.path}');
      }
    }

    section('Non-default settings (${r.configDeltas.length})');
    if (r.configDeltas.isEmpty) {
      b.writeln('(all defaults)');
    } else {
      for (final d in r.configDeltas) {
        b.writeln('  [${d.file}] ${d.key} = ${d.value}  (default ${d.defaultValue})');
      }
    }

    section('Recent NAMS warnings/errors (${r.recentLogIssues.length})');
    if (r.recentLogIssues.isEmpty) {
      b.writeln('(none in nams.log)');
    } else {
      for (final e in r.recentLogIssues) {
        b.writeln('  ${e.timestamp} ${e.level} ${e.module}: ${e.message}');
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
      b.writeln('Totals by kind: ' +
          byKind.entries.map((e) => '${e.key.name}=${e.value}').join(', '));
      b.writeln('With warnings: $withWarnings   '
          'With bundled assets: $withBundles   '
          'Disabled prefixes affecting mods: ${_countDisabledMods(r)}');
      b.writeln('');
    }

    for (final m in r.mods) {
      final disabledByPrefix = disabledSet.any((p) =>
          'mods/${m.id}' == p || 'mods/${m.id}'.startsWith('$p/'));
      final statusTag = disabledByPrefix ? '[DISABLED]' : '[enabled]';
      final warnTag = m.hasWarnings ? '  ⚠ warnings' : '';

      b.writeln('- ${m.id}  $statusTag  [${m.kind.name}]'
          '${m.manifest?.version != null ? "  v${m.manifest!.version}" : ""}'
          '$warnTag');
      if (m.displayName.isNotEmpty && m.displayName != m.id) {
        b.writeln('    name:     ${m.displayName}');
      }
      b.writeln('    rootPath: ${m.rootPath}');
      b.writeln('    installed: ${m.installedAt.toIso8601String()}');
      if (m.manifest != null) {
        final mf = m.manifest!;
        if (mf.author != null) b.writeln('    author:   ${mf.author}');
        if (mf.namsMinVersion != null) {
          b.writeln('    NAMS min version: ${mf.namsMinVersion}');
        }
        if (mf.requires.isNotEmpty) {
          b.writeln('    requires mods:    ${mf.requires.join(", ")}');
        }
        if (mf.requiresPlugins.isNotEmpty) {
          b.writeln('    requires plugins: ${mf.requiresPlugins.join(", ")}');
        }
        if (mf.replaces.isNotEmpty) {
          b.writeln('    replaces:         ${mf.replaces.join(", ")}');
        }
      } else {
        b.writeln('    manifest: (none)');
      }

      if (m.requiresMissing.isNotEmpty) {
        b.writeln('    !! missing requires: ${m.requiresMissing.join(", ")}');
      }

      if (m.native != null) {
        final n = m.native!;
        if (n.bundlesByKind.isNotEmpty || n.totalEntityFiles > 0) {
          final bundles = n.bundlesByKind.entries
              .map((e) => '${e.key}=${e.value}')
              .join(', ');
          b.writeln('    native:   bundles=[$bundles]'
              '  entityFiles=${n.totalEntityFiles}');
        }
      }

      if (m.data != null) {
        final d = m.data!;
        if (d.entries.isNotEmpty) {
          final byCategory = <DataCategory, int>{};
          final byCategoryFiles = <DataCategory, int>{};
          for (final e in d.entries) {
            byCategory[e.category] = (byCategory[e.category] ?? 0) + 1;
            byCategoryFiles[e.category] =
                (byCategoryFiles[e.category] ?? 0) + e.fileCount;
          }
          b.writeln('    data overlay:');
          final cats = byCategory.keys.toList()
            ..sort((a, b) => a.name.compareTo(b.name));
          for (final cat in cats) {
            b.writeln('      ${cat.name}: '
                '${byCategory[cat]} dir(s), ${byCategoryFiles[cat]} file(s)');
          }
          final entryNames = d.entries.map((e) => e.dirName).toList()..sort();
          b.writeln('      dirs: ${entryNames.join(", ")}');
        }
        if (d.players.isNotEmpty) {
          b.writeln('    player models:');
          for (final p in d.players) {
            final lookup = playerModelLookup[p.fileName.split('.').first];
            final label = lookup ?? p.label;
            b.writeln('      - ${p.fileName}  →  $label');
          }
        }
        if (d.hasCompatConfig) {
          b.writeln('    compat config: yes');
        }
      }

      if (m.bundledTexturePacks.isNotEmpty) {
        b.writeln('    bundled textures (${m.bundledTexturePacks.length}): '
            '${m.bundledTexturePacks.join(", ")}');
      }
      if (m.bundledCutscenes.isNotEmpty) {
        b.writeln('    bundled cutscenes (${m.bundledCutscenes.length}): '
            '${m.bundledCutscenes.join(", ")}');
      }

      if (m.conflicts.isNotEmpty) {
        b.writeln('    conflicts (${m.conflicts.length}):');
        for (final c in m.conflicts) {
          b.writeln('      - vs ${c.otherModId} '
              '[${_humanizeConflictKind(c.kind)}] ${c.detail}');
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
      b.writeln('- ${c.name}  [$origin]  ${c.usmCount} USM'
          '${c.hasH264 ? "  H264" : "  MPEG-2"}'
          '${c.maxWidth > 0 ? "  ${c.maxWidth}x${c.maxHeight}" : ""}');
      b.writeln('    ${c.fullPath}');
      if (c.missingOriginals.isNotEmpty) {
        b.writeln(
            '    missing originals: ${c.missingOriginals.length}');
      }
    }
    if (r.directOverrides.isNotEmpty) {
      b.writeln('');
      b.writeln(
          'Custom files in data/movie/ (${r.directOverrides.length}):');
      for (final f in r.directOverrides) {
        b.writeln('  - $f');
      }
    }
    b.writeln('');
    b.writeln('hd_cutscenes flag detection: '
        'hasHd=${r.cutsceneDetection.hasHdCutscenes}, '
        'needsH264=${r.cutsceneDetection.needsH264}, '
        'filesScanned=${r.cutsceneDetection.filesScanned}, '
        'largest=${r.cutsceneDetection.largestWidth}x${r.cutsceneDetection.largestHeight}');

    section('Texture packs');
    b.writeln('nams/inject/textures/ (${r.namsTextures.length}):');
    for (final n in r.namsTextures) {
      b.writeln('  - $n');
    }
    if (r.skResTextures.isNotEmpty) {
      b.writeln('');
      b.writeln('SK_Res/inject/textures/ (${r.skResTextures.length}):');
      for (final n in r.skResTextures) {
        b.writeln('  - $n');
      }
    }
    if (r.waxTextures.isNotEmpty) {
      b.writeln('');
      b.writeln('WAX textures (${r.waxTextures.length}):');
      for (final n in r.waxTextures) {
        b.writeln('  - $n');
      }
    }

    section('Vanilla data/ overlay (manual drops)');
    if (r.dataDirContents.isEmpty) {
      b.writeln('(empty / not present)');
    } else {
      final entries = r.dataDirContents.entries.toList()
        ..sort((a, b) => b.value.compareTo(a.value));
      for (final e in entries) {
        b.writeln('  ${e.key}/  (${e.value} files)');
      }
    }

    section('wax/mods/');
    if (r.waxModsRoot.isEmpty) {
      b.writeln('(empty / not present)');
    } else {
      for (final n in r.waxModsRoot) {
        b.writeln('  - $n');
      }
    }

    section('SK_Res/inject/textures/<exe>/');
    if (r.skResPacks.isEmpty) {
      b.writeln('(empty / not present)');
    } else {
      for (final n in r.skResPacks) {
        b.writeln('  - $n');
      }
    }

    section('reshade/');
    if (r.reshadeContents.isEmpty) {
      b.writeln('(empty / not present)');
    } else {
      for (final n in r.reshadeContents) {
        b.writeln('  - $n');
      }
    }

    section('Game root extras (non-vanilla files & folders)');
    if (r.gameRootExtras.isEmpty) {
      b.writeln('(none — clean install root)');
    } else {
      for (final n in r.gameRootExtras) {
        b.writeln('  - $n');
      }
    }

    section('nams/nams.toml');
    b.writeln(r.namsTomlRaw.isEmpty ? '(missing)' : r.namsTomlRaw);

    section('nams/lodmod.toml');
    b.writeln(r.lodmodTomlRaw.isEmpty ? '(missing)' : r.lodmodTomlRaw);

    section('nams/texture_injection.toml');
    b.writeln(
        r.textureInjectionTomlRaw.isEmpty ? '(missing)' : r.textureInjectionTomlRaw);

    return redact(b.toString(), r);
  }

  static void _writeReShade(StringBuffer b, ThirdPartyRuntimeStatus s) {
    if (!s.installed) {
      b.writeln('(not installed)');
      return;
    }
    b.writeln('installed: true   enabled: ${s.enabled}   '
        'shadersMissing: ${s.shadersMissing}');
    final info = s.reshadeInfo;
    if (info != null) {
      if (info.version != null) b.writeln('version: ${info.version}');
      if (info.dllName != null) {
        b.writeln('dll: ${info.dllName}   addonBuild: ${info.isAddonBuild}');
      }
      b.writeln('presets (${info.presets.length}): ${info.presets.join(", ")}');
      b.writeln('shader repos (${info.shaderRepos.length}): '
          '${info.shaderRepos.join(", ")}   effects: ${info.shaderCount}');
      if (info.addons.isNotEmpty) {
        b.writeln('addons (${info.addons.length}): ${info.addons.join(", ")}');
      }
      if (info.d3dCompilerMissing) {
        b.writeln('!! d3dcompiler_47.dll missing (Wine shaders will not compile)');
      }
      final c = info.config;
      b.writeln('config: performanceMode=${c.performanceMode} '
          'showFps=${c.showFps} showClock=${c.showClock}'
          '${c.activePreset != null ? " activePreset=${c.activePreset}" : ""}');
    }
  }

  static void _writeMigoto(StringBuffer b, ThirdPartyRuntimeStatus s) {
    if (!s.installed) {
      b.writeln('(not installed)');
      return;
    }
    b.writeln('installed: true   enabled: ${s.enabled}   '
        'hasShaderFixes: ${s.hasShaderFixes}');
    final info = s.migotoInfo;
    if (info != null) {
      b.writeln('files: ${info.files.join(", ")}');
      b.writeln('loader target: ${info.loaderTarget ?? "?"} '
          '(ok: ${info.loaderTargetOk})');
      b.writeln('shader fixes: ${info.shaderFixCount}');
      if (info.shaderFixNames.isNotEmpty) {
        b.writeln('  from: ${info.shaderFixNames.join(", ")}');
      }
      final c = info.config;
      b.writeln('config: hunting=${c.hunting.name} marking=${c.markingMode.name} '
          'cacheShaders=${c.cacheShaders} verboseOverlay=${c.verboseOverlay}');
    }
  }

  /// Writes the full report to `<launcherLogsDir>/diagnostics_<timestamp>.txt`
  /// and returns the resulting path.
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

class _RedactPair {
  final String from;
  final String to;
  const _RedactPair(this.from, this.to);
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

const _vanillaDataDirs = <String>{
  'movie',
  'enlighten',
  'sound',
  'movie_logo',
};

const _vanillaGameRootEntries = <String>{
  'data',
  'wallpaper',
  'redist',
  'logs',
  'nierautomata.exe',
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
  final skBase = Directory(
    p.join(gameDir, 'SK_Res', 'inject', 'textures'),
  );
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

final List<ConfigField> _diagNamsFields = <ConfigField>[
  NamsFields.validateModelData,
  NamsFields.validateScripts,
  NamsFields.loadingStallHints,
  NamsFields.fixWindTimerBug,
  NamsFields.disablePluginLoading,
  NamsFields.disableContentFeatures,
  NamsFields.contentItems,
  NamsFields.contentAccessories,
  NamsFields.contentAssembleMeshes,
  NamsFields.contentQuestIntegration,
  NamsFields.contentEffectsApplier,
  NamsFields.contentEquipTracker,
  NamsFields.contentMcd,
  NamsFields.contentBuddyRubySelector,
  NamsFields.outfitSwapVisualEffects,
  NamsFields.experimentalDefaultOutfits,
  NamsFields.disableReShadeLoading,
  NamsFields.disable3dmigotoLoading,
  NamsFields.disableTextureInjection,
  NamsFields.disableSplashScreen,
  NamsFields.fixCameraAcceleration,
  NamsFields.sensitivity,
  NamsFields.thirdPersonMode,
  NamsFields.aimMode,
  NamsFields.miscDisablePodPet,
  NamsFields.miscOpenDebugMenu,
  NamsFields.globalHeapExtra,
  NamsFields.plFileHeapExtra,
  NamsFields.plVramHeapExtra,
  NamsFields.emBgFileHeapExtra,
  NamsFields.emBgVramHeapExtra,
];

final List<ConfigField> _diagLodmodFields = <ConfigField>[
  LodModFields.enabled,
  LodModFields.lodMultiplier,
  LodModFields.disableManualCulling,
  LodModFields.disableVignette,
  LodModFields.shadowResolution,
  LodModFields.shadowModelHq,
  LodModFields.shadowModelForceAll,
  LodModFields.giEnabled,
  LodModFields.fpsUncapInMenus,
  LodModFields.fpsUncapInGameplay,
  LodModFields.fpsLimit,
];

final List<ConfigField> _diagTextureFields = <ConfigField>[
  TextureInjectionFields.vramBudgetMb,
  TextureInjectionFields.streamingEnabled,
  TextureInjectionFields.loadOnlyRelevant,
];

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
    findings.add(VanillaDirFinding(
      path: 'data/$name',
      sizeBytes: _lengthQuiet(entity),
      bucket: 'data-root-loose',
    ));
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
