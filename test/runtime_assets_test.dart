import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

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
    expect(bins, contains('assets/bins/7zz'));
  });

  test('extracting a bundled binary reproduces it byte for byte', () async {
    final data = await rootBundle.load('assets/bins/7zz');
    final extracted = data.buffer
        .asUint8List(data.offsetInBytes, data.lengthInBytes);

    final onDisk = File('assets/bins/7zz').readAsBytesSync();

    expect(sha256.convert(extracted), sha256.convert(onDisk));
  });
}
