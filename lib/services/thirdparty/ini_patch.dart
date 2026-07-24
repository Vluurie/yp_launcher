class IniPatch {
  IniPatch._();

  static String setKey(
    String content,
    String section,
    String key,
    String value,
  ) {
    final line = '$key = $value';
    final sectionHeader = '[$section]';
    if (content.trim().isEmpty) return '$sectionHeader\n$line\n';

    final lines = content.replaceAll('\r', '').split('\n');
    var sectionIdx = -1;
    for (var i = 0; i < lines.length; i++) {
      if (lines[i].trim().toLowerCase() == sectionHeader.toLowerCase()) {
        sectionIdx = i;
        break;
      }
    }
    if (sectionIdx == -1) {
      final trimmed = content.replaceAll(RegExp(r'\n+$'), '');
      return '$trimmed\n\n$sectionHeader\n$line\n';
    }

    var sectionEnd = lines.length;
    for (var i = sectionIdx + 1; i < lines.length; i++) {
      final t = lines[i].trim();
      if (t.startsWith('[') && t.endsWith(']')) {
        sectionEnd = i;
        break;
      }
    }

    var replaced = false;
    for (var i = sectionIdx + 1; i < sectionEnd; i++) {
      if (_isKeyLine(lines[i], key)) {
        lines[i] = line;
        replaced = true;
        break;
      }
    }
    if (!replaced) lines.insert(sectionIdx + 1, line);
    return lines.join('\n');
  }

  static String? getKey(String content, String section, String key) {
    var inSection = false;
    for (final raw in content.replaceAll('\r', '').split('\n')) {
      final t = raw.trim();
      if (t.startsWith('[') && t.endsWith(']')) {
        inSection = t.toLowerCase() == '[$section]'.toLowerCase();
        continue;
      }
      if (!inSection || t.startsWith(';')) continue;
      if (_isKeyLine(t, key)) {
        final eq = t.indexOf('=');
        if (eq != -1) return t.substring(eq + 1).trim();
      }
    }
    return null;
  }

  static bool _isKeyLine(String line, String key) {
    var t = line.trim();
    if (t.startsWith(';')) t = t.substring(1).trim();
    return t.toLowerCase().startsWith(key.toLowerCase()) &&
        t.substring(key.length).trimLeft().startsWith('=');
  }
}
