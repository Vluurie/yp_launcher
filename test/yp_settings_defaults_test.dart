import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/nams_settings_state.dart';

void main() {
  test('missing workspace keys read as the overlay defaults', () {
    const data = NamsSettingsData(settings: {});
    expect(data.gameKeybindsGlobal, isFalse);
    expect(data.loadingSpeedupEnabled, isFalse);
    expect(data.shadersEnabled, isTrue);
    expect(data.soundEnabled, isTrue);
    expect(data.editorMcpEnabled, isFalse);
  });

  test('saved workspace values win over the defaults', () {
    const data = NamsSettingsData(
      settings: {
        'shadersEnabled': false,
        'soundEnabled': false,
        'editorMcpEnabled': true,
      },
    );
    expect(data.shadersEnabled, isFalse);
    expect(data.soundEnabled, isFalse);
    expect(data.editorMcpEnabled, isTrue);
  });

  test('missing randomizer keys read as the overlay defaults', () {
    const randomizer = <String, dynamic>{};
    expect(YpRandomizerFields.autoStartOnStartup.valueIn(randomizer), isFalse);
    for (final field in [
      YpRandomizerFields.randomizeGroundEnemies,
      YpRandomizerFields.randomizeFlyingEnemies,
      YpRandomizerFields.allowBigEnemies,
      YpRandomizerFields.includeDlcEnemies,
    ]) {
      expect(field.valueIn(randomizer), isTrue, reason: field.key);
    }
  });
}
