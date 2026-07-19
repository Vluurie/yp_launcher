import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

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
    return Container(
      margin: EdgeInsets.only(bottom: AppSizes.paddingLG(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundCard,
        borderRadius: radius,
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
            curve: Curves.easeInOut,
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
                color: _hovered
                    ? AppColors.accentPrimary
                    : Colors.transparent,
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
