import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/services/gpu_preference_service.dart';
import 'package:yp_launcher/services/launch_failure.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';

class LaunchOutcome {
  final bool started;
  final LaunchFailure? failure;
  const LaunchOutcome.started()
      : started = true,
        failure = null;
  const LaunchOutcome.failed(this.failure) : started = false;
}

class ProcessService {
  static Future<LaunchOutcome> startNierAutomata({
    required String installDirectory,
    required VoidCallback onProcessStopped,
    required AppLocalizations l10n,
  }) async {
    try {
      await LauncherSetupService.ensureReady();

      final missing = await LauncherSetupService.findMissingFiles();
      if (missing.isNotEmpty) {
        return LaunchOutcome.failed(LaunchFailure(
          headline: AppStrings.errorFilesQuarantined(missing.join(', ')),
          rawOutput:
              'Missing required files in launcher directory:\n${missing.join('\n')}\n\n'
              'These files were quarantined or removed by antivirus software.',
        ));
      }

      if (!await LauncherSetupService.isLauncherDirWritable()) {
        return LaunchOutcome.failed(LaunchFailure(
          headline: l10n.errorDirNotWritable,
          rawOutput: l10n.errorDirNotWritableBody(
            LauncherSetupService.launcherDirectory,
          ),
        ));
      }

      final launcherPaths = await LauncherSetupService.getLauncherPaths();

      final nierExePath = path.join(installDirectory, AppStrings.gameExeName);
      if (!await File(nierExePath).exists()) {
        return LaunchOutcome.failed(LaunchFailure(
          headline: AppStrings.errorExeNotFound(installDirectory),
          rawOutput:
              'NieRAutomata.exe was not found at:\n$nierExePath\n\n'
              'The game install path saved in the launcher may be wrong, '
              'or the drive letter changed.',
        ));
      }

      final namsDir = path.join(installDirectory, 'nams');
      if (!await LauncherSetupService.isDirWritable(namsDir)) {
        return LaunchOutcome.failed(LaunchFailure(
          headline: l10n.errorGameDirNotWritable,
          rawOutput: l10n.errorGameDirNotWritableBody(installDirectory, namsDir),
        ));
      }

      await ModsService.syncDlcSlots(installDirectory);

      final LaunchCommand command;
      try {
        command = await PlatformAdapter.current.buildLaunchCommand(
          namsExe: launcherPaths['namsExe']!,
          gameDir: installDirectory,
          gameExe: nierExePath,
          launcherDir: launcherPaths['launcherDir']!,
          l10n: l10n,
        );
      } on LaunchUnavailable catch (e) {
        return LaunchOutcome.failed(
          LaunchFailure(headline: e.headline, rawOutput: e.detail),
        );
      }

      var preferGpu = true;
      try {
        final prefs = await SharedPreferences.getInstance();
        preferGpu =
            prefs.getBool(AppStrings.prefKeyPreferDedicatedGpu) ?? true;
      } catch (_) {}
      GpuPreferenceService.apply(
        launcherPaths['namsExe']!,
        enabled: preferGpu,
      );

      final process = await Process.start(
        command.command,
        command.args,
        workingDirectory: command.cwd,
        environment: GpuPreferenceService.mergedLaunchEnv(
          command.env,
          enabled: preferGpu,
        ),
        mode: ProcessStartMode.normal,
      );

      final stdoutBuf = StringBuffer();
      final stderrBuf = StringBuffer();
      const decoder = Utf8Decoder(allowMalformed: true);
      process.stdout.transform(decoder).listen(stdoutBuf.write);
      process.stderr.transform(decoder).listen(stderrBuf.write);

      final exitFuture = process.exitCode;
      final graceFuture = Future<int?>.delayed(
        const Duration(seconds: 10),
        () => null,
      );

      final exitCode = await Future.any([exitFuture, graceFuture]);

      if (exitCode == null) {
        unawaited(_monitorProcess(process, onProcessStopped));
        return const LaunchOutcome.started();
      }

      final combined = '${stdoutBuf.toString()}${stderrBuf.toString()}';
      final logPath = await _writeCapturedLog(combined, exitCode: exitCode);

      final parsed = LaunchFailureParser.parse(combined, capturedLogPath: logPath);
      if (parsed != null) {
        return LaunchOutcome.failed(parsed);
      }

      return LaunchOutcome.failed(LaunchFailure(
        code: exitCode,
        headline: 'NAMS exited with code $exitCode',
        rawOutput: combined.isEmpty
            ? '(NAMS produced no output)'
            : combined,
        capturedLogPath: logPath,
      ));
    } catch (e) {
      return LaunchOutcome.failed(LaunchFailure(
        headline: 'Internal launcher error',
        rawOutput: e.toString(),
      ));
    }
  }

  static Future<String?> _writeCapturedLog(
    String content, {
    int? exitCode,
  }) async {
    if (content.trim().isEmpty) return null;
    try {
      final dir = Directory(LogService.logsDirectory);
      if (!await dir.exists()) await dir.create(recursive: true);
      final stamp = DateTime.now()
          .toIso8601String()
          .replaceAll(':', '-')
          .replaceAll('.', '-');
      final out = path.join(dir.path, 'nams_$stamp.log');
      final header = StringBuffer()
        ..writeln('NAMS captured output')
        ..writeln('Generated: ${DateTime.now().toIso8601String()}')
        ..writeln('Exit code: ${exitCode ?? '<not exited>'}')
        ..writeln('=' * 60)
        ..writeln();
      await File(out).writeAsString('${header.toString()}$content');
      return out;
    } catch (_) {
      return null;
    }
  }

  static Future<String?> buildLaunchCommandPreview({
    required String installDirectory,
    required AppLocalizations l10n,
  }) async {
    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final command = await PlatformAdapter.current.buildLaunchCommand(
        namsExe: paths['namsExe']!,
        gameDir: installDirectory,
        gameExe: path.join(installDirectory, AppStrings.gameExeName),
        launcherDir: paths['launcherDir']!,
        l10n: l10n,
      );

      final line = [command.command, ...command.args].map(_shellQuote).join(' ');
      if (Platform.isWindows) {
        return 'cd /d ${_shellQuote(command.cwd)}\r\n$line';
      }
      return 'cd ${_shellQuote(command.cwd)} && $line';
    } catch (_) {
      return null;
    }
  }

  static String _shellQuote(String value) {
    if (value.isEmpty) return '""';
    final needsQuoting = value.contains(' ') ||
        value.contains('\t') ||
        value.contains('"');
    if (!needsQuoting) return value;
    final escaped = value.replaceAll('"', r'\"');
    return '"$escaped"';
  }

  static Future<bool> terminateNierAutomata() =>
      PlatformAdapter.current.terminateGame();

  static Future<bool> isNierAutomataRunningAsync() =>
      PlatformAdapter.current.isGameRunning();

  /// Waits on the handle we already hold rather than polling by name, which
  /// cannot confuse NAMS with any other process.
  static Future<void> _monitorProcess(
    Process process,
    VoidCallback onStopped,
  ) async {
    await process.exitCode;
    onStopped();
  }
}

class ProcessException implements Exception {
  final String message;
  ProcessException(this.message);

  @override
  String toString() => message;
}
