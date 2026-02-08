import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/platform_detection_service.dart';
import 'package:win32/win32.dart' if (dart.library.html) '';

class ProcessService {
  static Future<bool> startNierAutomata({
    required String installDirectory,
    required VoidCallback onProcessStopped,
  }) async {
    try {
      final isSetup = await LauncherSetupService.isLauncherSetup();
      if (!isSetup) {
        await LauncherSetupService.setupLauncher();
      }

      final launcherPaths = await LauncherSetupService.getLauncherPaths();

      final nierExePath = path.join(installDirectory, AppStrings.gameExeName);
      if (!await File(nierExePath).exists()) {
        throw ProcessException(
          AppStrings.errorExeNotFound(installDirectory),
        );
      }

      final List<String> arguments = _buildLaunchArguments(
        modloaderDllPath: launcherPaths['modloaderDll']!,
        yorhaDllPath: launcherPaths['yorhaDll']!,
      );

      final process = await Process.start(
        launcherPaths['launcherExe']!,
        arguments,
        workingDirectory: installDirectory,
        mode: ProcessStartMode.detachedWithStdio,
      );

      process.stdout.transform(const SystemEncoding().decoder).listen((_) {});
      process.stderr.transform(const SystemEncoding().decoder).listen((_) {});

      final started = await _waitForProcessStart(
        AppStrings.gameExeName,
        timeout: const Duration(seconds: 60),
      );

      if (started) {
        unawaited(_monitorProcess(AppStrings.gameExeName, onProcessStopped));
      }

      return started;
    } catch (_) {
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
      AppStrings.argModloaderDll,
      formatPath(modloaderDllPath),
      AppStrings.argModDll,
      formatPath(yorhaDllPath),
    ];
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
          final result = Process.runSync('pkill', ['-f', AppStrings.gameExeName]);
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
    if (Platform.isWindows) {
      return _isProcessRunning(AppStrings.gameExeName);
    } else {
      try {
        final result = Process.runSync('pgrep', ['-f', AppStrings.gameExeName]);
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
