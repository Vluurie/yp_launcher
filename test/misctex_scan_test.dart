import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/misctex_scan_service.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('misctex_scan_test');
  });

  tearDown(() {
    if (gameDir.existsSync()) gameDir.deleteSync(recursive: true);
  });

  String misctexDir(String modId) => p.join(
        gameDir.path,
        'nams',
        'mods',
        modId,
        'data',
        'misctex',
      );

  void writePair(String modId, String textureName, {bool withDtt = true}) {
    final dir = Directory(misctexDir(modId))..createSync(recursive: true);
    File(p.join(dir.path, 'misctex_$textureName.dat'))
        .writeAsBytesSync(List.filled(2048, 0));
    if (withDtt) {
      File(p.join(dir.path, 'misctex_$textureName.dtt'))
          .writeAsBytesSync(List.filled(1024, 0));
    }
  }

  test('returns nothing when there are no mods', () async {
    expect(await MisctexScanService.scan(gameDir.path), isEmpty);
  });

  test('returns nothing for an empty game dir', () async {
    expect(await MisctexScanService.scan(''), isEmpty);
  });

  test('finds a pair and reads its name back', () async {
    writePair('my_mod', 'item_thumb_8500');

    final found = await MisctexScanService.scan(gameDir.path);
    expect(found, hasLength(1));
    expect(found.first.textureName, 'item_thumb_8500');
    expect(found.first.modId, 'my_mod');
    expect(found.first.fileStem, 'misctex_item_thumb_8500');
  });

  test('adds up the size of both files', () async {
    writePair('my_mod', 'a');
    final found = await MisctexScanService.scan(gameDir.path);
    expect(found.first.totalBytes, 3072);
  });

  test('counts a dat without its dtt', () async {
    writePair('my_mod', 'lonely', withDtt: false);
    final found = await MisctexScanService.scan(gameDir.path);
    expect(found, hasLength(1));
    expect(found.first.totalBytes, 2048);
  });

  test('ignores files that are not misctex pairs', () async {
    final dir = Directory(misctexDir('my_mod'))..createSync(recursive: true);
    File(p.join(dir.path, 'notes.txt')).writeAsStringSync('x');
    File(p.join(dir.path, 'other_thing.dat')).writeAsStringSync('x');
    File(p.join(dir.path, 'misctex_a.dtt')).writeAsStringSync('x');

    expect(await MisctexScanService.scan(gameDir.path), isEmpty);
  });

  test('finds icons across several mods, sorted', () async {
    writePair('zeta_mod', 'b');
    writePair('alpha_mod', 'z');
    writePair('alpha_mod', 'a');

    final found = await MisctexScanService.scan(gameDir.path);
    expect(
      found.map((e) => '${e.modId}/${e.textureName}'),
      ['alpha_mod/a', 'alpha_mod/z', 'zeta_mod/b'],
    );
  });

  test('remove deletes both files', () async {
    writePair('my_mod', 'gone');
    final entry = (await MisctexScanService.scan(gameDir.path)).single;

    await MisctexScanService.remove(entry);

    expect(File(entry.datPath).existsSync(), isFalse);
    expect(File(entry.dttPath).existsSync(), isFalse);
    expect(await MisctexScanService.scan(gameDir.path), isEmpty);
  });

  test('remove does not throw when the dtt is already gone', () async {
    writePair('my_mod', 'half', withDtt: false);
    final entry = (await MisctexScanService.scan(gameDir.path)).single;
    await MisctexScanService.remove(entry);
    expect(File(entry.datPath).existsSync(), isFalse);
  });
}
