import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/config_state.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('yp_broken_toml_');
    Directory(p.join(gameDir.path, 'nams')).createSync(recursive: true);
  });

  File lodmodFile() => File(p.join(gameDir.path, 'nams', 'lodmod.toml'));

  test('a broken lodmod.toml is reported and never written back', () async {
    const broken = 'enabled = true\nfxaa = false\nfxaa = true\n';
    lodmodFile().writeAsStringSync(broken);

    final container = ProviderContainer();
    addTearDown(container.dispose);
    final notifier = container.read(configStateControllerProvider.notifier);

    await notifier.loadConfigs(gameDir.path);

    final state = container.read(configStateControllerProvider);
    expect(state.lodmodError, isNotNull);
    expect(state.lodmodValues, isEmpty);

    notifier.updateLodmod(LodModFields.fxaa.key, true);
    await notifier.saveConfigs(gameDir.path);

    expect(lodmodFile().readAsStringSync(), broken);
  });

  test('a valid lodmod.toml still saves', () async {
    lodmodFile().writeAsStringSync('enabled = false\nfxaa = false\n');

    final container = ProviderContainer();
    addTearDown(container.dispose);
    final notifier = container.read(configStateControllerProvider.notifier);

    await notifier.loadConfigs(gameDir.path);
    expect(container.read(configStateControllerProvider).lodmodError, isNull);

    notifier.updateLodmod(LodModFields.fxaa.key, true);
    await notifier.saveConfigs(gameDir.path);

    expect(lodmodFile().readAsStringSync(), contains('fxaa = true'));
  });
}
