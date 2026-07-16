import 'dart:io';
import 'dart:isolate';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/texture_install_service.dart';

Directory _packWith(Directory root, String name, List<String> ddsFiles) {
  final dir = Directory(p.join(root.path, name))..createSync(recursive: true);
  for (final f in ddsFiles) {
    final file = File(p.join(dir.path, f));
    file.parent.createSync(recursive: true);
    file.writeAsBytesSync(List<int>.filled(8, 0));
  }
  return dir;
}

Future<int> _install(List<String> paths, String textureDir) async {
  final port = ReceivePort();
  await Isolate.spawn(
    installTexturesWithProgress,
    TextureInstallProgressParams(
      paths: paths,
      textureDir: textureDir,
      sendPort: port.sendPort,
    ),
  );
  var count = 0;
  await for (final message in port) {
    if (message is Map<String, dynamic> && message['type'] == 'done') {
      count = message['count'] as int? ?? 0;
      port.close();
      break;
    }
  }
  return count;
}

List<String> _relDdsPaths(Directory root) {
  if (!root.existsSync()) return const [];
  return root
      .listSync(recursive: true)
      .whereType<File>()
      .where((f) => f.path.toLowerCase().endsWith('.dds'))
      .map((f) => p.relative(f.path, from: root.path).replaceAll('\\', '/'))
      .toList()
    ..sort();
}

void main() {
  late Directory tmp;
  late Directory textureDir;

  setUp(() {
    tmp = Directory.systemTemp.createTempSync('yp_tex_');
    textureDir = Directory(p.join(tmp.path, 'inject'))..createSync();
  });

  tearDown(() {
    try {
      tmp.deleteSync(recursive: true);
    } catch (_) {}
  });

  test('picks sharing a pack name merge into one folder', () async {
    final src = Directory(p.join(tmp.path, 'src'))..createSync();
    final a = _packWith(src, '2B', ['a.dds']);
    final b = _packWith(src, 'A2', ['b.dds']);
    final c = _packWith(src, 'Desert', ['c.dds']);

    await _install([
      '${a.path}|HD Pack',
      '${b.path}|HD Pack',
      '${c.path}|HD Pack',
    ], textureDir.path);

    final packs = textureDir.listSync().whereType<Directory>().toList();
    expect(packs.length, 1,
        reason: 'every pick from one archive must land in a single pack');
    expect(p.basename(packs.single.path), 'HD Pack');

    expect(_relDdsPaths(textureDir), [
      'HD Pack/a.dds',
      'HD Pack/b.dds',
      'HD Pack/c.dds',
    ]);
  });

  test('distinct pack names stay separate folders', () async {
    final src = Directory(p.join(tmp.path, 'src'))..createSync();
    final a = _packWith(src, '2B', ['a.dds']);
    final b = _packWith(src, 'A2', ['b.dds']);

    await _install(
        ['${a.path}|2B', '${b.path}|A2'], textureDir.path);

    final packs = textureDir
        .listSync()
        .whereType<Directory>()
        .map((d) => p.basename(d.path))
        .toList()
      ..sort();
    expect(packs, ['2B', 'A2']);
  });

  test('merging keeps each pick\'s subfolder structure', () async {
    final src = Directory(p.join(tmp.path, 'src'))..createSync();
    final a = _packWith(src, 'City', ['area/city.dds']);
    final b = _packWith(src, 'Desert', ['area/desert.dds']);

    await _install(
        ['${a.path}|HD Pack', '${b.path}|HD Pack'], textureDir.path);

    expect(_relDdsPaths(textureDir), [
      'HD Pack/area/city.dds',
      'HD Pack/area/desert.dds',
    ]);
  });

  test('sk_res/inject/textures wrappers are stripped when merging', () async {
    final src = Directory(p.join(tmp.path, 'src'))..createSync();
    final a = _packWith(src, '2B', ['SK_Res/inject/textures/a.dds']);

    await _install(['${a.path}|HD Pack'], textureDir.path);

    expect(_relDdsPaths(textureDir), ['HD Pack/a.dds']);
  });
}
