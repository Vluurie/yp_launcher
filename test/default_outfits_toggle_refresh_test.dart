import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/default_mods_state.dart';
import 'package:yp_launcher/services/default_mods_service.dart';

Directory _gameDir() {
  final dir = Directory.systemTemp.createTempSync('yp_defoutfit_');
  Directory(p.join(dir.path, 'nams')).createSync(recursive: true);
  return dir;
}

void main() {
  late Directory gameDir;
  late ProviderContainer container;

  setUp(() {
    gameDir = _gameDir();
    container = ProviderContainer();
  });

  tearDown(() {
    container.dispose();
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  ConfigStateController config() =>
      container.read(configStateControllerProvider.notifier);

  DefaultModsData defaults() =>
      container.read(defaultModsStateControllerProvider);

  Future<void> starOneMod() => DefaultModsService.setDefault(
        gameDir.path,
        'mods/kimono/pl/pl000d',
        DefaultKind.outfitConfig,
      );

  test('turning the feature on refreshes the stars without a restart',
      () async {
    await config().loadConfigs(gameDir.path);
    await starOneMod();
    await container
        .read(defaultModsStateControllerProvider.notifier)
        .load(gameDir.path);

    expect(defaults().defaultOutfitsEnabled, isFalse,
        reason: 'feature starts off, so no stars are shown');
    expect(defaults().entries, isEmpty);

    config().updateNams(NamsFields.experimentalDefaultOutfits.key, true);
    await config().saveConfigs(gameDir.path);

    expect(defaults().defaultOutfitsEnabled, isTrue,
        reason: 'saving the toggle must reload the default-mods state');
    expect(defaults().entries.map((e) => e.path), ['mods/kimono/pl/pl000d'],
        reason: 'the starred mod must appear without a restart');
  });

  test('turning the feature off clears the stars right away', () async {
    await config().loadConfigs(gameDir.path);
    await starOneMod();
    config().updateNams(NamsFields.experimentalDefaultOutfits.key, true);
    await config().saveConfigs(gameDir.path);
    expect(defaults().entries, isNotEmpty);

    config().updateNams(NamsFields.experimentalDefaultOutfits.key, false);
    await config().saveConfigs(gameDir.path);

    expect(defaults().defaultOutfitsEnabled, isFalse);
    expect(defaults().entries, isEmpty);
  });

  test('the toggle survives a round trip through disk', () async {
    await config().loadConfigs(gameDir.path);
    config().updateNams(NamsFields.experimentalDefaultOutfits.key, true);
    await config().saveConfigs(gameDir.path);

    final reread = ProviderContainer();
    addTearDown(reread.dispose);
    await reread
        .read(configStateControllerProvider.notifier)
        .loadConfigs(gameDir.path);

    expect(
      reread.read(configStateControllerProvider)
          .namsValues[NamsFields.experimentalDefaultOutfits.key],
      isTrue,
    );
  });

  test('saving an unrelated key leaves the starred entries alone', () async {
    await config().loadConfigs(gameDir.path);
    await starOneMod();
    config().updateNams(NamsFields.experimentalDefaultOutfits.key, true);
    await config().saveConfigs(gameDir.path);
    final before = defaults().entries.map((e) => e.path).toList();

    config().updateNams(NamsFields.validateScripts.key, true);
    await config().saveConfigs(gameDir.path);

    expect(defaults().entries.map((e) => e.path), before);
    expect(defaults().defaultOutfitsEnabled, isTrue);
  });
}
