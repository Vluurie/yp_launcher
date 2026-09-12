import 'dart:async';
import 'dart:ui';

import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';

class WindowStateService with WindowListener {
  WindowStateService._();

  static final instance = WindowStateService._();

  static const defaultSize = Size(1300, 750);
  static const minimumSize = Size(600, 420);

  static const _keyWidth = 'window_width';
  static const _keyHeight = 'window_height';
  static const _keyX = 'window_x';
  static const _keyY = 'window_y';
  static const _keyMaximized = 'window_maximized';
  static const _keyFullScreen = 'window_fullscreen';

  static const _saveDelay = Duration(milliseconds: 400);

  Timer? _saveTimer;
  bool _restoring = false;

  static Future<Size> savedSize() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final width = prefs.getDouble(_keyWidth);
      final height = prefs.getDouble(_keyHeight);
      if (width == null || height == null) return defaultSize;
      return _sane(Size(width, height));
    } catch (_) {
      return defaultSize;
    }
  }

  static Future<bool> savedMaximized() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return prefs.getBool(_keyMaximized) ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> savedFullScreen() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return prefs.getBool(_keyFullScreen) ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<Offset?> savedPosition() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final x = prefs.getDouble(_keyX);
      final y = prefs.getDouble(_keyY);
      if (x == null || y == null) return null;
      return Offset(x, y);
    } catch (_) {
      return null;
    }
  }

  static Size _sane(Size size) {
    final width = size.width.clamp(minimumSize.width, 10000.0);
    final height = size.height.clamp(minimumSize.height, 10000.0);
    return Size(width, height);
  }

  Future<void> restore() async {
    _restoring = true;
    try {
      final position = await savedPosition();
      if (position != null && _looksReachable(position)) {
        await windowManager.setPosition(position);
      } else {
        await windowManager.center();
      }

      if (await savedFullScreen()) {
        await windowManager.setFullScreen(true);
      } else if (await savedMaximized()) {
        await windowManager.maximize();
      }
    } catch (_) {
    } finally {
      _restoring = false;
    }
  }

  static bool _looksReachable(Offset position) {
    if (position.dy < -64) return false;
    return position.dx > -20000 && position.dx < 20000 && position.dy < 20000;
  }

  void start() {
    windowManager.addListener(this);
  }

  void stop() {
    _saveTimer?.cancel();
    windowManager.removeListener(this);
  }

  @override
  void onWindowResized() => _scheduleSave();

  @override
  void onWindowMoved() => _scheduleSave();

  @override
  void onWindowMaximize() => _scheduleSave();

  @override
  void onWindowUnmaximize() => _scheduleSave();

  @override
  void onWindowEnterFullScreen() => _scheduleSave();

  @override
  void onWindowLeaveFullScreen() => _scheduleSave();

  Future<void> saveNow() {
    _saveTimer?.cancel();
    return _save();
  }

  void _scheduleSave() {
    if (_restoring) return;
    _saveTimer?.cancel();
    _saveTimer = Timer(_saveDelay, () => unawaited(_save()));
  }

  Future<void> _save() async {
    try {
      final fullScreen = await windowManager.isFullScreen();
      final maximized = await windowManager.isMaximized();
      final prefs = await SharedPreferences.getInstance();
      await prefs.setBool(_keyFullScreen, fullScreen);
      await prefs.setBool(_keyMaximized, maximized);

      if (fullScreen || maximized) return;

      final size = await windowManager.getSize();
      final position = await windowManager.getPosition();
      await prefs.setDouble(_keyWidth, size.width);
      await prefs.setDouble(_keyHeight, size.height);
      await prefs.setDouble(_keyX, position.dx);
      await prefs.setDouble(_keyY, position.dy);
    } catch (_) {}
  }

  static Future<void> clear() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.remove(_keyWidth);
      await prefs.remove(_keyHeight);
      await prefs.remove(_keyX);
      await prefs.remove(_keyY);
      await prefs.remove(_keyMaximized);
      await prefs.remove(_keyFullScreen);
    } catch (_) {}
  }
}
