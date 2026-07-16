import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/nier_installation.dart';
import 'package:yp_launcher/services/platform/linux_adapter.dart';
import 'package:yp_launcher/services/platform/macos_adapter.dart';
import 'package:yp_launcher/services/platform/windows_adapter.dart';
import 'package:yp_launcher/services/reveal_service.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';

/// Thrown when the host cannot run the game at all, e.g. no compatibility
/// layer installed. Carries text meant for the user.
class LaunchUnavailable implements Exception {
  final String headline;
  final String detail;

  const LaunchUnavailable(this.headline, this.detail);

  @override
  String toString() => '$headline\n$detail';
}

/// Everything that differs per host OS. Services depend on this rather than
/// branching on [Platform] themselves.
abstract class PlatformAdapter {
  static PlatformAdapter? _current;

  static PlatformAdapter get current => _current ??= _detect();

  @visibleForTesting
  static void overrideCurrent(PlatformAdapter? adapter) => _current = adapter;

  static PlatformAdapter _detect() {
    if (Platform.isWindows) return WindowsAdapter();
    if (Platform.isMacOS) return MacOSAdapter();
    return LinuxAdapter();
  }

  /// Where NAMS.exe, its plugins and 7-Zip live at runtime.
  Future<String> resolveRuntimeDir();

  String get sevenZipExeName;

  List<String> sevenZipCandidates(String runtimeDir);

  bool get canLaunchGame;

  /// Launching works here but is unvalidated on real hardware.
  bool get isExperimental => false;

  Future<LaunchCommand> buildLaunchCommand({
    required String namsExe,
    required String gameDir,
    required String gameExe,
    required String launcherDir,
    required AppLocalizations l10n,
  });

  Future<bool> isGameRunning();

  Future<bool> terminateGame();

  Future<List<NierInstallation>> discoverFast();

  Future<List<NierInstallation>> discoverDeep();

  /// Why [pickedExePath] is not usable, or null when it is fine.
  String? rejectGameSelection(String pickedExePath, AppLocalizations l10n) =>
      null;

  /// Null when the bottle is not known yet; callers must not turn that into
  /// a write.
  Future<String?> resolveNamsSettingsPath(String? gameDir);

  bool get usesNativeTitleBar => false;

  bool get supportsDesktopShortcut => false;

  Future<void> reveal(String target, {bool isFile = false}) =>
      revealInFileManager(target, isFile: isFile);

  /// A temp dir on the same volume as the game, so installs can rename instead
  /// of copy. Falls back to the system temp when the game volume is unusable.
  Directory createScopedTemp(String? gameDir, String prefix) {
    if (gameDir != null && gameDir.isNotEmpty) {
      try {
        final root = Directory(p.join(gameDir, 'nams', '.tmp'))
          ..createSync(recursive: true);
        return root.createTempSync(prefix);
      } catch (_) {}
    }
    return Directory.systemTemp.createTempSync(prefix);
  }

  List<Directory> tempSweepRoots(String? gameDir) => [
        Directory.systemTemp,
        if (gameDir != null && gameDir.isNotEmpty)
          Directory(p.join(gameDir, 'nams', '.tmp')),
      ];
}
