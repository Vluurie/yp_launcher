import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class CutsceneDropZone extends StatefulWidget {
  final bool installing;
  final String progressText;
  final double progressPercent;
  final void Function(List<String> paths) onDrop;
  final VoidCallback onBrowse;

  const CutsceneDropZone({
    super.key,
    required this.installing,
    required this.progressText,
    required this.progressPercent,
    required this.onDrop,
    required this.onBrowse,
  });

  @override
  State<CutsceneDropZone> createState() => _CutsceneDropZoneState();
}

class _CutsceneDropZoneState extends State<CutsceneDropZone> {
  bool _dragging = false;
  bool _dropHovered = false;

  @override
  Widget build(BuildContext context) {
    if (widget.installing) {
      return AnimatedContainer(
        duration: const Duration(milliseconds: 150),
        width: double.infinity,
        padding: const EdgeInsets.symmetric(vertical: 24),
        decoration: BoxDecoration(
          color: AppColors.accentPrimary.withValues(alpha: 0.06),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: AppColors.accentPrimary.withValues(alpha: 0.3),
          ),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            SizedBox(
              width: 200,
              child: LinearProgressIndicator(
                value: widget.progressPercent > 0 ? widget.progressPercent : null,
                backgroundColor: AppColors.surfaceLight,
                color: AppColors.accentPrimary,
                minHeight: 6,
                borderRadius: BorderRadius.circular(3),
              ),
            ),
            const SizedBox(height: 10),
            Text(
              widget.progressText,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.accentPrimary,
              ),
            ),
          ],
        ),
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
            padding: const EdgeInsets.symmetric(vertical: 24),
            decoration: BoxDecoration(
              color: _dragging
                  ? AppColors.accentPrimary.withValues(alpha: 0.08)
                  : _dropHovered
                  ? AppColors.accentPrimary.withValues(alpha: 0.04)
                  : AppColors.surfaceLight,
              borderRadius: BorderRadius.circular(8),
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
                  _dragging
                      ? Icons.file_download
                      : Icons.movie_creation_outlined,
                  size: 40,
                  color: _dragging
                      ? AppColors.accentPrimary
                      : AppColors.textMuted,
                ),
                const SizedBox(height: 12),
                Text(
                  _dragging
                      ? AppLocalizations.of(context)!.dropToInstall
                      : AppLocalizations.of(context)!.dropCutsceneHere,
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                    color: _dragging
                        ? AppColors.accentPrimary
                        : AppColors.textSecondary,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  AppLocalizations.of(context)!.orClickToBrowse,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textMuted,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  AppLocalizations.of(context)!.cutsceneMovieHint,
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
