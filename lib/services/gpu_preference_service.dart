import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:win32/win32.dart';

class GpuPreferenceService {
  GpuPreferenceService._();

  static const _subKeyPath = r'Software\Microsoft\DirectX\UserGpuPreferences';
  static const _highPerformance = 'GpuPreference=2;';

  static const linuxDedicatedGpuEnv = <String, String>{
    'DRI_PRIME': '1',
    '__NV_PRIME_RENDER_OFFLOAD': '1',
    '__GLX_VENDOR_LIBRARY_NAME': 'nvidia',
    '__VK_LAYER_NV_optimus': 'NVIDIA_only',
  };

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

  static Map<String, String>? mergedLaunchEnv(
    Map<String, String>? baseEnv, {
    required bool enabled,
  }) {
    if (!enabled || !Platform.isLinux) return baseEnv;
    return {
      ...?baseEnv,
      ...linuxDedicatedGpuEnv,
    };
  }
}
