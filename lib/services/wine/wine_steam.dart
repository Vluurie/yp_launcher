import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/nier_installation.dart';
import 'package:yp_launcher/services/wine/crossover.dart';
import 'package:yp_launcher/services/wine/steam_vdf.dart';

const _programFilesVariants = ['Program Files (x86)', 'Program Files'];

final _windowsDrive = RegExp(r'^([a-zA-Z]):\\');

/// Turns a Windows path out of libraryfolders.vdf into a host path.
/// `C:` resolves to the prefix's drive_c, any other letter through
/// `dosdevices/<letter>:`, which is what makes libraries on other volumes work.
String normalizeSteamLibraryPath(String vdfPath, String prefix) {
  final unescaped = vdfPath.replaceAll(r'\"', '"').replaceAll(r'\\', '\\');

  final match = _windowsDrive.firstMatch(unescaped);
  if (match == null) return unescaped;

  final drive = match.group(1)!.toLowerCase();
  final segments =
      unescaped.substring(3).split('\\').where((part) => part.isNotEmpty);
  final driveRoot = drive == 'c'
      ? p.join(prefix, 'drive_c')
      : p.join(prefix, 'dosdevices', '$drive:');

  return p.joinAll([driveRoot, ...segments]);
}

List<String> parseLibraryFoldersInPrefix(String prefix) {
  final libraries = <String>[];
  for (final programFiles in _programFilesVariants) {
    final vdf = File(p.join(prefix, 'drive_c', programFiles, 'Steam',
        'steamapps', 'libraryfolders.vdf'));
    if (!vdf.existsSync()) continue;
    try {
      for (final path in vdfPathEntries(vdf.readAsStringSync())) {
        libraries.add(normalizeSteamLibraryPath(path, prefix));
      }
    } catch (_) {}
  }
  return libraries;
}

List<String> nierDirsInPrefix(String prefix) {
  final dirs = <String>{};

  for (final exe in nierCandidatesInBottle(prefix)) {
    if (File(exe).existsSync()) dirs.add(p.normalize(p.dirname(exe)));
  }

  for (final library in parseLibraryFoldersInPrefix(prefix)) {
    final dir = p.join(library, 'steamapps', 'common', 'NieRAutomata');
    if (File(p.join(dir, AppStrings.gameExeName)).existsSync()) {
      dirs.add(p.normalize(dir));
    }
  }

  return dirs.toList();
}

List<NierInstallation> findNierInPrefixes() {
  final seen = <String>{};
  final installations = <NierInstallation>[];

  for (final bottle in listCrossOverBottles()) {
    for (final dir in nierDirsInPrefix(bottle.path)) {
      final normalized = p.normalize(dir);
      if (!seen.add(normalized.toLowerCase())) continue;
      installations.add(NierInstallation(
        path: normalized,
        hasData: Directory(p.join(normalized, 'data')).existsSync(),
      ));
    }
  }

  return installations;
}

/// Prefix roots worth a deep scan: each bottle's drive_c plus whatever its
/// dosdevices letters point at.
List<String> deepScanRoots() {
  final roots = <String>{};
  for (final bottle in listCrossOverBottles()) {
    final driveC = p.join(bottle.path, 'drive_c');
    if (Directory(driveC).existsSync()) roots.add(driveC);

    final dosDevices = Directory(p.join(bottle.path, 'dosdevices'));
    if (!dosDevices.existsSync()) continue;
    try {
      for (final link in dosDevices.listSync(followLinks: false)) {
        final name = p.basename(link.path).toLowerCase();
        if (name == 'c:' || name == 'z:') continue;
        final resolved = _resolve(link.path);
        if (resolved != null && Directory(resolved).existsSync()) {
          roots.add(resolved);
        }
      }
    } catch (_) {}
  }
  return roots.toList();
}

String? _resolve(String linkPath) {
  try {
    return Directory(linkPath).resolveSymbolicLinksSync();
  } catch (_) {
    return null;
  }
}
