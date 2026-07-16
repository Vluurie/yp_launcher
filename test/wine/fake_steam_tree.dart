import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/native_steam.dart';

class FakeSteamTree {
  final Directory _dir;

  FakeSteamTree._(this._dir);

  factory FakeSteamTree.create() =>
      FakeSteamTree._(Directory.systemTemp.createTempSync('yp_steam_'));

  String get home => _dir.path;

  String get steamRoot => p.join(home, '.local', 'share', 'Steam');

  String addSteamRoot() {
    Directory(p.join(steamRoot, 'steamapps')).createSync(recursive: true);
    return steamRoot;
  }

  String addNier({String? library}) {
    final dir = p.join(
        library ?? steamRoot, 'steamapps', 'common', 'NieRAutomata');
    File(p.join(dir, 'NieRAutomata.exe'))
      ..parent.createSync(recursive: true)
      ..writeAsStringSync('PRJ_028');
    return dir;
  }

  String addLibraryFolders(String contents) {
    final vdf = p.join(steamRoot, 'steamapps', 'libraryfolders.vdf');
    File(vdf)
      ..parent.createSync(recursive: true)
      ..writeAsStringSync(contents);
    return vdf;
  }

  String addSteamClient() {
    final so = p.join(home, '.steam', 'sdk64', 'steamclient.so');
    File(so)
      ..parent.createSync(recursive: true)
      ..writeAsStringSync('');
    return so;
  }

  String addProton(String name) {
    final proton = p.join(steamRoot, 'compatibilitytools.d', name, 'proton');
    File(proton)
      ..parent.createSync(recursive: true)
      ..writeAsStringSync('#!/bin/sh\n');
    return proton;
  }

  String addCompatPrefix() {
    final pfx = p.join(steamRoot, 'steamapps', 'compatdata', '524220', 'pfx');
    Directory(p.join(pfx, 'drive_c', 'users', 'steamuser', 'AppData', 'Roaming'))
        .createSync(recursive: true);
    Directory(p.join(pfx, 'dosdevices', 'z:')).createSync(recursive: true);
    return pfx;
  }

  T runNative<T>(T Function() body) {
    overrideSteamRoots = [steamRoot];
    try {
      return body();
    } finally {
      overrideSteamRoots = null;
    }
  }

  void dispose() {
    try {
      _dir.deleteSync(recursive: true);
    } catch (_) {}
  }
}
