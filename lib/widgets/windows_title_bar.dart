import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/providers/nams_settings_state.dart';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class WindowsTitleBar extends ConsumerWidget {
  const WindowsTitleBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (!PlatformGate.isWindows) {
      return const SizedBox.shrink();
    }

    return Container(
      height: AppSizes.titleBarHeight,
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        border: Border(
          bottom: BorderSide(color: AppColors.borderLight, width: 1),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: GestureDetector(
              onPanStart: (_) => windowManager.startDragging(),
              onDoubleTap: () async {
                if (await windowManager.isMaximized()) {
                  windowManager.unmaximize();
                } else {
                  windowManager.maximize();
                }
              },
              child: Container(
                color: Colors.transparent,
                padding: const EdgeInsets.only(left: 12),
                alignment: Alignment.centerLeft,
                child: Row(
                  children: [
                    Icon(
                      Icons.sports_esports,
                      size: AppSizes.iconMD(context),
                      color: AppColors.accentPrimary,
                    ),
                    const SizedBox(width: 8),
                    Text(
                      'YP',
                      style: TextStyle(
                        fontSize: AppSizes.fontLG(context),
                        color: AppColors.textPrimary,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),

          _WindowButton(
            icon: Icons.remove,
            onPressed: () => windowManager.minimize(),
            tooltip: AppLocalizations.of(context)!.tooltipMinimize,
          ),
          _MaximizeButton(),
          _WindowButton(
            icon: Icons.close,
            onPressed: () => _handleClose(context, ref),
            tooltip: AppLocalizations.of(context)!.tooltipClose,
            isClose: true,
          ),
        ],
      ),
    );
  }

  void _handleClose(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final busyCount = ref.read(busyOperationsProvider);
    if (busyCount > 0) {
      showDialog(
        context: context,
        builder: (ctx) => AlertDialog(
          backgroundColor: AppColors.backgroundCard,
          title: Text(
            l10n.busyCloseTitle,
            style: TextStyle(
                fontSize: AppSizes.fontXL(ctx), color: AppColors.warning),
          ),
          content: Text(
            l10n.busyCloseBody,
            style: TextStyle(
                fontSize: AppSizes.fontMD(ctx), color: AppColors.textSecondary),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: Text(l10n.stay,
                  style: const TextStyle(color: AppColors.textMuted)),
            ),
            TextButton(
              onPressed: () => exit(0),
              child: Text(l10n.busyCloseForce,
                  style: const TextStyle(
                      color: AppColors.error, fontWeight: FontWeight.bold)),
            ),
          ],
        ),
      );
      return;
    }

    final configUnsaved = ref.read(configStateControllerProvider).hasUnsavedChanges;
    final settingsUnsaved = ref.read(namsSettingsStateControllerProvider).hasUnsavedChanges;

    if (!configUnsaved && !settingsUnsaved) {
      exit(0);
    }

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(l10n.unsavedChangesTitle, style: TextStyle(fontSize: AppSizes.fontXL(ctx), color: AppColors.textPrimary)),
        content: Text(l10n.unsavedChangesMessage, style: TextStyle(fontSize: AppSizes.fontMD(ctx), color: AppColors.textSecondary)),
        actions: [
          TextButton(onPressed: () => Navigator.of(ctx).pop(), child: Text(l10n.stay, style: const TextStyle(color: AppColors.textMuted))),
          TextButton(onPressed: () => exit(0), child: Text(l10n.discard, style: const TextStyle(color: AppColors.error, fontWeight: FontWeight.bold))),
        ],
      ),
    );
  }
}

class _WindowButton extends StatefulWidget {
  final IconData icon;
  final VoidCallback onPressed;
  final String tooltip;
  final bool isClose;

  const _WindowButton({
    required this.icon,
    required this.onPressed,
    required this.tooltip,
    this.isClose = false,
  });

  @override
  State<_WindowButton> createState() => _WindowButtonState();
}

class _WindowButtonState extends State<_WindowButton> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: Tooltip(
        message: widget.tooltip,
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            width: AppSizes.titleBarButtonWidth,
            height: AppSizes.titleBarHeight,
            color: _isHovered
                ? (widget.isClose ? AppColors.error : AppColors.borderLight)
                : Colors.transparent,
            child: Icon(
              widget.icon,
              size: AppSizes.iconMD(context),
              color: _isHovered && widget.isClose
                  ? Colors.white
                  : AppColors.textPrimary,
            ),
          ),
        ),
      ),
    );
  }
}

class _MaximizeButton extends StatefulWidget {
  @override
  State<_MaximizeButton> createState() => _MaximizeButtonState();
}

class _MaximizeButtonState extends State<_MaximizeButton> {
  bool _isHovered = false;
  bool _isMaximized = false;

  @override
  void initState() {
    super.initState();
    _checkMaximized();
  }

  Future<void> _checkMaximized() async {
    final maximized = await windowManager.isMaximized();
    if (mounted) {
      setState(() => _isMaximized = maximized);
    }
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: Tooltip(
        message: _isMaximized
            ? AppLocalizations.of(context)!.tooltipRestore
            : AppLocalizations.of(context)!.tooltipMaximize,
        child: GestureDetector(
          onTap: () async {
            if (_isMaximized) {
              await windowManager.unmaximize();
            } else {
              await windowManager.maximize();
            }
            _checkMaximized();
          },
          child: Container(
            width: AppSizes.titleBarButtonWidth,
            height: AppSizes.titleBarHeight,
            color: _isHovered ? AppColors.borderLight : Colors.transparent,
            child: Icon(
              _isMaximized ? Icons.filter_none : Icons.crop_square,
              size: AppSizes.iconSM(context),
              color: AppColors.textPrimary,
            ),
          ),
        ),
      ),
    );
  }
}
