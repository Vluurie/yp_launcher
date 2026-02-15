import 'dart:io';

class TomlService {
  static Map<String, dynamic> parse(String content) {
    final result = <String, dynamic>{};
    for (final line in content.split('\n')) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#')) continue;

      final eqIndex = trimmed.indexOf('=');
      if (eqIndex == -1) continue;

      final key = trimmed.substring(0, eqIndex).trim();
      var value = trimmed.substring(eqIndex + 1).trim();

      // Strip inline comments (only outside quotes)
      if (!value.startsWith('"') && !value.startsWith("'")) {
        final commentIdx = value.indexOf('#');
        if (commentIdx != -1) {
          value = value.substring(0, commentIdx).trim();
        }
      }

      result[key] = _parseValue(value);
    }
    return result;
  }

  static dynamic _parseValue(String value) {
    if (value == 'true') return true;
    if (value == 'false') return false;
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
    for (final line in lines) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#')) {
        result.add(line);
        continue;
      }
      final eqIndex = trimmed.indexOf('=');
      if (eqIndex == -1) {
        result.add(line);
        continue;
      }
      final key = trimmed.substring(0, eqIndex).trim();
      if (values.containsKey(key)) {
        result.add('$key = ${_formatValue(values[key])}');
      } else {
        result.add(line);
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
    return '"$value"';
  }

  static Future<String> readTomlFile(String path) async {
    final file = File(path);
    if (await file.exists()) {
      return await file.readAsString();
    }
    return '';
  }

  static Future<void> writeTomlFile(String path, String content) async {
    final file = File(path);
    await file.writeAsString(content);
  }
}
