import 'dart:io';

import 'package:yp_launcher/services/platform/platform_adapter.dart';

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

  /// The game is Windows-native. Launching works on Windows, and through a
  /// compatibility layer elsewhere when one is installed.
  static bool get canLaunchGame => PlatformAdapter.current.canLaunchGame;

  /// True when launching works but is not fully validated on this host.
  static bool get isWineExperimental =>
      PlatformAdapter.current.isExperimental &&
      PlatformAdapter.current.canLaunchGame;
}
