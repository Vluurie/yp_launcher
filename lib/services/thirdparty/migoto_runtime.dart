import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/detection/graphics_dll_id.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/graphics_runtime.dart';
import 'package:yp_launcher/services/thirdparty/ini_patch.dart';
import 'package:yp_launcher/services/thirdparty/shaderfix_names.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_flags.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';

class MigotoRuntime extends GraphicsRuntime {
  const MigotoRuntime({this.loaderTarget = AppStrings.namsExeName});

  final String loaderTarget;

  static const supportDlls = ['nvapi64.dll', 'd3dcompiler_46.dll'];

  @override
  ThirdPartyRuntime get runtime => ThirdPartyRuntime.migoto;

  @override
  String get disableFlagKey => NamsFields.disable3dmigotoLoading.key;

  @override
  String installDir(String gameDir) => ThirdPartyPaths.migoto();

  @override
  bool canInstall(ThirdPartyClassification c) => c.kind == ThirdPartyKind.migoto;

  @override
  Future<ThirdPartyInstallResult> install(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    final dest = installDir(gameDir);
    Directory(dest).createSync(recursive: true);

    _placeDll(c.sourceRoot, dest);
    _placeSupportDlls(c.sourceRoot, dest);
    final added = _mergeShaderFixes(c.sourceRoot, dest);
    _writeD3dx(c.sourceRoot, dest);

    final modName = c.sourceName;
    if (modName != null && modName.isNotEmpty && added.isNotEmpty) {
      ShaderFixNames.assign(dest, added, modName);
    }

    await ThirdPartyFlags.set(gameDir, disableFlagKey, false);

    return ThirdPartyInstallResult(ok: true, runtime: runtime);
  }

  @override
  Future<ThirdPartyUpdateInfo?> wouldUpdate(
    String gameDir,
    ThirdPartyClassification c,
  ) async {
    final incoming = findGraphicsDll(c.sourceRoot, GraphicsDll.migoto);
    if (incoming == null) return null;
    final installed = path.join(installDir(gameDir), 'd3d11.dll');
    if (!File(installed).existsSync()) return null;
    if (!FileOps.filesDiffer(incoming.file.path, installed)) return null;
    return ThirdPartyUpdateInfo(
      runtime: runtime,
      installedLabel: FileOps.sizeLabel(installed),
      incomingLabel: FileOps.sizeLabel(incoming.file.path),
    );
  }

  @override
  Future<void> repair(String gameDir) async {
    final dest = installDir(gameDir);
    if (!Directory(dest).existsSync()) return;
    final ini = File(path.join(dest, migotoIniName));
    final current = ini.existsSync() ? ini.readAsStringSync() : '';
    ini.writeAsStringSync(patchLoaderTarget(current, loaderTarget));
  }

  DllHit? findGameRootDll(String gameDir) {
    for (final name in const ['d3d11.dll', '3dmigoto.dll']) {
      final f = File(path.join(gameDir, name));
      if (f.existsSync() &&
          GraphicsDllId.identifyFile(f.path) == GraphicsDll.migoto) {
        return DllHit(f, false);
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

    FileOps.copyFileInto(hit.file.path, dest, asName: 'd3d11.dll');
    for (final name in [...supportDlls, migotoIniName]) {
      final f = File(path.join(gameDir, name));
      if (f.existsSync()) FileOps.copyFileInto(f.path, dest, asName: name);
    }
    final sf = path.join(gameDir, 'ShaderFixes');
    if (Directory(sf).existsSync()) {
      FileOps.mergeDirectory(sf, path.join(dest, 'ShaderFixes'),
          overwrite: false);
    }

    final ini = File(path.join(dest, migotoIniName));
    final current = ini.existsSync() ? ini.readAsStringSync() : '';
    ini.writeAsStringSync(patchLoaderTarget(current, loaderTarget));

    await ThirdPartyFlags.set(gameDir, disableFlagKey, false);
    return true;
  }

  @override
  Future<ThirdPartyRuntimeStatus> status(String gameDir) async {
    final dir = installDir(gameDir);
    final installed = Directory(dir).existsSync() &&
        (File(path.join(dir, 'd3d11.dll')).existsSync() ||
            File(path.join(dir, '3dmigoto.dll')).existsSync());
    return ThirdPartyRuntimeStatus(
      installed: installed,
      enabled: await isEnabled(gameDir),
      hasShaderFixes: Directory(path.join(dir, 'ShaderFixes')).existsSync(),
      migotoInfo: installed ? _collectInfo(dir) : null,
    );
  }

  MigotoInfo _collectInfo(String dir) {
    final files = <String>[];
    for (final name in const [
      'd3d11.dll',
      '3dmigoto.dll',
      'nvapi64.dll',
      'd3dcompiler_46.dll',
      migotoIniName,
    ]) {
      if (File(path.join(dir, name)).existsSync()) files.add(name);
    }

    var shaderFixCount = 0;
    final fixFiles = <String>[];
    final sf = Directory(path.join(dir, 'ShaderFixes'));
    if (sf.existsSync()) {
      FileOps.walkEntities(sf.path, (e) {
        if (e is File) {
          final lower = e.path.toLowerCase();
          if (lower.endsWith('.txt') || lower.endsWith('.hlsl')) {
            shaderFixCount++;
            fixFiles.add(path.basename(e.path));
          }
        }
        return true;
      });
    }

    ShaderFixNames.prune(dir, fixFiles.toSet());
    final nameMap = ShaderFixNames.read(dir);
    final modNames = <String>{};
    for (final f in fixFiles) {
      modNames.add(nameMap[f] ?? f);
    }
    final shaderFixNames = modNames.toList()..sort();

    String? target;
    var config = const MigotoConfig();
    final ini = File(path.join(dir, migotoIniName));
    if (ini.existsSync()) {
      final content = ini.readAsStringSync();
      target = IniPatch.getKey(content, 'Loader', 'target');
      config = _parseConfig(content);
    }
    final ok = target != null &&
        target.toLowerCase().endsWith(loaderTarget.toLowerCase());

    return MigotoInfo(
      files: files,
      shaderFixCount: shaderFixCount,
      shaderFixNames: shaderFixNames,
      loaderTarget: target,
      loaderTargetOk: ok,
      config: config,
    );
  }

  static MigotoConfig _parseConfig(String content) {
    return MigotoConfig(
      hunting: _huntingFrom(IniPatch.getKey(content, 'Hunting', 'hunting')),
      markingMode:
          _markingFrom(IniPatch.getKey(content, 'Hunting', 'marking_mode')),
      verboseOverlay:
          _boolFrom(IniPatch.getKey(content, 'Logging', 'verbose_overlay')),
      cacheShaders:
          _boolFrom(IniPatch.getKey(content, 'Rendering', 'cache_shaders'),
              orElse: true),
      checkForegroundWindow: _boolFrom(
          IniPatch.getKey(content, 'Device', 'check_foreground_window')),
      reloadFixesKey:
          IniPatch.getKey(content, 'Hunting', 'reload_fixes') ?? '',
      wipeCacheKey:
          IniPatch.getKey(content, 'Hunting', 'wipe_user_config') ?? '',
      toggleHuntKey:
          IniPatch.getKey(content, 'Hunting', 'toggle_hunting') ?? '',
    );
  }

  static MigotoHunting _huntingFrom(String? v) {
    switch (v?.trim()) {
      case '1':
        return MigotoHunting.on;
      case '2':
        return MigotoHunting.noMarking;
      default:
        return MigotoHunting.off;
    }
  }

  static MigotoMarking _markingFrom(String? v) {
    switch (v?.trim().toLowerCase()) {
      case 'original':
        return MigotoMarking.original;
      case 'pink':
        return MigotoMarking.pink;
      case 'mono':
        return MigotoMarking.mono;
      default:
        return MigotoMarking.skip;
    }
  }

  static bool _boolFrom(String? v, {bool orElse = false}) {
    final t = v?.trim().toLowerCase();
    if (t == null || t.isEmpty) return orElse;
    return t == '1' || t == 'true';
  }

  Future<void> applyConfig(String gameDir, MigotoConfig c) async {
    final ini = File(path.join(installDir(gameDir), migotoIniName));
    if (!ini.existsSync()) return;
    var s = ini.readAsStringSync();
    s = IniPatch.setKey(s, 'Hunting', 'hunting', _huntingTo(c.hunting));
    s = IniPatch.setKey(
        s, 'Hunting', 'marking_mode', _markingTo(c.markingMode));
    s = IniPatch.setKey(
        s, 'Logging', 'verbose_overlay', c.verboseOverlay ? '1' : '0');
    s = IniPatch.setKey(
        s, 'Rendering', 'cache_shaders', c.cacheShaders ? '1' : '0');
    s = IniPatch.setKey(s, 'Device', 'check_foreground_window',
        c.checkForegroundWindow ? '1' : '0');
    ini.writeAsStringSync(s);
  }

  static String _huntingTo(MigotoHunting h) {
    switch (h) {
      case MigotoHunting.on:
        return '1';
      case MigotoHunting.noMarking:
        return '2';
      case MigotoHunting.off:
        return '0';
    }
  }

  static String _markingTo(MigotoMarking m) {
    switch (m) {
      case MigotoMarking.original:
        return 'original';
      case MigotoMarking.pink:
        return 'pink';
      case MigotoMarking.mono:
        return 'mono';
      case MigotoMarking.skip:
        return 'skip';
    }
  }

  @override
  Future<void> remove(String gameDir) async {
    FileOps.deleteDirQuiet(installDir(gameDir));
    await ThirdPartyFlags.set(gameDir, disableFlagKey, true);
  }

  void _placeDll(String src, String dest) {
    final dll = findGraphicsDll(src, GraphicsDll.migoto);
    if (dll == null) return;
    FileOps.copyFileInto(dll.file.path, dest, asName: 'd3d11.dll');
    if (dll.file.path.toLowerCase().endsWith('3dmigoto.dll')) {
      FileOps.deleteFileQuiet(path.join(dest, '3dmigoto.dll'));
    }
  }

  void _placeSupportDlls(String src, String dest) {
    for (final name in supportDlls) {
      final f =
          FileOps.filesWhere(src, (rel, _) => baseName(rel) == name).firstOrNull;
      if (f != null) FileOps.copyFileInto(f.path, dest, asName: name);
    }
  }

  List<String> _mergeShaderFixes(String src, String dest) {
    final sf = findDirNamed(src, 'shaderfixes');
    if (sf == null) return const [];
    FileOps.mergeDirectory(
      sf,
      path.join(dest, 'ShaderFixes'),
      overwrite: false,
    );
    final added = <String>[];
    FileOps.walkEntities(sf, (e) {
      if (e is File) {
        final lower = e.path.toLowerCase();
        if (lower.endsWith('.txt') || lower.endsWith('.hlsl')) {
          added.add(path.basename(e.path));
        }
      }
      return true;
    });
    return added;
  }

  void _writeD3dx(String src, String dest) {
    final target = File(path.join(dest, migotoIniName));
    final packIni = FileOps.filesWhere(
      src,
      (rel, _) => baseName(rel) == migotoIniName,
    ).firstOrNull;
    if (!target.existsSync() && packIni != null) {
      target.writeAsStringSync(packIni.readAsStringSync());
    }
    final current = target.existsSync() ? target.readAsStringSync() : '';
    target.writeAsStringSync(patchLoaderTarget(current, loaderTarget));
  }

  static String patchLoaderTarget(String content, String target) {
    final targetLine = 'target = $target';
    if (content.trim().isEmpty) return '[Loader]\n$targetLine\n';

    final lines = content.replaceAll('\r', '').split('\n');
    var loaderIdx = -1;
    for (var i = 0; i < lines.length; i++) {
      if (lines[i].trim().toLowerCase() == '[loader]') {
        loaderIdx = i;
        break;
      }
    }
    if (loaderIdx == -1) return '[Loader]\n$targetLine\n\n$content';

    var sectionEnd = lines.length;
    for (var i = loaderIdx + 1; i < lines.length; i++) {
      final t = lines[i].trim();
      if (t.startsWith('[') && t.endsWith(']')) {
        sectionEnd = i;
        break;
      }
    }

    var replaced = false;
    for (var i = loaderIdx + 1; i < sectionEnd; i++) {
      if (_isTargetLine(lines[i])) {
        lines[i] = targetLine;
        replaced = true;
        break;
      }
    }
    if (!replaced) lines.insert(loaderIdx + 1, targetLine);
    return lines.join('\n');
  }

  static bool _isTargetLine(String line) {
    var t = line.trim();
    if (t.startsWith(';')) t = t.substring(1).trim();
    return t.toLowerCase().startsWith('target') &&
        t.substring('target'.length).trimLeft().startsWith('=');
  }
}

extension _IterableFirstOrNull<E> on Iterable<E> {
  E? get firstOrNull {
    final it = iterator;
    return it.moveNext() ? it.current : null;
  }
}
