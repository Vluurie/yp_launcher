import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/platform/linux_adapter.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/platform/windows_adapter.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';

void main() {
  late Directory root;
  late Directory source;

  setUp(() {
    root = Directory.systemTemp.createTempSync('yp_migoto_nvapi_');
    source = Directory(p.join(root.path, 'incoming'))
      ..createSync(recursive: true);
    File(p.join(source.path, 'd3d11.dll')).writeAsStringSync('migoto');
    File(p.join(source.path, 'nvapi64.dll')).writeAsStringSync('nvidia shim');
    File(
      p.join(source.path, 'd3dcompiler_46.dll'),
    ).writeAsStringSync('compiler');
  });

  tearDown(() {
    PlatformAdapter.overrideCurrent(null);
    try {
      root.deleteSync(recursive: true);
    } catch (_) {}
  });

  test('windows keeps nvapi64.dll', () {
    PlatformAdapter.overrideCurrent(WindowsAdapter());
    expect(PlatformAdapter.current.runsGameThroughWine, isFalse);
  });

  test('linux reports that the game runs through wine', () {
    PlatformAdapter.overrideCurrent(LinuxAdapter());
    expect(PlatformAdapter.current.runsGameThroughWine, isTrue);
  });

  test('repair parks an existing nvapi64.dll on wine hosts', () async {
    PlatformAdapter.overrideCurrent(LinuxAdapter());

    final dest = Directory(p.join(root.path, 'installed'))
      ..createSync(recursive: true);
    File(p.join(dest.path, '3dmigoto.dll')).writeAsStringSync('migoto');
    final nvapi = File(p.join(dest.path, 'nvapi64.dll'))
      ..writeAsStringSync('nvidia shim');

    await _RepairOnly(dest.path).repair(root.path);

    expect(nvapi.existsSync(), isFalse);
    expect(
      File(p.join(dest.path, 'nvapi64.dll.disabled-on-wine')).existsSync(),
      isTrue,
    );
    expect(
      File(p.join(dest.path, 'd3dcompiler_46.dll')).existsSync(),
      isFalse,
      reason: 'only nvapi is parked',
    );
  });

  test('repair leaves nvapi64.dll alone on windows', () async {
    PlatformAdapter.overrideCurrent(WindowsAdapter());

    final dest = Directory(p.join(root.path, 'installed_win'))
      ..createSync(recursive: true);
    File(p.join(dest.path, '3dmigoto.dll')).writeAsStringSync('migoto');
    final nvapi = File(p.join(dest.path, 'nvapi64.dll'))
      ..writeAsStringSync('nvidia shim');

    await _RepairOnly(dest.path).repair(root.path);

    expect(nvapi.existsSync(), isTrue);
  });
}

/// Pins [installDir] to a temp folder so the test never touches the real
/// launcher directory.
class _RepairOnly extends MigotoRuntime {
  const _RepairOnly(this._dir);

  final String _dir;

  @override
  String installDir(String gameDir) => _dir;

  @override
  ThirdPartyRuntime get runtime => ThirdPartyRuntime.migoto;
}
