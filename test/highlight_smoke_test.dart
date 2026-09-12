import 'package:flutter_test/flutter_test.dart';
import 'package:highlight/highlight.dart' show highlight;

void main() {
  group('highlight parses the languages used in the docs', () {
    test('toml through the ini grammar', () {
      const code = 'id = "my_first_mod"\n'
          'version = "1.0.0"\n'
          '\n'
          '[text.name]\n'
          '_us = "Test Pebble"\n';
      final result = highlight.parse(code, language: 'ini');
      expect(result.nodes, isNotNull);
      expect(result.nodes, isNotEmpty);
    });

    test('ruby', () {
      const code = 'def hello\n  puts "hi"\nend\n';
      final result = highlight.parse(code, language: 'ruby');
      expect(result.nodes, isNotEmpty);
    });

    test('xml', () {
      const code = '<root><child attr="1" /></root>';
      final result = highlight.parse(code, language: 'xml');
      expect(result.nodes, isNotEmpty);
    });

    test('json', () {
      final result = highlight.parse('{"a": 1}', language: 'json');
      expect(result.nodes, isNotEmpty);
    });

    test('an unknown language does not throw', () {
      expect(
        () => highlight.parse('x = 1', language: 'nonexistent'),
        returnsNormally,
      );
    });

    test('an ascii diagram stays intact through ini', () {
      const diagram = '   group OFF            group ON\n'
          '   ┌──────────────┐     ┌──────────────┐\n'
          '   │  1. Place    │     │  1. Place    │──► happens\n'
          '   └──────────────┘     └──────────────┘\n';
      final result = highlight.parse(diagram, language: 'ini');
      final flattened = _flatten(result.nodes!);
      expect(flattened, diagram);
    });
  });
}

String _flatten(List<dynamic> nodes) {
  final buffer = StringBuffer();
  void walk(List<dynamic> list) {
    for (final node in list) {
      if (node.value != null) buffer.write(node.value);
      final children = node.children;
      if (children != null) walk(children as List<dynamic>);
    }
  }

  walk(nodes);
  return buffer.toString();
}
