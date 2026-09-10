import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/toml_service.dart';

void main() {
  test('a broken toml parses to an empty map instead of throwing', () {
    final values = TomlService.parse('enabled = \nrings = [[[');
    expect(values, isEmpty);
  });

  test('wrong types fall back to the default instead of throwing', () {
    final values = <String, dynamic>{
      'enabled': 'yes',
      'shadow_resolution': 'huge',
      'lod_multiplier': true,
      'shadow_blur_scale': 42,
      'high_grids': 'not a table',
    };

    expect(LodModFields.enabled.valueIn(values), false);
    expect(LodModFields.shadowResolution.valueIn(values), 2048);
    expect(LodModFields.lodMultiplier.valueIn(values), 0.0);
    expect(LodModFields.shadowBlurScale.valueIn(values), [1.0, 1.0, 1.0, 1.0]);
    expect(LodModFields.highGridsEnabled.valueIn(values), false);
    expect(LodModFields.highGridsRings.valueIn(values), 1);
  });

  test('missing keys and empty values fall back to the default', () {
    expect(LodModFields.shadowResolution.valueIn(const {}), 2048);
    expect(LodModFields.highGridsRings.valueIn(const {}), 1);
    expect(LodModFields.highGridsRoomRings.valueIn(const {}), isEmpty);
  });

  test('numbers are coerced across int and double', () {
    expect(LodModFields.shadowResolution.valueIn({'shadow_resolution': 4096.0}),
        4096);
    expect(LodModFields.lodMultiplier.valueIn({'lod_multiplier': 1}), 1.0);
  });

  test('section values are read out of their table', () {
    final values = <String, dynamic>{
      'high_grids': {'enabled': true, 'rings': 3},
    };
    expect(LodModFields.highGridsEnabled.valueIn(values), true);
    expect(LodModFields.highGridsRings.valueIn(values), 3);
  });
}
