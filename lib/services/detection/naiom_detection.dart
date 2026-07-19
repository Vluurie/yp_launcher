import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/naiom_keys.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/models/config_fields.dart';

class NaiomMigrationResult {
  final bool migrated;
  final List<String> skippedEntries;

  const NaiomMigrationResult({
    required this.migrated,
    this.skippedEntries = const [],
  });
}

class NaiomDetection {
  NaiomDetection._();

  static const _bindingMap = <String, String>{
    'standardmoveforward': 'standard_move_forward',
    'standardmoveleft': 'standard_move_left',
    'standardmovebackward': 'standard_move_backward',
    'standardmoveright': 'standard_move_right',
    'standardjump': 'standard_jump',
    'standardwalk': 'standard_walk',
    'standardautorun': 'standard_auto_run',
    'standardlightattack': 'standard_light_attack',
    'standardheavyattack': 'standard_heavy_attack',
    'standardprogram': 'standard_program',
    'standardlockon': 'standard_lock_on',
    'standarduse': 'standard_use',
    'standardselfdestruct': 'standard_self_destruct',
    'standardlight': 'standard_light',
    'standardresetcamera': 'standard_reset_camera',
    'standardmenuup': 'standard_menu_up',
    'standardmenuleft': 'standard_menu_left',
    'standardmenudown': 'standard_menu_down',
    'standardmenuright': 'standard_menu_right',
    'standardmenuopen': 'standard_menu_open',
    'standardmenuback': 'standard_menu_back',
    'standardmenuenter': 'standard_menu_enter',
    'standardshortcutmenu': 'standard_shortcut_menu',
    'standardswitchweapon': 'standard_switch_weapon',
    'standardnextprogram': 'standard_next_program',
    'standardpreviousprogram': 'standard_previous_program',
    'standardfire': 'standard_fire',
    'nonstandardevade': 'non_standard_evade',
    'nonstandardautofire': 'non_standard_auto_fire',
    'nonstandardnextitem': 'non_standard_next_item',
    'nonstandardpreviousitem': 'non_standard_previous_item',
    'nonstandarduseitem': 'non_standard_use_item',
    'thirdpersonmodetoggle': 'third_person_mode_toggle',
    'aimmodetoggle': 'aim_mode_toggle',
  };

  static const _floatMap = <String, String>{
    'thirdpersonsensitivityx': 'third_person_sensitivity_x',
    'thirdpersonsensitivityy': 'third_person_sensitivity_y',
    'aimsensitivityx': 'aim_sensitivity_x',
    'aimsensitivityy': 'aim_sensitivity_y',
  };

  static const _boolMap = <String, String>{
    'movementdisabletapevade': 'movement_disable_tap_evade',
    'miscdisablepodpet': 'misc_disable_pod_pet',
  };

  static Future<Map<String, String>?> detectLegacyNaiom(String gameDir) async {
    for (final name in ['NAIOM.ini', 'NAIOM.cfg']) {
      final file = File(path.join(gameDir, name));
      if (await file.exists()) {
        final parsed = _parseIni(await file.readAsString());
        if (parsed.isNotEmpty) return parsed;
      }
    }
    return null;
  }

  static Map<String, String> _parseIni(String content) {
    final result = <String, String>{};

    for (final line in content.split('\n')) {
      var trimmed = line.trim();
      if (trimmed.isEmpty ||
          trimmed.startsWith(';') ||
          trimmed.startsWith('#') ||
          trimmed.startsWith('[')) {
        continue;
      }

      final commentIdx = trimmed.indexOf(RegExp(r'[;#]'));
      if (commentIdx != -1) {
        trimmed = trimmed.substring(0, commentIdx).trim();
        if (trimmed.isEmpty) continue;
      }

      final sepIdx = trimmed.indexOf(RegExp(r'\s'));
      if (sepIdx == -1) continue;

      final key = trimmed.substring(0, sepIdx).trim().toLowerCase();
      final value = trimmed.substring(sepIdx + 1).trim();
      if (key.isEmpty || value.isEmpty) continue;

      result[key] = value;
    }

    return result;
  }

  static bool _boolOf(String value) {
    final lower = value.toLowerCase();
    return lower == '1' || lower == 'true';
  }

  static bool _isUnbound(String value) =>
      value.toLowerCase() == '<unbound>' || value.isEmpty;

  static Future<NaiomMigrationResult> migrateNaiomIni(
    String gameDir,
    Map<String, String> iniValues,
  ) async {
    final tomlPath = path.join(gameDir, 'nams', 'nams.toml');
    final rawContent = await TomlService.readTomlFile(tomlPath);
    if (rawContent.isEmpty) {
      return const NaiomMigrationResult(migrated: false);
    }

    final currentValues = TomlService.parse(rawContent);
    final currentMouse =
        (currentValues['mouse'] as Map<String, dynamic>?) ?? const {};
    final currentBindings =
        (currentMouse['bindings'] as Map<String, dynamic>?) ?? const {};

    final alreadyConfigured =
        currentMouse[NamsFields.thirdPersonMode.key] == true ||
        currentMouse[NamsFields.aimMode.key] == true ||
        currentBindings.values.any((v) => v is String && v.isNotEmpty);
    if (alreadyConfigured) {
      return const NaiomMigrationResult(migrated: false);
    }

    final mouseUpdates = <String, dynamic>{};
    final bindingUpdates = <String, dynamic>{};
    final skipped = <String>[];

    for (final entry in iniValues.entries) {
      final key = entry.key;
      final value = entry.value;

      final floatKey = _floatMap[key];
      if (floatKey != null) {
        final parsed = double.tryParse(value);
        if (parsed != null) mouseUpdates[floatKey] = parsed;
        continue;
      }

      final boolKey = _boolMap[key];
      if (boolKey != null) {
        mouseUpdates[boolKey] = _boolOf(value);
        continue;
      }

      switch (key) {
        case 'thirdpersonmode':
          final mode = value.toLowerCase();
          mouseUpdates[NamsFields.thirdPersonMode.key] = mode != 'disabled';
          mouseUpdates[NamsFields.thirdPersonCharFollow.key] =
              mode == 'rawinputcharfollow';
          continue;
        case 'aimmode':
          final mode = value.toLowerCase();
          mouseUpdates[NamsFields.aimMode.key] = mode != 'disabled';
          mouseUpdates[NamsFields.aimCrosshair.key] = mode == 'crosshair';
          continue;
        case 'miscenablecustomcursors':
          mouseUpdates[NamsFields.miscDisableDefaultCursor.key] =
              !_boolOf(value);
          continue;
        case 'miscopendebugmenu':
          if (_isUnbound(value)) continue;
          final vk = NaiomKeys.keyNameToVk(value);
          if (vk != null) {
            mouseUpdates[NamsFields.miscOpenDebugMenu.key] = vk;
          } else {
            skipped.add('MiscOpenDebugMenu ($value)');
          }
          continue;
        case 'soundsenabled':
        case 'configreload':
          continue;
      }

      final bindingKey = _bindingMap[key];
      if (bindingKey == null) continue;
      if (_isUnbound(value)) continue;

      final normalized = NaiomKeys.comboParts(value).join('+');
      if (!NaiomKeys.isValidBinding(normalized) || normalized.isEmpty) {
        skipped.add('${_displayKey(bindingKey)} ($value)');
        continue;
      }
      if (NaiomKeys.comboContainsMouse(normalized) &&
          !NaiomKeys.allowsMouse(bindingKey)) {
        skipped.add('${_displayKey(bindingKey)} ($value)');
        continue;
      }
      bindingUpdates[bindingKey] = normalized;
    }

    if (mouseUpdates.isEmpty && bindingUpdates.isEmpty) {
      return NaiomMigrationResult(migrated: false, skippedEntries: skipped);
    }

    final mergedMouse = Map<String, dynamic>.from(currentMouse)
      ..addAll(mouseUpdates);
    if (bindingUpdates.isNotEmpty) {
      mergedMouse['bindings'] = Map<String, dynamic>.from(currentBindings)
        ..addAll(bindingUpdates);
    }
    final merged = Map<String, dynamic>.from(currentValues)
      ..['mouse'] = mergedMouse;

    final updatedContent = TomlService.updateToml(rawContent, merged);
    await TomlService.writeTomlFile(tomlPath, updatedContent);
    return NaiomMigrationResult(migrated: true, skippedEntries: skipped);
  }

  static String _displayKey(String tomlKey) {
    return tomlKey
        .split('_')
        .map((w) => w.isEmpty ? w : '${w[0].toUpperCase()}${w.substring(1)}')
        .join(' ');
  }
}
