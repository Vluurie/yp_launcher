import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/services/window_state_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('savedSize', () {
    test('falls back to the default when nothing is stored', () async {
      expect(await WindowStateService.savedSize(),
          WindowStateService.defaultSize);
    });

    test('returns what was stored', () async {
      SharedPreferences.setMockInitialValues({
        'window_width': 1600.0,
        'window_height': 900.0,
      });
      expect(await WindowStateService.savedSize(), const Size(1600, 900));
    });

    test('clamps a stored size below the minimum', () async {
      SharedPreferences.setMockInitialValues({
        'window_width': 120.0,
        'window_height': 80.0,
      });
      expect(
        await WindowStateService.savedSize(),
        WindowStateService.minimumSize,
      );
    });

    test('clamps an absurd size', () async {
      SharedPreferences.setMockInitialValues({
        'window_width': 999999.0,
        'window_height': 999999.0,
      });
      final size = await WindowStateService.savedSize();
      expect(size.width, lessThanOrEqualTo(10000));
      expect(size.height, lessThanOrEqualTo(10000));
    });

    test('ignores a half-written entry', () async {
      SharedPreferences.setMockInitialValues({'window_width': 1600.0});
      expect(await WindowStateService.savedSize(),
          WindowStateService.defaultSize);
    });
  });

  group('savedPosition', () {
    test('is null when nothing is stored', () async {
      expect(await WindowStateService.savedPosition(), isNull);
    });

    test('returns the stored offset', () async {
      SharedPreferences.setMockInitialValues({
        'window_x': 240.0,
        'window_y': 120.0,
      });
      expect(await WindowStateService.savedPosition(), const Offset(240, 120));
    });
  });

  group('savedMaximized', () {
    test('defaults to false', () async {
      expect(await WindowStateService.savedMaximized(), isFalse);
    });

    test('reads the stored flag', () async {
      SharedPreferences.setMockInitialValues({'window_maximized': true});
      expect(await WindowStateService.savedMaximized(), isTrue);
    });
  });

  group('savedFullScreen', () {
    test('defaults to false', () async {
      expect(await WindowStateService.savedFullScreen(), isFalse);
    });

    test('reads the stored flag', () async {
      SharedPreferences.setMockInitialValues({'window_fullscreen': true});
      expect(await WindowStateService.savedFullScreen(), isTrue);
    });

    test('is independent of the maximized flag', () async {
      SharedPreferences.setMockInitialValues({
        'window_fullscreen': true,
        'window_maximized': false,
      });
      expect(await WindowStateService.savedFullScreen(), isTrue);
      expect(await WindowStateService.savedMaximized(), isFalse);
    });
  });

  test('clear removes every stored key', () async {
    SharedPreferences.setMockInitialValues({
      'window_width': 1600.0,
      'window_height': 900.0,
      'window_x': 10.0,
      'window_y': 20.0,
      'window_maximized': true,
      'window_fullscreen': true,
    });

    await WindowStateService.clear();

    expect(await WindowStateService.savedSize(),
        WindowStateService.defaultSize);
    expect(await WindowStateService.savedPosition(), isNull);
    expect(await WindowStateService.savedMaximized(), isFalse);
    expect(await WindowStateService.savedFullScreen(), isFalse);
  });
}
