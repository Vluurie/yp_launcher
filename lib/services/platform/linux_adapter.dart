import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/platform/wine_adapter_base.dart';

class LinuxAdapter extends WineAdapterBase {
  @override
  bool get isExperimental => true;

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
