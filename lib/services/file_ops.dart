import 'dart:io';
import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as path;

class FileOps {
  FileOps._();

  static String? sizeLabel(String filePath) {
    final f = File(filePath);
    if (!f.existsSync()) return null;
    final bytes = f.lengthSync();
    if (bytes >= 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    if (bytes >= 1024) return '${(bytes / 1024).round()} KB';
    return '$bytes B';
  }

  static bool filesDiffer(String a, String b) {
    final fa = File(a);
    final fb = File(b);
    if (!fa.existsSync() || !fb.existsSync()) return true;
    if (fa.lengthSync() != fb.lengthSync()) return true;
    return sha256.convert(fa.readAsBytesSync()).toString() !=
        sha256.convert(fb.readAsBytesSync()).toString();
  }

  static void mergeDirectory(String src, String dest, {bool overwrite = true}) {
    if (!Directory(src).existsSync()) return;
    _mergeInto(src, dest, overwrite, <String>{});
  }

  static void _mergeInto(
    String src,
    String dest,
    bool overwrite,
    Set<String> visited,
  ) {
    try {
      if (!visited.add(Directory(src).resolveSymbolicLinksSync())) return;
    } catch (_) {
      return;
    }
    final List<FileSystemEntity> entries;
    try {
      Directory(dest).createSync(recursive: true);
      entries = Directory(src).listSync(followLinks: false);
    } catch (_) {
      return;
    }
    for (final entity in entries) {
      final target = path.join(dest, path.basename(entity.path));
      final type = FileSystemEntity.typeSync(entity.path);
      if (type == FileSystemEntityType.directory) {
        _mergeInto(entity.path, target, overwrite, visited);
      } else if (type == FileSystemEntityType.file) {
        if (!overwrite && File(target).existsSync()) continue;
        try {
          File(entity.path).copySync(target);
        } catch (_) {}
      }
    }
  }

  static void copyDirectory(String src, String dest) {
    mergeDirectory(src, dest, overwrite: true);
  }

  static void moveDirectory(String src, String dest) {
    final srcDir = Directory(src);
    if (!srcDir.existsSync()) return;
    Directory(dest).parent.createSync(recursive: true);
    try {
      srcDir.renameSync(dest);
      return;
    } catch (_) {}
    mergeDirectory(src, dest, overwrite: true);
    deleteDirQuiet(src);
  }

  static List<String> moveMerging(String src, String dest) {
    final left = <String>[];
    _moveMerging(src, dest, left);
    return left;
  }

  static void _moveMerging(String src, String dest, List<String> left) {
    final type = FileSystemEntity.typeSync(src, followLinks: false);
    final destType = FileSystemEntity.typeSync(dest, followLinks: false);

    if (type == FileSystemEntityType.directory) {
      final dir = Directory(src);
      if (destType == FileSystemEntityType.notFound) {
        try {
          Directory(dest).parent.createSync(recursive: true);
          dir.renameSync(dest);
          return;
        } catch (_) {}
      } else if (destType != FileSystemEntityType.directory) {
        left.add(src);
        return;
      }
      final List<FileSystemEntity> children;
      try {
        Directory(dest).createSync(recursive: true);
        children = dir.listSync(followLinks: false);
      } catch (_) {
        left.add(src);
        return;
      }
      for (final child in children) {
        _moveMerging(
          child.path,
          path.join(dest, path.basename(child.path)),
          left,
        );
      }
      try {
        dir.deleteSync();
      } catch (_) {}
      return;
    }

    if (type != FileSystemEntityType.file) {
      if (type != FileSystemEntityType.notFound) left.add(src);
      return;
    }

    final file = File(src);
    if (destType != FileSystemEntityType.notFound) {
      if (destType == FileSystemEntityType.file && !filesDiffer(src, dest)) {
        try {
          file.deleteSync();
        } catch (_) {}
      } else {
        left.add(src);
      }
      return;
    }
    try {
      Directory(path.dirname(dest)).createSync(recursive: true);
      file.renameSync(dest);
      return;
    } catch (_) {}
    try {
      file.copySync(dest);
      if (File(dest).lengthSync() != file.lengthSync()) {
        throw const FileSystemException('incomplete copy');
      }
    } catch (_) {
      try {
        File(dest).deleteSync();
      } catch (_) {}
      left.add(src);
      return;
    }
    try {
      file.deleteSync();
    } catch (_) {}
  }

  static void copyFileInto(String srcFile, String destDir, {String? asName}) {
    final f = File(srcFile);
    if (!f.existsSync()) return;
    Directory(destDir).createSync(recursive: true);
    final name = asName ?? path.basename(srcFile);
    try {
      f.copySync(path.join(destDir, name));
    } catch (_) {}
  }

  static void deleteDirQuiet(String dir) {
    final d = Directory(dir);
    if (d.existsSync()) {
      try {
        d.deleteSync(recursive: true);
      } catch (_) {}
    }
  }

  static void deleteFileQuiet(String file) {
    final f = File(file);
    if (f.existsSync()) {
      try {
        f.deleteSync();
      } catch (_) {}
    }
  }

  static String unwrapSingleChild(String root) {
    final dir = Directory(root);
    if (!dir.existsSync()) return root;
    final entries = dir.listSync(followLinks: false);
    final dirs = entries.whereType<Directory>().toList();
    final files = entries.whereType<File>().where((f) {
      final lower = path.basename(f.path).toLowerCase();
      return !lower.endsWith('.txt') &&
          !lower.endsWith('.md') &&
          !lower.endsWith('.pdf') &&
          !lower.endsWith('.url') &&
          !lower.startsWith('readme') &&
          !lower.startsWith('__macosx');
    }).toList();
    if (dirs.length == 1 && files.isEmpty) {
      return dirs.first.path;
    }
    return root;
  }

  static List<File> filesWhere(
    String root,
    bool Function(String relLower, String fullPath) predicate,
  ) {
    final out = <File>[];
    _walkFiles(root, (file) {
      final rel = path.relative(file.path, from: root).replaceAll('\\', '/');
      if (predicate(rel.toLowerCase(), file.path)) out.add(file);
      return true;
    });
    return out;
  }

  static bool anyFile(
    String root,
    bool Function(String relLower, String fullPath) predicate,
  ) {
    var found = false;
    _walkFiles(root, (file) {
      final rel = path.relative(file.path, from: root).replaceAll('\\', '/');
      if (predicate(rel.toLowerCase(), file.path)) {
        found = true;
        return false;
      }
      return true;
    });
    return found;
  }

  static void _walkFiles(String root, bool Function(File file) visit) {
    walkEntities(root, (e) {
      if (e is File) return visit(e);
      return true;
    });
  }

  static void walkEntities(
    String root,
    bool Function(FileSystemEntity entity) visit,
  ) {
    final dir = Directory(root);
    if (!dir.existsSync()) return;
    final stack = <Directory>[dir];
    while (stack.isNotEmpty) {
      final current = stack.removeLast();
      List<FileSystemEntity> entries;
      try {
        entries = current.listSync(followLinks: false);
      } catch (_) {
        continue;
      }
      for (final e in entries) {
        if (e is Directory) {
          if (!visit(e)) return;
          stack.add(e);
        } else if (!visit(e)) {
          return;
        }
      }
    }
  }
}
