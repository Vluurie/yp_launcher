import 'dart:convert';
import 'dart:io';
import 'package:yp_launcher/services/platform/platform_adapter.dart';

class NamsSettingsService {
  /// Null until the location is known, which on a compatibility layer means
  /// until the game directory identifies the prefix.
  static Future<String?> resolveSettingsPath(String? gameDir) =>
      PlatformAdapter.current.resolveNamsSettingsPath(gameDir);

  static Future<Map<String, dynamic>> loadSettings(String? gameDir) async {
    final settingsPath = await resolveSettingsPath(gameDir);
    if (settingsPath == null) return Map<String, dynamic>.from(_defaultSettings);

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

  /// Returns false when the location is unknown; never falls back to a
  /// relative path, which NAMS would not read.
  static Future<bool> saveSettings(
    Map<String, dynamic> settings,
    String? gameDir,
  ) async {
    final settingsPath = await resolveSettingsPath(gameDir);
    if (settingsPath == null) return false;

    final file = File(settingsPath);
    if (!await file.parent.exists()) {
      await file.parent.create(recursive: true);
    }
    await file.writeAsString(const JsonEncoder.withIndent('  ').convert(settings));
    return true;
  }

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