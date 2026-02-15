import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigFieldBool extends StatelessWidget {
  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;
  final String? tooltip;

  const ConfigFieldBool({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.tooltip,
  });

  @override
  Widget build(BuildContext context) {
    final row = Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: InkWell(
        onTap: () => onChanged(!value),
        borderRadius: BorderRadius.circular(4),
        child: Row(
          children: [
            SizedBox(
              width: AppSizes.checkboxSize,
              height: AppSizes.checkboxSize,
              child: Checkbox(
                value: value,
                onChanged: (v) => onChanged(v ?? false),
                activeColor: AppColors.accentPrimary,
                checkColor: AppColors.buttonText,
                side: const BorderSide(color: AppColors.borderMedium),
                materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                visualDensity: VisualDensity.compact,
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                label,
                style: const TextStyle(
                  fontSize: AppSizes.fontMD,
                  color: AppColors.textPrimary,
                ),
              ),
            ),
          ],
        ),
      ),
    );

    if (tooltip != null) {
      return Tooltip(
        richMessage: WidgetSpan(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 280),
            child: Text(
              tooltip!,
              style: const TextStyle(fontSize: AppSizes.fontXS, color: Colors.white),
            ),
          ),
        ),
        decoration: BoxDecoration(
          color: const Color(0xE6303030),
          borderRadius: BorderRadius.circular(6),
        ),
        child: row,
      );
    }
    return row;
  }
}
