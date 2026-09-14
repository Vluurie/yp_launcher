import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:win32/win32.dart';

const _th32csSnapProcess = 0x00000002;
const _exeNameLength = 260;

final class _ProcessEntry32W extends Struct {
  @Uint32()
  external int dwSize;
  @Uint32()
  external int cntUsage;
  @Uint32()
  external int th32ProcessID;
  @IntPtr()
  external int th32DefaultHeapID;
  @Uint32()
  external int th32ModuleID;
  @Uint32()
  external int cntThreads;
  @Uint32()
  external int th32ParentProcessID;
  @Int32()
  external int pcPriClassBase;
  @Uint32()
  external int dwFlags;
  @Array(_exeNameLength)
  external Array<Uint16> szExeFile;
}

final _kernel32 = DynamicLibrary.open('kernel32.dll');

final _createToolhelp32Snapshot = _kernel32
    .lookupFunction<
      Pointer Function(Uint32, Uint32),
      Pointer Function(int, int)
    >('CreateToolhelp32Snapshot');

final _process32FirstW = _kernel32
    .lookupFunction<
      Int32 Function(Pointer, Pointer<_ProcessEntry32W>),
      int Function(Pointer, Pointer<_ProcessEntry32W>)
    >('Process32FirstW');

final _process32NextW = _kernel32
    .lookupFunction<
      Int32 Function(Pointer, Pointer<_ProcessEntry32W>),
      int Function(Pointer, Pointer<_ProcessEntry32W>)
    >('Process32NextW');

bool isWin32ProcessRunning(String processName) {
  if (!Platform.isWindows) return false;
  try {
    return _processIdsNamed(processName).isNotEmpty;
  } catch (_) {
    return false;
  }
}

bool terminateWin32ProcessByName(String processName) {
  if (!Platform.isWindows) return false;
  try {
    var terminated = false;
    for (final id in _processIdsNamed(processName)) {
      final handle = OpenProcess(PROCESS_TERMINATE, false, id).value;
      if (handle == NULL) continue;
      try {
        terminated = TerminateProcess(handle, 0).value || terminated;
      } finally {
        CloseHandle(handle);
      }
    }
    return terminated;
  } catch (_) {
    return false;
  }
}

List<int> _processIdsNamed(String processName) {
  final target = processName.toLowerCase();
  final snapshot = HANDLE(_createToolhelp32Snapshot(_th32csSnapProcess, 0));
  if (!snapshot.isValid) return const [];
  final entry = calloc<_ProcessEntry32W>();
  try {
    entry.ref.dwSize = sizeOf<_ProcessEntry32W>();
    final ids = <int>[];
    var found = _process32FirstW(snapshot, entry) != 0;
    while (found) {
      if (_exeName(entry.ref.szExeFile).toLowerCase() == target) {
        ids.add(entry.ref.th32ProcessID);
      }
      found = _process32NextW(snapshot, entry) != 0;
    }
    return ids;
  } finally {
    free(entry);
    CloseHandle(snapshot);
  }
}

String _exeName(Array<Uint16> chars) {
  final units = <int>[];
  for (var i = 0; i < _exeNameLength && chars[i] != 0; i++) {
    units.add(chars[i]);
  }
  return String.fromCharCodes(units);
}
