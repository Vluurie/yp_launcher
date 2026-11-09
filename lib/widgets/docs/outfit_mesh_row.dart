import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/docs/outfit_builder_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';

class OutfitMeshRow extends StatefulWidget {
  final String name;
  final MeshRule rule;
  final OutfitCharacter character;
  final bool shown;
  final ValueChanged<MeshRule> onChanged;
  final VoidCallback onToggleShown;

  const OutfitMeshRow({
    super.key,
    required this.name,
    required this.rule,
    required this.character,
    required this.shown,
    required this.onChanged,
    required this.onToggleShown,
  });

  @override
  State<OutfitMeshRow> createState() => _OutfitMeshRowState();
}

class _OutfitMeshRowState extends State<OutfitMeshRow> {
  bool _hovered = false;
  bool _expanded = false;

  List<MeshVisibility> get _options => MeshVisibility.values
      .where((v) => v.appliesTo(widget.character))
      .toList();

  bool get _hasExtras =>
      widget.character.hasWig || true;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final rule = widget.rule;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 120),
        margin: const EdgeInsets.only(bottom: 4),
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.paddingSM(context),
          vertical: AppSizes.paddingXS(context),
        ),
        decoration: BoxDecoration(
          color: _hovered
              ? AppColors.accentPrimary.withValues(alpha: 0.06)
              : AppColors.surfaceMedium,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: rule.isDefault
                ? AppColors.borderLight
                : AppColors.accentPrimary.withValues(alpha: 0.35),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Tooltip(
                  message: l10n.outfitMeshPreviewToggle,
                  child: _EyeButton(
                    shown: widget.shown,
                    onTap: widget.onToggleShown,
                  ),
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                Expanded(
                  child: Text(
                    widget.name,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      fontFamily: AppSizes.monoFamily,
                      fontFamilyFallback: AppSizes.monoFallback,
                      color: AppColors.textPrimary,
                    ),
                  ),
                ),
                SizedBox(width: AppSizes.spacingSM(context)),
                AppDropdown<MeshVisibility>(
                  value: rule.visibility,
                  items: _options,
                  itemLabel: (v) => _label(l10n, v),
                  onChanged: (v) =>
                      widget.onChanged(rule.copyWith(visibility: v)),
                  maxWidth: 190,
                  minWidth: 190,
                ),
                if (_hasExtras)
                  _MoreButton(
                    label: l10n.outfitMeshMore,
                    expanded: _expanded,
                    marked: rule.hideOnSelfDestruct ||
                        rule.tintedByHairspray ||
                        rule.replacedByWig,
                    onTap: () => setState(() => _expanded = !_expanded),
                  ),
              ],
            ),
            if (_expanded) _extras(context, l10n, rule),
          ],
        ),
      ),
    );
  }

  Widget _extras(
    BuildContext context,
    AppLocalizations l10n,
    MeshRule rule,
  ) {
    return Padding(
      padding: EdgeInsets.only(
        left: AppSizes.paddingLG(context),
        top: AppSizes.spacingSM(context),
        bottom: AppSizes.spacingSM(context),
      ),
      child: Wrap(
        spacing: AppSizes.spacingMD(context),
        runSpacing: AppSizes.spacingSM(context),
        children: [
          _Extra(
            label: l10n.outfitMeshHideOnSelfDestruct,
            value: rule.hideOnSelfDestruct,
            onChanged: (v) =>
                widget.onChanged(rule.copyWith(hideOnSelfDestruct: v)),
          ),
          _Extra(
            label: l10n.outfitMeshHairsprayTint,
            value: rule.tintedByHairspray,
            onChanged: (v) =>
                widget.onChanged(rule.copyWith(tintedByHairspray: v)),
          ),
          if (widget.character.hasWig)
            _Extra(
              label: l10n.outfitMeshReplacedByWig,
              value: rule.replacedByWig,
              onChanged: (v) =>
                  widget.onChanged(rule.copyWith(replacedByWig: v)),
            ),
        ],
      ),
    );
  }

  String _label(AppLocalizations l10n, MeshVisibility v) {
    return switch (v) {
      MeshVisibility.always => l10n.outfitMeshAlways,
      MeshVisibility.combatOnly => l10n.outfitMeshCombatOnly,
      MeshVisibility.outOfCombatOnly => l10n.outfitMeshOutOfCombatOnly,
      MeshVisibility.eyemaskOnly => l10n.outfitMeshEyemaskOnly,
      MeshVisibility.noEyemaskOnly => l10n.outfitMeshNoEyemaskOnly,
      MeshVisibility.featherOnly => l10n.outfitMeshFeatherOnly,
      MeshVisibility.brokenOnly => l10n.outfitMeshBrokenOnly,
      MeshVisibility.never => l10n.outfitMeshNever,
    };
  }
}

class _EyeButton extends StatefulWidget {
  final bool shown;
  final VoidCallback onTap;

  const _EyeButton({required this.shown, required this.onTap});

  @override
  State<_EyeButton> createState() => _EyeButtonState();
}

class _EyeButtonState extends State<_EyeButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        behavior: HitTestBehavior.opaque,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.all(5),
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.15)
                : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
          ),
          child: Icon(
            widget.shown ? Icons.visibility : Icons.visibility_off_outlined,
            size: AppSizes.iconSM(context),
            color: widget.shown
                ? AppColors.textSecondary
                : AppColors.textMuted,
          ),
        ),
      ),
    );
  }
}

class _MoreButton extends StatefulWidget {
  final String label;
  final bool expanded;
  final bool marked;
  final VoidCallback onTap;

  const _MoreButton({
    required this.label,
    required this.expanded,
    required this.marked,
    required this.onTap,
  });

  @override
  State<_MoreButton> createState() => _MoreButtonState();
}

class _MoreButtonState extends State<_MoreButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        behavior: HitTestBehavior.opaque,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.all(5),
          margin: const EdgeInsets.only(left: 4),
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.15)
                : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                widget.label,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: widget.marked
                      ? AppColors.accentPrimary
                      : _hovered
                          ? AppColors.textSecondary
                          : AppColors.textMuted,
                ),
              ),
              const SizedBox(width: 2),
              Icon(
                widget.expanded ? Icons.expand_less : Icons.expand_more,
                size: AppSizes.iconSM(context),
                color: widget.marked
                    ? AppColors.accentPrimary
                    : AppColors.textMuted,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Extra extends StatefulWidget {
  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;

  const _Extra({
    required this.label,
    required this.value,
    required this.onChanged,
  });

  @override
  State<_Extra> createState() => _ExtraState();
}

class _ExtraState extends State<_Extra> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final on = widget.value;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: () => widget.onChanged(!on),
        behavior: HitTestBehavior.opaque,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              on ? Icons.check_box : Icons.check_box_outline_blank,
              size: AppSizes.iconSM(context),
              color: on
                  ? AppColors.accentPrimary
                  : _hovered
                      ? AppColors.textSecondary
                      : AppColors.textMuted,
            ),
            const SizedBox(width: 5),
            Text(
              widget.label,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: on ? AppColors.textPrimary : AppColors.textMuted,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
