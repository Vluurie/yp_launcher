import 'dart:async';

import 'package:flutter_riverpod/legacy.dart';
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

/// Newest entries kept per tab. Older ones fall off, so a long session
/// cannot grow the list until scrolling stalls.
const maxEntriesPerTab = 5000;

List<LogEntry> appendCapped(List<LogEntry> current, List<LogEntry> added) {
  if (added.isEmpty) return current;

  final total = current.length + added.length;
  if (total <= maxEntriesPerTab) {
    return [...current, ...added];
  }

  final keptFromCurrent = maxEntriesPerTab - added.length;
  if (keptFromCurrent <= 0) {
    return added.sublist(added.length - maxEntriesPerTab);
  }
  return [
    ...current.sublist(current.length - keptFromCurrent),
    ...added,
  ];
}

/// Levels the panel can filter by, in the order they are shown.
const logLevels = ['ERROR', 'WARN', 'INFO', 'DEBUG'];

/// Hidden until asked for: these two carry the routine chatter, so showing
/// them by default buries the errors people open the panel for.
const defaultHiddenLevels = {'INFO', 'DEBUG'};

class LogData {
  final List<LogEntry> modloaderEntries;
  final List<LogEntry> yorhaEntries;
  final bool isLoading;
  final String activeTab;
  final String searchQuery;
  final Set<String> hiddenLevels;

  const LogData({
    this.modloaderEntries = const [],
    this.yorhaEntries = const [],
    this.isLoading = false,
    this.activeTab = 'modloader',
    this.searchQuery = '',
    this.hiddenLevels = defaultHiddenLevels,
  });

  LogData copyWith({
    List<LogEntry>? modloaderEntries,
    List<LogEntry>? yorhaEntries,
    bool? isLoading,
    String? activeTab,
    String? searchQuery,
    Set<String>? hiddenLevels,
  }) {
    return LogData(
      modloaderEntries: modloaderEntries ?? this.modloaderEntries,
      yorhaEntries: yorhaEntries ?? this.yorhaEntries,
      isLoading: isLoading ?? this.isLoading,
      activeTab: activeTab ?? this.activeTab,
      searchQuery: searchQuery ?? this.searchQuery,
      hiddenLevels: hiddenLevels ?? this.hiddenLevels,
    );
  }

  List<LogEntry> get activeEntries =>
      activeTab == 'modloader' ? modloaderEntries : yorhaEntries;

  bool isLevelShown(String level) => !hiddenLevels.contains(level);

  /// Entry count per level for the active tab, counted in one pass so the
  /// panel does not walk the whole list once per chip on every build.
  Map<String, int> get levelCounts {
    final counts = <String, int>{};
    for (final entry in activeEntries) {
      counts[entry.level] = (counts[entry.level] ?? 0) + 1;
    }
    return counts;
  }

  /// Active entries with the level filter and search query applied
  /// (case-insensitive, matches level / module / message).
  List<LogEntry> get filteredEntries {
    final q = searchQuery.trim().toLowerCase();
    if (q.isEmpty && hiddenLevels.isEmpty) return activeEntries;

    return activeEntries.where((e) {
      if (hiddenLevels.contains(e.level)) return false;
      if (q.isEmpty) return true;
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
        modloaderEntries: appendCapped(state.modloaderEntries, modloader),
        yorhaEntries: appendCapped(state.yorhaEntries, yorha),
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

  void toggleLevel(String level) {
    final hidden = {...state.hiddenLevels};
    if (!hidden.remove(level)) hidden.add(level);
    state = state.copyWith(hiddenLevels: hidden);
  }

  Future<void> refresh() async {
    stopStreaming();
    await loadLogs();
  }
}
