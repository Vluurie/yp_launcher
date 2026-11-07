import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/docs/docs_marker.dart';

void main() {
  group('firstIn', () {
    test('returns null when there is no marker', () {
      expect(DocsMarker.firstIn('# Title\n\nJust prose.\n'), isNull);
    });

    test('reads a bare marker', () {
      const md = 'before\n\n<!-- yp:item-builder -->\n\nafter';
      final marker = DocsMarker.firstIn(md)!;

      expect(marker.id, 'item-builder');
      expect(marker.args, isEmpty);
      expect(md.substring(0, marker.start).trim(), 'before');
      expect(md.substring(marker.end).trim(), 'after');
    });

    test('reads arguments', () {
      const md =
          '<!-- yp:figure src="assets/docs/images/a.jpg" caption="Hello" '
          'side="left" -->';
      final marker = DocsMarker.firstIn(md)!;

      expect(marker.id, 'figure');
      expect(marker.args['src'], 'assets/docs/images/a.jpg');
      expect(marker.args['caption'], 'Hello');
      expect(marker.args['side'], 'left');
    });

    test('keeps spaces and punctuation inside a quoted value', () {
      const md = '<!-- yp:figure caption="It is, in fact, fine." -->';
      expect(
        DocsMarker.firstIn(md)!.args['caption'],
        'It is, in fact, fine.',
      );
    });

    test('finds the first of several markers', () {
      const md = 'a\n<!-- yp:first-mod -->\nb\n<!-- yp:figure src="x" -->\nc';
      expect(DocsMarker.firstIn(md)!.id, 'first-mod');
    });

    test('ignores an unterminated marker', () {
      expect(DocsMarker.firstIn('<!-- yp:figure src="x"'), isNull);
    });

    test('ignores an empty marker', () {
      expect(DocsMarker.firstIn('<!-- yp: -->'), isNull);
    });

    test('an unquoted argument is skipped rather than misparsed', () {
      final marker = DocsMarker.firstIn('<!-- yp:figure src=x.jpg -->')!;
      expect(marker.id, 'figure');
      expect(marker.args['src'], isNull);
    });

    test('start and end cover the whole comment', () {
      const md = 'A<!-- yp:first-mod -->B';
      final marker = DocsMarker.firstIn(md)!;
      expect(md.substring(marker.start, marker.end),
          '<!-- yp:first-mod -->');
    });
  });
}
