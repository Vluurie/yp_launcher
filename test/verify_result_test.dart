import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/models/verify_result.dart';

void main() {
  group('VerifyResult.fromJson', () {
    test('parses the real all-pass payload', () {
      // Captured from `NAMS.exe verify --json` on a healthy install.
      final json = jsonDecode('''
        { "ok": true, "checks": [
          { "name": "steam_install", "status": "ok", "detail": "F:/SteamLibrary/steamapps/common/NieRAutomata" },
          { "name": "nier_exe", "status": "ok", "detail": null },
          { "name": "steam_api64_source", "status": "ok", "detail": null },
          { "name": "runtime_writable", "status": "ok", "detail": "C:/x/runtime" },
          { "name": "runtime_library_cached", "status": "ok", "detail": null }
        ] }
      ''') as Map<String, dynamic>;

      final result = VerifyResult.fromJson(json);
      expect(result.ok, isTrue);
      expect(result.checks, hasLength(5));
      expect(result.checks.first.name, 'steam_install');
      expect(result.checks.first.ok, isTrue);
      expect(result.checks.first.detail, contains('NieRAutomata'));
      expect(result.checks[1].detail, isNull);
    });

    test('parses a failing payload with a detail message', () {
      final json = jsonDecode('''
        { "ok": false, "checks": [
          { "name": "steam_install", "status": "ok", "detail": null },
          { "name": "nier_exe", "status": "fail", "detail": "unsupported NieRAutomata.exe build (Windows 7)" }
        ] }
      ''') as Map<String, dynamic>;

      final result = VerifyResult.fromJson(json);
      expect(result.ok, isFalse);
      final nierExe = result.checks.firstWhere((c) => c.name == 'nier_exe');
      expect(nierExe.ok, isFalse);
      expect(nierExe.detail, contains('Windows 7'));
    });

    test('missing checks yields an empty list, not a crash', () {
      final result =
          VerifyResult.fromJson({'ok': false} as Map<String, dynamic>);
      expect(result.checks, isEmpty);
      expect(result.ok, isFalse);
    });

    test('unknown check name is preserved as-is', () {
      final json = jsonDecode(
        '{ "ok": true, "checks": [ { "name": "future_check", "status": "ok" } ] }',
      ) as Map<String, dynamic>;
      final result = VerifyResult.fromJson(json);
      expect(result.checks.single.name, 'future_check');
    });

    test('a check missing status defaults to not-ok', () {
      final c = VerifyCheck.fromJson({'name': 'x'});
      expect(c.ok, isFalse);
    });
  });
}
