import 'dart:io';
import 'package:yp_launcher/services/platform_gate.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/detection/reshade_detection.dart';
import 'package:yp_launcher/services/process_service.dart';
import 'package:yp_launcher/services/shortcut_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';
import 'package:yp_launcher/widgets/launch_failure_dialog.dart';
import 'package:yp_launcher/widgets/onboarding/shared.dart';

class ReadyDetections {
  final ReShadeStatus reshadeStatus;
  final List<String> textureResults;
  final bool hasLodMod;
  final bool hasSkRes;
  final bool hasNaiom;
  final bool hasCutsceneMods;
  final bool firstPlaythrough;
  const ReadyDetections({
    required this.reshadeStatus,
    required this.textureResults,
    required this.hasLodMod,
    required this.hasSkRes,
    required this.hasNaiom,
    required this.hasCutsceneMods,
    required this.firstPlaythrough,
  });
}

class ReadyStep extends ConsumerStatefulWidget {
  final String? selectedPath;
  final ReadyDetections detections;
  final Future<void> Function() onComplete;
  const ReadyStep({
    super.key,
    required this.selectedPath,
    required this.detections,
    required this.onComplete,
  });

  @override
  ConsumerState<ReadyStep> createState() => _ReadyStepState();
}

class _ReadyStepState extends ConsumerState<ReadyStep> {
  bool _createShortcut = true;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final d = widget.detections;
    return OnboardingStepCard(
      children: [
        Text(
          l.onboardingReadyTitle,
          style: TextStyle(
            fontSize: 28,
            fontWeight: FontWeight.bold,
            color: AppColors.accentPrimary,
            letterSpacing: 0.5,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: AppColors.surfaceMedium,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              OnboardingSummaryRow(
                icon: Icons.folder,
                label: l.labelGame,
                value: widget.selectedPath ?? l.detectionNotSet,
              ),
              const SizedBox(height: 6),
              OnboardingSummaryRow(
                icon: Icons.visibility,
                label: l.labelMode,
                value: d.firstPlaythrough
                    ? l.onboardingFirstPlaythroughSpoilerFree
                    : l.onboardingFullAccess,
              ),
              if (d.reshadeStatus != ReShadeStatus.notFound) ...[
                const SizedBox(height: 6),
                OnboardingSummaryRow(
                  icon: Icons.auto_fix_high,
                  label: l.detectionReShade,
                  value: d.reshadeStatus == ReShadeStatus.detected
                      ? l.detectionDetected
                      : l.detectionReShadeIncompatible,
                ),
              ],
              if (d.textureResults.isNotEmpty) ...[
                const SizedBox(height: 6),
                OnboardingSummaryRow(
                  icon: Icons.image,
                  label: l.detectionHDTextures,
                  value: d.textureResults.join(', '),
                ),
              ],
              if (d.hasLodMod) ...[
                const SizedBox(height: 6),
                OnboardingSummaryRow(
                  icon: Icons.visibility,
                  label: l.detectionLodMod,
                  value: l.detectionMigratedIntoNams,
                ),
              ],
              if (d.hasSkRes) ...[
                const SizedBox(height: 6),
                OnboardingSummaryRow(
                  icon: Icons.texture,
                  label: l.detectionSkRes,
                  value: l.detectionLoadedAutomatically,
                ),
              ],
              if (d.hasNaiom) ...[
                const SizedBox(height: 6),
                OnboardingSummaryRow(
                  icon: Icons.mouse,
                  label: l.detectionNaiom,
                  value: l.detectionMigrationComingSoon,
                ),
              ],
              if (d.hasCutsceneMods) ...[
                const SizedBox(height: 6),
                OnboardingSummaryRow(
                  icon: Icons.movie,
                  label: l.detectionCutscenes,
                  value: l.detectionInstalled,
                ),
              ],
            ],
          ),
        ),
        if (PlatformGate.isWindows) ...[
          const SizedBox(height: 16),
          GestureDetector(
            onTap: () => setState(() => _createShortcut = !_createShortcut),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                SizedBox(
                  width: 22,
                  height: 22,
                  child: Checkbox(
                    value: _createShortcut,
                    onChanged: (v) =>
                        setState(() => _createShortcut = v ?? false),
                    activeColor: AppColors.accentPrimary,
                    side: const BorderSide(color: AppColors.borderMedium),
                  ),
                ),
                const SizedBox(width: 10),
                Text(
                  l.onboardingCreateShortcut,
                  style: TextStyle(
                    fontSize: AppSizes.fontSM(context),
                    color: AppColors.textSecondary,
                  ),
                ),
              ],
            ),
          ),
        ],
        const SizedBox(height: 28),
        SizedBox(
          width: 220,
          height: 60,
          child: ElevatedButton(
            onPressed: () => _playAndComplete(context),
            style: ElevatedButton.styleFrom(
              backgroundColor: AppColors.buttonBackground,
              foregroundColor: AppColors.buttonText,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(6),
              ),
            ),
            child: Text(
              l.playButton,
              style: TextStyle(
                fontSize: AppSizes.fontXXL(context),
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ),
        const SizedBox(height: 12),
        TextButton(
          onPressed: () async {
            if (_createShortcut &&
                PlatformGate.isWindows &&
                widget.selectedPath != null) {
              await ShortcutService.createDesktopShortcut(
                gameDirectory: widget.selectedPath!,
              );
            }
            await widget.onComplete();
          },
          child: Text(
            l.buttonGoToLauncher,
            style: TextStyle(
              fontSize: AppSizes.fontMD(context),
              color: AppColors.textMuted,
              decoration: TextDecoration.underline,
              decorationColor: AppColors.textMuted,
            ),
          ),
        ),
      ],
    );
  }

  Future<void> _playAndComplete(BuildContext context) async {
    if (_createShortcut &&
        PlatformGate.isWindows &&
        widget.selectedPath != null) {
      await ShortcutService.createDesktopShortcut(
        gameDirectory: widget.selectedPath!,
      );
    }
    await widget.onComplete();

    final controller = ref.read(appStateControllerProvider.notifier);
    controller.setPlayButtonState(PlayButtonState.loading);
    controller.setStatus(AppStrings.statusLaunching);

    ProcessService.startNierAutomata(
      installDirectory: widget.selectedPath!,
      l10n: AppLocalizations.of(context)!,
      onProcessStopped: () async {
        controller.setPlayButtonState(PlayButtonState.idle);
        controller.setStatus(AppStrings.statusStopped);
        if (Platform.isWindows) {
          try {
            if (await windowManager.isMinimized()) {
              await windowManager.restore();
            }
            await windowManager.focus();
          } catch (_) {}
        }
      },
    ).then((outcome) async {
      if (outcome.started) {
        controller.setPlayButtonState(PlayButtonState.running);
        controller.setStatus(AppStrings.statusRunning);
        if (Platform.isWindows) {
          try {
            final prefs = await SharedPreferences.getInstance();
            final shouldMinimize =
                prefs.getBool(AppStrings.prefKeyMinimizeOnLaunch) ?? true;
            if (shouldMinimize) {
              await Future.delayed(const Duration(seconds: 3));
              if (ref.read(appStateControllerProvider).playButtonState ==
                  PlayButtonState.running) {
                await windowManager.minimize();
              }
            }
          } catch (_) {}
        }
      } else {
        controller.setPlayButtonState(PlayButtonState.idle);
        controller.setStatus(null);
        final failure = outcome.failure;
        if (failure != null && context.mounted) {
          showLaunchFailureDialog(
            context,
            failure,
            installDirectory: widget.selectedPath,
          );
        }
      }
    });
  }
}
