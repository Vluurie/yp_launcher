import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/models/mod_profile.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/default_mods_state.dart';
import 'package:yp_launcher/providers/disabled_mods_state.dart';
import 'package:yp_launcher/providers/mod_load_order_state.dart';
import 'package:yp_launcher/providers/mods_state.dart';
import 'package:yp_launcher/services/mod_profiles_service.dart';

part 'mod_profiles_state.g.dart';

@Riverpod(keepAlive: true)
class ModProfilesStateController extends _$ModProfilesStateController {
  @override
  ModProfileState build() => const ModProfileState();

  Future<void> load(String gameDir) async {
    if (gameDir.isEmpty) return;
    state = state.copyWith(isLoading: true, error: null);
    try {
      final next = await ModProfilesService.loadProfiles(gameDir);
      state = next;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  Future<bool> switchTo(String gameDir, String name) async {
    if (state.activeName == name) return true;
    state = state.copyWith(isLoading: true, error: null);
    try {
      final next = await ModProfilesService.switchProfile(gameDir, name);
      state = next;
      await _refreshDependents(gameDir);
      return true;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      return false;
    }
  }

  Future<bool> create(String gameDir, String name) async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final next = await ModProfilesService.createProfile(gameDir, name);
      state = next;
      await _refreshDependents(gameDir);
      return true;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      return false;
    }
  }

  Future<bool> delete(String gameDir, String name) async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final next = await ModProfilesService.deleteProfile(gameDir, name);
      state = next;
      ref.read(detectionRefreshProvider.notifier).state++;
      return true;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      return false;
    }
  }

  Future<bool> rename(
    String gameDir,
    String oldName,
    String newName,
  ) async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final next =
          await ModProfilesService.renameProfile(gameDir, oldName, newName);
      state = next;
      ref.read(detectionRefreshProvider.notifier).state++;
      return true;
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
      return false;
    }
  }

  Future<void> _refreshDependents(String gameDir) async {
    await ref.read(modsStateControllerProvider.notifier).loadMods(gameDir);
    await ref.read(disabledModsStateControllerProvider.notifier).load(gameDir);
    await ref.read(defaultModsStateControllerProvider.notifier).load(gameDir);
    await ref.read(modLoadOrderStateControllerProvider.notifier).load(gameDir);
    ref.read(detectionRefreshProvider.notifier).state++;
  }
}
