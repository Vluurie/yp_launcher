import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/nams_settings_state.dart';

void main() {
  test('a changed keybind is read back instead of the default', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final notifier = container.read(
      namsSettingsStateControllerProvider.notifier,
    );

    for (final (field, key) in [
      (YpKeybindFields.openWorkspace, 'F3'),
      (YpKeybindFields.freezeGame, 'G'),
      (YpKeybindFields.devMode, 'Home'),
    ]) {
      notifier.updateKeybind(field.section!, field.key, key);
      final binds = container.read(namsSettingsStateControllerProvider).keybinds;
      expect(field.valueIn(binds), key);
    }
  });

  test('keybinds loaded from settings.json show their saved value', () {
    final keybinds = <String, dynamic>{
      'main': {'yorha_protocol': 'F2'},
      'yorha_protocol': {'free_cam': 'K'},
    };
    expect(YpKeybindFields.openWorkspace.valueIn(keybinds), 'F2');
    expect(YpKeybindFields.freeCam.valueIn(keybinds), 'K');
    expect(YpKeybindFields.maxSpeed.valueIn(keybinds), 'F6');
  });
}
