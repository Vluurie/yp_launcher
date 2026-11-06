import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/widgets/busy_overlay.dart';
import 'package:yp_launcher/widgets/textures/texture_inject_card.dart';

void main() {
  for (final dismissOutside in [false, true]) {
    testWidgets(
      dismissOutside
          ? 'dismissing texture deletion keeps the pack and load order'
          : 'cancelling texture deletion keeps the pack and load order',
      (tester) async {
        final gameDir = Directory.systemTemp.createTempSync('yp_texture_cancel_');
        final container = ProviderContainer();
        addTearDown(() {
          container.dispose();
          gameDir.deleteSync(recursive: true);
        });
        final packDir = Directory(
          path.join(gameDir.path, 'nams', 'inject', 'textures', 'pack'),
        )..createSync(recursive: true);
        final configFile = File(
          path.join(gameDir.path, 'nams', 'texture_injection.toml'),
        );
        const raw = 'load_order = ["pack", "inactive_pack"]\n';
        configFile.writeAsStringSync(raw);
        final notifier = container.read(configStateControllerProvider.notifier);
        notifier.updateTextureInjectionSilent(
          TextureInjectionFields.loadOrder.key,
          ['pack', 'inactive_pack'],
        );
        notifier.updateTextureInjectionSilent(
          TextureInjectionFields.disabledPacks.key,
          ['inactive_pack'],
        );

        await tester.pumpWidget(
          UncontrolledProviderScope(
            container: container,
            child: MaterialApp(
              localizationsDelegates: AppLocalizations.localizationsDelegates,
              supportedLocales: AppLocalizations.supportedLocales,
              home: Scaffold(
                body: Consumer(
                  builder: (context, ref, _) => SingleChildScrollView(
                    child: TextureInjectCard(
                      tex: ref.watch(configStateControllerProvider)
                          .textureInjectionValues,
                      notifier: notifier,
                      gameDir: gameDir.path,
                      installedTextures: const ['pack', 'inactive_pack'],
                      detectedFolders: const ['pack', 'inactive_pack'],
                      conflicts: const {},
                      bundledOriginByPack: const {},
                      onDelete: (name) async {
                        final confirmed = await showDeleteConfirm(
                          context,
                          title: 'Delete texture',
                          body: name,
                          deleteLabel: 'Delete',
                        );
                        expect(confirmed, isFalse);
                      },
                    ),
                  ),
                ),
              ),
            ),
          ),
        );

        await tester.tap(find.byIcon(Icons.close));
        await tester.pumpAndSettle();
        if (dismissOutside) {
          await tester.tapAt(const Offset(5, 5));
        } else {
          await tester.tap(find.text('Cancel'));
        }
        await tester.pumpAndSettle();

        expect(
          container.read(configStateControllerProvider)
              .textureInjectionValues[TextureInjectionFields.loadOrder.key],
          ['pack', 'inactive_pack'],
        );
        expect(find.text('pack'), findsOneWidget);
        expect(packDir.existsSync(), isTrue);
        expect(configFile.readAsStringSync(), raw);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox.shrink());
      },
    );
  }
}
