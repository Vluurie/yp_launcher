import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/thirdparty/ini_patch.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';

String? systemKey(String ini, String key) =>
    IniPatch.getKey(ini, 'System', key);

void main() {
  test('an empty d3dx.ini gets the full NAMS compat set', () {
    final out = MigotoRuntime.patchNamsCompat('');

    expect(systemKey(out, 'hook'), 'recommended');
    expect(systemKey(out, 'load_library_redirect'), '1');
    expect(systemKey(out, 'allow_check_interface'), '1');
    expect(systemKey(out, 'allow_create_device'), '1');
    expect(systemKey(out, 'allow_platform_update'), '1');
  });

  test('values a mod shipped are overwritten, not appended', () {
    const shipped = '''
[System]
hook = 0
load_library_redirect = 2
allow_create_device = 0
''';
    final out = MigotoRuntime.patchNamsCompat(shipped);

    expect(systemKey(out, 'hook'), 'recommended');
    expect(systemKey(out, 'load_library_redirect'), '1');
    expect(systemKey(out, 'allow_create_device'), '1');
    expect(RegExp('hook').allMatches(out).length, 1);
    expect(RegExp('load_library_redirect').allMatches(out).length, 1);
  });

  test('other sections a mod ships are left alone', () {
    const shipped = '''
[System]
hook = 0

[Rendering]
override_directory = ShaderFixes

[Hunting]
hunting = 0
''';
    final out = MigotoRuntime.patchNamsCompat(shipped);

    expect(IniPatch.getKey(out, 'Rendering', 'override_directory'),
        'ShaderFixes');
    expect(IniPatch.getKey(out, 'Hunting', 'hunting'), '0');
    expect(systemKey(out, 'hook'), 'recommended');
  });
}
