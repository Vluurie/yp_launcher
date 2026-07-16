import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/wine_paths.dart';

const _bottle =
    '/Users/d/Library/Application Support/CrossOver/Bottles/Steam';
const _gameDir =
    '$_bottle/drive_c/Program Files (x86)/Steam/steamapps/common/NieRAutomata';

void main() {
  group('inferWinePrefixFromPath', () {
    test('finds the prefix root above drive_c', () {
      expect(inferWinePrefixFromPath(_gameDir), _bottle);
    });

    test('matches drive_c case-insensitively', () {
      expect(inferWinePrefixFromPath('$_bottle/DRIVE_C/Foo'), _bottle);
    });

    test('returns null outside any prefix', () {
      expect(
        inferWinePrefixFromPath(
          '/Users/d/Library/Application Support/com.vluurie.yplauncher/bins',
        ),
        isNull,
      );
    });

    test('a prefix at the filesystem root resolves to /', () {
      expect(inferWinePrefixFromPath('/drive_c/x'), '/');
    });

    test('a bare relative drive_c has no prefix above it', () {
      expect(inferWinePrefixFromPath('drive_c/x'), isNull);
    });

    test('the first drive_c wins when nested', () {
      expect(
        inferWinePrefixFromPath('/a/drive_c/b/drive_c/c'),
        '/a',
      );
    });
  });

  group('toWinePath', () {
    test('maps inside a prefix to C:', () {
      expect(
        toWinePath(_gameDir),
        r'C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata',
      );
    });

    test('maps outside a prefix to Z:', () {
      expect(
        toWinePath('/Users/d/Library/Application Support/x/bins/NAMS.exe'),
        r'Z:\Users\d\Library\Application Support\x\bins\NAMS.exe',
      );
    });

    test('drive_c itself maps to bare C:', () {
      expect(toWinePath('$_bottle/drive_c'), r'C:\');
    });

    test('matches drive_c case-insensitively', () {
      expect(toWinePath('$_bottle/DRIVE_C/Foo'), r'C:\Foo');
    });
  });

  group('isCrossOverPrefix', () {
    test('accepts a CrossOver bottle', () {
      expect(isCrossOverPrefix(_bottle), isTrue);
    });

    test('accepts a CrossOver Preview bottle', () {
      expect(
        isCrossOverPrefix(
          '/Users/d/Library/Application Support/CrossOver Preview/Bottles/Steam',
        ),
        isTrue,
      );
    });

    test('accepts a Linux .cxoffice bottle', () {
      expect(isCrossOverPrefix('/home/d/.cxoffice/Steam'), isTrue);
    });

    test('accepts a Linux Flatpak CrossOver bottle', () {
      expect(
        isCrossOverPrefix(
          '/home/d/.var/app/com.codeweavers.CrossOver/data/crossover/bottles/Steam',
        ),
        isTrue,
      );
    });

    test('rejects a Proton prefix', () {
      expect(
        isCrossOverPrefix('/home/d/.steam/steam/steamapps/compatdata/524220/pfx'),
        isFalse,
      );
    });
  });

  group('resolveWineUserName', () {
    late Directory tmp;

    setUp(() => tmp = Directory.systemTemp.createTempSync('yp_users_'));
    tearDown(() {
      try {
        tmp.deleteSync(recursive: true);
      } catch (_) {}
    });

    void mkUser(String name) =>
        Directory(p.join(tmp.path, name)).createSync(recursive: true);

    test('skips Public and picks the conventional CrossOver user', () {
      mkUser('Public');
      mkUser('crossover');
      expect(resolveWineUserName(tmp.path), 'crossover');
    });

    test('prefers the host user over steamuser', () {
      final host = Platform.environment['USER'];
      if (host == null) return;
      mkUser('steamuser');
      mkUser(host);
      expect(resolveWineUserName(tmp.path), host);
    });

    test('falls back to the only user present', () {
      mkUser('someoneelse');
      expect(resolveWineUserName(tmp.path), 'someoneelse');
    });

    test('missing users dir does not throw', () {
      expect(
        () => resolveWineUserName(p.join(tmp.path, 'nope')),
        returnsNormally,
      );
    });
  });

  group('getWineRoamingPath', () {
    test('builds the Roaming path under the resolved user', () {
      final tmp = Directory.systemTemp.createTempSync('yp_prefix_');
      addTearDown(() {
        try {
          tmp.deleteSync(recursive: true);
        } catch (_) {}
      });
      Directory(p.join(tmp.path, 'drive_c', 'users', 'crossover'))
          .createSync(recursive: true);

      expect(
        getWineRoamingPath(tmp.path),
        p.join(tmp.path, 'drive_c', 'users', 'crossover', 'AppData', 'Roaming'),
      );
    });
  });
}
