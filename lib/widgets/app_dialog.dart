import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class AppDialog extends StatelessWidget {
  final Widget? title;
  final Widget? content;
  final List<Widget> actions;
  final Color? backgroundColor;
  final ShapeBorder? shape;
  final EdgeInsetsGeometry? contentPadding;

  const AppDialog({
    super.key,
    this.title,
    this.content,
    this.actions = const [],
    this.backgroundColor,
    this.shape,
    this.contentPadding,
  });

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: backgroundColor ?? AppColors.backgroundCard,
      shape:
          shape ??
          RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(AppSizes.borderRadius(context)),
          ),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: AppSizes.dialogWidth(context)),
        child: Padding(
          padding: EdgeInsets.all(AppSizes.paddingLG(context)),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (title != null)
                DefaultTextStyle(
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    color: AppColors.textPrimary,
                    fontWeight: FontWeight.bold,
                  ),
                  child: title!,
                ),
              if (title != null && content != null)
                SizedBox(height: AppSizes.spacingMD(context)),
              if (content != null)
                Flexible(
                  child: Padding(
                    padding: contentPadding ?? EdgeInsets.zero,
                    child: DefaultTextStyle(
                      style: TextStyle(
                        fontSize: AppSizes.fontSM(context),
                        color: AppColors.textSecondary,
                        height: 1.4,
                      ),
                      child: content!,
                    ),
                  ),
                ),
              if (actions.isNotEmpty) ...[
                SizedBox(height: AppSizes.spacingLG(context)),
                Wrap(
                  alignment: WrapAlignment.end,
                  spacing: AppSizes.spacingSM(context),
                  runSpacing: AppSizes.spacingSM(context),
                  children: actions,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
