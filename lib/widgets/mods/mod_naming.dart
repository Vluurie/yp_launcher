import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

String prettifyModId(String suggested) {
  var s = suggested.replaceAll(RegExp(r'[-_]\d+([-_]\d+)*$'), '');
  s = s.replaceAll(RegExp(r'[_]+'), ' ').trim();
  if (s.isEmpty) return suggested;
  return s
      .split(' ')
      .map((w) => w.isEmpty ? w : w[0].toUpperCase() + w.substring(1))
      .join(' ');
}

Future<String?> showModNamingDialog(
  BuildContext context, {
  required String initial,
  String? existsId,
  String? title,
}) async {
  final l10n = AppLocalizations.of(context)!;
  final controller = TextEditingController(text: initial);
  return showDialog<String>(
    context: context,
    builder: (ctx) {
      return AlertDialog(
        backgroundColor: AppColors.backgroundCard,
        title: Text(
          title ?? l10n.modInstallNeedsName,
          style: TextStyle(
            color: AppColors.accentPrimary,
            fontSize: AppSizes.fontLG(ctx),
          ),
        ),
        content: SizedBox(
          width: 360,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (existsId != null)
                Padding(
                  padding: EdgeInsets.only(bottom: AppSizes.paddingMD(ctx)),
                  child: Text(
                    l10n.modInstallExistsPickAnother(existsId),
                    style: TextStyle(
                      color: AppColors.warning,
                      fontSize: AppSizes.fontSM(ctx),
                    ),
                  ),
                ),
              TextField(
                controller: controller,
                autofocus: true,
                maxLength: 64,
                inputFormatters: [
                  FilteringTextInputFormatter.allow(
                    RegExp(r'[a-zA-Z0-9 _-]'),
                  ),
                ],
                style: TextStyle(
                  color: AppColors.textPrimary,
                  fontSize: AppSizes.fontMD(ctx),
                ),
                decoration: InputDecoration(
                  isDense: true,
                  filled: true,
                  fillColor: AppColors.inputBackground,
                  counterText: '',
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(
                      AppSizes.borderRadius(ctx),
                    ),
                    borderSide: BorderSide.none,
                  ),
                ),
                onSubmitted: (v) => Navigator.of(ctx).pop(
                  v.trim().isEmpty ? null : v.trim(),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(
              l10n.buttonCancel,
              style: TextStyle(
                color: AppColors.textMuted,
                fontSize: AppSizes.fontSM(ctx),
              ),
            ),
          ),
          TextButton(
            onPressed: () {
              final v = controller.text.trim();
              Navigator.of(ctx).pop(v.isEmpty ? null : v);
            },
            child: Text(
              'OK',
              style: TextStyle(
                color: AppColors.accentPrimary,
                fontSize: AppSizes.fontSM(ctx),
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      );
    },
  );
}
