import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class DocsToolPanel extends StatefulWidget {
  final String title;
  final String subtitle;
  final IconData icon;
  final Widget child;
  final bool initiallyOpen;

  const DocsToolPanel({
    super.key,
    required this.title,
    required this.subtitle,
    required this.icon,
    required this.child,
    this.initiallyOpen = false,
  });

  @override
  State<DocsToolPanel> createState() => _DocsToolPanelState();
}

class _DocsToolPanelState extends State<DocsToolPanel> {
  late bool _open = widget.initiallyOpen;
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return SelectionContainer.disabled(child: _panel(context));
  }

  Widget _panel(BuildContext context) {
    return Container(
      margin: EdgeInsets.symmetric(vertical: AppSizes.paddingMD(context)),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(
          color: _open || _hovered
              ? AppColors.accentPrimary.withValues(alpha: 0.5)
              : AppColors.accentPrimary.withValues(alpha: 0.25),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _header(context),
          if (_open)
            Padding(
              padding: EdgeInsets.fromLTRB(
                AppSizes.paddingLG(context),
                0,
                AppSizes.paddingLG(context),
                AppSizes.paddingLG(context),
              ),
              child: widget.child,
            ),
        ],
      ),
    );
  }

  Widget _header(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () => setState(() => _open = !_open),
        child: Padding(
          padding: EdgeInsets.all(AppSizes.paddingLG(context)),
          child: Row(
            children: [
              Icon(
                widget.icon,
                size: AppSizes.iconMD(context),
                color: AppColors.accentPrimary,
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      widget.title,
                      style: TextStyle(
                        fontSize: AppSizes.fontMD(context),
                        fontWeight: FontWeight.w700,
                        color: AppColors.textPrimary,
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.only(top: 2),
                      child: Text(
                        widget.subtitle,
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          color: AppColors.textMuted,
                          height: 1.4,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              SizedBox(width: AppSizes.spacingMD(context)),
              AnimatedRotation(
                duration: const Duration(milliseconds: 140),
                turns: _open ? 0.5 : 0,
                child: Icon(
                  Icons.keyboard_arrow_down,
                  size: AppSizes.iconMD(context),
                  color: _hovered
                      ? AppColors.accentPrimary
                      : AppColors.textSecondary,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
