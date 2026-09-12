import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:yp_launcher/src/rust/frb_generated.dart';

class RustBridge {
  RustBridge._();

  static Future<void>? _initialization;

  static Future<void> initialize() {
    return _initialization ??= RustLib.init(externalLibrary: bundledLibrary())
        .catchError((error) {
      _initialization = null;
      throw error;
    });
  }

  static ExternalLibrary? bundledLibrary() {
    final executableDir = File(Platform.resolvedExecutable).parent;
    late final String path;
    if (Platform.isWindows) {
      path = '${executableDir.path}/yp_formats.dll';
    } else if (Platform.isLinux) {
      path = '${executableDir.path}/lib/libyp_formats.so';
    } else if (Platform.isMacOS) {
      path = '${executableDir.parent.path}/Frameworks/libyp_formats.dylib';
    } else {
      return null;
    }
    return File(path).existsSync() ? ExternalLibrary.open(path) : null;
  }
}
