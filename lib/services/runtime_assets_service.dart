import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';

/// Unpacks the bundled Windows binaries and 7-Zip into a writable directory.
/// NAMS resolves plugins/, runtime/ and logs/ next to its own exe, and the
/// signed .app bundle is sealed, so they cannot stay in the bundle.
class RuntimeAssetsService {
  static const _assetPrefix = 'assets/bins/';

  static Future<String>? _pending;

  static Future<String> ensureExtracted() => _pending ??= _extract();

  static Future<String> _extract() async {
    final adapter = PlatformAdapter.current;
    final dir = await adapter.resolveRuntimeDir();
    if (!adapter.needsRuntimeExtraction) return dir;

    final assets = await _bundledAssets();
    if (assets.isEmpty) return dir;

    final stamp = File(p.join(dir, '.stamp'));
    final expected = _stampFor(assets);
    if (await _stampMatches(stamp, expected)) return dir;

    for (final key in assets) {
      final dest = File(p.join(dir, key.substring(_assetPrefix.length)));
      await dest.parent.create(recursive: true);

      final data = await rootBundle.load(key);
      await dest.writeAsBytes(
        data.buffer.asUint8List(data.offsetInBytes, data.lengthInBytes),
        flush: true,
      );

      if (p.basename(dest.path) == adapter.sevenZipExeName) {
        await Process.run('chmod', ['+x', dest.path]);
      }
    }

    await stamp.writeAsString(expected);
    return dir;
  }

  static Future<List<String>> _bundledAssets() async {
    try {
      final manifest = await AssetManifest.loadFromAssetBundle(rootBundle);
      return manifest
          .listAssets()
          .where((key) => key.startsWith(_assetPrefix))
          .toList()
        ..sort();
    } catch (_) {
      return const [];
    }
  }

  static String _stampFor(List<String> assets) {
    final digest = sha256.convert(utf8.encode(assets.join('\n')));
    return '${AppStrings.appVersion}|$digest';
  }

  static Future<bool> _stampMatches(File stamp, String expected) async {
    try {
      return await stamp.exists() && await stamp.readAsString() == expected;
    } catch (_) {
      return false;
    }
  }
}
