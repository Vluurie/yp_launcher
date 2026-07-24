import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/toml_service.dart';

class ThirdPartyFlags {
  ThirdPartyFlags._();

  static String _tomlPath(String gameDir) =>
      path.join(gameDir, 'nams', 'nams.toml');

  static Future<void> set(String gameDir, String key, bool value) async {
    final raw = await TomlService.readTomlFile(_tomlPath(gameDir));
    if (raw.isEmpty) return;
    if (TomlService.parse(raw)[key] == value) return;
    final updated = TomlService.updateToml(raw, {key: value});
    await TomlService.writeTomlFile(_tomlPath(gameDir), updated);
  }

  static Future<Map<String, dynamic>> read(String gameDir) async {
    final raw = await TomlService.readTomlFile(_tomlPath(gameDir));
    if (raw.isEmpty) return {};
    return TomlService.parse(raw);
  }
}
