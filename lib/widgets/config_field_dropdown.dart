import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigFieldDropdown extends StatelessWidget {
  final String label;
  final int value;
  final List<int> options;
  final ValueChanged<int> onChanged;
  final String? tooltip;

  const ConfigFieldDropdown({
    super.key,
    required this.label,
    required this.value,
    required this.options,
    required this.onChanged,
    this.tooltip,
  });

  @override
  Widget build(BuildContext context) {
    final row = Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: const TextStyle(
                fontSize: AppSizes.fontMD,
                color: AppColors.textPrimary,
              ),
            ),
          ),
          const SizedBox(width: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(
              color: AppColors.inputBackground,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: AppColors.borderMedium),
            ),
            child: DropdownButton<int>(
              value: options.contains(value) ? value : options.first,
              items: options
                  .map(
                    (v) => DropdownMenuItem(
                      value: v,
                      child: Text(
                        '$v',
                        style: const TextStyle(
                          fontSize: AppSizes.fontMD,
                          color: AppColors.textPrimary,
                        ),
                      ),
                    ),
                  )
                  .toList(),
              onChanged: (v) {
                if (v != null) onChanged(v);
              },
              dropdownColor: AppColors.backgroundCard,
              underline: const SizedBox.shrink(),
              isDense: true,
            ),
          ),
        ],
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
