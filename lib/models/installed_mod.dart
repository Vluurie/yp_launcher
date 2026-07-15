enum ModKind { native, data, texture, unknown }

enum DataCategory {
  player3d,
  weapon3d,
  enemy3d,
  accessory3d,
  item3d,
  worldProp3d,
  modelVariant3d,
  map3d,
  scripting,
  localization,
  effects,
  ui,
  misc,
  cutscenes,
  audio,
  archive,
  other,
}

enum ModConflictKind {
  overlappingDataFile,
  outfitIdCollision,
  vanillaOutfitId,
}

class ManifestInfo {
  final String? id;
  final String? displayName;
  final String? author;
  final String? version;
  final String? namsMinVersion;
  final List<String> requires;
  final List<String> requiresPlugins;
  final List<String> replaces;

  const ManifestInfo({
    this.id,
    this.displayName,
    this.author,
    this.version,
    this.namsMinVersion,
    this.requires = const [],
    this.requiresPlugins = const [],
    this.replaces = const [],
  });
}

class NativeSummary {
  final Map<String, int> bundlesByKind;
  final int totalEntityFiles;

  const NativeSummary({
    this.bundlesByKind = const {},
    this.totalEntityFiles = 0,
  });
}


class DataDirEntry {
  final String dirName;
  final DataCategory category;
  final int fileCount;

  const DataDirEntry({
    required this.dirName,
    required this.category,
    required this.fileCount,
  });
}

class PlayerModelEntry {
  final String fileName;
  final String label;

  const PlayerModelEntry({required this.fileName, required this.label});
}

class DataSummary {
  final List<DataDirEntry> entries;
  final List<PlayerModelEntry> players;
  final bool hasCompatConfig;

  const DataSummary({
    this.entries = const [],
    this.players = const [],
    this.hasCompatConfig = false,
  });
}

class ModConflict {
  final String otherModId;
  final ModConflictKind kind;
  final String detail;

  const ModConflict({
    required this.otherModId,
    required this.kind,
    required this.detail,
  });
}

class InstalledMod {
  final String id;
  final String displayName;
  final String rootPath;
  final ModKind kind;
  final ManifestInfo? manifest;
  final NativeSummary? native;
  final DataSummary? data;
  final List<String> requiresMissing;
  final List<ModConflict> conflicts;
  final DateTime installedAt;
  final List<String> bundledTexturePacks;
  final List<String> bundledCutscenes;

  const InstalledMod({
    required this.id,
    required this.displayName,
    required this.rootPath,
    required this.kind,
    required this.installedAt,
    this.manifest,
    this.native,
    this.data,
    this.requiresMissing = const [],
    this.conflicts = const [],
    this.bundledTexturePacks = const [],
    this.bundledCutscenes = const [],
  });

  bool get hasDataOverlay => data != null && data!.entries.isNotEmpty;
  bool get hasWarnings => conflicts.isNotEmpty || requiresMissing.isNotEmpty;
}

class ModVariant {
  final String subPath;
  final String label;
  final ModKind kind;
  final bool textureOnly;
  final DataCategory? category;

  const ModVariant({
    required this.subPath,
    required this.label,
    required this.kind,
    this.textureOnly = false,
    this.category,
  });

  bool get isOutfit => category == DataCategory.player3d;
  bool get isWeapon => category == DataCategory.weapon3d;
}

const List<DataCategory> variantCategoryOrder = [
  DataCategory.player3d,
  DataCategory.weapon3d,
  DataCategory.accessory3d,
  DataCategory.enemy3d,
  DataCategory.modelVariant3d,
  DataCategory.item3d,
  DataCategory.worldProp3d,
  DataCategory.map3d,
  DataCategory.effects,
  DataCategory.scripting,
  DataCategory.localization,
  DataCategory.ui,
  DataCategory.cutscenes,
  DataCategory.audio,
  DataCategory.archive,
  DataCategory.misc,
  DataCategory.other,
];

const Set<DataCategory> mutuallyExclusiveVariantCategories = {
  DataCategory.weapon3d,
};

class TexturePack {
  final String path;
  final String label;

  const TexturePack({required this.path, required this.label});
}

class VariantInstallRequest {
  final String requestedName;
  final String variantSubPath;
  final List<String> texturePackSubPaths;

  const VariantInstallRequest({
    required this.requestedName,
    required this.variantSubPath,
    this.texturePackSubPaths = const [],
  });
}

class DetectedDrop {
  final String unwrappedRoot;
  final ModKind kind;
  final ManifestInfo? manifest;
  final NativeSummary? native;
  final DataSummary? data;
  final String suggestedId;
  final String? errorReason;
  final List<ModVariant> variants;
  final List<TexturePack> textureVariants;

  const DetectedDrop({
    required this.unwrappedRoot,
    required this.kind,
    required this.suggestedId,
    this.manifest,
    this.native,
    this.data,
    this.errorReason,
    this.variants = const [],
    this.textureVariants = const [],
  });

  bool get hasVariants => variants.isNotEmpty;
  bool get hasTextureVariants => textureVariants.length > 1;
}

class InstallResult {
  final bool success;
  final String? installedId;
  final String? errorMessage;
  final List<String> sideInstalledTexturePacks;

  const InstallResult.ok(
    String id, {
    this.sideInstalledTexturePacks = const [],
  })  : success = true,
        installedId = id,
        errorMessage = null;

  const InstallResult.fail(String message)
      : success = false,
        installedId = null,
        errorMessage = message,
        sideInstalledTexturePacks = const [];
}

const Map<String, DataCategory> dataDirCategoryTable = {
  'pl': DataCategory.player3d,
  'wp': DataCategory.weapon3d,
  'em': DataCategory.enemy3d,
  'ba': DataCategory.worldProp3d,
  'bg': DataCategory.worldProp3d,
  'bh': DataCategory.worldProp3d,
  'et': DataCategory.accessory3d,
  'it': DataCategory.item3d,
  'um': DataCategory.modelVariant3d,
  'wd1': DataCategory.map3d,
  'wd2': DataCategory.map3d,
  'wd3': DataCategory.map3d,
  'wd4': DataCategory.map3d,
  'wd5': DataCategory.map3d,
  'wda': DataCategory.map3d,
  'core': DataCategory.scripting,
  'ph1': DataCategory.scripting,
  'ph2': DataCategory.scripting,
  'ph3': DataCategory.scripting,
  'phf': DataCategory.scripting,
  'quest': DataCategory.scripting,
  'st1': DataCategory.scripting,
  'st2': DataCategory.scripting,
  'st5': DataCategory.scripting,
  'novel': DataCategory.localization,
  'subtitle': DataCategory.localization,
  'txtmess': DataCategory.localization,
  'ui': DataCategory.localization,
  'effect': DataCategory.effects,
  'enlighten': DataCategory.effects,
  'font': DataCategory.ui,
  'misctex': DataCategory.misc,
  'movie': DataCategory.cutscenes,
  'sound': DataCategory.audio,
};

const Map<String, String> playerModelLookup = {
  'pl0000': '2B (vanilla)',
  'pl000d': '2B (DLC clothes)',
  'pl0001': '2B (armoured)',
  'pl000e': 'dead YoRHa (2B body)',
  'pl0010': 'flight unit',
  'pl0013': 'YoRHa flight unit',
  'pl0100': 'A2 (long hair)',
  'pl010d': 'A2 (DLC clothes, long hair)',
  'pl010e': 'A2 short hair (DLC)',
  'pl0101': 'A2 short hair',
  'pl0200': '9S (vanilla)',
  'pl0203': '9S',
  'pl020d': '9S (DLC clothes)',
  'pl020e': 'dead YoRHa (9S body)',
  'pl0300': 'YoRHa (background squad variants)',
  'pl0400': 'YoRHa armour',
  'pl0500': 'Devola',
  'pl0600': 'Popola',
  'pl1000': 'Pascal',
  'pl1001': 'Pascal (flying)',
  'pl1010': 'Emil',
  'pl1011': 'Emil head',
  'pl1020': 'Commander',
  'pl1030': 'Anemone',
  'pl1040': 'Devola (alt files)',
  'pl1050': 'Popola (alt files)',
  'pl1100': "Engels' back",
  'pl2000': 'YoRHa (background variants)',
  'pl2010': 'YoRHa (armoured)',
  'pl2020': 'operator base',
  'pl2021': 'operator 21O hair',
  'pl2022': 'operator 6O hair',
  'pl2023': 'Adam hair',
  'pl2030': 'Jackass',
  'pl2031': 'strange resistance woman',
  'pl2040': 'resistance androids',
  'pl2050': 'resistance androids (more)',
  'pl2060': '4S',
  'plf000': 'high-detail face',
  'plf00d': 'high-detail face (DLC, 2B)',
  'plf10d': 'high-detail face (DLC, A2)',
  'plf20d': 'high-detail face (DLC, 9S)',
  'plf100': 'high-detail face',
  'plf200': 'high-detail face',
};

const Set<String> conditionalOutfitSlots = {
  'pl0000', 'pl000d',
  'pl0100', 'pl010d',
  'pl0200', 'pl020d',
};
