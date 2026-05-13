import 'package:flutter/material.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/detection_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/onboarding/shared.dart';

class MigrationStep extends StatefulWidget {
  final ReShadeStatus reshadeStatus;
  final List<String> textureResults;
  final bool hasLodMod;
  final bool hasSkRes;
  final bool hasNaiom;
  final bool hasCutsceneMods;
  final int existingNamsMods;
  final VoidCallback onNext;
  final VoidCallback onBack;
  const MigrationStep({
    super.key,
    required this.reshadeStatus,
    required this.textureResults,
    required this.hasLodMod,
    required this.hasSkRes,
    required this.hasNaiom,
    required this.hasCutsceneMods,
    required this.existingNamsMods,
    required this.onNext,
    required this.onBack,
  });

  @override
  State<MigrationStep> createState() => _MigrationStepState();
}

class _MigrationStepState extends State<MigrationStep> {
  bool? _hadModsBefore;

  @override
  Widget build(BuildContext context) {
    if (_hadModsBefore == true) return _buildResults();
    return _buildAsk();
  }

  Widget _buildAsk() {
    final l = AppLocalizations.of(context)!;
    return OnboardingStepCard(
      children: [
        Text(
          l.onboardingMigrationAskTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXL(context),
            fontWeight: FontWeight.bold,
            color: AppColors.accentPrimary,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 12),
        Text(
          l.onboardingMigrationAskBody,
          style: TextStyle(
            fontSize: AppSizes.fontSM(context),
            color: AppColors.textSecondary,
            height: 1.5,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 28),
        OnboardingChoiceButton(
          label: l.onboardingMigrationYes,
          selected: false,
          onTap: () => setState(() => _hadModsBefore = true),
        ),
        const SizedBox(height: 10),
        OnboardingChoiceButton(
          label: l.onboardingMigrationNo,
          selected: false,
          onTap: widget.onNext,
        ),
        const SizedBox(height: 28),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [OnboardingBackButton(onTap: widget.onBack)],
        ),
      ],
    );
  }

  Widget _buildResults() {
    final l = AppLocalizations.of(context)!;
    final rows = <_MigrationRow>[];

    if (widget.existingNamsMods > 0) {
      rows.add(_MigrationRow(
        icon: Icons.extension_outlined,
        label: l.onboardingMigrationLabelExistingMods(widget.existingNamsMods),
        action: l.onboardingMigrationActionExistingMods,
        ok: true,
      ));
    }
    if (widget.reshadeStatus == ReShadeStatus.detected) {
      rows.add(_MigrationRow(
        icon: Icons.auto_fix_high,
        label: l.detectionReShade,
        action: l.onboardingMigrationActionReshadeKept,
        ok: true,
      ));
    } else if (widget.reshadeStatus == ReShadeStatus.incompatibleAddon) {
      rows.add(_MigrationRow(
        icon: Icons.auto_fix_high,
        label: l.detectionReShade,
        action: l.onboardingMigrationActionReshadeIncompatible,
        ok: false,
      ));
    }
    if (widget.textureResults.isNotEmpty) {
      rows.add(_MigrationRow(
        icon: Icons.image,
        label: l.detectionHDTextures,
        action: l.onboardingMigrationActionTextures,
        ok: true,
      ));
    }
    if (widget.hasLodMod) {
      rows.add(_MigrationRow(
        icon: Icons.visibility,
        label: l.detectionLodMod,
        action: l.onboardingMigrationActionLodMod,
        ok: true,
      ));
    }
    if (widget.hasSkRes) {
      rows.add(_MigrationRow(
        icon: Icons.texture,
        label: l.detectionSkRes,
        action: l.onboardingMigrationActionSkRes,
        ok: true,
      ));
    }
    if (widget.hasNaiom) {
      rows.add(_MigrationRow(
        icon: Icons.mouse,
        label: l.detectionNaiom,
        action: l.onboardingMigrationActionNaiom,
        ok: false,
      ));
    }
    if (widget.hasCutsceneMods) {
      rows.add(_MigrationRow(
        icon: Icons.movie,
        label: l.detectionCutscenes,
        action: l.onboardingMigrationActionCutscenes,
        ok: true,
      ));
    }

    return OnboardingStepCard(
      children: [
        Text(
          l.onboardingMigrationResultsTitle,
          style: TextStyle(
            fontSize: AppSizes.fontXL(context),
            fontWeight: FontWeight.bold,
            color: AppColors.accentPrimary,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        if (rows.isEmpty)
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: AppColors.surfaceMedium,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              l.onboardingMigrationNothingFound,
              style: TextStyle(
                fontSize: AppSizes.fontSM(context),
                color: AppColors.textSecondary,
                height: 1.4,
              ),
              textAlign: TextAlign.center,
            ),
          )
        else
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: AppColors.surfaceMedium,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                for (var i = 0; i < rows.length; i++) ...[
                  if (i > 0) const SizedBox(height: 10),
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Icon(
                        rows[i].ok
                            ? Icons.check_circle_outline
                            : Icons.warning_amber_rounded,
                        size: 18,
                        color:
                            rows[i].ok ? AppColors.success : AppColors.warning,
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              rows[i].label,
                              style: TextStyle(
                                fontSize: AppSizes.fontSM(context),
                                fontWeight: FontWeight.w600,
                                color: AppColors.textPrimary,
                              ),
                            ),
                            const SizedBox(height: 2),
                            Text(
                              rows[i].action,
                              style: TextStyle(
                                fontSize: AppSizes.fontXS(context),
                                color: AppColors.textSecondary,
                                height: 1.35,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                ],
              ],
            ),
          ),
        const SizedBox(height: 24),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            OnboardingBackButton(onTap: () {
              setState(() => _hadModsBefore = null);
            }),
            const SizedBox(width: 12),
            SizedBox(
              width: 160,
              height: 48,
              child: ElevatedButton(
                onPressed: widget.onNext,
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.buttonBackground,
                  foregroundColor: AppColors.buttonText,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(6),
                  ),
                ),
                child: Text(
                  AppLocalizations.of(context)!.buttonContinue,
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

class _MigrationRow {
  final IconData icon;
  final String label;
  final String action;
  final bool ok;
  const _MigrationRow({
    required this.icon,
    required this.label,
    required this.action,
    required this.ok,
  });
}
