import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/proton.dart';
import 'package:yp_launcher/services/wine/steam_compat_tool.dart';
import 'package:yp_launcher/services/wine/steam_vdf.dart';

import 'fake_steam_tree.dart';

String _steamConfig(Map<String, String> mapping) {
  final entries = mapping.entries
      .map(
        (e) =>
            '\t\t\t\t\t"${e.key}"\n'
            '\t\t\t\t\t{\n'
            '\t\t\t\t\t\t"name"\t\t"${e.value}"\n'
            '\t\t\t\t\t\t"config"\t\t""\n'
            '\t\t\t\t\t\t"priority"\t\t"250"\n'
            '\t\t\t\t\t}\n',
      )
      .join();
  return '"InstallConfigStore"\n'
      '{\n'
      '\t"Software"\n'
      '\t{\n'
      '\t\t"valve"\n'
      '\t\t{\n'
      '\t\t\t"Steam"\n'
      '\t\t\t{\n'
      '\t\t\t\t"CompatToolMapping"\n'
      '\t\t\t\t{\n'
      '$entries'
      '\t\t\t\t}\n'
      '\t\t\t}\n'
      '\t\t}\n'
      '\t}\n'
      '}\n';
}

String _geManifest(String internalName) =>
    '"compatibilitytools"\n'
    '{\n'
    '  "compat_tools"\n'
    '  {\n'
    '    "$internalName" // Internal name of this tool\n'
    '    {\n'
    '      // - The tool can be placed as a subdirectory in compatibilitytools.d, in which case this\n'
    "      //   should be '.'\n"
    '      "install_path" "."\n'
    '\n'
    '      "display_name" "$internalName"\n'
    '      "from_oslist"  "windows"\n'
    '      "to_oslist"    "linux"\n'
    '    }\n'
    '  }\n'
    '}\n';

void main() {
  late FakeSteamTree tree;

  setUp(() => tree = FakeSteamTree.create());
  tearDown(() => tree.dispose());

  group('officialProtonToolName', () {
    test('maps Valve folder names onto the names Steam stores', () {
      const expected = {
        'Proton - Experimental': 'proton_experimental',
        'Proton Hotfix': 'proton_hotfix',
        'Proton 9.0 (Beta)': 'proton_9',
        'Proton 10.0': 'proton_10',
        'Proton 6.3': 'proton_63',
        'Proton 5.13': 'proton_513',
      };
      for (final MapEntry(:key, :value) in expected.entries) {
        expect(officialProtonToolName(key), value, reason: key);
      }
    });

    test('ignores runtimes and other apps', () {
      expect(officialProtonToolName('Proton EasyAntiCheat Runtime'), isNull);
      expect(officialProtonToolName('Steam Linux Runtime 3.0 (sniper)'), isNull);
      expect(officialProtonToolName('NieRAutomata'), isNull);
    });
  });

  group('steamCompatToolName', () {
    test('reads the per-game and the default entry', () {
      final config = parseVdf(
        _steamConfig({'0': 'proton_experimental', nierSteamAppId: 'proton_9'}),
      );
      expect(steamCompatToolName(config, nierSteamAppId), 'proton_9');
      expect(steamCompatToolName(config, '0'), 'proton_experimental');
    });

    test('is null when the game has no entry', () {
      final config = parseVdf(_steamConfig({'0': 'proton_experimental'}));
      expect(steamCompatToolName(config, nierSteamAppId), isNull);
    });
  });

  group('findSteamConfiguredProton', () {
    test('uses the Proton Steam runs NieR with, not the newest install', () {
      final root = tree.addSteamRoot();
      tree.addSteamConfig(
        _steamConfig({'0': 'proton_experimental', nierSteamAppId: 'proton_9'}),
      );
      final nine = tree.addOfficialProton('Proton 9.0 (Beta)');
      tree.addOfficialProton('Proton - Experimental');

      expect(tree.runNative(() => findSteamConfiguredProton(root, root)), nine);
    });

    test('resolves a custom tool by the name in its compatibilitytool.vdf', () {
      final root = tree.addSteamRoot();
      tree.addSteamConfig(_steamConfig({nierSteamAppId: 'GE-Proton9-20'}));
      final ge = tree.addCompatTool('ge-folder', _geManifest('GE-Proton9-20'));

      expect(
        tree.runNative(() => findSteamConfiguredProton(root, root)),
        p.normalize(ge),
      );
    });

    test('falls back to the default tool when the game tool is missing', () {
      final root = tree.addSteamRoot();
      tree.addSteamConfig(
        _steamConfig({'0': 'proton_experimental', nierSteamAppId: 'proton_8'}),
      );
      final experimental = tree.addOfficialProton('Proton - Experimental');

      expect(
        tree.runNative(() => findSteamConfiguredProton(root, root)),
        experimental,
      );
    });

    test('is null without a Steam config', () {
      final root = tree.addSteamRoot();
      tree.addOfficialProton('Proton 9.0');

      expect(tree.runNative(() => findSteamConfiguredProton(root, root)), isNull);
    });
  });
}
