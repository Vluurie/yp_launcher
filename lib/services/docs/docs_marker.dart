class DocsMarker {
  final String id;
  final Map<String, String> args;
  final int start;
  final int end;

  const DocsMarker({
    required this.id,
    required this.args,
    required this.start,
    required this.end,
  });

  static const _open = '<!-- yp:';
  static const _close = '-->';

  static DocsMarker? firstIn(String markdown) {
    final start = markdown.indexOf(_open);
    if (start == -1) return null;

    final close = markdown.indexOf(_close, start);
    if (close == -1) return null;

    final inner = markdown.substring(start + _open.length, close).trim();
    if (inner.isEmpty) return null;

    final space = inner.indexOf(' ');
    final id = space == -1 ? inner : inner.substring(0, space);
    final rest = space == -1 ? '' : inner.substring(space + 1);

    return DocsMarker(
      id: id,
      args: _parseArgs(rest),
      start: start,
      end: close + _close.length,
    );
  }

  static Map<String, String> _parseArgs(String text) {
    final args = <String, String>{};
    var index = 0;

    while (index < text.length) {
      final equals = text.indexOf('=', index);
      if (equals == -1) break;

      final key = text.substring(index, equals).trim();
      if (key.isEmpty) break;

      final afterEquals = equals + 1;
      if (afterEquals >= text.length || text[afterEquals] != '"') break;

      final closingQuote = text.indexOf('"', afterEquals + 1);
      if (closingQuote == -1) break;

      args[key] = text.substring(afterEquals + 1, closingQuote);
      index = closingQuote + 1;
    }
    return args;
  }
}
