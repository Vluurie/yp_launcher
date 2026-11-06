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
  String installDir(String gameDir) => ThirdPartyPaths.graphicMods();

  String disabledDir() => ThirdPartyPaths.graphicModsDisabled();

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
    final dest = installDir(gameDir);
    Directory(dest).createSync(recursive: true);

    for (final folder in c.graphicModDlls.map(path.dirname).toSet()) {
      FileOps.mergeDirectory(folder, dest);
    }
    for (final dll in c.graphicModDlls) {
      FileOps.deleteFileQuiet(path.join(disabledDir(), path.basename(dll)));
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
      final installed = [
        path.join(installDir(gameDir), name),
        path.join(disabledDir(), name),
      ].where((f) => File(f).existsSync()).firstOrNull;
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
          ..._dllsIn(disabledDir(), disabled: true),
        ]..sort(
          (a, b) =>
              a.fileName.toLowerCase().compareTo(b.fileName.toLowerCase()),
        );
    return ThirdPartyRuntimeStatus(
      installed: mods.isNotEmpty,
      enabled: await isEnabled(gameDir),
      graphicModsInfo: GameModsInfo(mods: mods),
    );
  }

  List<GameModEntry> _dllsIn(String dir, {required bool disabled}) {
    final d = Directory(dir);
    if (!d.existsSync()) return const [];
    return [
      for (final f in d.listSync().whereType<File>())
        if (f.path.toLowerCase().endsWith('.dll'))
          GameModEntry(
            fileName: path.basename(f.path),
            sizeLabel: FileOps.sizeLabel(f.path),
            disabled: disabled,
          ),
    ];
  }

  Future<void> setModDisabled(
    String gameDir,
    String fileName,
    bool disabled,
  ) async {
    final active = path.join(installDir(gameDir), fileName);
    final parked = path.join(disabledDir(), fileName);
    final from = disabled ? active : parked;
    final to = disabled ? parked : active;
    if (!File(from).existsSync()) return;
    Directory(path.dirname(to)).createSync(recursive: true);
    FileOps.deleteFileQuiet(to);
    File(from).renameSync(to);
  }

  String? pathOf(String gameDir, String fileName) => [
    path.join(installDir(gameDir), fileName),
    path.join(disabledDir(), fileName),
  ].where((f) => File(f).existsSync()).firstOrNull;

  Future<void> removeMod(String gameDir, String fileName) async {
    FileOps.deleteFileQuiet(path.join(installDir(gameDir), fileName));
    FileOps.deleteFileQuiet(path.join(disabledDir(), fileName));
  }

  @override
  Future<void> remove(String gameDir) async {
    for (final dir in [installDir(gameDir), disabledDir()]) {
      final d = Directory(dir);
      if (!d.existsSync()) continue;
      for (final f in d.listSync().whereType<File>()) {
        if (f.path.toLowerCase().endsWith('.dll')) {
          FileOps.deleteFileQuiet(f.path);
        }
      }
    }
    await ThirdPartyFlags.set(gameDir, disableFlagKey, true);
  }
}
