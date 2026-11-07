import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class DocsImage extends StatefulWidget {
  final String image;
  final String? caption;

  const DocsImage({super.key, required this.image, this.caption});

  @override
  State<DocsImage> createState() => _DocsImageState();
}

class _DocsImageState extends State<DocsImage> {
  bool _hovered = false;
  bool _failed = false;

  @override
  Widget build(BuildContext context) {
    if (_failed) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        MouseRegion(
          cursor: SystemMouseCursors.click,
          onEnter: (_) => setState(() => _hovered = true),
          onExit: (_) => setState(() => _hovered = false),
          child: GestureDetector(
            onTap: () => _openFullscreen(context),
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 140),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(6),
                border: Border.all(
                  color: _hovered
                      ? AppColors.accentPrimary.withValues(alpha: 0.6)
                      : AppColors.borderLight,
                ),
              ),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(5),
                child: Image.asset(
                  widget.image,
                  fit: BoxFit.cover,
                  errorBuilder: (context, error, stack) {
                    WidgetsBinding.instance.addPostFrameCallback((_) {
                      if (mounted && !_failed) {
                        setState(() => _failed = true);
                      }
                    });
                    return const SizedBox.shrink();
                  },
                ),
              ),
            ),
          ),
        ),
        if (widget.caption != null)
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: Text(
              widget.caption!,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                height: 1.4,
              ),
            ),
          ),
      ],
    );
  }

  void _openFullscreen(BuildContext context) {
    showDialog<void>(
      context: context,
      barrierColor: AppColors.scrim.withValues(alpha: 0.85),
      builder: (ctx) => GestureDetector(
        onTap: () => Navigator.of(ctx).pop(),
        child: Padding(
          padding: EdgeInsets.all(AppSizes.paddingXL(ctx)),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Flexible(
                child: InteractiveViewer(
                  maxScale: 5,
                  child: Image.asset(widget.image),
                ),
              ),
              if (widget.caption != null)
                Padding(
                  padding: EdgeInsets.only(top: AppSizes.paddingMD(ctx)),
                  child: Text(
                    widget.caption!,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: AppSizes.fontSM(ctx),
                      color: AppColors.textSecondary,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
