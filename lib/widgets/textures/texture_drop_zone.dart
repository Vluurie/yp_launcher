import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/texture_widgets.dart';

class TextureDropZone extends StatefulWidget {
  final bool installing;
  final String installPhase;
  final double progressPercent;
  final String Function(AppLocalizations l10n) buildProgressText;
  final void Function(List<String> paths) onDrop;
  final VoidCallback onBrowse;

  const TextureDropZone({
    super.key,
    required this.installing,
    required this.installPhase,
    required this.progressPercent,
    required this.buildProgressText,
    required this.onDrop,
    required this.onBrowse,
  });

  @override
  State<TextureDropZone> createState() => _TextureDropZoneState();
}

class _TextureDropZoneState extends State<TextureDropZone> {
  bool _dragging = false;
  bool _dropHovered = false;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    if (widget.installing) {
      return ProgressInstallCard(
        phase: widget.installPhase.isNotEmpty ? widget.installPhase : l10n.installingTextures,
        progress: widget.progressPercent,
        progressText: widget.buildProgressText(l10n),
      );
    }
    return DropTarget(
      onDragEntered: (_) => setState(() => _dragging = true),
      onDragExited: (_) => setState(() => _dragging = false),
      onDragDone: (details) {
        setState(() => _dragging = false);
        widget.onDrop(details.files.map((f) => f.path).toList());
      },
      child: GestureDetector(
        onTap: widget.onBrowse,
        child: MouseRegion(
          cursor: SystemMouseCursors.click,
          onEnter: (_) => setState(() => _dropHovered = true),
          onExit: (_) => setState(() => _dropHovered = false),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            width: double.infinity,
            padding: EdgeInsets.symmetric(
              vertical: AppSizes.dropZonePaddingV(context),
            ),
            decoration: BoxDecoration(
              color: _dragging
                  ? AppColors.accentPrimary.withValues(alpha: 0.08)
                  : _dropHovered
                  ? AppColors.accentPrimary.withValues(alpha: 0.04)
                  : AppColors.surfaceLight,
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context),
              ),
              border: Border.all(
                color: _dragging
                    ? AppColors.accentPrimary
                    : _dropHovered
                    ? AppColors.accentPrimary.withValues(alpha: 0.5)
                    : AppColors.borderLight,
                width: _dragging
                    ? 2.0
                    : _dropHovered
                    ? 1.5
                    : 1.0,
              ),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  _dragging ? Icons.file_download : Icons.texture,
                  size: 28,
                  color: _dragging
                      ? AppColors.accentPrimary
                      : AppColors.textMuted,
                ),
                SizedBox(height: AppSizes.spacingMD(context)),
                Text(
                  _dragging ? l10n.dropToInstall : l10n.dropTextureHere,
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                    color: _dragging
                        ? AppColors.accentPrimary
                        : AppColors.textSecondary,
                  ),
                ),
                SizedBox(height: AppSizes.paddingXS(context)),
                Text(
                  l10n.orClickToBrowse,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textMuted,
                  ),
                ),
                SizedBox(height: AppSizes.spacingMD(context)),
                Text(
                  l10n.installedToTextures,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
