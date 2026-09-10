import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/graphics_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_flags.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';
import 'package:yp_launcher/services/toml_service.dart';

class GameModsRuntime extends GraphicsRuntime {
  const GameModsRuntime();

  @override
  ThirdPartyRuntime get runtime => ThirdPartyRuntime.gameMods;

  @override
  String get disableFlagKey => NamsFields.disableGameModsLoading.key;

  @override
  String installDir(String gameDir) => ThirdPartyPaths.gameMods();

  String configPath(String gameDir) =>
      path.join(installDir(gameDir), AppStrings.gameModsConfigName);

  @override
  bool canInstall(ThirdPartyClassification c) =>
      c.kind == ThirdPartyKind.gameMod;

  @override
  Future<ThirdPartyInstallResult> install(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    if (c.gameModDlls.isEmpty) {
      return const ThirdPartyInstallResult(ok: false, errorKey: 'no_dll');
    }
    final dest = installDir(gameDir);
    Directory(dest).createSync(recursive: true);

    for (final dll in c.gameModDlls) {
      FileOps.copyFileInto(dll, dest);
    }

    await ThirdPartyFlags.set(gameDir, disableFlagKey, false);
    return ThirdPartyInstallResult(ok: true, runtime: runtime);
  }

  @override
  Future<ThirdPartyUpdateInfo?> wouldUpdate(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    for (final incoming in c.gameModDlls) {
      final installed = path.join(installDir(gameDir), path.basename(incoming));
      if (!File(installed).existsSync()) continue;
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
    final dir = installDir(gameDir);
    final info = _collectInfo(dir);
    return ThirdPartyRuntimeStatus(
      installed: info.mods.isNotEmpty,
      enabled: await isEnabled(gameDir),
      gameModsInfo: info,
    );
  }

  GameModsInfo _collectInfo(String dir) {
    final d = Directory(dir);
    if (!d.existsSync()) return const GameModsInfo();

    final configFile = File(path.join(dir, AppStrings.gameModsConfigName));
    final disabled = <String>{};
    if (configFile.existsSync()) {
      try {
        final parsed = TomlService.parse(configFile.readAsStringSync());
        parsed.forEach((key, value) {
          if (key == 'defaults') return;
          if (value is Map && value['disabled'] == true) {
            disabled.add(key.toLowerCase());
          }
        });
      } catch (_) {}
    }

    final mods = <GameModEntry>[];
    for (final entity in d.listSync().whereType<File>()) {
      final name = path.basename(entity.path);
      if (!name.toLowerCase().endsWith('.dll')) continue;
      mods.add(
        GameModEntry(
          fileName: name,
          sizeLabel: FileOps.sizeLabel(entity.path),
          disabled: disabled.contains(name.toLowerCase()),
        ),
      );
    }
    mods.sort(
      (a, b) => a.fileName.toLowerCase().compareTo(b.fileName.toLowerCase()),
    );

    return GameModsInfo(mods: mods, hasConfig: configFile.existsSync());
  }

  Future<void> setModDisabled(
    String gameDir,
    String fileName,
    bool disabled,
  ) async {
    final file = File(configPath(gameDir));
    if (!file.existsSync()) {
      Directory(installDir(gameDir)).createSync(recursive: true);
      file.writeAsStringSync('');
    }
    file.writeAsStringSync(
      setDisabledIn(file.readAsStringSync(), fileName, disabled),
    );
  }

  static String setDisabledIn(String raw, String fileName, bool disabled) {
    final lines = raw.replaceAll('\r', '').split('\n');
    final wanted = fileName.toLowerCase();

    var sectionStart = -1;
    for (var i = 0; i < lines.length; i++) {
      final name = _sectionNameOf(lines[i]);
      if (name != null && name.toLowerCase() == wanted) {
        sectionStart = i;
        break;
      }
    }

    if (sectionStart == -1) {
      final buf = StringBuffer(raw);
      if (raw.isNotEmpty && !raw.endsWith('\n')) buf.write('\n');
      if (raw.trim().isNotEmpty) buf.write('\n');
      buf.writeln('["$fileName"]');
      buf.writeln('disabled = $disabled');
      return buf.toString();
    }

    var sectionEnd = lines.length;
    for (var i = sectionStart + 1; i < lines.length; i++) {
      if (_sectionNameOf(lines[i]) != null) {
        sectionEnd = i;
        break;
      }
    }

    for (var i = sectionStart + 1; i < sectionEnd; i++) {
      if (RegExp(r'^\s*disabled\s*=').hasMatch(lines[i])) {
        lines[i] = 'disabled = $disabled';
        return lines.join('\n');
      }
    }

    lines.insert(sectionStart + 1, 'disabled = $disabled');
    return lines.join('\n');
  }

  static String? _sectionNameOf(String line) {
    final t = line.trim();
    if (!t.startsWith('[') || !t.endsWith(']')) return null;
    var inner = t.substring(1, t.length - 1).trim();
    if (inner.length >= 2 &&
        ((inner.startsWith('"') && inner.endsWith('"')) ||
            (inner.startsWith("'") && inner.endsWith("'")))) {
      inner = inner.substring(1, inner.length - 1);
    }
    return inner;
  }

  Future<void> removeMod(String gameDir, String fileName) async {
    FileOps.deleteFileQuiet(path.join(installDir(gameDir), fileName));
  }

  @override
  Future<void> remove(String gameDir) async {
    final dir = Directory(installDir(gameDir));
    if (dir.existsSync()) {
      for (final f in dir.listSync().whereType<File>()) {
        if (f.path.toLowerCase().endsWith('.dll')) {
          FileOps.deleteFileQuiet(f.path);
        }
      }
    }
    await ThirdPartyFlags.set(gameDir, disableFlagKey, true);
  }
}
