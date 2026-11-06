import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/nier_curves.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_change_status.dart';

class CollapsibleCard extends StatefulWidget {
  final String title;
  final List<Widget> children;
  final bool initiallyExpanded;

  const CollapsibleCard({
    super.key,
    required this.title,
    required this.children,
    this.initiallyExpanded = false,
  });

  @override
  State<CollapsibleCard> createState() => _CollapsibleCardState();
}

class _CollapsibleCardState extends State<CollapsibleCard> {
  late bool _collapsed = !widget.initiallyExpanded;

  @override
  Widget build(BuildContext context) {
    final radius = BorderRadius.circular(AppSizes.borderRadius(context));
    final changed = collectChangedLabels(widget.children);
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: radius,
        border: Border.all(color: AppColors.borderLight),
        boxShadow: [
          BoxShadow(
            color: AppColors.shadow,
            blurRadius: 4,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _HoverHeader(
            radius: radius,
            collapsed: _collapsed,
            onTap: () => setState(() => _collapsed = !_collapsed),
            child: Container(
              width: double.infinity,
              padding: EdgeInsets.symmetric(
                horizontal: AppSizes.cardPaddingH(context),
                vertical: AppSizes.cardPaddingV(context),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      widget.title,
                      style: TextStyle(
                        fontSize: AppSizes.fontSM(context),
                        fontWeight: FontWeight.bold,
                        color: AppColors.accentPrimary,
                        letterSpacing: 1.0,
                      ),
                    ),
                  ),
                  if (changed.isNotEmpty) ...[
                    _ChangedBadge(labels: changed),
                    SizedBox(width: AppSizes.spacingMD(context)),
                  ],
                  AnimatedRotation(
                    turns: _collapsed ? -0.25 : 0,
                    duration: const Duration(milliseconds: 150),
                    child: Icon(
                      Icons.keyboard_arrow_down,
                      size: AppSizes.iconMD(context),
                      color: AppColors.accentPrimary,
                    ),
                  ),
                ],
              ),
            ),
          ),
          AnimatedSize(
            duration: const Duration(milliseconds: 180),
            curve: NierCurves.smooth,
            alignment: Alignment.topCenter,
            child: _collapsed
                ? const SizedBox(width: double.infinity)
                : Padding(
                    padding: EdgeInsets.all(AppSizes.cardPaddingH(context)),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: widget.children,
                    ),
                  ),
          ),
        ],
      ),
    );
  }
}

class _HoverHeader extends StatefulWidget {
  final BorderRadius radius;
  final bool collapsed;
  final VoidCallback onTap;
  final Widget child;

  const _HoverHeader({
    required this.radius,
    required this.collapsed,
    required this.onTap,
    required this.child,
  });

  @override
  State<_HoverHeader> createState() => _HoverHeaderState();
}

class _HoverHeaderState extends State<_HoverHeader> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final radius = BorderRadius.only(
      topLeft: widget.radius.topLeft,
      topRight: widget.radius.topRight,
      bottomLeft: widget.collapsed ? widget.radius.bottomLeft : Radius.zero,
      bottomRight: widget.collapsed ? widget.radius.bottomRight : Radius.zero,
    );
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.10)
                : AppColors.surfaceMedium,
            borderRadius: radius,
            border: Border(
              left: BorderSide(
                color: _hovered ? AppColors.accentPrimary : Colors.transparent,
                width: 2,
              ),
            ),
          ),
          child: widget.child,
        ),
      ),
    );
  }
}

class _ChangedBadge extends StatefulWidget {
  final List<String> labels;

  const _ChangedBadge({required this.labels});

  @override
  State<_ChangedBadge> createState() => _ChangedBadgeState();
}

class _ChangedBadgeState extends State<_ChangedBadge>
    with SingleTickerProviderStateMixin {
  static const _pulses = 3;

  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 900),
  );

  @override
  void initState() {
    super.initState();
    _startPulse();
  }

  @override
  void didUpdateWidget(_ChangedBadge oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.labels.length != widget.labels.length) _startPulse();
  }

  void _startPulse() {
    _pulse
      ..reset()
      ..repeat(count: _pulses);
  }

  @override
  void dispose() {
    _pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final color = AppColors.success;
    return Tooltip(
      message: [
        l10n.cardChangedTooltipHeader,
        for (final label in widget.labels) '• $label',
      ].join('\n'),
      decoration: BoxDecoration(
        color: AppColors.tooltipBackground,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
      ),
      textStyle: TextStyle(
        fontSize: AppSizes.fontXS(context),
        color: AppColors.textOnDark,
        height: 1.4,
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(
            width: 16,
            height: 16,
            child: AnimatedBuilder(
              animation: _pulse,
              builder: (context, _) {
                final t = _pulse.isAnimating ? _pulse.value : 1.0;
                return Stack(
                  alignment: Alignment.center,
                  children: [
                    if (_pulse.isAnimating)
                      Container(
                        width: 8 + 8 * t,
                        height: 8 + 8 * t,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: color.withValues(alpha: 0.45 * (1 - t)),
                        ),
                      ),
                    Container(
                      width: 8,
                      height: 8,
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: color,
                        boxShadow: [
                          BoxShadow(
                            color: color.withValues(alpha: 0.6),
                            blurRadius: 4,
                          ),
                        ],
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Text(
            l10n.cardChangedCount(widget.labels.length),
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontWeight: FontWeight.w600,
              color: color,
            ),
          ),
        ],
      ),
    );
  }
}
