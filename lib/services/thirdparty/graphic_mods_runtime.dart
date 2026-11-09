import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/graphics_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_flags.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';

class GraphicModsRuntime extends GraphicsRuntime {
  const GraphicModsRuntime();

  @override
  ThirdPartyRuntime get runtime => ThirdPartyRuntime.graphicMods;

  @override
  String get disableFlagKey => NamsFields.disableGraphicModsLoading.key;

  @override
  String installDir(String gameDir) => ThirdPartyPaths.graphicMods(gameDir);

  String disabledDir(String gameDir) =>
      ThirdPartyPaths.graphicModsDisabled(gameDir);

  @override
  bool canInstall(ThirdPartyClassification c) =>
      c.kind == ThirdPartyKind.graphicMod;

  @override
  Future<ThirdPartyInstallResult> install(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    if (c.graphicModDlls.isEmpty) {
      return const ThirdPartyInstallResult(ok: false, errorKey: 'no_dll');
    }
    final names = c.graphicModDlls.map(path.basename).toList();
    final active = installDir(gameDir);

    var dest = ThirdPartyPaths.modDirWithDll(active, names);
    if (dest == null) {
      final parked = ThirdPartyPaths.modDirWithDll(disabledDir(gameDir), names);
      if (parked != null) {
        dest = path.join(
          active,
          ThirdPartyPaths.uniqueDirName(active, path.basename(parked)),
        );
        FileOps.moveMerging(parked, dest);
      }
    }
    dest ??= path.join(
      active,
      ThirdPartyPaths.uniqueDirName(
        active,
        ThirdPartyPaths.modDirName(
          c.sourceName ?? '',
          fallback: path.basenameWithoutExtension(names.first),
        ),
      ),
    );
    Directory(dest).createSync(recursive: true);

    for (final folder in c.graphicModDlls.map(path.dirname).toSet()) {
      FileOps.mergeDirectory(folder, dest);
    }

    await ThirdPartyFlags.set(gameDir, disableFlagKey, false);
    return ThirdPartyInstallResult(ok: true, runtime: runtime);
  }

  @override
  Future<ThirdPartyUpdateInfo?> wouldUpdate(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    for (final incoming in c.graphicModDlls) {
      final name = path.basename(incoming);
      final installed =
          ThirdPartyPaths.findDll(installDir(gameDir), name) ??
          ThirdPartyPaths.findDll(disabledDir(gameDir), name);
      if (installed == null) continue;
      if (!FileOps.filesDiffer(incoming, installed)) continue;
      return ThirdPartyUpdateInfo(
        runtime: runtime,
        installedLabel: FileOps.sizeLabel(installed),
        incomingLabel: FileOps.sizeLabel(incoming),
      );
    }
    return null;
  }

  @override
  Future<void> repair(String gameDir) async {}

  @override
  Future<bool> importFromGameRoot(String gameDir) async => false;

  @override
  Future<ThirdPartyRuntimeStatus> status(String gameDir) async {
    final mods =
        [
          ..._dllsIn(installDir(gameDir), disabled: false),
          ..._dllsIn(disabledDir(gameDir), disabled: true),
        ]..sort(
          (a, b) => a.key.toLowerCase().compareTo(b.key.toLowerCase()),
        );
    return ThirdPartyRuntimeStatus(
      installed: mods.isNotEmpty,
      enabled: await isEnabled(gameDir),
      graphicModsInfo: GameModsInfo(mods: mods),
    );
  }

  List<GameModEntry> _dllsIn(String root, {required bool disabled}) {
    return [
      for (final modDir in ThirdPartyPaths.modDirs(root))
        for (final dll in ThirdPartyPaths.dllsIn(modDir.path))
          GameModEntry(
            fileName: path.basename(dll.path),
            modDir: path.basename(modDir.path),
            sizeLabel: FileOps.sizeLabel(dll.path),
            disabled: disabled,
          ),
    ];
  }

  /// Moves the whole mod folder [modDir] in or out of the folder NAMS loads.
  Future<void> setModDisabled(
    String gameDir,
    String modDir,
    bool disabled,
  ) async {
    if (!ThirdPartyPaths.isPlainName(modDir)) return;
    final from = path.join(
      disabled ? installDir(gameDir) : disabledDir(gameDir),
      modDir,
    );
    final to = path.join(
      disabled ? disabledDir(gameDir) : installDir(gameDir),
      modDir,
    );
    if (!Directory(from).existsSync()) return;
    FileOps.moveMerging(from, to);
  }

  String? pathOf(String gameDir, String modDir) {
    if (!ThirdPartyPaths.isPlainName(modDir)) return null;
    return [
      path.join(installDir(gameDir), modDir),
      path.join(disabledDir(gameDir), modDir),
    ].where((d) => Directory(d).existsSync()).firstOrNull;
  }

  Future<void> removeMod(String gameDir, String modDir) async {
    if (!ThirdPartyPaths.isPlainName(modDir)) return;
    FileOps.deleteDirQuiet(path.join(installDir(gameDir), modDir));
    FileOps.deleteDirQuiet(path.join(disabledDir(gameDir), modDir));
  }

  @override
  Future<void> remove(String gameDir) async {
    for (final root in [installDir(gameDir), disabledDir(gameDir)]) {
      for (final dir in ThirdPartyPaths.modDirs(root)) {
        FileOps.deleteDirQuiet(dir.path);
      }
    }
    await ThirdPartyFlags.set(gameDir, disableFlagKey, true);
  }
}
