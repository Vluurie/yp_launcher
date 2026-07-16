import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/crossover.dart';

class FakeBottleTree {
  final Directory _dir;

  FakeBottleTree._(this._dir);

  factory FakeBottleTree.create() =>
      FakeBottleTree._(Directory.systemTemp.createTempSync('yp_bottles_'));

  String get root => _dir.path;

  String addBottle(String name, {String user = 'crossover'}) {
    final bottle = p.join(root, name);
    Directory(p.join(bottle, 'drive_c', 'users', user, 'AppData', 'Roaming'))
        .createSync(recursive: true);
    Directory(p.join(bottle, 'drive_c', 'users', 'Public'))
        .createSync(recursive: true);
    return bottle;
  }

  String addNier(String bottleName) {
    final exe = p.join(
      root,
      bottleName,
      'drive_c',
      'Program Files (x86)',
      'Steam',
      'steamapps',
      'common',
      'NieRAutomata',
      'NieRAutomata.exe',
    );
    File(exe)
      ..parent.createSync(recursive: true)
      ..writeAsStringSync('PRJ_028');
    return exe;
  }

  String addLibraryFolders(String bottleName, String contents) {
    final vdf = p.join(root, bottleName, 'drive_c', 'Program Files (x86)',
        'Steam', 'steamapps', 'libraryfolders.vdf');
    File(vdf)
      ..parent.createSync(recursive: true)
      ..writeAsStringSync(contents);
    return vdf;
  }

  String addWineScript() {
    final script = p.join(root, 'fake-wine');
    final log = p.join(root, 'wine-argv.txt');
    File(script).writeAsStringSync('#!/bin/sh\nprintf "%s\\n" "\$@" > $log\n');
    Process.runSync('chmod', ['+x', script]);
    return script;
  }

  List<String> recordedArgv() {
    final log = File(p.join(root, 'wine-argv.txt'));
    if (!log.existsSync()) return const [];
    return log.readAsLinesSync();
  }

  T run<T>(T Function() body, {String? wineCommand}) {
    overrideBottleRoots = [root];
    overrideWineCommand = wineCommand;
    try {
      return body();
    } finally {
      overrideBottleRoots = null;
      overrideWineCommand = null;
    }
  }

  void dispose() {
    try {
      _dir.deleteSync(recursive: true);
    } catch (_) {}
  }
}
