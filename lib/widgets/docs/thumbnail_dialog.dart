import 'dart:io';

import 'package:desktop_drop/desktop_drop.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/docs/thumbnail_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class ThumbnailChoice {
  final String imagePath;
  final int width;
  final int height;

  const ThumbnailChoice({
    required this.imagePath,
    required this.width,
    required this.height,
  });
}

class ThumbnailDialog extends StatefulWidget {
  final String subtitle;
  final ThumbnailChoice? initial;
  final ({int width, int height})? fixedSize;

  const ThumbnailDialog({
    super.key,
    required this.subtitle,
    this.initial,
    this.fixedSize,
  });

  static Future<ThumbnailChoice?> show(
    BuildContext context, {
    required String subtitle,
    ThumbnailChoice? initial,
    ({int width, int height})? fixedSize,
  }) {
    return showDialog<ThumbnailChoice>(
      context: context,
      builder: (_) => ThumbnailDialog(
        subtitle: subtitle,
        initial: initial,
        fixedSize: fixedSize,
      ),
    );
  }

  @override
  State<ThumbnailDialog> createState() => _ThumbnailDialogState();
}

class _ThumbnailDialogState extends State<ThumbnailDialog> {
  static const _minSide = 8;
  static const _maxSide = 1024;

  String? _imagePath;
  Uint8List? _preview;

  int _width = ThumbnailService.defaultWidth;
  int _height = ThumbnailService.defaultHeight;
  late final TextEditingController _widthController;
  late final TextEditingController _heightController;

  bool _dragging = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    final fixed = widget.fixedSize;
    final initial = widget.initial;
    if (fixed != null) {
      _width = fixed.width;
      _height = fixed.height;
    } else if (initial != null) {
      _width = initial.width;
      _height = initial.height;
    }
    _widthController = TextEditingController(text: '$_width');
    _heightController = TextEditingController(text: '$_height');
    if (initial != null) _loadImage(initial.imagePath);
  }

  @override
  void dispose() {
    _widthController.dispose();
    _heightController.dispose();
    super.dispose();
  }

  int _snapToFour(int value) => (value + 2) ~/ 4 * 4;

  int _clampSide(int value) =>
      _snapToFour(value.clamp(_minSide, _maxSide));

  Future<void> _loadImage(String path) async {
    try {
      final bytes = await File(path).readAsBytes();
      if (!mounted) return;
      setState(() {
        _imagePath = path;
        _preview = bytes;
        _error = null;
      });
    } catch (_) {}
  }

  Future<void> _pick() async {
    final result = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'tga'],
    );
    final picked = result?.files.single.path;
    if (picked != null) await _loadImage(picked);
  }

  void _onDrop(List<String> paths) {
    final image = paths.firstWhere(
      ThumbnailService.isAccepted,
      orElse: () => '',
    );
    if (image.isEmpty) {
      final l10n = AppLocalizations.of(context)!;
      setState(() => _error = l10n.errImageUnreadable);
      return;
    }
    _loadImage(image);
  }

  void _confirm() {
    final path = _imagePath;
    if (path == null) return;
    Navigator.of(context).pop(
      ThumbnailChoice(imagePath: path, width: _width, height: _height),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Dialog(
      backgroundColor: AppColors.backgroundCard,
      insetPadding: EdgeInsets.all(AppSizes.paddingXL(context)),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 640),
        child: SingleChildScrollView(
          padding: EdgeInsets.all(AppSizes.paddingLG(context)),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _header(context, l10n),
              SizedBox(height: AppSizes.paddingMD(context)),
              Center(child: _canvas(context, l10n)),
              SizedBox(height: AppSizes.paddingMD(context)),
              _sizeRow(context, l10n),
              if (_error != null) ...[
                SizedBox(height: AppSizes.spacingMD(context)),
                Text(
                  _error!,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.error,
                    height: 1.4,
                  ),
                ),
              ],
              SizedBox(height: AppSizes.spacingMD(context)),
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(
                    Icons.bolt_outlined,
                    size: AppSizes.iconSM(context),
                    color: AppColors.textMuted,
                  ),
                  SizedBox(width: AppSizes.spacingSM(context)),
                  Expanded(
                    child: Text(
                      l10n.thumbnailHotReload,
                      style: TextStyle(
                        fontSize: AppSizes.fontXS(context),
                        color: AppColors.textMuted,
                        height: 1.4,
                      ),
                    ),
                  ),
                ],
              ),
              SizedBox(height: AppSizes.paddingLG(context)),
              _actions(context, l10n),
            ],
          ),
        ),
      ),
    );
  }

  Widget _header(BuildContext context, AppLocalizations l10n) {
    return Row(
      children: [
        Icon(
          Icons.image_outlined,
          size: AppSizes.iconMD(context),
          color: AppColors.accentPrimary,
        ),
        SizedBox(width: AppSizes.spacingMD(context)),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                l10n.thumbnailTitle,
                style: TextStyle(
                  fontSize: AppSizes.fontMD(context),
                  fontWeight: FontWeight.w700,
                  color: AppColors.textPrimary,
                ),
              ),
              Text(
                widget.subtitle,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  fontFamily: AppSizes.monoFamily,
                  fontFamilyFallback: AppSizes.monoFallback,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  static const _maxPreviewSide = 420.0;

  double get _previewScale {
    final longest = _width > _height ? _width : _height;
    if (longest <= _maxPreviewSide) return 1;
    return _maxPreviewSide / longest;
  }

  Widget _canvas(BuildContext context, AppLocalizations l10n) {
    final scale = _previewScale;
    final shownWidth = _width * scale;
    final shownHeight = _height * scale;

    return SizedBox(
      width: shownWidth + 16,
      height: shownHeight + 16,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          DropTarget(
            onDragEntered: (_) => setState(() => _dragging = true),
            onDragExited: (_) => setState(() => _dragging = false),
            onDragDone: (detail) {
              setState(() => _dragging = false);
              _onDrop(detail.files.map((f) => f.path).toList());
            },
            child: MouseRegion(
              cursor: SystemMouseCursors.click,
              child: GestureDetector(
                onTap: _pick,
                child: Container(
                  width: shownWidth,
                  height: shownHeight,
                  decoration: BoxDecoration(
                    color: _dragging
                        ? AppColors.accentPrimary.withValues(alpha: 0.10)
                        : AppColors.surfaceLight,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(
                      color: _dragging
                          ? AppColors.accentPrimary
                          : AppColors.borderMedium,
                    ),
                  ),
                  child: _preview != null
                      ? ClipRRect(
                          borderRadius: BorderRadius.circular(3),
                          child: Image.memory(
                            _preview!,
                            fit: BoxFit.fill,
                            width: double.infinity,
                            height: double.infinity,
                          ),
                        )
                      : _emptyHint(context, l10n),
                ),
              ),
            ),
          ),
          Positioned(
            left: shownWidth - 8,
            top: shownHeight - 8,
            child: _resizeHandle(context, scale),
          ),
        ],
      ),
    );
  }

  Widget _emptyHint(BuildContext context, AppLocalizations l10n) {
    return Center(
      child: Padding(
        padding: EdgeInsets.all(AppSizes.paddingSM(context)),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.add_photo_alternate_outlined,
              size: AppSizes.iconLG(context),
              color: AppColors.textMuted,
            ),
            SizedBox(height: AppSizes.spacingSM(context)),
            Text(
              l10n.thumbnailDrop,
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _resizeHandle(BuildContext context, double scale) {
    return MouseRegion(
      cursor: SystemMouseCursors.resizeDownRight,
      child: GestureDetector(
        onPanUpdate: (details) {
          setState(() {
            _width = _clampSide((_width + details.delta.dx / scale).round());
            _height = _clampSide((_height + details.delta.dy / scale).round());
            _widthController.text = '$_width';
            _heightController.text = '$_height';
          });
        },
        child: Container(
          width: 16,
          height: 16,
          decoration: BoxDecoration(
            color: AppColors.accentPrimary,
            borderRadius: BorderRadius.circular(3),
          ),
          child: Icon(
            Icons.open_in_full,
            size: 10,
            color: AppColors.backgroundPrimary,
          ),
        ),
      ),
    );
  }

  Widget _sizeRow(BuildContext context, AppLocalizations l10n) {
    if (widget.fixedSize != null) {
      return Row(
        children: [
          Text(
            l10n.thumbnailSize,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textSecondary,
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Text(
            '$_width × $_height',
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              fontWeight: FontWeight.w600,
              color: AppColors.accentPrimary,
              fontFamily: AppSizes.monoFamily,
              fontFamilyFallback: AppSizes.monoFallback,
            ),
          ),
          SizedBox(width: AppSizes.spacingMD(context)),
          Expanded(
            child: Text(
              l10n.thumbnailSizeFixed,
              style: TextStyle(
                fontSize: AppSizes.fontXS(context),
                color: AppColors.textMuted,
                height: 1.3,
              ),
            ),
          ),
        ],
      );
    }

    return Row(
      children: [
        Text(
          l10n.thumbnailSize,
          style: TextStyle(
            fontSize: AppSizes.fontXS(context),
            color: AppColors.textSecondary,
          ),
        ),
        SizedBox(width: AppSizes.spacingMD(context)),
        _sizeField(context, _widthController, (value) {
          setState(() {
            _width = _clampSide(value);
            _widthController.text = '$_width';
          });
        }),
        Padding(
          padding: EdgeInsets.symmetric(
            horizontal: AppSizes.spacingSM(context),
          ),
          child: Text(
            '×',
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: AppColors.textMuted,
            ),
          ),
        ),
        _sizeField(context, _heightController, (value) {
          setState(() {
            _height = _clampSide(value);
            _heightController.text = '$_height';
          });
        }),
        SizedBox(width: AppSizes.spacingMD(context)),
        Expanded(
          child: Text(
            _previewScale < 1
                ? l10n.thumbnailScaled(
                    (_previewScale * 100).round(),
                  )
                : l10n.thumbnailSizeHint,
            style: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textMuted,
              height: 1.3,
            ),
          ),
        ),
      ],
    );
  }

  Widget _sizeField(
    BuildContext context,
    TextEditingController controller,
    ValueChanged<int> onSubmitted,
  ) {
    return SizedBox(
      width: 64,
      child: Container(
        decoration: BoxDecoration(
          color: AppColors.inputBackground,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: AppColors.borderMedium),
        ),
        child: TextField(
          controller: controller,
          keyboardType: TextInputType.number,
          inputFormatters: [FilteringTextInputFormatter.digitsOnly],
          textAlign: TextAlign.center,
          onSubmitted: (text) {
            final value = int.tryParse(text);
            if (value != null) onSubmitted(value);
          },
          onEditingComplete: () {
            final value = int.tryParse(controller.text);
            if (value != null) onSubmitted(value);
          },
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textPrimary,
            fontFamily: AppSizes.monoFamily,
            fontFamilyFallback: AppSizes.monoFallback,
          ),
          decoration: const InputDecoration(
            isDense: true,
            border: InputBorder.none,
            contentPadding: EdgeInsets.symmetric(vertical: 6),
          ),
        ),
      ),
    );
  }

  Widget _actions(BuildContext context, AppLocalizations l10n) {
    return Row(
      children: [
        const Spacer(),
        HoverButton(
          label: MaterialLocalizations.of(context).cancelButtonLabel,
          color: AppColors.textMuted,
          onTap: () => Navigator.of(context).pop(),
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        HoverButton(
          label: l10n.thumbnailUse,
          color: _imagePath == null ? AppColors.textMuted : AppColors.success,
          onTap: () {
            if (_imagePath == null) return;
            _confirm();
          },
        ),
      ],
    );
  }
}
