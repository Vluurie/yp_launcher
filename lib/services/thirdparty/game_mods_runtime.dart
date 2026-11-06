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

const _defaultsKey = 'defaults';

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
    final config = configFile.existsSync()
        ? TomlService.parse(configFile.readAsStringSync())
        : const <String, dynamic>{};

    final mods = <GameModEntry>[];
    for (final entity in d.listSync().whereType<File>()) {
      final name = path.basename(entity.path);
      if (!name.toLowerCase().endsWith('.dll')) continue;
      final settings = settingsFor(config, name);
      mods.add(
        GameModEntry(
          fileName: name,
          sizeLabel: FileOps.sizeLabel(entity.path),
          disabled: settings.disabled,
          settings: settings,
        ),
      );
    }
    mods.sort(
      (a, b) => a.fileName.toLowerCase().compareTo(b.fileName.toLowerCase()),
    );

    return GameModsInfo(mods: mods, hasConfig: configFile.existsSync());
  }

  static String? _sectionKey(Map<String, dynamic> config, String fileName) {
    final wanted = fileName.toLowerCase();
    for (final entry in config.entries) {
      if (entry.key == _defaultsKey || entry.value is! Map) continue;
      if (entry.key.toLowerCase() == wanted) return entry.key;
    }
    return null;
  }

  static GameModSettings settingsFor(
    Map<String, dynamic> config,
    String fileName,
  ) {
    final key = _sectionKey(config, fileName);
    final table = config[key ?? _defaultsKey];
    return table is Map
        ? GameModSettings.fromToml(table)
        : const GameModSettings();
  }

  static String? withSettings(
    String raw,
    String fileName,
    GameModSettings settings,
  ) {
    final parsed = TomlService.parseStrict(raw);
    if (!parsed.ok) return null;
    final values = Map<String, dynamic>.from(parsed.values);
    final key = _sectionKey(values, fileName) ?? fileName;
    final existing = values[key];
    values[key] = <String, dynamic>{
      if (existing is Map<String, dynamic>) ...existing,
      ...settings.toToml(),
    };
    return TomlService.serialize(values, commentsFrom: raw);
  }

  Future<bool> setModSettings(
    String gameDir,
    String fileName,
    GameModSettings settings,
  ) async {
    final file = File(configPath(gameDir));
    final raw = file.existsSync() ? file.readAsStringSync() : '';
    final updated = withSettings(raw, fileName, settings);
    if (updated == null) return false;
    Directory(installDir(gameDir)).createSync(recursive: true);
    file.writeAsStringSync(updated);
    return true;
  }

  Future<bool> setModDisabled(
    String gameDir,
    String fileName,
    bool disabled,
  ) async {
    final file = File(configPath(gameDir));
    final config = file.existsSync()
        ? TomlService.parse(file.readAsStringSync())
        : const <String, dynamic>{};
    return setModSettings(
      gameDir,
      fileName,
      settingsFor(config, fileName).copyWith(disabled: disabled),
    );
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
