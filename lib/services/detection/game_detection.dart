import 'dart:io';
import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/isolate_service.dart';

enum ExeVariant { missing, original, wolfLimitBreak, unknown }

class GameDetection {
  GameDetection._();

  static Future<List<String>> detectHDTextures(String gameDir) async {
    final found = <String>[];

    final skRes = Directory(path.join(gameDir, 'SK_Res'));
    if (await skRes.exists() && await _isNonEmpty(skRes)) {
      found.add('SK_Res/');
    }

    return found;
  }

  static Future<bool> _isNonEmpty(Directory dir) async {
    await for (final _ in dir.list(recursive: false)) {
      return true;
    }
    return false;
  }

  static const String wolfLimitBreakSha256 =
      'c9904a39e448d6cb3c98c28a90159cf9314753b0cfea5ceb4e76a12d3308a355';

  /// Checks if DLC `data100.cpk` is present and ~928 MB. Steam's
  /// "3C3C-Concert & Costume Pack" DLC ships exactly this file at this
  /// size (with a small tolerance). Other unrelated mods can ship a
  /// `data100.cpk` of a different size, hence the size guard.
  static Future<bool> hasDlc(String gameDir) async {
    final file = File(path.join(gameDir, 'data', 'data100.cpk'));
    if (!await file.exists()) return false;
    final size = await file.length();
    const lower = (928 - 5) * 1024 * 1024;
    const upper = 937 * 1024 * 1024;
    return size >= lower && size < upper;
  }

  static Future<ExeVariant> detectExeVariant(String gameDir) async {
    final exePath = path.join(gameDir, AppStrings.gameExeName);
    if (!await File(exePath).exists()) return ExeVariant.missing;
    final hash = await IsolateService.run(_hashFileSync, exePath);
    if (hash == null) return ExeVariant.unknown;
    if (hash == wolfLimitBreakSha256) return ExeVariant.wolfLimitBreak;
    return ExeVariant.original;
  }
}

String? _hashFileSync(String filePath) {
  try {
    final bytes = File(filePath).readAsBytesSync();
    return sha256.convert(bytes).toString();
  } catch (_) {
    return null;
  }
}
