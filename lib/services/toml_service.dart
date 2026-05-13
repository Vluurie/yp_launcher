import 'dart:io';
import 'package:toml/toml.dart';
import 'package:yp_launcher/services/isolate_service.dart';

String _readTomlFileSync(String path) {
  final file = File(path);
  if (file.existsSync()) {
    return file.readAsStringSync();
  }
  return '';
}

void _writeTomlFileSync(Map<String, String> params) {
  final file = File(params['path']!);
  file.writeAsStringSync(params['content']!);
}

class TomlService {
  static final _sectionRegex = RegExp(r'^\[([a-zA-Z_]\w*)\]$');

  static Map<String, dynamic> parse(String content) {
    if (content.trim().isEmpty) return {};
    try {
      final doc = TomlDocument.parse(content);
      return _convertToml(doc.toMap());
    } catch (_) {
      return _parseFallback(content);
    }
  }

  static Map<String, dynamic> _convertToml(Map<String, dynamic> map) {
    final result = <String, dynamic>{};
    for (final entry in map.entries) {
      final value = entry.value;
      if (value is Map<String, dynamic>) {
        result[entry.key] = _convertToml(value);
      } else if (value is BigInt) {
        result[entry.key] = value.toInt();
      } else {
        result[entry.key] = value;
      }
    }
    return result;
  }

  static Map<String, dynamic> _parseFallback(String content) {
    final result = <String, dynamic>{};
    String? currentSection;

    for (final line in content.split('\n')) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#')) continue;

      final sectionMatch = _sectionRegex.firstMatch(trimmed);
      if (sectionMatch != null) {
        currentSection = sectionMatch.group(1)!;
        result.putIfAbsent(currentSection, () => <String, dynamic>{});
        continue;
      }

      final eqIndex = trimmed.indexOf('=');
      if (eqIndex == -1) continue;

      final key = trimmed.substring(0, eqIndex).trim();
      var value = trimmed.substring(eqIndex + 1).trim();

      if (!value.startsWith('"') && !value.startsWith("'")) {
        final commentIdx = value.indexOf('#');
        if (commentIdx != -1) {
          value = value.substring(0, commentIdx).trim();
        }
      }

      final parsed = _parseValueFallback(value);

      if (currentSection != null) {
        (result[currentSection] as Map<String, dynamic>)[key] = parsed;
      } else {
        result[key] = parsed;
      }
    }
    return result;
  }

  static dynamic _parseValueFallback(String value) {
    if (value == 'true') return true;
    if (value == 'false') return false;

    if (value.startsWith('[') && value.endsWith(']')) {
      final inner = value.substring(1, value.length - 1).trim();
      if (inner.isEmpty) return <String>[];
      return inner.split(',').map((e) {
        final trimmed = e.trim();
        if (trimmed.startsWith('"') && trimmed.endsWith('"')) {
          return trimmed.substring(1, trimmed.length - 1);
        }
        return trimmed;
      }).toList();
    }

    if (value.startsWith('0x') || value.startsWith('0X')) {
      final hexVal = int.tryParse(value);
      if (hexVal != null) return hexVal;
    }

    final intVal = int.tryParse(value);
    if (intVal != null && !value.contains('.')) return intVal;
    final doubleVal = double.tryParse(value);
    if (doubleVal != null) return doubleVal;
    if (value.startsWith('"') && value.endsWith('"')) {
      return value.substring(1, value.length - 1);
    }
    return value;
  }

  static String updateToml(
    String originalContent,
    Map<String, dynamic> values,
  ) {
    final lines = originalContent.split('\n');
    final result = <String>[];
    String? currentSection;
    final writtenTopLevel = <String>{};

    for (final line in lines) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#')) {
        result.add(line);
        continue;
      }

      final sectionMatch = _sectionRegex.firstMatch(trimmed);
      if (sectionMatch != null) {
        currentSection = sectionMatch.group(1)!;
        result.add(line);
        continue;
      }

      final eqIndex = trimmed.indexOf('=');
      if (eqIndex == -1) {
        result.add(line);
        continue;
      }

      final key = trimmed.substring(0, eqIndex).trim();

      if (currentSection != null) {
        final sectionMap = values[currentSection];
        if (sectionMap is Map<String, dynamic> && sectionMap.containsKey(key)) {
          result.add('$key = ${_formatValue(sectionMap[key])}');
        } else {
          result.add(line);
        }
      } else {
        if (values.containsKey(key) && values[key] is! Map) {
          result.add('$key = ${_formatValue(values[key])}');
          writtenTopLevel.add(key);
        } else {
          result.add(line);
        }
      }
    }

    final missingTopLevel = values.entries
        .where((e) => e.value is! Map && !writtenTopLevel.contains(e.key))
        .toList();
    if (missingTopLevel.isNotEmpty) {
      if (result.isNotEmpty && result.last.trim().isNotEmpty) {
        result.add('');
      }
      for (final entry in missingTopLevel) {
        result.add('${entry.key} = ${_formatValue(entry.value)}');
      }
    }

    return result.join('\n');
  }

  static String _formatValue(dynamic value) {
    if (value is bool) return value.toString();
    if (value is int) return value.toString();
    if (value is double) {
      final s = value.toString();
      return s.contains('.') ? s : '$s.0';
    }
    if (value is List) {
      if (value.isEmpty) return '[]';
      final items = value.map((e) => '"$e"').join(', ');
      return '[$items]';
    }
    return '"$value"';
  }

  static Future<String> readTomlFile(String path) {
    return IsolateService.run(_readTomlFileSync, path);
  }

  static Future<void> writeTomlFile(String path, String content) {
    return IsolateService.run(_writeTomlFileSync, {
      'path': path,
      'content': content,
    });
  }
}
