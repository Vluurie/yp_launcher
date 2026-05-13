import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class HoverButton extends StatefulWidget {
  final String label;
  final Color color;
  final VoidCallback onTap;

  const HoverButton({
    super.key,
    required this.label,
    required this.color,
    required this.onTap,
  });

  @override
  State<HoverButton> createState() => _HoverButtonState();
}

class _HoverButtonState extends State<HoverButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: _hovered
                ? widget.color.withValues(alpha: 0.15)
                : Colors.transparent,
            border: Border.all(color: widget.color),
            borderRadius: BorderRadius.circular(4),
          ),
          child: Text(
            widget.label,
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: widget.color,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ),
    );
  }
}

class HoverIconButton extends StatefulWidget {
  final Widget icon;
  final String tooltip;
  final VoidCallback? onTap;
  final Color? borderColor;
  final EdgeInsets padding;
  final double radius;
  final bool bordered;

  const HoverIconButton({
    super.key,
    required this.icon,
    required this.tooltip,
    required this.onTap,
    this.borderColor,
    this.padding = const EdgeInsets.all(6),
    this.radius = 4,
    this.bordered = true,
  });

  @override
  State<HoverIconButton> createState() => _HoverIconButtonState();
}

class _HoverIconButtonState extends State<HoverIconButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onTap != null;
    final accent = widget.borderColor ?? AppColors.accentPrimary;
    final fillColor = !enabled
        ? Colors.transparent
        : _hovered
            ? accent.withValues(alpha: 0.15)
            : Colors.transparent;
    final borderColor = !enabled ? AppColors.textMuted : accent;
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        cursor:
            enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
        onEnter: (_) {
          if (!enabled) return;
          setState(() => _hovered = true);
        },
        onExit: (_) {
          if (!_hovered) return;
          setState(() => _hovered = false);
        },
        child: GestureDetector(
          onTap: widget.onTap,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 120),
            padding: widget.padding,
            decoration: BoxDecoration(
              color: fillColor,
              border: widget.bordered
                  ? Border.all(color: borderColor)
                  : null,
              borderRadius: BorderRadius.circular(widget.radius),
            ),
            child: widget.icon,
          ),
        ),
      ),
    );
  }
}
