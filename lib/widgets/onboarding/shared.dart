import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class OnboardingStepCard extends StatelessWidget {
  final List<Widget> children;
  const OnboardingStepCard({super.key, required this.children});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 24),
        padding: const EdgeInsets.symmetric(horizontal: 36, vertical: 28),
        decoration: BoxDecoration(
          color: AppColors.backgroundCard.withValues(alpha: 0.85),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: AppColors.borderLight),
        ),
        child: Column(mainAxisSize: MainAxisSize.min, children: children),
      ),
    );
  }
}

class OnboardingChoiceButton extends StatelessWidget {
  final String label;
  final bool selected;
  final VoidCallback onTap;
  const OnboardingChoiceButton({
    super.key,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      height: 52,
      child: OutlinedButton(
        onPressed: onTap,
        style: OutlinedButton.styleFrom(
          backgroundColor: selected
              ? AppColors.accentPrimary.withValues(alpha: 0.12)
              : Colors.transparent,
          foregroundColor: selected
              ? AppColors.accentPrimary
              : AppColors.textSecondary,
          side: BorderSide(
            color: selected ? AppColors.accentPrimary : AppColors.borderMedium,
            width: selected ? 2.0 : 1.0,
          ),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            if (selected)
              const Padding(
                padding: EdgeInsets.only(right: 10),
                child: Icon(
                  Icons.check_circle,
                  size: 20,
                  color: AppColors.accentPrimary,
                ),
              ),
            Text(
              label,
              style: TextStyle(
                fontSize: AppSizes.fontMD(context),
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class OnboardingBackButton extends StatelessWidget {
  final VoidCallback onTap;
  const OnboardingBackButton({super.key, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 48,
      child: TextButton.icon(
        onPressed: onTap,
        icon: const Icon(
          Icons.arrow_back,
          size: 18,
          color: AppColors.textMuted,
        ),
        label: Text(
          AppLocalizations.of(context)!.buttonBack,
          style: TextStyle(
            fontSize: AppSizes.fontMD(context),
            color: AppColors.textMuted,
          ),
        ),
      ),
    );
  }
}

class OnboardingDots extends StatelessWidget {
  final int currentStep;
  final int totalSteps;
  const OnboardingDots({
    super.key,
    required this.currentStep,
    required this.totalSteps,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: List.generate(totalSteps, (i) {
        final isActive = i == currentStep;
        return AnimatedContainer(
          duration: const Duration(milliseconds: 250),
          margin: const EdgeInsets.symmetric(horizontal: 4),
          width: isActive ? 24 : 8,
          height: 8,
          decoration: BoxDecoration(
            color: isActive ? AppColors.accentPrimary : AppColors.borderMedium,
            borderRadius: BorderRadius.circular(4),
          ),
        );
      }),
    );
  }
}

class OnboardingSummaryRow extends StatelessWidget {
  final IconData icon;
  final String label;
  final String value;
  const OnboardingSummaryRow({
    super.key,
    required this.icon,
    required this.label,
    required this.value,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 16, color: AppColors.accentPrimary),
        const SizedBox(width: 8),
        Text(
          '$label: ',
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textMuted,
          ),
        ),
        Expanded(
          child: Text(
            value,
            style: TextStyle(
              fontSize: AppSizes.fontSM(context),
              color: AppColors.textPrimary,
            ),
            overflow: TextOverflow.ellipsis,
          ),
        ),
      ],
    );
  }
}
