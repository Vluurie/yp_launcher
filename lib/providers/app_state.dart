import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

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
    try {
      final prefs = await SharedPreferences.getInstance();
      final savedDir = prefs.getString('nier_directory');
      if (savedDir != null && savedDir.isNotEmpty) {
        state = state.copyWith(selectedDirectory: savedDir);
        debugPrint('Loaded saved directory: $savedDir');
      }
    } catch (e) {
      debugPrint('Failed to load saved directory: $e');
    }
  }

  Future<void> setDirectory(String path) async {
    state = state.copyWith(selectedDirectory: path, errorMessage: null);

    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('nier_directory', path);
      debugPrint('Directory persisted: $path');
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
