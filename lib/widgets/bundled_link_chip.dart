import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class BundledLinkChip extends StatefulWidget {
  final IconData icon;
  final String label;
  final String tooltip;
  final VoidCallback onTap;

  const BundledLinkChip({
    super.key,
    required this.icon,
    required this.label,
    required this.tooltip,
    required this.onTap,
  });

  /// Strips a trailing `(modId@profile)` suffix off bundled-pack names so
  /// the visible label stays compact. The full name remains in the tooltip.
  static String shortenLabel(String raw) {
    final trimmed = raw.replaceAll(
      RegExp(r'\s*\([^()@]+@[^()]+\)\s*$'),
      '',
    );
    return trimmed.isEmpty ? raw : trimmed;
  }

  @override
  State<BundledLinkChip> createState() => _BundledLinkChipState();
}

class _BundledLinkChipState extends State<BundledLinkChip> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final color = AppColors.accentPrimary;
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onTap,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 100),
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: _hovered
                  ? color.withValues(alpha: 0.18)
                  : color.withValues(alpha: 0.08),
              border: Border.all(
                color: color.withValues(alpha: _hovered ? 0.6 : 0.4),
              ),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(widget.icon, size: 12, color: color),
                const SizedBox(width: 5),
                Flexible(
                  child: Text(
                    widget.label,
                    overflow: TextOverflow.ellipsis,
                    maxLines: 1,
                    softWrap: false,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: color,
                      fontWeight: FontWeight.w600,
                      decoration: _hovered ? TextDecoration.underline : null,
                      decorationColor: color,
                    ),
                  ),
                ),
                const SizedBox(width: 4),
                Icon(
                  Icons.arrow_outward,
                  size: 11,
                  color: color.withValues(alpha: 0.7),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
