import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_field_bool.dart';

class CutsceneSubtitleCard extends ConsumerWidget {
  const CutsceneSubtitleCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final gameDir = ref.watch(appStateControllerProvider).selectedDirectory;
    final cutscene = (ref.watch(configStateControllerProvider).namsValues[
            'cutscene'] as Map<String, dynamic>?) ??
        const {};

    bool valueOf(ConfigField<bool> field) =>
        cutscene[field.key] as bool? ?? field.defaultValue;

    void set(ConfigField<bool> field, bool v) {
      if (gameDir.isEmpty) return;
      ref.read(configStateControllerProvider.notifier).updateNamsNow(
            gameDir,
            field.key,
            v,
            section: field.section,
          );
    }

    final hideOverlay = valueOf(NamsFields.hideSubtitleOverlay);

    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
        boxShadow: [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 4,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: double.infinity,
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.cardPaddingH(context),
              vertical: AppSizes.cardPaddingV(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.surfaceMedium,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(AppSizes.borderRadius(context)),
                topRight: Radius.circular(AppSizes.borderRadius(context)),
              ),
            ),
            child: Text(
              l10n.cutsceneSubtitleCard,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
                letterSpacing: 1.0,
              ),
            ),
          ),
          Padding(
            padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                ConfigFieldBool(
                  label: NamsFields.hideSubtitleOverlay.label(l10n),
                  tooltip: NamsFields.hideSubtitleOverlay.tooltip?.call(l10n),
                  value: hideOverlay,
                  onChanged: (v) => set(NamsFields.hideSubtitleOverlay, v),
                ),
                if (hideOverlay) ...[
                  ConfigFieldBool(
                    label: NamsFields.keepSubtitleText.label(l10n),
                    tooltip: NamsFields.keepSubtitleText.tooltip?.call(l10n),
                    value: valueOf(NamsFields.keepSubtitleText),
                    onChanged: (v) => set(NamsFields.keepSubtitleText, v),
                  ),
                  ConfigFieldBool(
                    label: NamsFields.hideSubtitleInEvents.label(l10n),
                    tooltip:
                        NamsFields.hideSubtitleInEvents.tooltip?.call(l10n),
                    value: valueOf(NamsFields.hideSubtitleInEvents),
                    onChanged: (v) => set(NamsFields.hideSubtitleInEvents, v),
                  ),
                ],
                ConfigFieldBool(
                  label: NamsFields.solidLetterboxBars.label(l10n),
                  tooltip: NamsFields.solidLetterboxBars.tooltip?.call(l10n),
                  value: valueOf(NamsFields.solidLetterboxBars),
                  onChanged: (v) => set(NamsFields.solidLetterboxBars, v),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
