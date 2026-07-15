import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/launch_failure.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/platform_detection_service.dart';
import 'package:win32/win32.dart' if (dart.library.html) '';

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

      final List<String> arguments = _buildRunArguments(gameDir: installDirectory);

      final process = await Process.start(
        launcherPaths['namsExe']!,
        arguments,
        workingDirectory: launcherPaths['launcherDir']!,
        mode: ProcessStartMode.normal,
      );

      final stdoutBuf = StringBuffer();
      final stderrBuf = StringBuffer();
      process.stdout.transform(const SystemEncoding().decoder).listen(stdoutBuf.write);
      process.stderr.transform(const SystemEncoding().decoder).listen(stderrBuf.write);

      final exitFuture = process.exitCode;
      final graceFuture = Future<int?>.delayed(
        const Duration(seconds: 10),
        () => null,
      );

      final exitCode = await Future.any([exitFuture, graceFuture]);

      if (exitCode == null) {
        unawaited(_monitorProcess(AppStrings.namsExeName, onProcessStopped));
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
  }) async {
    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final args = _buildRunArguments(gameDir: installDirectory);
      final exe = paths['namsExe']!;
      final launcherDir = paths['launcherDir']!;
      final quotedArgs = args.map(_shellQuote).join(' ');
      if (Platform.isWindows) {
        return 'cd /d ${_shellQuote(launcherDir)}\r\n'
            '${_shellQuote(exe)} $quotedArgs';
      }
      return 'cd ${_shellQuote(launcherDir)} && '
          '${_shellQuote(exe)} $quotedArgs';
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

  static List<String> _buildRunArguments({required String gameDir}) {
    String formatPath(String filePath) {
      if (Platform.isWindows) {
        return filePath.replaceAll('/', '\\');
      } else if (PlatformDetectionService.isWine) {
        return PlatformDetectionService.unixToWinePath(filePath);
      } else {
        final normalizedPath = filePath.replaceAll('\\', '/');
        if (!normalizedPath.startsWith('/')) {
          return normalizedPath;
        }
        return 'Z:$normalizedPath';
      }
    }

    return <String>[
      AppStrings.argRun,
      AppStrings.argNierPath,
      formatPath(gameDir),
    ];
  }

  static bool terminateNierAutomata() {
    if (Platform.isWindows) {
      return _terminateProcessByName(AppStrings.namsExeName);
    } else if (PlatformDetectionService.isWine) {
      try {
        Process.runSync('wineserver', ['-k']);
        return true;
      } catch (_) {
        try {
          final result = Process.runSync('pkill', [
            '-f',
            AppStrings.namsExeName,
          ]);
          return result.exitCode == 0;
        } catch (_) {
          return false;
        }
      }
    } else {
      try {
        final result = Process.runSync('pkill', ['-f', AppStrings.namsExeName]);
        return result.exitCode == 0;
      } catch (_) {
        return false;
      }
    }
  }

  static bool isNierAutomataRunning() {
    return _isProcessRunning(AppStrings.namsExeName);
  }

  static Future<bool> isNierAutomataRunningAsync() async {
    if (Platform.isWindows) {
      try {
        final result = await Process.run('tasklist', [
          '/FI',
          'IMAGENAME eq ${AppStrings.namsExeName}',
          '/NH',
        ]);
        return result.stdout.toString().toLowerCase().contains(
          AppStrings.namsExeName.toLowerCase(),
        );
      } catch (_) {
        return _isProcessRunning(AppStrings.namsExeName);
      }
    } else {
      try {
        final result = await Process.run('pgrep', [
          '-f',
          AppStrings.namsExeName,
        ]);
        return result.exitCode == 0;
      } catch (_) {
        return false;
      }
    }
  }

  static Future<void> _monitorProcess(
    String processName,
    VoidCallback onStopped,
  ) async {
    while (true) {
      await Future.delayed(const Duration(seconds: 2));
      if (!await isNierAutomataRunningAsync()) {
        await Future.delayed(const Duration(seconds: 2));
        if (!await isNierAutomataRunningAsync()) {
          onStopped();
          return;
        }
      }
    }
  }

  static bool _isProcessRunning(String processName) {
    if (!Platform.isWindows) return false;

    final processIds = calloc<Uint32>(1024);
    final cb = sizeOf<Uint32>() * 1024;
    final cbNeeded = calloc<Uint32>();

    try {
      if (!EnumProcesses(processIds, cb, cbNeeded).value) {
        return false;
      }

      final count = cbNeeded.value ~/ sizeOf<Uint32>();
      final targetName = processName.toLowerCase();

      for (var i = 0; i < count; i++) {
        final processHandle = OpenProcess(
          PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
          false,
          processIds[i],
        ).value;

        if (processHandle != NULL) {
          final exeName = wsalloc(MAX_PATH);
          try {
            if (GetModuleBaseName(processHandle, null, PWSTR(exeName), MAX_PATH)
                    .value >
                0) {
              final currentName = exeName.toDartString().toLowerCase();
              if (currentName.startsWith(targetName)) {
                return true;
              }
            }
          } finally {
            free(exeName);
            CloseHandle(processHandle);
          }
        }
      }
    } catch (_) {
      // ignore
    } finally {
      free(processIds);
      free(cbNeeded);
    }

    return false;
  }

  static bool _terminateProcessByName(String processName) {
    if (!Platform.isWindows) return false;

    final processIds = calloc<Uint32>(1024);
    final cb = sizeOf<Uint32>() * 1024;
    final cbNeeded = calloc<Uint32>();

    try {
      if (!EnumProcesses(processIds, cb, cbNeeded).value) {
        return false;
      }

      final count = cbNeeded.value ~/ sizeOf<Uint32>();
      final targetName = processName.toLowerCase();

      for (var i = 0; i < count; i++) {
        final processHandle = OpenProcess(
          PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE,
          false,
          processIds[i],
        ).value;

        if (processHandle != NULL) {
          final exeName = wsalloc(MAX_PATH);
          try {
            if (GetModuleBaseName(processHandle, null, PWSTR(exeName), MAX_PATH)
                    .value >
                0) {
              final currentName = exeName.toDartString().toLowerCase();
              if (currentName == targetName) {
                final terminated = TerminateProcess(processHandle, 0).value;
                return terminated;
              }
            }
          } finally {
            free(exeName);
            CloseHandle(processHandle);
          }
        }
      }
    } catch (_) {
      // ignore
    } finally {
      free(processIds);
      free(cbNeeded);
    }

    return false;
  }
}

Pointer<Utf16> wsalloc(int size) => calloc<Uint16>(size).cast<Utf16>();

class ProcessException implements Exception {
  final String message;
  ProcessException(this.message);

  @override
  String toString() => message;
}
