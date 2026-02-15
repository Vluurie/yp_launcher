import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class ConfigSection extends StatelessWidget {
  final String title;
  final List<Widget> children;

  const ConfigSection({
    super.key,
    required this.title,
    required this.children,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: const TextStyle(
              fontSize: AppSizes.fontSM,
              fontWeight: FontWeight.bold,
              color: AppColors.accentPrimary,
              letterSpacing: 1.0,
            ),
          ),
          const Divider(color: AppColors.borderLight, height: 8),
          ...children,
        ],
      ),
    );
  }
}
