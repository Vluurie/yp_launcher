import 'dart:io';
import 'dart:typed_data';
import 'package:yp_launcher/services/isolate_service.dart';

class PluginValidationResult {
  final bool valid;
  final String? failureReason;
  final List<String> exports;
  final String? internalDllName;

  const PluginValidationResult({
    required this.valid,
    this.failureReason,
    this.exports = const [],
    this.internalDllName,
  });
}

const _requiredExport = 'get_plugin_register';

const _knownIncompatibleSubstrings = <String, String>{
  'reshade': 'ReShade',
  'speedhack': 'SpeedHack',
  'special_k': 'Special K',
  'specialk': 'Special K',
  'far_': 'FAR',
  'nier_far': 'FAR',
  'wax_': 'WAX',
  'lodmod': 'Automata-LodMod',
};

PluginValidationResult _validateSync(String dllPath) {
  final file = File(dllPath);
  if (!file.existsSync()) {
    return const PluginValidationResult(
      valid: false,
      failureReason: 'file_not_found',
    );
  }
  Uint8List bytes;
  try {
    bytes = file.readAsBytesSync();
  } catch (_) {
    return const PluginValidationResult(
      valid: false,
      failureReason: 'read_failed',
    );
  }
  return _parseAndValidate(bytes);
}

class PluginValidator {
  PluginValidator._();

  static const requiredExport = _requiredExport;

  static Future<PluginValidationResult> validate(String dllPath) {
    return IsolateService.run(_validateSync, dllPath);
  }
}

PluginValidationResult _parseAndValidate(Uint8List b) {
    final view = ByteData.sublistView(b);

    if (b.length < 0x40 || b[0] != 0x4D || b[1] != 0x5A) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'not_a_dll',
      );
    }
    final peOff = view.getUint32(0x3C, Endian.little);
    if (peOff <= 0 || peOff + 24 > b.length) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'corrupt_pe_header',
      );
    }
    if (b[peOff] != 0x50 || b[peOff + 1] != 0x45) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'not_a_dll',
      );
    }

    final coff = peOff + 4;
    final numSections = view.getUint16(coff + 2, Endian.little);
    final sizeOptHdr = view.getUint16(coff + 16, Endian.little);
    final optHdrOff = coff + 20;
    if (optHdrOff + sizeOptHdr > b.length) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'corrupt_pe_header',
      );
    }

    final magic = view.getUint16(optHdrOff, Endian.little);
    final is64 = magic == 0x20B;

    final dataDirOff = is64 ? optHdrOff + 112 : optHdrOff + 96;
    if (dataDirOff + 8 > b.length) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'corrupt_pe_header',
      );
    }
    final exportRva = view.getUint32(dataDirOff, Endian.little);

    if (!is64) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'not_64bit',
      );
    }

    final sectionsOff = optHdrOff + sizeOptHdr;
    int rvaToFile(int rva) {
      for (var i = 0; i < numSections; i++) {
        final s = sectionsOff + i * 40;
        if (s + 24 > b.length) return -1;
        final vSize = view.getUint32(s + 8, Endian.little);
        final vAddr = view.getUint32(s + 12, Endian.little);
        final rawSize = view.getUint32(s + 16, Endian.little);
        final rawPtr = view.getUint32(s + 20, Endian.little);
        final size = vSize > rawSize ? vSize : rawSize;
        if (rva >= vAddr && rva < vAddr + size) {
          return rva - vAddr + rawPtr;
        }
      }
      return -1;
    }

    if (exportRva == 0) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'no_exports',
      );
    }

    final expOff = rvaToFile(exportRva);
    if (expOff < 0 || expOff + 40 > b.length) {
      return const PluginValidationResult(
        valid: false,
        failureReason: 'corrupt_exports',
      );
    }

    final nameRva = view.getUint32(expOff + 12, Endian.little);
    final numNames = view.getUint32(expOff + 24, Endian.little);
    final namesRva = view.getUint32(expOff + 32, Endian.little);

    String? internalName;
    if (nameRva > 0) {
      final off = rvaToFile(nameRva);
      if (off > 0) internalName = _readCString(b, off);
    }

    final exports = <String>[];
    final namesFile = rvaToFile(namesRva);
    if (namesFile > 0) {
      for (var i = 0; i < numNames; i++) {
        if (namesFile + i * 4 + 4 > b.length) break;
        final nameRvaI = view.getUint32(namesFile + i * 4, Endian.little);
        final nameOff = rvaToFile(nameRvaI);
        if (nameOff <= 0) continue;
        final s = _readCString(b, nameOff);
        if (s.isNotEmpty) exports.add(s);
      }
    }

    final lowerName = (internalName ?? '').toLowerCase();
    for (final entry in _knownIncompatibleSubstrings.entries) {
      if (lowerName.contains(entry.key)) {
        return PluginValidationResult(
          valid: false,
          failureReason: 'incompatible:${entry.value}',
          exports: exports,
          internalDllName: internalName,
        );
      }
    }

    if (!exports.contains(_requiredExport)) {
      return PluginValidationResult(
        valid: false,
        failureReason: 'missing_nams_export',
        exports: exports,
        internalDllName: internalName,
      );
    }

    return PluginValidationResult(
      valid: true,
      exports: exports,
      internalDllName: internalName,
    );
  }

String _readCString(Uint8List b, int offset) {
  final end = b.indexOf(0, offset);
  final stop = end < 0 ? b.length : end;
  return String.fromCharCodes(b.sublist(offset, stop));
}
