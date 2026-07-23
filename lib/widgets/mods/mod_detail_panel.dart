import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/default_mods_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/providers/mod_names_state.dart';
import 'package:yp_launcher/providers/mods_state.dart';
import 'package:yp_launcher/services/default_mods_service.dart';
import 'package:yp_launcher/services/reveal_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
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
                  _dataSection(context, ref, m, m.data!, l10n),
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
                  _conflictsSection(context, ref, m, l10n),
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

  Widget _dataSection(
    BuildContext context,
    WidgetRef ref,
    InstalledMod mod,
    DataSummary d,
    AppLocalizations l10n,
  ) {
    final lines = <Widget>[];

    if (d.hasCompatConfig) {
      lines.add(Padding(
        padding: EdgeInsets.only(bottom: AppSizes.paddingSM(context)),
        child: ModCompatChip(label: l10n.modCompatChip, tooltip: l10n.modCompatChipTooltip),
      ));
    }

    if (d.players.isNotEmpty) {
      final defaults = ref.watch(defaultModsStateControllerProvider);
      final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
      final seen = <String>{};
      lines.add(Padding(
        padding: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
        child: Text(
          l10n.modDataPlayerModels,
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textSecondary,
          ),
        ),
      ));
      for (final p in d.players) {
        final stem = p.fileName.contains('.')
            ? p.fileName.substring(0, p.fileName.lastIndexOf('.'))
            : p.fileName;
        if (!seen.add(stem)) continue;
        final relPath = 'mods/${mod.id}/pl/$stem';
        final kind = defaults.kindOf(relPath);
        final target = DefaultModsService.kindFor(
          stem,
          hasOutfitConfig: d.hasOutfitConfig,
        );
        final choices = d.outfitsByStem[stem] ?? const <OutfitChoice>[];
        lines.add(_StemRow(
          label: p.label,
          kind: kind,
          target: target,
          choices: target == DefaultKind.outfitConfig ? choices : const [],
          outfitId: defaults.outfitIdOf(relPath),
          onTap: () async {
            final notifier =
                ref.read(defaultModsStateControllerProvider.notifier);
            if (kind != null) {
              await notifier.setDefault(gameDir, relPath, null);
              return;
            }
            final replaced = defaults.wouldReplace(relPath, target);
            if (replaced != null) {
              final ok = await _confirmReplace(
                context,
                model: p.label,
                current: _modLabel(ref, replaced.path),
                next: mod.displayName,
              );
              if (!ok) return;
            }
            await notifier.setDefault(
              gameDir,
              relPath,
              target,
              outfitId: DefaultModsService.initialOutfitId(target, choices),
            );
          },
          onOutfitChanged: (id) => ref
              .read(defaultModsStateControllerProvider.notifier)
              .setDefault(gameDir, relPath, target, outfitId: id),
        ));
      }
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
                  tooltip: AppLocalizations.of(
                    context,
                  )!.tooltipOpenInTexturesTab(name),
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
                  tooltip: AppLocalizations.of(
                    context,
                  )!.tooltipOpenInCutscenesTab,
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

  Widget _conflictsSection(
    BuildContext context,
    WidgetRef ref,
    InstalledMod m,
    AppLocalizations l10n,
  ) {
    final disabledState = ref.watch(disabledModsStateControllerProvider);
    final candidates = <String>{m.id, ...m.conflicts.map((c) => c.otherModId)}
        .where((id) => !disabledState.isDisabled('mods/$id'))
        .toList()
      ..sort();

    if (candidates.length < 2) return const SizedBox.shrink();

    final files = <String>{for (final c in m.conflicts) c.detail};

    Future<void> keepOnly(String winner) async {
      final gameDir = ref.read(appStateControllerProvider).selectedDirectory;
      final notifier = ref.read(disabledModsStateControllerProvider.notifier);
      for (final id in candidates) {
        if (id == winner) continue;
        await notifier.setDisabled(gameDir, 'mods/$id', true);
      }
    }

    return _section(
      context,
      l10n.modConflictsLabel,
      Row(
        children: [
          Icon(
            Icons.warning_amber_rounded,
            size: AppSizes.iconSM(context),
            color: AppColors.warning,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Text(
              l10n.modConflictPickBody(candidates.length, files.length),
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textSecondary,
              ),
            ),
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          HoverButton(
            label: l10n.modConflictResolve,
            color: AppColors.warning,
            onTap: () => _showConflictDialog(
              context,
              candidates: candidates,
              selfId: m.id,
              fileCount: files.length,
              onKeep: keepOnly,
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _showConflictDialog(
    BuildContext context, {
    required List<String> candidates,
    required String selfId,
    required int fileCount,
    required Future<void> Function(String) onKeep,
  }) {
    final l10n = AppLocalizations.of(context)!;
    return showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.surfaceMedium,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(ctx)),
          side: BorderSide(color: AppColors.borderLight),
        ),
        title: Text(
          l10n.modConflictDialogTitle,
          style: TextStyle(
            fontSize: AppSizes.fontMD(ctx),
            fontWeight: FontWeight.bold,
            color: AppColors.textPrimary,
          ),
        ),
        content: SizedBox(
          width: (MediaQuery.of(ctx).size.width * 0.5).clamp(420.0, 620.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                padding: EdgeInsets.all(AppSizes.paddingSM(ctx)),
                decoration: BoxDecoration(
                  color: AppColors.warning.withValues(alpha: 0.10),
                  borderRadius:
                      BorderRadius.circular(AppSizes.borderRadius(ctx) / 2),
                  border: Border.all(
                    color: AppColors.warning.withValues(alpha: 0.35),
                  ),
                ),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Icon(
                      Icons.warning_amber_rounded,
                      size: AppSizes.iconSM(ctx),
                      color: AppColors.warning,
                    ),
                    SizedBox(width: AppSizes.spacingSM(ctx)),
                    Expanded(
                      child: Text(
                        l10n.modConflictPickBody(candidates.length, fileCount),
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(ctx),
                          color: AppColors.textSecondary,
                          height: 1.4,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              SizedBox(height: AppSizes.spacingMD(ctx)),
              ConstrainedBox(
                constraints: BoxConstraints(
                  maxHeight:
                      (MediaQuery.of(ctx).size.height * 0.4).clamp(180.0, 340.0),
                ),
                child: _ConflictList(
                  candidates: candidates,
                  selfId: selfId,
                  onKeep: (id) async {
                    Navigator.of(ctx).pop();
                    await onKeep(id);
                  },
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(
              l10n.buttonCancel,
              style: TextStyle(color: AppColors.textMuted),
            ),
          ),
        ],
      ),
    );
  }

  static String _modLabel(WidgetRef ref, String entryPath) {
    final parts = entryPath.split('/');
    final id = parts.length > 1 ? parts[1] : entryPath;
    final custom = ref.read(modNamesStateControllerProvider).customNameOf(id);
    if (custom != null) return custom;
    final mods = ref.read(modsStateControllerProvider).mods;
    for (final m in mods) {
      if (m.id == id) return m.displayName;
    }
    return id;
  }

  static Future<bool> _confirmReplace(
    BuildContext context, {
    required String model,
    required String current,
    required String next,
  }) async {
    final l10n = AppLocalizations.of(context)!;
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.surfaceMedium,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(ctx)),
          side: BorderSide(color: AppColors.borderLight),
        ),
        title: Text(
          l10n.modDefaultReplaceTitle,
          style: TextStyle(
            fontSize: AppSizes.fontMD(ctx),
            fontWeight: FontWeight.bold,
            color: AppColors.textPrimary,
          ),
        ),
        content: SizedBox(
          width: (MediaQuery.of(ctx).size.width * 0.5).clamp(380.0, 560.0),
          child: Text(
            l10n.modDefaultReplaceBody(model, current, next),
            style: TextStyle(
              fontSize: AppSizes.fontSM(ctx),
              color: AppColors.textSecondary,
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(
              l10n.buttonCancel,
              style: TextStyle(color: AppColors.textMuted),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              l10n.modDefaultReplaceConfirm,
              style: TextStyle(color: AppColors.accentSecondary),
            ),
          ),
        ],
      ),
    );
    return ok ?? false;
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
    revealInFileManager(dirPath);
  }
}

class _ConflictList extends StatefulWidget {
  final List<String> candidates;
  final String selfId;
  final Future<void> Function(String) onKeep;

  const _ConflictList({
    required this.candidates,
    required this.selfId,
    required this.onKeep,
  });

  @override
  State<_ConflictList> createState() => _ConflictListState();
}

class _ConflictListState extends State<_ConflictList> {
  final _controller = ScrollController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ScrollConfiguration(
      behavior: ScrollConfiguration.of(context).copyWith(scrollbars: false),
      child: Scrollbar(
        controller: _controller,
        thumbVisibility: true,
        trackVisibility: true,
        interactive: true,
        thickness: 8,
        radius: const Radius.circular(4),
        child: SingleChildScrollView(
          controller: _controller,
          padding: EdgeInsets.only(right: AppSizes.paddingMD(context)),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (final id in widget.candidates)
                _ConflictChoice(
                  id: id,
                  isSelf: id == widget.selfId,
                  onTap: () => widget.onKeep(id),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ConflictChoice extends StatefulWidget {
  final String id;
  final bool isSelf;
  final VoidCallback onTap;

  const _ConflictChoice({
    required this.id,
    required this.isSelf,
    required this.onTap,
  });

  @override
  State<_ConflictChoice> createState() => _ConflictChoiceState();
}

class _ConflictChoiceState extends State<_ConflictChoice> {
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
        behavior: HitTestBehavior.opaque,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          margin: EdgeInsets.only(bottom: AppSizes.spacingSM(context)),
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.paddingSM(context),
            vertical: AppSizes.paddingSM(context),
          ),
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentSecondary.withValues(alpha: 0.12)
                : AppColors.surfaceLight,
            borderRadius:
                BorderRadius.circular(AppSizes.borderRadius(context) / 2),
            border: Border.all(
              color: _hovered
                  ? AppColors.accentSecondary
                  : AppColors.borderLight,
            ),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Icon(
                widget.isSelf ? Icons.adjust : Icons.circle_outlined,
                size: AppSizes.iconSM(context),
                color: _hovered
                    ? AppColors.accentSecondary
                    : AppColors.textMuted,
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Expanded(
                child: Text(
                  widget.id,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: _hovered
                        ? AppColors.textPrimary
                        : AppColors.textSecondary,
                    fontWeight:
                        widget.isSelf ? FontWeight.w600 : FontWeight.normal,
                    height: 1.3,
                  ),
                ),
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              Opacity(
                opacity: _hovered ? 1 : 0,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      l10n.modConflictKeep,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.accentSecondary,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 0.5,
                      ),
                    ),
                    SizedBox(width: AppSizes.spacingSM(context)),
                    Icon(
                      Icons.arrow_forward,
                      size: AppSizes.iconSM(context),
                      color: AppColors.accentSecondary,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _StemRow extends StatelessWidget {
  final String label;
  final DefaultKind? kind;
  final DefaultKind target;
  final List<OutfitChoice> choices;
  final int? outfitId;
  final VoidCallback onTap;
  final ValueChanged<int?> onOutfitChanged;

  const _StemRow({
    required this.label,
    required this.kind,
    required this.target,
    required this.choices,
    required this.outfitId,
    required this.onTap,
    required this.onOutfitChanged,
  });

  static String labelOf(DefaultKind kind, AppLocalizations l10n) {
    switch (kind) {
      case DefaultKind.outfitBare:
        return l10n.modDefaultKindOutfitBare;
      case DefaultKind.outfitConfig:
        return l10n.modDefaultKindOutfitConfig;
      case DefaultKind.outfitAnimation:
        return l10n.modDefaultKindOutfitAnimation;
    }
  }

  static String tooltipOf(DefaultKind kind, AppLocalizations l10n) {
    switch (kind) {
      case DefaultKind.outfitBare:
        return l10n.modDefaultKindOutfitBareTooltip;
      case DefaultKind.outfitConfig:
        return l10n.modDefaultKindOutfitConfigTooltip;
      case DefaultKind.outfitAnimation:
        return l10n.modDefaultKindOutfitAnimationTooltip;
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final starred = kind != null;
    final shown = kind ?? target;
    return Padding(
      padding: EdgeInsets.only(
        left: AppSizes.paddingSM(context),
        bottom: AppSizes.paddingXS(context),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: starred
                    ? AppColors.accentSecondary
                    : AppColors.textSecondary,
                fontWeight: starred ? FontWeight.w600 : FontWeight.normal,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          if (starred && choices.length > 1)
            Padding(
              padding: EdgeInsets.only(right: AppSizes.paddingXS(context)),
              child: Tooltip(
                message: l10n.modDefaultOutfitPickTooltip,
                child: AppDropdown<int?>(
                  value: choices.any((c) => c.outfitId == outfitId)
                      ? outfitId
                      : null,
                  items: [null, ...choices.map((c) => c.outfitId)],
                  itemLabel: (id) {
                    if (id == null) return l10n.modDefaultOutfitAuto;
                    final match =
                        choices.where((c) => c.outfitId == id).toList();
                    return match.isEmpty ? '$id' : match.first.name;
                  },
                  maxWidth: 170,
                  onChanged: onOutfitChanged,
                ),
              ),
            ),
          if (starred)
            Padding(
              padding: EdgeInsets.only(right: AppSizes.paddingXS(context)),
              child: Tooltip(
                message: tooltipOf(shown, l10n),
                child: Container(
                  padding: EdgeInsets.symmetric(
                    horizontal: AppSizes.paddingXS(context),
                  ),
                  decoration: BoxDecoration(
                    color: AppColors.accentSecondary.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(3),
                  ),
                  child: Text(
                    labelOf(shown, l10n),
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.accentSecondary,
                    ),
                  ),
                ),
              ),
            ),
          HoverIconButton(
            tooltip:
                starred ? l10n.modDefaultTooltip : tooltipOf(shown, l10n),
            onTap: onTap,
            bordered: false,
            borderColor: AppColors.accentSecondary,
            padding: EdgeInsets.all(AppSizes.paddingXS(context)),
            icon: Icon(
              starred ? Icons.star : Icons.star_outline,
              size: AppSizes.iconSM(context),
              color:
                  starred ? AppColors.accentSecondary : AppColors.textMuted,
            ),
          ),
        ],
      ),
    );
  }
}
