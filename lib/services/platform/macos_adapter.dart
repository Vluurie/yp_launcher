import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:yp_launcher/services/platform/wine_adapter_base.dart';

class MacOSAdapter extends WineAdapterBase {
  @override
  Future<String> resolveRuntimeDir() async =>
      p.join((await getApplicationSupportDirectory()).path, 'bins');

  @override
  List<String> get systemSevenZipCandidates => const [
        '/opt/homebrew/bin/7zz',
        '/usr/local/bin/7zz',
      ];
}
