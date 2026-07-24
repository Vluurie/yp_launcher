import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:toml/toml.dart';
import 'package:yp_launcher/models/mod_profile.dart';
import 'package:yp_launcher/services/isolate_service.dart';

class ProfileSwitchException implements Exception {
  final String message;
  ProfileSwitchException(this.message);
  @override
  String toString() => 'ProfileSwitchException: $message';
}

class ModProfilesService {
  ModProfilesService._();

  static const String _profilesTomlName = 'profiles.toml';
  static const String _disabledModsTomlName = 'disabled_mods.toml';
  static const String _defaultModsTomlName = 'default_mods.toml';
  static const String _loadOrderTomlName = 'mod_load_order.toml';
  static const String _textureInjectionTomlName = 'texture_injection.toml';
  static const String _bundledTexturesSidecar = '.bundled_textures.json';
  static const String _profilePrefix = 'mods_profile_';
  static const String _disabledProfilePrefix = 'disabled_mods_profile_';
  static const String _defaultProfilePrefix = 'default_mods_profile_';
  static const String _loadOrderProfilePrefix = 'mod_load_order_profile_';
  static const String defaultProfileName = 'default';

  static Future<ModProfileState> loadProfiles(String gameDir) {
    return IsolateService.run(_loadProfilesSync, gameDir);
  }

  static Future<ModProfileState> createProfile(
    String gameDir,
    String name,
  ) {
    return IsolateService.run(
      _createProfileSync,
      _CreateParams(gameDir: gameDir, name: name),
    );
  }

  static Future<ModProfileState> switchProfile(
    String gameDir,
    String to,
  ) {
    return IsolateService.run(
      _switchProfileSync,
      _SwitchParams(gameDir: gameDir, to: to),
    );
  }

  static Future<ModProfileState> deleteProfile(
    String gameDir,
    String name,
  ) {
    return IsolateService.run(
      _deleteProfileSync,
      _CreateParams(gameDir: gameDir, name: name),
    );
  }

  static Future<ModProfileState> renameProfile(
    String gameDir,
    String oldName,
    String newName,
  ) {
    return IsolateService.run(
      _renameProfileSync,
      _RenameParams(gameDir: gameDir, oldName: oldName, newName: newName),
    );
  }

  /// Sync read of the active profile name. Used at install time inside the
  /// existing install isolate where firing a separate isolate hop would be
  /// awkward. Returns 'default' if profiles.toml doesn't exist yet.
  static String activeNameSync(String gameDir) {
    try {
      final file = File(path.join(gameDir, 'nams', _profilesTomlName));
      if (!file.existsSync()) return defaultProfileName;
      final content = file.readAsStringSync();
      final parsed = TomlDocument.parse(content).toMap();
      final active = parsed['active'];
      if (active is String && active.isNotEmpty) return active;
    } catch (_) {}
    return defaultProfileName;
  }

  static String sanitizeName(String raw) {
    var s = raw.trim().toLowerCase();
    s = s.replaceAll(RegExp(r'\s+'), '_');
    s = s.replaceAll(RegExp(r'[^a-z0-9_-]'), '_');
    s = s.replaceAll(RegExp(r'^[._]+'), '');
    s = s.replaceAll(RegExp(r'_+'), '_');
    if (s.length > 48) s = s.substring(0, 48);
    return s;
  }

  static bool isValidName(String sanitized) {
    if (sanitized.isEmpty) return false;
    if (sanitized == 'mods') return false;
    return RegExp(r'^[a-z0-9][a-z0-9_-]*$').hasMatch(sanitized);
  }
}

class _CreateParams {
  final String gameDir;
  final String name;
  const _CreateParams({required this.gameDir, required this.name});
}

class _SwitchParams {
  final String gameDir;
  final String to;
  const _SwitchParams({required this.gameDir, required this.to});
}

class _RenameParams {
  final String gameDir;
  final String oldName;
  final String newName;
  const _RenameParams({
    required this.gameDir,
    required this.oldName,
    required this.newName,
  });
}

String _profilesTomlPath(String gameDir) =>
    path.join(gameDir, 'nams', ModProfilesService._profilesTomlName);

String _textureInjectionTomlPath(String gameDir) =>
    path.join(gameDir, 'nams', ModProfilesService._textureInjectionTomlName);

String _modsActiveDir(String gameDir) => path.join(gameDir, 'nams', 'mods');

void _backupOrphan(String gameDir, String from) {
  final orphan = Directory(_modsInactiveDir(gameDir, from));
  final orphanSidecars = _sidecars
      .map((s) => MapEntry(s, File(s.inactivePath(gameDir, from))))
      .where((e) => e.value.existsSync())
      .toList();
  if (!orphan.existsSync() && orphanSidecars.isEmpty) return;

  final backupRoot = Directory(path.join(gameDir, 'nams', '.orphan_backup'));
  if (!backupRoot.existsSync()) backupRoot.createSync(recursive: true);

  if (orphan.existsSync()) {
    var n = 1;
    var dest = path.join(backupRoot.path, '${_profilePrefixName(from)}_$n');
    while (Directory(dest).existsSync()) {
      n++;
      dest = path.join(backupRoot.path, '${_profilePrefixName(from)}_$n');
    }
    orphan.renameSync(dest);
  }
  for (final entry in orphanSidecars) {
    var n = 1;
    var dest =
        path.join(backupRoot.path, '${entry.key.backupPrefix}_${from}_$n.toml');
    while (File(dest).existsSync()) {
      n++;
      dest = path.join(
          backupRoot.path, '${entry.key.backupPrefix}_${from}_$n.toml');
    }
    entry.value.renameSync(dest);
  }
}

String _profilePrefixName(String name) =>
    '${ModProfilesService._profilePrefix}$name';

String _modsInactiveDir(String gameDir, String name) =>
    path.join(gameDir, 'nams', '${ModProfilesService._profilePrefix}$name');

/// A per-profile config file that travels with `mods/` when profiles switch.
///
/// Each one lives at `nams/<activeName>` while its profile is active and is
/// parked as `nams/<prefix><profile>.toml` while it is not. Adding another
/// per-profile config means adding an entry here.
class _Sidecar {
  final String activeName;
  final String inactivePrefix;
  final String backupPrefix;

  const _Sidecar({
    required this.activeName,
    required this.inactivePrefix,
    required this.backupPrefix,
  });

  String activePath(String gameDir) => path.join(gameDir, 'nams', activeName);

  String inactivePath(String gameDir, String profile) =>
      path.join(gameDir, 'nams', '$inactivePrefix$profile.toml');
}

const _sidecars = <_Sidecar>[
  _Sidecar(
    activeName: ModProfilesService._disabledModsTomlName,
    inactivePrefix: ModProfilesService._disabledProfilePrefix,
    backupPrefix: 'disabled',
  ),
  _Sidecar(
    activeName: ModProfilesService._defaultModsTomlName,
    inactivePrefix: ModProfilesService._defaultProfilePrefix,
    backupPrefix: 'default_mods',
  ),
  _Sidecar(
    activeName: ModProfilesService._loadOrderTomlName,
    inactivePrefix: ModProfilesService._loadOrderProfilePrefix,
    backupPrefix: 'load_order',
  ),
];

/// Moves every sidecar that exists, returning the ones actually moved so a
/// failing caller can put them back.
List<_Sidecar> _moveSidecars(
  String Function(_Sidecar) from,
  String Function(_Sidecar) to,
) {
  final moved = <_Sidecar>[];
  for (final s in _sidecars) {
    final src = File(from(s));
    if (!src.existsSync()) continue;
    try {
      src.renameSync(to(s));
      moved.add(s);
    } catch (e) {
      _restoreSidecars(moved, to, from);
      throw ProfileSwitchException('sidecar_move_failed:${s.activeName}:$e');
    }
  }
  return moved;
}

void _restoreSidecars(
  List<_Sidecar> moved,
  String Function(_Sidecar) from,
  String Function(_Sidecar) to,
) {
  for (final s in moved) {
    try {
      File(from(s)).renameSync(to(s));
    } catch (_) {}
  }
}

ModProfileState _loadProfilesSync(String gameDir) {
  final namsDir = Directory(path.join(gameDir, 'nams'));
  if (!namsDir.existsSync()) namsDir.createSync(recursive: true);

  final profilesFile = File(_profilesTomlPath(gameDir));

  if (!profilesFile.existsSync()) {
    return _migrateFirstRun(gameDir);
  }

  Map<String, dynamic> parsed;
  try {
    parsed = TomlDocument.parse(profilesFile.readAsStringSync()).toMap();
  } catch (_) {
    return _migrateFirstRun(gameDir);
  }

  var active =
      (parsed['active'] is String) ? parsed['active'] as String : 'default';
  final rawProfiles = parsed['profile'];
  final profiles = <ModProfile>[];
  if (rawProfiles is List) {
    for (final entry in rawProfiles) {
      if (entry is! Map) continue;
      final name = entry['name'];
      if (name is! String || name.isEmpty) continue;
      final ts = entry['created_at'];
      DateTime created;
      if (ts is String) {
        created = DateTime.tryParse(ts) ?? DateTime.now().toUtc();
      } else {
        created = DateTime.now().toUtc();
      }
      profiles.add(ModProfile(name: name, createdAt: created));
    }
  }

  // Drop entries whose folder is missing on disk (except the active one,
  // whose "folder" is always nams/mods).
  final filtered = <ModProfile>[];
  for (final p in profiles) {
    if (p.name == active) {
      filtered.add(p);
      if (!Directory(_modsActiveDir(gameDir)).existsSync()) {
        Directory(_modsActiveDir(gameDir)).createSync(recursive: true);
      }
    } else if (Directory(_modsInactiveDir(gameDir, p.name)).existsSync()) {
      filtered.add(p);
    }
  }

  // Adopt stray mods_profile_* folders not in the list.
  final knownNames = filtered.map((p) => p.name).toSet();
  for (final entity in namsDir.listSync()) {
    if (entity is! Directory) continue;
    final folder = path.basename(entity.path);
    if (!folder.startsWith(ModProfilesService._profilePrefix)) continue;
    final inferredName =
        folder.substring(ModProfilesService._profilePrefix.length);
    if (inferredName.isEmpty || knownNames.contains(inferredName)) continue;
    if (!ModProfilesService.isValidName(inferredName)) continue;
    DateTime created;
    try {
      created = entity.statSync().modified.toUtc();
    } catch (_) {
      created = DateTime.now().toUtc();
    }
    filtered.add(ModProfile(name: inferredName, createdAt: created));
    knownNames.add(inferredName);
  }

  // If the active doesn't exist as an entry, backfill it.
  if (!filtered.any((p) => p.name == active)) {
    if (!Directory(_modsActiveDir(gameDir)).existsSync()) {
      Directory(_modsActiveDir(gameDir)).createSync(recursive: true);
    }
    filtered.insert(
      0,
      ModProfile(name: active, createdAt: DateTime.now().toUtc()),
    );
  }

  // Detect any unprefixed-pack-name migration we still need to do (older
  // installs created with the launcher before profiles existed).
  final renameMap = _planPackUpgradeRenames(gameDir, filtered);
  if (renameMap.isNotEmpty) {
    _applyPackRenames(gameDir, renameMap);
    _rewriteLoadOrder(gameDir, renameMap);
  }

  // Always recompute disabled_packs from scratch; this self-heals stale state.
  _writeDisabledPacks(gameDir, _computeDisabledPacks(gameDir, filtered, active));

  // Persist (re)written profiles.toml if the on-disk content drifted.
  final stateChanged = filtered.length != profiles.length ||
      !filtered.every(
          (p) => profiles.any((q) => q.name == p.name)) ||
      renameMap.isNotEmpty;
  if (stateChanged) {
    _writeProfilesToml(gameDir, active, filtered);
  }

  return ModProfileState(activeName: active, profiles: filtered);
}

ModProfileState _migrateFirstRun(String gameDir) {
  final namsDir = Directory(path.join(gameDir, 'nams'));
  if (!namsDir.existsSync()) namsDir.createSync(recursive: true);

  if (!Directory(_modsActiveDir(gameDir)).existsSync()) {
    Directory(_modsActiveDir(gameDir)).createSync(recursive: true);
  }

  final profiles = <ModProfile>[];
  DateTime defaultCreated;
  try {
    defaultCreated =
        Directory(_modsActiveDir(gameDir)).statSync().modified.toUtc();
  } catch (_) {
    defaultCreated = DateTime.now().toUtc();
  }
  profiles.add(ModProfile(
    name: ModProfilesService.defaultProfileName,
    createdAt: defaultCreated,
  ));

  // Adopt pre-existing inactive folders.
  for (final entity in namsDir.listSync()) {
    if (entity is! Directory) continue;
    final folder = path.basename(entity.path);
    if (!folder.startsWith(ModProfilesService._profilePrefix)) continue;
    final inferredName =
        folder.substring(ModProfilesService._profilePrefix.length);
    if (!ModProfilesService.isValidName(inferredName)) continue;
    if (inferredName == ModProfilesService.defaultProfileName) continue;
    DateTime created;
    try {
      created = entity.statSync().modified.toUtc();
    } catch (_) {
      created = DateTime.now().toUtc();
    }
    profiles.add(ModProfile(name: inferredName, createdAt: created));
  }

  // Pack-name upgrade.
  final renameMap = _planPackUpgradeRenames(gameDir, profiles);
  if (renameMap.isNotEmpty) {
    _applyPackRenames(gameDir, renameMap);
    _rewriteLoadOrder(gameDir, renameMap);
  }
  _writeDisabledPacks(
    gameDir,
    _computeDisabledPacks(
      gameDir,
      profiles,
      ModProfilesService.defaultProfileName,
    ),
  );

  _writeProfilesToml(
    gameDir,
    ModProfilesService.defaultProfileName,
    profiles,
  );

  return ModProfileState(
    activeName: ModProfilesService.defaultProfileName,
    profiles: profiles,
  );
}

ModProfileState _createProfileSync(_CreateParams p) {
  final state = _loadProfilesSync(p.gameDir);
  final sanitized = ModProfilesService.sanitizeName(p.name);
  if (!ModProfilesService.isValidName(sanitized)) {
    throw ProfileSwitchException('invalid_name');
  }
  if (state.profiles.any((pr) => pr.name == sanitized)) {
    throw ProfileSwitchException('name_collision');
  }
  if (Directory(_modsInactiveDir(p.gameDir, sanitized)).existsSync()) {
    throw ProfileSwitchException('name_collision');
  }

  final from = state.activeName;
  _backupOrphan(p.gameDir, from);

  // Step A: park the active sidecars under from's name.
  final movedFrom = _moveSidecars(
    (s) => s.activePath(p.gameDir),
    (s) => s.inactivePath(p.gameDir, from),
  );

  void undoStepA() => _restoreSidecars(
        movedFrom,
        (s) => s.inactivePath(p.gameDir, from),
        (s) => s.activePath(p.gameDir),
      );

  // Step B: park current mods/ as mods_profile_<from>/.
  final activeMods = Directory(_modsActiveDir(p.gameDir));
  final fromInactive = Directory(_modsInactiveDir(p.gameDir, from));
  try {
    activeMods.renameSync(fromInactive.path);
  } catch (e) {
    undoStepA();
    throw ProfileSwitchException('rename_failed:$e');
  }

  // Step C: fresh empty mods/.
  try {
    Directory(_modsActiveDir(p.gameDir)).createSync();
  } catch (e) {
    fromInactive.renameSync(activeMods.path);
    undoStepA();
    throw ProfileSwitchException('create_failed:$e');
  }

  final newProfiles = [
    ...state.profiles,
    ModProfile(name: sanitized, createdAt: DateTime.now().toUtc()),
  ];
  _writeDisabledPacks(
    p.gameDir,
    _computeDisabledPacks(p.gameDir, newProfiles, sanitized),
  );
  _writeProfilesToml(p.gameDir, sanitized, newProfiles);

  return ModProfileState(activeName: sanitized, profiles: newProfiles);
}

ModProfileState _switchProfileSync(_SwitchParams p) {
  final state = _loadProfilesSync(p.gameDir);
  final from = state.activeName;
  final to = p.to;
  if (from == to) return state;
  if (!state.profiles.any((pr) => pr.name == to)) {
    throw ProfileSwitchException('unknown_target');
  }
  if (!Directory(_modsInactiveDir(p.gameDir, to)).existsSync()) {
    throw ProfileSwitchException('target_missing');
  }
  if (!Directory(_modsActiveDir(p.gameDir)).existsSync()) {
    throw ProfileSwitchException('active_missing');
  }
  _backupOrphan(p.gameDir, from);

  // Step A: park the active sidecars under from's name.
  final movedFrom = _moveSidecars(
    (s) => s.activePath(p.gameDir),
    (s) => s.inactivePath(p.gameDir, from),
  );

  void undoStepA() => _restoreSidecars(
        movedFrom,
        (s) => s.inactivePath(p.gameDir, from),
        (s) => s.activePath(p.gameDir),
      );

  // Step B: mods/ -> mods_profile_<from>/.
  try {
    Directory(_modsActiveDir(p.gameDir))
        .renameSync(_modsInactiveDir(p.gameDir, from));
  } catch (e) {
    undoStepA();
    throw ProfileSwitchException('step_b_failed:$e');
  }

  // Step C: mods_profile_<to>/ -> mods/.
  try {
    Directory(_modsInactiveDir(p.gameDir, to))
        .renameSync(_modsActiveDir(p.gameDir));
  } catch (e) {
    Directory(_modsInactiveDir(p.gameDir, from))
        .renameSync(_modsActiveDir(p.gameDir));
    undoStepA();
    throw ProfileSwitchException('step_c_failed:$e');
  }

  void undoStepsBC() {
    Directory(_modsActiveDir(p.gameDir))
        .renameSync(_modsInactiveDir(p.gameDir, to));
    Directory(_modsInactiveDir(p.gameDir, from))
        .renameSync(_modsActiveDir(p.gameDir));
    undoStepA();
  }

  // Step D: the target profile's sidecars become the active ones.
  List<_Sidecar> movedTo;
  try {
    movedTo = _moveSidecars(
      (s) => s.inactivePath(p.gameDir, to),
      (s) => s.activePath(p.gameDir),
    );
  } catch (e) {
    undoStepsBC();
    throw ProfileSwitchException('step_d_failed:$e');
  }

  // Step E: rewrite disabled_packs.
  final newDisabledPacks =
      _computeDisabledPacks(p.gameDir, state.profiles, to);
  try {
    _writeDisabledPacks(p.gameDir, newDisabledPacks);
  } catch (e) {
    _restoreSidecars(
      movedTo,
      (s) => s.activePath(p.gameDir),
      (s) => s.inactivePath(p.gameDir, to),
    );
    undoStepsBC();
    throw ProfileSwitchException('step_e_failed:$e');
  }

  // Step F: rewrite profiles.toml.
  try {
    _writeProfilesToml(p.gameDir, to, state.profiles);
  } catch (e) {
    throw ProfileSwitchException('step_f_failed:$e');
  }

  return ModProfileState(activeName: to, profiles: state.profiles);
}

ModProfileState _deleteProfileSync(_CreateParams p) {
  final state = _loadProfilesSync(p.gameDir);
  if (state.profiles.length <= 1) {
    throw ProfileSwitchException('cannot_delete_last');
  }
  if (state.activeName == p.name) {
    throw ProfileSwitchException('cannot_delete_active');
  }
  final entry = state.profiles.where((pr) => pr.name == p.name).toList();
  if (entry.isEmpty) {
    throw ProfileSwitchException('unknown_target');
  }

  // Collect bundled pack folders owned by this profile so we can remove them.
  final ownedPacks = _collectProfileBundledPacks(p.gameDir, p.name);

  // Delete the inactive profile folder + its sidecars.
  try {
    final dir = Directory(_modsInactiveDir(p.gameDir, p.name));
    if (dir.existsSync()) dir.deleteSync(recursive: true);
  } catch (_) {}
  for (final s in _sidecars) {
    try {
      final f = File(s.inactivePath(p.gameDir, p.name));
      if (f.existsSync()) f.deleteSync();
    } catch (_) {}
  }

  // Cascade-delete bundled texture packs owned by the profile.
  final injectRoot = Directory(path.join(p.gameDir, 'nams', 'inject', 'textures'));
  for (final pack in ownedPacks) {
    try {
      final packDir = Directory(path.join(injectRoot.path, pack));
      if (packDir.existsSync()) packDir.deleteSync(recursive: true);
    } catch (_) {}
  }

  // Update load_order: drop deleted pack names. Update disabled_packs: drop them too.
  final renameMap = <String, String?>{for (final pack in ownedPacks) pack: null};
  _rewriteLoadOrder(p.gameDir, renameMap);

  final newProfiles =
      state.profiles.where((pr) => pr.name != p.name).toList();
  _writeDisabledPacks(
    p.gameDir,
    _computeDisabledPacks(p.gameDir, newProfiles, state.activeName),
  );
  _writeProfilesToml(p.gameDir, state.activeName, newProfiles);

  return ModProfileState(
    activeName: state.activeName,
    profiles: newProfiles,
  );
}

ModProfileState _renameProfileSync(_RenameParams p) {
  final state = _loadProfilesSync(p.gameDir);
  final sanitized = ModProfilesService.sanitizeName(p.newName);
  if (!ModProfilesService.isValidName(sanitized)) {
    throw ProfileSwitchException('invalid_name');
  }
  if (sanitized == p.oldName) return state;
  if (state.profiles.any((pr) => pr.name == sanitized)) {
    throw ProfileSwitchException('name_collision');
  }
  if (Directory(_modsInactiveDir(p.gameDir, sanitized)).existsSync()) {
    throw ProfileSwitchException('name_collision');
  }
  if (!state.profiles.any((pr) => pr.name == p.oldName)) {
    throw ProfileSwitchException('unknown_target');
  }

  final isActive = state.activeName == p.oldName;

  // Plan pack renames for every mod owned by the renamed profile.
  final renameMap =
      _planProfilePrefixRenames(p.gameDir, p.oldName, sanitized, isActive);

  if (!isActive) {
    // Rename mods_profile_<old>/ -> mods_profile_<new>/
    try {
      Directory(_modsInactiveDir(p.gameDir, p.oldName))
          .renameSync(_modsInactiveDir(p.gameDir, sanitized));
    } catch (e) {
      throw ProfileSwitchException('mods_rename_failed:$e');
    }
    // Every sidecar follows the profile's new name.
    try {
      _moveSidecars(
        (s) => s.inactivePath(p.gameDir, p.oldName),
        (s) => s.inactivePath(p.gameDir, sanitized),
      );
    } catch (e) {
      Directory(_modsInactiveDir(p.gameDir, sanitized))
          .renameSync(_modsInactiveDir(p.gameDir, p.oldName));
      throw ProfileSwitchException('disabled_rename_failed:$e');
    }
  }

  _applyPackRenames(p.gameDir, renameMap);
  _rewriteLoadOrder(p.gameDir, renameMap);

  final newProfiles = state.profiles.map((pr) {
    if (pr.name == p.oldName) {
      return ModProfile(name: sanitized, createdAt: pr.createdAt);
    }
    return pr;
  }).toList();
  final newActive = isActive ? sanitized : state.activeName;
  _writeDisabledPacks(
    p.gameDir,
    _computeDisabledPacks(p.gameDir, newProfiles, newActive),
  );
  _writeProfilesToml(p.gameDir, newActive, newProfiles);

  return ModProfileState(activeName: newActive, profiles: newProfiles);
}

// ----- helpers -----

void _writeProfilesToml(
  String gameDir,
  String active,
  List<ModProfile> profiles,
) {
  final buf = StringBuffer();
  buf.writeln('# YP Launcher mod profile state. Edit via the launcher.');
  buf.writeln('active = "$active"');
  buf.writeln();
  for (final p in profiles) {
    buf.writeln('[[profile]]');
    buf.writeln('name = "${p.name}"');
    buf.writeln('created_at = "${p.createdAt.toIso8601String()}"');
    buf.writeln();
  }
  final file = File(_profilesTomlPath(gameDir));
  file.writeAsStringSync(buf.toString());
}

List<String> _readPacksFromSidecar(File sidecar) {
  try {
    if (!sidecar.existsSync()) return const [];
    final raw = jsonDecode(sidecar.readAsStringSync());
    if (raw is Map && raw['packs'] is List) {
      return (raw['packs'] as List)
          .where((e) => e is String && e.isNotEmpty)
          .cast<String>()
          .toList();
    }
  } catch (_) {}
  return const [];
}

void _writePacksToSidecar(File sidecar, List<String> packs) {
  try {
    sidecar.writeAsStringSync(jsonEncode({'packs': packs}));
  } catch (_) {}
}

/// Walk every mod folder of every profile, find packs that don't yet have
/// the `(<profile>:<modId>)` suffix, and plan a rename.
Map<String, String?> _planPackUpgradeRenames(
  String gameDir,
  List<ModProfile> profiles,
) {
  final result = <String, String?>{};
  for (final profile in profiles) {
    final modsRoot = Directory(_modsRootForProfile(gameDir, profile.name));
    if (!modsRoot.existsSync()) continue;
    for (final modDir in modsRoot.listSync().whereType<Directory>()) {
      final modId = path.basename(modDir.path);
      if (modId.startsWith('.') || modId.startsWith('_')) continue;
      final sidecar = File(
        path.join(modDir.path, ModProfilesService._bundledTexturesSidecar),
      );
      final packs = _readPacksFromSidecar(sidecar);
      if (packs.isEmpty) continue;
      final newPacks = <String>[];
      var anyChanged = false;
      for (final pack in packs) {
        final upgraded = _upgradePackNameIfNeeded(pack, profile.name, modId);
        if (upgraded != pack) {
          result[pack] = upgraded;
          anyChanged = true;
        }
        newPacks.add(upgraded);
      }
      if (anyChanged) {
        _writePacksToSidecar(sidecar, newPacks);
      }
    }
  }
  return result;
}

String _modsRootForProfile(String gameDir, String profileName) {
  // Active profile has unknown name from this helper's POV; fall back to mods/
  // when the profile-prefixed folder doesn't exist.
  final inactive = _modsInactiveDir(gameDir, profileName);
  if (Directory(inactive).existsSync()) return inactive;
  return _modsActiveDir(gameDir);
}

/// Returns the profile-scoped pack name. If the existing name already has a
/// `(<modId>@<profile>)` suffix, leave it alone.
String _upgradePackNameIfNeeded(String pack, String profile, String modId) {
  // Already in profile-scoped form?
  if (RegExp(r'\([^()@]+@[^()]+\)\s*$').hasMatch(pack)) return pack;

  // Has the legacy `(modId)` suffix (no `@`)? Rewrite to include profile.
  final legacyMatch = RegExp(r'^(.*) \(([^()@]+)\)\s*$').firstMatch(pack);
  if (legacyMatch != null) {
    final base = legacyMatch.group(1)!;
    final id = legacyMatch.group(2)!;
    return '$base ($id@$profile)';
  }

  // Plain name → append the profile-scoped suffix.
  return '$pack ($modId@$profile)';
}

/// Apply pack folder renames. `null` value means delete (used by deleteProfile).
void _applyPackRenames(String gameDir, Map<String, String?> renameMap) {
  if (renameMap.isEmpty) return;
  final injectRoot = Directory(
    path.join(gameDir, 'nams', 'inject', 'textures'),
  );
  if (!injectRoot.existsSync()) return;
  for (final entry in renameMap.entries) {
    final from = Directory(path.join(injectRoot.path, entry.key));
    if (!from.existsSync()) continue;
    final newName = entry.value;
    if (newName == null) {
      try {
        from.deleteSync(recursive: true);
      } catch (_) {}
      continue;
    }
    final to = Directory(path.join(injectRoot.path, newName));
    if (to.existsSync()) continue;
    try {
      from.renameSync(to.path);
    } catch (_) {}
  }
}

/// For renameProfile: rename every pack owned by `oldProfile` to use
/// `newProfile` in its suffix; rewrite each mod's sidecar.
Map<String, String?> _planProfilePrefixRenames(
  String gameDir,
  String oldProfile,
  String newProfile,
  bool isActive,
) {
  final result = <String, String?>{};
  final modsRoot = Directory(
    isActive
        ? _modsActiveDir(gameDir)
        : _modsInactiveDir(gameDir, oldProfile),
  );
  if (!modsRoot.existsSync()) return result;

  for (final modDir in modsRoot.listSync().whereType<Directory>()) {
    final sidecar = File(
      path.join(modDir.path, ModProfilesService._bundledTexturesSidecar),
    );
    final packs = _readPacksFromSidecar(sidecar);
    if (packs.isEmpty) continue;
    final newPacks = <String>[];
    var anyChanged = false;
    for (final pack in packs) {
      final renamed = _swapProfileInPackName(pack, oldProfile, newProfile);
      if (renamed != pack) {
        result[pack] = renamed;
        anyChanged = true;
      }
      newPacks.add(renamed);
    }
    if (anyChanged) _writePacksToSidecar(sidecar, newPacks);
  }
  return result;
}

String _swapProfileInPackName(
  String pack,
  String oldProfile,
  String newProfile,
) {
  final m = RegExp(r'^(.*) \(([^()@]+)@([^()]+)\)\s*$').firstMatch(pack);
  if (m == null) return pack;
  if (m.group(3) != oldProfile) return pack;
  return '${m.group(1)} (${m.group(2)}@$newProfile)';
}

/// Walk an inactive profile's mods and union up its bundled pack names.
Set<String> _collectProfileBundledPacks(String gameDir, String profileName) {
  final out = <String>{};
  final root = Directory(_modsInactiveDir(gameDir, profileName));
  if (!root.existsSync()) return out;
  for (final modDir in root.listSync().whereType<Directory>()) {
    final sidecar = File(
      path.join(modDir.path, ModProfilesService._bundledTexturesSidecar),
    );
    out.addAll(_readPacksFromSidecar(sidecar));
  }
  return out;
}

/// Compute `disabled_packs` = union of every inactive profile's bundled packs
/// given a target active profile.
List<String> _computeDisabledPacks(
  String gameDir,
  List<ModProfile> profiles,
  String activeName,
) {
  final out = <String>{};
  for (final profile in profiles) {
    if (profile.name == activeName) continue;
    out.addAll(_collectProfileBundledPacks(gameDir, profile.name));
  }
  final list = out.toList()..sort();
  return list;
}

void _writeDisabledPacks(String gameDir, List<String> packs) {
  final file = File(_textureInjectionTomlPath(gameDir));
  if (!file.existsSync()) return; // honored: written by NamsConfigService on first run
  final raw = file.readAsStringSync();
  final lines = raw.split('\n');
  final newValue = '[${packs.map((p) => '"${_escapeToml(p)}"').join(', ')}]';

  var found = false;
  for (var i = 0; i < lines.length; i++) {
    final trimmed = lines[i].trimLeft();
    if (trimmed.startsWith('disabled_packs')) {
      lines[i] = 'disabled_packs = $newValue';
      found = true;
      break;
    }
  }
  if (!found) {
    lines.add('');
    lines.add('# Pack folder names skipped during scanning. Launcher-managed');
    lines.add('# (overwritten on every profile switch).');
    lines.add('disabled_packs = $newValue');
  }
  file.writeAsStringSync(lines.join('\n'));
}

void _rewriteLoadOrder(String gameDir, Map<String, String?> renameMap) {
  if (renameMap.isEmpty) return;
  final file = File(_textureInjectionTomlPath(gameDir));
  if (!file.existsSync()) return;
  final raw = file.readAsStringSync();
  final lines = raw.split('\n');

  for (var i = 0; i < lines.length; i++) {
    final trimmed = lines[i].trimLeft();
    if (!trimmed.startsWith('load_order')) continue;
    // Single-line array form. We're conservative: only handle the form
    // `load_order = ["a", "b"]`. Comments are preserved.
    final eq = lines[i].indexOf('=');
    if (eq < 0) break;
    final after = lines[i].substring(eq + 1).trim();
    if (!after.startsWith('[') || !after.endsWith(']')) break;
    final inner = after.substring(1, after.length - 1);
    final entries = <String>[];
    for (final piece in _splitTomlArray(inner)) {
      final unquoted = _unquoteToml(piece.trim());
      if (unquoted == null) continue;
      if (renameMap.containsKey(unquoted)) {
        final mapped = renameMap[unquoted];
        if (mapped == null) continue; // dropped
        entries.add('"${_escapeToml(mapped)}"');
      } else {
        entries.add('"${_escapeToml(unquoted)}"');
      }
    }
    lines[i] = 'load_order = [${entries.join(', ')}]';
    break;
  }
  file.writeAsStringSync(lines.join('\n'));
}

Iterable<String> _splitTomlArray(String inner) sync* {
  // Naive splitter — good enough for scalar string arrays.
  final buf = StringBuffer();
  var inString = false;
  var escape = false;
  for (var i = 0; i < inner.length; i++) {
    final c = inner[i];
    if (escape) {
      buf.write(c);
      escape = false;
      continue;
    }
    if (c == r'\') {
      buf.write(c);
      escape = true;
      continue;
    }
    if (c == '"') {
      inString = !inString;
      buf.write(c);
      continue;
    }
    if (c == ',' && !inString) {
      final piece = buf.toString();
      if (piece.trim().isNotEmpty) yield piece;
      buf.clear();
      continue;
    }
    buf.write(c);
  }
  final last = buf.toString();
  if (last.trim().isNotEmpty) yield last;
}

String? _unquoteToml(String s) {
  if (s.length < 2) return null;
  if (!s.startsWith('"') || !s.endsWith('"')) return null;
  return s
      .substring(1, s.length - 1)
      .replaceAll(r'\"', '"')
      .replaceAll(r'\\', r'\');
}

String _escapeToml(String s) =>
    s.replaceAll(r'\', r'\\').replaceAll('"', r'\"');
