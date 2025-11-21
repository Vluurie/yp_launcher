import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/settings_service.dart';
import 'package:yp_launcher/services/platform_detection_service.dart';
import 'package:yp_launcher/services/logging_service.dart';

import 'package:win32/win32.dart' if (dart.library.html) '';

class ProcessService {
  static Future<bool> startNierAutomata({
    required String installDirectory,
    required VoidCallback onProcessStopped,
    SettingsService? settings,
  }) async {
    try {
      await LoggingService.logSection('Starting NieR:Automata');
      await LoggingService.log('Install directory: $installDirectory');

      final isSetup = await LauncherSetupService.isLauncherSetup();
      if (!isSetup && (settings == null || !settings.hasOverrides)) {
        await LoggingService.log('Launcher not set up, copying files...');
        await LauncherSetupService.setupLauncher();
        await LoggingService.log('Launcher setup complete');
      } else {
        await LoggingService.log('Launcher already set up');
      }

      final launcherPaths = settings != null
          ? await LauncherSetupService.getLauncherPathsWithOverrides(settings)
          : await LauncherSetupService.getLauncherPaths();

      if (settings != null && settings.hasOverrides) {
        await LoggingService.log('Using custom file overrides');
      }

      final nierExePath = path.join(installDirectory, 'NierAutomata.exe');
      if (!await File(nierExePath).exists()) {
        await LoggingService.logError('NierAutomata.exe not found in $installDirectory');
        throw ProcessException(
          'NierAutomata.exe not found in $installDirectory',
        );
      }

      final List<String> arguments = _buildLaunchArguments(
        modloaderDllPath: launcherPaths['modloaderDll']!,
        yorhaDllPath: launcherPaths['yorhaDll']!,
      );

      await LoggingService.log('Launcher executable: ${launcherPaths['launcherExe']}');
      await LoggingService.log('Arguments: $arguments');
      await LoggingService.log('Working directory: $installDirectory');

      final process = await Process.start(
        launcherPaths['launcherExe']!,
        arguments,
        workingDirectory: installDirectory,
      );

      await LoggingService.log('Process started, PID: ${process.pid}');

      process.stdout.transform(const SystemEncoding().decoder).listen((data) {
        LoggingService.log('Launcher stdout: $data');
      });

      process.stderr.transform(const SystemEncoding().decoder).listen((data) {
        LoggingService.log('Launcher stderr: $data');
      });

      await LoggingService.log('Waiting for NieR:Automata process to start...');
      final started = await _waitForProcessStart(
        'NierAutomata.exe',
        timeout: const Duration(seconds: 60),
      );

      if (started) {
        await LoggingService.log('NieR:Automata process started successfully');
        unawaited(_monitorProcess('NierAutomata.exe', onProcessStopped));
      } else {
        await LoggingService.log('NieR:Automata process did not start within timeout period');
      }

      return started;
    } catch (e, stackTrace) {
      await LoggingService.logError('Error starting NieR:Automata', e, stackTrace);
      return false;
    }
  }

  static List<String> _buildLaunchArguments({
    required String modloaderDllPath,
    required String yorhaDllPath,
  }) {
    String formatPath(String filePath) {
      if (Platform.isWindows) {
        return filePath.replaceAll('/', '\\');
      } else if (PlatformDetectionService.isWine) {
        // When running under Wine, convert Unix paths to Windows-style paths
        return PlatformDetectionService.unixToWinePath(filePath);
      } else {
        final normalizedPath = filePath.replaceAll('\\', '/');
        if (!normalizedPath.startsWith('/')) {
          return normalizedPath;
        }
        return 'Z:$normalizedPath';
      }
    }

    return [
      '--modloader-dll',
      formatPath(modloaderDllPath),
      '--mod-dll',
      formatPath(yorhaDllPath),
    ];
  }

  static bool terminateNierAutomata() {
    LoggingService.log('Attempting to terminate NieR:Automata...');
    if (Platform.isWindows) {
      final result = _terminateProcessByName('NierAutomata.exe');
      LoggingService.log('Termination result (Windows): $result');
      return result;
    } else if (PlatformDetectionService.isWine) {
      // Use wineserver to kill the process
      try {
        final result = Process.runSync('wineserver', ['-k']);
        LoggingService.log('Killed Wine processes via wineserver: ${result.exitCode}');
        return true;
      } catch (e) {
        LoggingService.log('Error terminating via wineserver: $e');
        // Fallback to pkill
        try {
          final result = Process.runSync('pkill', ['-f', 'NierAutomata.exe']);
          LoggingService.log('pkill result: ${result.exitCode}');
          return result.exitCode == 0;
        } catch (e2) {
          LoggingService.logError('Error terminating NieR:Automata', e2);
          return false;
        }
      }
    } else {
      try {
        final result = Process.runSync('pkill', ['-f', 'NierAutomata.exe']);
        LoggingService.log('pkill result: ${result.exitCode}');
        return result.exitCode == 0;
      } catch (e) {
        LoggingService.logError('Error terminating NieR:Automata', e);
        return false;
      }
    }
  }

  static bool isNierAutomataRunning() {
    if (Platform.isWindows) {
      return _isProcessRunning('NierAutomata.exe');
    } else if (PlatformDetectionService.isWine) {
      // Check for Wine processes
      try {
        final result = Process.runSync('pgrep', ['-f', 'NierAutomata.exe']);
        return result.exitCode == 0;
      } catch (e) {
        return false;
      }
    } else {
      try {
        final result = Process.runSync('pgrep', ['-f', 'NierAutomata.exe']);
        return result.exitCode == 0;
      } catch (e) {
        return false;
      }
    }
  }

  static Future<bool> _waitForProcessStart(
    String processName, {
    required Duration timeout,
  }) async {
    final completer = Completer<bool>();
    Timer? checkTimer;
    Timer? timeoutTimer;

    checkTimer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (isNierAutomataRunning()) {
        timer.cancel();
        timeoutTimer?.cancel();
        if (!completer.isCompleted) {
          completer.complete(true);
        }
      }
    });

    timeoutTimer = Timer(timeout, () {
      checkTimer?.cancel();
      if (!completer.isCompleted) {
        completer.complete(false);
      }
    });

    return completer.future;
  }

  static Future<void> _monitorProcess(
    String processName,
    VoidCallback onStopped,
  ) async {
    Timer.periodic(const Duration(seconds: 1), (timer) async {
      if (!isNierAutomataRunning()) {
        await Future.delayed(const Duration(seconds: 2));

        if (!isNierAutomataRunning()) {
          timer.cancel();
          await LoggingService.log('NieR:Automata process stopped');
          onStopped();
        }
      }
    });
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
          PROCESS_QUERY_INFORMATION |
              PROCESS_VM_READ,
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
    } catch (e) {
      LoggingService.logError('Error checking process', e);
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
          PROCESS_QUERY_INFORMATION |
              PROCESS_VM_READ |
              PROCESS_TERMINATE,
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
                LoggingService.log('Terminated process $currentName: $terminated');
                return terminated;
              }
            }
          } finally {
            free(exeName);
            CloseHandle(processHandle);
          }
        }
      }
    } catch (e) {
      LoggingService.logError('Error terminating process', e);
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
