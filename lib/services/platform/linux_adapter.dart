import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/nier_installation.dart';
import 'package:yp_launcher/services/platform/wine_adapter_base.dart';
import 'package:yp_launcher/services/wine/native_steam.dart';

class LinuxAdapter extends WineAdapterBase {
  @override
  bool get isExperimental => true;

  /// Native Steam/Proton installs plus whatever CrossOver bottles expose,
  /// deduplicated by path.
  @override
  Future<List<NierInstallation>> discoverFast() async =>
      _merge(findNierInNativeSteam(), await super.discoverFast());

  @override
  Future<List<NierInstallation>> discoverDeep() async =>
      _merge(findNierInNativeSteam(), await super.discoverDeep());

  List<NierInstallation> _merge(
    List<NierInstallation> first,
    List<NierInstallation> second,
  ) {
    final seen = <String>{};
    final merged = <NierInstallation>[];
    for (final install in [...first, ...second]) {
      if (seen.add(p.normalize(install.path).toLowerCase())) {
        merged.add(install);
      }
    }
    return merged;
  }

  @override
  Future<String> resolveRuntimeDir() async {
    final env = Platform.environment;
    final dataHome = env['XDG_DATA_HOME'];
    final root = (dataHome != null && dataHome.isNotEmpty)
        ? dataHome
        : p.join(env['HOME'] ?? '', '.local', 'share');
    return p.join(root, 'yp_launcher', 'bins');
  }

  @override
  List<String> get systemSevenZipCandidates => const [
        '/usr/bin/7zz',
        '/usr/local/bin/7zz',
      ];
}
