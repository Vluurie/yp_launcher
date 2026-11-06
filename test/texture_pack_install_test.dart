import 'dart:io';

import 'package:archive/archive.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/texture_install_service.dart';
import 'package:yp_launcher/widgets/textures/texture_pack_install.dart';

void main() {
  late Directory work;

  setUp(() {
    ArchiveService.overrideSevenZipPath = p.join(
      Directory.current.path,
      'assets',
      'bins',
      '7z.exe',
    );
    work = Directory.systemTemp.createTempSync('yp_tex_pack_');
  });

  tearDown(() {
    ArchiveService.overrideSevenZipPath = null;
    try {
      work.deleteSync(recursive: true);
    } catch (_) {}
  });

  String zipOf(String name, Map<String, List<int>> files) {
    final archive = Archive();
    for (final e in files.entries) {
      archive.addFile(ArchiveFile(e.key, e.value.length, e.value));
    }
    final out = File(p.join(work.path, name));
    out.writeAsBytesSync(ZipEncoder().encode(archive));
    return out.path;
  }

  testWidgets('a texture pack zip lands in its own folder', (tester) async {
    late BuildContext context;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (c) {
            context = c;
            return const SizedBox();
          },
        ),
      ),
    );
    final zip = zipOf('HD Pack.zip', {
      'Read me.txt': [1, 2, 3],
      'SK_Res/desktop.ini': [1],
      'SK_Res/inject/textures/04ad8a8f.dds': List<int>.filled(64, 7),
      'SK_Res/inject/textures/0a2b11c3.dds': List<int>.filled(64, 9),
    });
    final gameDir = p.join(work.path, 'game');
    var copyStarted = false;

    final count = await tester.runAsync(
      () => TexturePackInstall.run(
        context,
        paths: [zip],
        textureDir: textureInjectDir(gameDir),
        onCopyStart: () => copyStarted = true,
      ),
    );

    expect(count, 2);
    expect(copyStarted, isTrue);
    final packDir = Directory(p.join(textureInjectDir(gameDir), 'HD Pack'));
    expect(
      packDir
          .listSync(recursive: true)
          .whereType<File>()
          .map((f) => p.basename(f.path))
          .toSet(),
      {'04ad8a8f.dds', '0a2b11c3.dds'},
    );
  });
}
