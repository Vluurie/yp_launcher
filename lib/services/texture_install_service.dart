import 'dart:io';
import 'dart:isolate';
import 'package:archive/archive.dart';
import 'package:path/path.dart' as path;

const archiveExts = ['.zip', '.tar', '.gz', '.bz2', '.7z', '.rar'];
const textureExts = ['.dds'];

bool isArchive(String p) =>
    archiveExts.any((e) => p.toLowerCase().endsWith(e));
bool isTexture(String p) =>
    textureExts.any((e) => p.toLowerCase().endsWith(e));

class TextureInstallProgressParams {
  final List<String> paths;
  final String textureDir;
  final SendPort sendPort;
  const TextureInstallProgressParams({
    required this.paths,
    required this.textureDir,
    required this.sendPort,
  });
}

class LoadTexturesParams {
  final String textureDir;
  final String skResDir;
  final String waxModsDir;
  final List<String> disabledPacks;
  const LoadTexturesParams({
    required this.textureDir,
    required this.skResDir,
    required this.waxModsDir,
    this.disabledPacks = const [],
  });
}

bool _containsDds(Directory dir) {
  try {
    for (final entity in dir.listSync(recursive: true, followLinks: false)) {
      if (entity is File && entity.path.toLowerCase().endsWith('.dds')) {
        return true;
      }
    }
  } catch (_) {}
  return false;
}

Map<String, dynamic> loadTexturesIsolate(LoadTexturesParams params) {
  final disabled = params.disabledPacks.toSet();
  final namsEntries = <String>[];
  final namsFolders = <String>[];
  final namsDir = Directory(params.textureDir);
  if (namsDir.existsSync()) {
    for (final entity in namsDir.listSync()) {
      final name = path.basename(entity.path);
      if (disabled.contains(name)) continue;
      namsEntries.add(name);
      if (entity is Directory) {
        namsFolders.add(name);
      }
    }
    namsEntries.sort();
    namsFolders.sort();
  }

  final skEntries = <String>[];
  final skTexDir = Directory(params.skResDir);
  if (skTexDir.existsSync()) {
    for (final entity in skTexDir.listSync()) {
      if (entity is Directory) {
        skEntries.add(path.basename(entity.path));
      }
    }
    skEntries.sort();
  }

  final waxEntries = <String>[];
  final waxDir = Directory(params.waxModsDir);
  if (waxDir.existsSync()) {
    for (final entity in waxDir.listSync()) {
      if (entity is! Directory) continue;
      final name = path.basename(entity.path);
      if (name == '!base') continue;
      if (_containsDds(entity)) {
        waxEntries.add(name);
      }
    }
    waxEntries.sort();
  }

  final conflicts = <String, List<String>>{};
  final uniqueFolders = namsFolders.toSet().toList();
  if (uniqueFolders.length > 1) {
    final fileOwners = <String, Set<String>>{};
    for (final folder in uniqueFolders) {
      final folderDir = Directory(path.join(params.textureDir, folder));
      if (!folderDir.existsSync()) continue;
      for (final file in folderDir.listSync(recursive: true)) {
        if (file is! File) continue;
        final rel = path.basename(file.path).toLowerCase();
        fileOwners.putIfAbsent(rel, () => {}).add(folder);
      }
    }
    for (final entry in fileOwners.entries) {
      if (entry.value.length > 1) {
        for (final folder in entry.value) {
          conflicts.putIfAbsent(folder, () => []);
          for (final other in entry.value) {
            if (other != folder && !conflicts[folder]!.contains(other)) {
              conflicts[folder]!.add(other);
            }
          }
        }
      }
    }
  }

  final conflictStrings = <String>[];
  for (final entry in conflicts.entries) {
    conflictStrings.add('${entry.key}|${entry.value.join(",")}');
  }

  return {
    'nams': namsEntries,
    'sk': skEntries,
    'wax': waxEntries,
    'namsFolders': namsFolders,
    'conflicts': conflictStrings,
  };
}

bool deleteTextureIsolate(String texturePath) {
  final dir = Directory(texturePath);
  if (dir.existsSync()) {
    dir.deleteSync(recursive: true);
    return true;
  }
  final file = File(texturePath);
  if (file.existsSync()) {
    file.deleteSync();
    return true;
  }
  return false;
}

void installTexturesWithProgress(TextureInstallProgressParams params) {
  final dir = Directory(params.textureDir);
  if (!dir.existsSync()) dir.createSync(recursive: true);

  final allFiles = <(File, String)>[];
  for (final entry in params.paths) {
    String sourcePath;
    String? folderName;
    if (entry.contains('|')) {
      final parts = entry.split('|');
      sourcePath = parts[0];
      folderName = parts[1];
    } else {
      sourcePath = entry;
    }

    if (isArchive(sourcePath)) {
      final extracted = extractArchiveSync(sourcePath);
      if (extracted == null) continue;
      folderName ??= path.basenameWithoutExtension(sourcePath);
      collectTextureFiles(extracted, folderName, allFiles);
    } else if (Directory(sourcePath).existsSync()) {
      folderName ??= path.basename(sourcePath);
      collectTextureFiles(sourcePath, folderName, allFiles);
    } else if (File(sourcePath).existsSync() && isTexture(sourcePath)) {
      allFiles.add((File(sourcePath), ''));
    }
  }

  if (allFiles.isEmpty) {
    params.sendPort.send({'type': 'done', 'count': 0});
    return;
  }

  int totalBytes = 0;
  for (final (file, _) in allFiles) {
    totalBytes += file.lengthSync();
  }

  int copiedBytes = 0;
  int filesDone = 0;
  int lastReport = 0;

  for (final (file, folderName) in allFiles) {
    final String targetPath;
    if (folderName.isEmpty) {
      targetPath = path.join(params.textureDir, path.basename(file.path));
    } else {
      final targetDir = Directory(path.join(params.textureDir, folderName));
      targetDir.createSync(recursive: true);
      final baseName = path.basename(file.path);
      targetPath = path.join(targetDir.path, baseName);
    }

    File(targetPath).parent.createSync(recursive: true);

    final input = file.openSync();
    final output = File(targetPath).openSync(mode: FileMode.write);
    const chunkSize = 1024 * 1024;

    while (true) {
      final chunk = input.readSync(chunkSize);
      if (chunk.isEmpty) break;
      output.writeFromSync(chunk);
      copiedBytes += chunk.length;

      if (copiedBytes - lastReport > 2 * 1024 * 1024) {
        lastReport = copiedBytes;
        params.sendPort.send({
          'type': 'progress',
          'copied': copiedBytes,
          'total': totalBytes,
          'files': filesDone,
          'totalFiles': allFiles.length,
        });
      }
    }

    input.closeSync();
    output.closeSync();
    filesDone++;
  }

  params.sendPort.send({
    'type': 'progress',
    'copied': totalBytes,
    'total': totalBytes,
    'files': filesDone,
    'totalFiles': allFiles.length,
  });
  params.sendPort.send({'type': 'done', 'count': filesDone});
}

void collectTextureFiles(String sourcePath, String folderName, List<(File, String)> result) {
  final sourceDir = Directory(sourcePath);
  if (!sourceDir.existsSync()) return;
  for (final file in sourceDir.listSync(recursive: true)) {
    if (file is! File || !isTexture(file.path)) continue;
    final relativePath = path.relative(file.path, from: sourcePath);
    final combined = path.join(folderName, relativePath);
    result.add((file, combined));
  }
}

String? extractArchiveSync(String archivePath) {
  try {
    final bytes = File(archivePath).readAsBytesSync();
    final archive = ZipDecoder().decodeBytes(bytes);
    final tempDir = Directory.systemTemp.createTempSync('texture_');
    for (final file in archive) {
      final outPath = path.join(tempDir.path, file.name);
      if (file.isFile) {
        final outFile = File(outPath);
        outFile.parent.createSync(recursive: true);
        outFile.writeAsBytesSync(file.content as List<int>);
      } else {
        Directory(outPath).createSync(recursive: true);
      }
    }
    return tempDir.path;
  } catch (_) {
    return null;
  }
}
