import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/nier_installation.dart';
import 'package:yp_launcher/services/wine/proton.dart';
import 'package:yp_launcher/services/wine/steam_vdf.dart';

/// Overrides the Steam roots scanned. Lets tests inject a fake Steam tree.
@visibleForTesting
List<String>? overrideSteamRoots;

List<String> _steamRoots() => overrideSteamRoots ?? steamRootCandidates();

/// Every Steam library directory: each root plus its `libraryfolders.vdf`
/// entries. Native (host) paths — no Wine prefix translation.
List<String> nativeSteamLibraries() {
  final libraries = <String>{};
  for (final root in _steamRoots()) {
    libraries.add(root);
    final vdf = File(p.join(root, 'steamapps', 'libraryfolders.vdf'));
    if (!vdf.existsSync()) continue;
    try {
      libraries.addAll(vdfPathEntries(vdf.readAsStringSync()));
    } catch (_) {}
  }
  return libraries.toList();
}

/// NieR:Automata installs found across all native Steam libraries.
List<NierInstallation> findNierInNativeSteam() {
  final seen = <String>{};
  final installations = <NierInstallation>[];
  for (final library in nativeSteamLibraries()) {
    final dir =
        p.join(library, 'steamapps', 'common', 'NieRAutomata');
    if (!File(p.join(dir, AppStrings.gameExeName)).existsSync()) continue;
    final normalized = p.normalize(dir);
    if (!seen.add(normalized.toLowerCase())) continue;
    installations.add(NierInstallation(
      path: normalized,
      hasData: Directory(p.join(normalized, 'data')).existsSync(),
    ));
  }
  return installations;
}
