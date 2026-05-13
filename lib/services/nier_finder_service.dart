import 'dart:async';
import 'dart:collection';
import 'dart:io';
import 'dart:isolate';

import 'package:path/path.dart' as path;

class NierInstallation {
  final String path;
  final bool hasData;

  const NierInstallation({required this.path, required this.hasData});
}

class NierFinderService {
  static const _steamAppId = '524220';

  static Future<List<NierInstallation>> findFast() async {
    final seen = <String>{};
    final results = <NierInstallation>[];

    for (final i in await _findViaRegistry()) {
      if (seen.add(i.path.toLowerCase())) results.add(i);
    }
    for (final i in await _findViaSteamLibraries()) {
      if (seen.add(i.path.toLowerCase())) results.add(i);
    }

    return results;
  }

  static Future<List<NierInstallation>> findDeep() async {
    return _findViaBfs();
  }

  static Future<List<NierInstallation>> _findViaRegistry() async {
    try {
      final result = await Process.run('reg', [
        'query',
        'HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App $_steamAppId',
        '/v',
        'InstallLocation',
      ]);

      if (result.exitCode == 0) {
        final output = result.stdout as String;
        final match = RegExp(
          r'InstallLocation\s+REG_SZ\s+(.+)',
        ).firstMatch(output);
        if (match != null) {
          final installPath = match.group(1)!.trim();
          final exe = File(path.join(installPath, 'NieRAutomata.exe'));
          if (exe.existsSync()) {
            final dataDir = Directory(path.join(installPath, 'data'));
            return [
              NierInstallation(
                path: path.normalize(installPath),
                hasData: dataDir.existsSync(),
              ),
            ];
          }
        }
      }
    } catch (_) {}
    return [];
  }

  static Future<List<NierInstallation>> _findViaSteamLibraries() async {
    try {
      final steamResult = await Process.run('reg', [
        'query',
        'HKCU\\SOFTWARE\\Valve\\Steam',
        '/v',
        'SteamPath',
      ]);

      if (steamResult.exitCode != 0) return [];

      final output = steamResult.stdout as String;
      final match = RegExp(r'SteamPath\s+REG_SZ\s+(.+)').firstMatch(output);
      if (match == null) return [];

      final steamPath = match.group(1)!.trim();
      final libraryFolders = File(
        path.join(steamPath, 'steamapps', 'libraryfolders.vdf'),
      );
      if (!libraryFolders.existsSync()) return [];

      final content = libraryFolders.readAsStringSync();
      final pathMatches = RegExp(r'"path"\s+"([^"]+)"').allMatches(content);
      final results = <NierInstallation>[];

      for (final m in pathMatches) {
        final libPath = m.group(1)!.replaceAll('\\\\', '\\');
        final nierPath = path.join(
          libPath,
          'steamapps',
          'common',
          'NieRAutomata',
        );
        final exe = File(path.join(nierPath, 'NieRAutomata.exe'));
        if (exe.existsSync()) {
          final dataDir = Directory(path.join(nierPath, 'data'));
          results.add(
            NierInstallation(
              path: path.normalize(nierPath),
              hasData: dataDir.existsSync(),
            ),
          );
        }
      }

      return results;
    } catch (_) {}
    return [];
  }

  static Future<List<NierInstallation>> _findViaBfs() async {
    final receivePort = ReceivePort();
    final results = <NierInstallation>[];
    final isolates = <Isolate>[];
    final completer = Completer<void>();
    int completedDrives = 0;

    final drives = <String>[];
    for (int i = 67; i <= 90; i++) {
      final drivePath = '${String.fromCharCode(i)}:\\';
      if (Directory(drivePath).existsSync()) {
        drives.add(drivePath);
      }
    }

    if (drives.isEmpty) {
      return results;
    }

    final totalDrives = drives.length;

    receivePort.listen((message) {
      if (message is List<Map<String, String>>) {
        for (final entry in message) {
          results.add(
            NierInstallation(
              path: entry['path']!,
              hasData: entry['hasData'] == 'true',
            ),
          );
        }
      }
      if (message is String && message == '_done') {
        completedDrives++;
        if (completedDrives >= totalDrives && !completer.isCompleted) {
          completer.complete();
        }
      }
    });

    for (final drive in drives) {
      final isolate = await Isolate.spawn(_searchDriveEntry, [
        drive,
        receivePort.sendPort,
      ]);
      isolates.add(isolate);
    }

    try {
      await completer.future.timeout(const Duration(seconds: 30));
    } catch (_) {}

    for (final isolate in isolates) {
      isolate.kill(priority: Isolate.immediate);
    }
    receivePort.close();

    return results;
  }

  static void _searchDriveEntry(List<dynamic> args) {
    final String drive = args[0];
    final SendPort sendPort = args[1];

    try {
      final found = _bfsSearch(Directory(drive));
      if (found.isNotEmpty) {
        sendPort.send(found);
      }
    } catch (_) {}

    sendPort.send('_done');
  }

  static List<Map<String, String>> _bfsSearch(Directory root) {
    final results = <Map<String, String>>[];
    final queue = Queue<Directory>();
    queue.add(root);
    final visited = <String>{};

    while (queue.isNotEmpty) {
      final current = queue.removeFirst();

      try {
        final entities = current.listSync(followLinks: false);

        for (final entity in entities) {
          if (entity is File) {
            final name = path.basename(entity.path);
            if (name.toLowerCase() == 'nierautomata.exe') {
              final parentDir = path.dirname(entity.path);
              final dataDir = Directory(path.join(parentDir, 'data'));
              results.add({
                'path': path.normalize(parentDir),
                'hasData': dataDir.existsSync().toString(),
              });
            }
          } else if (entity is Directory) {
            final dirName = path.basename(entity.path).toLowerCase();
            if (dirName.startsWith('.') ||
                dirName == '\$recycle.bin' ||
                dirName == 'windows' ||
                dirName == 'system volume information' ||
                dirName == 'recovery' ||
                dirName == 'programdata' ||
                dirName == 'appdata') {
              continue;
            }
            final normalized = path.normalize(entity.path);
            if (visited.add(normalized)) {
              queue.add(entity);
            }
          }
        }
      } catch (_) {
        continue;
      }
    }

    return results;
  }
}
