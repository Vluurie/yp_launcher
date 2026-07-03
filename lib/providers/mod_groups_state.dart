import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/models/mod_group.dart';
import 'package:yp_launcher/services/mod_groups_service.dart';

part 'mod_groups_state.g.dart';

@Riverpod(keepAlive: true)
class ModGroupsStateController extends _$ModGroupsStateController {
  @override
  ModGroupsData build() => const ModGroupsData();

  Future<void> load(String gameDir) async {
    if (gameDir.isEmpty) return;
    state = await ModGroupsService.load(gameDir);
  }

  Future<void> _persist(String gameDir, ModGroupsData next) async {
    state = next;
    await ModGroupsService.save(gameDir, next);
  }

  Future<String> addGroup(String gameDir, String name) async {
    final id = 'g_${DateTime.now().microsecondsSinceEpoch}';
    final maxOrder = state.groups.fold<int>(
      -1,
      (acc, g) => g.order > acc ? g.order : acc,
    );
    final next = state.copyWith(
      groups: [
        ...state.groups,
        ModGroup(id: id, name: name.trim(), order: maxOrder + 1),
      ],
    );
    await _persist(gameDir, next);
    return id;
  }

  Future<void> renameGroup(String gameDir, String id, String name) async {
    final next = state.copyWith(
      groups: state.groups
          .map((g) => g.id == id ? g.copyWith(name: name.trim()) : g)
          .toList(),
    );
    await _persist(gameDir, next);
  }

  Future<void> deleteGroup(String gameDir, String id) async {
    final groups = state.groups.where((g) => g.id != id).toList();
    final assignments = Map<String, String>.from(state.assignments)
      ..removeWhere((_, gid) => gid == id);
    await _persist(
      gameDir,
      state.copyWith(groups: groups, assignments: assignments),
    );
  }

  Future<void> setGroupCollapsed(
    String gameDir,
    String id,
    bool collapsed,
  ) async {
    final next = state.copyWith(
      groups: state.groups
          .map((g) => g.id == id ? g.copyWith(collapsed: collapsed) : g)
          .toList(),
    );
    await _persist(gameDir, next);
  }

  Future<void> assignMod(
    String gameDir,
    String modId,
    String? groupId,
  ) async {
    final assignments = Map<String, String>.from(state.assignments);
    if (groupId == null) {
      assignments.remove(modId);
    } else {
      assignments[modId] = groupId;
    }
    await _persist(gameDir, state.copyWith(assignments: assignments));
  }

  Future<void> renameMod(String gameDir, String modId, String? name) async {
    final modNames = Map<String, String>.from(state.modNames);
    final trimmed = name?.trim();
    if (trimmed == null || trimmed.isEmpty) {
      modNames.remove(modId);
    } else {
      modNames[modId] = trimmed;
    }
    await _persist(gameDir, state.copyWith(modNames: modNames));
  }

  Future<void> reorderGroups(String gameDir, List<String> orderedIds) async {
    final byId = {for (final g in state.groups) g.id: g};
    final reordered = <ModGroup>[];
    for (var i = 0; i < orderedIds.length; i++) {
      final g = byId[orderedIds[i]];
      if (g != null) reordered.add(g.copyWith(order: i));
    }
    await _persist(gameDir, state.copyWith(groups: reordered));
  }
}
