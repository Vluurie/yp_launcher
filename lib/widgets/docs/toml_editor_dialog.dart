import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class TomlEditorDialog extends StatefulWidget {
  final String title;
  final String fileName;
  final String explanation;
  final String initialContent;
  final String resetContent;

  const TomlEditorDialog({
    super.key,
    required this.title,
    required this.fileName,
    required this.explanation,
    required this.initialContent,
    required this.resetContent,
  });

  static Future<String?> show(
    BuildContext context, {
    required String title,
    required String fileName,
    required String explanation,
    required String initialContent,
    required String resetContent,
  }) {
    return showDialog<String>(
      context: context,
      builder: (_) => TomlEditorDialog(
        title: title,
        fileName: fileName,
        explanation: explanation,
        initialContent: initialContent,
        resetContent: resetContent,
      ),
    );
  }

  @override
  State<TomlEditorDialog> createState() => _TomlEditorDialogState();
}

class _TomlEditorDialogState extends State<TomlEditorDialog> {
  late final TextEditingController _controller;
  final _editorScroll = ScrollController();
  Map<String, dynamic> _parsed = const {};
  bool _valid = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialContent);
    _validate(widget.initialContent);
  }

  @override
  void dispose() {
    _controller.dispose();
    _editorScroll.dispose();
    super.dispose();
  }

  void _validate(String text) {
    final result = TomlService.parseStrict(text);
    setState(() {
      _parsed = result.values;
      _valid = result.ok;
      _error = result.error;
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Dialog(
      backgroundColor: AppColors.backgroundCard,
      insetPadding: EdgeInsets.all(AppSizes.paddingXL(context)),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 820, maxHeight: 720),
        child: Padding(
          padding: EdgeInsets.all(AppSizes.paddingLG(context)),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              _header(context),
              SizedBox(height: AppSizes.spacingSM(context)),
              Text(
                widget.explanation,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  height: 1.5,
                ),
              ),
              SizedBox(height: AppSizes.paddingMD(context)),
              Flexible(child: _editor(context)),
              SizedBox(height: AppSizes.spacingMD(context)),
              _preview(context, l10n),
              SizedBox(height: AppSizes.paddingMD(context)),
              _actions(context, l10n),
            ],
          ),
        ),
      ),
    );
  }

  Widget _header(BuildContext context) {
    return Row(
      children: [
        Icon(
          Icons.description_outlined,
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
              Text(
                widget.fileName,
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

  Widget _editor(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: AppColors.surfaceLight,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
          color: _valid ? AppColors.borderLight : AppColors.error,
        ),
      ),
      child: TextField(
        controller: _controller,
        onChanged: _validate,
        maxLines: null,
        keyboardType: TextInputType.multiline,
        scrollController: _editorScroll,
        style: TextStyle(
          fontSize: AppSizes.fontSM(context),
          fontFamily: AppSizes.monoFamily,
          fontFamilyFallback: AppSizes.monoFallback,
          color: AppColors.textPrimary,
          height: 1.5,
        ),
        decoration: InputDecoration(
          isDense: true,
          border: InputBorder.none,
          contentPadding: EdgeInsets.all(AppSizes.paddingMD(context)),
        ),
      ),
    );
  }

  Widget _preview(BuildContext context, AppLocalizations l10n) {
    if (!_valid) {
      return Container(
        width: double.infinity,
        padding: EdgeInsets.all(AppSizes.paddingMD(context)),
        decoration: BoxDecoration(
          color: AppColors.error.withValues(alpha: 0.08),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: AppColors.error.withValues(alpha: 0.5)),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(
              Icons.error_outline,
              size: AppSizes.iconSM(context),
              color: AppColors.error,
            ),
            SizedBox(width: AppSizes.spacingSM(context)),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.tomlEditorInvalid,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      color: AppColors.error,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  if (_error != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 3),
                      child: Text(
                        _error!,
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          color: AppColors.textMuted,
                          fontFamily: AppSizes.monoFamily,
                          fontFamilyFallback: AppSizes.monoFallback,
                          height: 1.4,
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ],
        ),
      );
    }

    final keys = _parsed.keys.toList();
    if (keys.isEmpty) return const SizedBox.shrink();

    return Container(
      width: double.infinity,
      padding: EdgeInsets.all(AppSizes.paddingMD(context)),
      decoration: BoxDecoration(
        color: AppColors.surfaceMedium,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.check_circle_outline,
                size: AppSizes.iconSM(context),
                color: AppColors.success,
              ),
              SizedBox(width: AppSizes.spacingSM(context)),
              Text(
                l10n.tomlEditorValid,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.success,
                ),
              ),
            ],
          ),
          SizedBox(height: AppSizes.spacingSM(context)),
          Wrap(
            spacing: AppSizes.spacingSM(context),
            runSpacing: AppSizes.spacingSM(context),
            children: [
              for (final key in keys)
                Container(
                  padding: EdgeInsets.symmetric(
                    horizontal: AppSizes.chipPaddingH(context),
                    vertical: AppSizes.chipPaddingV(context),
                  ),
                  decoration: BoxDecoration(
                    color: AppColors.surfaceLight,
                    borderRadius: BorderRadius.circular(999),
                    border: Border.all(color: AppColors.borderLight),
                  ),
                  child: Text(
                    _parsed[key] is Map ? '[$key]' : key,
                    style: TextStyle(
                      fontSize: AppSizes.fontXS(context),
                      fontFamily: AppSizes.monoFamily,
                      fontFamilyFallback: AppSizes.monoFallback,
                      color: _parsed[key] is Map
                          ? AppColors.accentPrimary
                          : AppColors.textSecondary,
                    ),
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _actions(BuildContext context, AppLocalizations l10n) {
    return Row(
      children: [
        HoverButton(
          label: l10n.tomlEditorReset,
          color: AppColors.textMuted,
          onTap: () {
            _controller.text = widget.resetContent;
            _validate(widget.resetContent);
          },
        ),
        const Spacer(),
        HoverButton(
          label: MaterialLocalizations.of(context).cancelButtonLabel,
          color: AppColors.textMuted,
          onTap: () => Navigator.of(context).pop(),
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        HoverButton(
          label: l10n.tomlEditorWrite,
          color: _valid ? AppColors.success : AppColors.textMuted,
          onTap: () {
            if (!_valid) return;
            Navigator.of(context).pop(_controller.text);
          },
        ),
      ],
    );
  }
}
