import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ModKindBadge extends StatelessWidget {
  final ModKind kind;
  final bool dense;

  const ModKindBadge({super.key, required this.kind, this.dense = true});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final (label, color, tooltip) = switch (kind) {
      ModKind.native => (l10n.modKindNative, AppColors.accentPrimary, l10n.modKindNativeTooltip),
      ModKind.data => (l10n.modKindData, AppColors.textMuted, l10n.modKindDataTooltip),
      ModKind.unknown => (l10n.modKindUnknown, AppColors.error, l10n.modKindUnknownTooltip),
    };
    return Tooltip(
      message: tooltip,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.15),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: color.withValues(alpha: 0.5)),
        ),
        child: Text(
          label,
          style: TextStyle(
            fontSize: dense ? 11 : AppSizes.fontSM(context),
            fontWeight: FontWeight.bold,
            color: color,
            letterSpacing: 0.5,
            height: 1.2,
          ),
        ),
      ),
    );
  }
}

class ModCompatChip extends StatelessWidget {
  final String label;
  final String? tooltip;
  const ModCompatChip({super.key, required this.label, this.tooltip});

  @override
  Widget build(BuildContext context) {
    final chip = Container(
      margin: EdgeInsets.only(left: AppSizes.spacingSM(context)),
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
      decoration: BoxDecoration(
        color: AppColors.borderMedium.withValues(alpha: 0.4),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: 11,
          color: AppColors.textMuted,
          fontWeight: FontWeight.w500,
          letterSpacing: 0.3,
          height: 1.2,
        ),
      ),
    );
    if (tooltip == null) return chip;
    return Tooltip(
      message: tooltip!,
      child: chip,
    );
  }
}
