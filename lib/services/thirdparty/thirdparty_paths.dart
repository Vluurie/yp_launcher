import 'dart:io';
import 'dart:typed_data';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/detection/graphics_dll_id.dart';
import 'package:yp_launcher/services/file_ops.dart';

const reshadeIniName = 'ReShade.ini';
const migotoIniName = 'd3dx.ini';

class DllHit {
  final File file;
  final bool isAddon;
  const DllHit(this.file, this.isAddon);
}

/// Third-party mods live in `<game>\nams\thirdparty\<kind>\<mod>\`, one folder
/// per mod. NAMS loads the mod folders of a kind sorted by name and reads the
/// `.dll` files directly inside each one.
class ThirdPartyPaths {
  ThirdPartyPaths._();

  static String root(String gameDir) =>
      path.join(gameDir, 'nams', AppStrings.thirdPartyDirName);

  /// `thirdparty\` next to NAMS.exe, which NAMS still loads as a fallback.
  static String legacyRoot(String launcherDir) =>
      path.join(launcherDir, AppStrings.thirdPartyDirName);

  static String reshade(String gameDir) =>
      path.join(root(gameDir), AppStrings.reshadeDirName);

  static String migoto(String gameDir) =>
      path.join(root(gameDir), AppStrings.migotoDirName);

  static String gameMods(String gameDir) =>
      path.join(root(gameDir), AppStrings.gameModsDirName);

  static String graphicMods(String gameDir) =>
      path.join(root(gameDir), AppStrings.graphicModsDirName);

  /// Outside the `graphic` folder, so NAMS never sees a turned-off mod.
  static String graphicModsDisabled(String gameDir) => path.join(
    root(gameDir),
    AppStrings.thirdPartyDisabledDirName,
    AppStrings.graphicModsDirName,
  );

  static List<Directory> modDirs(String kindDir) {
    final dir = Directory(kindDir);
    if (!dir.existsSync()) return const [];
    final List<Directory> dirs;
    try {
      dirs = dir.listSync(followLinks: false).whereType<Directory>().toList();
    } catch (_) {
      return const [];
    }
    dirs.sort(
      (a, b) => path
          .basename(a.path)
          .toLowerCase()
          .compareTo(path.basename(b.path).toLowerCase()),
    );
    return dirs;
  }

  static List<File> dllsIn(String dir) {
    final d = Directory(dir);
    if (!d.existsSync()) return const [];
    final List<File> dlls;
    try {
      dlls = d
          .listSync(followLinks: false)
          .whereType<File>()
          .where((f) => f.path.toLowerCase().endsWith('.dll'))
          .toList();
    } catch (_) {
      return const [];
    }
    dlls.sort(
      (a, b) => path
          .basename(a.path)
          .toLowerCase()
          .compareTo(path.basename(b.path).toLowerCase()),
    );
    return dlls;
  }

  /// The first mod folder of [kindDir] that holds a DLL named like one of
  /// [dllNames].
  static String? modDirWithDll(String kindDir, Iterable<String> dllNames) {
    final wanted = dllNames.map((n) => n.toLowerCase()).toSet();
    for (final dir in modDirs(kindDir)) {
      if (dllsIn(
        dir.path,
      ).any((f) => wanted.contains(path.basename(f.path).toLowerCase()))) {
        return dir.path;
      }
    }
    return null;
  }

  /// The first DLL named [fileName] in a mod folder of [kindDir].
  static String? findDll(String kindDir, String fileName) {
    final wanted = fileName.toLowerCase();
    for (final dir in modDirs(kindDir)) {
      for (final dll in dllsIn(dir.path)) {
        if (path.basename(dll.path).toLowerCase() == wanted) return dll.path;
      }
    }
    return null;
  }

  /// [raw] as a folder name Windows accepts, or [fallback] when nothing is left.
  static String modDirName(String raw, {required String fallback}) {
    final name = raw
        .replaceAll(RegExp(r'[<>:"/\\|?*\x00-\x1F]'), ' ')
        .replaceAll(RegExp(r'\s+'), ' ')
        .replaceAll(RegExp(r'[. ]+$'), '')
        .trim();
    return name.isEmpty ? fallback : name;
  }

  /// [name], or `name (2)`, `name (3)`, ... when [parent] already has an entry
  /// of that name.
  static String uniqueDirName(String parent, String name) {
    bool taken(String n) =>
        FileSystemEntity.typeSync(path.join(parent, n), followLinks: false) !=
        FileSystemEntityType.notFound;
    if (!taken(name)) return name;
    var i = 2;
    while (taken('$name ($i)')) {
      i++;
    }
    return '$name ($i)';
  }

  /// Whether [name] is a single folder name, so joining it to a kind folder
  /// cannot point anywhere else.
  static bool isPlainName(String name) =>
      name.isNotEmpty &&
      name != '.' &&
      name != '..' &&
      path.basename(name) == name &&
      !name.contains('/') &&
      !name.contains('\\');
}

const directXProxyNames = <String>{
  'd3d9.dll',
  'd3d10.dll',
  'd3d11.dll',
  'd3d12.dll',
  'dxgi.dll',
};

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
    Directory(
      path.join(dest, sub.replaceAll('/', path.separator)),
    ).createSync(recursive: true);
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

const _specialKMarker = 'SpecialK';

bool isSpecialKDll(String filePath) {
  try {
    final f = File(filePath);
    if (!f.existsSync()) return false;
    return _containsAscii(f.readAsBytesSync(), _specialKMarker);
  } catch (_) {
    return false;
  }
}

bool _containsAscii(Uint8List haystack, String needle) {
  final n = needle.codeUnits;
  if (n.isEmpty || n.length > haystack.length) return false;
  final last = haystack.length - n.length;
  for (var i = 0; i <= last; i++) {
    if (haystack[i] != n[0]) continue;
    var match = true;
    for (var j = 1; j < n.length; j++) {
      if (haystack[i + j] != n[j]) {
        match = false;
        break;
      }
    }
    if (match) return true;
  }
  return false;
}

List<String> findGameModDlls(String root) {
  final out = FileOps.filesWhere(root, (rel, full) {
    if (!baseName(rel).endsWith('.dll')) return false;
    return GraphicsDllId.identifyFile(full) == GraphicsDll.none;
  }).map((f) => f.path).toList();
  out.sort(
    (a, b) => path
        .basename(a)
        .toLowerCase()
        .compareTo(path.basename(b).toLowerCase()),
  );
  return out;
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
