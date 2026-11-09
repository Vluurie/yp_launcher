import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/hover_button.dart';

class MarkdownEditorDialog extends StatefulWidget {
  final String fileName;
  final String filePath;
  final String initialContent;
  final String saveLabel;

  const MarkdownEditorDialog({
    super.key,
    required this.fileName,
    required this.filePath,
    required this.initialContent,
    required this.saveLabel,
  });

  static Future<String?> show(
    BuildContext context, {
    required String fileName,
    required String filePath,
    required String initialContent,
    required String saveLabel,
  }) {
    return showDialog<String>(
      context: context,
      builder: (_) => MarkdownEditorDialog(
        fileName: fileName,
        filePath: filePath,
        initialContent: initialContent,
        saveLabel: saveLabel,
      ),
    );
  }

  @override
  State<MarkdownEditorDialog> createState() => _MarkdownEditorDialogState();
}

class _MarkdownEditorDialogState extends State<MarkdownEditorDialog> {
  late final TextEditingController _controller;
  final _editorScroll = ScrollController();
  final _previewScroll = ScrollController();
  late String _preview;
  bool _dirty = false;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialContent);
    _preview = widget.initialContent;
  }

  @override
  void dispose() {
    _controller.dispose();
    _editorScroll.dispose();
    _previewScroll.dispose();
    super.dispose();
  }

  void _save() {
    Navigator.of(context).pop(_controller.text);
  }

  @override
  Widget build(BuildContext context) {
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.keyS, control: true): _save,
        const SingleActivator(LogicalKeyboardKey.keyS, meta: true): _save,
      },
      child: Focus(autofocus: true, child: _dialog(context)),
    );
  }

  Widget _dialog(BuildContext context) {
    return Dialog(
      backgroundColor: AppColors.backgroundCard,
      insetPadding: EdgeInsets.all(AppSizes.paddingLG(context)),
      child: Padding(
        padding: EdgeInsets.all(AppSizes.paddingLG(context)),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _header(context),
            SizedBox(height: AppSizes.paddingMD(context)),
            Expanded(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Expanded(child: _editor(context)),
                  SizedBox(width: AppSizes.spacingMD(context)),
                  Expanded(child: _previewPane(context)),
                ],
              ),
            ),
            SizedBox(height: AppSizes.paddingMD(context)),
            _actions(context),
          ],
        ),
      ),
    );
  }

  Widget _header(BuildContext context) {
    return Row(
      children: [
        Icon(
          Icons.edit_note,
          size: AppSizes.iconMD(context),
          color: AppColors.accentPrimary,
        ),
        SizedBox(width: AppSizes.spacingMD(context)),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(
                    widget.fileName,
                    style: TextStyle(
                      fontSize: AppSizes.fontMD(context),
                      fontWeight: FontWeight.w700,
                      color: AppColors.textPrimary,
                    ),
                  ),
                  if (_dirty)
                    Padding(
                      padding: const EdgeInsets.only(left: 6),
                      child: Text(
                        '*',
                        style: TextStyle(
                          fontSize: AppSizes.fontMD(context),
                          fontWeight: FontWeight.bold,
                          color: AppColors.warning,
                        ),
                      ),
                    ),
                ],
              ),
              Text(
                widget.filePath,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
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
        border: Border.all(color: AppColors.borderLight),
      ),
      child: TextField(
        controller: _controller,
        onChanged: (v) => setState(() {
          _dirty = v != widget.initialContent;
        }),
        maxLines: null,
        expands: true,
        textAlignVertical: TextAlignVertical.top,
        keyboardType: TextInputType.multiline,
        scrollController: _editorScroll,
        style: TextStyle(
          fontSize: AppSizes.fontXS(context),
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

  Widget _previewPane(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(AppSizes.paddingMD(context)),
      decoration: BoxDecoration(
        color: AppColors.backgroundPrimary,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: AppColors.borderLight),
      ),
      child: SingleChildScrollView(
        controller: _previewScroll,
        child: MarkdownBody(
          data: _preview,
          softLineBreak: true,
          styleSheet: MarkdownStyleSheet(
            p: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textSecondary,
              height: 1.6,
            ),
            h1: TextStyle(
              fontSize: AppSizes.fontMD(context),
              color: AppColors.textPrimary,
              fontWeight: FontWeight.bold,
            ),
            h2: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: AppColors.textPrimary,
              fontWeight: FontWeight.w700,
            ),
            code: TextStyle(
              fontSize: AppSizes.fontXS(context),
              fontFamily: AppSizes.monoFamily,
              fontFamilyFallback: AppSizes.monoFallback,
              color: AppColors.accentPrimary,
            ),
            codeblockDecoration: BoxDecoration(
              color: AppColors.surfaceLight,
              borderRadius: BorderRadius.circular(4),
            ),
            listBullet: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.accentPrimary,
            ),
            tableBody: TextStyle(
              fontSize: AppSizes.fontXS(context),
              color: AppColors.textSecondary,
            ),
            tableBorder: TableBorder.all(color: AppColors.borderLight),
          ),
        ),
      ),
    );
  }

  Widget _actions(BuildContext context) {
    return Row(
      children: [
        HoverButton(
          label: AppLocalizations.of(context)!.docsPreview,
          color: AppColors.accentPrimary,
          onTap: () => setState(() => _preview = _controller.text),
        ),
        const Spacer(),
        HoverButton(
          label: MaterialLocalizations.of(context).cancelButtonLabel,
          color: AppColors.textMuted,
          onTap: () => Navigator.of(context).pop(),
        ),
        SizedBox(width: AppSizes.spacingSM(context)),
        Tooltip(
          message: 'Ctrl+S',
          child: HoverButton(
            label: widget.saveLabel,
            color: AppColors.success,
            onTap: _save,
          ),
        ),
      ],
    );
  }
}
