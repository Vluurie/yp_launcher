import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/models/mod_group.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/providers/mod_groups_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
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
    final groupsData = ref.watch(modGroupsStateControllerProvider);
    final displayName = groupsData.customNameOf(m.id) ?? m.displayName;
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
                if (_hovered || widget.selected)
                  _modMenu(context, l10n, groupsData),
                SizedBox(width: AppSizes.spacingMD(context)),
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

    return Draggable<String>(
      data: widget.mod.id,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: Material(
        color: Colors.transparent,
        child: Container(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.cardPaddingH(context),
            vertical: AppSizes.cardPaddingV(context),
          ),
          decoration: BoxDecoration(
            color: AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            border: Border.all(color: AppColors.accentPrimary),
          ),
          child: Text(
            displayName,
            style: TextStyle(
              color: AppColors.textPrimary,
              fontSize: AppSizes.fontMD(context),
            ),
          ),
        ),
      ),
      childWhenDragging: Opacity(opacity: 0.4, child: row),
      child: row,
    );
  }

  Widget _modMenu(
    BuildContext context,
    AppLocalizations l10n,
    ModGroupsData groupsData,
  ) {
    return PopupMenuButton<_ModMenuAction>(
      tooltip: '',
      icon: Icon(
        Icons.more_horiz,
        size: AppSizes.iconMD(context),
        color: AppColors.textMuted,
      ),
      color: AppColors.backgroundCard,
      padding: EdgeInsets.zero,
      itemBuilder: (ctx) => [
        PopupMenuItem(
          onTap: () => WidgetsBinding.instance.addPostFrameCallback(
            (_) => _onMenu(l10n, _ModMenuAction.rename),
          ),
          child: _menuRow(ctx, Icons.edit_outlined, l10n.modRename),
        ),
        PopupMenuItem(
          onTap: () => WidgetsBinding.instance.addPostFrameCallback(
            (_) => _onMenu(l10n, _ModMenuAction.move),
          ),
          child: _menuRow(ctx, Icons.drive_file_move_outline,
              l10n.modMoveToGroup),
        ),
      ],
    );
  }

  Widget _menuRow(BuildContext context, IconData icon, String label) {
    return Row(
      children: [
        Icon(icon, size: 16, color: AppColors.accentPrimary),
        const SizedBox(width: 8),
        Flexible(
          child: Text(
            label,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              color: AppColors.textPrimary,
              fontSize: AppSizes.fontSM(context),
            ),
          ),
        ),
      ],
    );
  }

  Future<void> _onMenu(
    AppLocalizations l10n,
    _ModMenuAction action,
  ) async {
    if (!mounted) return;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    final notifier = ref.read(modGroupsStateControllerProvider.notifier);
    if (action == _ModMenuAction.rename) {
      final current = ref
          .read(modGroupsStateControllerProvider)
          .customNameOf(widget.mod.id);
      final name = await showModNamingDialog(
        context,
        initial: current ?? widget.mod.displayName,
        title: l10n.modRenameDialogTitle,
      );
      if (name == null) return;
      await notifier.renameMod(gameDir, widget.mod.id, name);
    } else {
      await _showMoveToGroupDialog(l10n);
    }
  }

  Future<void> _showMoveToGroupDialog(
    AppLocalizations l10n,
  ) async {
    if (!mounted) return;
    final groupsData = ref.read(modGroupsStateControllerProvider);
    final groups = groupsData.sortedGroups;
    final currentGroup = groupsData.groupIdOf(widget.mod.id);

    final picked = await showDialog<_GroupPick>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          l10n.modMoveToGroup,
          style: TextStyle(
            color: AppColors.accentPrimary,
            fontSize: AppSizes.fontLG(ctx),
          ),
        ),
        contentPadding: const EdgeInsets.fromLTRB(12, 16, 12, 8),
        content: SizedBox(
          width: 360,
          height: 360,
          child: Scrollbar(
            thumbVisibility: true,
            child: ListView(
              children: [
                _groupPickItem(
                  ctx,
                  icon: currentGroup == null
                      ? Icons.check
                      : Icons.folder_off_outlined,
                  label: l10n.modGroupUngrouped,
                  selected: currentGroup == null,
                  onTap: () =>
                      Navigator.of(ctx).pop(const _GroupPick(null)),
                ),
                for (final g in groups)
                  _groupPickItem(
                    ctx,
                    icon: currentGroup == g.id
                        ? Icons.check
                        : Icons.folder_outlined,
                    label: g.name,
                    selected: currentGroup == g.id,
                    onTap: () => Navigator.of(ctx).pop(_GroupPick(g.id)),
                  ),
              ],
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(
              l10n.buttonCancel,
              style: TextStyle(
                color: AppColors.textMuted,
                fontSize: AppSizes.fontSM(ctx),
              ),
            ),
          ),
        ],
      ),
    );
    if (picked == null) return;
    final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
    await ref
        .read(modGroupsStateControllerProvider.notifier)
        .assignMod(gameDir, widget.mod.id, picked.groupId);
  }

  Widget _groupPickItem(
    BuildContext context, {
    required IconData icon,
    required String label,
    required bool selected,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 11),
        child: Row(
          children: [
            Icon(
              icon,
              size: 18,
              color: selected ? AppColors.accentPrimary : AppColors.textMuted,
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Text(
                label,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  color: selected
                      ? AppColors.accentPrimary
                      : AppColors.textPrimary,
                  fontSize: AppSizes.fontMD(context),
                  fontWeight: selected ? FontWeight.w700 : FontWeight.w500,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

enum _ModMenuAction { rename, move }

class _GroupPick {
  final String? groupId;
  const _GroupPick(this.groupId);
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
