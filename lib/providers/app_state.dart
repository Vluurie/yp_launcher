import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/providers/settings_provider.dart';

part 'app_state.g.dart';

enum PlayButtonState {
  idle,
  loading,
  running,
}

class AppState {
  final String selectedDirectory;
  final PlayButtonState playButtonState;
  final String? errorMessage;

  const AppState({
    this.selectedDirectory = '',
    this.playButtonState = PlayButtonState.idle,
    this.errorMessage,
  });

  AppState copyWith({
    String? selectedDirectory,
    PlayButtonState? playButtonState,
    String? errorMessage,
  }) {
    return AppState(
      selectedDirectory: selectedDirectory ?? this.selectedDirectory,
      playButtonState: playButtonState ?? this.playButtonState,
      errorMessage: errorMessage,
    );
  }

  bool get isDirectorySelected => selectedDirectory.isNotEmpty;
  bool get canPlay =>
      isDirectorySelected && playButtonState != PlayButtonState.loading;
}

@riverpod
class AppStateController extends _$AppStateController {
  @override
  AppState build() {
    // Load saved directory on initialization
    _loadSavedDirectory();
    return const AppState();
  }

  Future<void> _loadSavedDirectory() async {
    final settingsAsync = ref.read(settingsServiceProvider);
    settingsAsync.whenData((settings) {
      final savedDir = settings.nierDirectory;
      if (savedDir != null && savedDir.isNotEmpty) {
        state = state.copyWith(selectedDirectory: savedDir);
      }
    });
  }

  Future<void> setDirectory(String path) async {
    state = state.copyWith(selectedDirectory: path, errorMessage: null);

    // Save to settings
    try {
      final settingsAsync = ref.read(settingsServiceProvider);
      await settingsAsync.when(
        data: (settings) async {
          await settings.setNierDirectory(path);
          debugPrint('Directory persisted: $path');
        },
        loading: () async {},
        error: (e, stack) async {
          debugPrint('Failed to save directory: $e');
        },
      );
    } catch (e) {
      debugPrint('Failed to save directory: $e');
    }
  }

  void setPlayButtonState(PlayButtonState buttonState) {
    state = state.copyWith(playButtonState: buttonState);
  }

  void setError(String error) {
    state = state.copyWith(errorMessage: error);
  }

  void clearError() {
    state = state.copyWith(errorMessage: null);
  }
}
