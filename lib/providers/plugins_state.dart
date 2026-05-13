import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/plugins_service.dart';

part 'plugins_state.g.dart';

class PluginsData {
  final List<LauncherPlugin> plugins;
  final bool isLoading;
  final String? error;

  const PluginsData({
    this.plugins = const [],
    this.isLoading = false,
    this.error,
  });

  PluginsData copyWith({
    List<LauncherPlugin>? plugins,
    bool? isLoading,
    String? error,
    bool clearError = false,
  }) {
    return PluginsData(
      plugins: plugins ?? this.plugins,
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
    );
  }
}

@Riverpod(keepAlive: true)
class PluginsStateController extends _$PluginsStateController {
  @override
  PluginsData build() => const PluginsData();

  Future<void> load() async {
    state = state.copyWith(isLoading: true, clearError: true);
    try {
      final list = await PluginsService.list();
      state = PluginsData(plugins: list);
    } catch (e) {
      state = PluginsData(error: e.toString());
    }
  }

  Future<PluginInstallResult> install(String sourceDllPath) async {
    final result = await PluginsService.install(sourceDllPath);
    await load();
    return result;
  }

  Future<void> setEnabled(String name, bool enabled) async {
    await PluginsService.setEnabled(name, enabled);
    await load();
  }

  Future<void> delete(String name) async {
    await PluginsService.delete(name);
    await load();
  }
}
