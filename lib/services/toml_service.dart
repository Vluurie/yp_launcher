import 'dart:convert';
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
  static Iterable<({TomlExpression? expression, int start, int end})>
  sourceExpressions(String raw) sync* {
    final parser = TomlExpression.parser;
    var offset = 0;
    while (offset < raw.length) {
      final parsed = parser.parse(raw, start: offset);
      final expression = parsed.value;
      var end = parsed.position;
      if (raw.startsWith('\r\n', end)) {
        end += 2;
      } else if (raw.startsWith('\n', end)) {
        end++;
      }
      if (end <= offset) throw const FormatException('Invalid TOML expression');
      yield (expression: expression, start: offset, end: end);
      offset = end;
    }
  }

  static bool sameValues(dynamic a, dynamic b) {
    if (a is Map && b is Map) {
      return a.length == b.length &&
          a.keys.every(
            (key) => b.containsKey(key) && sameValues(a[key], b[key]),
          );
    }
    if (a is List && b is List) {
      return a.length == b.length &&
          a.asMap().entries.every((e) => sameValues(e.value, b[e.key]));
    }
    if (a is double && b is double && a.isNaN && b.isNaN) return true;
    return a == b;
  }

  static Map<String, dynamic> _mergeValues(
    Map<String, dynamic> original,
    Map<String, dynamic> updates,
  ) {
    final merged = Map<String, dynamic>.from(original);
    for (final entry in updates.entries) {
      final previous = merged[entry.key];
      merged[entry.key] =
          previous is Map<String, dynamic> &&
              entry.value is Map<String, dynamic>
          ? _mergeValues(previous, entry.value as Map<String, dynamic>)
          : entry.value;
    }
    return merged;
  }

  static String serialize(
    Map<String, dynamic> values, {
    String commentsFrom = '',
  }) {
    final comments = StringBuffer();
    for (final span in sourceExpressions(commentsFrom)) {
      final source = commentsFrom.substring(span.start, span.end);
      final expression = span.expression;
      if (expression == null) {
        if (source.trimLeft().startsWith('#')) comments.write(source);
      } else {
        final trimmed = source.trimLeft();
        final end = expression is TomlKeyValuePair
            ? TomlKeyValuePair.parser.parse(trimmed).position
            : TomlTable.parser.parse(trimmed).position;
        final trailing = trimmed.substring(end).trim();
        if (trailing.startsWith('#')) comments.writeln(trailing);
      }
      if (comments.isNotEmpty && !comments.toString().endsWith('\n')) {
        comments.writeln();
      }
    }
    final output = '$comments${TomlDocument.fromMap(values)}';
    final parsed = parseStrict(output);
    if (!parsed.ok || !sameValues(parsed.values, values)) {
      throw const FormatException(
        'TOML write would change configuration values',
      );
    }
    return output;
  }

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
    Map<String, dynamic> updates,
  ) {
    final original = parseStrict(originalContent);
    if (!original.ok) throw FormatException(original.error ?? 'Invalid TOML');
    final values = _mergeValues(original.values, updates);
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

    for (final span in sourceExpressions(originalContent)) {
      final line = originalContent
          .substring(span.start, span.end)
          .replaceFirst(RegExp(r'\r?\n$'), '');
      final expression = span.expression;
      if (expression == null) {
        result.add(line);
        continue;
      }

      if (expression is TomlTable) {
        flushSection(currentSection);
        if (firstSectionIndex == -1) {
          var insertAt = result.length;
          while (insertAt > 0 && result[insertAt - 1].trim().isEmpty) {
            insertAt--;
          }
          firstSectionIndex = insertAt;
        }
        currentSection = expression.name.parts
            .map((part) => part.name)
            .join('.');
        writtenBySection[currentSection] ??= <String>{};
        result.add(line);
        continue;
      }

      if (expression is! TomlKeyValuePair) continue;
      final key = expression.key.parts.map((part) => part.name).join('.');

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

    final output = result.join('\n');
    final parsed = parseStrict(output);
    if (parsed.ok && sameValues(parsed.values, values)) return output;
    return serialize(values, commentsFrom: originalContent);
  }

  static String _formatValue(dynamic value) {
    if (value is String) return jsonEncode(value);
    if (value is List) return '[${value.map(_formatValue).join(', ')}]';
    if (value is Map) {
      final entries = value.entries.map(
        (entry) =>
            '${TomlSimpleKey.from(entry.key)} = ${_formatValue(entry.value)}',
      );
      return '{ ${entries.join(', ')} }';
    }
    return TomlValue.from(value).toString();
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
