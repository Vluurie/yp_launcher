import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/bundled_link_chip.dart';
import 'package:yp_launcher/widgets/hover_button.dart';
import 'package:yp_launcher/widgets/mods/mod_kind_badge.dart';

class ModDetailPanel extends ConsumerWidget {
  final InstalledMod? mod;
  final void Function(InstalledMod mod) onUninstall;

  const ModDetailPanel({
    super.key,
    required this.mod,
    required this.onUninstall,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    if (mod == null) {
      return Container(
        alignment: Alignment.center,
        padding: EdgeInsets.all(AppSizes.paddingMD(context)),
        child: Text(
          l10n.modDetailNoSelection,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: AppColors.textMuted,
            fontStyle: FontStyle.italic,
          ),
          textAlign: TextAlign.center,
        ),
      );
    }
    final m = mod!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(
            AppSizes.cardPaddingH(context),
            AppSizes.cardPaddingH(context),
            AppSizes.cardPaddingH(context),
            AppSizes.spacingMD(context),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _header(context, m),
              SizedBox(height: AppSizes.spacingMD(context)),
              _actions(context, ref, m, l10n),
            ],
          ),
        ),
        Container(height: 1, color: AppColors.borderLight),
        Expanded(
          child: SingleChildScrollView(
            padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _meta(context, m, l10n),
                if (m.native != null) ...[
                  SizedBox(height: AppSizes.paddingMD(context)),
                  _nativeSection(context, m.native!, l10n),
                ],
                if (m.data != null) ...[
                  SizedBox(height: AppSizes.paddingMD(context)),
                  _dataSection(context, m.data!, l10n),
                ],
                if (m.bundledTexturePacks.isNotEmpty) ...[
                  SizedBox(height: AppSizes.paddingMD(context)),
                  _bundledTexturesSection(context, ref, m, l10n),
                ],
                if (m.bundledCutscenes.isNotEmpty) ...[
                  SizedBox(height: AppSizes.paddingMD(context)),
                  _bundledCutscenesSection(context, ref, m, l10n),
                ],
                if (m.manifest?.requires.isNotEmpty == true ||
                    m.manifest?.requiresPlugins.isNotEmpty == true) ...[
                  SizedBox(height: AppSizes.paddingMD(context)),
                  _requiresSection(context, m, l10n),
                ],
                if (m.conflicts.isNotEmpty) ...[
                  SizedBox(height: AppSizes.paddingMD(context)),
                  _conflictsSection(context, m, l10n),
                ],
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _header(BuildContext context, InstalledMod m) {
    return Row(
      children: [
        Expanded(
          child: Tooltip(
            message: m.rootPath,
            child: Text(
              m.displayName,
              style: TextStyle(
                fontSize: AppSizes.fontLG(context),
                fontWeight: FontWeight.bold,
                color: AppColors.textPrimary,
              ),
            ),
          ),
        ),
        SizedBox(width: AppSizes.spacingMD(context)),
        ModKindBadge(kind: m.kind, dense: false),
      ],
    );
  }

  Widget _meta(BuildContext context, InstalledMod m, AppLocalizations l10n) {
    final manifest = m.manifest;
    final lines = <Widget>[];
    if (manifest?.author != null) {
      lines.add(_kv(context, l10n.modAuthor, manifest!.author!));
    }
    if (manifest?.version != null) {
      lines.add(_kv(context, l10n.modVersion, manifest!.version!));
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: lines,
    );
  }

  Widget _kv(BuildContext context, String k, String v) {
    return Padding(
      padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: AppSizes.configInputWidth(context),
            child: Text(
              k,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
          ),
          Expanded(
            child: SelectableText(
              v,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _section(BuildContext context, String title, Widget child) {
    return Container(
      width: double.infinity,
      padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontWeight: FontWeight.bold,
              color: AppColors.accentPrimary,
              letterSpacing: 0.5,
            ),
          ),
          SizedBox(height: AppSizes.spacingSM(context)),
          child,
        ],
      ),
    );
  }

  Widget _nativeSection(BuildContext context, NativeSummary n, AppLocalizations l10n) {
    final lines = <Widget>[];
    final entries = n.bundlesByKind.entries.toList()
      ..sort((a, b) => a.key.compareTo(b.key));
    for (final e in entries) {
      lines.add(Text(
        '${e.key}: ${e.value}',
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textSecondary,
        ),
      ));
    }
    if (lines.isEmpty) {
      lines.add(Text(
        l10n.modCountFiles(n.totalEntityFiles),
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textSecondary,
        ),
      ));
    }
    return _section(context, l10n.modNativeBundles, Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: lines,
    ));
  }

  Widget _dataSection(BuildContext context, DataSummary d, AppLocalizations l10n) {
    final lines = <Widget>[];

    if (d.hasCompatConfig) {
      lines.add(Padding(
        padding: EdgeInsets.only(bottom: AppSizes.paddingSM(context)),
        child: ModCompatChip(label: l10n.modCompatChip, tooltip: l10n.modCompatChipTooltip),
      ));
    }

    if (d.players.isNotEmpty) {
      final labels = d.players.map((p) => p.label).toSet().toList();
      lines.add(Padding(
        padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
        child: Text(
          '${l10n.modDataPlayerModels}: ${labels.join(", ")}',
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textSecondary,
          ),
        ),
      ));
    }

    for (final e in d.entries) {
      if (e.category == DataCategory.player3d && d.players.isNotEmpty) continue;
      final label = _categoryLabel(e.category, e.dirName);
      lines.add(Padding(
        padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
        child: Text(
          '$label: ${l10n.modCountFiles(e.fileCount)}',
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textSecondary,
          ),
        ),
      ));
    }

    if (lines.isEmpty) {
      lines.add(Text(
        '—',
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textMuted,
        ),
      ));
    }

    return _section(context, l10n.modDataContent, Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: lines,
    ));
  }

  String _categoryLabel(DataCategory cat, String dirName) {
    switch (cat) {
      case DataCategory.player3d: return 'Player models';
      case DataCategory.weapon3d: return 'Weapons (wp)';
      case DataCategory.enemy3d: return 'Enemies (em)';
      case DataCategory.accessory3d: return 'Accessories / items (et)';
      case DataCategory.item3d: return 'Items (it)';
      case DataCategory.worldProp3d: return 'World props ($dirName)';
      case DataCategory.modelVariant3d: return 'Model variants (um)';
      case DataCategory.map3d: return 'Map ($dirName)';
      case DataCategory.scripting: return 'Scripts ($dirName)';
      case DataCategory.localization: return 'Localization ($dirName)';
      case DataCategory.effects: return 'Effects ($dirName)';
      case DataCategory.ui: return 'UI ($dirName)';
      case DataCategory.misc: return 'Misc textures (misctex)';
      case DataCategory.cutscenes: return 'Cutscenes (movie)';
      case DataCategory.audio: return 'Audio (sound)';
      case DataCategory.archive: return 'CPK archives';
      case DataCategory.other: return 'Other ($dirName)';
    }
  }

  Widget _bundledTexturesSection(
    BuildContext context,
    WidgetRef ref,
    InstalledMod m,
    AppLocalizations l10n,
  ) {
    return _section(
      context,
      l10n.modBundledTexturesLabel,
      Wrap(
        spacing: AppSizes.spacingMD(context),
        runSpacing: AppSizes.spacingSM(context),
        children: m.bundledTexturePacks
            .map((name) => BundledLinkChip(
                  icon: Icons.texture,
                  label: BundledLinkChip.shortenLabel(name),
                  tooltip: '$name\n\nOpen in Textures tab',
                  onTap: () {
                    ref.read(pendingTabSelectionProvider.notifier).state =
                        TabSelectionRequest(tabIndex: 3, key: name);
                    ref.read(activeTabProvider.notifier).state = 3;
                  },
                ))
            .toList(),
      ),
    );
  }

  Widget _bundledCutscenesSection(
    BuildContext context,
    WidgetRef ref,
    InstalledMod m,
    AppLocalizations l10n,
  ) {
    return _section(
      context,
      l10n.modBundledCutscenesLabel,
      Wrap(
        spacing: AppSizes.spacingMD(context),
        runSpacing: AppSizes.spacingSM(context),
        children: m.bundledCutscenes
            .map((name) => BundledLinkChip(
                  icon: Icons.movie_creation_outlined,
                  label: name,
                  tooltip: 'Open in Cutscenes tab',
                  onTap: () {
                    ref.read(pendingTabSelectionProvider.notifier).state =
                        TabSelectionRequest(tabIndex: 7, key: name);
                    ref.read(activeTabProvider.notifier).state = 7;
                  },
                ))
            .toList(),
      ),
    );
  }

  Widget _requiresSection(BuildContext context, InstalledMod m, AppLocalizations l10n) {
    final lines = <Widget>[];
    final missing = m.requiresMissing.toSet();
    final reqs = m.manifest?.requires ?? const <String>[];
    if (reqs.isNotEmpty) {
      lines.add(Wrap(
        spacing: AppSizes.spacingMD(context),
        runSpacing: AppSizes.spacingSM(context),
        children: reqs.map((id) {
          final isMissing = missing.contains(id);
          return Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.chipPaddingH(context),
              vertical: AppSizes.chipPaddingV(context),
            ),
            decoration: BoxDecoration(
              color: isMissing
                  ? AppColors.error.withValues(alpha: 0.15)
                  : AppColors.surfaceLight,
              borderRadius: BorderRadius.circular(AppSizes.borderRadius(context) / 2),
              border: Border.all(
                color: isMissing
                    ? AppColors.error.withValues(alpha: 0.5)
                    : AppColors.borderLight,
              ),
            ),
            child: Text(
              isMissing ? '$id (${l10n.modRequiresMissing})' : id,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: isMissing ? AppColors.error : AppColors.textSecondary,
                fontWeight: isMissing ? FontWeight.w600 : FontWeight.normal,
              ),
            ),
          );
        }).toList(),
      ));
    }
    final reqPlugins = m.manifest?.requiresPlugins ?? const <String>[];
    if (reqPlugins.isNotEmpty) {
      lines.add(SizedBox(height: AppSizes.spacingMD(context)));
      lines.add(Text(
        '${l10n.modRequiresPluginsLabel}: ${reqPlugins.join(", ")}',
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          color: AppColors.textMuted,
        ),
      ));
    }
    return _section(context, l10n.modRequiresLabel, Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: lines,
    ));
  }

  Widget _conflictsSection(BuildContext context, InstalledMod m, AppLocalizations l10n) {
    final lines = <Widget>[];
    for (final c in m.conflicts) {
      lines.add(Padding(
        padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
        child: Text(
          l10n.modConflictOverlapFile(c.otherModId, c.detail),
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.warning,
          ),
        ),
      ));
    }
    return _section(context, l10n.modConflictsLabel, Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: lines,
    ));
  }

  Widget _actions(
    BuildContext context,
    WidgetRef ref,
    InstalledMod m,
    AppLocalizations l10n,
  ) {
    final relPath = 'mods/${m.id}';
    final disabled =
        ref.watch(disabledModsStateControllerProvider).isDisabled(relPath);
    final iconSize = AppSizes.iconSM(context);
    return Wrap(
      spacing: AppSizes.spacingSM(context),
      runSpacing: AppSizes.spacingSM(context),
      children: [
        HoverIconButton(
          tooltip: l10n.modOpenFolder,
          onTap: () => _openFolder(m.rootPath),
          borderColor: AppColors.borderMedium,
          icon: Icon(
            Icons.folder_open,
            size: iconSize,
            color: AppColors.textSecondary,
          ),
        ),
        HoverIconButton(
          tooltip: disabled ? l10n.modEnable : l10n.modDisable,
          onTap: () {
            final gameDir =
                ref.read(appStateControllerProvider).selectedDirectory;
            ref
                .read(disabledModsStateControllerProvider.notifier)
                .setDisabled(gameDir, relPath, !disabled);
          },
          borderColor: disabled
              ? AppColors.borderMedium
              : AppColors.success.withValues(alpha: 0.7),
          icon: Icon(
            disabled ? Icons.toggle_off_outlined : Icons.toggle_on,
            size: iconSize,
            color: disabled ? AppColors.textMuted : AppColors.success,
          ),
        ),
        HoverIconButton(
          tooltip: l10n.modUninstall,
          onTap: () => onUninstall(m),
          borderColor: AppColors.error.withValues(alpha: 0.7),
          icon: Icon(
            Icons.delete_outline,
            size: iconSize,
            color: AppColors.error,
          ),
        ),
      ],
    );
  }

  void _openFolder(String dirPath) {
    if (Platform.isWindows) {
      Process.run('explorer', [dirPath]);
    } else if (Platform.isMacOS) {
      Process.run('open', [dirPath]);
    } else {
      Process.run('xdg-open', [dirPath]);
    }
  }
}
