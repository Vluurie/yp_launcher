import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/models/mod_grouping.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/mods_service.dart';

import 'support/corpus_reader.dart';

const _configSubdirs = [
  'accessories',
  'behaviors',
  'inventory',
  'item_info',
  'mcd',
  'outfits',
];

const _outfitsJson = {
  'pl_2b_outfits': [
    {
      'item_id': 90210,
      'outfit_id': 9010,
      'outfit_name': 'YoRHa Uniform 1',
      'meshes': ['NS_KIMONO_Normal', 'NS_KIMONO_Body', 'Hair'],
      'effects': [],
    },
    {
      'item_id': 90213,
      'outfit_id': 9013,
      'outfit_name': "2P's Body Replica",
      'meshes': ['NS_2P_Body', 'NS_2P_Skirt', 'Hair'],
      'effects': [],
    },
  ],
  'pl_9s_outfits': [
    {
      'item_id': 90211,
      'outfit_id': 9011,
      'outfit_name': 'YoRHa Uniform 2',
      'meshes': ['NS_KIMONO_Pants', 'NS_KIMONO_Body', 'Hair'],
      'effects': [],
    },
  ],
  'pl_a2_outfits': [
    {
      'item_id': 90212,
      'outfit_id': 9012,
      'outfit_name': 'YoRHa Uniform Prototype',
      'meshes': ['NS_KIMONO_Body', 'NS_KIMONO_Cloth', 'Hair'],
      'effects': [],
    },
  ],
};

const _itemsJson = [
  {
    'item_id': 90210,
    'name': 'item_uq_kimono2B',
    'filename': 'it0000',
    'max_num': 1,
    'no_trash': true,
    'category': 6,
  },
  {
    'item_id': 90216,
    'name': 'item_uq_whiteFoxMask',
    'filename': 'ete00d',
    'max_num': 1,
    'no_trash': true,
    'category': 6,
  },
];

const _accessoriesJson = [
  {
    'item_name': 'item_uq_whiteFoxMask',
    'item_id': 90216,
    'accessory_id': 20,
    'filename': 'ete00d',
    'mesh_names_2b': ['_2BA2_Body'],
    'effects': [],
  },
];

const _behaviorsJson = [
  {'filename': 'ete00d', 'behavior': 'PlFaceMask', 'group_id': 8},
];

const _giveJson = {
  'auto_give': {'90210': 1, '90216': 1},
};

const _mcdJson = [
  {
    'filename': 'messcore.mcd',
    'messages': [
      {
        'event_name': 'WAX_ITEM_NAME_90210',
        'locale': '*',
        'font_id': 36,
        'paragraphs': [
          ['YoRHa Uniform 1'],
        ],
      },
    ],
  },
];

void _putBytes(String root, String rel) {
  final f = File(p.joinAll([root, ...rel.split('/')]));
  f.parent.createSync(recursive: true);
  f.writeAsBytesSync(List<int>.filled(32, 0));
}

void _putJson(String root, String rel, Object json) {
  final f = File(p.joinAll([root, ...rel.split('/')]));
  f.parent.createSync(recursive: true);
  f.writeAsStringSync(jsonEncode(json));
}

void _writeKimonoPayload(String root, {String wrapperDir = 'Kimono'}) {
  for (final stem in ['pl000d', 'pl010d', 'pl020d']) {
    _putBytes(root, 'data/pl/$stem.dat');
    _putBytes(root, 'data/pl/$stem.dtt');
  }
  for (final stem in ['ete00d', 'ete00e', 'ete0fa']) {
    _putBytes(root, 'data/et/$stem.dat');
    _putBytes(root, 'data/et/$stem.dtt');
  }
  for (final id in [90210, 90216]) {
    _putBytes(root, 'data/misctex/misctex_wax_$id.dat');
    _putBytes(root, 'data/misctex/misctex_wax_$id.dtt');
  }
  _putJson(root, '$wrapperDir/outfits/kimono_outfits.json', _outfitsJson);
  _putJson(root, '$wrapperDir/item_info/kimono_items.json', _itemsJson);
  _putJson(root, '$wrapperDir/accessories/kimono_accessories.json',
      _accessoriesJson);
  _putJson(root, '$wrapperDir/behaviors/kimono_behaviors.json', _behaviorsJson);
  _putJson(root, '$wrapperDir/inventory/kimono_give.json', _giveJson);
  _putJson(root, '$wrapperDir/mcd/kimono_mcd.json', _mcdJson);
}

Directory _kimonoDrop() {
  final tmp = Directory.systemTemp.createTempSync('yp_kimono_');
  _writeKimonoPayload(p.join(tmp.path, 'kimono_wax_0.0.3'));
  return tmp;
}

Directory _tempGameDir() => Directory.systemTemp.createTempSync('yp_kimono_g_');

void _cleanup(Directory dir) {
  try {
    dir.deleteSync(recursive: true);
  } catch (_) {}
}

String _realKimonoZip() =>
    Platform.environment['YP_KIMONO_ZIP'] ??
    p.join(repoRoot(), 'lib', 'widgets',
        'Kimono Wax 0.0.3 812 0.0.3 2026-07-30T15-53Z MWGBtyVlH.zip');

void main() {
  group('wax mod with configs in a mod-named wrapper folder', () {
    test('detectDrop reports the wax config without touching the drop',
        () async {
      final drop = _kimonoDrop();
      addTearDown(() => _cleanup(drop));
      final root = p.join(drop.path, 'kimono_wax_0.0.3');

      final detected = await ModsService.detectDrop(drop.path);

      expect(detected.kind, ModKind.data,
          reason: 'rejected: ${detected.errorReason}');
      expect(detected.data?.hasCompatConfig, isTrue);
      expect(detected.data?.hasOutfitConfig, isTrue);

      final for2b = detected.data?.outfitsByStem['pl000d'] ?? const [];
      expect(for2b.map((c) => c.outfitId), containsAll([9010, 9013]));
      expect(for2b.map((c) => c.name),
          containsAll(['YoRHa Uniform 1', "2P's Body Replica"]));
      expect(for2b.every((c) => c.needsItem), isTrue);
      expect(detected.data?.outfitsByStem['pl020d']?.single.outfitId, 9011);
      expect(detected.data?.outfitsByStem['pl010d']?.single.outfitId, 9012);

      expect(Directory(p.join(root, 'Kimono')).existsSync(), isTrue);
      expect(Directory(p.join(root, 'wax')).existsSync(), isFalse);
    });

    test('install relocates the wrapper to wax/mods and keeps data intact',
        () async {
      final drop = _kimonoDrop();
      final gameDir = _tempGameDir();
      addTearDown(() => _cleanup(drop));
      addTearDown(() => _cleanup(gameDir));

      final result = await ModsService.install(
        gameDir.path,
        drop.path,
        requestedName: 'kimono_wax',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final modRoot =
          p.join(gameDir.path, 'nams', 'mods', result.installedId!);
      final waxMod = p.join(modRoot, 'wax', 'mods', 'Kimono');
      for (final sub in _configSubdirs) {
        final files = Directory(p.join(waxMod, sub))
            .listSync()
            .whereType<File>()
            .where((f) => f.path.endsWith('.json'));
        expect(files, isNotEmpty, reason: 'missing wax/mods/Kimono/$sub');
      }
      expect(Directory(p.join(modRoot, 'Kimono')).existsSync(), isFalse);

      expect(File(p.join(modRoot, 'data', 'pl', 'pl0000.dat')).existsSync(),
          isTrue,
          reason: 'pl000d should be DLC-renamed to pl0000 on a no-DLC dir');
      expect(File(p.join(modRoot, 'data', 'et', 'ete00d.dat')).existsSync(),
          isTrue);
      expect(
          File(p.join(modRoot, 'data', 'misctex', 'misctex_wax_90210.dtt'))
              .existsSync(),
          isTrue);
    });

    test('listInstalled exposes the config, outfits and wax grouping',
        () async {
      final drop = _kimonoDrop();
      final gameDir = _tempGameDir();
      addTearDown(() => _cleanup(drop));
      addTearDown(() => _cleanup(gameDir));

      final result = await ModsService.install(
        gameDir.path,
        drop.path,
        requestedName: 'kimono_wax',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final mods = await ModsService.listInstalled(gameDir.path);
      final mod = mods.singleWhere((m) => m.id == 'kimono_wax');

      expect(mod.kind, ModKind.data);
      expect(mod.data?.hasCompatConfig, isTrue);
      expect(mod.data?.hasOutfitConfig, isTrue);
      expect(mod.data?.outfitsByStem['pl0000']?.map((c) => c.outfitId),
          containsAll([9010, 9013]));
      expect(groupsForMod(mod), {ModGroupKind.wax});
    });

    test('an already broken install is healed on the next list', () async {
      final gameDir = _tempGameDir();
      addTearDown(() => _cleanup(gameDir));
      final modRoot = p.join(gameDir.path, 'nams', 'mods', 'kimono_old');
      _writeKimonoPayload(modRoot);

      final mods = await ModsService.listInstalled(gameDir.path);
      final mod = mods.singleWhere((m) => m.id == 'kimono_old');

      expect(
          File(p.join(modRoot, 'wax', 'mods', 'Kimono', 'outfits',
                  'kimono_outfits.json'))
              .existsSync(),
          isTrue);
      expect(Directory(p.join(modRoot, 'Kimono')).existsSync(), isFalse);
      expect(mod.data?.hasCompatConfig, isTrue);
      expect(mod.data?.outfitsByStem['pl000d']?.map((c) => c.outfitId),
          containsAll([9010, 9013]));
    });

    test('a correctly packaged wax mod is left untouched', () async {
      final drop = Directory.systemTemp.createTempSync('yp_kimono_ok_');
      final gameDir = _tempGameDir();
      addTearDown(() => _cleanup(drop));
      addTearDown(() => _cleanup(gameDir));
      final root = p.join(drop.path, 'wax_alice');
      _putBytes(root, 'data/pl/pl000d.dat');
      _putBytes(root, 'data/pl/pl000d.dtt');
      _putJson(root, 'wax/mods/NikkeAlice/outfits/NikkeAlice.json',
          _outfitsJson);
      _putJson(root, 'wax/mods/NikkeAlice/item_info/AliceHeadset.json',
          _itemsJson);

      final result = await ModsService.install(
        gameDir.path,
        drop.path,
        requestedName: 'wax_alice',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final modRoot = p.join(gameDir.path, 'nams', 'mods', 'wax_alice');
      expect(
          File(p.join(modRoot, 'wax', 'mods', 'NikkeAlice', 'outfits',
                  'NikkeAlice.json'))
              .existsSync(),
          isTrue);
      expect(
          Directory(p.join(modRoot, 'wax', 'mods'))
              .listSync()
              .whereType<Directory>()
              .map((d) => p.basename(d.path)),
          ['NikkeAlice']);
    });

    test('a wrapper carrying its own data payload is not relocated', () async {
      final drop = Directory.systemTemp.createTempSync('yp_kimono_var_');
      final gameDir = _tempGameDir();
      addTearDown(() => _cleanup(drop));
      addTearDown(() => _cleanup(gameDir));
      final root = p.join(drop.path, 'pack');
      _putBytes(root, 'data/pl/pl000d.dat');
      _putBytes(root, 'VariantB/data/pl/pl000d.dat');
      _putJson(root, 'VariantB/outfits/variant.json', _outfitsJson);

      final result = await ModsService.install(
        gameDir.path,
        drop.path,
        requestedName: 'variant_pack',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final modRoot = p.join(gameDir.path, 'nams', 'mods', 'variant_pack');
      expect(Directory(p.join(modRoot, 'VariantB')).existsSync(), isTrue);
      expect(Directory(p.join(modRoot, 'wax')).existsSync(), isFalse);
    });
  });

  group('real Kimono Wax archive', () {
    final zip = _realKimonoZip();
    final sevenZip = bundledSevenZipPath();
    final ready = File(zip).existsSync() && File(sevenZip).existsSync();

    test('full pipeline: detect, install, wax/mods layout', () async {
      ArchiveService.overrideSevenZipPath = sevenZip;
      addTearDown(() => ArchiveService.overrideSevenZipPath = null);
      final gameDir = _tempGameDir();
      addTearDown(() => _cleanup(gameDir));

      final detected = await ModsService.detectDrop(zip);
      expect(detected.kind, ModKind.data,
          reason: 'rejected: ${detected.errorReason}');
      expect(detected.data?.hasCompatConfig, isTrue);
      expect(detected.data?.hasOutfitConfig, isTrue);
      expect(
          detected.data?.outfitsByStem['pl000d']?.map((c) => c.name) ??
              const [],
          contains('YoRHa Uniform 1'));

      final result = await ModsService.install(
        gameDir.path,
        zip,
        requestedName: 'kimono_wax',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final modRoot =
          p.join(gameDir.path, 'nams', 'mods', result.installedId!);
      final waxMod = p.join(modRoot, 'wax', 'mods', 'Kimono');
      for (final sub in _configSubdirs) {
        expect(Directory(p.join(waxMod, sub)).existsSync(), isTrue,
            reason: 'missing wax/mods/Kimono/$sub');
      }
      expect(Directory(p.join(modRoot, 'Kimono')).existsSync(), isFalse);
      expect(File(p.join(modRoot, 'data', 'pl', 'pl0000.dat')).existsSync(),
          isTrue);
      expect(
          File(p.join(modRoot, 'data', 'misctex', 'misctex_wax_90221.dtt'))
              .existsSync(),
          isTrue);

      final mods = await ModsService.listInstalled(gameDir.path);
      final mod = mods.singleWhere((m) => m.id == 'kimono_wax');
      expect(mod.data?.outfitsByStem['pl0000']?.map((c) => c.name),
          containsAll(['YoRHa Uniform 1', "2P's Body Replica"]));
      expect(groupsForMod(mod), {ModGroupKind.wax});
    },
        timeout: const Timeout(Duration(minutes: 3)),
        skip: ready ? false : 'kimono zip or 7z not present ($zip)');
  });
}
