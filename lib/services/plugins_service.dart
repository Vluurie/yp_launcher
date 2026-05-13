import 'dart:io';
import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/plugin_validator.dart';

const _reservedPluginNames = <String>{
  'yorha_protocol.dll',
  'modloader.dll',
  'launch_nier.exe',
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

  static const _prefsKey = 'disabled_plugins';

  static Future<String> pluginsDir() async {
    final base = await LauncherSetupService.getLauncherDirectory();
    return p.join(base, 'nierdecrypt', 'devmods');
  }

  static Future<List<LauncherPlugin>> list() async {
    final dir = await pluginsDir();
    final raw = await IsolateService.run(_listSync, dir);
    if (raw.isEmpty) return const [];
    final disabled = await _loadDisabledSet();
    return raw
        .map((e) => LauncherPlugin(
              name: e.name,
              filePath: e.filePath,
              sizeBytes: e.sizeBytes,
              enabled: !disabled.contains(e.name.toLowerCase()),
            ))
        .toList();
  }

  static Future<List<String>> enabledPaths() async {
    return (await list())
        .where((p) => p.enabled)
        .map((p) => p.filePath)
        .toList();
  }

  static Future<void> setEnabled(String name, bool enabled) async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getStringList(_prefsKey) ?? const <String>[];
    final set = raw.map((s) => s.toLowerCase()).toSet();
    final key = name.toLowerCase();
    if (enabled) {
      set.remove(key);
    } else {
      set.add(key);
    }
    await prefs.setStringList(_prefsKey, set.toList()..sort());
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
    final disabled = await _loadDisabledSet();
    return PluginInstallResult.ok(LauncherPlugin(
      name: name,
      filePath: p.join(dir, name),
      sizeBytes: size,
      enabled: !disabled.contains(name.toLowerCase()),
    ));
  }

  static Future<void> delete(String name) async {
    final dir = await pluginsDir();
    final f = File(p.join(dir, name));
    if (await f.exists()) {
      await f.delete();
    }
  }

  static Future<Set<String>> _loadDisabledSet() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getStringList(_prefsKey) ?? const <String>[];
    return raw.map((s) => s.toLowerCase()).toSet();
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

class _PluginFileEntry {
  final String name;
  final String filePath;
  final int sizeBytes;
  const _PluginFileEntry({
    required this.name,
    required this.filePath,
    required this.sizeBytes,
  });
}

List<_PluginFileEntry> _listSync(String dirPath) {
  final dir = Directory(dirPath);
  if (!dir.existsSync()) return const [];
  final out = <_PluginFileEntry>[];
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
    ));
  }
  out.sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
  return out;
}
