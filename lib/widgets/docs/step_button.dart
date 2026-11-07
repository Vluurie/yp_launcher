import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class StepButton extends StatefulWidget {
  final IconData icon;
  final VoidCallback? onTap;

  const StepButton({super.key, required this.icon, required this.onTap});

  @override
  State<StepButton> createState() => _StepButtonState();
}

class _StepButtonState extends State<StepButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onTap != null;
    final active = enabled && _hovered;

    return MouseRegion(
      cursor: enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
      onEnter: (_) {
        if (enabled) setState(() => _hovered = true);
      },
      onExit: (_) {
        if (_hovered) setState(() => _hovered = false);
      },
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.all(4),
          decoration: BoxDecoration(
            color: active
                ? AppColors.accentPrimary.withValues(alpha: 0.15)
                : AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: active ? AppColors.accentPrimary : AppColors.borderLight,
            ),
          ),
          child: Icon(
            widget.icon,
            size: AppSizes.iconSM(context),
            color: !enabled
                ? AppColors.textMuted
                : active
                    ? AppColors.accentPrimary
                    : AppColors.textSecondary,
          ),
        ),
      ),
    );
  }
}
