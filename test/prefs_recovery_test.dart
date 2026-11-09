import 'dart:convert';
import 'dart:io';

import 'package:automato_theme/automato_theme.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider_linux/path_provider_linux.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:path_provider_windows/path_provider_windows.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shared_preferences_linux/shared_preferences_linux.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_platform_interface.dart';
import 'package:shared_preferences_windows/shared_preferences_windows.dart';
import 'package:yp_launcher/services/prefs_recovery_service.dart';

class _WindowsSupportDir extends PathProviderWindows {
  _WindowsSupportDir(this.dir);

  final String dir;

  @override
  Future<String?> getApplicationSupportPath() async => dir;
}

class _LinuxSupportDir extends PathProviderLinux {
  _LinuxSupportDir(this.dir);

  final String dir;

  @override
  Future<String?> getApplicationSupportPath() async => dir;
}

void _useRealStore(String dir) {
  if (Platform.isWindows) {
    final provider = _WindowsSupportDir(dir);
    PathProviderPlatform.instance = provider;
    SharedPreferencesStorePlatform.instance = SharedPreferencesWindows()
      ..pathProvider = provider;
  } else {
    final provider = _LinuxSupportDir(dir);
    PathProviderPlatform.instance = provider;
    SharedPreferencesStorePlatform.instance = SharedPreferencesLinux()
      ..pathProvider = provider;
  }
}

void main() {
  late Directory dir;
  late File prefsFile;
  late File backupFile;

  setUp(() {
    dir = Directory.systemTemp.createTempSync('prefs_recovery_');
    prefsFile = File(p.join(dir.path, 'shared_preferences.json'));
    backupFile = File(p.join(dir.path, 'shared_preferences.corrupt.json'));
    SharedPreferences.resetStatic();
    _useRealStore(dir.path);
  });

  tearDown(() {
    SharedPreferences.resetStatic();
    dir.deleteSync(recursive: true);
  });

  group('PrefsRecoveryService', () {
    final corruptions = <String, List<int>>{
      'null bytes after a killed write': List<int>.filled(512, 0),
      'truncated json': utf8.encode('{"flutter.app_theme":"ni'),
      'invalid utf8': [0x7B, 0xC3, 0x28, 0xFF, 0xFE],
    };

    for (final entry in corruptions.entries) {
      test('recovers from ${entry.key}', () async {
        prefsFile.writeAsBytesSync(entry.value);

        await expectLater(SharedPreferences.getInstance(), throwsA(anything));
        await expectLater(
          AutomatoThemeNotifier.loadFromPreferences(),
          throwsA(anything),
        );

        await PrefsRecoveryService.ensureReadable();

        expect(prefsFile.existsSync(), isFalse);
        expect(backupFile.readAsBytesSync(), entry.value);

        final prefs = await SharedPreferences.getInstance();
        expect(prefs.getKeys(), isEmpty);
        await AutomatoThemeNotifier.loadFromPreferences();

        await prefs.setString('app_theme', 'nier');
        final saved = jsonDecode(prefsFile.readAsStringSync()) as Map;
        expect(saved['flutter.app_theme'], 'nier');
      });
    }

    test('leaves a valid file untouched', () async {
      const content = '{"flutter.app_theme":"nier","flutter.window_width":1600.0}';
      prefsFile.writeAsStringSync(content);

      await PrefsRecoveryService.ensureReadable();

      expect(prefsFile.readAsStringSync(), content);
      expect(backupFile.existsSync(), isFalse);
      final prefs = await SharedPreferences.getInstance();
      expect(prefs.getString('app_theme'), 'nier');
      expect(prefs.getDouble('window_width'), 1600.0);
    });

    test('does nothing on first launch without a file', () async {
      await PrefsRecoveryService.ensureReadable();

      expect(prefsFile.existsSync(), isFalse);
      expect(backupFile.existsSync(), isFalse);
      final prefs = await SharedPreferences.getInstance();
      expect(prefs.getKeys(), isEmpty);
    });

    test('overwrites an older backup', () async {
      backupFile.writeAsStringSync('old');
      prefsFile.writeAsStringSync('{"flutter.x":');

      await PrefsRecoveryService.ensureReadable();

      expect(backupFile.readAsStringSync(), '{"flutter.x":');
      expect(prefsFile.existsSync(), isFalse);
    });
  }, skip: Platform.isMacOS ? 'macOS stores prefs in NSUserDefaults' : null);
}
