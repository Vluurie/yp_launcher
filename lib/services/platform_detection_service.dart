import 'dart:io';

class PlatformDetectionService {
  static bool? _isWineCache;

  static bool get isWine {
    if (_isWineCache != null) return _isWineCache!;

    if (Platform.isWindows) {
      _isWineCache = false;
      return false;
    }

    final home = Platform.environment['HOME'];
    if (home != null) {
      final crossoverDir = Directory('$home/.cxoffice');
      final wineDir = Directory('$home/.wine');

      if (crossoverDir.existsSync() || wineDir.existsSync()) {
        _isWineCache = true;
        return true;
      }
    }

    if (Platform.environment.containsKey('WINEPREFIX')) {
      _isWineCache = true;
      return true;
    }

    _isWineCache = false;
    return false;
  }

  static String? getWinePrefix() {
    final winePrefix = Platform.environment['WINEPREFIX'];
    if (winePrefix != null) {
      return winePrefix;
    }

    final home = Platform.environment['HOME'];
    if (home != null) {
      if (Platform.isMacOS) {
        final crossoverBottles = Directory('$home/Library/Application Support/CrossOver/Bottles');
        if (crossoverBottles.existsSync()) {
          final bottles = crossoverBottles.listSync();
          if (bottles.isNotEmpty) {
            return bottles.first.path;
          }
        }
      }

      final wineDir = Directory('$home/.wine');
      if (wineDir.existsSync()) {
        return wineDir.path;
      }
    }

    return null;
  }

  static String unixToWinePath(String unixPath) {
    if (Platform.isWindows) return unixPath;

    final normalized = unixPath.replaceAll('\\', '/');
    if (!normalized.startsWith('/')) {
      return normalized;
    }
    return 'Z:$normalized';
  }

  static bool get useWindowsOperations {
    return Platform.isWindows || isWine;
  }
}
