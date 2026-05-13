import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/nams_settings_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/onboarding/shared.dart';

class FirstPlaythroughStep extends StatefulWidget {
  final VoidCallback onNext;
  final VoidCallback onBack;
  const FirstPlaythroughStep({
    super.key,
    required this.onNext,
    required this.onBack,
  });

  @override
  State<FirstPlaythroughStep> createState() => _FirstPlaythroughStepState();
}

class _FirstPlaythroughStepState extends State<FirstPlaythroughStep> {
  bool _firstPlaythrough = true;

  @override
  void initState() {
    super.initState();
    _loadExistingConfig();
  }

  Future<void> _loadExistingConfig() async {
    final settings = await NamsSettingsService.loadSettings();
    if (mounted) {
      setState(() {
        _firstPlaythrough = settings['firstPlaythrough'] ?? true;
      });
    }
  }

  Future<void> _saveFirstPlaythrough() async {
    final settings = await NamsSettingsService.loadSettings();
    settings['firstPlaythrough'] = _firstPlaythrough;
    await NamsSettingsService.saveSettings(settings);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    return OnboardingStepCard(
      children: [
        Text(
          l.onboardingFirstPlaythroughTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXL(context),
            fontWeight: FontWeight.bold,
            color: AppColors.accentPrimary,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 12),
        Text(
          l.onboardingFirstPlaythroughHint,
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textMuted,
            height: 1.5,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 28),
        OnboardingChoiceButton(
          label: l.onboardingFirstYes,
          selected: _firstPlaythrough,
          onTap: () => setState(() => _firstPlaythrough = true),
        ),
        const SizedBox(height: 10),
        OnboardingChoiceButton(
          label: l.onboardingFirstNo,
          selected: !_firstPlaythrough,
          onTap: () => setState(() => _firstPlaythrough = false),
        ),
        const SizedBox(height: 28),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            OnboardingBackButton(onTap: widget.onBack),
            const SizedBox(width: 12),
            SizedBox(
              width: 160,
              height: 48,
              child: ElevatedButton(
                onPressed: () async {
                  await _saveFirstPlaythrough();
                  if (!mounted) return;
                  widget.onNext();
                },
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.buttonBackground,
                  foregroundColor: AppColors.buttonText,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(6),
                  ),
                ),
                child: Text(
                  l.buttonContinue,
                  style: TextStyle(
                    fontSize: AppSizes.fontLG(context),
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}
