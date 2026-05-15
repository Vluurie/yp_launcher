import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ModDropZone extends StatefulWidget {
  final void Function(List<String> paths) onDrop;
  final VoidCallback onBrowse;
  final VoidCallback? onBrowseFolder;
  final String? title;
  final String? hint;

  const ModDropZone({
    super.key,
    required this.onDrop,
    required this.onBrowse,
    this.onBrowseFolder,
    this.title,
    this.hint,
  });

  @override
  State<ModDropZone> createState() => _ModDropZoneState();
}

class _ModDropZoneState extends State<ModDropZone> {
  bool _dragging = false;
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final title = widget.title ?? l10n.dropModHere;
    final hint = widget.hint ?? l10n.dropModHereHint;
    return DropTarget(
      onDragEntered: (_) => setState(() => _dragging = true),
      onDragExited: (_) => setState(() => _dragging = false),
      onDragDone: (d) {
        setState(() => _dragging = false);
        widget.onDrop(d.files.map((f) => f.path).toList());
      },
      child: GestureDetector(
        onTap: widget.onBrowse,
        child: MouseRegion(
          cursor: SystemMouseCursors.click,
          onEnter: (_) => setState(() => _hovered = true),
          onExit: (_) => setState(() => _hovered = false),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 120),
            width: double.infinity,
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.cardPaddingH(context),
              vertical: AppSizes.dropZonePaddingV(context) / 2,
            ),
            decoration: BoxDecoration(
              color: _dragging
                  ? AppColors.accentPrimary.withValues(alpha: 0.10)
                  : _hovered
                      ? AppColors.accentPrimary.withValues(alpha: 0.04)
                      : AppColors.surfaceLight,
              borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
              border: Border.all(
                color: _dragging
                    ? AppColors.accentPrimary
                    : _hovered
                        ? AppColors.accentPrimary.withValues(alpha: 0.5)
                        : AppColors.borderLight,
                width: _dragging ? 1.5 : 1.0,
              ),
            ),
            child: Wrap(
              alignment: WrapAlignment.center,
              crossAxisAlignment: WrapCrossAlignment.center,
              spacing: AppSizes.spacingMD(context),
              runSpacing: AppSizes.spacingSM(context),
              children: [
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      _dragging ? Icons.file_download : Icons.add_box_outlined,
                      size: AppSizes.iconMD(context),
                      color: _dragging
                          ? AppColors.accentPrimary
                          : AppColors.textMuted,
                    ),
                    SizedBox(width: AppSizes.spacingMD(context)),
                    Text(
                      title,
                      style: TextStyle(
                        fontSize: AppSizes.fontMD(context),
                        fontWeight: FontWeight.bold,
                        color: _dragging
                            ? AppColors.accentPrimary
                            : AppColors.textSecondary,
                      ),
                    ),
                  ],
                ),
                Text(
                  hint,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textMuted,
                  ),
                ),
                if (widget.onBrowseFolder != null)
                  _FolderButton(onTap: widget.onBrowseFolder!),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _FolderButton extends StatefulWidget {
  final VoidCallback onTap;
  const _FolderButton({required this.onTap});

  @override
  State<_FolderButton> createState() => _FolderButtonState();
}

class _FolderButtonState extends State<_FolderButton> {
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
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.folder_open,
              size: 14,
              color: _hovered
                  ? AppColors.accentPrimary
                  : AppColors.textMuted,
            ),
            const SizedBox(width: 6),
            Text(
              l10n.dropZoneBrowseFolder,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: _hovered
                    ? AppColors.accentPrimary
                    : AppColors.textMuted,
                decoration:
                    _hovered ? TextDecoration.underline : TextDecoration.none,
                decorationColor: AppColors.accentPrimary,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
