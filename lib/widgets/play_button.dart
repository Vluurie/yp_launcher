import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/process_service.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_sizes.dart';

class PlayButton extends ConsumerWidget {
  const PlayButton({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appState = ref.watch(appStateControllerProvider);
    final controller = ref.read(appStateControllerProvider.notifier);

    final buttonColor = _getButtonColor(appState);
    final bgColor = _getBackgroundColor(appState);

    return SizedBox(
      width: AppSizes.playButtonWidth,
      height: AppSizes.playButtonHeight,
      child: TextButton(
        onPressed: appState.canPlay
            ? () => _handlePlayButton(ref, controller, appState)
            : null,
        style: TextButton.styleFrom(
          foregroundColor: buttonColor,
          backgroundColor: bgColor,
          disabledBackgroundColor: AppColors.surfaceLight.withValues(alpha: 0.5),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(6.0),
            side: BorderSide(
              color: buttonColor,
              width: 1.5,
            ),
          ),
          padding: const EdgeInsets.symmetric(vertical: 8.0, horizontal: 20.0),
        ),
        child: _buildButtonContent(ref, appState),
      ),
    );
  }

  Widget _buildButtonContent(WidgetRef ref, AppState appState) {
    if (appState.playButtonState == PlayButtonState.loading) {
      return SizedBox(
        width: AppSizes.playButtonLoadingSize,
        height: AppSizes.playButtonLoadingSize,
        child: AutomatoLoading(
          color: AppColors.accentPrimary,
          translateX: 0,
          svgString: AutomatoSvgStrings.automatoSvgStrHead,
        ),
      );
    }

    final buttonColor = _getButtonColor(appState);
    return Text(
      appState.playButtonState == PlayButtonState.running
          ? AppStrings.stopButton
          : AppStrings.playButton,
      style: TextStyle(
        fontSize: AppSizes.fontXXL,
        fontWeight: FontWeight.bold,
        color: buttonColor,
      ),
    );
  }

  Color _getButtonColor(AppState appState) {
    if (appState.playButtonState == PlayButtonState.running) {
      return AppColors.error;
    } else if (!appState.canPlay) {
      return AppColors.textMuted;
    } else {
      return AppColors.success;
    }
  }

  Color _getBackgroundColor(AppState appState) {
    if (appState.playButtonState == PlayButtonState.running) {
      return AppColors.surfaceLight;
    } else {
      return AppColors.surfaceLight;
    }
  }

  Future<void> _handlePlayButton(
    WidgetRef ref,
    AppStateController controller,
    AppState appState,
  ) async {
    if (appState.playButtonState == PlayButtonState.idle) {
      controller.setPlayButtonState(PlayButtonState.loading);
      controller.setStatus(AppStrings.statusPreparing);

      try {
        controller.setStatus(AppStrings.statusLaunching);

        final started = await ProcessService.startNierAutomata(
          installDirectory: appState.selectedDirectory,
          onProcessStopped: () {
            controller.setPlayButtonState(PlayButtonState.idle);
            controller.setStatus(AppStrings.statusStopped);
          },
        );

        if (started) {
          controller.setPlayButtonState(PlayButtonState.running);
          controller.setStatus(AppStrings.statusRunning);
        } else {
          controller.setError(AppStrings.errorStartFailed);
          controller.setPlayButtonState(PlayButtonState.idle);
          controller.setStatus(null);
        }
      } catch (e) {
        controller.setError(e.toString());
        controller.setPlayButtonState(PlayButtonState.idle);
        controller.setStatus(null);
      }
    } else if (appState.playButtonState == PlayButtonState.running) {
      controller.setPlayButtonState(PlayButtonState.loading);
      controller.setStatus(AppStrings.statusStopping);

      final success = ProcessService.terminateNierAutomata();
      if (success) {
        controller.setPlayButtonState(PlayButtonState.idle);
        controller.setStatus(AppStrings.statusStopped);
      } else {
        controller.setError(AppStrings.errorStopFailed);
        controller.setPlayButtonState(PlayButtonState.running);
        controller.setStatus(null);
      }
    }
  }
}
