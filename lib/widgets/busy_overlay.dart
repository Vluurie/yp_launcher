import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class BusyOverlay extends StatelessWidget {
  final String message;

  const BusyOverlay({super.key, required this.message});

  @override
  Widget build(BuildContext context) {
    return Positioned.fill(
      child: AbsorbPointer(
        child: Container(
          color: Colors.black.withValues(alpha: 0.4),
          alignment: Alignment.center,
          child: Container(
            padding: EdgeInsets.symmetric(
              horizontal: AppSizes.paddingXL(context),
              vertical: AppSizes.paddingLG(context),
            ),
            decoration: BoxDecoration(
              color: AppColors.backgroundCard,
              borderRadius:
                  BorderRadius.circular(AppSizes.borderRadius(context)),
              border: Border.all(color: AppColors.borderLight),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(
                  width: AppSizes.iconLG(context),
                  height: AppSizes.iconLG(context),
                  child: const CircularProgressIndicator(
                    strokeWidth: 2,
                    color: AppColors.accentPrimary,
                  ),
                ),
                SizedBox(width: AppSizes.spacingLG(context)),
                Flexible(
                  child: Text(
                    message,
                    style: TextStyle(
                      color: AppColors.textPrimary,
                      fontSize: AppSizes.fontMD(context),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

Future<bool> showDeleteConfirm(
  BuildContext context, {
  required String title,
  required String body,
  required String deleteLabel,
}) async {
  final result = await showDialog<bool>(
    context: context,
    builder: (ctx) => AlertDialog(
      backgroundColor: AppColors.backgroundCard,
      title: Text(
        title,
        style: TextStyle(
          color: AppColors.error,
          fontSize: AppSizes.fontLG(ctx),
        ),
      ),
      content: Text(
        body,
        style: TextStyle(
          color: AppColors.textSecondary,
          fontSize: AppSizes.fontSM(ctx),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(false),
          child: Text(
            'Cancel',
            style: TextStyle(
              color: AppColors.textMuted,
              fontSize: AppSizes.fontSM(ctx),
            ),
          ),
        ),
        TextButton(
          onPressed: () => Navigator.of(ctx).pop(true),
          child: Text(
            deleteLabel,
            style: TextStyle(
              color: AppColors.error,
              fontSize: AppSizes.fontSM(ctx),
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ],
    ),
  );
  return result == true;
}
