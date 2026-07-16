import 'dart:io';

import 'package:path/path.dart' as p;

/// Opens the host file manager at [target], selecting it when [isFile] is set.
/// Never throws; callers invoke this from tap handlers.
Future<void> revealInFileManager(String target, {bool isFile = false}) async {
  if (target.isEmpty) return;
  try {
    if (Platform.isWindows) {
      await Process.run('explorer', isFile ? ['/select,', target] : [target]);
    } else if (Platform.isMacOS) {
      await Process.run('open', isFile ? ['-R', target] : [target]);
    } else {
      await Process.run('xdg-open', [isFile ? p.dirname(target) : target]);
    }
  } catch (_) {}
}
