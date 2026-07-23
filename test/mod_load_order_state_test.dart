import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/providers/mod_load_order_state.dart';
import 'package:yp_launcher/services/mod_load_order_service.dart';

void main() {
  late Directory gameDir;
  late ProviderContainer container;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('yp_loadorder_state_');
    container = ProviderContainer();
  });

  tearDown(() {
    container.dispose();
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  ModLoadOrderStateController notifier() =>
      container.read(modLoadOrderStateControllerProvider.notifier);

  List<String> uiOrder() =>
      container.read(modLoadOrderStateControllerProvider).order;

  File orderFile() =>
      File(p.join(gameDir.path, 'nams', 'mod_load_order.toml'));

  test('the UI list is highest-priority-first', () async {
    await notifier().save(gameDir.path, ['mods/winner', 'mods/loser']);

    expect(uiOrder(), ['mods/winner', 'mods/loser']);
  });

  test('the file stores lowest-first so NAMS sees the winner last', () async {
    await notifier().save(gameDir.path, ['mods/winner', 'mods/loser']);

    expect(await ModLoadOrderService.list(gameDir.path),
        ['mods/loser', 'mods/winner'],
        reason: 'NAMS resolves the LAST entry as the winner, the UI shows '
            'the winner first, so the two must be stored inverted');
  });

  test('save then load is stable, not flipped each time', () async {
    await notifier().save(gameDir.path, ['mods/a', 'mods/b', 'mods/c']);
    await notifier().load(gameDir.path);
    await notifier().load(gameDir.path);

    expect(uiOrder(), ['mods/a', 'mods/b', 'mods/c'],
        reason: 'reversing on both read and write must cancel out');
  });

  test('a file written by hand is shown winner-first', () async {
    orderFile().parent.createSync(recursive: true);
    orderFile().writeAsStringSync(
      'load_order = ["mods/loser", "mods/winner"]\n',
    );

    await notifier().load(gameDir.path);

    expect(uiOrder(), ['mods/winner', 'mods/loser']);
  });

  test('indexOf reports UI position', () async {
    await notifier().save(gameDir.path, ['mods/a', 'mods/b']);
    final data = container.read(modLoadOrderStateControllerProvider);

    expect(data.indexOf('mods/a'), 0);
    expect(data.indexOf('mods/b'), 1);
    expect(data.indexOf('mods/missing'), isNull);
  });

  test('an empty game dir is a no-op', () async {
    await notifier().load('');

    expect(uiOrder(), isEmpty);
  });
}
