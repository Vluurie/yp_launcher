import 'dart:io';
import 'dart:typed_data';

enum GraphicsDll { none, reshade, migoto }

class GraphicsDllId {
  GraphicsDllId._();

  static const _reshadeMarkers = <List<int>>[
    [0x72, 0x65, 0x73, 0x68, 0x61, 0x64, 0x65, 0x2E, 0x6D, 0x65], // reshade.me
    [0x63, 0x72, 0x6F, 0x73, 0x69, 0x72, 0x65], // crosire
  ];

  static const _migotoMarkers = <List<int>>[
    [
      0x48, 0x61, 0x63, 0x6B, 0x65, 0x72, //
      0x44, 0x65, 0x76, 0x69, 0x63, 0x65, // HackerDevice
    ],
    [
      0x48, 0x61, 0x63, 0x6B, 0x65, 0x72, //
      0x43, 0x6F, 0x6E, 0x74, 0x65, 0x78, 0x74, // HackerContext
    ],
    [0x33, 0x44, 0x4D, 0x69, 0x67, 0x6F, 0x74, 0x6F], // 3DMigoto
  ];

  static GraphicsDll identifyBytes(Uint8List bytes) {
    if (_anyMarker(bytes, _migotoMarkers)) return GraphicsDll.migoto;
    if (_anyMarker(bytes, _reshadeMarkers)) return GraphicsDll.reshade;
    return GraphicsDll.none;
  }

  static GraphicsDll identifyFile(String filePath) {
    try {
      final f = File(filePath);
      if (!f.existsSync()) return GraphicsDll.none;
      return identifyBytes(f.readAsBytesSync());
    } catch (_) {
      return GraphicsDll.none;
    }
  }

  static bool isReShade(Uint8List bytes) =>
      identifyBytes(bytes) == GraphicsDll.reshade;

  static bool isMigoto(Uint8List bytes) =>
      identifyBytes(bytes) == GraphicsDll.migoto;

  static bool _anyMarker(Uint8List haystack, List<List<int>> markers) {
    for (final needle in markers) {
      if (_contains(haystack, needle)) return true;
    }
    return false;
  }

  static bool _contains(Uint8List haystack, List<int> needle) {
    if (needle.isEmpty || needle.length > haystack.length) return false;
    final first = needle[0];
    final last = haystack.length - needle.length;
    for (var i = 0; i <= last; i++) {
      if (haystack[i] != first) continue;
      var match = true;
      for (var j = 1; j < needle.length; j++) {
        if (haystack[i + j] != needle[j]) {
          match = false;
          break;
        }
      }
      if (match) return true;
    }
    return false;
  }
}
