import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/providers/notification_state.dart';
import 'package:yp_launcher/services/nams_config_service.dart';

part 'app_state.g.dart';

final autoSearchingProvider = StateProvider<bool>((ref) => false);
final activeTabProvider = StateProvider<int>((ref) => 0);
final detectionRefreshProvider = StateProvider<int>((ref) => 0);
final texturesBusyProvider = StateProvider<bool>((ref) => false);

/// One-shot cross-tab selection request. When a widget on tab A wants to
/// open tab B and pre-select an item there, it sets this value and switches
/// the active tab. Tab B reads it once on build, applies the selection, and
/// clears it (returns null).
final pendingTabSelectionProvider =
    StateProvider<TabSelectionRequest?>((ref) => null);

class TabSelectionRequest {
  final int tabIndex;
  final String key;
  const TabSelectionRequest({required this.tabIndex, required this.key});
}

/// Global refcount of in-progress destructive/long-running operations
/// (installs, deletes, extractions, etc). The window close handler checks
/// this and warns the user before exiting if non-zero.
final busyOperationsProvider = StateProvider<int>((ref) => 0);

enum PlayButtonState { idle, loading, running }

class AppState {
  final String selectedDirectory;
  final PlayButtonState playButtonState;
  final String? errorMessage;
  final String? statusMessage;

  const AppState({
    this.selectedDirectory = '',
    this.playButtonState = PlayButtonState.idle,
    this.errorMessage,
    this.statusMessage,
  });

  AppState copyWith({
    String? selectedDirectory,
    PlayButtonState? playButtonState,
    String? errorMessage,
    String? statusMessage,
  }) {
    return AppState(
      selectedDirectory: selectedDirectory ?? this.selectedDirectory,
      playButtonState: playButtonState ?? this.playButtonState,
      errorMessage: errorMessage,
      statusMessage: statusMessage,
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
    _loadSavedDirectory();
    return const AppState();
  }

  Future<void> _loadSavedDirectory() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final savedDir = prefs.getString(AppStrings.prefKeyDirectory);
      if (savedDir != null && savedDir.isNotEmpty) {
        state = state.copyWith(selectedDirectory: savedDir);
        await NamsConfigService.ensureConfigs(savedDir);
        ref
            .read(notificationStateControllerProvider.notifier)
            .runDetections(savedDir);
      }
    } catch (_) {}
  }

  Future<void> setDirectory(String path) async {
    state = state.copyWith(selectedDirectory: path, errorMessage: null);

    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(AppStrings.prefKeyDirectory, path);
    } catch (_) {}
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

  void setStatus(String? status) {
    state = state.copyWith(statusMessage: status);
  }
}
