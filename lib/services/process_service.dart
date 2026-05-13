import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/launch_failure.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/platform_detection_service.dart';
import 'package:yp_launcher/services/plugins_service.dart';
import 'package:yp_launcher/services/steam_service.dart';
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

      // Best-effort: try to start Steam if it isn't running. If this fails,
      // we still attempt the game launch — NAMS will surface its own Steam
      // error if the game can't initialise its Steam connection.
      await SteamService.ensureRunning();

      await LauncherSetupService.invalidateDecryptCacheIfChanged(nierExePath);
      await ModsService.syncDlcSlots(installDirectory);

      final pluginPaths = await PluginsService.enabledPaths();

      final List<String> arguments = _buildLaunchArguments(
        modloaderDllPath: launcherPaths['modloaderDll']!,
        yorhaDllPath: launcherPaths['yorhaDll']!,
        extraModDllPaths: pluginPaths,
      );

      final process = await Process.start(
        launcherPaths['launcherExe']!,
        arguments,
        workingDirectory: installDirectory,
        mode: ProcessStartMode.normal,
      );

      final stdoutBuf = StringBuffer();
      final stderrBuf = StringBuffer();
      process.stdout.transform(const SystemEncoding().decoder).listen(stdoutBuf.write);
      process.stderr.transform(const SystemEncoding().decoder).listen(stderrBuf.write);

      // Race: did the game window appear, OR did launch_nier exit early?
      final exitFuture = process.exitCode;
      final startedFuture = _waitForProcessStart(
        AppStrings.gameExeName,
        timeout: const Duration(seconds: 60),
      );

      final result = await Future.any([
        exitFuture.then((code) => _RaceResult.exited(code)),
        startedFuture.then((s) => s
            ? const _RaceResult.started()
            : const _RaceResult.timeout()),
      ]);

      if (result.kind == _RaceKind.started) {
        unawaited(_monitorProcess(AppStrings.gameExeName, onProcessStopped));
        return const LaunchOutcome.started();
      }

      // launch_nier exited early or we timed out. Wait for any remaining
      // stdio drain so the captured output is complete.
      try {
        await exitFuture.timeout(const Duration(milliseconds: 500));
      } catch (_) {}

      final combined = '${stdoutBuf.toString()}${stderrBuf.toString()}';
      final logPath = await _writeCapturedLog(combined, exitCode: result.exitCode);

      final parsed = LaunchFailureParser.parse(combined, capturedLogPath: logPath);
      if (parsed != null) {
        return LaunchOutcome.failed(parsed);
      }

      // No structured error block. Synthesise a fallback failure with whatever
      // info we have.
      final headline = result.kind == _RaceKind.timeout
          ? 'Game did not start within 60 seconds'
          : 'launch_nier exited with code ${result.exitCode ?? "?"}';
      return LaunchOutcome.failed(LaunchFailure(
        code: result.exitCode,
        headline: headline,
        rawOutput: combined.isEmpty
            ? '(launch_nier produced no output)'
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
      final out = path.join(dir.path, 'launch_nier_$stamp.log');
      final header = StringBuffer()
        ..writeln('launch_nier captured output')
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

  /// Builds the exact `cd <gameDir> && <launchExe> <args>` command we'd run.
  /// Returns null if the launcher binaries haven't been extracted yet.
  /// Safe to call without launching — used by the failure dialog and
  /// "Copy launch command" affordances to give users something they can paste
  /// into a terminal for debugging.
  static Future<String?> buildLaunchCommandPreview({
    required String installDirectory,
  }) async {
    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final pluginPaths = await PluginsService.enabledPaths();
      final args = _buildLaunchArguments(
        modloaderDllPath: paths['modloaderDll']!,
        yorhaDllPath: paths['yorhaDll']!,
        extraModDllPaths: pluginPaths,
      );
      final exe = paths['launcherExe']!;
      final quotedArgs = args.map(_shellQuote).join(' ');
      if (Platform.isWindows) {
        return 'cd /d ${_shellQuote(installDirectory)}\r\n'
            '${_shellQuote(exe)} $quotedArgs';
      }
      return 'cd ${_shellQuote(installDirectory)} && '
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

  static List<String> _buildLaunchArguments({
    required String modloaderDllPath,
    required String yorhaDllPath,
    List<String> extraModDllPaths = const [],
  }) {
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

    final args = <String>[
      AppStrings.argModloaderDll,
      formatPath(modloaderDllPath),
      AppStrings.argModDll,
      formatPath(yorhaDllPath),
    ];
    for (final extra in extraModDllPaths) {
      args.add(AppStrings.argModDll);
      args.add(formatPath(extra));
    }
    return args;
  }

  static bool terminateNierAutomata() {
    if (Platform.isWindows) {
      return _terminateProcessByName(AppStrings.gameExeName);
    } else if (PlatformDetectionService.isWine) {
      try {
        Process.runSync('wineserver', ['-k']);
        return true;
      } catch (_) {
        try {
          final result = Process.runSync('pkill', [
            '-f',
            AppStrings.gameExeName,
          ]);
          return result.exitCode == 0;
        } catch (_) {
          return false;
        }
      }
    } else {
      try {
        final result = Process.runSync('pkill', ['-f', AppStrings.gameExeName]);
        return result.exitCode == 0;
      } catch (_) {
        return false;
      }
    }
  }

  static bool isNierAutomataRunning() {
    return _isProcessRunning(AppStrings.gameExeName);
  }

  static Future<bool> isNierAutomataRunningAsync() async {
    if (Platform.isWindows) {
      try {
        final result = await Process.run('tasklist', [
          '/FI',
          'IMAGENAME eq ${AppStrings.gameExeName}',
          '/NH',
        ]);
        return result.stdout.toString().toLowerCase().contains(
          AppStrings.gameExeName.toLowerCase(),
        );
      } catch (_) {
        return _isProcessRunning(AppStrings.gameExeName);
      }
    } else {
      try {
        final result = await Process.run('pgrep', [
          '-f',
          AppStrings.gameExeName,
        ]);
        return result.exitCode == 0;
      } catch (_) {
        return false;
      }
    }
  }

  static Future<bool> _waitForProcessStart(
    String processName, {
    required Duration timeout,
  }) async {
    final deadline = DateTime.now().add(timeout);
    while (DateTime.now().isBefore(deadline)) {
      if (await isNierAutomataRunningAsync()) return true;
      await Future.delayed(const Duration(seconds: 1));
    }
    return false;
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
      if (EnumProcesses(processIds, cb, cbNeeded) == 0) {
        return false;
      }

      final count = cbNeeded.value ~/ sizeOf<Uint32>();
      final targetName = processName.toLowerCase();

      for (var i = 0; i < count; i++) {
        final processHandle = OpenProcess(
          PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
          FALSE,
          processIds[i],
        );

        if (processHandle != NULL) {
          final exeName = wsalloc(MAX_PATH);
          try {
            if (GetModuleBaseName(processHandle, NULL, exeName, MAX_PATH) > 0) {
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
      if (EnumProcesses(processIds, cb, cbNeeded) == 0) {
        return false;
      }

      final count = cbNeeded.value ~/ sizeOf<Uint32>();
      final targetName = processName.toLowerCase();

      for (var i = 0; i < count; i++) {
        final processHandle = OpenProcess(
          PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE,
          FALSE,
          processIds[i],
        );

        if (processHandle != NULL) {
          final exeName = wsalloc(MAX_PATH);
          try {
            if (GetModuleBaseName(processHandle, NULL, exeName, MAX_PATH) > 0) {
              final currentName = exeName.toDartString().toLowerCase();
              if (currentName == targetName) {
                final terminated = TerminateProcess(processHandle, 0) != 0;
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

enum _RaceKind { started, exited, timeout }

class _RaceResult {
  final _RaceKind kind;
  final int? exitCode;
  const _RaceResult.started()
      : kind = _RaceKind.started,
        exitCode = null;
  const _RaceResult.timeout()
      : kind = _RaceKind.timeout,
        exitCode = null;
  const _RaceResult.exited(this.exitCode) : kind = _RaceKind.exited;
}
