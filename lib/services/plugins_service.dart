import 'dart:io';
import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/plugin_validator.dart';

const _reservedPluginNames = <String>{
  'yorha_protocol.dll',
};

class PluginInstallResult {
  final LauncherPlugin? plugin;
  final String? failureReason;
  const PluginInstallResult.ok(this.plugin) : failureReason = null;
  const PluginInstallResult.fail(this.failureReason) : plugin = null;
  bool get success => plugin != null;
}

class LauncherPlugin {
  final String name;
  final String filePath;
  final int sizeBytes;
  final bool enabled;

  const LauncherPlugin({
    required this.name,
    required this.filePath,
    required this.sizeBytes,
    required this.enabled,
  });
}

class PluginsService {
  PluginsService._();

  static const _legacyPrefsKey = 'disabled_plugins';

  static Future<String> pluginsDir() async {
    final base = LauncherSetupService.launcherDirectory;
    return p.join(base, AppStrings.pluginsDirName);
  }

  static Future<String> disabledPluginsDir() async {
    final base = LauncherSetupService.launcherDirectory;
    return p.join(base, AppStrings.pluginsDisabledDirName);
  }

  static Future<List<LauncherPlugin>> list() async {
    await _migrateLegacyDisabledPrefs();
    final enabledDir = await pluginsDir();
    final disabledDir = await disabledPluginsDir();
    final raw = await IsolateService.run(
      _listSync,
      _ListParams(enabledDir: enabledDir, disabledDir: disabledDir),
    );
    return raw
        .map((e) => LauncherPlugin(
              name: e.name,
              filePath: e.filePath,
              sizeBytes: e.sizeBytes,
              enabled: e.enabled,
            ))
        .toList();
  }

  static Future<void> setEnabled(String name, bool enabled) async {
    final enabledDir = await pluginsDir();
    final disabledDir = await disabledPluginsDir();
    final from = enabled ? disabledDir : enabledDir;
    final to = enabled ? enabledDir : disabledDir;
    await IsolateService.run(
      _moveSync,
      _MoveParams(name: name, fromDir: from, toDir: to),
    );
  }

  static Future<PluginInstallResult> install(String sourceDllPath) async {
    final src = File(sourceDllPath);
    if (!await src.exists()) {
      return const PluginInstallResult.fail('file_not_found');
    }
    final name = p.basename(sourceDllPath);
    final lower = name.toLowerCase();
    if (!lower.endsWith('.dll')) {
      return const PluginInstallResult.fail('not_a_dll');
    }
    if (_reservedPluginNames.contains(lower)) {
      return const PluginInstallResult.fail('reserved_name');
    }

    final result = await PluginValidator.validate(sourceDllPath);
    if (!result.valid) {
      return PluginInstallResult.fail(result.failureReason ?? 'invalid_plugin');
    }

    final dir = await pluginsDir();
    final size = await IsolateService.run(
      _copyFileSync,
      _CopyParams(src: sourceDllPath, destDir: dir, destName: name),
    );
    if (size < 0) {
      return const PluginInstallResult.fail('copy_failed');
    }
    return PluginInstallResult.ok(LauncherPlugin(
      name: name,
      filePath: p.join(dir, name),
      sizeBytes: size,
      enabled: true,
    ));
  }

  static Future<void> delete(String name) async {
    final enabledDir = await pluginsDir();
    final disabledDir = await disabledPluginsDir();
    for (final dir in [enabledDir, disabledDir]) {
      final f = File(p.join(dir, name));
      if (await f.exists()) {
        await f.delete();
      }
    }
  }

  static Future<void> _migrateLegacyDisabledPrefs() async {
    final prefs = await SharedPreferences.getInstance();
    final legacy = prefs.getStringList(_legacyPrefsKey);
    if (legacy == null) return;
    final enabledDir = await pluginsDir();
    final disabledDir = await disabledPluginsDir();
    final wanted = legacy.map((s) => s.toLowerCase()).toSet();
    if (wanted.isNotEmpty) {
      final dir = Directory(enabledDir);
      if (await dir.exists()) {
        await for (final e in dir.list(followLinks: false)) {
          if (e is! File) continue;
          final n = p.basename(e.path);
          if (_reservedPluginNames.contains(n.toLowerCase())) continue;
          if (!wanted.contains(n.toLowerCase())) continue;
          await IsolateService.run(
            _moveSync,
            _MoveParams(name: n, fromDir: enabledDir, toDir: disabledDir),
          );
        }
      }
    }
    await prefs.remove(_legacyPrefsKey);
  }
}

class _CopyParams {
  final String src;
  final String destDir;
  final String destName;
  const _CopyParams({
    required this.src,
    required this.destDir,
    required this.destName,
  });
}

int _copyFileSync(_CopyParams params) {
  try {
    final dir = Directory(params.destDir);
    if (!dir.existsSync()) dir.createSync(recursive: true);
    final dest = p.join(params.destDir, params.destName);
    final destFile = File(params.src).copySync(dest);
    return destFile.lengthSync();
  } catch (_) {
    return -1;
  }
}

class _MoveParams {
  final String name;
  final String fromDir;
  final String toDir;
  const _MoveParams({
    required this.name,
    required this.fromDir,
    required this.toDir,
  });
}

bool _moveSync(_MoveParams params) {
  try {
    final src = File(p.join(params.fromDir, params.name));
    if (!src.existsSync()) return false;
    final toDir = Directory(params.toDir);
    if (!toDir.existsSync()) toDir.createSync(recursive: true);
    final dest = p.join(params.toDir, params.name);
    try {
      src.renameSync(dest);
    } catch (_) {
      src.copySync(dest);
      src.deleteSync();
    }
    return true;
  } catch (_) {
    return false;
  }
}

class _ListParams {
  final String enabledDir;
  final String disabledDir;
  const _ListParams({required this.enabledDir, required this.disabledDir});
}

class _PluginFileEntry {
  final String name;
  final String filePath;
  final int sizeBytes;
  final bool enabled;
  const _PluginFileEntry({
    required this.name,
    required this.filePath,
    required this.sizeBytes,
    required this.enabled,
  });
}

List<_PluginFileEntry> _listSync(_ListParams params) {
  final out = <_PluginFileEntry>[];

  void scan(String dirPath, bool enabled) {
    final dir = Directory(dirPath);
    if (!dir.existsSync()) return;
    for (final entity in dir.listSync(followLinks: false)) {
      if (entity is! File) continue;
      final name = p.basename(entity.path);
      final lower = name.toLowerCase();
      if (!lower.endsWith('.dll')) continue;
      if (_reservedPluginNames.contains(lower)) continue;
      int size = 0;
      try {
        size = entity.lengthSync();
      } catch (_) {}
      out.add(_PluginFileEntry(
        name: name,
        filePath: entity.path,
        sizeBytes: size,
        enabled: enabled,
      ));
    }
  }

  scan(params.enabledDir, true);
  scan(params.disabledDir, false);
  out.sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
  return out;
}
