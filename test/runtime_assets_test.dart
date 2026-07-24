import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

List<String> get _sevenZipAssets => Platform.isWindows
    ? const ['assets/bins/7z.exe', 'assets/bins/7z.dll']
    : const ['assets/bins/7zz'];

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('the runtime binaries are bundled under assets/bins/', () async {
    final manifest = await AssetManifest.loadFromAssetBundle(rootBundle);
    final bins = manifest
        .listAssets()
        .where((key) => key.startsWith('assets/bins/'))
        .toSet();

    expect(bins, contains('assets/bins/NAMS.exe'));
    expect(bins, contains('assets/bins/plugins/yorha_protocol.dll'));
    for (final asset in _sevenZipAssets) {
      expect(bins, contains(asset));
    }
  });

  test('extracting a bundled binary reproduces it byte for byte', () async {
    for (final asset in _sevenZipAssets) {
      final data = await rootBundle.load(asset);
      final extracted =
          data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes);

      final onDisk = File(asset).readAsBytesSync();

      expect(sha256.convert(extracted), sha256.convert(onDisk));
    }
  });
}
