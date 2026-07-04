import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ClickableName extends StatefulWidget {
  final String name;
  final String folderPath;

  const ClickableName({super.key, required this.name, required this.folderPath});

  @override
  State<ClickableName> createState() => _ClickableNameState();
}

class _ClickableNameState extends State<ClickableName> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: () => Process.run(
            'explorer.exe', [widget.folderPath.replaceAll('/', '\\')]),
        child: Text(
          widget.name,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: _hovered ? AppColors.accentPrimary : AppColors.textSecondary,
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

class LoadOrderItem extends ConsumerStatefulWidget {
  final int index;
  final String folder;
  final String folderPath;
  final String? priority;
  final bool exists;
  final List<String>? conflictsWith;
  final String? bundledWithModId;
  final VoidCallback onRemove;

  const LoadOrderItem({
    super.key,
    required this.index,
    required this.folder,
    required this.folderPath,
    required this.priority,
    required this.exists,
    this.conflictsWith,
    this.bundledWithModId,
    required this.onRemove,
  });

  @override
  ConsumerState<LoadOrderItem> createState() => _LoadOrderItemState();
}

class _LoadOrderItemState extends ConsumerState<LoadOrderItem> {
  bool _hovered = false;
  String get _relPath => 'inject/textures/${widget.folder}';

  @override
  Widget build(BuildContext context) {
    final disabled =
        ref.watch(disabledModsStateControllerProvider).isDisabled(_relPath);
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 120),
        margin: EdgeInsets.only(bottom: AppSizes.paddingXS(context)),
        padding: EdgeInsets.symmetric(
          horizontal: AppSizes.paddingMD(context),
          vertical: AppSizes.paddingSM(context),
        ),
        decoration: BoxDecoration(
          color: _hovered
              ? AppColors.accentPrimary.withValues(alpha: 0.06)
              : AppColors.surfaceLight,
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.3)
                : AppColors.borderLight,
          ),
        ),
        child: Opacity(
          opacity: disabled ? 0.45 : 1.0,
          child: Row(
          children: [
            ReorderableDragStartListener(
              index: widget.index,
              child: MouseRegion(
                cursor: SystemMouseCursors.grab,
                child: Icon(
                  Icons.drag_handle,
                  size: AppSizes.iconLG(context),
                  color: _hovered
                      ? AppColors.accentPrimary
                      : AppColors.textMuted,
                ),
              ),
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Icon(
              Icons.folder,
              size: 14,
              color: widget.exists ? AppColors.textMuted : AppColors.warning,
            ),
            SizedBox(width: AppSizes.spacingMD(context)),
            Expanded(
              child: Tooltip(
                message: widget.folder,
                child: widget.exists
                    ? ClickableName(
                        name: widget.folder,
                        folderPath: widget.folderPath,
                      )
                    : Text(
                        widget.folder,
                        style: TextStyle(
                          fontSize: AppSizes.fontSM(context),
                          color: AppColors.warning,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
              ),
            ),
            if (widget.bundledWithModId != null) ...[
              SizedBox(width: AppSizes.spacingSM(context)),
              Tooltip(
                message: AppLocalizations.of(context)!
                    .textureBundledWithMod(widget.bundledWithModId!),
                child: InkWell(
                  borderRadius: BorderRadius.circular(4),
                  onTap: () {
                    ref.read(pendingTabSelectionProvider.notifier).state =
                        TabSelectionRequest(
                      tabIndex: 5,
                      key: widget.bundledWithModId!,
                    );
                    ref.read(activeTabProvider.notifier).state = 5;
                  },
                  child: Padding(
                    padding: const EdgeInsets.all(4),
                    child: Icon(
                      Icons.extension_outlined,
                      size: 14,
                      color: AppColors.accentPrimary,
                    ),
                  ),
                ),
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
            ],
            if (widget.conflictsWith != null &&
                widget.conflictsWith!.isNotEmpty)
              Tooltip(
                message: AppLocalizations.of(
                  context,
                )!.tooltipTextureOverlap(widget.conflictsWith!.join(", ")),
                child: Container(
                  margin: EdgeInsets.only(right: AppSizes.spacingMD(context)),
                  padding: EdgeInsets.symmetric(
                    horizontal: AppSizes.paddingSM(context),
                    vertical: AppSizes.paddingXS(context),
                  ),
                  decoration: BoxDecoration(
                    color: AppColors.warning.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(
                      AppSizes.borderRadius(context),
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.warning_amber,
                        size: 10,
                        color: AppColors.warning,
                      ),
                      SizedBox(width: AppSizes.paddingXS(context)),
                      Text(
                        AppLocalizations.of(context)!.textureOverlapLabel,
                        style: TextStyle(
                          fontSize: 9,
                          fontWeight: FontWeight.bold,
                          color: AppColors.warning,
                          letterSpacing: 0.5,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            if (widget.priority != null)
              Container(
                margin: EdgeInsets.only(right: AppSizes.spacingMD(context)),
                padding: EdgeInsets.symmetric(
                  horizontal: AppSizes.paddingSM(context),
                  vertical: AppSizes.paddingXS(context),
                ),
                decoration: BoxDecoration(
                  color: AppColors.accentPrimary.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(
                    AppSizes.borderRadius(context),
                  ),
                ),
                child: Text(
                  widget.priority!,
                  style: const TextStyle(
                    fontSize: 9,
                    fontWeight: FontWeight.bold,
                    color: AppColors.accentPrimary,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
            if (!widget.exists)
              Tooltip(
                message: AppLocalizations.of(context)!.tooltipFolderNotFound,
                child: Padding(
                  padding: EdgeInsets.only(right: AppSizes.paddingXS(context)),
                  child: Icon(
                    Icons.warning_amber,
                    size: 14,
                    color: AppColors.warning,
                  ),
                ),
              ),
            InkWell(
              onTap: () {
                final gameDir =
                    ref.read(appStateControllerProvider).selectedDirectory;
                ref
                    .read(disabledModsStateControllerProvider.notifier)
                    .setDisabled(gameDir, _relPath, !disabled);
              },
              borderRadius: BorderRadius.circular(
                AppSizes.borderRadius(context),
              ),
              hoverColor: AppColors.success.withValues(alpha: 0.15),
              child: Tooltip(
                message: disabled
                    ? AppLocalizations.of(context)!.modDisabledTooltip
                    : AppLocalizations.of(context)!.modEnableTooltip,
                child: Padding(
                  padding: EdgeInsets.all(AppSizes.paddingXS(context)),
                  child: Icon(
                    disabled ? Icons.toggle_off_outlined : Icons.toggle_on,
                    size: AppSizes.iconLG(context),
                    color: disabled ? AppColors.textMuted : AppColors.success,
                  ),
                ),
              ),
            ),
            if (widget.bundledWithModId == null)
              InkWell(
                onTap: widget.onRemove,
                borderRadius: BorderRadius.circular(
                  AppSizes.borderRadius(context),
                ),
                hoverColor: AppColors.error.withValues(alpha: 0.15),
                splashColor: AppColors.error.withValues(alpha: 0.2),
                child: Padding(
                  padding: EdgeInsets.all(AppSizes.paddingXS(context)),
                  child: Icon(Icons.close, size: 14, color: AppColors.error),
                ),
              ),
          ],
        ),
      ),
      ),
    );
  }
}

class ProgressInstallCard extends StatelessWidget {
  final String phase;
  final double progress;
  final String progressText;

  const ProgressInstallCard({
    super.key,
    required this.phase,
    required this.progress,
    required this.progressText,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      width: double.infinity,
      padding: EdgeInsets.symmetric(
        vertical: AppSizes.dropZonePaddingV(context),
        horizontal: AppSizes.paddingLG(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.accentPrimary.withValues(alpha: 0.06),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(
          color: AppColors.accentPrimary.withValues(alpha: 0.3),
        ),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          AnimatedSwitcher(
            duration: const Duration(milliseconds: 200),
            child: Text(
              phase,
              key: ValueKey(phase),
              style: TextStyle(
                fontSize: AppSizes.fontLG(context),
                fontWeight: FontWeight.bold,
                color: AppColors.accentPrimary,
              ),
            ),
          ),
          SizedBox(height: AppSizes.spacingMD(context)),
          ClipRRect(
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
            child: LinearProgressIndicator(
              value: progress > 0 ? progress.clamp(0.0, 1.0) : null,
              minHeight: 6,
              backgroundColor: AppColors.accentPrimary.withValues(alpha: 0.12),
              color: AppColors.accentPrimary,
            ),
          ),
          if (progressText.isNotEmpty) ...[
            SizedBox(height: AppSizes.spacingSM(context)),
            Text(
              progressText,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textSecondary,
              ),
            ),
          ],
        ],
      ),
    );
  }
}
