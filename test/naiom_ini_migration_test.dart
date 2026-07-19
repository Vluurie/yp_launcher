import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/detection/naiom_detection.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

Directory _gameDirWith(String naiomIni) {
  final dir = Directory.systemTemp.createTempSync('yp_naiom_');
  File(p.join(dir.path, 'NAIOM.ini')).writeAsStringSync(naiomIni);
  return dir;
}

Future<Map<String, dynamic>> _mouse(Directory gameDir) async {
  final raw = File(p.join(gameDir.path, 'nams', 'nams.toml'))
      .readAsStringSync();
  return (TomlService.parse(raw)['mouse'] as Map<String, dynamic>?) ?? {};
}

const _fullIni = '''
;===================================================================================================
; --- NAIOM - Configuration File ---
;===================================================================================================

; Type: float
ThirdPersonSensitivityX 1.500000

; Type: float
ThirdPersonSensitivityY -0.750000

; Type: float
AimSensitivityX 2.000000

; Type: float
AimSensitivityY 1.000000

; Type: bool
MovementDisableTapEvade 1

; Type: bool
MiscDisablePodPet 1

; Type: bool
MiscEnableCustomCursors 0

; Type: binding
configReload <unbound>

; Type: bool
soundsEnabled 1

; <items>;Disabled;Fix Disabled;RawInput;Raw Input;RawInputCharFollow;Raw Input Char Follow
; Type: string
ThirdPersonMode RawInputCharFollow

; <items>;Disabled;Fix Disabled;RawInputOnly;Raw Input Only;Crosshair;Raw Input Crosshair
; Type: string
AimMode Crosshair

; Type: binding
ThirdPersonModeToggle F5

; Type: binding
AimModeToggle <unbound>

; Type: binding
StandardMoveForward W

; Type: binding
StandardJump SPACE

; Type: binding
StandardFire MOUSE1

; Type: binding
StandardWalk MOUSE2

; Type: binding
NonStandardEvade LSHIFT

; Type: binding
NonStandardUseItem lctrl+x

; Type: binding
NonStandardAutoFire CONTROLLER_A

; Type: binding
MiscOpenDebugMenu K
''';

void main() {
  group('NAIOM.ini import', () {
    test('parses the space-separated INI format', () async {
      final gameDir = _gameDirWith(_fullIni);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      final parsed = await NaiomDetection.detectLegacyNaiom(gameDir.path);

      expect(parsed, isNotNull);
      expect(parsed!['thirdpersonsensitivityx'], '1.500000');
      expect(parsed['thirdpersonmode'], 'RawInputCharFollow');
      expect(parsed['nonstandardevade'], 'LSHIFT');
      expect(parsed.containsKey('configreload'), isTrue);
    });

    test('imports values into nams.toml with correct mappings', () async {
      final gameDir = _gameDirWith(_fullIni);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final ini = await NaiomDetection.detectLegacyNaiom(gameDir.path);
      final result = await NaiomDetection.migrateNaiomIni(gameDir.path, ini!);

      expect(result.migrated, isTrue);

      final mouse = await _mouse(gameDir);
      final bindings = mouse['bindings'] as Map<String, dynamic>;

      expect(mouse['third_person_sensitivity_x'], 1.5);
      expect(mouse['third_person_sensitivity_y'], -0.75);
      expect(mouse['aim_sensitivity_x'], 2.0);
      expect(mouse['movement_disable_tap_evade'], isTrue);
      expect(mouse['misc_disable_pod_pet'], isTrue);

      expect(mouse['third_person_mode'], isTrue,
          reason: 'RawInputCharFollow enables the camera fix');
      expect(mouse['third_person_char_follow'], isTrue);
      expect(mouse['aim_mode'], isTrue, reason: 'Crosshair enables aim fix');
      expect(mouse['aim_crosshair'], isTrue);

      expect(mouse['misc_disable_default_cursor'], isTrue,
          reason: 'MiscEnableCustomCursors 0 keeps the system cursor');

      expect(mouse['misc_open_debug_menu'], 0x4B,
          reason: 'binding K becomes VK code 0x4B');

      expect(bindings['standard_move_forward'], 'W');
      expect(bindings['standard_jump'], 'SPACE');
      expect(bindings['standard_fire'], 'MOUSE1',
          reason: 'mouse buttons are allowed on standard_fire');
      expect(bindings['non_standard_evade'], 'LSHIFT');
      expect(bindings['non_standard_use_item'], 'LCTRL+X',
          reason: 'combinations are normalized to uppercase with +');
      expect(bindings['aim_mode_toggle'], '',
          reason: '<unbound> stays unbound');
    });

    test('skips unsupported keys and reports them by name', () async {
      final gameDir = _gameDirWith(_fullIni);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final ini = await NaiomDetection.detectLegacyNaiom(gameDir.path);
      final result = await NaiomDetection.migrateNaiomIni(gameDir.path, ini!);

      expect(
        result.skippedEntries.any((e) => e.contains('F5')),
        isTrue,
        reason: 'F5 is not a valid NAMS key name',
      );
      expect(
        result.skippedEntries.any((e) => e.contains('CONTROLLER_A')),
        isTrue,
        reason: 'controller bindings are not supported by NAMS',
      );
      expect(
        result.skippedEntries.any((e) => e.contains('MOUSE2')),
        isTrue,
        reason: 'standard_walk is keyboard-only, MOUSE2 would be a no-op',
      );

      final mouse = await _mouse(gameDir);
      final bindings = mouse['bindings'] as Map<String, dynamic>;
      expect(bindings['third_person_mode_toggle'], '');
      expect(bindings['non_standard_auto_fire'], '');
      expect(bindings['standard_walk'], '');
    });

    test('second run does not overwrite existing settings', () async {
      final gameDir = _gameDirWith(_fullIni);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final ini = await NaiomDetection.detectLegacyNaiom(gameDir.path);
      await NaiomDetection.migrateNaiomIni(gameDir.path, ini!);

      final tomlPath = p.join(gameDir.path, 'nams', 'nams.toml');
      var content = File(tomlPath).readAsStringSync();
      content = content.replaceFirst(
        'standard_jump = "SPACE"',
        'standard_jump = "E"',
      );
      File(tomlPath).writeAsStringSync(content);

      final second = await NaiomDetection.migrateNaiomIni(gameDir.path, ini);
      expect(second.migrated, isFalse,
          reason: 'guard must refuse once bindings are configured');

      final bindings =
          (await _mouse(gameDir))['bindings'] as Map<String, dynamic>;
      expect(bindings['standard_jump'], 'E',
          reason: 'user changes must survive');
    });

    test('returns null when no NAIOM files exist', () async {
      final dir = Directory.systemTemp.createTempSync('yp_naiom_none_');
      addTearDown(() => dir.deleteSync(recursive: true));

      expect(await NaiomDetection.detectLegacyNaiom(dir.path), isNull);
    });
  });
}
