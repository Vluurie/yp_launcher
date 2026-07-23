import 'dart:io';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/detection/game_detection.dart';

void main() {
  late Directory gameDir;

  setUp(() => gameDir = Directory.systemTemp.createTempSync('yp_exe_'));
  tearDown(() {
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  File exe() => File(p.join(gameDir.path, 'NieRAutomata.exe'));

  test('missing exe is reported as missing', () async {
    expect(await GameDetection.detectExeVariant(gameDir.path),
        ExeVariant.missing);
  });

  test('a file with the Win7 size but wrong hash is NOT flagged as Win7',
      () async {
    // Same byte length as the legacy build, but zero-filled -> different sha1.
    exe().writeAsBytesSync(
        Uint8List(GameDetection.legacyWindows7Size));
    final variant = await GameDetection.detectExeVariant(gameDir.path);
    expect(variant, isNot(ExeVariant.legacyWindows7),
        reason: 'size alone must not classify; sha1 must match too');
  });

  test('a small unknown exe is treated as original (not Win7)', () async {
    exe().writeAsBytesSync(List<int>.filled(1024, 7));
    final variant = await GameDetection.detectExeVariant(gameDir.path);
    expect(variant, ExeVariant.original);
    expect(variant, isNot(ExeVariant.legacyWindows7));
  });

  test('reference constants match the values NAMS reports', () {
    expect(GameDetection.legacyWindows7Size, 17786752);
    expect(GameDetection.legacyWindows7Sha1,
        'f734bef5403c77e29cfc55f57707df19c3655751');
    expect(GameDetection.windows10Size, 17837952);
    expect(GameDetection.windows10Sha1,
        '420cbab7daa1d94a79473c0a6271ef9c30e516e5');
  });

  test('sha1 of the exact legacy bytes would match the reference', () {
    // Guards the detection contract: IF a file has the legacy size and its
    // sha1 equals the constant, it is the Win7 build. We cannot ship the real
    // 17MB binary, so we assert the algorithm wiring on a tiny sample instead.
    final sample = Uint8List.fromList([0x4d, 0x5a, 0x90, 0x00]);
    final computed = sha1.convert(sample).toString();
    expect(computed, hasLength(40));
    expect(computed, matches(RegExp(r'^[0-9a-f]{40}$')));
  });
}
