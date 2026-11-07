import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/docs/outfit_builder_service.dart';

OutfitDraft draftWith({
  OutfitCharacter character = OutfitCharacter.twoB,
  Map<MeshState, Set<String>> meshes = const {},
  String displayName = 'Black Dress',
}) {
  return OutfitDraft(
    displayName: displayName,
    description: 'A dress',
    character: character,
    outfitId: 10001,
    itemId: 9100,
    meshes: meshes,
  );
}

void main() {
  group('toToml', () {
    test('writes the required fields', () {
      final toml = OutfitBuilderService.toToml(draftWith());

      expect(toml, contains('outfit_id = 10001'));
      expect(toml, contains('character = "2B"'));
      expect(toml, contains('item_id = 9100'));
      expect(toml, contains('name = "black_dress"'));
    });

    test('omits mesh lists entirely when nothing is selected', () {
      final toml = OutfitBuilderService.toToml(draftWith());

      expect(toml, isNot(contains('meshes')));
    });

    test('writes only the states that have meshes', () {
      final toml = OutfitBuilderService.toToml(
        draftWith(
          meshes: {
            MeshState.base: {'Body', 'Hair'},
            MeshState.combat: {'facial_serious'},
            MeshState.nonCombat: <String>{},
          },
        ),
      );

      expect(toml, contains('meshes = ["Body", "Hair"]'));
      expect(toml, contains('combat_meshes = ["facial_serious"]'));
      expect(toml, isNot(contains('non_combat_meshes')));
    });

    test('drops states that do not apply to the character', () {
      final toml = OutfitBuilderService.toToml(
        draftWith(
          character: OutfitCharacter.a2,
          meshes: {
            MeshState.wig: {'Hair'},
            MeshState.eyemask: {'Eyemask'},
            MeshState.tower: {'Body'},
          },
        ),
      );

      expect(toml, contains('wig_meshes = ["Hair"]'));
      expect(toml, isNot(contains('eyemask_meshes')));
      expect(toml, isNot(contains('tower_meshes')));
    });

    test('9S keeps the story states, 2B does not', () {
      final nineS = OutfitBuilderService.toToml(
        draftWith(
          character: OutfitCharacter.nineS,
          meshes: {
            MeshState.tower: {'mesh_es0202'},
          },
        ),
      );
      final twoB = OutfitBuilderService.toToml(
        draftWith(
          meshes: {
            MeshState.tower: {'mesh_es0202'},
          },
        ),
      );

      expect(nineS, contains('tower_meshes'));
      expect(twoB, isNot(contains('tower_meshes')));
    });

    test('escapes quotes in mesh names and the outfit name', () {
      final toml = OutfitBuilderService.toToml(
        draftWith(
          displayName: 'The "Best" Dress',
          meshes: {
            MeshState.base: {'Od"d'},
          },
        ),
      );

      expect(toml, contains(r'"Od\"d"'));
      expect(toml, contains('name = "the_best_dress"'));
    });
  });

  group('character detection', () {
    test('maps every known model stem to its character', () {
      expect(
        OutfitBuilderService.characterFromModelPath('mods/x/pl0000.dat'),
        OutfitCharacter.twoB,
      );
      expect(
        OutfitBuilderService.characterFromModelPath('mods/x/pl020d.dtt'),
        OutfitCharacter.nineS,
      );
      expect(
        OutfitBuilderService.characterFromModelPath('mods/x/pl0100.dat'),
        OutfitCharacter.a2,
      );
    });

    test('is case insensitive', () {
      expect(
        OutfitBuilderService.characterFromModelPath('X/PL000D.DAT'),
        OutfitCharacter.twoB,
      );
    });

    test('returns null for something that is not a player model', () {
      expect(
        OutfitBuilderService.characterFromModelPath('mods/x/em0100.dat'),
        isNull,
      );
    });
  });

  group('untouched meshes', () {
    test('write no lists at all, so NAMS keeps its own assembly', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.twoB,
        const {
          'Body': MeshRule(),
          'Hair': MeshRule(),
          'Armor_Body': MeshRule(),
          'DLC_Body': MeshRule(),
        },
      );

      expect(lists, isEmpty);
    });

    test('one changed mesh switches the whole outfit to custom lists', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.twoB,
        const {
          'Body': MeshRule(),
          'Armor_Body': MeshRule(visibility: MeshVisibility.never),
        },
      );

      expect(lists[MeshState.base], {'Body'});
      expect(lists[MeshState.base], isNot(contains('Armor_Body')));
    });
  });

  group('rules produce the vanilla lists', () {
    test('2B Normal', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.twoB,
        const {
          'Body': MeshRule(),
          'Hair': MeshRule(tintedByHairspray: true),
          'Skirt': MeshRule(
            visibility: MeshVisibility.featherOnly,
            hideOnSelfDestruct: true,
          ),
          'Feather': MeshRule(visibility: MeshVisibility.featherOnly),
          'Broken': MeshRule(
            visibility: MeshVisibility.brokenOnly,
            hideOnSelfDestruct: true,
          ),
          'Eyemask': MeshRule(visibility: MeshVisibility.eyemaskOnly),
          'Eyelash': MeshRule(visibility: MeshVisibility.noEyemaskOnly),
          'facial_serious': MeshRule(visibility: MeshVisibility.combatOnly),
          'facial_normal': MeshRule(
            visibility: MeshVisibility.outOfCombatOnly,
          ),
        },
      );

      expect(lists[MeshState.base], {'Body', 'Hair'});
      expect(lists[MeshState.nonBroken], {'Feather', 'Skirt'});
      expect(lists[MeshState.broken], {'Broken'});
      expect(lists[MeshState.selfDestructHide], {'Broken', 'Skirt'});
      expect(lists[MeshState.eyemask], {'Eyemask'});
      expect(lists[MeshState.nonEyemask], {'Eyelash'});
      expect(lists[MeshState.combat], {'facial_serious'});
      expect(lists[MeshState.nonCombat], {'facial_normal'});
      expect(lists[MeshState.hairspray], {'Hair'});
    });

    test('A2 Normal', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.a2,
        const {
          'Body': MeshRule(),
          'Cloth': MeshRule(hideOnSelfDestruct: true),
          'Hair': MeshRule(tintedByHairspray: true, replacedByWig: true),
          'facial_serious': MeshRule(visibility: MeshVisibility.combatOnly),
          'facial_normal': MeshRule(
            visibility: MeshVisibility.outOfCombatOnly,
          ),
        },
      );

      expect(lists[MeshState.base], {'Body', 'Cloth', 'Hair'});
      expect(lists[MeshState.selfDestructHide], {'Cloth'});
      expect(lists[MeshState.wig], {'Hair'});
      expect(lists[MeshState.hairspray], {'Hair'});
      expect(lists[MeshState.combat], {'facial_serious'});
    });

    test('2B Full Armor needs only the base list', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.twoB,
        const {
          'Armor_Body': MeshRule(),
          'Armor_Head': MeshRule(),
          'Body': MeshRule(visibility: MeshVisibility.never),
          'Hair': MeshRule(visibility: MeshVisibility.never),
        },
      );

      expect(lists[MeshState.base], {'Armor_Body', 'Armor_Head'});
      expect(lists.length, 1);
    });

    test('combat and non-combat can never be half filled', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.twoB,
        const {
          'a': MeshRule(visibility: MeshVisibility.combatOnly),
          'b': MeshRule(visibility: MeshVisibility.outOfCombatOnly),
        },
      );

      expect(lists[MeshState.combat], {'a'});
      expect(lists[MeshState.nonCombat], {'b'});
    });

    test('a mesh set to never appears in no list at all', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.twoB,
        const {'gone': MeshRule(visibility: MeshVisibility.never)},
      );

      expect(lists, isEmpty);
    });

    test('a state the character lacks falls back to always visible', () {
      final lists = OutfitBuilderService.listsFromRules(
        OutfitCharacter.a2,
        const {'x': MeshRule(visibility: MeshVisibility.eyemaskOnly)},
      );

      expect(lists[MeshState.base], {'x'});
      expect(lists.containsKey(MeshState.eyemask), isFalse);
    });
  });

  group('archive pairing', () {
    test('finds the sibling of the other extension', () {
      expect(
        OutfitBuilderService.siblingPath(r'C:\mods\pl0000.dat', '.dtt'),
        endsWith('pl0000.dtt'),
      );
      expect(
        OutfitBuilderService.siblingPath(r'C:\mods\pl0000.dtt', '.dat'),
        endsWith('pl0000.dat'),
      );
    });

    test('keeps the sibling in the same folder', () {
      final sibling =
          OutfitBuilderService.siblingPath(r'C:\mods\deep\pl0000.dat', '.dtt');
      expect(sibling, contains('deep'));
    });
  });

  group('isModelFile', () {
    test('accepts dat and dtt in any case', () {
      expect(OutfitBuilderService.isModelFile('a/pl0000.dat'), isTrue);
      expect(OutfitBuilderService.isModelFile('a/pl0000.DTT'), isTrue);
    });

    test('rejects anything else', () {
      expect(OutfitBuilderService.isModelFile('a/pl0000.wmb'), isFalse);
      expect(OutfitBuilderService.isModelFile('a/icon.png'), isFalse);
    });
  });

  group('shop entry', () {
    test('writes shop_name, item_id and state', () {
      final toml = OutfitBuilderService.shopItemToml(
        draftWith().copyWith(shopName: 'shop_RC_Grocery', shopStates: {2}),
      );

      expect(toml, contains('shop_name = "shop_RC_Grocery"'));
      expect(toml, contains('item_id = 9100'));
      expect(toml, contains('state = 2'));
    });

    test('writes one entry per story state, in order', () {
      final toml = OutfitBuilderService.shopItemToml(
        draftWith().copyWith(
          shopName: 'shop_RC_Grocery',
          shopStates: {3, 0, 1},
        ),
      );

      expect('[[shop_item]]'.allMatches(toml).length, 3);
      expect(toml.indexOf('state = 0'), lessThan(toml.indexOf('state = 1')));
      expect(toml.indexOf('state = 1'), lessThan(toml.indexOf('state = 3')));
    });

    test('names the entity kind, so the file name is not parsed as one', () {
      final toml = OutfitBuilderService.shopItemToml(
        draftWith().copyWith(shopName: 'shop_RC_Grocery'),
      );

      expect(toml, startsWith('[[shop_item]]'));
    });

    test('the file name has no underscore NAMS could split on', () {
      final name = OutfitBuilderService.shopFileNameFor(draftWith());
      final stem = name.substring(0, name.length - '.toml'.length);

      expect(stem, isNot(contains('_')));
    });

    test('is only requested when a shop is set', () {
      expect(draftWith().sellsInShop, isFalse);
      expect(
        draftWith().copyWith(shopName: 'shop_RC_Grocery').sellsInShop,
        isTrue,
      );
    });

    test('clearShop removes it again', () {
      final draft =
          draftWith().copyWith(shopName: 'shop_RC_Grocery');
      expect(draft.copyWith(clearShop: true).sellsInShop, isFalse);
    });

    test('shop file is named after the outfit', () {
      expect(
        OutfitBuilderService.shopFileNameFor(draftWith()),
        'shop9100.toml',
      );
    });

    test('every offered shop name is a real vanilla shop', () {
      const vanilla = {
        'shop_AP_Grocery', 'shop_BK_Grocery', 'shop_CD_Grocery',
        'shop_CF_Grocery', 'shop_CS_Grocery', 'shop_DC_Grocery',
        'shop_DO_Grocery', 'shop_DP_Grocery', 'shop_Emile',
        'shop_FC_Grocery', 'shop_FC_Smith', 'shop_GP_Grocery',
        'shop_PC_Grocery', 'shop_PC_Grocery_first', 'shop_PV_Grocery',
        'shop_PV_Smith', 'shop_PV_UniqueShop', 'shop_RC_Grocery',
        'shop_RC_Grocery_small', 'shop_RC_Smith', 'shop_TEST_Chip1',
        'shop_TEST_Chip2', 'shop_TEST_Chip3', 'shop_TEST_Chip4',
        'shop_TEST_Chip5', 'shop_TEST_Chip6', 'shop_TEST_Grocery',
        'shop_TEST_Smith', 'shop_Upgrade', 'shop_VF_Grocery',
        'shop_chipEmile', 'shop_rareChipEmile', 'shop_rareEmile',
      };

      for (final name in OutfitBuilderService.shopNames) {
        expect(vanilla, contains(name), reason: '$name is not a vanilla shop');
      }
    });

    test('state stays inside the range NAMS accepts', () {
      expect(OutfitBuilderService.maxShopState, 6);
    });
  });

  test('file name follows the outfit id', () {
    expect(
      OutfitBuilderService.fileNameFor(draftWith()),
      'outfit_10001.toml',
    );
  });

  test('each character starts in its own id block', () {
    expect(OutfitCharacter.twoB.firstOutfitId, 10001);
    expect(OutfitCharacter.nineS.firstOutfitId, 20001);
    expect(OutfitCharacter.a2.firstOutfitId, 30001);
  });
}
