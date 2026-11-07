import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/docs/weapon_builder_service.dart';

WeaponDraft draftWith({
  WeaponType type = WeaponType.smallSword,
  int objId = 0x30000,
  List<WeaponLevel> levels = const [],
  String displayName = 'Test Blade',
}) {
  return WeaponDraft(
    displayName: displayName,
    description: 'A blade',
    type: type,
    id: 8000,
    objId: objId,
    levels: levels,
  );
}

void main() {
  group('obj id ranges', () {
    test('each type maps to its own block', () {
      expect(
        WeaponBuilderService.typeFromObjId(0x30001),
        WeaponType.smallSword,
      );
      expect(
        WeaponBuilderService.typeFromObjId(0x30201),
        WeaponType.largeSword,
      );
      expect(WeaponBuilderService.typeFromObjId(0x30401), WeaponType.spear);
      expect(
        WeaponBuilderService.typeFromObjId(0x30601),
        WeaponType.combatBracer,
      );
    });

    test('anything outside every range has no type', () {
      expect(WeaponBuilderService.typeFromObjId(0x2FFFF), isNull);
      expect(WeaponBuilderService.typeFromObjId(0x30800), isNull);
    });

    test('vanilla ids are refused', () {
      expect(WeaponBuilderService.isObjIdAvailable(0x30003), isFalse);
      expect(WeaponBuilderService.isObjIdAvailable(0x30655), isFalse);
    });

    test('wpmask ids are refused', () {
      for (final id in WeaponBuilderService.wpmaskObjIds) {
        expect(
          WeaponBuilderService.isObjIdAvailable(id),
          isFalse,
          reason: '0x${id.toRadixString(16)} is engine-reserved',
        );
      }
    });

    test('a free id inside a range is accepted', () {
      expect(WeaponBuilderService.isObjIdAvailable(0x30100), isTrue);
    });
  });

  group('allocation', () {
    test('picks a usable id for every type', () {
      for (final type in WeaponType.values) {
        final id = WeaponBuilderService.allocateObjId(type, const {});
        expect(id, isNotNull);
        expect(WeaponBuilderService.isObjIdAvailable(id!), isTrue);
        expect(WeaponBuilderService.typeFromObjId(id), type);
      }
    });

    test('different mod names land on different slots', () {
      final names = [
        'awakened_protocol',
        'flaming_sword',
        'my_weapon',
        'rusty_blade',
        'moonlight',
        'test',
      ];
      final picked = <int>{};

      for (final name in names) {
        final id = WeaponBuilderService.allocateObjId(
          WeaponType.smallSword,
          const {},
          seed: name,
        )!;
        picked.add(id);
      }

      expect(
        picked.length,
        names.length,
        reason: 'two mods would ship the same weapon number',
      );
    });

    test('the same mod name always gives the same slot', () {
      final first = WeaponBuilderService.allocateObjId(
        WeaponType.spear,
        const {},
        seed: 'my_spear',
      );
      final second = WeaponBuilderService.allocateObjId(
        WeaponType.spear,
        const {},
        seed: 'my_spear',
      );

      expect(first, second);
    });

    test('a seeded slot is still valid and of the right class', () {
      for (final type in WeaponType.values) {
        final id = WeaponBuilderService.allocateObjId(
          type,
          const {},
          seed: 'some_mod_name',
        )!;

        expect(WeaponBuilderService.isObjIdAvailable(id), isTrue);
        expect(WeaponBuilderService.typeFromObjId(id), type);
      }
    });

    test('a taken seeded slot moves on to the next free one', () {
      final first = WeaponBuilderService.allocateObjId(
        WeaponType.largeSword,
        const {},
        seed: 'taken_test',
      )!;
      final second = WeaponBuilderService.allocateObjId(
        WeaponType.largeSword,
        {first},
        seed: 'taken_test',
      )!;

      expect(second, isNot(first));
      expect(WeaponBuilderService.typeFromObjId(second), WeaponType.largeSword);
    });

    test('skips ids already used by another mod', () {
      final first = WeaponBuilderService.allocateObjId(
        WeaponType.spear,
        const {},
      )!;
      final second = WeaponBuilderService.allocateObjId(
        WeaponType.spear,
        {first},
      );

      expect(second, isNot(first));
      expect(WeaponBuilderService.typeFromObjId(second!), WeaponType.spear);
    });
  });

  group('toToml', () {
    test('category always matches the type, so they cannot disagree', () {
      for (final type in WeaponType.values) {
        final objId = WeaponBuilderService.allocateObjId(type, const {})!;
        final toml = WeaponBuilderService.toToml(
          draftWith(type: type, objId: objId),
        );

        expect(toml, contains('category = ${type.category}'));
        expect(
          WeaponBuilderService.typeFromObjId(objId),
          type,
          reason: 'written obj_id must sit in the category range',
        );
      }
    });

    test('writes obj_id in hex so it matches the documented ranges', () {
      final toml = WeaponBuilderService.toToml(draftWith(objId: 0x30100));
      expect(toml, contains('obj_id = 0x30100'));
    });

    test('effect ids follow the weapon type', () {
      final toml = WeaponBuilderService.toToml(
        draftWith(type: WeaponType.spear, objId: 0x30401),
      );

      expect(toml, contains('est_id_normal = 620'));
      expect(toml, contains('est_id_berserk = 635'));
    });

    test('always clones stats from a vanilla weapon of the same class', () {
      final toml = WeaponBuilderService.toToml(draftWith());
      expect(toml, contains('[params]'));
      expect(toml, contains('clone_from_obj_id = 0x30003'));
      expect(toml, isNot(contains('levels = [')));
    });

    test('writes the levels as one inline array', () {
      final toml = WeaponBuilderService.toToml(
        draftWith(
          levels: const [
            WeaponLevel(minDamage: 10, maxDamage: 20),
            WeaponLevel(minDamage: 30, maxDamage: 40),
          ],
        ),
      );

      expect(toml, contains('levels = ['));
      expect(toml, contains('min_damage = 10'));
      expect(toml, contains('max_damage = 40'));
    });

    test('writes one shop block per story state', () {
      final toml = WeaponBuilderService.toToml(
        draftWith().copyWith(
          shopName: 'shop_RC_Grocery',
          shopStates: {2, 0},
        ),
      );

      expect('[[shops]]'.allMatches(toml).length, 2);
      expect(toml.indexOf('state = 0'), lessThan(toml.indexOf('state = 2')));
    });

    test('internal name is prefixed and cleaned', () {
      expect(draftWith(displayName: 'The "Best" Blade').internalName,
          'weapon_the_best_blade');
    });

    test('escapes quotes', () {
      final toml = WeaponBuilderService.toToml(
        draftWith(displayName: 'A "Quoted" Blade'),
      );
      expect(toml, contains(r'\"Quoted\"'));
    });
  });

  test('file name follows the weapon id', () {
    expect(
      WeaponBuilderService.fileNameFor(draftWith()),
      'weapon_8000.toml',
    );
  });
}
