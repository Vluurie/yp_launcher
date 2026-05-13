import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/app_dropdown.dart';
import 'package:yp_launcher/widgets/config_apply_marker.dart';

class ConfigFieldDropdown extends StatefulWidget {
  final String label;
  final int value;
  final List<int> options;
  final ValueChanged<int> onChanged;
  final String? tooltip;
  final Map<int, String>? labels;
  final bool restartRequired;
  final bool showApplyMarker;

  const ConfigFieldDropdown({
    super.key,
    required this.label,
    required this.value,
    required this.options,
    required this.onChanged,
    this.tooltip,
    this.labels,
    this.restartRequired = false,
    this.showApplyMarker = false,
  });

  @override
  State<ConfigFieldDropdown> createState() => _ConfigFieldDropdownState();
}

class _ConfigFieldDropdownState extends State<ConfigFieldDropdown> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 8),
        margin: const EdgeInsets.symmetric(vertical: 2),
        decoration: BoxDecoration(
          color: _hovered ? AppColors.surfaceLight : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Flexible(
                        child: Text(
                          widget.label,
                          style: TextStyle(
                            fontSize: AppSizes.fontMD(context),
                            color: AppColors.textPrimary,
                          ),
                        ),
                      ),
                      if (widget.showApplyMarker) ...[
                        const SizedBox(width: 6),
                        ConfigApplyMarker(
                          restartRequired: widget.restartRequired,
                        ),
                      ],
                    ],
                  ),
                  if (widget.tooltip != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 3),
                      child: Text(
                        widget.tooltip!,
                        style: TextStyle(
                          fontSize: AppSizes.fontXS(context),
                          color: AppColors.textMuted,
                          height: 1.3,
                        ),
                      ),
                    ),
                ],
              ),
            ),
            const SizedBox(width: 12),
            AppDropdown<int>(
              value: widget.value,
              items: widget.options,
              itemLabel: (v) => widget.labels?[v] ?? '$v',
              onChanged: widget.onChanged,
            ),
          ],
        ),
      ),
    );
  }
}
