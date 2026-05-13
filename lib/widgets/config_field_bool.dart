import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_apply_marker.dart';

class ConfigFieldBool extends StatefulWidget {
  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;
  final String? tooltip;
  final bool restartRequired;
  final bool showApplyMarker;

  const ConfigFieldBool({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.tooltip,
    this.restartRequired = false,
    this.showApplyMarker = false,
  });

  @override
  State<ConfigFieldBool> createState() => _ConfigFieldBoolState();
}

class _ConfigFieldBoolState extends State<ConfigFieldBool> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: () => widget.onChanged(!widget.value),
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
              Padding(
                padding: const EdgeInsets.only(top: 2),
                child: SizedBox(
                  width: 36,
                  height: 20,
                  child: FittedBox(
                    fit: BoxFit.contain,
                    child: Switch(
                      value: widget.value,
                      onChanged: (v) => widget.onChanged(v),
                      activeThumbColor: AppColors.accentPrimary,
                      activeTrackColor: AppColors.accentPrimary.withValues(
                        alpha: 0.4,
                      ),
                      inactiveThumbColor: AppColors.textMuted,
                      inactiveTrackColor: AppColors.borderLight,
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 12),
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
            ],
          ),
        ),
      ),
    );
  }
}
