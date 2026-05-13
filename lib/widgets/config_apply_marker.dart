import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';

class ConfigApplyMarker extends StatelessWidget {
  final bool restartRequired;
  const ConfigApplyMarker({super.key, required this.restartRequired});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final icon = restartRequired ? Icons.restart_alt : Icons.bolt;
    final tip = restartRequired
        ? l10n.configAppliesOnRestart
        : l10n.configAppliesLive;
    final color = restartRequired ? AppColors.warning : AppColors.success;
    return Tooltip(
      message: tip,
      child: Icon(
        icon,
        size: 14,
        color: color.withValues(alpha: 0.85),
      ),
    );
  }
}
