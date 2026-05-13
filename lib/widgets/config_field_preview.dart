import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigFieldPreview extends StatelessWidget {
  final Widget child;
  final String? previewImage;
  final String? description;

  const ConfigFieldPreview({
    super.key,
    required this.child,
    this.previewImage,
    this.description,
  });

  @override
  Widget build(BuildContext context) {
    return child;
  }
}

class ConfigPreviewImage extends StatefulWidget {
  final String image;
  final String label;

  const ConfigPreviewImage({
    super.key,
    required this.image,
    required this.label,
  });

  @override
  State<ConfigPreviewImage> createState() => _ConfigPreviewImageState();
}

class _ConfigPreviewImageState extends State<ConfigPreviewImage> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 4, bottom: 8),
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: () => _openFullscreen(context),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            height: 48,
            decoration: BoxDecoration(
              color: _hovered
                  ? AppColors.accentPrimary.withValues(alpha: 0.08)
                  : AppColors.surfaceLight,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(
                color: _hovered
                    ? AppColors.accentPrimary.withValues(alpha: 0.5)
                    : AppColors.borderLight,
              ),
            ),
            child: Row(
              children: [
                ClipRRect(
                  borderRadius: const BorderRadius.only(
                    topLeft: Radius.circular(5),
                    bottomLeft: Radius.circular(5),
                  ),
                  child: Image.asset(
                    widget.image,
                    width: 80,
                    height: 48,
                    fit: BoxFit.cover,
                    errorBuilder: (_, __, ___) =>
                        const SizedBox(width: 80, height: 48),
                  ),
                ),
                Expanded(
                  child: Center(
                    child: Text(
                      widget.label,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textSecondary,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(right: 10),
                  child: Icon(
                    Icons.fullscreen,
                    size: 16,
                    color: _hovered
                        ? AppColors.accentPrimary
                        : AppColors.textMuted,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void _openFullscreen(BuildContext context) {
    showDialog(
      context: context,
      barrierColor: Colors.black87,
      builder: (ctx) => GestureDetector(
        onTap: () => Navigator.of(ctx).pop(),
        child: Dialog(
          backgroundColor: Colors.transparent,
          insetPadding: const EdgeInsets.all(32),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(8),
            child: Image.asset(
              widget.image,
              fit: BoxFit.contain,
              errorBuilder: (_, __, ___) => const SizedBox.shrink(),
            ),
          ),
        ),
      ),
    );
  }
}

class ConfigComparison extends StatelessWidget {
  final String beforeImage;
  final String afterImage;
  final String beforeLabel;
  final String afterLabel;

  const ConfigComparison({
    super.key,
    required this.beforeImage,
    required this.afterImage,
    required this.beforeLabel,
    required this.afterLabel,
  });

  void _openFullscreen(BuildContext context) {
    showDialog(
      context: context,
      barrierColor: Colors.black87,
      builder: (ctx) => _ComparisonDialog(
        beforeImage: beforeImage,
        afterImage: afterImage,
        beforeLabel: beforeLabel,
        afterLabel: afterLabel,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 4, bottom: 8),
      child: _ComparisonThumbnail(
        beforeImage: beforeImage,
        afterImage: afterImage,
        beforeLabel: beforeLabel,
        afterLabel: afterLabel,
        onTap: () => _openFullscreen(context),
      ),
    );
  }
}

class _ComparisonThumbnail extends StatefulWidget {
  final String beforeImage;
  final String afterImage;
  final String beforeLabel;
  final String afterLabel;
  final VoidCallback onTap;

  const _ComparisonThumbnail({
    required this.beforeImage,
    required this.afterImage,
    required this.beforeLabel,
    required this.afterLabel,
    required this.onTap,
  });

  @override
  State<_ComparisonThumbnail> createState() => _ComparisonThumbnailState();
}

class _ComparisonThumbnailState extends State<_ComparisonThumbnail> {
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
          duration: const Duration(milliseconds: 150),
          height: 48,
          decoration: BoxDecoration(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.08)
                : AppColors.surfaceLight,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(
              color: _hovered
                  ? AppColors.accentPrimary.withValues(alpha: 0.5)
                  : AppColors.borderLight,
            ),
          ),
          child: Row(
            children: [
              ClipRRect(
                borderRadius: const BorderRadius.only(
                  topLeft: Radius.circular(5),
                  bottomLeft: Radius.circular(5),
                ),
                child: Image.asset(
                  widget.beforeImage,
                  width: 80,
                  height: 48,
                  fit: BoxFit.cover,
                  errorBuilder: (_, __, ___) =>
                      const SizedBox(width: 80, height: 48),
                ),
              ),
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 10),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Flexible(
                        child: Text(
                          widget.beforeLabel,
                          style: TextStyle(
                            fontSize: AppSizes.fontXS(context),
                            color: AppColors.textMuted,
                            fontWeight: FontWeight.bold,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 8),
                        child: Icon(
                          Icons.compare_arrows,
                          size: 16,
                          color: _hovered
                              ? AppColors.accentPrimary
                              : AppColors.textMuted,
                        ),
                      ),
                      Flexible(
                        child: Text(
                          widget.afterLabel,
                          style: TextStyle(
                            fontSize: AppSizes.fontXS(context),
                            color: AppColors.textSecondary,
                            fontWeight: FontWeight.bold,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              ClipRRect(
                borderRadius: const BorderRadius.only(
                  topRight: Radius.circular(5),
                  bottomRight: Radius.circular(5),
                ),
                child: Image.asset(
                  widget.afterImage,
                  width: 80,
                  height: 48,
                  fit: BoxFit.cover,
                  errorBuilder: (_, __, ___) =>
                      const SizedBox(width: 80, height: 48),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ComparisonDialog extends StatefulWidget {
  final String beforeImage;
  final String afterImage;
  final String beforeLabel;
  final String afterLabel;

  const _ComparisonDialog({
    required this.beforeImage,
    required this.afterImage,
    required this.beforeLabel,
    required this.afterLabel,
  });

  @override
  State<_ComparisonDialog> createState() => _ComparisonDialogState();
}

class _ComparisonDialogState extends State<_ComparisonDialog> {
  double _blend = 0.0;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => Navigator.of(context).pop(),
      child: Dialog(
        backgroundColor: Colors.transparent,
        insetPadding: const EdgeInsets.all(32),
        child: GestureDetector(
          onTap: () {},
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Flexible(
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(8),
                  child: LayoutBuilder(
                    builder: (context, constraints) {
                      return SizedBox(
                        width: constraints.maxWidth,
                        child: AspectRatio(
                          aspectRatio: 16 / 9,
                          child: Stack(
                            fit: StackFit.expand,
                            children: [
                              Image.asset(
                                widget.beforeImage,
                                fit: BoxFit.cover,
                                errorBuilder: (_, __, ___) => Container(
                                  color: AppColors.surfaceMedium,
                                  alignment: Alignment.center,
                                  child: const Icon(
                                    Icons.image_not_supported,
                                    color: AppColors.textMuted,
                                    size: 48,
                                  ),
                                ),
                              ),
                              Opacity(
                                opacity: _blend,
                                child: Image.asset(
                                  widget.afterImage,
                                  fit: BoxFit.cover,
                                  errorBuilder: (_, __, ___) =>
                                      const SizedBox.shrink(),
                                ),
                              ),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 10,
                ),
                decoration: BoxDecoration(
                  color: AppColors.backgroundCard,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      widget.beforeLabel,
                      style: TextStyle(
                        fontSize: AppSizes.fontMD(context),
                        fontWeight: FontWeight.bold,
                        color: _blend <= 0.3
                            ? AppColors.accentPrimary
                            : AppColors.textMuted,
                      ),
                    ),
                    SizedBox(
                      width: 300,
                      child: SliderTheme(
                        data: SliderThemeData(
                          activeTrackColor: AppColors.accentPrimary,
                          inactiveTrackColor: AppColors.borderLight,
                          thumbColor: AppColors.accentPrimary,
                          overlayColor: AppColors.accentPrimary.withValues(
                            alpha: 0.15,
                          ),
                          trackHeight: 3,
                          thumbShape: const RoundSliderThumbShape(
                            enabledThumbRadius: 8,
                          ),
                        ),
                        child: Slider(
                          value: _blend,
                          onChanged: (v) => setState(() => _blend = v),
                        ),
                      ),
                    ),
                    Text(
                      widget.afterLabel,
                      style: TextStyle(
                        fontSize: AppSizes.fontMD(context),
                        fontWeight: FontWeight.bold,
                        color: _blend >= 0.7
                            ? AppColors.accentPrimary
                            : AppColors.textMuted,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
