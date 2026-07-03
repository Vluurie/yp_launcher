class ModGroup {
  final String id;
  final String name;
  final int order;
  final bool collapsed;

  const ModGroup({
    required this.id,
    required this.name,
    required this.order,
    this.collapsed = false,
  });

  ModGroup copyWith({String? name, int? order, bool? collapsed}) {
    return ModGroup(
      id: id,
      name: name ?? this.name,
      order: order ?? this.order,
      collapsed: collapsed ?? this.collapsed,
    );
  }

  factory ModGroup.fromJson(Map<String, dynamic> json) {
    return ModGroup(
      id: json['id'] as String,
      name: json['name'] as String? ?? '',
      order: (json['order'] as num?)?.toInt() ?? 0,
      collapsed: json['collapsed'] as bool? ?? false,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'order': order,
        'collapsed': collapsed,
      };
}

class ModGroupsData {
  final List<ModGroup> groups;
  final Map<String, String> assignments;
  final Map<String, String> modNames;

  const ModGroupsData({
    this.groups = const [],
    this.assignments = const {},
    this.modNames = const {},
  });

  ModGroupsData copyWith({
    List<ModGroup>? groups,
    Map<String, String>? assignments,
    Map<String, String>? modNames,
  }) {
    return ModGroupsData(
      groups: groups ?? this.groups,
      assignments: assignments ?? this.assignments,
      modNames: modNames ?? this.modNames,
    );
  }

  List<ModGroup> get sortedGroups {
    final list = List<ModGroup>.of(groups);
    list.sort((a, b) => a.order.compareTo(b.order));
    return list;
  }

  String? groupIdOf(String modId) => assignments[modId];

  String? customNameOf(String modId) => modNames[modId];

  factory ModGroupsData.fromJson(Map<String, dynamic> json) {
    final rawGroups = json['groups'] as List? ?? const [];
    final rawAssign = json['assignments'] as Map? ?? const {};
    final rawNames = json['modNames'] as Map? ?? const {};
    return ModGroupsData(
      groups: rawGroups
          .whereType<Map>()
          .map((m) => ModGroup.fromJson(Map<String, dynamic>.from(m)))
          .toList(),
      assignments: rawAssign.map(
        (k, v) => MapEntry(k.toString(), v.toString()),
      ),
      modNames: rawNames.map(
        (k, v) => MapEntry(k.toString(), v.toString()),
      ),
    );
  }

  Map<String, dynamic> toJson() => {
        'groups': groups.map((g) => g.toJson()).toList(),
        'assignments': assignments,
        'modNames': modNames,
      };
}
