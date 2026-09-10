import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/mods/mod_drop_zone.dart';
import 'package:yp_launcher/widgets/two_column_layout.dart';

Widget host(Widget child, {double width = 380, double height = 700}) {
  return MaterialApp(
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    home: Scaffold(
      body: Center(
        child: SizedBox(width: width, height: height, child: child),
      ),
    ),
  );
}

void main() {
  testWidgets('two columns stack below the breakpoint and sit side by side '
      'above it', (tester) async {
    const left = SizedBox(key: Key('left'), height: 40);
    const right = SizedBox(key: Key('right'), height: 40);

    Future<void> pumpAt(double width) => tester.pumpWidget(
      host(
        const TwoColumnLayout(left: left, right: right),
        width: width,
        height: 400,
      ),
    );

    await pumpAt(TwoColumnLayout.defaultBreakpoint - 1);
    expect(
      tester.getTopLeft(find.byKey(const Key('left'))).dy,
      lessThan(tester.getTopLeft(find.byKey(const Key('right'))).dy),
    );
    expect(
      tester.getTopLeft(find.byKey(const Key('left'))).dx,
      tester.getTopLeft(find.byKey(const Key('right'))).dx,
    );

    await pumpAt(TwoColumnLayout.defaultBreakpoint + 1);
    expect(
      tester.getTopLeft(find.byKey(const Key('left'))).dy,
      tester.getTopLeft(find.byKey(const Key('right'))).dy,
    );
    expect(
      tester.getTopLeft(find.byKey(const Key('left'))).dx,
      lessThan(tester.getTopLeft(find.byKey(const Key('right'))).dx),
    );
  });

  testWidgets('the drop zone title wraps instead of overflowing', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(700, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      host(
        Align(
          alignment: Alignment.topCenter,
          child: ModDropZone(
            onDrop: (_) {},
            onBrowse: () {},
            onBrowseFolder: () {},
            title: 'Drop ReShade presets, 3DMigoto mods or game-mod DLLs here',
            hint: 'or click to browse',
          ),
        ),
      ),
    );
    await tester.pump();

    expect(tester.takeException(), isNull);

    final zone = tester.getSize(find.byType(ModDropZone));
    final title = tester.getSize(
      find.text('Drop ReShade presets, 3DMigoto mods or game-mod DLLs here'),
    );
    expect(title.width, lessThanOrEqualTo(zone.width));
  });

  testWidgets('the dropdown menu stays inside the window', (tester) async {
    tester.view.physicalSize = const Size(500, 400);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: Scaffold(
          body: Stack(
            children: [
              Positioned(
                right: 4,
                bottom: 4,
                child: AppDropdown<int>(
                  value: 0,
                  items: const [0, 1, 2],
                  itemLabel: (v) => 'A fairly long option label number $v',
                  onChanged: (_) {},
                  maxWidth: 160,
                ),
              ),
            ],
          ),
        ),
      ),
    );

    await tester.tap(find.byType(AppDropdown<int>));
    await tester.pumpAndSettle();

    expect(tester.takeException(), isNull);

    final screen = tester.view.physicalSize / tester.view.devicePixelRatio;
    final menu = find.text('A fairly long option label number 1');
    expect(menu, findsOneWidget);

    final topLeft = tester.getTopLeft(menu);
    final bottomRight = tester.getBottomRight(menu);
    expect(topLeft.dx, greaterThanOrEqualTo(0));
    expect(topLeft.dy, greaterThanOrEqualTo(0));
    expect(bottomRight.dx, lessThanOrEqualTo(screen.width));
    expect(bottomRight.dy, lessThanOrEqualTo(screen.height));
  });
}
