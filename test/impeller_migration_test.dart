import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/nams_settings_service.dart';

Map<String, dynamic> decode(String raw) =>
    jsonDecode(raw) as Map<String, dynamic>;

void main() {
  test('impeller is turned off, everything else survives', () {
    final result = decode(
      NamsConfigService.withImpellerOff(
        jsonEncode({
          NamsSettingsService.impellerKey: true,
          'firstPlaythrough': true,
          'keybinds': {
            'main': {'yorha_protocol': 'F2'},
          },
        }),
      ),
    );

    expect(result[NamsSettingsService.impellerKey], false);
    expect(result['firstPlaythrough'], true);
    expect((result['keybinds'] as Map)['main'], {'yorha_protocol': 'F2'});
  });

  test('a settings file that already has it off is left alone', () {
    const raw = '{"impeller": false}';
    expect(NamsConfigService.withImpellerOff(raw), raw);
  });

  test('a settings file without the key is left alone', () {
    const raw = '{"firstPlaythrough": true}';
    expect(NamsConfigService.withImpellerOff(raw), raw);
  });

  test('unreadable json is passed through instead of being lost', () {
    const raw = '{ this is not json';
    expect(NamsConfigService.withImpellerOff(raw), raw);
  });
}
