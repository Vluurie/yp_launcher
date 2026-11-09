import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';

class ThirdPartyMigrationResult {
  final String from;
  final String to;
  final int movedFiles;
  final List<String> leftBehind;

  const ThirdPartyMigrationResult({
    this.from = '',
    this.to = '',
    this.movedFiles = 0,
    this.leftBehind = const [],
  });
}

/// Moves the content of `thirdparty\<kind>\` next to NAMS.exe into mod folders
/// under `<game>\nams\thirdparty\<kind>\`. Nothing that cannot be moved is
/// deleted; it stays in place and is listed in the result.
class ThirdPartyMigration {
  ThirdPartyMigration._();

  static Future<ThirdPartyMigrationResult>? _running;

  static Future<ThirdPartyMigrationResult> run(String gameDir) =>
      _running ??= _run(gameDir).whenComplete(() => _running = null);

  static Future<ThirdPartyMigrationResult> _run(String gameDir) async {
    const none = ThirdPartyMigrationResult();
    if (gameDir.isEmpty) return none;
    final legacy = ThirdPartyPaths.legacyRoot(
      await LauncherSetupService.getLauncherDirectory(),
    );
    if (!Directory(legacy).existsSync()) return none;
    // NAMS may hold these DLLs open while the game runs.
    if (await PlatformAdapter.current.isGameRunning()) return none;

    final before = _fileCount(legacy);
    final left = <String>[];

    final oldReShade = path.join(legacy, AppStrings.reshadeDirName);
    if (Directory(oldReShade).existsSync()) {
      left.addAll(
        FileOps.moveMerging(
          oldReShade,
          const ReShadeRuntime().installDir(gameDir),
        ),
      );
    }

    final oldMigoto = path.join(legacy, AppStrings.migotoDirName);
    if (Directory(oldMigoto).existsSync()) {
      left.addAll(
        FileOps.moveMerging(oldMigoto, const MigotoRuntime().installDir(gameDir)),
      );
    }

    final oldGame = path.join(legacy, AppStrings.gameModsDirName);
    if (Directory(oldGame).existsSync()) {
      left.addAll(_moveGameMods(oldGame, ThirdPartyPaths.gameMods(gameDir)));
    }

    final oldGraphic = path.join(legacy, AppStrings.graphicModsDirName);
    if (Directory(oldGraphic).existsSync()) {
      left.addAll(
        _moveGraphicMods(
          oldGraphic,
          ThirdPartyPaths.graphicMods(gameDir),
          ThirdPartyPaths.graphicModsDisabled(gameDir),
        ),
      );
    }

    _deleteIfEmpty(legacy);
    return ThirdPartyMigrationResult(
      from: legacy,
      to: ThirdPartyPaths.root(gameDir),
      movedFiles: before - _fileCount(legacy),
      leftBehind: left,
    );
  }

  /// Every DLL becomes its own mod folder, together with the files named after
  /// it (`celestial.dll`, `celestial.ini`, `celestial_log.txt`). Other files
  /// join the mod only when there is exactly one; `game.toml` stays beside the
  /// mod folders.
  static List<String> _moveGameMods(String old, String kindDir) {
    final left = <String>[];
    final entries = _list(old);
    final rest = <FileSystemEntity>[];
    for (final e in entries) {
      if (e is File &&
          path.basename(e.path).toLowerCase() ==
              AppStrings.gameModsConfigName) {
        left.addAll(
          FileOps.moveMerging(
            e.path,
            path.join(kindDir, AppStrings.gameModsConfigName),
          ),
        );
      } else {
        rest.add(e);
      }
    }

    final dlls = rest.whereType<File>().where(_isDll).toList();
    final groups = <String, List<FileSystemEntity>>{
      for (final dll in dlls) dll.path: [dll],
    };
    final unowned = <FileSystemEntity>[];
    for (final e in rest) {
      if (e is File && _isDll(e)) continue;
      final owner = _ownerDll(path.basename(e.path), dlls);
      if (owner == null) {
        unowned.add(e);
      } else {
        groups[owner]!.add(e);
      }
    }
    if (groups.length == 1) {
      groups.values.single.addAll(unowned);
    } else {
      left.addAll(unowned.map((e) => e.path));
    }

    for (final entry in groups.entries) {
      final dllName = path.basename(entry.key);
      final stem = path.basenameWithoutExtension(entry.key);
      final dest =
          ThirdPartyPaths.modDirWithDll(kindDir, [dllName]) ??
          path.join(
            kindDir,
            ThirdPartyPaths.uniqueDirName(
              kindDir,
              ThirdPartyPaths.modDirName(stem, fallback: stem),
            ),
          );
      for (final e in entry.value) {
        left.addAll(
          FileOps.moveMerging(e.path, path.join(dest, path.basename(e.path))),
        );
      }
    }

    _deleteIfEmpty(old);
    return left;
  }

  /// Everything in the folder was merged together on install, so it becomes
  /// one mod folder; the DLLs that were turned off go into the turned-off mod
  /// folder of the same name.
  static List<String> _moveGraphicMods(
    String old,
    String activeRoot,
    String disabledRoot,
  ) {
    final left = <String>[];
    final entries = _list(old);
    final parkedDir = entries
        .whereType<Directory>()
        .where(
          (d) =>
              path.basename(d.path).toLowerCase() ==
              AppStrings.thirdPartyDisabledDirName,
        )
        .firstOrNull;
    final active = entries.where((e) => e.path != parkedDir?.path).toList();
    final parked = parkedDir == null
        ? const <FileSystemEntity>[]
        : _list(parkedDir.path);

    if (active.isNotEmpty || parked.isNotEmpty) {
      final activeDlls = active
          .whereType<File>()
          .where(_isDll)
          .map((f) => path.basename(f.path))
          .toList();
      final allDlls = [
        ...activeDlls,
        ...parked
            .whereType<File>()
            .where(_isDll)
            .map((f) => path.basename(f.path)),
      ]..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
      final proxyStems =
          allDlls
              .where((n) => directXProxyNames.contains(n.toLowerCase()))
              .map((n) => path.basenameWithoutExtension(n).toLowerCase())
              .toSet()
              .toList()
            ..sort();
      final name = ThirdPartyPaths.modDirName(
        proxyStems.isNotEmpty
            ? proxyStems.join('+')
            : allDlls.isNotEmpty
            ? path.basenameWithoutExtension(allDlls.first)
            : AppStrings.graphicModsDirName,
        fallback: AppStrings.graphicModsDirName,
      );
      final activeDest =
          ThirdPartyPaths.modDirWithDll(activeRoot, activeDlls) ??
          path.join(
            activeRoot,
            ThirdPartyPaths.uniqueDirName(activeRoot, name),
          );
      final parkedDest = path.join(disabledRoot, path.basename(activeDest));

      for (final e in active) {
        left.addAll(
          FileOps.moveMerging(
            e.path,
            path.join(activeDest, path.basename(e.path)),
          ),
        );
      }
      for (final e in parked) {
        left.addAll(
          FileOps.moveMerging(
            e.path,
            path.join(parkedDest, path.basename(e.path)),
          ),
        );
      }
    }

    if (parkedDir != null) _deleteIfEmpty(parkedDir.path);
    _deleteIfEmpty(old);
    return left;
  }

  static String? _ownerDll(String fileName, List<File> dlls) {
    final name = fileName.toLowerCase();
    String? owner;
    var best = -1;
    for (final dll in dlls) {
      final stem = path.basenameWithoutExtension(dll.path).toLowerCase();
      if (stem.length <= best || !name.startsWith(stem)) continue;
      if (name.length > stem.length &&
          RegExp(r'[a-z0-9]').hasMatch(name[stem.length])) {
        continue;
      }
      owner = dll.path;
      best = stem.length;
    }
    return owner;
  }

  static bool _isDll(File f) => f.path.toLowerCase().endsWith('.dll');

  static List<FileSystemEntity> _list(String dir) {
    try {
      return Directory(dir).listSync(followLinks: false);
    } catch (_) {
      return const [];
    }
  }

  static int _fileCount(String dir) {
    var count = 0;
    FileOps.walkEntities(dir, (e) {
      if (e is File) count++;
      return true;
    });
    return count;
  }

  static void _deleteIfEmpty(String dir) {
    try {
      Directory(dir).deleteSync();
    } catch (_) {}
  }
}
