import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:win32/win32.dart';

Pointer<Utf16> _wsalloc(int size) => calloc<Uint16>(size).cast<Utf16>();

bool isWin32ProcessRunning(String processName) =>
    _withProcesses(processName, terminate: false);

bool terminateWin32ProcessByName(String processName) =>
    _withProcesses(processName, terminate: true);

bool _withProcesses(String processName, {required bool terminate}) {
  if (!Platform.isWindows) return false;

  final processIds = calloc<Uint32>(1024);
  final cb = sizeOf<Uint32>() * 1024;
  final cbNeeded = calloc<Uint32>();

  try {
    if (!EnumProcesses(processIds, cb, cbNeeded).value) return false;

    final count = cbNeeded.value ~/ sizeOf<Uint32>();
    final target = processName.toLowerCase();

    for (var i = 0; i < count; i++) {
      final access = terminate
          ? PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_TERMINATE
          : PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
      final handle = OpenProcess(access, false, processIds[i]).value;
      if (handle == NULL) continue;

      final exeName = _wsalloc(MAX_PATH);
      try {
        final length =
            GetModuleBaseName(handle, null, PWSTR(exeName), MAX_PATH).value;
        if (length <= 0) continue;

        final current = exeName.toDartString().toLowerCase();
        if (terminate) {
          if (current == target) return TerminateProcess(handle, 0).value;
        } else if (current.startsWith(target)) {
          return true;
        }
      } finally {
        free(exeName);
        CloseHandle(handle);
      }
    }
  } catch (_) {
  } finally {
    free(processIds);
    free(cbNeeded);
  }

  return false;
}
