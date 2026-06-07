import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/log_service.dart';

part 'log_state.g.dart';

bool _isModloaderModule(String module) {
  final m = module.toLowerCase();
  return m.startsWith('modloader') ||
      m.startsWith('nams') ||
      m.startsWith('nier') ||
      m.startsWith('wgpu') ||
      m.startsWith('naga');
}

final logPanelOpenProvider = StateProvider<bool>((ref) => false);

class LogData {
  final List<LogEntry> modloaderEntries;
  final List<LogEntry> yorhaEntries;
  final bool isLoading;
  final String activeTab;
  final String searchQuery;

  const LogData({
    this.modloaderEntries = const [],
    this.yorhaEntries = const [],
    this.isLoading = false,
    this.activeTab = 'modloader',
    this.searchQuery = '',
  });

  LogData copyWith({
    List<LogEntry>? modloaderEntries,
    List<LogEntry>? yorhaEntries,
    bool? isLoading,
    String? activeTab,
    String? searchQuery,
  }) {
    return LogData(
      modloaderEntries: modloaderEntries ?? this.modloaderEntries,
      yorhaEntries: yorhaEntries ?? this.yorhaEntries,
      isLoading: isLoading ?? this.isLoading,
      activeTab: activeTab ?? this.activeTab,
      searchQuery: searchQuery ?? this.searchQuery,
    );
  }

  List<LogEntry> get activeEntries =>
      activeTab == 'modloader' ? modloaderEntries : yorhaEntries;

  /// Active entries with the search query applied (case-insensitive,
  /// matches level / module / message).
  List<LogEntry> get filteredEntries {
    final q = searchQuery.trim().toLowerCase();
    if (q.isEmpty) return activeEntries;
    return activeEntries.where((e) {
      return e.level.toLowerCase().contains(q) ||
          e.module.toLowerCase().contains(q) ||
          e.message.toLowerCase().contains(q);
    }).toList();
  }
}

@Riverpod(keepAlive: true)
class LogStateController extends _$LogStateController {
  StreamSubscription<List<LogEntry>>? _modloaderSub;

  @override
  LogData build() {
    ref.onDispose(() {
      _modloaderSub?.cancel();
    });
    return const LogData();
  }

  Future<void> loadLogs() async {
    // Watcher's first emission contains the full current file. Reset state
    // and let the stream populate everything in order.
    state = state.copyWith(
      modloaderEntries: const [],
      yorhaEntries: const [],
      isLoading: true,
    );
    startStreaming();
    state = state.copyWith(isLoading: false);
  }

  /// Begin tailing both log files. The watcher's first emission contains
  /// the entire existing file content; subsequent emissions are appended.
  void startStreaming() {
    _modloaderSub?.cancel();
    _modloaderSub = LogService.watchLog(AppStrings.namsLogName).listen((
      newEntries,
    ) {
      if (newEntries.isEmpty) {
        state = state.copyWith(
          modloaderEntries: const [],
          yorhaEntries: const [],
        );
        return;
      }
      final modloader = <LogEntry>[];
      final yorha = <LogEntry>[];
      for (final e in newEntries) {
        (_isModloaderModule(e.module) ? modloader : yorha).add(e);
      }
      state = state.copyWith(
        modloaderEntries: [...state.modloaderEntries, ...modloader],
        yorhaEntries: [...state.yorhaEntries, ...yorha],
      );
    });
  }

  void stopStreaming() {
    _modloaderSub?.cancel();
    _modloaderSub = null;
  }

  void setActiveTab(String tab) {
    state = state.copyWith(activeTab: tab);
  }

  void setSearchQuery(String query) {
    state = state.copyWith(searchQuery: query);
  }

  Future<void> refresh() async {
    stopStreaming();
    await loadLogs();
  }
}
