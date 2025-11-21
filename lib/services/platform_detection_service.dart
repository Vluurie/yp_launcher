import 'dart:io';
import 'package:yp_launcher/services/logging_service.dart';

class PlatformDetectionService {
  static bool? _isWineCache;

  /// Check if running under Wine/CrossOver on macOS or Linux
  static bool get isWine {
    if (_isWineCache != null) return _isWineCache!;

    if (Platform.isWindows) {
      _isWineCache = false;
      return false;
    }

    // Check for Wine registry path
    final home = Platform.environment['HOME'];
    if (home != null) {
      // CrossOver stores Wine prefix in standard locations
      final crossoverDir = Directory('$home/.cxoffice');
      final wineDir = Directory('$home/.wine');

      if (crossoverDir.existsSync() || wineDir.existsSync()) {
        _isWineCache = true;
        LoggingService.log('Detected Wine/CrossOver environment');
        return true;
      }
    }

    // Check WINEPREFIX environment variable
    if (Platform.environment.containsKey('WINEPREFIX')) {
      _isWineCache = true;
      LoggingService.log('Detected Wine via WINEPREFIX: ${Platform.environment['WINEPREFIX']}');
      return true;
    }

    _isWineCache = false;
    return false;
  }

  /// Get Wine prefix path (directory containing drive_c)
  static String? getWinePrefix() {
    // Check WINEPREFIX environment variable first
    final winePrefix = Platform.environment['WINEPREFIX'];
    if (winePrefix != null) {
      LoggingService.log('Using WINEPREFIX: $winePrefix');
      return winePrefix;
    }

    // Check CrossOver default locations
    final home = Platform.environment['HOME'];
    if (home != null) {
      // CrossOver bottles are typically in ~/Library/Application Support/CrossOver/Bottles/
      if (Platform.isMacOS) {
        final crossoverBottles = Directory('$home/Library/Application Support/CrossOver/Bottles');
        if (crossoverBottles.existsSync()) {
          // Try to find a bottle (use first available)
          final bottles = crossoverBottles.listSync();
          if (bottles.isNotEmpty) {
            final bottlePath = bottles.first.path;
            LoggingService.log('Using CrossOver bottle: $bottlePath');
            return bottlePath;
          }
        }
      }

      // Standard Wine prefix
      final wineDir = Directory('$home/.wine');
      if (wineDir.existsSync()) {
        LoggingService.log('Using Wine prefix: ${wineDir.path}');
        return wineDir.path;
      }
    }

    return null;
  }

  /// Convert Unix path to Windows path for Wine (e.g., /path/to/file -> Z:/path/to/file)
  static String unixToWinePath(String unixPath) {
    if (Platform.isWindows) return unixPath;

    final normalized = unixPath.replaceAll('\\', '/');
    if (!normalized.startsWith('/')) {
      return normalized;
    }
    return 'Z:$normalized';
  }

  /// Should we use Windows-style operations (native Windows or Wine)
  static bool get useWindowsOperations {
    return Platform.isWindows || isWine;
  }
}
