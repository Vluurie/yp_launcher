import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/mod_grouping.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

String modGroupLabel(ModGroupKind kind, AppLocalizations l10n) {
  return switch (kind) {
    ModGroupKind.mixed => l10n.modGroupMixed,
    ModGroupKind.outfit2b => l10n.modGroup2b,
    ModGroupKind.outfit9s => l10n.modGroup9s,
    ModGroupKind.outfitA2 => l10n.modGroupA2,
    ModGroupKind.outfitOther => l10n.modGroupOtherOutfits,
    ModGroupKind.weapon => l10n.modGroupWeapons,
    ModGroupKind.accessory => l10n.modGroupAccessories,
    ModGroupKind.item => l10n.modGroupItems,
    ModGroupKind.enemy => l10n.modGroupEnemies,
    ModGroupKind.worldProp => l10n.modGroupWorldProps,
    ModGroupKind.modelVariant => l10n.modGroupModelVariants,
    ModGroupKind.map => l10n.modGroupMaps,
    ModGroupKind.effect => l10n.modGroupEffects,
    ModGroupKind.scripting => l10n.modGroupScripting,
    ModGroupKind.localization => l10n.modGroupLocalization,
    ModGroupKind.ui => l10n.modGroupUi,
    ModGroupKind.cutscene => l10n.modGroupCutscenes,
    ModGroupKind.audio => l10n.modGroupAudio,
    ModGroupKind.texture => l10n.modGroupTextures,
    ModGroupKind.misc => l10n.modGroupMisc,
    ModGroupKind.archive => l10n.modGroupArchives,
    ModGroupKind.native => l10n.modGroupNative,
    ModGroupKind.other => l10n.modGroupOther,
  };
}

IconData modGroupIcon(ModGroupKind kind) {
  return switch (kind) {
    ModGroupKind.mixed => Icons.dashboard_customize_outlined,
    ModGroupKind.outfit2b ||
    ModGroupKind.outfit9s ||
    ModGroupKind.outfitA2 ||
    ModGroupKind.outfitOther =>
      Icons.checkroom_outlined,
    ModGroupKind.weapon => Icons.gavel_outlined,
    ModGroupKind.accessory => Icons.diamond_outlined,
    ModGroupKind.item => Icons.inventory_2_outlined,
    ModGroupKind.enemy => Icons.bug_report_outlined,
    ModGroupKind.worldProp => Icons.chair_outlined,
    ModGroupKind.modelVariant => Icons.category_outlined,
    ModGroupKind.map => Icons.map_outlined,
    ModGroupKind.effect => Icons.auto_awesome_outlined,
    ModGroupKind.scripting => Icons.code_outlined,
    ModGroupKind.localization => Icons.translate_outlined,
    ModGroupKind.ui => Icons.web_outlined,
    ModGroupKind.cutscene => Icons.movie_creation_outlined,
    ModGroupKind.audio => Icons.volume_up_outlined,
    ModGroupKind.texture => Icons.texture_outlined,
    ModGroupKind.misc => Icons.image_outlined,
    ModGroupKind.archive => Icons.archive_outlined,
    ModGroupKind.native => Icons.extension_outlined,
    ModGroupKind.other => Icons.folder_outlined,
  };
}

class ModGroupHeader extends StatefulWidget {
  final ModGroupKind kind;
  final int count;
  final bool collapsed;
  final VoidCallback onTap;

  const ModGroupHeader({
    super.key,
    required this.kind,
    required this.count,
    required this.collapsed,
    required this.onTap,
  });

  @override
  State<ModGroupHeader> createState() => _ModGroupHeaderState();
}

class _ModGroupHeaderState extends State<ModGroupHeader> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.cardPaddingH(context),
            vertical: AppSizes.cardPaddingV(context),
          ),
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.08)
                : AppColors.surfaceMedium.withValues(alpha: 0.6),
            border: Border(
              bottom: BorderSide(
                color: AppColors.borderLight.withValues(alpha: 0.4),
              ),
            ),
          ),
          child: Row(
            children: [
              Icon(
                widget.collapsed ? Icons.chevron_right : Icons.expand_more,
                size: AppSizes.iconMD(context),
                color: _hovered ? AppColors.accentPrimary : AppColors.textMuted,
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Icon(
                modGroupIcon(widget.kind),
                size: AppSizes.iconSM(context),
                color: AppColors.accentPrimary,
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Flexible(
                child: Text(
                  modGroupLabel(widget.kind, l10n),
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    fontWeight: FontWeight.bold,
                    color: AppColors.accentPrimary,
                    letterSpacing: 1.0,
                  ),
                ),
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
                decoration: BoxDecoration(
                  color: AppColors.borderMedium.withValues(alpha: 0.4),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  '${widget.count}',
                  style: const TextStyle(
                    fontSize: 11,
                    color: AppColors.textMuted,
                    fontWeight: FontWeight.w500,
                    height: 1.2,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
