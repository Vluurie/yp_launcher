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

class TomlParseResult {
  final Map<String, dynamic> values;
  final String? error;

  const TomlParseResult({required this.values, this.error});

  bool get ok => error == null;
}

class TomlService {
  static final _sectionRegex = RegExp(r'^\[([a-zA-Z_][\w.]*)\]$');

  static Map<String, dynamic>? _sectionIn(
    Map<String, dynamic> values,
    String path,
  ) {
    Map<String, dynamic>? current = values;
    for (final part in path.split('.')) {
      final next = current?[part];
      if (next is! Map<String, dynamic>) return null;
      current = next;
    }
    return current;
  }

  static void _collectSectionPaths(
    Map<String, dynamic> map,
    String prefix,
    List<MapEntry<String, Map<String, dynamic>>> out,
  ) {
    for (final entry in map.entries) {
      final value = entry.value;
      if (value is Map<String, dynamic>) {
        final path = prefix.isEmpty ? entry.key : '$prefix.${entry.key}';
        out.add(MapEntry(path, value));
        _collectSectionPaths(value, path, out);
      }
    }
  }

  static Map<String, dynamic> parse(String content) {
    if (content.trim().isEmpty) return {};
    try {
      final doc = TomlDocument.parse(content);
      return _convertToml(doc.toMap());
    } catch (_) {
      return {};
    }
  }

  static TomlParseResult parseStrict(String content) {
    if (content.trim().isEmpty) {
      return const TomlParseResult(values: {});
    }
    try {
      final doc = TomlDocument.parse(content);
      return TomlParseResult(values: _convertToml(doc.toMap()));
    } catch (e) {
      return TomlParseResult(values: const {}, error: _describe(e));
    }
  }

  static String _describe(Object error) {
    final text = error.toString();
    const prefix = 'TomlParserException: ';
    if (text.startsWith(prefix)) return text.substring(prefix.length);
    return text;
  }

  static Map<String, dynamic> _convertToml(Map<String, dynamic> map) {
    final result = <String, dynamic>{};
    for (final entry in map.entries) {
      result[entry.key] = _convertTomlValue(entry.value);
    }
    return result;
  }

  static dynamic _convertTomlValue(dynamic value) {
    if (value is Map<String, dynamic>) return _convertToml(value);
    if (value is BigInt) return value.toInt();
    if (value is List) return value.map(_convertTomlValue).toList();
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
    final writtenBySection = <String, Set<String>>{};
    var firstSectionIndex = -1;

    void flushSection(String? section) {
      if (section == null) return;
      final sectionMap = _sectionIn(values, section);
      if (sectionMap == null) return;
      final written = writtenBySection[section] ?? const <String>{};
      final missing = sectionMap.entries
          .where((e) => e.value is! Map && !written.contains(e.key))
          .toList();
      if (missing.isEmpty) return;

      while (result.isNotEmpty && result.last.trim().isEmpty) {
        result.removeLast();
      }
      for (final entry in missing) {
        result.add('${entry.key} = ${_formatValue(entry.value)}');
      }
      result.add('');
    }

    var skipUntilArrayEnd = 0;

    for (final line in lines) {
      final trimmed = line.trim();

      if (skipUntilArrayEnd > 0) {
        skipUntilArrayEnd += _bracketDelta(trimmed);
        continue;
      }

      if (trimmed.isEmpty || trimmed.startsWith('#')) {
        result.add(line);
        continue;
      }

      final sectionMatch = _sectionRegex.firstMatch(trimmed);
      if (sectionMatch != null) {
        flushSection(currentSection);
        if (firstSectionIndex == -1) {
          var insertAt = result.length;
          while (insertAt > 0 && result[insertAt - 1].trim().isEmpty) {
            insertAt--;
          }
          firstSectionIndex = insertAt;
        }
        currentSection = sectionMatch.group(1)!;
        writtenBySection[currentSection] ??= <String>{};
        result.add(line);
        continue;
      }

      final eqIndex = trimmed.indexOf('=');
      if (eqIndex == -1) {
        result.add(line);
        continue;
      }

      final key = trimmed.substring(0, eqIndex).trim();
      final delta = _bracketDelta(trimmed.substring(eqIndex + 1));
      if (delta > 0) skipUntilArrayEnd = delta;

      if (currentSection != null) {
        final sectionMap = _sectionIn(values, currentSection);
        if (sectionMap != null &&
            sectionMap.containsKey(key) &&
            sectionMap[key] is! Map) {
          result.add('$key = ${_formatValue(sectionMap[key])}');
          (writtenBySection[currentSection] ??= <String>{}).add(key);
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

    flushSection(currentSection);

    final missingTopLevel = values.entries
        .where((e) => e.value is! Map && !writtenTopLevel.contains(e.key))
        .toList();
    if (missingTopLevel.isNotEmpty) {
      final rendered = missingTopLevel
          .map((e) => '${e.key} = ${_formatValue(e.value)}')
          .toList();
      if (firstSectionIndex == -1) {
        if (result.isNotEmpty && result.last.trim().isNotEmpty) {
          result.add('');
        }
        result.addAll(rendered);
      } else {
        result.insertAll(firstSectionIndex, rendered);
      }
    }

    final allSections = <MapEntry<String, Map<String, dynamic>>>[];
    _collectSectionPaths(values, '', allSections);
    for (final section in allSections) {
      if (writtenBySection.containsKey(section.key)) continue;
      final scalarEntries = section.value.entries
          .where((e) => e.value is! Map)
          .toList();
      if (scalarEntries.isEmpty) continue;
      if (result.isNotEmpty && result.last.trim().isNotEmpty) {
        result.add('');
      }
      result.add('[${section.key}]');
      for (final entry in scalarEntries) {
        result.add('${entry.key} = ${_formatValue(entry.value)}');
      }
    }

    return result.join('\n');
  }

  /// Net `[` minus `]` in [text], ignoring brackets inside quotes or comments.
  /// A positive result means the value continues on the following lines.
  static int _bracketDelta(String text) {
    var depth = 0;
    var inSingle = false;
    var inDouble = false;
    for (var i = 0; i < text.length; i++) {
      final ch = text[i];
      if (ch == "'" && !inDouble) {
        inSingle = !inSingle;
      } else if (ch == '"' && !inSingle) {
        inDouble = !inDouble;
      } else if (!inSingle && !inDouble) {
        if (ch == '#') break;
        if (ch == '[') depth++;
        if (ch == ']') depth--;
      }
    }
    return depth;
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
      final items = value.map(_formatValue).join(', ');
      return '[$items]';
    }
    if (value is Map) {
      final pairs = value.entries
          .map((e) => '${e.key} = ${_formatValue(e.value)}')
          .join(', ');
      return '{ $pairs }';
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
