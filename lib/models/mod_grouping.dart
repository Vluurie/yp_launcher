import 'package:yp_launcher/models/installed_mod.dart';

enum ModGroupKind {
  outfit2b,
  outfit9s,
  outfitA2,
  outfitOther,
  weapon,
  enemy,
  effect,
  scripting,
  localization,
  cutscene,
  audio,
  texture,
  native,
  other,
}

const List<ModGroupKind> modGroupOrder = [
  ModGroupKind.native,
  ModGroupKind.outfit2b,
  ModGroupKind.outfit9s,
  ModGroupKind.outfitA2,
  ModGroupKind.outfitOther,
  ModGroupKind.weapon,
  ModGroupKind.enemy,
  ModGroupKind.effect,
  ModGroupKind.scripting,
  ModGroupKind.localization,
  ModGroupKind.cutscene,
  ModGroupKind.audio,
  ModGroupKind.texture,
  ModGroupKind.other,
];

const Map<String, ModGroupKind> _playerSlotGroups = {
  'pl0000': ModGroupKind.outfit2b,
  'pl000d': ModGroupKind.outfit2b,
  'pl0001': ModGroupKind.outfit2b,
  'pl000e': ModGroupKind.outfit2b,
  'plf000': ModGroupKind.outfit2b,
  'plf00d': ModGroupKind.outfit2b,
  'pl0200': ModGroupKind.outfit9s,
  'pl0203': ModGroupKind.outfit9s,
  'pl020d': ModGroupKind.outfit9s,
  'pl020e': ModGroupKind.outfit9s,
  'plf200': ModGroupKind.outfit9s,
  'plf20d': ModGroupKind.outfit9s,
  'pl0100': ModGroupKind.outfitA2,
  'pl0101': ModGroupKind.outfitA2,
  'pl010d': ModGroupKind.outfitA2,
  'pl010e': ModGroupKind.outfitA2,
  'plf100': ModGroupKind.outfitA2,
  'plf10d': ModGroupKind.outfitA2,
};

ModGroupKind? playerSlotGroup(String fileName) {
  final lower = fileName.toLowerCase();
  final stem = lower.length >= 6 ? lower.substring(0, 6) : lower;
  return _playerSlotGroups[stem];
}

Set<ModGroupKind> groupsForMod(InstalledMod mod) {
  if (mod.kind == ModKind.native) return {ModGroupKind.native};

  final groups = <ModGroupKind>{};

  for (final player in mod.data?.players ?? const <PlayerModelEntry>[]) {
    final group = playerSlotGroup(player.fileName);
    groups.add(group ?? ModGroupKind.outfitOther);
  }
  if (groups.isNotEmpty) return groups;

  final categories = <DataCategory>{
    for (final e in mod.data?.entries ?? const <DataDirEntry>[]) e.category,
  };

  for (final entry in const {
    DataCategory.player3d: ModGroupKind.outfitOther,
    DataCategory.weapon3d: ModGroupKind.weapon,
    DataCategory.enemy3d: ModGroupKind.enemy,
    DataCategory.effects: ModGroupKind.effect,
    DataCategory.scripting: ModGroupKind.scripting,
    DataCategory.localization: ModGroupKind.localization,
    DataCategory.cutscenes: ModGroupKind.cutscene,
    DataCategory.audio: ModGroupKind.audio,
  }.entries) {
    if (categories.contains(entry.key)) groups.add(entry.value);
  }
  if (groups.isNotEmpty) return groups;

  if (mod.kind == ModKind.texture) return {ModGroupKind.texture};
  return {ModGroupKind.other};
}

Map<ModGroupKind, List<InstalledMod>> groupMods(List<InstalledMod> mods) {
  final buckets = <ModGroupKind, List<InstalledMod>>{};
  for (final mod in mods) {
    for (final group in groupsForMod(mod)) {
      buckets.putIfAbsent(group, () => []).add(mod);
    }
  }
  return {
    for (final kind in modGroupOrder)
      if (buckets[kind] != null) kind: buckets[kind]!,
  };
}
