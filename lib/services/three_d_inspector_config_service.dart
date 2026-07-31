import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/toml_service.dart';

const inspectorStateSelfDestructed = 'self_destructed';
const inspectorStateEyemask = 'eyemask';
const inspectorStateCombat = 'combat';
const inspectorStatePrologue = 'prologue';
const inspectorStateHoly = 'holy';
const inspectorStateNoRight = 'no_right';
const inspectorStateTower = 'tower';
const inspectorStateWig = 'wig';

class InspectorMeshState {
  final String id;
  final List<String> activeShowMeshes;
  final List<String> activeHideMeshes;
  final List<String> inactiveShowMeshes;
  final List<String> inactiveHideMeshes;
  final bool replacesBase;

  const InspectorMeshState({
    required this.id,
    this.activeShowMeshes = const [],
    this.activeHideMeshes = const [],
    this.inactiveShowMeshes = const [],
    this.inactiveHideMeshes = const [],
    this.replacesBase = false,
  });

  bool get hasMeshes =>
      activeShowMeshes.isNotEmpty ||
      activeHideMeshes.isNotEmpty ||
      inactiveShowMeshes.isNotEmpty ||
      inactiveHideMeshes.isNotEmpty;
}

class InspectorOutfitPreset {
  final String id;
  final int outfitId;
  final String name;
  final bool customAssembly;
  final List<String> baseMeshes;
  final List<InspectorMeshState> states;
  final List<String> forceShowMeshes;
  final List<String> forceHideMeshes;

  const InspectorOutfitPreset({
    required this.id,
    required this.outfitId,
    required this.name,
    required this.customAssembly,
    this.baseMeshes = const [],
    this.states = const [],
    this.forceShowMeshes = const [],
    this.forceHideMeshes = const [],
  });

  Set<String> visibleMeshNames(Map<String, bool> stateValues) {
    final visible = <String>{...baseMeshes};
    for (final state in states) {
      if (stateValues[state.id] ?? false) {
        if (state.replacesBase) visible.removeAll(baseMeshes);
        visible
          ..removeAll(state.activeHideMeshes)
          ..addAll(state.activeShowMeshes);
      } else {
        visible
          ..removeAll(state.inactiveHideMeshes)
          ..addAll(state.inactiveShowMeshes);
      }
    }
    visible
      ..removeAll(forceHideMeshes)
      ..addAll(forceShowMeshes);
    return visible;
  }

  Set<String> get configuredMeshNames => {
    ...baseMeshes,
    for (final state in states) ...state.activeShowMeshes,
    for (final state in states) ...state.activeHideMeshes,
    for (final state in states) ...state.inactiveShowMeshes,
    for (final state in states) ...state.inactiveHideMeshes,
    ...forceShowMeshes,
    ...forceHideMeshes,
  };
}

class ThreeDInspectorConfigService {
  static List<InspectorOutfitPreset> load(DataArchivePair archive) {
    final character = _characterForStem(archive.stem);
    if (character == null) return const [];
    final root = Directory(archive.configRoot);
    if (!root.existsSync()) return const [];
    final presets = <InspectorOutfitPreset>[
      ..._loadNative(root, character),
      ..._loadWax(root, character),
    ];
    presets.sort((a, b) {
      final byId = a.outfitId.compareTo(b.outfitId);
      return byId != 0 ? byId : a.name.compareTo(b.name);
    });
    return presets;
  }

  static List<InspectorOutfitPreset> inferVanillaVariants({
    required String archiveStem,
    required Iterable<String> meshNames,
  }) {
    final available = meshNames.toSet();
    final definitions = _vanillaVariants[_characterForStem(archiveStem)];
    if (definitions == null) return const [];
    final variants = <InspectorOutfitPreset>[];
    final seen = <String>{};
    for (final definition in definitions) {
      final meshes = definition.meshes.where(available.contains).toSet();
      if (meshes.isEmpty) continue;
      final key = (meshes.toList()..sort()).join('\u0000');
      if (!seen.add(key)) continue;
      variants.add(_vanillaPreset(definition.id, definition.outfitId, meshes));
    }
    return variants;
  }
}

InspectorOutfitPreset _vanillaPreset(
  String id,
  int outfitId,
  Set<String> meshes,
) {
  return InspectorOutfitPreset(
    id: id,
    outfitId: outfitId,
    name: '',
    customAssembly: true,
    baseMeshes: meshes.toList()..sort(),
  );
}

class _VanillaVariant {
  final String id;
  final int outfitId;
  final List<String> meshes;

  const _VanillaVariant(this.id, this.outfitId, this.meshes);
}

const _vanillaVariants = <String, List<_VanillaVariant>>{
  '2B': [
    _VanillaVariant('vanilla:normal', -1, [
      'Body',
      'Eyelash',
      'Eyemask',
      'facial_normal',
      'facial_serious',
      'Feather',
      'Hair',
      'Skirt',
    ]),
    _VanillaVariant('vanilla:damaged', -2, [
      'Broken',
      'Body',
      'Eyelash',
      'Eyemask',
      'facial_normal',
      'facial_serious',
      'Feather',
      'Hair',
    ]),
    _VanillaVariant('vanilla:base', -3, [
      'Body',
      'Eyelash',
      'Eyemask',
      'facial_normal',
      'facial_serious',
      'Hair',
    ]),
    _VanillaVariant('vanilla:armor', -4, ['Armor_Body', 'Armor_Head']),
    _VanillaVariant('vanilla:dlc', -5, [
      'DLC_Body',
      'DLC_Skirt',
      'Eyelash',
      'facial_normal',
      'facial_serious',
      'Hair',
    ]),
    _VanillaVariant('vanilla:dlc_damaged', -6, [
      'DLC_Body',
      'DLC_Broken',
      'Eyelash',
      'facial_normal',
      'facial_serious',
      'Hair',
    ]),
  ],
  '9S': [
    _VanillaVariant('vanilla:normal', -1, [
      'Body',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'Leg',
      'Pants',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
      'mesh_pl0200',
    ]),
    _VanillaVariant('vanilla:self_destruct', -2, [
      'Body',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'Leg',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
      'mesh_pl0200',
    ]),
    _VanillaVariant('vanilla:damaged_left', -3, [
      'Body',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'Leg',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
      'mesh_es0200',
    ]),
    _VanillaVariant('vanilla:damaged_right', -4, [
      'Body',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'Leg',
      'Pants',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
      'mesh_es0206',
    ]),
    _VanillaVariant('vanilla:damaged_holes', -5, [
      'Body',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'Leg',
      'Pants',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
      'mesh_es0201',
    ]),
    _VanillaVariant('vanilla:damaged_2b_hand', -6, [
      'Body',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'Leg',
      'Pants',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
      'mesh_es0202',
    ]),
    _VanillaVariant('vanilla:dlc', -7, [
      'DLC_Body',
      'DLC_Leg',
      'DLC_Pants',
      'DLC_mesh_pl0200',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
    ]),
    _VanillaVariant('vanilla:dlc_damaged_left', -8, [
      'DLC_Body',
      'DLC_Leg',
      'DLC_mesh_es0200',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
    ]),
    _VanillaVariant('vanilla:dlc_damaged_right', -9, [
      'DLC_Body',
      'DLC_Leg',
      'DLC_Pants',
      'DLC_mesh_es0206',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
    ]),
    _VanillaVariant('vanilla:dlc_damaged_holes', -10, [
      'DLC_Body',
      'DLC_Leg',
      'DLC_Pants',
      'DLC_mesh_es0201',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
    ]),
    _VanillaVariant('vanilla:dlc_damaged_2b_hand', -11, [
      'DLC_Body',
      'DLC_Leg',
      'DLC_Pants',
      'DLC_mesh_es0202',
      'Eyelash',
      'Eyelash_serious',
      'Eyemask',
      'facial_normal',
      'facial_normal-2',
      'facial_serious',
      'facial_serious-2',
    ]),
  ],
  'A2': [
    _VanillaVariant('vanilla:normal', -1, [
      'Body',
      'Cloth',
      'Hair',
      'facial_normal',
      'facial_serious',
    ]),
    _VanillaVariant('vanilla:berserk', -2, [
      'Body',
      'Hair',
      'facial_normal',
      'facial_serious',
    ]),
    _VanillaVariant('vanilla:dlc', -3, [
      'DLC_Body',
      'DLC_Cloth',
      'Hair',
      'facial_normal',
      'facial_serious',
    ]),
  ],
};

List<InspectorOutfitPreset> _loadNative(Directory root, String character) {
  final entities = Directory(path.join(root.path, 'entities'));
  if (!entities.existsSync()) return const [];
  final files =
      entities
          .listSync(recursive: true, followLinks: false)
          .whereType<File>()
          .where((file) => file.path.toLowerCase().endsWith('.toml'))
          .toList()
        ..sort((a, b) => a.path.compareTo(b.path));
  final parsedFiles = <({File file, Map<String, dynamic> values})>[];
  for (final file in files) {
    final values = TomlService.parse(file.readAsStringSync());
    parsedFiles.add((file: file, values: values));
  }
  final overrides = <_MeshOverride>[];
  for (final parsed in parsedFiles) {
    overrides.addAll(_readOverrides(parsed.values, character));
  }
  final presets = <InspectorOutfitPreset>[];
  for (final parsed in parsedFiles) {
    final fileName = path.basename(parsed.file.path).toLowerCase();
    if (!fileName.startsWith('outfit_')) continue;
    final values = parsed.values;
    if (values['character'] != character) continue;
    final outfitId = _intValue(values['outfit_id']);
    if (outfitId == null) continue;
    final baseMeshes = _stringList(values['meshes']);
    final states = _readStates(values);
    final customAssembly = _hasAnyMeshList(values);
    final matchingOverrides = overrides.where(
      (override) => override.outfitId == null || override.outfitId == outfitId,
    );
    final rawName = values['name'];
    final name = rawName is String ? rawName : '';
    presets.add(
      InspectorOutfitPreset(
        id: '${parsed.file.path}|$outfitId',
        outfitId: outfitId,
        name: name,
        customAssembly: customAssembly,
        baseMeshes: baseMeshes,
        states: states,
        forceShowMeshes: [
          for (final override in matchingOverrides)
            if (override.show) override.mesh,
        ],
        forceHideMeshes: [
          for (final override in matchingOverrides)
            if (!override.show) override.mesh,
        ],
      ),
    );
  }
  return presets;
}

List<InspectorOutfitPreset> _loadWax(Directory root, String character) {
  final listKey = switch (character) {
    '2B' => 'pl_2b_outfits',
    '9S' => 'pl_9s_outfits',
    'A2' => 'pl_a2_outfits',
    _ => '',
  };
  if (listKey.isEmpty) return const [];
  final files =
      root
          .listSync(recursive: true, followLinks: false)
          .whereType<File>()
          .where((file) {
            if (!file.path.toLowerCase().endsWith('.json')) return false;
            final parts = path
                .split(file.path)
                .map((part) => part.toLowerCase());
            return parts.contains('outfits');
          })
          .toList()
        ..sort((a, b) => a.path.compareTo(b.path));
  final presets = <InspectorOutfitPreset>[];
  for (final file in files) {
    Object? decoded;
    try {
      decoded = jsonDecode(file.readAsStringSync());
    } catch (_) {
      continue;
    }
    if (decoded is! Map) continue;
    final outfits = decoded[listKey];
    if (outfits is! List) continue;
    for (final raw in outfits) {
      if (raw is! Map) continue;
      final values = raw.map((key, value) => MapEntry(key.toString(), value));
      final outfitId = _intValue(values['outfit_id']);
      if (outfitId == null) continue;
      final baseMeshes = _stringList(values['meshes']);
      final states = _readStates(values);
      final rawName = values['outfit_name'];
      presets.add(
        InspectorOutfitPreset(
          id: '${file.path}|$outfitId',
          outfitId: outfitId,
          name: rawName is String && rawName.isNotEmpty ? rawName : '',
          customAssembly: _hasAnyMeshList(values),
          baseMeshes: baseMeshes,
          states: states,
        ),
      );
    }
  }
  return presets;
}

List<InspectorMeshState> _readStates(Map values) {
  final combatMeshes = _stringList(values['combat_meshes']);
  final nonCombatMeshes = _stringList(values['non_combat_meshes']);
  final nonEyemaskMeshes = _stringList(values['non_eyemask_meshes']);
  final plainEyelash =
      [...nonEyemaskMeshes, ...combatMeshes, ...nonCombatMeshes].where(
        (name) => name.endsWith('Eyelash') && !name.endsWith('Eyelash_serious'),
      );
  final seriousEyelash = plainEyelash.isEmpty
      ? const <String>[]
      : <String>['${plainEyelash.first}_serious'];
  // Mirrors the in-game plan order: story -> broken/self-destruct -> eyemask
  // -> combat. Broken/eyemask branches only show meshes; hiding comes from the
  // hide-all baseline, except the explicit self_destruct_hide list.
  return [
    InspectorMeshState(
      id: inspectorStatePrologue,
      activeShowMeshes: _stringList(values['prologue_meshes']),
      replacesBase: true,
    ),
    InspectorMeshState(
      id: inspectorStateHoly,
      activeShowMeshes: _stringList(values['holy_meshes']),
      replacesBase: true,
    ),
    InspectorMeshState(
      id: inspectorStateNoRight,
      activeShowMeshes: _stringList(values['no_right_meshes']),
      replacesBase: true,
    ),
    InspectorMeshState(
      id: inspectorStateTower,
      activeShowMeshes: _stringList(values['tower_meshes']),
      replacesBase: true,
    ),
    InspectorMeshState(
      id: inspectorStateSelfDestructed,
      activeShowMeshes: [
        ..._stringList(values['broken_meshes']),
        ..._stringList(values['self_destruct_show_meshes']),
      ],
      activeHideMeshes: _stringList(values['self_destruct_hide_meshes']),
      inactiveShowMeshes: _stringList(values['non_broken_meshes']),
    ),
    InspectorMeshState(
      id: inspectorStateEyemask,
      activeShowMeshes: _stringList(values['eyemask_meshes']),
      inactiveShowMeshes: nonEyemaskMeshes,
    ),
    InspectorMeshState(
      id: inspectorStateCombat,
      activeShowMeshes: [...combatMeshes, ...seriousEyelash],
      activeHideMeshes: nonCombatMeshes,
      inactiveShowMeshes: nonCombatMeshes,
      inactiveHideMeshes: [...combatMeshes, ...seriousEyelash],
    ),
    InspectorMeshState(
      id: inspectorStateWig,
      activeHideMeshes: _stringList(values['wig_meshes']),
    ),
  ].where((state) => state.hasMeshes).toList();
}

List<_MeshOverride> _readOverrides(
  Map<String, dynamic> values,
  String character,
) {
  final raw = values['mesh_override'];
  final entries = raw is List ? raw : const [];
  final acceptedModels = switch (character) {
    '2B' => const {'pl2b'},
    '9S' => const {'pl9s'},
    'A2' => const {'pla2'},
    _ => const <String>{},
  };
  final overrides = <_MeshOverride>[];
  for (final entry in entries) {
    if (entry is! Map) continue;
    final model = entry['model'];
    final mesh = entry['mesh'];
    final force = entry['force'];
    if (model is! String ||
        !acceptedModels.contains(model) ||
        mesh is! String ||
        mesh.isEmpty ||
        force is! String ||
        (force != 'show' && force != 'hide')) {
      continue;
    }
    overrides.add(
      _MeshOverride(
        mesh: mesh,
        show: force == 'show',
        outfitId: _intValue(entry['outfit_id']),
      ),
    );
  }
  return overrides;
}

String? _characterForStem(String stem) {
  final normalized = stem.toLowerCase();
  if (normalized.startsWith('pl000')) return '2B';
  if (normalized.startsWith('pl020')) return '9S';
  if (normalized.startsWith('pl010')) return 'A2';
  return null;
}

List<String> _stringList(Object? value) {
  if (value is! List) return const [];
  return [
    for (final item in value)
      if (item is String && item.isNotEmpty) item,
  ];
}

int? _intValue(Object? value) {
  if (value is int) return value;
  if (value is BigInt) return value.toInt();
  return null;
}

bool _hasAnyMeshList(Map values) {
  const fields = {
    'meshes',
    'self_destruct_show_meshes',
    'self_destruct_hide_meshes',
    'combat_meshes',
    'non_combat_meshes',
    'eyemask_meshes',
    'non_eyemask_meshes',
    'wig_meshes',
    'hairspray_meshes',
    'broken_meshes',
    'non_broken_meshes',
    'prologue_meshes',
    'holy_meshes',
    'no_right_meshes',
    'tower_meshes',
  };
  return fields.any((field) => _stringList(values[field]).isNotEmpty);
}

class _MeshOverride {
  final String mesh;
  final bool show;
  final int? outfitId;

  const _MeshOverride({
    required this.mesh,
    required this.show,
    required this.outfitId,
  });
}
