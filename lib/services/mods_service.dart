import 'dart:io';
import 'dart:convert';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/detection_service.dart';
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
      final hasDlc = await DetectionService.hasDlc(gameDir);
      final activeProfile = ModProfilesService.activeNameSync(gameDir);
      final params = _InstallParams(
        gameDir: gameDir,
        workDir: workDir,
        requestedName: requestedName,
        sourceFolderName: path.basenameWithoutExtension(sourcePath),
        hasDlc: hasDlc,
        activeProfile: activeProfile,
      );
      return await IsolateService.run(_installSync, params);
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
    final hasDlc = await DetectionService.hasDlc(gameDir);
    return IsolateService.run(
      _syncDlcSlotsSync,
      _SyncDlcParams(gameDir: gameDir, hasDlc: hasDlc),
    );
  }
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
  const _InstallParams({
    required this.gameDir,
    required this.workDir,
    required this.requestedName,
    required this.sourceFolderName,
    required this.hasDlc,
    required this.activeProfile,
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
    bundledTexturePacks: _readBundledTexturesSidecar(rootPath),
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

  if (hasEntities && hasCompat) return ModKind.unknown;
  if (hasEntities) return ModKind.native;
  if (hasData || hasCompat || hasLooseDataDir || hasLooseDataFiles) {
    return ModKind.data;
  }
  return ModKind.unknown;
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

  if (kind == ModKind.unknown) {
    final hasEntities = _entitiesHasContent(Directory(path.join(unwrapped, 'entities')));
    final hasCompat = _hasCompatConfig(unwrapped);
    errorReason = (hasEntities && hasCompat) ? 'invalid_mixed' : 'unknown_drop';
  }

  if (kind == ModKind.native) {
    native = _scanNative(Directory(path.join(unwrapped, 'entities')));
  }
  final dataDir = Directory(path.join(unwrapped, 'data'));
  final hasCompat = _hasCompatConfig(unwrapped);
  if (dataDir.existsSync() || hasCompat) {
    data = _scanData(dataDir, hasCompatConfig: hasCompat);
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
  if (detect.kind == ModKind.unknown) {
    return InstallResult.fail(detect.errorReason ?? 'unknown_drop');
  }

  String workRoot = detect.unwrappedRoot;
  _normalizeLooseDataDirs(workRoot);
  _normalizeLooseDataFiles(workRoot);

  final modsRoot = ModsService.modsDir(p.gameDir);
  final targetId = _sanitizeId(p.requestedName?.isNotEmpty == true
      ? p.requestedName!
      : (detect.manifest?.id ?? p.sourceFolderName));
  final targetDir = Directory(path.join(modsRoot, targetId));
  if (targetDir.existsSync()) {
    return InstallResult.fail('exists:$targetId');
  }

  final sideTextures = _extractSideTexturePacks(
    workRoot,
    p.gameDir,
    targetId,
    p.activeProfile,
  );

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
      final hasRecognised = entries.any((e) => e.category != DataCategory.other);
      if (!hasRecognised && !hasCompat && entries.isEmpty) {
        return const InstallResult.fail('data_empty');
      }
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

  if (!p.hasDlc) {
    _renameDlcSlotsToVanilla(targetDir.path);
  }

  if (sideTextures.isNotEmpty) {
    _writeBundledTexturesSidecar(targetDir.path, sideTextures);
  }

  return InstallResult.ok(targetId, sideInstalledTexturePacks: sideTextures);
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

void _writeBundledTexturesSidecar(String modDir, List<String> packNames) {
  try {
    final file = File(path.join(modDir, _bundledTexturesSidecarName));
    file.writeAsStringSync(jsonEncode({'packs': packNames}));
  } catch (_) {}
}

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

/// Mods sometimes ship a Special-K-style texture overlay alongside their
/// real NAMS payload (`SK_Res/inject/textures/NieRAutomata.exe/<pack>/...`)
/// or a modern `inject/textures/<pack>/...` layout. We split those texture
/// packs into `<gameDir>/nams/inject/textures/<pack>/` so they actually load
/// instead of being buried inside the mod folder.
///
/// Returns the names of the packs we moved.
List<String> _extractSideTexturePacks(
  String workRoot,
  String gameDir,
  String modId,
  String activeProfile,
) {
  final moved = <String>[];
  final injectRoot =
      Directory(path.join(gameDir, 'nams', 'inject', 'textures'));

  void movePack(Directory pack) {
    final name = path.basename(pack.path);
    if (name.isEmpty || name.startsWith('.') || name.startsWith('_')) return;

    var finalName = '$name ($modId@$activeProfile)';
    var dest = Directory(path.join(injectRoot.path, finalName));
    var n = 2;
    while (dest.existsSync()) {
      finalName = '$name ($modId@$activeProfile $n)';
      dest = Directory(path.join(injectRoot.path, finalName));
      n++;
    }
    injectRoot.createSync(recursive: true);
    try {
      pack.renameSync(dest.path);
      moved.add(finalName);
    } catch (_) {
      try {
        _copyDirectorySync(pack, dest);
        pack.deleteSync(recursive: true);
        moved.add(finalName);
      } catch (_) {}
    }
  }

  // Layout 1: SK_Res/inject/textures/<exe>/<pack>/...
  final skRes = Directory(path.join(workRoot, 'SK_Res', 'inject', 'textures'));
  if (skRes.existsSync()) {
    for (final exeDir in skRes.listSync().whereType<Directory>()) {
      for (final pack in exeDir.listSync().whereType<Directory>()) {
        movePack(pack);
      }
    }
    try {
      Directory(path.join(workRoot, 'SK_Res')).deleteSync(recursive: true);
    } catch (_) {}
  }

  // Layout 2: inject/textures/<pack>/... at the workRoot.
  final inject = Directory(path.join(workRoot, 'inject', 'textures'));
  if (inject.existsSync()) {
    for (final pack in inject.listSync().whereType<Directory>()) {
      movePack(pack);
    }
    try {
      Directory(path.join(workRoot, 'inject')).deleteSync(recursive: true);
    } catch (_) {}
  }

  return moved;
}

void _copyDirectorySync(Directory src, Directory dest) {
  dest.createSync(recursive: true);
  for (final entity in src.listSync(recursive: true, followLinks: false)) {
    final rel = path.relative(entity.path, from: src.path);
    final destPath = path.join(dest.path, rel);
    if (entity is Directory) {
      Directory(destPath).createSync(recursive: true);
    } else if (entity is File) {
      Directory(path.dirname(destPath)).createSync(recursive: true);
      entity.copySync(destPath);
    }
  }
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
