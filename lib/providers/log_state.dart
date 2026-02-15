import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/log_service.dart';

part 'log_state.g.dart';

final logPanelOpenProvider = StateProvider<bool>((ref) => false);

class LogData {
  final List<LogEntry> modloaderEntries;
  final List<LogEntry> yorhaEntries;
  final bool isLoading;
  final String activeTab;

  const LogData({
    this.modloaderEntries = const [],
    this.yorhaEntries = const [],
    this.isLoading = false,
    this.activeTab = 'modloader',
  });

  LogData copyWith({
    List<LogEntry>? modloaderEntries,
    List<LogEntry>? yorhaEntries,
    bool? isLoading,
    String? activeTab,
  }) {
    return LogData(
      modloaderEntries: modloaderEntries ?? this.modloaderEntries,
      yorhaEntries: yorhaEntries ?? this.yorhaEntries,
      isLoading: isLoading ?? this.isLoading,
      activeTab: activeTab ?? this.activeTab,
    );
  }

  List<LogEntry> get activeEntries =>
      activeTab == 'modloader' ? modloaderEntries : yorhaEntries;
}

@Riverpod(keepAlive: true)
class LogStateController extends _$LogStateController {
  @override
  LogData build() => const LogData();

  Future<void> loadLogs() async {
    state = state.copyWith(isLoading: true);

    final modloader = await LogService.readLog('modloader.log');
    final yorha = await LogService.readLog('yorha_protocol.log');

    state = LogData(
      modloaderEntries: modloader,
      yorhaEntries: yorha,
    );
  }

  void setActiveTab(String tab) {
    state = state.copyWith(activeTab: tab);
  }

  Future<void> refresh() async {
    await loadLogs();
  }
}
