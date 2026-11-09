import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

const _maxBytes = 100 * 1024;

const _allowedExtensions = {'.jpg', '.jpeg', '.png', '.webp'};

void main() {
  final root = Directory('assets/docs/images');

  test('every docs image stays under 100 KB', () {
    if (!root.existsSync()) return;

    final tooBig = <String>[];
    for (final file in _imageFiles(root)) {
      final bytes = file.lengthSync();
      if (bytes > _maxBytes) {
        final kb = (bytes / 1024).toStringAsFixed(0);
        tooBig.add('${p.basename(file.path)} is ${kb} KB');
      }
    }

    expect(
      tooBig,
      isEmpty,
      reason:
          'Docs images must be optimised to 100 KB or less before they are '
          'committed. Re-export at a smaller size or higher compression:\n'
          '  ${tooBig.join('\n  ')}',
    );
  });

  test('docs images use a web-friendly format', () {
    if (!root.existsSync()) return;

    final wrongType = <String>[];
    for (final file in root.listSync(recursive: true).whereType<File>()) {
      final ext = p.extension(file.path).toLowerCase();
      if (ext.isEmpty || ext == '.md') continue;
      if (!_allowedExtensions.contains(ext)) {
        wrongType.add('${p.basename(file.path)} ($ext)');
      }
    }

    expect(
      wrongType,
      isEmpty,
      reason:
          'Docs images must be .jpg, .png or .webp. Convert these:\n'
          '  ${wrongType.join('\n  ')}',
    );
  });

  test('every image referenced by a doc page exists', () {
    final docs = Directory('assets/docs');
    if (!docs.existsSync()) return;

    final image = RegExp(r'!\[[^\]]*\]\(([^)\s]+)\)');
    final missing = <String>[];
    final markers = <String>[];

    for (final file in docs.listSync(recursive: true).whereType<File>()) {
      if (!file.path.endsWith('.md')) continue;
      final text = _withoutCodeBlocks(file.readAsStringSync());

      if (text.contains('yp:figure')) {
        markers.add(p.basename(file.path));
      }
      for (final match in image.allMatches(text)) {
        final src = match.group(1)!;
        if (src.startsWith('http')) continue;
        final target = p.normalize(p.join(p.dirname(file.path), src));
        if (!File(target).existsSync()) {
          missing.add('${p.basename(file.path)} -> $src');
        }
      }
    }

    expect(missing, isEmpty, reason: 'Broken image references:\n'
        '  ${missing.join('\n  ')}');
    expect(markers, isEmpty,
        reason: 'yp:figure is retired; use a plain markdown image instead:\n'
            '  ${markers.join('\n  ')}');
  });
}

String _withoutCodeBlocks(String markdown) {
  final buffer = StringBuffer();
  var inFence = false;
  for (final line in markdown.split('\n')) {
    if (line.trimLeft().startsWith('```')) {
      inFence = !inFence;
      continue;
    }
    if (!inFence) buffer.writeln(line);
  }
  return buffer.toString();
}

Iterable<File> _imageFiles(Directory root) {
  return root
      .listSync(recursive: true)
      .whereType<File>()
      .where((f) => _allowedExtensions.contains(
            p.extension(f.path).toLowerCase(),
          ));
}
