import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigErrorBanner extends StatelessWidget {
  final String fileName;
  final String error;

  const ConfigErrorBanner({
    super.key,
    required this.fileName,
    required this.error,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Container(
      width: double.infinity,
      margin: EdgeInsets.only(bottom: AppSizes.paddingMD(context)),
      padding: EdgeInsets.symmetric(
        horizontal: AppSizes.paddingMD(context),
        vertical: AppSizes.paddingSM(context),
      ),
      decoration: BoxDecoration(
        color: AppColors.error.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
        border: Border.all(color: AppColors.error.withValues(alpha: 0.5)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.error_outline,
            size: AppSizes.iconMD(context),
            color: AppColors.error,
          ),
          SizedBox(width: AppSizes.spacingSM(context)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.configParseErrorTitle(fileName),
                  style: TextStyle(
                    fontSize: AppSizes.fontMD(context),
                    fontWeight: FontWeight.w700,
                    color: AppColors.error,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  l10n.configParseErrorBody,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textSecondary,
                    height: 1.35,
                  ),
                ),
                SizedBox(height: AppSizes.spacingSM(context)),
                SelectableText(
                  error,
                  style: TextStyle(
                    fontSize: AppSizes.fontXS(context),
                    color: AppColors.textMuted,
                    height: 1.35,
                    fontFamily: AppSizes.monoFamily,
                    fontFamilyFallback: AppSizes.monoFallback,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
