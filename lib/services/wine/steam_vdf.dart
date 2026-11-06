import 'package:vdf/vdf.dart';

Map<dynamic, dynamic> parseVdf(String content) => vdfDecode(content);

/// Every library `path` entry in [content], with VDF escaping undone.
List<String> vdfPathEntries(String content) {
  final paths = <String>[];
  void collect(Map<dynamic, dynamic> block) {
    for (final entry in block.entries) {
      final value = entry.value;
      if (value is Map) {
        collect(value);
      } else if ('${entry.key}'.toLowerCase() == 'path' && value != null) {
        paths.add('$value'.replaceAll(r'\\', r'\').replaceAll(r'\"', '"'));
      }
    }
  }

  collect(parseVdf(content));
  return paths;
}

Map<dynamic, dynamic>? vdfBlock(Map<dynamic, dynamic> block, String key) {
  final value = _vdfChild(block, key);
  return value is Map ? value : null;
}

String? vdfString(Map<dynamic, dynamic> block, String key) {
  final value = _vdfChild(block, key);
  return value == null || value is Map ? null : '$value';
}

Map<dynamic, dynamic>? vdfFindBlock(Map<dynamic, dynamic> block, String key) {
  final direct = vdfBlock(block, key);
  if (direct != null) return direct;
  for (final value in block.values) {
    if (value is! Map) continue;
    final found = vdfFindBlock(value, key);
    if (found != null) return found;
  }
  return null;
}

Object? _vdfChild(Map<dynamic, dynamic> block, String key) {
  final wanted = key.toLowerCase();
  for (final entry in block.entries) {
    if ('${entry.key}'.toLowerCase() == wanted) return entry.value;
  }
  return null;
}
