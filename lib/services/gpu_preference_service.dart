import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:win32/win32.dart';
import 'package:yp_launcher/constants/app_strings.dart';

class GpuProbe {
  final bool nvidiaDriverLoaded;
  final int renderNodes;

  const GpuProbe({required this.nvidiaDriverLoaded, required this.renderNodes});

  factory GpuProbe.detect() {
    if (!Platform.isLinux) {
      return const GpuProbe(nvidiaDriverLoaded: false, renderNodes: 0);
    }
    var nodes = 0;
    try {
      nodes = Directory('/dev/dri')
          .listSync(followLinks: false)
          .where((e) => e.path.split('/').last.startsWith('renderD'))
          .length;
    } catch (_) {}
    return GpuProbe(
      nvidiaDriverLoaded: File('/proc/driver/nvidia/version').existsSync(),
      renderNodes: nodes,
    );
  }

  bool get isHybrid => renderNodes > 1;
}

class GpuPreferenceService {
  GpuPreferenceService._();

  static const _subKeyPath = r'Software\Microsoft\DirectX\UserGpuPreferences';
  static const _highPerformance = 'GpuPreference=2;';

  static const nvidiaOffloadEnv = <String, String>{
    '__NV_PRIME_RENDER_OFFLOAD': '1',
    '__GLX_VENDOR_LIBRARY_NAME': 'nvidia',
    '__VK_LAYER_NV_optimus': 'NVIDIA_only',
  };

  static const mesaPrimeEnv = <String, String>{'DRI_PRIME': '1'};

  static Future<bool> preferDedicatedGpu() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return prefs.getBool(AppStrings.prefKeyPreferDedicatedGpu) ?? true;
    } catch (_) {
      return true;
    }
  }

  static Map<String, String> linuxGpuEnv(GpuProbe probe) {
    if (!probe.isHybrid) return const {};
    return probe.nvidiaDriverLoaded ? nvidiaOffloadEnv : mesaPrimeEnv;
  }

  static Map<String, String>? mergedLaunchEnv(
    Map<String, String>? baseEnv, {
    required bool enabled,
    GpuProbe? probe,
  }) {
    if (!enabled || !Platform.isLinux) return baseEnv;
    final extra = linuxGpuEnv(probe ?? GpuProbe.detect());
    if (extra.isEmpty) return baseEnv;
    return {...?baseEnv, ...extra};
  }

  static void apply(String namsExePath, {required bool enabled}) {
    if (!Platform.isWindows) return;
    final phkResult = calloc<Pointer>();
    final subKey = PCWSTR(_subKeyPath.toNativeUtf16());
    final valueName = PCWSTR(namsExePath.toNativeUtf16());
    try {
      final created = RegCreateKeyEx(
        HKEY_CURRENT_USER,
        subKey,
        null,
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        null,
        phkResult,
        null,
      );
      if (created != ERROR_SUCCESS) return;
      final key = HKEY(phkResult.value);

      if (enabled) {
        final data = _highPerformance.toNativeUtf16();
        try {
          RegSetValueEx(
            key,
            valueName,
            REG_SZ,
            data.cast<Uint8>(),
            (_highPerformance.length + 1) * 2,
          );
        } finally {
          free(data);
        }
      } else {
        RegDeleteValue(key, valueName);
      }
      RegCloseKey(key);
    } catch (_) {
    } finally {
      free(valueName);
      free(subKey);
      free(phkResult);
    }
  }
}
