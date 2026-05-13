import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/config_apply_marker.dart';

class ConfigFieldSlider extends StatelessWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final double step;
  final int decimals;
  final ValueChanged<double> onChanged;
  final String? tooltip;
  final bool restartRequired;
  final bool showApplyMarker;

  const ConfigFieldSlider({
    super.key,
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.step,
    required this.onChanged,
    this.decimals = 2,
    this.tooltip,
    this.restartRequired = false,
    this.showApplyMarker = false,
  });

  @override
  Widget build(BuildContext context) {
    final clamped = value.clamp(min, max).toDouble();
    final divisions = ((max - min) / step).round().clamp(1, 10000);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Row(
                  children: [
                    Flexible(
                      child: Text(
                        label,
                        style: TextStyle(
                          fontSize: AppSizes.fontMD(context),
                          color: AppColors.textPrimary,
                        ),
                      ),
                    ),
                    if (showApplyMarker) ...[
                      const SizedBox(width: 6),
                      ConfigApplyMarker(restartRequired: restartRequired),
                    ],
                  ],
                ),
              ),
              const SizedBox(width: 12),
              Text(
                clamped.toStringAsFixed(decimals),
                style: TextStyle(
                  fontSize: AppSizes.fontMD(context),
                  color: AppColors.accentPrimary,
                  fontWeight: FontWeight.w600,
                  fontFeatures: const [FontFeature.tabularFigures()],
                ),
              ),
            ],
          ),
          if (tooltip != null)
            Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Text(
                tooltip!,
                style: TextStyle(
                  fontSize: AppSizes.fontXS(context),
                  color: AppColors.textMuted,
                  height: 1.3,
                ),
              ),
            ),
          SliderTheme(
            data: SliderTheme.of(context).copyWith(
              trackHeight: 3,
              activeTrackColor: AppColors.accentPrimary,
              inactiveTrackColor: AppColors.borderLight,
              thumbColor: AppColors.accentPrimary,
              overlayColor: AppColors.accentPrimary.withValues(alpha: 0.15),
              thumbShape:
                  const RoundSliderThumbShape(enabledThumbRadius: 7),
              overlayShape:
                  const RoundSliderOverlayShape(overlayRadius: 16),
              valueIndicatorColor: AppColors.accentPrimary,
            ),
            child: Slider(
              value: clamped,
              min: min,
              max: max,
              divisions: divisions,
              onChanged: (v) {
                final snapped = (v / step).round() * step;
                final out = snapped.clamp(min, max).toDouble();
                onChanged(out);
              },
            ),
          ),
        ],
      ),
    );
  }
}
