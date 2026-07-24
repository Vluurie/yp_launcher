import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/detection/graphics_dll_id.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/graphics_runtime.dart';
import 'package:yp_launcher/services/thirdparty/ini_patch.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_flags.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';

const d3dCompilerName = 'd3dcompiler_47.dll';

class ReShadeRuntime extends GraphicsRuntime {
  const ReShadeRuntime();

  static const gameRootDlls = [
    'ReShade64.dll',
    'ReShade64_Addon.dll',
    'reshade64.dll',
    'dxgi.dll',
    'd3d11.dll',
  ];

  @override
  ThirdPartyRuntime get runtime => ThirdPartyRuntime.reshade;

  @override
  String get disableFlagKey => NamsFields.disableReShadeLoading.key;

  @override
  String installDir(String gameDir) => ThirdPartyPaths.reshade();

  @override
  bool canInstall(ThirdPartyClassification c) =>
      c.kind == ThirdPartyKind.reshadeWholeInstall ||
      c.kind == ThirdPartyKind.reshadePreset;

  @override
  Future<ThirdPartyInstallResult> install(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    final dest = installDir(gameDir);
    Directory(dest).createSync(recursive: true);
    ensureReShadeTree(dest);

    _placeDll(c.sourceRoot, dest);
    _placeShaders(c.sourceRoot, dest);
    _placeAddons(c.sourceRoot, dest);
    _placePresets(c.sourceRoot, dest);
    _placeIni(c.sourceRoot, dest);
    _placeD3dCompiler(c.sourceRoot, dest);

    await ThirdPartyFlags.set(gameDir, disableFlagKey, false);

    return ThirdPartyInstallResult(
      ok: true,
      runtime: runtime,
      shadersMissing: !hasReShadeShaders(dest),
      presetCount: findPresets(c.sourceRoot).length,
    );
  }

  @override
  Future<ThirdPartyUpdateInfo?> wouldUpdate(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    final incoming = _pickDll(c.sourceRoot);
    if (incoming == null) return null;
    final installed = findGraphicsDll(installDir(gameDir), GraphicsDll.reshade);
    if (installed == null) return null;
    if (!FileOps.filesDiffer(incoming.file.path, installed.file.path)) {
      return null;
    }
    return ThirdPartyUpdateInfo(
      runtime: runtime,
      installedLabel: _dllLabel(installed.file),
      incomingLabel: _dllLabel(incoming.file),
    );
  }

  String _dllLabel(File dll) {
    final version = _readVersion(dll);
    final size = FileOps.sizeLabel(dll.path);
    if (version != null) return size == null ? version : '$version · $size';
    return size ?? path.basename(dll.path);
  }

  @override
  Future<void> repair(String gameDir) async {
    final dest = installDir(gameDir);
    if (!Directory(dest).existsSync()) return;
    ensureReShadeTree(dest);
  }

  DllHit? findGameRootDll(String gameDir) {
    for (final name in gameRootDlls) {
      final f = File(path.join(gameDir, name));
      if (f.existsSync() &&
          GraphicsDllId.identifyFile(f.path) == GraphicsDll.reshade) {
        return DllHit(f, name.toLowerCase().contains('_addon'));
      }
    }
    return null;
  }

  @override
  Future<bool> importFromGameRoot(String gameDir) async {
    final hit = findGameRootDll(gameDir);
    if (hit == null) return false;

    final dest = installDir(gameDir);
    Directory(dest).createSync(recursive: true);
    ensureReShadeTree(dest);

    final dllName = hit.isAddon ? 'ReShade64_Addon.dll' : 'ReShade64.dll';
    FileOps.copyFileInto(hit.file.path, dest, asName: dllName);

    for (final name in const [
      reshadeIniName,
      'ReShadePreset.ini',
      'reshade-shaders',
      'reshade-presets',
      'reshade-addons',
      d3dCompilerName,
    ]) {
      final srcPath = path.join(gameDir, name);
      if (FileSystemEntity.isDirectorySync(srcPath)) {
        FileOps.mergeDirectory(srcPath, path.join(dest, name), overwrite: false);
      } else if (File(srcPath).existsSync()) {
        final into =
            name == 'ReShadePreset.ini' ? path.join(dest, 'reshade-presets') : dest;
        FileOps.copyFileInto(srcPath, into);
      }
    }

    final ini = File(path.join(dest, reshadeIniName));
    if (ini.existsSync()) {
      ini.writeAsStringSync(fixIniPaths(ini.readAsStringSync()));
    }

    await ThirdPartyFlags.set(gameDir, disableFlagKey, false);
    return true;
  }

  @override
  Future<ThirdPartyRuntimeStatus> status(String gameDir) async {
    final dir = installDir(gameDir);
    final dllHit = findGraphicsDll(dir, GraphicsDll.reshade);
    final installed = Directory(dir).existsSync() &&
        (dllHit != null ||
            hasReShadeShaders(dir) ||
            File(path.join(dir, reshadeIniName)).existsSync());
    return ThirdPartyRuntimeStatus(
      installed: installed,
      enabled: await isEnabled(gameDir),
      shadersMissing: installed && !hasReShadeShaders(dir),
      presetCount: findPresets(path.join(dir, 'reshade-presets')).length,
      reshadeInfo: installed ? _collectInfo(dir, dllHit) : null,
    );
  }

  ReShadeInfo _collectInfo(String dir, DllHit? dllHit) {
    final presetsDir = path.join(dir, 'reshade-presets');
    final presets = findPresets(presetsDir)
        .map((f) => path.basenameWithoutExtension(f.path))
        .toList()
      ..sort();

    final shadersDir = path.join(dir, 'reshade-shaders', 'Shaders');
    final repos = <String>{};
    var shaderCount = 0;
    final sd = Directory(shadersDir);
    if (sd.existsSync()) {
      for (final e in sd.listSync(recursive: true, followLinks: false)) {
        if (e is File && e.path.toLowerCase().endsWith('.fx')) {
          shaderCount++;
          final rel = path.relative(e.path, from: shadersDir);
          final parts = path.split(rel);
          if (parts.length > 1) repos.add(parts.first);
        }
      }
    }
    final repoList = repos.toList()..sort();

    final addons = FileOps.filesWhere(
      path.join(dir, 'reshade-addons'),
      (rel, _) =>
          rel.endsWith('.addon') ||
          rel.endsWith('.addon32') ||
          rel.endsWith('.addon64'),
    ).map((f) => path.basename(f.path)).toList()
      ..sort();

    final iniFile = File(path.join(dir, reshadeIniName));
    final config = iniFile.existsSync()
        ? _parseConfig(iniFile.readAsStringSync())
        : const ReShadeConfig();

    return ReShadeInfo(
      presets: presets,
      shaderRepos: repoList,
      shaderCount: shaderCount,
      addons: addons,
      isAddonBuild: dllHit?.isAddon ?? false,
      version: dllHit == null ? null : _readVersion(dllHit.file),
      dllName: dllHit == null ? null : path.basename(dllHit.file.path),
      d3dCompilerMissing:
          Platform.isLinux && !File(path.join(dir, d3dCompilerName)).existsSync(),
      config: config,
    );
  }

  static ReShadeConfig _parseConfig(String content) {
    final presetPath = IniPatch.getKey(content, 'GENERAL', 'PresetPath');
    return ReShadeConfig(
      performanceMode:
          _boolFrom(IniPatch.getKey(content, 'GENERAL', 'PerformanceMode')),
      skipLoadingCheck: _boolFrom(
          IniPatch.getKey(content, 'GENERAL', 'NoReloadOnInitException')),
      activePreset: presetPath == null
          ? null
          : path.basenameWithoutExtension(presetPath),
      overlayKey: IniPatch.getKey(content, 'INPUT', 'KeyOverlay') ?? '',
      effectToggleKey: IniPatch.getKey(content, 'INPUT', 'KeyEffects') ?? '',
      screenshotKey: IniPatch.getKey(content, 'INPUT', 'KeyScreenshot') ?? '',
      screenshotPath: IniPatch.getKey(content, 'SCREENSHOT', 'SavePath'),
      screenshotFormat: int.tryParse(
              IniPatch.getKey(content, 'SCREENSHOT', 'FileFormat') ?? '') ??
          1,
      showFps: _boolFrom(IniPatch.getKey(content, 'OVERLAY', 'ShowFPS')),
      showClock: _boolFrom(IniPatch.getKey(content, 'OVERLAY', 'ShowClock')),
    );
  }

  static bool _boolFrom(String? v) {
    final t = v?.trim().toLowerCase();
    return t == '1' || t == 'true';
  }

  Future<void> applyConfig(String gameDir, ReShadeConfig c) async {
    final ini = File(path.join(installDir(gameDir), reshadeIniName));
    if (!ini.existsSync()) return;
    var s = ini.readAsStringSync();
    s = IniPatch.setKey(
        s, 'GENERAL', 'PerformanceMode', c.performanceMode ? '1' : '0');
    s = IniPatch.setKey(s, 'OVERLAY', 'ShowFPS', c.showFps ? '1' : '0');
    s = IniPatch.setKey(s, 'OVERLAY', 'ShowClock', c.showClock ? '1' : '0');
    ini.writeAsStringSync(s);
  }

  String? _readVersion(File dll) {
    try {
      final bytes = dll.readAsBytesSync();
      const needle = [0x52, 0x65, 0x53, 0x68, 0x61, 0x64, 0x65, 0x20];
      for (var i = 0; i < bytes.length - needle.length - 12; i++) {
        var match = true;
        for (var j = 0; j < needle.length; j++) {
          if (bytes[i + j] != needle[j]) {
            match = false;
            break;
          }
        }
        if (!match) continue;
        final sb = StringBuffer('ReShade ');
        for (var k = i + needle.length; k < bytes.length; k++) {
          final c = bytes[k];
          if (c >= 0x30 && c <= 0x39 || c == 0x2E) {
            sb.writeCharCode(c);
          } else {
            break;
          }
        }
        final v = sb.toString();
        if (v.length > 'ReShade '.length) return v;
      }
    } catch (_) {}
    return null;
  }

  @override
  Future<void> remove(String gameDir) async {
    FileOps.deleteDirQuiet(installDir(gameDir));
    await ThirdPartyFlags.set(gameDir, disableFlagKey, true);
  }

  void _placeDll(String src, String dest) {
    final dll = _pickDll(src);
    if (dll == null) return;
    final targetName = dll.isAddon ? 'ReShade64_Addon.dll' : 'ReShade64.dll';
    for (final name in const [
      'reshade64.dll',
      'reshade64_addon.dll',
      'dxgi.dll',
      'd3d11.dll',
    ]) {
      if (name.toLowerCase() != targetName.toLowerCase()) {
        FileOps.deleteFileQuiet(path.join(dest, name));
      }
    }
    FileOps.copyFileInto(dll.file.path, dest, asName: targetName);
  }

  DllHit? _pickDll(String src) {
    final byId = findGraphicsDll(src, GraphicsDll.reshade);
    if (byId != null) return byId;
    final byName = FileOps.filesWhere(src, (rel, _) {
      final b = baseName(rel);
      return b == 'reshade64.dll' || b == 'reshade64_addon.dll';
    }).firstOrNull;
    if (byName == null) return null;
    return DllHit(
      byName,
      path.basename(byName.path).toLowerCase().contains('_addon'),
    );
  }

  void _placeShaders(String src, String dest) {
    final shadersSrc = findDirNamed(src, 'reshade-shaders');
    if (shadersSrc != null) {
      FileOps.mergeDirectory(
        shadersSrc,
        path.join(dest, 'reshade-shaders'),
        overwrite: false,
      );
      return;
    }
    final bareShaders = findDirNamed(src, 'shaders');
    if (bareShaders != null) {
      FileOps.mergeDirectory(
        bareShaders,
        path.join(dest, 'reshade-shaders', 'Shaders'),
        overwrite: false,
      );
    }
    final bareTextures = findDirNamed(src, 'textures');
    if (bareTextures != null) {
      FileOps.mergeDirectory(
        bareTextures,
        path.join(dest, 'reshade-shaders', 'Textures'),
        overwrite: false,
      );
    }
  }

  void _placeAddons(String src, String dest) {
    final addonsSrc = findDirNamed(src, 'reshade-addons');
    if (addonsSrc != null) {
      FileOps.mergeDirectory(
        addonsSrc,
        path.join(dest, 'reshade-addons'),
        overwrite: false,
      );
    }
    for (final af in FileOps.filesWhere(
      src,
      (rel, _) =>
          rel.endsWith('.addon') ||
          rel.endsWith('.addon32') ||
          rel.endsWith('.addon64'),
    )) {
      FileOps.copyFileInto(af.path, path.join(dest, 'reshade-addons'));
    }
  }

  void _placePresets(String src, String dest) {
    final presetsDir = path.join(dest, 'reshade-presets');
    Directory(presetsDir).createSync(recursive: true);
    for (final p in findPresets(src)) {
      final name = uniqueFileName(presetsDir, path.basename(p.path));
      FileOps.copyFileInto(p.path, presetsDir, asName: name);
    }
  }

  void _placeIni(String src, String dest) {
    final target = File(path.join(dest, reshadeIniName));
    if (target.existsSync()) return;
    final packIni = FileOps.filesWhere(
      src,
      (rel, _) => baseName(rel) == reshadeIniName.toLowerCase(),
    ).firstOrNull;
    if (packIni == null) return;
    target.writeAsStringSync(fixIniPaths(packIni.readAsStringSync()));
  }

  void _placeD3dCompiler(String src, String dest) {
    if (File(path.join(dest, d3dCompilerName)).existsSync()) return;
    final dll = FileOps.filesWhere(
      src,
      (rel, _) => baseName(rel) == d3dCompilerName,
    ).firstOrNull;
    if (dll != null) FileOps.copyFileInto(dll.path, dest, asName: d3dCompilerName);
  }

  static String fixIniPaths(String content) {
    final out = <String>[];
    for (final line in content.replaceAll('\r', '').split('\n')) {
      final eq = line.indexOf('=');
      if (eq <= 0) {
        out.add(line);
        continue;
      }
      final rawKey = line.substring(0, eq).trim();
      final key = rawKey.toLowerCase();
      final value = line.substring(eq + 1);
      if (key == 'effectsearchpaths') {
        out.add('EffectSearchPaths=${_fixSearch(value, 'Shaders')}');
      } else if (key == 'texturesearchpaths') {
        out.add('TextureSearchPaths=${_fixSearch(value, 'Textures')}');
      } else if (key == 'intermediatecachepath') {
        out.add(r'IntermediateCachePath=.\reshade-cache');
      } else if (key == 'presetpath' || key == 'currentpresetpath') {
        out.add('PresetPath=${_fixPreset(value)}');
      } else if (key == 'startuppresetpath') {
        out.add('StartupPresetPath=${_fixPreset(value)}');
      } else if (_isAbsolute(value.trim())) {
        out.add('$rawKey=');
      } else {
        out.add(line);
      }
    }
    return '${out.join('\n')}\n';
  }

  static bool _isAbsolute(String path) {
    if (path.isEmpty) return false;
    if (path.startsWith('/') || path.startsWith('\\')) return true;
    return path.length >= 3 &&
        path[1] == ':' &&
        (path[2] == '\\' || path[2] == '/');
  }

  static String _lastSegment(String path) {
    final slash = path.lastIndexOf('/');
    final back = path.lastIndexOf('\\');
    final cut = slash > back ? slash : back;
    return cut == -1 ? path : path.substring(cut + 1);
  }

  static String _fixSearch(String value, String kind) {
    final cleaned = <String>[];
    for (var part in value.split(',')) {
      part = part.trim();
      if (part.isEmpty || _isAbsolute(part)) continue;
      cleaned.add(part);
    }
    if (cleaned.isEmpty) cleaned.add('.\\reshade-shaders\\$kind\\**');
    return cleaned.join(',');
  }

  static String _fixPreset(String value) {
    final v = value.trim();
    if (v.isEmpty || _isAbsolute(v)) {
      return '.\\reshade-presets\\ReShadePreset.ini';
    }
    return '.\\reshade-presets\\${_lastSegment(v)}';
  }
}

extension _IterableFirstOrNull<E> on Iterable<E> {
  E? get firstOrNull {
    final it = iterator;
    return it.moveNext() ? it.current : null;
  }
}
