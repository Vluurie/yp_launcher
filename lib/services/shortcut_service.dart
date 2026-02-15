import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as path;
import 'package:win32/win32.dart';
import 'package:yp_launcher/constants/app_strings.dart';

class ShortcutService {
  static Future<bool> createDesktopShortcut({
    required String gameDirectory,
  }) async {
    if (!Platform.isWindows) return false;

    try {
      final launcherExe = Platform.resolvedExecutable;

      final desktopPath = _getDesktopPath();
      if (desktopPath == null) return false;

      final shortcutPath =
          path.join(desktopPath, '${AppStrings.shortcutName}.lnk');

      final gameExePath =
          path.join(gameDirectory, AppStrings.gameExeName);

      return _createShortcut(
        shortcutPath: shortcutPath,
        targetPath: launcherExe,
        arguments: '--launch',
        workingDirectory: path.dirname(launcherExe),
        iconPath: gameExePath,
        description: AppStrings.shortcutDescription,
      );
    } catch (_) {
      return false;
    }
  }

  static String? _getDesktopPath() {
    final userProfile = Platform.environment['USERPROFILE'];
    if (userProfile == null) return null;
    final desktop = path.join(userProfile, 'Desktop');
    if (Directory(desktop).existsSync()) return desktop;
    return null;
  }

  static bool _createShortcut({
    required String shortcutPath,
    required String targetPath,
    required String arguments,
    required String workingDirectory,
    required String iconPath,
    required String description,
  }) {
    final hr = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    if (FAILED(hr) && hr != RPC_E_CHANGED_MODE) return false;

    try {
      final shellLink = ShellLink.createInstance();

      final pTarget = targetPath.toNativeUtf16();
      final pArgs = arguments.toNativeUtf16();
      final pWorkDir = workingDirectory.toNativeUtf16();
      final pIcon = iconPath.toNativeUtf16();
      final pDesc = description.toNativeUtf16();

      try {
        shellLink.setPath(pTarget);
        shellLink.setArguments(pArgs);
        shellLink.setWorkingDirectory(pWorkDir);
        shellLink.setIconLocation(pIcon, 0);
        shellLink.setDescription(pDesc);

        final persistFile = IPersistFile.from(shellLink);
        final pShortcutPath = shortcutPath.toNativeUtf16();

        try {
          final saveResult = persistFile.save(pShortcutPath, TRUE);
          return SUCCEEDED(saveResult);
        } finally {
          free(pShortcutPath);
        }
      } finally {
        free(pTarget);
        free(pArgs);
        free(pWorkDir);
        free(pIcon);
        free(pDesc);
      }
    } catch (_) {
      return false;
    } finally {
      CoUninitialize();
    }
  }
}
