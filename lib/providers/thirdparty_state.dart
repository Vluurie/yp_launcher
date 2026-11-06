import 'dart:async';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/game_mods_runtime.dart';
import 'package:yp_launcher/services/thirdparty/graphic_mods_runtime.dart';
import 'package:yp_launcher/services/thirdparty/graphics_runtime.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_classifier.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';

part 'thirdparty_state.g.dart';

class ThirdPartyData {
  final ThirdPartyRuntimeStatus reshade;
  final ThirdPartyRuntimeStatus migoto;
  final ThirdPartyRuntimeStatus gameMods;
  final ThirdPartyRuntimeStatus graphicMods;
  final bool isLoading;
  final bool busy;

  const ThirdPartyData({
    this.reshade = const ThirdPartyRuntimeStatus(),
    this.migoto = const ThirdPartyRuntimeStatus(),
    this.gameMods = const ThirdPartyRuntimeStatus(),
    this.graphicMods = const ThirdPartyRuntimeStatus(),
    this.isLoading = false,
    this.busy = false,
  });

  ThirdPartyData copyWith({
    ThirdPartyRuntimeStatus? reshade,
    ThirdPartyRuntimeStatus? migoto,
    ThirdPartyRuntimeStatus? gameMods,
    ThirdPartyRuntimeStatus? graphicMods,
    bool? isLoading,
    bool? busy,
  }) {
    return ThirdPartyData(
      reshade: reshade ?? this.reshade,
      migoto: migoto ?? this.migoto,
      gameMods: gameMods ?? this.gameMods,
      graphicMods: graphicMods ?? this.graphicMods,
      isLoading: isLoading ?? this.isLoading,
      busy: busy ?? this.busy,
    );
  }
}

@Riverpod(keepAlive: true)
class ThirdPartyStateController extends _$ThirdPartyStateController {
  static const reshade = ReShadeRuntime();
  static const migoto = MigotoRuntime();
  static const gameMods = GameModsRuntime();
  static const graphicMods = GraphicModsRuntime();

  StreamSubscription<FileSystemEvent>? _watchSub;
  String? _watchedDir;
  Timer? _debounce;

  @override
  ThirdPartyData build() {
    ref.onDispose(() {
      _debounce?.cancel();
      _watchSub?.cancel();
    });
    return const ThirdPartyData();
  }

  String? get _gameDir {
    final dir = ref.read(appStateControllerProvider).selectedDirectory;
    return dir.isEmpty ? null : dir;
  }

  GraphicsRuntime _runtime(ThirdPartyRuntime which) {
    switch (which) {
      case ThirdPartyRuntime.reshade:
        return reshade;
      case ThirdPartyRuntime.migoto:
        return migoto;
      case ThirdPartyRuntime.gameMods:
        return gameMods;
      case ThirdPartyRuntime.graphicMods:
        return graphicMods;
    }
  }

  Future<void> refresh() async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    state = state.copyWith(isLoading: true);
    await _autoImportFromGameRoot(gameDir);
    await _refreshInternal(gameDir);
    state = state.copyWith(isLoading: false);
    _startWatching(gameDir);
  }

  void _startWatching(String gameDir) {
    if (_watchedDir == gameDir && _watchSub != null) return;
    _watchSub?.cancel();
    _watchSub = null;
    _watchedDir = null;

    final dir = Directory(gameDir);
    if (!dir.existsSync()) return;
    try {
      _watchSub = dir
          .watch(recursive: false)
          .listen(
            (event) => _onGameRootChanged(gameDir, event),
            onError: (_) {},
          );
      _watchedDir = gameDir;
    } catch (_) {}
  }

  void _onGameRootChanged(String gameDir, FileSystemEvent event) {
    final name = p.basename(event.path).toLowerCase();
    if (!name.endsWith('.dll')) return;
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 600), () async {
      if (state.busy) return;
      final changed = await _autoImportFromGameRoot(gameDir);
      if (changed) await _refreshInternal(gameDir);
    });
  }

  Future<bool> _autoImportFromGameRoot(String gameDir) async {
    var changed = false;
    if (!(await reshade.status(gameDir)).installed &&
        reshade.findGameRootDll(gameDir) != null) {
      changed = await reshade.importFromGameRoot(gameDir) || changed;
    }
    if (!(await migoto.status(gameDir)).installed &&
        migoto.findGameRootDll(gameDir) != null) {
      changed = await migoto.importFromGameRoot(gameDir) || changed;
    }
    return changed;
  }

  Future<ThirdPartyClassification?> classify(String extractedRoot) async {
    return ThirdPartyClassifier.classify(extractedRoot);
  }

  static const _installers = <GraphicsRuntime>[
    reshade,
    migoto,
    gameMods,
    graphicMods,
  ];

  GraphicsRuntime? _installerFor(ThirdPartyClassification c) {
    for (final r in _installers) {
      if (r.canInstall(c)) return r;
    }
    return null;
  }

  Future<ThirdPartyUpdateInfo?> wouldUpdate(ThirdPartyClassification c) async {
    final gameDir = _gameDir;
    if (gameDir == null) return null;
    return _installerFor(c)?.wouldUpdate(gameDir, c);
  }

  Future<ThirdPartyInstallResult?> install(ThirdPartyClassification c) async {
    final gameDir = _gameDir;
    if (gameDir == null) return null;
    return _withBusy(() async {
      final runtime = _installerFor(c);
      if (runtime == null) return null;
      final result = await runtime.install(gameDir, c);
      await _refreshInternal(gameDir);
      return result;
    });
  }

  Future<bool> importFromGameRoot(ThirdPartyRuntime which) async {
    final gameDir = _gameDir;
    if (gameDir == null) return false;
    return _withBusy(() async {
      final ok = await _runtime(which).importFromGameRoot(gameDir);
      await _refreshInternal(gameDir);
      return ok;
    });
  }

  Future<void> setEnabled(ThirdPartyRuntime which, bool enabled) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await _runtime(which).setEnabled(gameDir, enabled);
    await _refreshInternal(gameDir);
  }

  Future<void> applyReShadeConfig(ReShadeConfig config) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await reshade.applyConfig(gameDir, config);
    await _refreshInternal(gameDir);
  }

  Future<void> applyMigotoConfig(MigotoConfig config) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await migoto.applyConfig(gameDir, config);
    await _refreshInternal(gameDir);
  }

  Future<void> setGameModDisabled(String fileName, bool disabled) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await gameMods.setModDisabled(gameDir, fileName, disabled);
    await _refreshInternal(gameDir);
  }

  Future<void> setGameModSettings(
    String fileName,
    GameModSettings settings,
  ) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await gameMods.setModSettings(gameDir, fileName, settings);
    await _refreshInternal(gameDir);
  }

  Future<void> setGraphicModDisabled(String fileName, bool disabled) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await graphicMods.setModDisabled(gameDir, fileName, disabled);
    await _refreshInternal(gameDir);
  }

  Future<void> removeGraphicMod(String fileName) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await _withBusy(() async {
      await graphicMods.removeMod(gameDir, fileName);
      await _refreshInternal(gameDir);
    });
  }

  Future<void> moveDll(String fileName, ThirdPartyRuntime from) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await _withBusy(() async {
      final toGraphic = from == ThirdPartyRuntime.gameMods;
      final source = toGraphic
          ? p.join(gameMods.installDir(gameDir), fileName)
          : graphicMods.pathOf(gameDir, fileName);
      if (source == null || !File(source).existsSync()) return;
      final target = toGraphic
          ? graphicMods.installDir(gameDir)
          : gameMods.installDir(gameDir);
      FileOps.copyFileInto(source, target);
      if (!File(p.join(target, fileName)).existsSync()) return;
      FileOps.deleteFileQuiet(source);
      final runtime = toGraphic ? graphicMods : gameMods;
      await runtime.setEnabled(gameDir, true);
      await _refreshInternal(gameDir);
    });
  }

  Future<void> removeGameMod(String fileName) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await _withBusy(() async {
      await gameMods.removeMod(gameDir, fileName);
      await _refreshInternal(gameDir);
    });
  }

  Future<void> remove(ThirdPartyRuntime which) async {
    final gameDir = _gameDir;
    if (gameDir == null) return;
    await _withBusy(() async {
      await _runtime(which).remove(gameDir);
      await _refreshInternal(gameDir);
    });
  }

  Future<T> _withBusy<T>(Future<T> Function() action) async {
    state = state.copyWith(busy: true);
    try {
      return await action();
    } finally {
      state = state.copyWith(busy: false);
    }
  }

  Future<void> _refreshInternal(String gameDir) async {
    final r = await reshade.status(gameDir);
    final m = await migoto.status(gameDir);
    final g = await gameMods.status(gameDir);
    final x = await graphicMods.status(gameDir);
    state = state.copyWith(reshade: r, migoto: m, gameMods: g, graphicMods: x);
  }
}
