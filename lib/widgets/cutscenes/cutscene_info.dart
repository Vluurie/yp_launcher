import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/header_info_icon.dart';

class CutsceneInfoCard extends StatelessWidget {
  const CutsceneInfoCard({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppColors.borderLight),
        boxShadow: const [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 4,
            offset: Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
            decoration: const BoxDecoration(
              color: AppColors.surfaceMedium,
              borderRadius: BorderRadius.only(
                topLeft: Radius.circular(8),
                topRight: Radius.circular(8),
              ),
            ),
            child: Text(
              AppLocalizations.of(context)!.cardHowItWorks,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
                letterSpacing: 1.0,
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  AppLocalizations.of(context)!.cutsceneHowItWorks1,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  AppLocalizations.of(context)!.cutsceneHowItWorks2,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  AppLocalizations.of(context)!.cutsceneHowItWorks3,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 12),
                Text(
                  AppLocalizations.of(context)!.cardStructure,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    fontWeight: FontWeight.bold,
                    color: AppColors.accentPrimary,
                    letterSpacing: 0.5,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  AppLocalizations.of(context)!.cutsceneStructurePath,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    height: 1.6,
                    fontFamily: 'monospace',
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  AppLocalizations.of(context)!.cutsceneFolderNameLimit,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class CutsceneHeader extends StatelessWidget {
  final bool installing;

  const CutsceneHeader({
    super.key,
    required this.installing,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.cardPaddingH(context),
        vertical: AppSizes.cardPaddingV(context),
      ),
      decoration: const BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(bottom: BorderSide(color: AppColors.borderLight)),
      ),
      child: Row(
        children: [
          Text(
            l10n.headerCutscenes,
            style: TextStyle(
              fontSize: AppSizes.fontXL(context),
              fontWeight: FontWeight.bold,
              color: AppColors.textPrimary,
              letterSpacing: 1.0,
            ),
          ),
          Consumer(
            builder: (ctx, ref, _) {
              final gameDir =
                  ref.watch(appStateControllerProvider).selectedDirectory;
              return HeaderInfoIcon(
                tooltip: l10n.tooltipCutscenesLocation,
                revealPath: p.join(gameDir, 'nams', 'cutscenes'),
                isFile: false,
              );
            },
          ),
          const Spacer(),
          Consumer(
            builder: (ctx, ref, _) {
              final cutscene = (ref
                          .watch(configStateControllerProvider)
                          .namsValues['cutscene']
                      as Map<String, dynamic>?) ??
                  const {};
              final hd = cutscene['hd_cutscenes'] == true;
              final h264 = cutscene['enable_h264'] == true;
              return Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  _StatusChip(
                    label: l10n.cutsceneStatusHd,
                    on: hd,
                    tooltip: l10n.cutsceneStatusHdTooltip,
                  ),
                  SizedBox(width: AppSizes.spacingSM(context)),
                  _StatusChip(
                    label: l10n.cutsceneStatusH264,
                    on: h264,
                    tooltip: l10n.cutsceneStatusH264Tooltip,
                  ),
                ],
              );
            },
          ),
        ],
      ),
    );
  }
}

class _StatusChip extends StatelessWidget {
  final String label;
  final bool on;
  final String tooltip;

  const _StatusChip({
    required this.label,
    required this.on,
    required this.tooltip,
  });

  @override
  Widget build(BuildContext context) {
    final color = on ? AppColors.success : AppColors.textMuted;
    return Tooltip(
      message: tooltip,
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.chipPaddingH(context),
          vertical: AppSizes.chipPaddingV(context),
        ),
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.12),
          borderRadius:
              BorderRadius.circular(AppSizes.borderRadius(context) / 2),
          border: Border.all(color: color.withValues(alpha: 0.4)),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              on ? Icons.check_circle : Icons.remove_circle_outline,
              size: AppSizes.iconSM(context),
              color: color,
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            Text(
              label,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                fontWeight: FontWeight.bold,
                color: color,
                letterSpacing: 0.4,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class CutsceneMigrationBanner extends StatelessWidget {
  final List<String> directOverrides;

  const CutsceneMigrationBanner({
    super.key,
    required this.directOverrides,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
      decoration: BoxDecoration(
        color: AppColors.warning.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.warning.withValues(alpha: 0.4)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.warning_amber, size: 20, color: AppColors.warning),
          SizedBox(width: AppSizes.paddingMD(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.cutsceneMigrationTitle,
                  style: TextStyle(
                    fontSize: AppSizes.fontMD(context),
                    fontWeight: FontWeight.bold,
                    color: AppColors.warning,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  l10n.cutsceneMigrationBannerBody(directOverrides.length),
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                    height: 1.4,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
