import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:automato_theme/automato_theme.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/settings_provider.dart';
import 'package:yp_launcher/services/process_service.dart';

class PlayButton extends ConsumerWidget {
  const PlayButton({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appState = ref.watch(appStateControllerProvider);
    final controller = ref.read(appStateControllerProvider.notifier);

    return SizedBox(
      width: 200,
      height: 70,
      child: TextButton(
        onPressed: appState.canPlay
            ? () => _handlePlayButton(ref, controller, appState)
            : null,
        style: TextButton.styleFrom(
          foregroundColor: _getButtonColor(ref, appState),
          backgroundColor: AutomatoThemeColors.darkBrown(ref),
          disabledBackgroundColor:
              AutomatoThemeColors.darkBrown(ref).withOpacity(0.5),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8.0),
            side: BorderSide(
              color: _getButtonColor(ref, appState),
              width: 2,
            ),
          ),
          padding: const EdgeInsets.symmetric(vertical: 12.0, horizontal: 24.0),
        ),
        child: _buildButtonContent(ref, appState),
      ),
    );
  }

  Widget _buildButtonContent(WidgetRef ref, AppState appState) {
    if (appState.playButtonState == PlayButtonState.loading) {
      return SizedBox(
        width: 40,
        height: 40,
        child: AutomatoLoading(
          color: AutomatoThemeColors.bright(ref),
          translateX: 0,
          svgString: AutomatoSvgStrings.automatoSvgStrHead,
        ),
      );
    }

    return Text(
      appState.playButtonState == PlayButtonState.running ? 'STOP' : 'PLAY',
      style: TextStyle(
        fontSize: 32.0,
        fontWeight: FontWeight.bold,
        color: _getButtonColor(ref, appState),
      ),
    );
  }

  Color _getButtonColor(WidgetRef ref, AppState appState) {
    if (appState.playButtonState == PlayButtonState.running) {
      return AutomatoThemeColors.dangerZone(ref);
    } else if (!appState.canPlay) {
      return AutomatoThemeColors.primaryColor(ref).withOpacity(0.3);
    } else {
      return AutomatoThemeColors.primaryColor(ref);
    }
  }

  Future<void> _handlePlayButton(
    WidgetRef ref,
    AppStateController controller,
    AppState appState,
  ) async {
    if (appState.playButtonState == PlayButtonState.idle) {
      controller.setPlayButtonState(PlayButtonState.loading);

      try {
        // Get settings service
        final settingsAsync = ref.read(settingsServiceProvider);
        final settings = settingsAsync.maybeWhen(
          data: (value) => value,
          orElse: () => null,
        );

        final started = await ProcessService.startNierAutomata(
          installDirectory: appState.selectedDirectory,
          onProcessStopped: () {
            controller.setPlayButtonState(PlayButtonState.idle);
          },
          settings: settings,
        );

        if (started) {
          controller.setPlayButtonState(PlayButtonState.running);
        } else {
          controller.setError('Failed to start NieR:Automata');
          controller.setPlayButtonState(PlayButtonState.idle);
        }
      } catch (e) {
        controller.setError(e.toString());
        controller.setPlayButtonState(PlayButtonState.idle);
      }
    } else if (appState.playButtonState == PlayButtonState.running) {
      controller.setPlayButtonState(PlayButtonState.loading);

      final success = ProcessService.terminateNierAutomata();
      if (success) {
        controller.setPlayButtonState(PlayButtonState.idle);
      } else {
        controller.setError('Failed to stop NieR:Automata');
        controller.setPlayButtonState(PlayButtonState.running);
      }
    }
  }
}
