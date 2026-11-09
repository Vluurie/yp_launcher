import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/docs/docs_service.dart';

void main() {
  group('anchorFor', () {
    test('slugifies headings the way markdown anchors do', () {
      expect(DocsService.anchorFor('Getting started'), 'getting-started');
      expect(
        DocsService.anchorFor('What you need first'),
        'what-you-need-first',
      );
      expect(
        DocsService.anchorFor('Step 1 - Make a folder'),
        'step-1---make-a-folder',
      );
    });

    test('drops punctuation and collapses spacing', () {
      expect(DocsService.anchorFor('Why? Because!'), 'why-because');
      expect(DocsService.anchorFor('  spaced   out  '), 'spaced-out');
    });

    test('never starts with a dash', () {
      expect(DocsService.anchorFor('  leading'), 'leading');
    });
  });

  group('outline', () {
    test('collects h2 and h3 but ignores h1 and fenced code', () {
      const markdown = '''
# Title

## First

```
## not a heading
```

### Nested

#### Too deep

## Second
''';
      final headings = DocsService.outline(markdown);
      expect(headings.map((h) => h.text), ['First', 'Nested', 'Second']);
      expect(headings.map((h) => h.level), [2, 3, 2]);
      expect(headings.first.anchor, 'first');
    });

    test('ignores a hash run with no space after it', () {
      final headings = DocsService.outline('##NoSpace\n\n## Real\n');
      expect(headings.map((h) => h.text), ['Real']);
    });

    test('shows inline code and emphasis as plain text', () {
      final headings = DocsService.outline(
        '## How do mods in `thirdparty\\graphic\\` work?\n\n## A **bold** one\n',
      );
      expect(headings.map((h) => h.text), [
        'How do mods in thirdparty\\graphic\\ work?',
        'A bold one',
      ]);
      expect(headings.first.anchor, 'how-do-mods-in-thirdpartygraphic-work');
    });

    test('numbers repeated headings so anchors stay unique', () {
      const markdown =
          '## Attribute 0x400\n\n'
          'text\n\n'
          '## Attribute 0x400\n\n'
          'more\n\n'
          '## Attribute 0x400\n';
      final anchors = DocsService.outline(
        markdown,
      ).map((h) => h.anchor).toList();
      expect(anchors, [
        'attribute-0x400',
        'attribute-0x400-1',
        'attribute-0x400-2',
      ]);
      expect(anchors.toSet().length, anchors.length);
    });

    test('anchors of a real page are unique', () {
      const markdown =
          '## Step 2 - Create `mod.toml`\n\n'
          '## `mod.toml` fields\n\n'
          '### Notes\n\n'
          '### Notes\n';
      final anchors = DocsService.outline(
        markdown,
      ).map((h) => h.anchor).toList();
      expect(anchors.toSet().length, anchors.length);
    });
  });

  group('prettyName', () {
    test('strips the numeric prefix and extension', () {
      expect(DocsService.prettyName('03_item.md'), 'Item');
      expect(
        DocsService.prettyName('16_wax_to_toml_migration.md'),
        'Wax to toml migration',
      );
    });

    test('leaves names without a numeric prefix intact', () {
      expect(DocsService.prettyName('AreaAction.md'), 'AreaAction');
    });
  });

  group('anchorIn', () {
    test('returns the fragment when present', () {
      expect(DocsService.anchorIn('07_quest.md#rewards'), 'rewards');
      expect(DocsService.anchorIn('#top'), 'top');
    });

    test('returns null without a usable fragment', () {
      expect(DocsService.anchorIn('07_quest.md'), isNull);
      expect(DocsService.anchorIn('07_quest.md#'), isNull);
    });
  });

  group('resolveLink', () {
    const from = 'assets/docs/scripting/03_getting_started.md';

    test('resolves a sibling page', () {
      expect(
        DocsService.resolveLink(from, '01_hap.md'),
        'assets/docs/scripting/01_hap.md',
      );
    });

    test('resolves a parent-relative page', () {
      expect(
        DocsService.resolveLink(from, '../native/07_quest.md'),
        'assets/docs/native/07_quest.md',
      );
    });

    test('drops the fragment when resolving', () {
      expect(
        DocsService.resolveLink(from, '../native/07_quest.md#rewards'),
        'assets/docs/native/07_quest.md',
      );
    });

    test('returns null for external links', () {
      expect(DocsService.resolveLink(from, 'https://example.com'), isNull);
    });
  });
}
