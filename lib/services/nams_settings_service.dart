import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as path;

class NamsSettingsService {
  static String get settingsPath {
    final appData = Platform.environment['APPDATA'] ?? '';
    return path.join(appData, 'NAMS', 'settings.json');
  }

  static Future<Map<String, dynamic>> loadSettings() async {
    final file = File(settingsPath);
    if (!await file.exists()) {
      return Map<String, dynamic>.from(_defaultSettings);
    }
    final content = await file.readAsString();
    if (content.trim().isEmpty) {
      return Map<String, dynamic>.from(_defaultSettings);
    }
    return Map<String, dynamic>.from(
      jsonDecode(content) as Map<String, dynamic>,
    );
  }

  static Future<void> saveSettings(Map<String, dynamic> settings) async {
    final file = File(settingsPath);
    final dir = file.parent;
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }
    final encoder = const JsonEncoder.withIndent('  ');
    await file.writeAsString(encoder.convert(settings));
  }

  static bool get settingsFileExists => File(settingsPath).existsSync();

  static const Map<String, dynamic> _defaultSettings = {
    'firstPlaythrough': true, 
    'keybinds': {
      'main': {'yorha_protocol': 'F1'},
      'yorha_protocol': {
        'freeze_game': 'F5',
        'max_speed': 'F6',
        'free_cam': 'F7',
        'phase_jump': 'F8',
        'toggle_game_input': 'F9',
        'advance_frame': 'F10',
        'dev_mode': 'Insert',
      },
    },
    'gameKeybindsGlobal': false,
    'loadingSpeedupEnabled': true,
    'shadersEnabled': true,
    'soundEnabled': true,
    'randomizerConfig': {
      'autoStartOnStartup': false,
      'randomizeGroundEnemies': true,
      'randomizeFlyingEnemies': true,
      'allowBigEnemies': true,
      'includeDlcEnemies': true,
      'excludedEnemies': <String>[],
    },
  };
}