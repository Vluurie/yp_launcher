import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/providers/mod_names_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/mods/mod_kind_badge.dart';
import 'package:yp_launcher/widgets/mods/mod_naming.dart';

class ModListRow extends ConsumerStatefulWidget {
  final InstalledMod mod;
  final bool selected;
  final VoidCallback onTap;

  const ModListRow({
    super.key,
    required this.mod,
    required this.selected,
    required this.onTap,
  });

  @override
  ConsumerState<ModListRow> createState() => _ModListRowState();
}

class _ModListRowState extends ConsumerState<ModListRow> {
  bool _hovered = false;

  String get _relPath => 'mods/${widget.mod.id}';

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final m = widget.mod;
    final disabled = ref.watch(disabledModsStateControllerProvider).isDisabled(_relPath);
    final customName =
        ref.watch(modNamesStateControllerProvider).customNameOf(m.id);
    final displayName = customName ?? m.displayName;
    final showDataChip = m.kind == ModKind.native && m.hasDataOverlay;
    final showCompatChip = m.data?.hasCompatConfig == true;

    final row = MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Container(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.cardPaddingH(context),
            vertical: AppSizes.cardPaddingV(context),
          ),
          decoration: BoxDecoration(
            color: widget.selected
                ? AppColors.accentPrimary.withValues(alpha: 0.10)
                : _hovered
                    ? AppColors.surfaceLight
                    : Colors.transparent,
            border: widget.selected
                ? Border(left: BorderSide(color: AppColors.accentPrimary, width: 2))
                : null,
          ),
          child: Opacity(
            opacity: disabled ? 0.45 : 1.0,
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    displayName,
                    style: TextStyle(
                      fontSize: AppSizes.fontMD(context),
                      color: AppColors.textPrimary,
                      fontWeight: widget.selected ? FontWeight.w600 : FontWeight.normal,
                      decoration: disabled ? TextDecoration.lineThrough : null,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                if (m.manifest?.version != null) ...[
                  Text(
                    m.manifest!.version!,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                    ),
                  ),
                  SizedBox(width: AppSizes.spacingMD(context)),
                ],
                ModKindBadge(kind: m.kind),
                if (showDataChip)
                  ModCompatChip(label: l10n.modDataChip, tooltip: l10n.modDataChipTooltip),
                if (showCompatChip)
                  ModCompatChip(label: l10n.modCompatChip, tooltip: l10n.modCompatChipTooltip),
                if (m.hasWarnings) ...[
                  SizedBox(width: AppSizes.spacingMD(context)),
                  Icon(
                    Icons.warning_amber_rounded,
                    size: AppSizes.iconSM(context),
                    color: AppColors.warning,
                  ),
                ],
                SizedBox(width: AppSizes.spacingMD(context)),
                if (_hovered || widget.selected)
                  HoverIconButton(
                    tooltip: l10n.modRename,
                    bordered: false,
                    padding: EdgeInsets.all(AppSizes.paddingXS(context)),
                    icon: Icon(
                      Icons.edit_outlined,
                      size: AppSizes.iconSM(context),
                      color: AppColors.textMuted,
                    ),
                    onTap: () => _onRename(l10n, customName),
                  ),
                _DisableToggle(
                  disabled: disabled,
                  onTap: () {
                    final gameDir = ref
                        .read(appStateControllerProvider)
                        .selectedDirectory;
                    ref
                        .read(disabledModsStateControllerProvider.notifier)
                        .setDisabled(gameDir, _relPath, !disabled);
                  },
                ),
              ],
            ),
          ),
        ),
      ),
    );

    return row;
  }

  Future<void> _onRename(AppLocalizations l10n, String? customName) async {
    final mod = widget.mod;
    final name = await showModNamingDialog(
      context,
      initial: customName ?? mod.displayName,
      title: l10n.modRenameDialogTitle,
    );
    if (name == null || !mounted) return;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    await ref.read(modNamesStateControllerProvider.notifier).rename(
          gameDir,
          mod.id,
          name == mod.displayName ? null : name,
        );
  }
}

class _DisableToggle extends StatelessWidget {
  final bool disabled;
  final VoidCallback onTap;

  const _DisableToggle({required this.disabled, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Tooltip(
      message: disabled ? l10n.modDisabledTooltip : l10n.modEnableTooltip,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context) / 2),
        child: Padding(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.chipPaddingH(context),
            vertical: AppSizes.chipPaddingV(context),
          ),
          child: Icon(
            disabled ? Icons.toggle_off_outlined : Icons.toggle_on,
            size: AppSizes.iconLG(context) + 4,
            color: disabled ? AppColors.textMuted : AppColors.success,
          ),
        ),
      ),
    );
  }
}
