import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/widgets/collapsible_card.dart';
import 'package:yp_launcher/widgets/config_change_status.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';
import 'package:yp_launcher/widgets/config_field_slider.dart';

void main() {
  final l = lookupAppLocalizations(const Locale('en'));

  Future<void> pumpCard(WidgetTester tester, List<Widget> children) {
    return tester.pumpWidget(
      MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: Scaffold(
          body: SingleChildScrollView(
            child: CollapsibleCard(title: 'CARD', children: children),
          ),
        ),
      ),
    );
  }

  ConfigFieldBool toggle(String label, bool value, bool def) => ConfigFieldBool(
    label: label,
    value: value,
    defaultValue: def,
    onChanged: (_) {},
  );

  testWidgets('a collapsed card counts settings that differ from default', (
    tester,
  ) async {
    await pumpCard(tester, [
      toggle('MSAA', true, false),
      Padding(
        padding: EdgeInsets.zero,
        child: Column(
          children: [
            toggle('Bloom', true, true),
            ConfigFieldSlider(
              label: 'Shadow distance',
              value: 2.0,
              defaultValue: 1.0,
              min: 1,
              max: 3,
              step: 0.1,
              onChanged: (_) {},
            ),
          ],
        ),
      ),
      ConfigChangeGroup(
        changedLabels: const ['New items'],
        child: const SizedBox(),
      ),
    ]);
    await tester.pumpAndSettle();

    expect(find.text('MSAA'), findsNothing);
    expect(find.text(l.cardChangedCount(3)), findsOneWidget);
    final tooltip = tester.widget<Tooltip>(find.byType(Tooltip).first);
    expect(tooltip.message, contains('MSAA'));
    expect(tooltip.message, contains('Shadow distance'));
    expect(tooltip.message, contains('New items'));
    expect(tooltip.message, isNot(contains('Bloom')));
  });

  testWidgets('a card left at defaults shows no badge', (tester) async {
    await pumpCard(tester, [
      toggle('MSAA', false, false),
      toggle('Bloom', true, true),
    ]);
    await tester.pumpAndSettle();

    expect(find.textContaining('changed'), findsNothing);
  });

  test('values compare by meaning, not by type', () {
    expect(configValueEquals(1, 1.0), isTrue);
    expect(configValueEquals(1.0000001, 1.0), isTrue);
    expect(configValueEquals([1.0, 2], [1, 2.0]), isTrue);
    expect(configValueEquals({'a': 1}, {'a': 1.0}), isTrue);
    expect(configValueEquals([1, 2], [1, 3]), isFalse);
  });
}
