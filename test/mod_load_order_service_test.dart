import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mod_load_order_service.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('yp_loadorder_');
  });

  tearDown(() {
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  File orderFile() =>
      File(p.join(gameDir.path, 'nams', 'mod_load_order.toml'));

  test('missing file yields no order', () async {
    expect(await ModLoadOrderService.list(gameDir.path), isEmpty);
  });

  test('an order round-trips through disk', () async {
    await ModLoadOrderService.save(gameDir.path, ['mods/a', 'mods/b']);

    expect(await ModLoadOrderService.list(gameDir.path),
        ['mods/a', 'mods/b']);
    expect(orderFile().existsSync(), isTrue);
  });

  test('order is preserved exactly, not sorted', () async {
    await ModLoadOrderService.save(gameDir.path, ['mods/z', 'mods/a']);

    expect(await ModLoadOrderService.list(gameDir.path),
        ['mods/z', 'mods/a'],
        reason: 'position is the priority, so it must never be reordered');
  });

  test('saving replaces the previous order', () async {
    await ModLoadOrderService.save(gameDir.path, ['mods/a', 'mods/b']);
    await ModLoadOrderService.save(gameDir.path, ['mods/b', 'mods/a']);

    expect(await ModLoadOrderService.list(gameDir.path),
        ['mods/b', 'mods/a']);
  });

  test('backslashes are normalized to forward slashes', () async {
    await ModLoadOrderService.save(gameDir.path, [r'mods\a']);

    expect(await ModLoadOrderService.list(gameDir.path), ['mods/a']);
  });

  test('blank entries are dropped', () async {
    await ModLoadOrderService.save(gameDir.path, ['mods/a', '', '   ']);

    expect(await ModLoadOrderService.list(gameDir.path), ['mods/a']);
  });

  test('an empty order clears the list', () async {
    await ModLoadOrderService.save(gameDir.path, ['mods/a']);
    await ModLoadOrderService.save(gameDir.path, []);

    expect(await ModLoadOrderService.list(gameDir.path), isEmpty);
  });

  test('comments in the file survive a write', () async {
    orderFile().parent.createSync(recursive: true);
    orderFile().writeAsStringSync('# keep me\nload_order = []\n');

    await ModLoadOrderService.save(gameDir.path, ['mods/a']);

    expect(orderFile().readAsStringSync(), contains('# keep me'));
  });

  test('a malformed file degrades to no order instead of throwing', () async {
    orderFile().parent.createSync(recursive: true);
    orderFile().writeAsStringSync('load_order = "not a list"\n');

    expect(await ModLoadOrderService.list(gameDir.path), isEmpty);
  });
}
