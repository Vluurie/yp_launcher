import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/bundled_link_chip.dart';
import 'package:yp_launcher/widgets/cutscenes/cutscene_isolates.dart';

class CutsceneInstalledList extends StatelessWidget {
  final List<CutsceneMod> mods;
  final void Function(CutsceneMod mod) onDelete;

  const CutsceneInstalledList({
    super.key,
    required this.mods,
    required this.onDelete,
  });

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
              AppLocalizations.of(
                context,
              )!.installedCutsceneModsCount(mods.length),
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
            child: mods.isEmpty
                ? Center(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(vertical: 20),
                      child: Text(
                        AppLocalizations.of(context)!.noCutsceneModsInstalled,
                        style: TextStyle(
                          fontSize: AppSizes.fontMD(context),
                          color: AppColors.textMuted,
                        ),
                      ),
                    ),
                  )
                : Column(children: mods.map((mod) => _buildModTile(context, mod)).toList()),
          ),
        ],
      ),
    );
  }

  Widget _buildModTile(BuildContext context, CutsceneMod mod) {
    final allValid = mod.missingOriginals.isEmpty;
    final statusColor = allValid ? AppColors.success : AppColors.warning;
    final statusIcon = allValid ? Icons.check_circle : Icons.warning_amber;
    final l10n = AppLocalizations.of(context)!;
    final statusText = allValid
        ? l10n.cutsceneUsmCount(mod.usmCount)
        : l10n.cutsceneMatchCount(mod.matchingOriginals.length, mod.usmCount);

    final resolutionText = mod.maxWidth > 0 ? '${mod.maxWidth}x${mod.maxHeight}' : '';
    final codecText = mod.hasH264 ? 'H264' : (mod.usmCount > 0 ? 'MPEG-2' : '');

    return _ModTileHover(
      mod: mod,
      statusColor: statusColor,
      statusIcon: statusIcon,
      statusText: statusText,
      resolutionText: resolutionText,
      codecText: codecText,
      allValid: allValid,
      onDelete: () => onDelete(mod),
    );
  }
}

class _ClickableName extends StatefulWidget {
  final String name;
  final String folderPath;

  const _ClickableName({required this.name, required this.folderPath});

  @override
  State<_ClickableName> createState() => _ClickableNameState();
}

class _ClickableNameState extends State<_ClickableName> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: () => Process.run('explorer.exe', [widget.folderPath]),
        child: Text(
          widget.name,
          style: TextStyle(
            fontSize: AppSizes.fontMD(context),
            color: _hovered ? AppColors.accentPrimary : AppColors.textPrimary,
            fontWeight: FontWeight.normal,
            decoration: _hovered
                ? TextDecoration.underline
                : TextDecoration.none,
            decorationColor: AppColors.accentPrimary,
          ),
          overflow: TextOverflow.ellipsis,
        ),
      ),
    );
  }
}

class _ModTileHover extends ConsumerStatefulWidget {
  final CutsceneMod mod;
  final Color statusColor;
  final IconData statusIcon;
  final String statusText;
  final String resolutionText;
  final String codecText;
  final bool allValid;
  final VoidCallback onDelete;

  const _ModTileHover({
    required this.mod,
    required this.statusColor,
    required this.statusIcon,
    required this.statusText,
    this.resolutionText = '',
    this.codecText = '',
    required this.allValid,
    required this.onDelete,
  });

  @override
  ConsumerState<_ModTileHover> createState() => _ModTileHoverState();
}

class _ModTileHoverState extends ConsumerState<_ModTileHover> {
  bool _hovered = false;
  String get _relPath => widget.mod.bundledWithModId == null
      ? 'cutscenes/${widget.mod.name}'
      : 'mods/${widget.mod.bundledWithModId}/cutscenes/${widget.mod.name}';

  @override
  Widget build(BuildContext context) {
    final disabled =
        ref.watch(disabledModsStateControllerProvider).isDisabled(_relPath);
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 120),
        margin: const EdgeInsets.only(bottom: 6),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        decoration: BoxDecoration(
          color: _hovered
              ? AppColors.accentPrimary.withValues(alpha: 0.06)
              : AppColors.inputBackground,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.3)
                : AppColors.borderLight,
          ),
        ),
        child: Opacity(
          opacity: disabled ? 0.45 : 1.0,
          child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: _ClickableName(
                    name: widget.mod.name,
                    folderPath: widget.mod.fullPath,
                  ),
                ),
                Tooltip(
                  message: disabled
                      ? AppLocalizations.of(context)!.modDisabledTooltip
                      : AppLocalizations.of(context)!.modEnableTooltip,
                  child: InkWell(
                    onTap: () {
                      final gameDir =
                          ref.read(appStateControllerProvider).selectedDirectory;
                      ref
                          .read(disabledModsStateControllerProvider.notifier)
                          .setDisabled(gameDir, _relPath, !disabled);
                    },
                    borderRadius: BorderRadius.circular(4),
                    hoverColor: AppColors.success.withValues(alpha: 0.15),
                    child: Padding(
                      padding: const EdgeInsets.all(4),
                      child: Icon(
                        disabled ? Icons.toggle_off_outlined : Icons.toggle_on,
                        size: AppSizes.iconLG(context),
                        color: disabled ? AppColors.textMuted : AppColors.success,
                      ),
                    ),
                  ),
                ),
                if (widget.mod.bundledWithModId == null)
                  InkWell(
                    onTap: widget.onDelete,
                    borderRadius: BorderRadius.circular(4),
                    hoverColor: AppColors.error.withValues(alpha: 0.15),
                    splashColor: AppColors.error.withValues(alpha: 0.2),
                    child: Padding(
                      padding: const EdgeInsets.all(6),
                      child: Icon(
                        Icons.delete_outline,
                        size: AppSizes.iconMD(context),
                        color: AppColors.error,
                      ),
                    ),
                  ),
              ],
            ),
            const SizedBox(height: 4),
            Wrap(
              spacing: 6,
              runSpacing: 4,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(widget.statusIcon, size: 14, color: widget.statusColor),
                    const SizedBox(width: 6),
                    Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 2,
                      ),
                      decoration: BoxDecoration(
                        color: widget.statusColor.withValues(alpha: 0.15),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        widget.statusText,
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          fontWeight: FontWeight.bold,
                          color: widget.statusColor,
                        ),
                      ),
                    ),
                  ],
                ),
                if (widget.resolutionText.isNotEmpty)
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: AppColors.accentPrimary.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(widget.resolutionText, style: TextStyle(fontSize: AppSizes.fontXS(context), fontWeight: FontWeight.bold, color: AppColors.accentPrimary)),
                  ),
                if (widget.codecText.isNotEmpty)
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: widget.codecText == 'H264' ? AppColors.success.withValues(alpha: 0.15) : AppColors.textMuted.withValues(alpha: 0.15),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(widget.codecText, style: TextStyle(fontSize: 9, fontWeight: FontWeight.bold, color: widget.codecText == 'H264' ? AppColors.success : AppColors.textMuted)),
                  ),
                if (widget.mod.bundledWithModId != null)
                  BundledLinkChip(
                    icon: Icons.extension_outlined,
                    label: AppLocalizations.of(context)!.cutsceneBundledWith(
                      widget.mod.bundledWithModId!,
                    ),
                    tooltip: 'Open in Mod Manager',
                    onTap: () {
                      ref.read(pendingTabSelectionProvider.notifier).state =
                          TabSelectionRequest(
                        tabIndex: 5,
                        key: widget.mod.bundledWithModId!,
                      );
                      ref.read(activeTabProvider.notifier).state = 5;
                    },
                  ),
              ],
            ),
            if (!widget.allValid) ...[
              const SizedBox(height: 6),
              Tooltip(
                message: AppLocalizations.of(context)!.tooltipMissingOriginals(
                  widget.mod.missingOriginals.join(", "),
                ),
                child: Text(
                  AppLocalizations.of(context)!.cutsceneMismatchHint,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    height: 1.3,
                  ),
                ),
              ),
            ],
          ],
        ),
        ),
      ),
    );
  }
}
