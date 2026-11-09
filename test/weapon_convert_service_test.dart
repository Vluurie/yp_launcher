import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/docs/weapon_builder_service.dart';
import 'package:yp_launcher/services/docs/weapon_convert_service.dart';

void main() {
  test('stem maps to the weapon class the ranges define', () {
    expect(WeaponConvertService.stemType('wp0003'), WeaponType.smallSword);
    expect(WeaponConvertService.stemType('wp0240'), WeaponType.largeSword);
    expect(WeaponConvertService.stemType('wp0450'), WeaponType.spear);
    expect(WeaponConvertService.stemType('wp0610'), WeaponType.combatBracer);
  });

  test('non-weapon stems are refused', () {
    expect(WeaponConvertService.stemType('pl0000'), isNull);
    expect(WeaponConvertService.stemType('em1234'), isNull);
    expect(WeaponConvertService.stemType('wp'), isNull);
  });

  test('obj id and stem convert back and forth', () {
    for (final stem in ['wp0003', 'wp0100', 'wp0450', 'wp0610']) {
      final type = WeaponConvertService.stemType(stem)!;
      final objId = 0x30000 + int.parse(stem.substring(2), radix: 16);
      expect(WeaponConvertService.stemForObjId(objId), stem);
      expect(WeaponBuilderService.typeFromObjId(objId), type);
    }
  });

  test('the target slot is always free and of the same class', () {
    for (final type in WeaponType.values) {
      final id = WeaponBuilderService.allocateObjId(type, const {})!;
      expect(WeaponBuilderService.isObjIdAvailable(id), isTrue);
      expect(WeaponBuilderService.typeFromObjId(id), type);
      final stem = WeaponConvertService.stemForObjId(id);
      expect(WeaponConvertService.stemType(stem), type);
    }
  });

  test('a file that is not a weapon is rejected', () {
    expect(WeaponConvertService.isWeaponFile('a/wp0003.dat'), isTrue);
    expect(WeaponConvertService.isWeaponFile('a/wp0003.dtt'), isTrue);
    expect(WeaponConvertService.isWeaponFile('a/pl0000.dat'), isFalse);
    expect(WeaponConvertService.isWeaponFile('a/wp0003.wmb'), isFalse);
  });
}
