import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/disabled_mods_service.dart';

part 'disabled_mods_state.g.dart';

class DisabledModsData {
  final List<String> entries;
  final bool isLoading;

  const DisabledModsData({this.entries = const [], this.isLoading = false});

  bool isDisabled(String relPath) =>
      DisabledModsService.matches(entries, relPath);
}

@Riverpod(keepAlive: true)
class DisabledModsStateController extends _$DisabledModsStateController {
  @override
  DisabledModsData build() => const DisabledModsData();

  Future<void> load(String gameDir) async {
    if (gameDir.isEmpty) return;
    state = DisabledModsData(entries: state.entries, isLoading: true);
    final entries = await DisabledModsService.list(gameDir);
    state = DisabledModsData(entries: entries);
  }

  Future<void> setDisabled(
    String gameDir,
    String relPath,
    bool disabled,
  ) async {
    await DisabledModsService.setDisabled(gameDir, relPath, disabled);
    await load(gameDir);
  }
}