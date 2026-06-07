import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as path;
import 'package:win32/win32.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';

class ShortcutService {
  static Future<bool> createDesktopShortcut({
    required String gameDirectory,
  }) async {
    if (!Platform.isWindows) return false;

    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final namsExe = paths['namsExe']!;
      final launcherDir = paths['launcherDir']!;

      final desktopPath = _getDesktopPath();
      if (desktopPath == null) return false;

      final shortcutPath = path.join(
        desktopPath,
        '${AppStrings.shortcutName}.lnk',
      );

      final gameExePath = path.join(gameDirectory, AppStrings.gameExeName);
      final arguments =
          '${AppStrings.argRun} ${AppStrings.argNierPath} "$gameDirectory"';

      return _createShortcut(
        shortcutPath: shortcutPath,
        targetPath: namsExe,
        arguments: arguments,
        workingDirectory: launcherDir,
        iconPath: gameExePath,
        description: AppStrings.shortcutDescription,
      );
    } catch (_) {
      return false;
    }
  }

  static String? _getDesktopPath() {
    final userProfile = Platform.environment['USERPROFILE'];
    if (userProfile != null) {
      final desktop = path.join(userProfile, 'Desktop');
      if (Directory(desktop).existsSync()) return desktop;
    }
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
    final hr = CoInitializeEx(COINIT_APARTMENTTHREADED);
    if (FAILED(hr) && hr != RPC_E_CHANGED_MODE) return false;

    final pTarget = PCWSTR(targetPath.toNativeUtf16());
    final pArgs = PCWSTR(arguments.toNativeUtf16());
    final pWorkDir = PCWSTR(workingDirectory.toNativeUtf16());
    final pIcon = PCWSTR(iconPath.toNativeUtf16());
    final pDesc = PCWSTR(description.toNativeUtf16());
    final pShortcutPath = PCWSTR(shortcutPath.toNativeUtf16());

    try {
      final shellLink = createInstance<IShellLink>(ShellLink);
      try {
        shellLink.setPath(pTarget);
        shellLink.setArguments(pArgs);
        shellLink.setWorkingDirectory(pWorkDir);
        shellLink.setIconLocation(pIcon, 0);
        shellLink.setDescription(pDesc);

        final persistFile = IPersistFile.from(shellLink);
        try {
          persistFile.save(pShortcutPath, true);
          return File(shortcutPath).existsSync();
        } finally {
          persistFile.release();
        }
      } finally {
        shellLink.release();
      }
    } catch (_) {
      return false;
    } finally {
      free(pTarget);
      free(pArgs);
      free(pWorkDir);
      free(pIcon);
      free(pDesc);
      free(pShortcutPath);
      CoUninitialize();
    }
  }
}
