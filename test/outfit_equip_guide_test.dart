import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/widgets/mods/outfit_equip_guide.dart';

void _putBytes(String root, String rel) {
  final f = File(p.joinAll([root, ...rel.split('/')]));
  f.parent.createSync(recursive: true);
  f.writeAsBytesSync(List<int>.filled(32, 0));
}

void main() {
  final l = lookupAppLocalizations(const Locale('en'));
  late Directory gameDir;

  setUp(() {
    SharedPreferences.setMockInitialValues({});
    gameDir = Directory.systemTemp.createTempSync('yp_outfit_guide_');
    Directory(p.join(gameDir.path, 'nams', 'mods')).createSync(recursive: true);
  });

  tearDown(() {
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  String modRoot(String id) => p.join(gameDir.path, 'nams', 'mods', id);

  Future<BuildContext> host(WidgetTester tester) async {
    late BuildContext captured;
    await tester.pumpWidget(
      ProviderScope(
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: Builder(
            builder: (context) {
              captured = context;
              return const SizedBox();
            },
          ),
        ),
      ),
    );
    return captured;
  }

  Future<bool> shownFlag() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getBool(AppStrings.prefKeyOutfitGuideShown) == true;
  }

  Future<void> install(
    WidgetTester tester,
    BuildContext context,
    void Function() write,
  ) async {
    await tester.runAsync(() async {
      final run = OutfitEquipGuide.watchInstall(
        context,
        gameDir.path,
        () async => write(),
      );
      for (var i = 0; i < 200 && !await shownFlag(); i++) {
        await Future<void>.delayed(const Duration(milliseconds: 10));
      }
      if (!await shownFlag()) await run;
      await Future<void>.delayed(const Duration(milliseconds: 50));
    });
    await tester.pumpAndSettle();
  }

  testWidgets('the first new outfit mod shows the guide once', (tester) async {
    final context = await host(tester);

    await install(tester, context, () {
      _putBytes(modRoot('kimono'), 'data/pl/pl000d.dat');
      _putBytes(modRoot('kimono'), 'data/pl/pl000d.dtt');
    });

    expect(find.text(l.outfitGuideTitle), findsOneWidget);
    expect(find.text(l.outfitGuideWardrobeBody('F1')), findsOneWidget);
    expect(find.text(l.outfitGuideNpcTitle), findsOneWidget);
    expect(find.text(l.outfitGuideDefaultTitle), findsOneWidget);
    expect(await tester.runAsync(shownFlag), isTrue);

    await tester.tap(find.text(l.outfitGuideDone));
    await tester.pumpAndSettle();
    expect(find.text(l.outfitGuideTitle), findsNothing);

    await install(tester, context, () {
      _putBytes(modRoot('dress'), 'data/pl/pl000d.dat');
      _putBytes(modRoot('dress'), 'data/pl/pl000d.dtt');
    });
    expect(find.text(l.outfitGuideTitle), findsNothing);
  });

  testWidgets('a mod without player models shows no guide', (tester) async {
    final context = await host(tester);

    await install(tester, context, () {
      _putBytes(modRoot('enemy_tweak'), 'data/em/em1000.dat');
      _putBytes(modRoot('enemy_tweak'), 'data/em/em1000.dtt');
    });

    expect(find.text(l.outfitGuideTitle), findsNothing);
    expect(await tester.runAsync(shownFlag), isFalse);
  });

  testWidgets('an outfit mod that was already installed shows no guide', (
    tester,
  ) async {
    _putBytes(modRoot('kimono'), 'data/pl/pl000d.dat');
    _putBytes(modRoot('kimono'), 'data/pl/pl000d.dtt');
    final context = await host(tester);

    await install(tester, context, () {
      _putBytes(modRoot('kimono'), 'data/pl/pl000d.dtt');
    });

    expect(find.text(l.outfitGuideTitle), findsNothing);
    expect(await tester.runAsync(shownFlag), isFalse);
  });
}
