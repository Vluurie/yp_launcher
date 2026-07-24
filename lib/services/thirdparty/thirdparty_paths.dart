import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/detection/graphics_dll_id.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';

const reshadeIniName = 'ReShade.ini';
const migotoIniName = 'd3dx.ini';

class DllHit {
  final File file;
  final bool isAddon;
  const DllHit(this.file, this.isAddon);
}

class ThirdPartyPaths {
  ThirdPartyPaths._();

  static String root() =>
      path.join(LauncherSetupService.launcherDirectory,
          AppStrings.thirdPartyDirName);

  static String reshade() =>
      path.join(root(), AppStrings.reshadeDirName);

  static String migoto() =>
      path.join(root(), AppStrings.migotoDirName);
}

String baseName(String rel) {
  final i = rel.lastIndexOf('/');
  return i == -1 ? rel : rel.substring(i + 1);
}

void ensureReShadeTree(String dest) {
  for (final sub in const [
    'reshade-presets',
    'reshade-shaders/Shaders',
    'reshade-shaders/Textures',
    'reshade-addons',
    'reshade-cache',
  ]) {
    Directory(path.join(dest, sub.replaceAll('/', path.separator)))
        .createSync(recursive: true);
  }
}

bool hasFileMatching(String root, bool Function(String relLower) pred) =>
    FileOps.anyFile(root, (rel, _) => pred(rel));

bool hasDirNamed(String root, String nameLower) =>
    findDirNamed(root, nameLower) != null;

String? findDirNamed(String root, String nameLower) {
  String? hit;
  FileOps.walkEntities(root, (e) {
    if (e is Directory && path.basename(e.path).toLowerCase() == nameLower) {
      hit = e.path;
      return false;
    }
    return true;
  });
  return hit;
}

bool hasReShadeShaders(String root) {
  final shaders = findDirNamed(root, 'shaders');
  if (shaders == null) return false;
  return FileOps.anyFile(shaders, (rel, _) => rel.endsWith('.fx'));
}

bool hasAddonFiles(String root) => FileOps.anyFile(
      root,
      (rel, _) =>
          rel.endsWith('.addon') ||
          rel.endsWith('.addon32') ||
          rel.endsWith('.addon64'),
    );

List<File> findPresets(String root) {
  return FileOps.filesWhere(root, (rel, full) {
    if (!rel.endsWith('.ini')) return false;
    try {
      final content = File(full).readAsStringSync();
      return RegExp(r'^\s*Techniques\s*=', multiLine: true).hasMatch(content);
    } catch (_) {
      return false;
    }
  });
}

bool onlyImages(String root) {
  final files = FileOps.filesWhere(root, (rel, _) {
    final b = baseName(rel);
    return !b.endsWith('.txt') &&
        !b.endsWith('.md') &&
        !b.endsWith('.pdf') &&
        !b.startsWith('readme');
  });
  if (files.isEmpty) return false;
  return files.every((f) {
    final b = path.basename(f.path).toLowerCase();
    return b.endsWith('.png') ||
        b.endsWith('.jpg') ||
        b.endsWith('.jpeg') ||
        b.endsWith('.dds');
  });
}

DllHit? findGraphicsDll(String root, GraphicsDll want) {
  DllHit? hit;
  FileOps.walkEntities(root, (e) {
    if (e is! File) return true;
    final name = path.basename(e.path).toLowerCase();
    if (!name.endsWith('.dll')) return true;
    if (name == 'nvapi64.dll' || name == 'd3dcompiler_46.dll') return true;
    if (GraphicsDllId.identifyFile(e.path) == want) {
      hit = DllHit(e, name.contains('_addon'));
      return false;
    }
    return true;
  });
  return hit;
}

List<String> findInnerArchives(String root) {
  return FileOps.filesWhere(root, (rel, full) {
    final b = baseName(rel);
    return b.endsWith('.zip') || b.endsWith('.7z') || b.endsWith('.rar');
  }).map((f) => f.path).toList();
}

String uniqueFileName(String dir, String fileName) {
  if (!File(path.join(dir, fileName)).existsSync()) return fileName;
  final ext = path.extension(fileName);
  final base = path.basenameWithoutExtension(fileName);
  var i = 2;
  var candidate = '$base ($i)$ext';
  while (File(path.join(dir, candidate)).existsSync()) {
    i++;
    candidate = '$base ($i)$ext';
  }
  return candidate;
}
