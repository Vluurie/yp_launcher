import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mod_profiles_service.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('yp_profiles_');
    Directory(p.join(gameDir.path, 'nams', 'mods')).createSync(recursive: true);
  });

  tearDown(() {
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  String nams(String name) => p.join(gameDir.path, 'nams', name);

  void writeSidecars(String marker) {
    File(nams('disabled_mods.toml'))
        .writeAsStringSync('disabled = ["mods/$marker"]\n');
    File(nams('default_mods.toml'))
        .writeAsStringSync('outfit_bare = ["mods/$marker"]\n');
    File(nams('mod_load_order.toml'))
        .writeAsStringSync('load_order = ["mods/$marker"]\n');
  }

  Matcher hasMarker(String marker) => contains('mods/$marker');

  void expectActiveSidecars(String marker) {
    expect(File(nams('disabled_mods.toml')).readAsStringSync(),
        hasMarker(marker));
    expect(
        File(nams('default_mods.toml')).readAsStringSync(), hasMarker(marker));
    expect(File(nams('mod_load_order.toml')).readAsStringSync(),
        hasMarker(marker));
  }

  void expectParkedSidecars(String profile, String marker) {
    expect(File(nams('disabled_mods_profile_$profile.toml')).readAsStringSync(),
        hasMarker(marker));
    expect(File(nams('default_mods_profile_$profile.toml')).readAsStringSync(),
        hasMarker(marker));
    expect(
        File(nams('mod_load_order_profile_$profile.toml')).readAsStringSync(),
        hasMarker(marker));
  }

  void expectNoActiveSidecars() {
    expect(File(nams('disabled_mods.toml')).existsSync(), isFalse);
    expect(File(nams('default_mods.toml')).existsSync(), isFalse);
    expect(File(nams('mod_load_order.toml')).existsSync(), isFalse);
  }

  group('createProfile', () {
    test('parks every sidecar under the previous profile', () async {
      writeSidecars('from_default');

      await ModProfilesService.createProfile(gameDir.path, 'second');

      expectParkedSidecars('default', 'from_default');
      expectNoActiveSidecars();
    });

    test('a fresh profile starts with an empty mods folder', () async {
      File(p.join(gameDir.path, 'nams', 'mods', 'marker.txt'))
          .writeAsStringSync('x');

      final state =
          await ModProfilesService.createProfile(gameDir.path, 'second');

      expect(state.activeName, 'second');
      expect(
          Directory(p.join(gameDir.path, 'nams', 'mods')).listSync(), isEmpty);
      expect(
          File(p.join(
                  gameDir.path, 'nams', 'mods_profile_default', 'marker.txt'))
              .existsSync(),
          isTrue);
    });
  });

  group('switchProfile', () {
    test('every sidecar follows the profile it belongs to', () async {
      writeSidecars('alpha');
      await ModProfilesService.createProfile(gameDir.path, 'beta');
      writeSidecars('beta');

      await ModProfilesService.switchProfile(gameDir.path, 'default');

      expectActiveSidecars('alpha');
      expectParkedSidecars('beta', 'beta');
    });

    test('switching back restores the other profile again', () async {
      writeSidecars('alpha');
      await ModProfilesService.createProfile(gameDir.path, 'beta');
      writeSidecars('beta');

      await ModProfilesService.switchProfile(gameDir.path, 'default');
      await ModProfilesService.switchProfile(gameDir.path, 'beta');

      expectActiveSidecars('beta');
      expectParkedSidecars('default', 'alpha');
    });

    test('mods folders swap with the profile', () async {
      File(p.join(gameDir.path, 'nams', 'mods', 'alpha.txt'))
          .writeAsStringSync('x');
      await ModProfilesService.createProfile(gameDir.path, 'beta');
      File(p.join(gameDir.path, 'nams', 'mods', 'beta.txt'))
          .writeAsStringSync('x');

      await ModProfilesService.switchProfile(gameDir.path, 'default');

      expect(
          File(p.join(gameDir.path, 'nams', 'mods', 'alpha.txt')).existsSync(),
          isTrue);
      expect(
          File(p.join(gameDir.path, 'nams', 'mods', 'beta.txt')).existsSync(),
          isFalse);
    });

    test('a profile with no sidecars leaves none behind', () async {
      await ModProfilesService.createProfile(gameDir.path, 'beta');
      writeSidecars('beta');

      await ModProfilesService.switchProfile(gameDir.path, 'default');

      expectNoActiveSidecars();
      expectParkedSidecars('beta', 'beta');
    });

    test('a partial sidecar set travels without inventing the others',
        () async {
      File(nams('default_mods.toml'))
          .writeAsStringSync('outfit_bare = ["mods/only_default"]\n');

      await ModProfilesService.createProfile(gameDir.path, 'second');

      expect(File(nams('default_mods_profile_default.toml')).existsSync(),
          isTrue);
      expect(File(nams('disabled_mods_profile_default.toml')).existsSync(),
          isFalse);
      expect(File(nams('mod_load_order_profile_default.toml')).existsSync(),
          isFalse);
    });

    test('switching to the active profile is a no-op', () async {
      writeSidecars('alpha');

      await ModProfilesService.switchProfile(gameDir.path, 'default');

      expectActiveSidecars('alpha');
    });
  });

  group('renameProfile', () {
    test('an inactive profile takes all its sidecars along', () async {
      writeSidecars('alpha');
      await ModProfilesService.createProfile(gameDir.path, 'beta');

      await ModProfilesService.renameProfile(gameDir.path, 'default', 'renamed');

      expectParkedSidecars('renamed', 'alpha');
      expect(File(nams('default_mods_profile_default.toml')).existsSync(),
          isFalse);
    });
  });

  group('deleteProfile', () {
    test('removes every sidecar of the deleted profile', () async {
      writeSidecars('alpha');
      await ModProfilesService.createProfile(gameDir.path, 'beta');

      await ModProfilesService.deleteProfile(gameDir.path, 'default');

      expect(File(nams('disabled_mods_profile_default.toml')).existsSync(),
          isFalse);
      expect(File(nams('default_mods_profile_default.toml')).existsSync(),
          isFalse);
      expect(File(nams('mod_load_order_profile_default.toml')).existsSync(),
          isFalse);
    });

    test('leaves the surviving profile untouched', () async {
      await ModProfilesService.createProfile(gameDir.path, 'beta');
      writeSidecars('beta');
      await ModProfilesService.createProfile(gameDir.path, 'gamma');

      await ModProfilesService.deleteProfile(gameDir.path, 'default');

      expectParkedSidecars('beta', 'beta');
    });
  });
}
