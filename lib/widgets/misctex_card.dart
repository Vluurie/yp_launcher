import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/docs/thumbnail_service.dart';
import 'package:yp_launcher/services/misctex_scan_service.dart';
import 'package:yp_launcher/src/rust/api/texture.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class MisctexCard extends StatefulWidget {
  final MisctexEntry entry;
  final bool busy;
  final VoidCallback onReplace;
  final VoidCallback onDelete;

  const MisctexCard({
    super.key,
    required this.entry,
    required this.busy,
    required this.onReplace,
    required this.onDelete,
  });

  @override
  State<MisctexCard> createState() => _MisctexCardState();
}

class _MisctexCardState extends State<MisctexCard> {
  static const _imageAspect = 16 / 9;

  Uint8List? _png;
  int? _pixelWidth;
  int? _pixelHeight;
  bool _failed = false;
  String? _failureDetail;
  bool _hovered = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void didUpdateWidget(MisctexCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.entry.totalBytes != widget.entry.totalBytes) _load();
  }

  Future<void> _load() async {
    try {
      final preview = await ThumbnailService.preview(widget.entry.dttPath);
      if (!mounted) return;
      setState(() {
        _png = Uint8List.fromList(preview.png);
        _pixelWidth = preview.width;
        _pixelHeight = preview.height;
        _failed = false;
      });
    } on MiscTexError catch (e) {
      if (!mounted) return;
      setState(() {
        _failed = true;
        _failureDetail = 'code ${e.code}${e.detail.isEmpty ? '' : ': '
            '${e.detail}'}';
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _failed = true;
        _failureDetail = '$e';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 120),
        decoration: BoxDecoration(
          color: AppColors.surfaceLight,
          borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          border: Border.all(
            color: _hovered
                ? AppColors.accentPrimary.withValues(alpha: 0.5)
                : AppColors.borderLight,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            _image(context),
            Padding(
              padding: EdgeInsets.all(AppSizes.paddingSM(context)),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    widget.entry.textureName,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textPrimary,
                      fontFamily: AppSizes.monoFamily,
                      fontFamilyFallback: AppSizes.monoFallback,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    _subtitle(),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.textMuted,
                    ),
                  ),
                  SizedBox(height: AppSizes.spacingSM(context)),
                  _actions(context, l10n),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  String _subtitle() {
    final size = _pixelWidth != null ? '${_pixelWidth}×$_pixelHeight' : null;
    final kb = (widget.entry.totalBytes / 1024).toStringAsFixed(0);
    return [
      widget.entry.modId,
      if (size != null) size,
      '$kb KB',
    ].join('  ·  ');
  }

  Widget _image(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.vertical(
        top: Radius.circular(AppSizes.borderRadius(context) - 1),
      ),
      child: AspectRatio(
        aspectRatio: _imageAspect,
        child: Container(
        color: AppColors.backgroundPrimary,
        alignment: Alignment.center,
        child: widget.busy
            ? SizedBox(
                width: AppSizes.iconMD(context),
                height: AppSizes.iconMD(context),
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  color: AppColors.accentPrimary,
                ),
              )
            : _png != null
                ? Image.memory(
                    _png!,
                    fit: BoxFit.contain,
                    filterQuality: FilterQuality.medium,
                    gaplessPlayback: true,
                  )
                : _failed
                    ? Tooltip(
                        message: _failureDetail ?? '',
                        child: Icon(
                          Icons.broken_image_outlined,
                          size: AppSizes.iconLG(context),
                          color: AppColors.warning,
                        ),
                      )
                    : SizedBox(
                        width: AppSizes.iconMD(context),
                        height: AppSizes.iconMD(context),
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: AppColors.textMuted,
                        ),
                      ),
        ),
      ),
    );
  }

  Widget _actions(BuildContext context, AppLocalizations l10n) {
    return Row(
      children: [
        Expanded(
          child: HoverButton(
            label: l10n.toolsIconsReplace,
            color: AppColors.accentPrimary,
            onTap: widget.busy ? () {} : widget.onReplace,
          ),
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        HoverButton(
          label: l10n.toolsIconsDelete,
          color: AppColors.textMuted,
          onTap: widget.busy ? () {} : widget.onDelete,
        ),
      ],
    );
  }
}
