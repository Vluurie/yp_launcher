import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as path;

class LoggingService {
  static File? _logFile;
  static bool _initialized = false;

  /// Initialize the logging service
  static Future<void> initialize() async {
    if (_initialized) return;

    try {
      final exePath = Platform.resolvedExecutable;
      final exeDir = path.dirname(exePath);
      final logPath = path.join(exeDir, 'yp_launcher.log');

      _logFile = File(logPath);

      // Create or clear the log file
      if (await _logFile!.exists()) {
        // Rotate old log if it exists
        final oldLogPath = path.join(exeDir, 'yp_launcher.old.log');
        final oldLogFile = File(oldLogPath);
        if (await oldLogFile.exists()) {
          await oldLogFile.delete();
        }
        await _logFile!.rename(oldLogPath);
        _logFile = File(logPath);
      }

      await _logFile!.create();

      _initialized = true;

      await log('=== YorHa Protocol Launcher Started ===');
      await log('Log file: $logPath');
      await log('Executable: $exePath');
      await log('Platform: ${Platform.operatingSystem}');
      await log('Working directory: ${Directory.current.path}');
    } catch (e) {
      debugPrint('Failed to initialize logging service: $e');
    }
  }

  /// Log a message to both debug console and file
  static Future<void> log(String message) async {
    final timestamp = DateTime.now().toIso8601String();
    final logMessage = '[$timestamp] $message';

    // Always print to debug console
    debugPrint(logMessage);

    // Write to file if initialized
    if (_initialized && _logFile != null) {
      try {
        await _logFile!.writeAsString(
          '$logMessage\n',
          mode: FileMode.append,
          flush: true,
        );
      } catch (e) {
        debugPrint('Failed to write to log file: $e');
      }
    }
  }

  /// Log an error with stack trace
  static Future<void> logError(String message, [Object? error, StackTrace? stackTrace]) async {
    await log('ERROR: $message');
    if (error != null) {
      await log('Error details: $error');
    }
    if (stackTrace != null) {
      await log('Stack trace:\n$stackTrace');
    }
  }

  /// Log a section separator
  static Future<void> logSection(String title) async {
    await log('');
    await log('=== $title ===');
  }
}
