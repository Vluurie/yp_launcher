import 'dart:io';

import 'package:yp_launcher/services/platform_detection_service.dart';

/// Simulated platform identity for UI gating only.
///
/// Native services (process polling, shortcut creation, WMIC, COM)
/// must keep using raw `Platform.is*` because they actually hit the host OS.
/// This gate exists so we can preview the Linux/macOS UI on a Windows dev
/// machine (or vice versa) without breaking real platform calls.
enum SimulatedOs { windows, linux, macos }

class PlatformGate {
  PlatformGate._();

  /// Set to a non-null value to force UI gates to behave as that OS.
  /// Leave null in production builds.
  static SimulatedOs? overrideAs;

  static SimulatedOs get _current {
    if (overrideAs != null) return overrideAs!;
    if (Platform.isWindows) return SimulatedOs.windows;
    if (Platform.isMacOS) return SimulatedOs.macos;
    return SimulatedOs.linux;
  }

  static bool get isWindows => _current == SimulatedOs.windows;
  static bool get isLinux => _current == SimulatedOs.linux;
  static bool get isMacOS => _current == SimulatedOs.macos;

  /// True when the simulated OS matches the host. Use this as a guard
  /// before doing anything native (process polling, shortcuts, COM).
  static bool get isHostMatching {
    if (Platform.isWindows && isWindows) return true;
    if (Platform.isLinux && isLinux) return true;
    if (Platform.isMacOS && isMacOS) return true;
    return false;
  }

  /// The game and the launcher's process tooling are Windows-native.
  /// Launching is supported on real Windows, and *experimentally* under Wine
  /// on Linux/macOS. Native Linux without Wine cannot run the .exe.
  static bool get canLaunchGame {
    if (isWindows) return true;
    return PlatformDetectionService.isWine;
  }

  /// True when launching works but is not fully validated on this host.
  static bool get isWineExperimental {
    return !isWindows && PlatformDetectionService.isWine;
  }
}
