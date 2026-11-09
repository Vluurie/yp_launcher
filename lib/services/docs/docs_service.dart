import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:markdown/markdown.dart' as md;

class DocPage {
  final String path;
  final String category;
  final String title;
  final String fileName;

  const DocPage({
    required this.path,
    required this.category,
    required this.title,
    required this.fileName,
  });

  bool get isResource => path.contains('/resources/');

  bool get isAuthoring => path.contains('/authoring/');
}

class DocHeading {
  final int level;
  final String text;
  final String anchor;

  const DocHeading({
    required this.level,
    required this.text,
    required this.anchor,
  });
}

class DocSearchHit {
  final DocPage page;
  final String? excerpt;

  const DocSearchHit({required this.page, this.excerpt});
}

class DocsService {
  DocsService._();

  static const _root = 'assets/docs/';

  static List<DocPage>? _cache;
  static final Map<String, String> _contentCache = {};

  static String? _sourceRoot;
  static bool _sourceRootResolved = false;

  static bool get canEdit => sourceRoot != null;

  static String? get sourceRoot {
    if (_sourceRootResolved) return _sourceRoot;
    _sourceRootResolved = true;
    if (!kDebugMode) return null;

    var dir = Directory.current;
    for (var depth = 0; depth < 5; depth++) {
      final candidate = Directory(pathJoin(dir.path, 'assets', 'docs'));
      if (candidate.existsSync()) {
        _sourceRoot = candidate.path;
        return _sourceRoot;
      }
      final parent = dir.parent;
      if (parent.path == dir.path) break;
      dir = parent;
    }
    return null;
  }

  static String pathJoin(String a, String b, [String? c]) {
    final sep = Platform.pathSeparator;
    return c == null ? '$a$sep$b' : '$a$sep$b$sep$c';
  }

  static File? sourceFileFor(String assetPath) {
    final root = sourceRoot;
    if (root == null || !assetPath.startsWith(_root)) return null;
    final relative = assetPath
        .substring(_root.length)
        .replaceAll('/', Platform.pathSeparator);
    final file = File('$root${Platform.pathSeparator}$relative');
    return file.existsSync() ? file : null;
  }

  static Future<void> saveSource(String assetPath, String content) async {
    final file = sourceFileFor(assetPath);
    if (file == null) return;
    await file.writeAsString(content);
    _contentCache[assetPath] = content;
    _cache = null;
  }

  static void invalidate() {
    _contentCache.clear();
    _cache = null;
  }

  static Future<List<DocPage>> loadIndex() async {
    final cached = _cache;
    if (cached != null) return cached;

    final manifest = await AssetManifest.loadFromAssetBundle(rootBundle);

    final paths =
        manifest
            .listAssets()
            .where(
              (key) =>
                  key.startsWith(_root) &&
                  key.endsWith('.md') &&
                  !key.startsWith('${_root}images/'),
            )
            .toList()
          ..sort(_comparePaths);

    final pages = <DocPage>[];
    for (final path in paths) {
      final relative = path.substring(_root.length);
      final slash = relative.indexOf('/');
      if (slash == -1) continue;
      final fileName = relative.substring(relative.lastIndexOf('/') + 1);
      pages.add(
        DocPage(
          path: path,
          category: relative.substring(0, slash),
          fileName: fileName,
          title: _titleFor(await _read(path), fileName),
        ),
      );
    }

    _cache = pages;
    return pages;
  }

  static Future<String> content(String path) => _read(path);

  static Future<String> _read(String path) async {
    final cached = _contentCache[path];
    if (cached != null) return cached;

    final source = sourceFileFor(path);
    if (source != null) {
      final fromDisk = await source.readAsString();
      _contentCache[path] = fromDisk;
      return fromDisk;
    }

    final data = await rootBundle.loadString(path);
    _contentCache[path] = data;
    return data;
  }

  static String _titleFor(String markdown, String fileName) {
    for (final line in const LineSplitter().convert(markdown)) {
      final trimmed = line.trim();
      if (trimmed.isEmpty) continue;
      if (trimmed.startsWith('# ')) return trimmed.substring(2).trim();
      break;
    }
    return prettyName(fileName);
  }

  static String prettyName(String fileName) {
    var name = fileName;
    if (name.endsWith('.md')) {
      name = name.substring(0, name.length - 3);
    }

    final underscore = name.indexOf('_');
    if (underscore > 0 && int.tryParse(name.substring(0, underscore)) != null) {
      name = name.substring(underscore + 1);
    }

    name = name.replaceAll('_', ' ').trim();
    if (name.isEmpty) return fileName;
    return name[0].toUpperCase() + name.substring(1);
  }

  static List<DocHeading> outline(String markdown) {
    final headings = <DocHeading>[];
    final seen = <String, int>{};
    var inFence = false;

    for (final line in const LineSplitter().convert(markdown)) {
      final trimmed = line.trim();

      if (trimmed.startsWith('```')) {
        inFence = !inFence;
        continue;
      }
      if (inFence || !trimmed.startsWith('##')) continue;

      var level = 0;
      while (level < trimmed.length && trimmed[level] == '#') {
        level++;
      }
      if (level > 3 || level >= trimmed.length || trimmed[level] != ' ') {
        continue;
      }

      final text = md.Document()
          .parseInline(trimmed.substring(level + 1).trim())
          .map((node) => node.textContent)
          .join()
          .trim();
      if (text.isEmpty) continue;

      final base = anchorFor(text);
      final count = seen.update(base, (v) => v + 1, ifAbsent: () => 0);
      headings.add(
        DocHeading(
          level: level,
          text: text,
          anchor: count == 0 ? base : '$base-$count',
        ),
      );
    }
    return headings;
  }

  static String anchorFor(String text) {
    final buffer = StringBuffer();
    var pendingDash = false;

    for (final rune in text.toLowerCase().runes) {
      final char = String.fromCharCode(rune);
      final isWord =
          (rune >= 0x30 && rune <= 0x39) ||
          (rune >= 0x61 && rune <= 0x7a) ||
          rune > 0x7f ||
          char == '_';

      if (isWord || char == '-') {
        if (pendingDash && buffer.isNotEmpty) buffer.write('-');
        pendingDash = false;
        buffer.write(char);
      } else if (char == ' ' || char == '\t') {
        pendingDash = true;
      }
    }
    return buffer.toString();
  }

  static String? resolveLink(String fromPath, String href) {
    if (href.startsWith('http://') || href.startsWith('https://')) return null;

    final target = href.split('#').first;
    if (target.isEmpty) return null;

    final segments = <String>[
      ...fromPath.split('/')..removeLast(),
      ...target.split('/'),
    ];

    final stack = <String>[];
    for (final segment in segments) {
      if (segment == '.' || segment.isEmpty) continue;
      if (segment == '..') {
        if (stack.isNotEmpty) stack.removeLast();
        continue;
      }
      stack.add(segment);
    }

    final resolved = stack.join('/');
    final pages = _cache;
    if (pages == null) return resolved;

    for (final page in pages) {
      if (page.path == resolved) return page.path;
    }

    final fileName = target.substring(target.lastIndexOf('/') + 1);
    for (final page in pages) {
      if (page.fileName == fileName) return page.path;
    }
    return null;
  }

  static String? anchorIn(String href) {
    final hash = href.indexOf('#');
    if (hash == -1 || hash == href.length - 1) return null;
    return href.substring(hash + 1);
  }

  static Future<List<DocSearchHit>> search(String query) async {
    final pages = await loadIndex();
    final needle = query.trim().toLowerCase();
    if (needle.isEmpty) return const [];

    final titleHits = <DocSearchHit>[];
    final bodyHits = <DocSearchHit>[];

    for (final page in pages) {
      final body = await _read(page.path);
      if (page.title.toLowerCase().contains(needle)) {
        titleHits.add(DocSearchHit(page: page));
        continue;
      }
      final index = body.toLowerCase().indexOf(needle);
      if (index == -1) continue;
      bodyHits.add(
        DocSearchHit(
          page: page,
          excerpt: _excerptAt(body, index, needle.length),
        ),
      );
    }
    return [...titleHits, ...bodyHits];
  }

  static String _excerptAt(String body, int index, int length) {
    final start = (index - 60).clamp(0, body.length);
    final end = (index + length + 80).clamp(0, body.length);
    final snippet = _collapseWhitespace(body.substring(start, end));

    final prefix = start > 0 ? '...' : '';
    final suffix = end < body.length ? '...' : '';
    return '$prefix$snippet$suffix';
  }

  static String _collapseWhitespace(String text) {
    final buffer = StringBuffer();
    var lastWasSpace = false;
    for (var i = 0; i < text.length; i++) {
      final char = text[i];
      final isSpace =
          char == ' ' || char == '\n' || char == '\r' || char == '\t';
      if (isSpace) {
        if (!lastWasSpace) buffer.write(' ');
        lastWasSpace = true;
      } else {
        buffer.write(char);
        lastWasSpace = false;
      }
    }
    return buffer.toString().trim();
  }

  static int _comparePaths(String a, String b) {
    final rankA = _pathRank(a);
    final rankB = _pathRank(b);
    if (rankA != rankB) return rankA - rankB;
    return a.compareTo(b);
  }

  static int _pathRank(String path) {
    if (path.contains('/authoring/')) return 2;
    if (path.contains('/resources/')) return 1;
    return 0;
  }
}
