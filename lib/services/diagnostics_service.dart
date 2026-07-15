import 'dart:io';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/cutscene_detection_service.dart';
import 'package:yp_launcher/services/disabled_mods_service.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_isolates.dart';

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
  });
}

class DiagnosticsService {
  DiagnosticsService._();

  static Future<DiagnosticsReport> collect(String gameDir) async {
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

    return DiagnosticsReport(
      generatedAt: DateTime.now(),
      gameDir: gameDir,
      systemInfo: {
        'OS': Platform.operatingSystem,
        'OS version': Platform.operatingSystemVersion,
        'Locale': Platform.localeName,
        'Dart version': Platform.version,
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
    );
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
