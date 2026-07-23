import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/default_mods_service.dart';

part 'default_mods_state.g.dart';

class DefaultModsData {
  final List<DefaultEntry> entries;
  final bool isLoading;

  const DefaultModsData({this.entries = const [], this.isLoading = false});

  bool isDefault(String relPath) =>
      DefaultModsService.matches(entries, relPath);

  /// How `relPath` is configured, or null when it is not a default.
  DefaultKind? kindOf(String relPath) =>
      DefaultModsService.entryFor(entries, relPath)?.kind;

  /// Which outfit `relPath` is set to wear, if it names one.
  int? outfitIdOf(String relPath) =>
      DefaultModsService.entryFor(entries, relPath)?.outfitId;

  /// The entry `relPath` would take the slot from, if any.
  DefaultEntry? wouldReplace(String relPath, DefaultKind kind) {
    final rel = DefaultModsService.normalize(relPath);
    final stem = DefaultModsService.stemOf(rel);
    if (stem == null) return null;
    for (final e in entries) {
      if (e.path == rel) continue;
      if (!kind.conflictsWith(e.kind)) continue;
      if (DefaultModsService.stemOf(e.path) == stem) return e;
    }
    return null;
  }

  /// Starred entries belonging to this mod, e.g. for `mods/foo` returns
  /// `mods/foo` itself and any `mods/foo/pl/<stem>` entries.
  List<DefaultEntry> entriesUnder(String relPath) {
    final rel = DefaultModsService.normalize(relPath);
    return entries
        .where((e) => e.path == rel || e.path.startsWith('$rel/'))
        .toList();
  }
}

@Riverpod(keepAlive: true)
class DefaultModsStateController extends _$DefaultModsStateController {
  @override
  DefaultModsData build() => const DefaultModsData();

  Future<void> load(String gameDir) async {
    if (gameDir.isEmpty) return;
    state = DefaultModsData(entries: state.entries, isLoading: true);
    final entries = await DefaultModsService.list(gameDir);
    state = DefaultModsData(entries: entries);
  }

  /// Sets `relPath` to `kind`, or clears it when `kind` is null.
  Future<void> setDefault(
    String gameDir,
    String relPath,
    DefaultKind? kind, {
    int? outfitId,
  }) async {
    await DefaultModsService.setDefault(
      gameDir,
      relPath,
      kind,
      outfitId: outfitId,
    );
    await load(gameDir);
  }
}
