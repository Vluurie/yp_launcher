import 'dart:io';
import 'dart:convert';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/mod_manifest_service.dart';
import 'package:yp_launcher/services/mod_profiles_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

const _compatConfigSubdirs = {
  'item_info', 'outfits', 'accessories', 'hairsprays',
  'behaviors', 'mcd', 'shops', 'inventory', 'weapons',
};

class ModsService {
  ModsService._();

  static String modsDir(String gameDir) =>
      path.join(gameDir, 'nams', 'mods');

  static Future<List<TexturePack>> detectTexturePacks(String root) {
    return IsolateService.run(_detectTexturePacksParam, root);
  }

  static Future<List<InstalledMod>> listInstalled(String gameDir) {
    return IsolateService.run(
      _listInstalledSync,
      _ListInstalledParams(gameDir: gameDir, modsRoot: modsDir(gameDir)),
    );
  }

  static Future<DetectedDrop> detectDrop(
    String sourcePath, {
    void Function(double percent, String? currentFile)? onExtractProgress,
  }) async {
    String workDir = sourcePath;
    String? tempDir;
    if (ArchiveService.isArchive(sourcePath)) {
      final extracted = await ArchiveService.extract(
        sourcePath,
        onProgress: onExtractProgress,
      );
      if (extracted == null) {
        return DetectedDrop(
          unwrappedRoot: sourcePath,
          kind: ModKind.unknown,
          suggestedId: '',
          errorReason: 'archive_extract_failed',
        );
      }
      tempDir = extracted;
      workDir = extracted;
    }
    final params = _DetectParams(
      workDir: workDir,
      sourceBaseName: path.basenameWithoutExtension(sourcePath),
    );
    final result = await IsolateService.run(_detectDropSync, params);
    if (tempDir != null) {
      try { Directory(tempDir).deleteSync(recursive: true); } catch (_) {}
    }
    return result;
  }

  static Future<InstallResult> install(
    String gameDir,
    String sourcePath, {
    String? requestedName,
    String? variantSubPath,
    List<String>? texturePackSubPaths,
    void Function(double percent, String? currentFile)? onExtractProgress,
  }) async {
    String workDir = sourcePath;
    String? tempDir;
    if (ArchiveService.isArchive(sourcePath)) {
      final extracted = await ArchiveService.extract(
        sourcePath,
        onProgress: onExtractProgress,
      );
      if (extracted == null) {
        return const InstallResult.fail('archive_extract_failed');
      }
      tempDir = extracted;
      workDir = extracted;
    }
    try {
      final hasDlc = await GameDetection.hasDlc(gameDir);
      final activeProfile = ModProfilesService.activeNameSync(gameDir);
      final params = _InstallParams(
        gameDir: gameDir,
        workDir: workDir,
        requestedName: requestedName,
        sourceFolderName: path.basenameWithoutExtension(sourcePath),
        hasDlc: hasDlc,
        activeProfile: activeProfile,
        variantSubPath: variantSubPath,
        texturePackSubPaths: texturePackSubPaths,
      );
      return await IsolateService.run(_installSync, params);
    } finally {
      if (tempDir != null) {
        try { Directory(tempDir).deleteSync(recursive: true); } catch (_) {}
      }
    }
  }

  static Future<List<InstallResult>> installVariants(
    String gameDir,
    String sourcePath,
    List<VariantInstallRequest> requests, {
    void Function(double percent, String? currentFile)? onExtractProgress,
  }) async {
    String workDir = sourcePath;
    String? tempDir;
    if (ArchiveService.isArchive(sourcePath)) {
      final extracted = await ArchiveService.extract(
        sourcePath,
        onProgress: onExtractProgress,
      );
      if (extracted == null) {
        return [
          for (final _ in requests)
            const InstallResult.fail('archive_extract_failed')
        ];
      }
      tempDir = extracted;
      workDir = extracted;
    }
    try {
      final hasDlc = await GameDetection.hasDlc(gameDir);
      final activeProfile = ModProfilesService.activeNameSync(gameDir);
      final params = _InstallBatchParams(
        gameDir: gameDir,
        workDir: workDir,
        sourceFolderName: path.basenameWithoutExtension(sourcePath),
        hasDlc: hasDlc,
        activeProfile: activeProfile,
        requests: requests,
      );
      return await IsolateService.run(_installBatchSync, params);
    } finally {
      if (tempDir != null) {
        try { Directory(tempDir).deleteSync(recursive: true); } catch (_) {}
      }
    }
  }

  static Future<void> uninstall(
    String gameDir,
    String modId, {
    bool deleteBundledTextures = false,
  }) {
    return IsolateService.run(
      _uninstallSync,
      _UninstallParams(gameDir, modId, deleteBundledTextures),
    );
  }

  static Future<int> syncDlcSlots(String gameDir) async {
    final hasDlc = await GameDetection.hasDlc(gameDir);
    return IsolateService.run(
      _syncDlcSlotsSync,
      _SyncDlcParams(gameDir: gameDir, hasDlc: hasDlc),
    );
  }

  static Future<int> migrateBundledTexturesInPlace(String gameDir) {
    final activeProfile = ModProfilesService.activeNameSync(gameDir);
    return IsolateService.run(
      _migrateBundledTexturesSync,
      _MigrateTexturesParams(gameDir: gameDir, activeProfile: activeProfile),
    );
  }
}

class _MigrateTexturesParams {
  final String gameDir;
  final String activeProfile;
  const _MigrateTexturesParams({
    required this.gameDir,
    required this.activeProfile,
  });
}

final _pulledOutPackPattern = RegExp(r'^(.*) \(([^()@]+)@([^()]+?)( \d+)?\)$');

int _migrateBundledTexturesSync(_MigrateTexturesParams p) {
  final injectRoot =
      Directory(path.join(p.gameDir, 'nams', 'inject', 'textures'));
  if (!injectRoot.existsSync()) return 0;

  final migratedNames = <String>[];
  for (final entity in injectRoot.listSync()) {
    if (entity is! Directory) continue;
    final packName = path.basename(entity.path);
    final m = _pulledOutPackPattern.firstMatch(packName);
    if (m == null) continue;
    final base = m.group(1)!;
    final modId = m.group(2)!;
    final profile = m.group(3)!;

    final modRoot = profile == p.activeProfile
        ? path.join(p.gameDir, 'nams', 'mods', modId)
        : path.join(p.gameDir, 'nams', 'mods_profile_$profile', modId);
    if (!Directory(modRoot).existsSync()) continue;

    final destDir = Directory(path.join(modRoot, 'textures'));
    var dest = path.join(destDir.path, base);
    if (Directory(dest).existsSync() || File(dest).existsSync()) {
      dest = path.join(destDir.path, packName);
      if (Directory(dest).existsSync()) continue;
    }
    try {
      destDir.createSync(recursive: true);
      _moveDirectory(entity.path, dest);
    } catch (_) {
      continue;
    }
    migratedNames.add(packName);

    try {
      final sidecar = File(path.join(modRoot, _bundledTexturesSidecarName));
      if (sidecar.existsSync()) {
        final raw = jsonDecode(sidecar.readAsStringSync());
        if (raw is Map && raw['packs'] is List) {
          final remaining = (raw['packs'] as List)
              .whereType<String>()
              .where((s) => s != packName)
              .toList();
          if (remaining.isEmpty) {
            sidecar.deleteSync();
          } else {
            sidecar.writeAsStringSync(jsonEncode({'packs': remaining}));
          }
        }
      }
    } catch (_) {}
  }

  if (migratedNames.isEmpty) return 0;

  try {
    final tomlFile =
        File(path.join(p.gameDir, 'nams', 'texture_injection.toml'));
    if (tomlFile.existsSync()) {
      final raw = tomlFile.readAsStringSync();
      final parsed = TomlService.parse(raw);
      final updates = <String, dynamic>{};
      for (final key in ['load_order', 'disabled_packs']) {
        final list = parsed[key];
        if (list is List) {
          final pruned = list
              .whereType<String>()
              .where((s) => !migratedNames.contains(s))
              .toList();
          if (pruned.length != list.length) updates[key] = pruned;
        }
      }
      if (updates.isNotEmpty) {
        tomlFile.writeAsStringSync(TomlService.updateToml(raw, updates));
      }
    }
  } catch (_) {}

  return migratedNames.length;
}

class _DetectParams {
  final String workDir;
  final String sourceBaseName;
  const _DetectParams({required this.workDir, required this.sourceBaseName});
}

class _InstallParams {
  final String gameDir;
  final String workDir;
  final String? requestedName;
  final String sourceFolderName;
  final bool hasDlc;
  final String activeProfile;
  final String? variantSubPath;
  final List<String>? texturePackSubPaths;
  const _InstallParams({
    required this.gameDir,
    required this.workDir,
    required this.requestedName,
    required this.sourceFolderName,
    required this.hasDlc,
    required this.activeProfile,
    this.variantSubPath,
    this.texturePackSubPaths,
  });
}

class _InstallBatchParams {
  final String gameDir;
  final String workDir;
  final String sourceFolderName;
  final bool hasDlc;
  final String activeProfile;
  final List<VariantInstallRequest> requests;
  const _InstallBatchParams({
    required this.gameDir,
    required this.workDir,
    required this.sourceFolderName,
    required this.hasDlc,
    required this.activeProfile,
    required this.requests,
  });
}

class _ListInstalledParams {
  final String gameDir;
  final String modsRoot;
  const _ListInstalledParams({required this.gameDir, required this.modsRoot});
}

class _UninstallParams {
  final String gameDir;
  final String modId;
  final bool deleteBundledTextures;
  const _UninstallParams(
    this.gameDir,
    this.modId,
    this.deleteBundledTextures,
  );
}

class _SyncDlcParams {
  final String gameDir;
  final bool hasDlc;
  const _SyncDlcParams({required this.gameDir, required this.hasDlc});
}

int _syncDlcSlotsSync(_SyncDlcParams p) {
  final modsDir = Directory(ModsService.modsDir(p.gameDir));
  if (!modsDir.existsSync()) return 0;
  var touched = 0;
  for (final entity in modsDir.listSync()) {
    if (entity is! Directory) continue;
    final id = path.basename(entity.path);
    if (id.startsWith('.') || id.startsWith('_')) continue;
    if (_syncOneModDlcSlots(entity.path, hasDlc: p.hasDlc)) touched++;
  }
  return touched;
}

bool _syncOneModDlcSlots(String modRoot, {required bool hasDlc}) {
  final sidecar = File(path.join(modRoot, _dlcRenamesSidecarName));
  Map<String, String> renames = {};
  if (sidecar.existsSync()) {
    try {
      final raw = jsonDecode(sidecar.readAsStringSync());
      if (raw is Map) {
        renames = raw.map((k, v) => MapEntry(k.toString(), v.toString()));
      }
    } catch (_) {}
  }

  if (hasDlc) {
    final reverted = renames.isEmpty
        ? false
        : _revertRenamesToDlc(modRoot, renames, sidecar);
    final inferred = _applyRenamesToDlc(modRoot, sidecar);
    return reverted || inferred;
  } else {
    return _applyRenamesToVanilla(modRoot, renames, sidecar);
  }
}

bool _revertRenamesToDlc(
  String modRoot,
  Map<String, String> renames,
  File sidecar,
) {
  var changed = false;
  final remaining = <String, String>{};
  renames.forEach((vanillaRel, dlcName) {
    final vanillaAbs = path.join(modRoot, vanillaRel.replaceAll('/', path.separator));
    final dirName = path.dirname(vanillaAbs);
    final dlcAbs = path.join(dirName, dlcName);
    if (!File(vanillaAbs).existsSync()) {
      changed = true;
      return;
    }
    if (File(dlcAbs).existsSync()) {
      remaining[vanillaRel] = dlcName;
      return;
    }
    try {
      File(vanillaAbs).renameSync(dlcAbs);
      changed = true;
    } catch (_) {
      remaining[vanillaRel] = dlcName;
    }
  });
  if (changed) {
    if (remaining.isEmpty) {
      try {
        sidecar.deleteSync();
      } catch (_) {}
    } else {
      try {
        sidecar.writeAsStringSync(jsonEncode(remaining));
      } catch (_) {}
    }
  }
  return changed;
}

bool _applyRenamesToDlc(String modRoot, File sidecar) {
  final vanillaToDlc = <String, String>{
    for (final e in _dlcToVanillaSlot.entries) e.value: e.key,
  };

  final dir = Directory(modRoot);
  if (!dir.existsSync()) return false;

  final renames = <String, String>{};
  if (sidecar.existsSync()) {
    try {
      final raw = jsonDecode(sidecar.readAsStringSync());
      if (raw is Map) {
        renames.addAll(raw.map((k, v) => MapEntry(k.toString(), v.toString())));
      }
    } catch (_) {}
  }

  var changed = false;
  for (final entity in dir.listSync(recursive: true, followLinks: false)) {
    if (entity is! File) continue;
    final fileName = path.basename(entity.path);
    String? newName;
    for (final entry in vanillaToDlc.entries) {
      if (fileName.startsWith(entry.key)) {
        newName = entry.value + fileName.substring(entry.key.length);
        break;
      }
    }
    if (newName == null) continue;

    final dirName = path.dirname(entity.path);
    final target = path.join(dirName, newName);
    if (File(target).existsSync()) continue;

    try {
      entity.renameSync(target);
      final relDir = path.relative(dirName, from: modRoot);
      final key = relDir == '.' ? newName : '$relDir/$newName';
      renames[key.replaceAll('\\', '/')] = fileName;
      changed = true;
    } catch (_) {}
  }

  if (changed) {
    try {
      sidecar.writeAsStringSync(jsonEncode(renames));
    } catch (_) {}
  }
  return changed;
}

bool _applyRenamesToVanilla(
  String modRoot,
  Map<String, String> existingRenames,
  File sidecar,
) {
  var changed = false;
  final renames = Map<String, String>.from(existingRenames);
  final dir = Directory(modRoot);
  if (!dir.existsSync()) return false;

  for (final entity in dir.listSync(recursive: true, followLinks: false)) {
    if (entity is! File) continue;
    final fileName = path.basename(entity.path);
    String? newName;
    for (final entry in _dlcToVanillaSlot.entries) {
      if (fileName.startsWith(entry.key)) {
        newName = entry.value + fileName.substring(entry.key.length);
        break;
      }
    }
    if (newName == null) continue;

    final dirName = path.dirname(entity.path);
    final target = path.join(dirName, newName);
    if (File(target).existsSync()) continue;

    try {
      entity.renameSync(target);
      final relDir = path.relative(dirName, from: modRoot);
      final key = relDir == '.' ? newName : '$relDir/$newName';
      renames[key.replaceAll('\\', '/')] = fileName;
      changed = true;
    } catch (_) {}
  }

  if (changed) {
    try {
      sidecar.writeAsStringSync(jsonEncode(renames));
    } catch (_) {}
  }
  return changed;
}

List<InstalledMod> _listInstalledSync(_ListInstalledParams p) {
  final dir = Directory(p.modsRoot);
  if (!dir.existsSync()) return const [];

  final mods = <InstalledMod>[];
  for (final entity in dir.listSync()) {
    if (entity is! Directory) continue;
    final id = path.basename(entity.path);
    if (id.startsWith('.') || id.startsWith('_')) continue;
    final mod = _scanInstalledMod(entity.path, id, p.gameDir);
    if (mod != null) mods.add(mod);
  }

  mods.sort((a, b) => b.installedAt.compareTo(a.installedAt));
  return _computeConflicts(mods);
}

InstalledMod? _scanInstalledMod(String rootPath, String id, String gameDir) {
  final contentRoot = _unwrapSingleChild(rootPath);
  final manifest = ModManifestService.loadSync(contentRoot);
  final kind = _classifyKind(contentRoot);

  final bundledTextures = _readBundledTexturesSidecar(rootPath);

  NativeSummary? native;
  DataSummary? data;

  if (kind == ModKind.native) {
    native = _scanNative(Directory(path.join(contentRoot, 'entities')));
  }

  final dataDir = Directory(path.join(contentRoot, 'data'));
  final hasCompat = _hasCompatConfig(contentRoot);
  if (dataDir.existsSync() || hasCompat) {
    data = _scanData(dataDir, hasCompatConfig: hasCompat);
  }

  DateTime installedAt;
  try {
    installedAt = Directory(rootPath).statSync().modified;
  } catch (_) {
    installedAt = DateTime.fromMillisecondsSinceEpoch(0);
  }

  return InstalledMod(
    id: id,
    displayName: manifest?.displayName ?? id,
    rootPath: rootPath,
    kind: kind,
    manifest: manifest,
    native: native,
    data: data,
    installedAt: installedAt,
    bundledTexturePacks: bundledTextures,
    bundledCutscenes: _scanBundledCutscenes(rootPath),
  );
}

String _unwrapSingleChild(String rootPath) {
  var current = rootPath;
  for (var i = 0; i < 6; i++) {
    final dir = Directory(current);
    if (!dir.existsSync()) return current;
    final entries = dir.listSync();
    final dirs = entries.whereType<Directory>().toList();
    if (dirs.length != 1 || entries.length != 1) return current;
    if (Directory(path.join(current, 'entities')).existsSync()) return current;
    if (Directory(path.join(current, 'wax')).existsSync()) return current;
    if (Directory(path.join(current, 'data')).existsSync()) return current;
    if (File(path.join(current, 'mod.toml')).existsSync()) return current;
    if (dataDirCategoryTable.containsKey(
        path.basename(dirs.single.path).toLowerCase())) {
      return current;
    }
    current = dirs.single.path;
  }
  return current;
}

ModKind _classifyKind(String contentRoot) {
  final hasEntities = _entitiesHasContent(Directory(path.join(contentRoot, 'entities')));
  final hasCompat = _hasCompatConfig(contentRoot);
  final hasData = _dataHasSubdirs(Directory(path.join(contentRoot, 'data')));
  final hasLooseDataDir = _hasLooseDataDir(contentRoot);
  final hasLooseDataFiles = _hasLooseDataFiles(contentRoot);
  final hasCpk = _hasCpk(contentRoot);

  if (hasEntities && hasCompat) return ModKind.unknown;
  if (hasEntities) return ModKind.native;
  if (hasData || hasCompat || hasLooseDataDir || hasLooseDataFiles || hasCpk) {
    return ModKind.data;
  }
  return ModKind.unknown;
}

bool _hasNamsPayload(String root) {
  if (_entitiesHasContent(Directory(path.join(root, 'entities')))) return true;
  if (Directory(path.join(root, 'wax')).existsSync()) return true;
  if (_dataHasSubdirs(Directory(path.join(root, 'data')))) return true;
  if (_hasCompatConfig(root)) return true;
  if (_hasLooseDataDir(root)) return true;
  if (_hasLooseDataFiles(root)) return true;
  if (_hasCpk(root)) return true;
  return false;
}

bool _dirHasDds(Directory dir) {
  if (!dir.existsSync()) return false;
  for (final f in dir.listSync(recursive: true, followLinks: false)) {
    if (f is File && f.path.toLowerCase().endsWith('.dds')) return true;
  }
  return false;
}

bool _dirHasDirectDds(Directory dir) {
  if (!dir.existsSync()) return false;
  return dir
      .listSync()
      .whereType<File>()
      .any((f) => f.path.toLowerCase().endsWith('.dds'));
}

bool _isTexturePackRoot(String root) {
  if (_hasNamsPayload(root)) return false;
  for (final resName in const ['SK_Res', 'FAR_Res']) {
    if (Directory(path.join(root, resName, 'inject', 'textures')).existsSync()) {
      return true;
    }
  }
  if (Directory(path.join(root, 'inject', 'textures')).existsSync()) return true;
  if (Directory(path.join(root, 'textures')).existsSync() &&
      _dirHasDds(Directory(path.join(root, 'textures')))) {
    return true;
  }
  if (_dirHasDirectDds(Directory(root))) return true;
  return _dirHasDds(Directory(root));
}

ModKind _installableKindAt(String root) {
  return _classifyKind(root);
}

List<ModVariant> _detectVariants(String root) {
  final out = <ModVariant>[];
  _collectVariants(root, root, '', 0, out);
  out.sort((a, b) => a.label.toLowerCase().compareTo(b.label.toLowerCase()));
  return out;
}

void _collectVariants(
  String baseRoot,
  String current,
  String labelPrefix,
  int depth,
  List<ModVariant> out,
) {
  if (depth > 4) return;
  final dir = Directory(current);
  if (!dir.existsSync()) return;
  for (final sub in dir.listSync().whereType<Directory>()) {
    final name = path.basename(sub.path);
    if (name.startsWith('.') || name.startsWith('_')) continue;
    final unwrapped = _unwrapSingleChild(sub.path);
    final label = labelPrefix.isEmpty ? name : '$labelPrefix / $name';
    final kind = _installableKindAt(unwrapped);
    if (kind != ModKind.unknown) {
      final textureOnly =
          !_hasNamsPayload(unwrapped) && _isTexturePackRoot(unwrapped);
      out.add(ModVariant(
        subPath: path.relative(unwrapped, from: baseRoot),
        label: label,
        kind: kind,
        textureOnly: textureOnly,
        category: _primaryDataCategoryAt(unwrapped),
      ));
    } else {
      _collectVariants(baseRoot, unwrapped, label, depth + 1, out);
    }
  }
}

DataCategory? _primaryDataCategoryAt(String contentRoot) {
  final counts = <DataCategory, int>{};
  void bump(String dirKey, int n) {
    final cat = dataDirCategoryTable[dirKey.toLowerCase()];
    if (cat != null) counts[cat] = (counts[cat] ?? 0) + n;
  }

  for (final base in [contentRoot, path.join(contentRoot, 'data')]) {
    final dir = Directory(base);
    if (!dir.existsSync()) continue;
    for (final entity in dir.listSync().whereType<Directory>()) {
      final name = path.basename(entity.path);
      final files = entity
          .listSync(recursive: true, followLinks: false)
          .whereType<File>()
          .length;
      bump(name, files > 0 ? files : 1);
    }
    for (final f in dir.listSync().whereType<File>()) {
      final key = _dataDirForLooseFile(path.basename(f.path));
      if (key != null) bump(key, 1);
    }
  }

  if (counts.isEmpty) return null;
  var best = counts.entries.first;
  for (final e in counts.entries) {
    if (e.value > best.value) best = e;
  }
  return best.key;
}

bool _hasLooseDataDir(String contentRoot) {
  final dir = Directory(contentRoot);
  if (!dir.existsSync()) return false;
  for (final entity in dir.listSync().whereType<Directory>()) {
    final name = path.basename(entity.path).toLowerCase();
    if (dataDirCategoryTable.containsKey(name)) return true;
  }
  return false;
}

bool _hasLooseDataFiles(String contentRoot) {
  final dir = Directory(contentRoot);
  if (!dir.existsSync()) return false;
  for (final f in dir.listSync().whereType<File>()) {
    if (_dataDirForLooseFile(path.basename(f.path)) != null) return true;
  }
  return false;
}

bool _isCpkFile(String fileName) => fileName.toLowerCase().endsWith('.cpk');

bool _hasCpk(String contentRoot) {
  for (final base in [contentRoot, path.join(contentRoot, 'data')]) {
    final dir = Directory(base);
    if (!dir.existsSync()) continue;
    if (dir
        .listSync()
        .whereType<File>()
        .any((f) => _isCpkFile(path.basename(f.path)))) {
      return true;
    }
  }
  return false;
}

String? _dataDirForLooseFile(String fileName) {
  final lower = fileName.toLowerCase();
  if (!lower.endsWith('.dat') && !lower.endsWith('.dtt')) return null;
  final stem = lower.substring(0, lower.length - 4);
  String? best;
  for (final key in dataDirCategoryTable.keys) {
    if (stem.startsWith(key)) {
      if (best == null || key.length > best.length) best = key;
    }
  }
  return best;
}

bool _entitiesHasContent(Directory dir) {
  if (!dir.existsSync()) return false;
  for (final f in dir.listSync(recursive: true, followLinks: false)) {
    if (f is! File) continue;
    final lower = f.path.toLowerCase();
    if (lower.endsWith('.toml') || lower.endsWith('.json')) return true;
  }
  return false;
}

bool _hasCompatConfig(String contentRoot) {
  final wrappedRoot = Directory(path.join(contentRoot, 'wax', 'mods'));
  if (wrappedRoot.existsSync()) {
    for (final inner in wrappedRoot.listSync().whereType<Directory>()) {
      if (_hasCompatSubdirAt(inner.path)) return true;
    }
  }
  return _hasCompatSubdirAt(contentRoot);
}

bool _hasCompatSubdirAt(String root) {
  for (final sub in _compatConfigSubdirs) {
    final d = Directory(path.join(root, sub));
    if (!d.existsSync()) continue;
    final hasJson = d.listSync().whereType<File>().any((f) => f.path.toLowerCase().endsWith('.json'));
    if (hasJson) return true;
  }
  return false;
}

bool _dataHasSubdirs(Directory dir) {
  if (!dir.existsSync()) return false;
  // Only accept data/ that contains at least one NAMS-recognized subdir
  // (pl, wp, em, core, ph1, ...). A bare data/ with random contents (e.g.
  // a Flutter app bundle's data/flutter_assets/) is NOT a NAMS data overlay.
  for (final entity in dir.listSync().whereType<Directory>()) {
    final name = path.basename(entity.path).toLowerCase();
    if (dataDirCategoryTable.containsKey(name)) return true;
  }
  return false;
}

NativeSummary _scanNative(Directory entitiesDir) {
  if (!entitiesDir.existsSync()) return const NativeSummary();
  final counts = <String, int>{};
  int total = 0;
  for (final f in entitiesDir.listSync(recursive: true, followLinks: false).whereType<File>()) {
    final lower = f.path.toLowerCase();
    final isToml = lower.endsWith('.toml');
    final isJson = lower.endsWith('.json');
    if (!isToml && !isJson) continue;
    total++;
    final kind = _peekBundleKind(f, isToml: isToml);
    final bucket = kind ?? 'unknown';
    counts[bucket] = (counts[bucket] ?? 0) + 1;
  }
  return NativeSummary(bundlesByKind: Map.unmodifiable(counts), totalEntityFiles: total);
}

String? _peekBundleKind(File file, {required bool isToml}) {
  final fromName = _bundleKindFromFileName(path.basename(file.path));
  if (fromName != null) return fromName;

  if (!isToml) {
    try {
      final raw = file.readAsStringSync();
      final decoded = jsonDecode(raw);
      if (decoded is Map && decoded['kind'] is String) return decoded['kind'] as String;
      if (decoded is List && decoded.isNotEmpty && decoded.first is Map) {
        final first = decoded.first as Map;
        if (first['kind'] is String) return first['kind'] as String;
      }
    } catch (_) {}
  }
  return null;
}

String? _bundleKindFromFileName(String fileName) {
  final stem = fileName
      .toLowerCase()
      .replaceFirst(RegExp(r'\.(toml|json)$'), '');
  if (stem.isEmpty) return null;
  final stripped = stem.replaceFirst(RegExp(r'_[a-z0-9]+$'), '');
  if (stripped.isEmpty || stripped == stem) return null;
  return '${stripped}_bundle';
}

int _countAssetEntries(Directory dir) {
  final stems = <String>{};
  int other = 0;
  for (final f in dir.listSync(recursive: true, followLinks: false).whereType<File>()) {
    final base = path.basename(f.path);
    final lower = base.toLowerCase();
    if (lower.endsWith('.dat') || lower.endsWith('.dtt')) {
      final dirRel = path.dirname(f.path);
      stems.add('$dirRel/${base.substring(0, base.length - 4)}');
    } else {
      other++;
    }
  }
  return stems.length + other;
}

DataSummary _scanData(Directory dataDir, {required bool hasCompatConfig}) {
  final entries = <DataDirEntry>[];
  final players = <PlayerModelEntry>[];

  if (dataDir.existsSync()) {
    final cpks = dataDir
        .listSync()
        .whereType<File>()
        .where((f) => _isCpkFile(path.basename(f.path)))
        .toList();
    if (cpks.isNotEmpty) {
      entries.add(DataDirEntry(
        dirName: 'cpk',
        category: DataCategory.archive,
        fileCount: cpks.length,
      ));
    }

    for (final sub in dataDir.listSync().whereType<Directory>()) {
      final dirName = path.basename(sub.path);
      final cat = dataDirCategoryTable[dirName.toLowerCase()] ?? DataCategory.other;
      final fileCount = _countAssetEntries(sub);
      entries.add(DataDirEntry(dirName: dirName, category: cat, fileCount: fileCount));

      if (cat == DataCategory.player3d) {
        for (final f in sub.listSync().whereType<File>()) {
          final base = path.basename(f.path);
          final lower = base.toLowerCase();
          if (!lower.endsWith('.dat')) continue;
          final stem = base.substring(0, base.length - 4).toLowerCase();
          final label = playerModelLookup[stem] ?? base;
          players.add(PlayerModelEntry(fileName: base, label: label));
        }
      }
    }
  }

  entries.sort((a, b) => a.dirName.compareTo(b.dirName));
  players.sort((a, b) => a.fileName.compareTo(b.fileName));
  return DataSummary(entries: entries, players: players, hasCompatConfig: hasCompatConfig);
}

List<InstalledMod> _computeConflicts(List<InstalledMod> mods) {
  final dataFilesByMod = <String, Set<String>>{};
  for (final m in mods) {
    dataFilesByMod[m.id] = _collectDataRelPaths(m.rootPath);
  }
  final installedIds = mods.map((m) => m.id).toSet();

  final result = <InstalledMod>[];
  for (final m in mods) {
    final conflicts = <ModConflict>[];
    final myFiles = dataFilesByMod[m.id]!;
    for (final other in mods) {
      if (other.id == m.id) continue;
      final overlap = myFiles.intersection(dataFilesByMod[other.id]!);
      for (final rel in overlap) {
        conflicts.add(ModConflict(
          otherModId: other.id,
          kind: ModConflictKind.overlappingDataFile,
          detail: rel,
        ));
      }
    }

    final missing = (m.manifest?.requires ?? const <String>[])
        .where((id) => !installedIds.contains(id))
        .toList();

    result.add(InstalledMod(
      id: m.id,
      displayName: m.displayName,
      rootPath: m.rootPath,
      kind: m.kind,
      installedAt: m.installedAt,
      manifest: m.manifest,
      native: m.native,
      data: m.data,
      requiresMissing: missing,
      conflicts: conflicts,
      bundledTexturePacks: m.bundledTexturePacks,
      bundledCutscenes: m.bundledCutscenes,
    ));
  }
  return result;
}

const _conditionallyMountedDirs = {'pl'};

Set<String> _collectDataRelPaths(String rootPath) {
  final contentRoot = _unwrapSingleChild(rootPath);
  final dataDir = Directory(path.join(contentRoot, 'data'));
  if (!dataDir.existsSync()) return const {};
  final base = dataDir.path;
  final files = <String>{};
  for (final f in dataDir.listSync(recursive: true, followLinks: false).whereType<File>()) {
    final rel = path.relative(f.path, from: base).replaceAll('\\', '/');
    final firstSegment = rel.split('/').first.toLowerCase();
    if (_conditionallyMountedDirs.contains(firstSegment)) continue;
    files.add('data/$rel');
  }
  return files;
}

DetectedDrop _detectDropSync(_DetectParams p) {
  final unwrapped = _unwrapSingleChild(p.workDir);
  final manifest = ModManifestService.loadSync(unwrapped);
  final kind = _classifyKind(unwrapped);

  NativeSummary? native;
  DataSummary? data;
  String? errorReason;
  var variants = const <ModVariant>[];

  if (kind == ModKind.unknown) {
    variants = _detectVariants(unwrapped);
    if (variants.isEmpty) {
      if (!_hasNamsPayload(unwrapped) && _isTexturePackRoot(unwrapped)) {
        errorReason = 'texture_only';
      } else {
        final hasEntities = _entitiesHasContent(Directory(path.join(unwrapped, 'entities')));
        final hasCompat = _hasCompatConfig(unwrapped);
        errorReason = (hasEntities && hasCompat) ? 'invalid_mixed' : 'unknown_drop';
      }
    }
  }

  if (kind == ModKind.native) {
    native = _scanNative(Directory(path.join(unwrapped, 'entities')));
  }
  final dataDir = Directory(path.join(unwrapped, 'data'));
  final hasCompat = _hasCompatConfig(unwrapped);
  if (dataDir.existsSync() || hasCompat) {
    data = _scanData(dataDir, hasCompatConfig: hasCompat);
  }

  var textureVariants = const <TexturePack>[];
  {
    final packs = _detectTexturePacks(unwrapped, ignorePayload: true);
    if (packs.length > 1) {
      textureVariants = packs;
    }
  }

  return DetectedDrop(
    unwrappedRoot: unwrapped,
    kind: kind,
    manifest: manifest,
    native: native,
    data: data,
    suggestedId: _suggestId(
      manifest: manifest,
      sourceBaseName: p.sourceBaseName,
      unwrappedRoot: unwrapped,
    ),
    errorReason: errorReason,
    variants: variants,
    textureVariants: textureVariants,
  );
}

String _suggestId({
  required ManifestInfo? manifest,
  required String sourceBaseName,
  required String unwrappedRoot,
}) {
  if (manifest?.id != null && manifest!.id!.trim().isNotEmpty) {
    return _sanitizeId(manifest.id!);
  }
  final raw = _firstUsableName(
    candidates: [sourceBaseName, path.basename(unwrappedRoot)],
    unwrappedRoot: unwrappedRoot,
  );
  return _sanitizeId(raw);
}

String _firstUsableName({
  required List<String> candidates,
  required String unwrappedRoot,
}) {
  for (final c in candidates) {
    if (_isUsableName(c)) return c;
  }
  final fromPlayers = _nameFromPlayerFiles(unwrappedRoot);
  if (fromPlayers != null) return fromPlayers;
  for (final c in candidates) {
    if (c.trim().isNotEmpty) return c;
  }
  return 'mod';
}

bool _isUsableName(String s) {
  final t = s.trim();
  if (t.length < 3) return false;
  final lower = t.toLowerCase();
  if (dataDirCategoryTable.containsKey(lower)) return false;
  if (lower == 'data' || lower == 'mod' || lower == 'mods') return false;
  if (RegExp(r'^archive[_-]?[a-f0-9]{4,}$', caseSensitive: false).hasMatch(t)) return false;
  if (RegExp(r'^[a-f0-9]{6,}$', caseSensitive: false).hasMatch(t)) return false;
  return true;
}

String? _nameFromPlayerFiles(String root) {
  final dir = Directory(root);
  if (!dir.existsSync()) return null;
  final scanRoots = <Directory>[
    dir,
    Directory(path.join(root, 'data', 'pl')),
    Directory(path.join(root, 'pl')),
  ];
  final stems = <String>{};
  for (final r in scanRoots) {
    if (!r.existsSync()) continue;
    for (final f in r.listSync().whereType<File>()) {
      final base = path.basename(f.path).toLowerCase();
      if (!base.startsWith('pl')) continue;
      if (!base.endsWith('.dat') && !base.endsWith('.dtt')) continue;
      final stem = base.substring(0, base.length - 4);
      if (stem.length >= 6) stems.add(stem.substring(0, 6));
    }
  }
  if (stems.isEmpty) return null;
  final labels = <String>{};
  for (final s in stems) {
    final label = playerModelLookup[s];
    if (label != null) labels.add(label);
  }
  if (labels.isEmpty) return null;
  return labels.join(' & ');
}

String _sanitizeId(String raw) {
  final lower = raw.toLowerCase().trim();
  final sb = StringBuffer();
  for (final r in lower.runes) {
    final c = String.fromCharCode(r);
    if (RegExp(r'[a-z0-9]').hasMatch(c)) {
      sb.write(c);
    } else if (RegExp(r'[\s\-_().,]').hasMatch(c)) {
      sb.write('_');
    }
  }
  final cleaned = sb.toString().replaceAll(RegExp(r'_+'), '_').replaceAll(RegExp(r'^_|_$'), '');
  return cleaned.isEmpty ? 'mod' : cleaned;
}

InstallResult _installSync(_InstallParams p) {
  final detectParams = _DetectParams(
    workDir: p.workDir,
    sourceBaseName: p.sourceFolderName,
  );
  final detect = _detectDropSync(detectParams);

  final dropRoot = detect.unwrappedRoot;
  String workRoot = dropRoot;
  if (p.variantSubPath != null && p.variantSubPath!.isNotEmpty) {
    final resolved = path.normalize(path.join(workRoot, p.variantSubPath!));
    if (!Directory(resolved).existsSync()) {
      return const InstallResult.fail('variant_missing');
    }
    workRoot = resolved;
  } else if (detect.kind == ModKind.unknown) {
    return InstallResult.fail(detect.errorReason ?? 'unknown_drop');
  }

  _normalizeLooseDataDirs(workRoot);
  _normalizeLooseDataFiles(workRoot);
  _normalizeCpks(workRoot);

  final modsRoot = ModsService.modsDir(p.gameDir);
  final targetId = _sanitizeId(p.requestedName?.isNotEmpty == true
      ? p.requestedName!
      : (detect.manifest?.id ?? p.sourceFolderName));
  final targetDir = Directory(path.join(modsRoot, targetId));
  if (targetDir.existsSync()) {
    return InstallResult.fail('exists:$targetId');
  }

  if (!_hasNamsPayload(workRoot) && _isTexturePackRoot(workRoot)) {
    return const InstallResult.fail('texture_only');
  }

  final detect2 = _detectDropSync(_DetectParams(
    workDir: workRoot,
    sourceBaseName: p.sourceFolderName,
  ));

  if (detect2.kind == ModKind.unknown) {
    return InstallResult.fail(detect2.errorReason ?? 'unknown_drop');
  }
  switch (detect2.kind) {
    case ModKind.native:
      if ((detect2.native?.totalEntityFiles ?? 0) == 0) {
        return const InstallResult.fail('native_empty');
      }
      break;
    case ModKind.data:
      final entries = detect2.data?.entries ?? const [];
      final hasCompat = detect2.data?.hasCompatConfig ?? false;
      final hasRecognised =
          entries.any((e) => e.category != DataCategory.other);
      if (!hasRecognised &&
          !hasCompat &&
          entries.isEmpty &&
          !_hasCpk(workRoot)) {
        return const InstallResult.fail('data_empty');
      }
      break;
    case ModKind.texture:
      break;
    case ModKind.unknown:
      return const InstallResult.fail('unknown_drop');
  }

  Directory(modsRoot).createSync(recursive: true);
  try {
    _moveDirectory(workRoot, targetDir.path);
  } catch (e) {
    return InstallResult.fail('move_failed:$e');
  }

  final localSubPaths = _stageSiblingTextureSets(
    dropRoot,
    workRoot,
    targetDir.path,
    p.texturePackSubPaths,
  );

  _normalizeInPlaceTextures(targetDir.path, localSubPaths);

  if (!p.hasDlc) {
    _renameDlcSlotsToVanilla(targetDir.path);
  }

  return InstallResult.ok(targetId);
}

List<InstallResult> _installBatchSync(_InstallBatchParams p) {
  final detect = _detectDropSync(_DetectParams(
    workDir: p.workDir,
    sourceBaseName: p.sourceFolderName,
  ));
  final dropRoot = detect.unwrappedRoot;
  final modsRoot = ModsService.modsDir(p.gameDir);
  Directory(modsRoot).createSync(recursive: true);

  final results = <InstallResult>[];
  final usedIds = <String>{};

  for (final req in p.requests) {
    final variantRoot = path.normalize(path.join(dropRoot, req.variantSubPath));
    if (!Directory(variantRoot).existsSync()) {
      results.add(const InstallResult.fail('variant_missing'));
      continue;
    }

    var targetId = _sanitizeId(req.requestedName);
    var attempt = 1;
    while (usedIds.contains(targetId) ||
        Directory(path.join(modsRoot, targetId)).existsSync()) {
      attempt++;
      targetId = _sanitizeId('${req.requestedName} $attempt');
    }
    final targetDir = Directory(path.join(modsRoot, targetId));

    try {
      _copyDirectory(variantRoot, targetDir.path);
    } catch (e) {
      results.add(InstallResult.fail('move_failed:$e'));
      continue;
    }

    _normalizeLooseDataDirs(targetDir.path);
    _normalizeLooseDataFiles(targetDir.path);
    _normalizeCpks(targetDir.path);

    final localSubPaths = _stageSiblingTextureSets(
      dropRoot,
      variantRoot,
      targetDir.path,
      req.texturePackSubPaths,
    );
    _normalizeInPlaceTextures(targetDir.path, localSubPaths);

    if (!p.hasDlc) {
      _renameDlcSlotsToVanilla(targetDir.path);
    }

    usedIds.add(targetId);
    results.add(InstallResult.ok(targetId));
  }

  return results;
}

/// Consolidates a mod's inject textures into a single `<mod>/textures/` dir so
/// NAMS can scan one well-known location. Strips `SK_Res`/`FAR_Res`/`inject`/
/// `<exe>` wrappers and flattens the .dds files. When the mod shipped several
/// texture sets (variants), only the one named by [chosenSubPath] is kept; the
/// rest are removed. Leaves the NAMS payload (data/entities/wax) untouched.
List<String>? _stageSiblingTextureSets(
  String dropRoot,
  String variantRoot,
  String targetDir,
  List<String>? chosenSubPaths,
) {
  if (chosenSubPaths == null || chosenSubPaths.isEmpty) return chosenSubPaths;

  final local = <String>[];
  for (final sub in chosenSubPaths) {
    final abs = path.normalize(path.join(dropRoot, sub));
    if (path.equals(abs, variantRoot) || path.isWithin(variantRoot, abs)) {
      final rel = path.relative(abs, from: variantRoot);
      local.add(rel);
      continue;
    }
    if (!Directory(abs).existsSync()) continue;
    final destName = _uniqueChildName(targetDir, path.basename(abs));
    final dest = path.join(targetDir, destName);
    _copyDirectory(abs, dest);
    local.add(destName);
  }
  return local;
}

String _uniqueChildName(String parent, String name) {
  var candidate = name;
  var n = 2;
  while (Directory(path.join(parent, candidate)).existsSync() ||
      File(path.join(parent, candidate)).existsSync()) {
    candidate = '$name ($n)';
    n++;
  }
  return candidate;
}

void _copyDirectory(String src, String dest) {
  final srcDir = Directory(src);
  Directory(dest).createSync(recursive: true);
  for (final entity in srcDir.listSync(recursive: true, followLinks: false)) {
    final rel = path.relative(entity.path, from: src);
    final target = path.join(dest, rel);
    if (entity is Directory) {
      Directory(target).createSync(recursive: true);
    } else if (entity is File) {
      Directory(path.dirname(target)).createSync(recursive: true);
      entity.copySync(target);
    }
  }
}

void _normalizeInPlaceTextures(String modRoot, List<String>? chosenSubPaths) {
  final packs = _detectTexturePacks(modRoot, ignorePayload: true);
  if (packs.isEmpty) return;

  final texturesDir = Directory(path.join(modRoot, 'textures'));

  List<TexturePack> selected;
  if (chosenSubPaths != null && chosenSubPaths.isNotEmpty) {
    final targets = chosenSubPaths
        .map((s) => path.normalize(path.join(modRoot, s)))
        .toList();
    selected = packs
        .where((pk) => targets.any((t) => path.equals(pk.path, t)))
        .toList();
    if (selected.isEmpty) selected = packs;
  } else {
    selected = packs;
  }

  // Move each selected pack's dds into <mod>/textures/, wrapper-stripped.
  for (final pack in selected) {
    final packDir = Directory(pack.path);
    if (!packDir.existsSync()) continue;
    if (path.equals(packDir.path, texturesDir.path)) continue;
    for (final f in packDir.listSync(recursive: true, followLinks: false)) {
      if (f is! File) continue;
      if (!f.path.toLowerCase().endsWith('.dds')) continue;
      final dest = File(path.join(texturesDir.path, path.basename(f.path)));
      if (dest.existsSync()) continue;
      try {
        dest.parent.createSync(recursive: true);
        f.renameSync(dest.path);
      } catch (_) {
        try {
          f.copySync(dest.path);
        } catch (_) {}
      }
    }
  }

  // Remove every wrapper/variant dir that isn't the canonical textures/.
  for (final name in const ['SK_Res', 'FAR_Res', 'inject']) {
    _deleteDirQuiet(path.join(modRoot, name));
  }
  for (final pack in packs) {
    final packDir = Directory(pack.path);
    if (!packDir.existsSync()) continue;
    if (path.equals(packDir.path, texturesDir.path)) continue;
    if (path.equals(path.dirname(packDir.path), modRoot)) {
      _deleteDirQuiet(packDir.path);
    }
  }
}

void _deleteDirQuiet(String dir) {
  final d = Directory(dir);
  if (d.existsSync()) {
    try {
      d.deleteSync(recursive: true);
    } catch (_) {}
  }
}

const Map<String, String> _dlcToVanillaSlot = {
  'pl000d': 'pl0000',
  'pl010d': 'pl0100',
  'pl020d': 'pl0200',
  'plf00d': 'plf000',
  'plf10d': 'plf100',
  'plf20d': 'plf200',
};
const _dlcRenamesSidecarName = '.dlc_renames.json';

/// Records relPath → originalFileName for every file we renamed away from a
/// DLC slot. Lets `syncDlcSlots` revert when the user later installs the DLC.
void _renameDlcSlotsToVanilla(String modRoot) {
  final renames = <String, String>{};
  try {
    final dir = Directory(modRoot);
    if (!dir.existsSync()) return;
    for (final entity in dir.listSync(recursive: true, followLinks: false)) {
      if (entity is! File) continue;
      final fileName = path.basename(entity.path);
      String? newName;
      for (final entry in _dlcToVanillaSlot.entries) {
        if (fileName.startsWith(entry.key)) {
          newName = entry.value + fileName.substring(entry.key.length);
          break;
        }
      }
      if (newName == null) continue;
      final dirName = path.dirname(entity.path);
      final target = path.join(dirName, newName);
      if (File(target).existsSync()) continue;
      try {
        entity.renameSync(target);
        final relDir = path.relative(dirName, from: modRoot);
        final key = relDir == '.' ? newName : '$relDir/$newName';
        renames[key.replaceAll('\\', '/')] = fileName;
      } catch (_) {}
    }
  } catch (_) {}
  if (renames.isNotEmpty) {
    try {
      File(path.join(modRoot, _dlcRenamesSidecarName))
          .writeAsStringSync(jsonEncode(renames));
    } catch (_) {}
  }
}

const _bundledTexturesSidecarName = '.bundled_textures.json';

List<String> _scanBundledCutscenes(String modDir) {
  try {
    final dir = Directory(path.join(modDir, 'cutscenes'));
    if (!dir.existsSync()) return const [];
    final out = <String>[];
    for (final entity in dir.listSync()) {
      if (entity is! Directory) continue;
      final name = path.basename(entity.path);
      if (name.startsWith('.') || name.startsWith('_')) continue;
      final movie = Directory(path.join(entity.path, 'movie'));
      if (!movie.existsSync()) continue;
      final hasUsm = movie
          .listSync()
          .whereType<File>()
          .any((f) => f.path.toLowerCase().endsWith('.usm'));
      if (hasUsm) out.add(name);
    }
    out.sort();
    return out;
  } catch (_) {}
  return const [];
}

List<String> _readBundledTexturesSidecar(String modDir) {
  final names = <String>[];
  try {
    final file = File(path.join(modDir, _bundledTexturesSidecarName));
    if (!file.existsSync()) return const [];
    final raw = file.readAsStringSync();
    final decoded = jsonDecode(raw);
    if (decoded is Map && decoded['packs'] is List) {
      for (final s in (decoded['packs'] as List).whereType<String>()) {
        if (s.isNotEmpty) names.add(s);
      }
    }
  } catch (_) {}
  return names;
}

const _textureWrapperDirs = {
  'sk_res', 'far_res', 'inject', 'textures', 'nierautomata.exe',
};

bool _isTextureWrapper(String name) =>
    _textureWrapperDirs.contains(name.toLowerCase());

const _modPayloadDirs = {'data', 'entities', 'wax', 'cutscenes'};

List<TexturePack> _detectTexturePacksParam(String root) =>
    _detectTexturePacks(root);

List<TexturePack> _detectTexturePacks(String root, {bool ignorePayload = false}) {
  final packs = <TexturePack>[];
  final seen = <String>{};

  void addPack(String dir, String label) {
    if (seen.add(dir)) {
      packs.add(TexturePack(path: dir, label: label));
    }
  }

  void scan(Directory dir, String? namedParent) {
    if (!dir.existsSync()) return;
    final name = path.basename(dir.path);
    final label = _isTextureWrapper(name) ? namedParent : name;

    if (_dirHasDirectDds(dir)) {
      addPack(dir.path, label ?? path.basename(root));
      return;
    }

    final ddsSubdirs =
        dir.listSync().whereType<Directory>().where(_dirHasDds).toList();
    if (ddsSubdirs.isEmpty) return;

    final namedChildren = ddsSubdirs
        .where((d) => !_isTextureWrapper(path.basename(d.path)))
        .toList();
    final wrapperChildren = ddsSubdirs
        .where((d) => _isTextureWrapper(path.basename(d.path)))
        .toList();

    if (namedChildren.isEmpty && label != null) {
      final deeperNamed = wrapperChildren.any((w) => w
          .listSync()
          .whereType<Directory>()
          .any((d) => _dirHasDds(d) && !_isTextureWrapper(path.basename(d.path))));
      if (!deeperNamed) {
        addPack(dir.path, label);
        return;
      }
    }

    for (final sub in ddsSubdirs) {
      scan(sub, label);
    }
  }

  if (!ignorePayload && _hasNamsPayload(root)) return const [];
  final rootDir = Directory(root);
  if (!ignorePayload && _dirHasDirectDds(rootDir)) {
    addPack(root, path.basename(root));
  } else {
    for (final child in rootDir.listSync().whereType<Directory>()) {
      final name = path.basename(child.path);
      if (name.startsWith('.') || name.startsWith('_')) continue;
      if (ignorePayload && _modPayloadDirs.contains(name.toLowerCase())) {
        continue;
      }
      scan(child, null);
    }
  }

  packs.sort((a, b) => a.label.toLowerCase().compareTo(b.label.toLowerCase()));
  return packs;
}

void _normalizeLooseDataDirs(String root) {
  final dir = Directory(root);
  if (!dir.existsSync()) return;
  if (Directory(path.join(root, 'entities')).existsSync()) return;
  if (Directory(path.join(root, 'wax')).existsSync()) return;

  final loose = <Directory>[];
  for (final entity in dir.listSync().whereType<Directory>()) {
    final name = path.basename(entity.path).toLowerCase();
    if (dataDirCategoryTable.containsKey(name)) {
      loose.add(entity);
    }
  }
  if (loose.isEmpty) return;

  final dataDir = Directory(path.join(root, 'data'));
  dataDir.createSync(recursive: true);
  for (final src in loose) {
    final target = path.join(dataDir.path, path.basename(src.path));
    src.renameSync(target);
  }
}

void _normalizeLooseDataFiles(String root) {
  final dir = Directory(root);
  if (!dir.existsSync()) return;
  final moves = <File, String>{};
  for (final f in dir.listSync().whereType<File>()) {
    final name = path.basename(f.path);
    final target = _dataDirForLooseFile(name);
    if (target == null) continue;
    moves[f] = target;
  }
  if (moves.isEmpty) return;
  for (final entry in moves.entries) {
    final destDir = Directory(path.join(root, 'data', entry.value));
    destDir.createSync(recursive: true);
    entry.key.renameSync(path.join(destDir.path, path.basename(entry.key.path)));
  }
}

void _normalizeCpks(String root) {
  final dir = Directory(root);
  if (!dir.existsSync()) return;
  final loose = dir
      .listSync()
      .whereType<File>()
      .where((f) => _isCpkFile(path.basename(f.path)))
      .toList();
  if (loose.isEmpty) return;

  final dataDir = Directory(path.join(root, 'data'));
  dataDir.createSync(recursive: true);
  for (final f in loose) {
    final dest = path.join(dataDir.path, path.basename(f.path));
    if (File(dest).existsSync()) continue;
    try {
      f.renameSync(dest);
    } catch (_) {
      try {
        f.copySync(dest);
        f.deleteSync();
      } catch (_) {}
    }
  }
}

void _moveDirectory(String src, String dest) {
  try {
    Directory(src).renameSync(dest);
    return;
  } catch (_) {}
  Directory(dest).createSync(recursive: true);
  for (final entity in Directory(src).listSync(recursive: true, followLinks: false)) {
    final rel = path.relative(entity.path, from: src);
    final destPath = path.join(dest, rel);
    if (entity is Directory) {
      Directory(destPath).createSync(recursive: true);
    } else if (entity is File) {
      Directory(path.dirname(destPath)).createSync(recursive: true);
      entity.copySync(destPath);
    }
  }
  try { Directory(src).deleteSync(recursive: true); } catch (_) {}
}

void _uninstallSync(_UninstallParams p) {
  final dir = Directory(path.join(ModsService.modsDir(p.gameDir), p.modId));
  if (!dir.existsSync()) return;

  final packs = p.deleteBundledTextures
      ? _readBundledTexturesSidecar(dir.path)
      : const <String>[];

  dir.deleteSync(recursive: true);

  if (packs.isEmpty) return;

  final injectRoot =
      Directory(path.join(p.gameDir, 'nams', 'inject', 'textures'));
  for (final name in packs) {
    final pack = Directory(path.join(injectRoot.path, name));
    if (pack.existsSync()) {
      try {
        pack.deleteSync(recursive: true);
      } catch (_) {}
    }
  }

  // Prune from texture_injection.toml `load_order` and `disabled_packs`.
  final tomlPath =
      path.join(p.gameDir, 'nams', 'texture_injection.toml');
  final tomlFile = File(tomlPath);
  if (!tomlFile.existsSync()) return;
  try {
    final raw = tomlFile.readAsStringSync();
    final parsed = TomlService.parse(raw);
    final updates = <String, dynamic>{};

    final loadOrder = parsed['load_order'];
    if (loadOrder is List) {
      final pruned = loadOrder
          .whereType<String>()
          .where((s) => !packs.contains(s))
          .toList();
      if (pruned.length != loadOrder.length) {
        updates['load_order'] = pruned;
      }
    }

    final disabledPacks = parsed['disabled_packs'];
    if (disabledPacks is List) {
      final pruned = disabledPacks
          .whereType<String>()
          .where((s) => !packs.contains(s))
          .toList();
      if (pruned.length != disabledPacks.length) {
        updates['disabled_packs'] = pruned;
      }
    }

    if (updates.isEmpty) return;
    final newRaw = TomlService.updateToml(raw, updates);
    tomlFile.writeAsStringSync(newRaw);
  } catch (_) {}
}
