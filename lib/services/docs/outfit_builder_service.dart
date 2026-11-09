import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mods_service.dart';

enum OutfitCharacter { twoB, nineS, a2 }

extension OutfitCharacterInfo on OutfitCharacter {
  String get tomlValue => switch (this) {
        OutfitCharacter.twoB => '2B',
        OutfitCharacter.nineS => '9S',
        OutfitCharacter.a2 => 'A2',
      };

  List<String> get modelStems => switch (this) {
        OutfitCharacter.twoB => const ['pl000d', 'pl0000'],
        OutfitCharacter.nineS => const ['pl020d', 'pl0200'],
        OutfitCharacter.a2 => const ['pl010d', 'pl0100'],
      };

  int get firstOutfitId => switch (this) {
        OutfitCharacter.twoB => 10001,
        OutfitCharacter.nineS => 20001,
        OutfitCharacter.a2 => 30001,
      };

  bool get hasEyemask => this != OutfitCharacter.a2;

  bool get hasWig => this == OutfitCharacter.a2;

  bool get hasStoryStates => this == OutfitCharacter.nineS;

  bool get hasBrokenStates => this == OutfitCharacter.twoB;
}

enum MeshState {
  base,
  combat,
  nonCombat,
  eyemask,
  nonEyemask,
  selfDestructShow,
  selfDestructHide,
  broken,
  nonBroken,
  wig,
  hairspray,
  prologue,
  holy,
  noRight,
  tower,
}

extension MeshStateInfo on MeshState {
  String get field => switch (this) {
        MeshState.base => 'meshes',
        MeshState.combat => 'combat_meshes',
        MeshState.nonCombat => 'non_combat_meshes',
        MeshState.eyemask => 'eyemask_meshes',
        MeshState.nonEyemask => 'non_eyemask_meshes',
        MeshState.selfDestructShow => 'self_destruct_show_meshes',
        MeshState.selfDestructHide => 'self_destruct_hide_meshes',
        MeshState.broken => 'broken_meshes',
        MeshState.nonBroken => 'non_broken_meshes',
        MeshState.wig => 'wig_meshes',
        MeshState.hairspray => 'hairspray_meshes',
        MeshState.prologue => 'prologue_meshes',
        MeshState.holy => 'holy_meshes',
        MeshState.noRight => 'no_right_meshes',
        MeshState.tower => 'tower_meshes',
      };

  bool appliesTo(OutfitCharacter character) => switch (this) {
        MeshState.eyemask || MeshState.nonEyemask => character.hasEyemask,
        MeshState.wig => character.hasWig,
        MeshState.broken || MeshState.nonBroken => character.hasBrokenStates,
        MeshState.prologue ||
        MeshState.holy ||
        MeshState.noRight ||
        MeshState.tower =>
          character.hasStoryStates,
        _ => true,
      };
}

enum MeshVisibility {
  always,
  combatOnly,
  outOfCombatOnly,
  eyemaskOnly,
  noEyemaskOnly,
  featherOnly,
  brokenOnly,
  never,
}

extension MeshVisibilityInfo on MeshVisibility {
  MeshState? get state => switch (this) {
        MeshVisibility.always => MeshState.base,
        MeshVisibility.combatOnly => MeshState.combat,
        MeshVisibility.outOfCombatOnly => MeshState.nonCombat,
        MeshVisibility.eyemaskOnly => MeshState.eyemask,
        MeshVisibility.noEyemaskOnly => MeshState.nonEyemask,
        MeshVisibility.featherOnly => MeshState.nonBroken,
        MeshVisibility.brokenOnly => MeshState.broken,
        MeshVisibility.never => null,
      };

  bool appliesTo(OutfitCharacter character) => switch (this) {
        MeshVisibility.eyemaskOnly ||
        MeshVisibility.noEyemaskOnly =>
          character.hasEyemask,
        MeshVisibility.featherOnly ||
        MeshVisibility.brokenOnly =>
          character.hasBrokenStates,
        _ => true,
      };
}

class MeshRule {
  final MeshVisibility visibility;
  final bool hideOnSelfDestruct;
  final bool tintedByHairspray;
  final bool replacedByWig;

  const MeshRule({
    this.visibility = MeshVisibility.always,
    this.hideOnSelfDestruct = false,
    this.tintedByHairspray = false,
    this.replacedByWig = false,
  });

  MeshRule copyWith({
    MeshVisibility? visibility,
    bool? hideOnSelfDestruct,
    bool? tintedByHairspray,
    bool? replacedByWig,
  }) {
    return MeshRule(
      visibility: visibility ?? this.visibility,
      hideOnSelfDestruct: hideOnSelfDestruct ?? this.hideOnSelfDestruct,
      tintedByHairspray: tintedByHairspray ?? this.tintedByHairspray,
      replacedByWig: replacedByWig ?? this.replacedByWig,
    );
  }

  bool get isDefault =>
      visibility == MeshVisibility.always &&
      !hideOnSelfDestruct &&
      !tintedByHairspray &&
      !replacedByWig;
}

class OutfitDraft {
  final String displayName;
  final String description;
  final OutfitCharacter character;
  final int outfitId;
  final int itemId;
  final String? modelSourcePath;
  final Map<MeshState, Set<String>> meshes;
  final bool autoGive;
  final String? shopName;
  final Set<int> shopStates;
  final int buyPrice;

  const OutfitDraft({
    required this.displayName,
    required this.description,
    required this.character,
    required this.outfitId,
    required this.itemId,
    this.modelSourcePath,
    this.meshes = const {},
    this.autoGive = true,
    this.shopName,
    this.shopStates = const {0},
    this.buyPrice = 5000,
  });

  bool get sellsInShop => shopName != null && shopName!.isNotEmpty;

  OutfitDraft copyWith({
    String? displayName,
    String? description,
    OutfitCharacter? character,
    int? outfitId,
    int? itemId,
    String? modelSourcePath,
    Map<MeshState, Set<String>>? meshes,
    bool? autoGive,
    String? shopName,
    Set<int>? shopStates,
    int? buyPrice,
    bool clearModel = false,
    bool clearShop = false,
  }) {
    return OutfitDraft(
      displayName: displayName ?? this.displayName,
      description: description ?? this.description,
      character: character ?? this.character,
      outfitId: outfitId ?? this.outfitId,
      itemId: itemId ?? this.itemId,
      modelSourcePath:
          clearModel ? null : (modelSourcePath ?? this.modelSourcePath),
      meshes: meshes ?? this.meshes,
      autoGive: autoGive ?? this.autoGive,
      shopName: clearShop ? null : (shopName ?? this.shopName),
      shopStates: shopStates ?? this.shopStates,
      buyPrice: buyPrice ?? this.buyPrice,
    );
  }

  Set<String> meshesFor(MeshState state) => meshes[state] ?? const {};

  bool get hasAnyMeshRules =>
      meshes.values.any((names) => names.isNotEmpty);

  String get internalName {
    final buffer = StringBuffer();
    var lastWasUnderscore = true;

    for (final rune in displayName.toLowerCase().runes) {
      final char = String.fromCharCode(rune);
      final isWord =
          (rune >= 0x30 && rune <= 0x39) || (rune >= 0x61 && rune <= 0x7a);

      if (isWord) {
        buffer.write(char);
        lastWasUnderscore = false;
      } else if (!lastWasUnderscore) {
        buffer.write('_');
        lastWasUnderscore = true;
      }
    }

    var name = buffer.toString();
    if (name.endsWith('_')) name = name.substring(0, name.length - 1);
    return name.isEmpty ? 'unnamed_outfit' : name;
  }
}

class OutfitBuilderService {
  OutfitBuilderService._();

  static const modelExtensions = ['.dat', '.dtt'];

  static const outfitItemCategory = 6;

  static const maxShopState = 6;

  static const shopStateKeys = [
    'routeABeforeRuinsCollapse',
    'routeAAfterRuinsCollapse',
    'routeBBeforeRuinsCollapse',
    'routeBAfterRuinsCollapse',
    'routeC',
    'endingCOrD',
    'endingCAndD',
  ];

  static const shopNames = [
    'shop_RC_Grocery',
    'shop_RC_Grocery_small',
    'shop_PV_Grocery',
    'shop_PV_UniqueShop',
    'shop_AP_Grocery',
    'shop_CS_Grocery',
    'shop_DC_Grocery',
    'shop_DO_Grocery',
    'shop_DP_Grocery',
    'shop_FC_Grocery',
    'shop_GP_Grocery',
    'shop_PC_Grocery',
    'shop_PC_Grocery_first',
    'shop_VF_Grocery',
    'shop_BK_Grocery',
    'shop_CD_Grocery',
    'shop_CF_Grocery',
    'shop_Emile',
    'shop_rareEmile',
  ];

  static String shopItemToml(OutfitDraft draft) {
    final buffer = StringBuffer();
    final states = draft.shopStates.toList()..sort();

    for (final state in states) {
      if (buffer.isNotEmpty) buffer.writeln();
      buffer
        ..writeln('[[shop_item]]')
        ..writeln('shop_name = "${_escape(draft.shopName ?? '')}"')
        ..writeln('item_id = ${draft.itemId}')
        ..writeln('state = $state');
    }

    return buffer.toString();
  }

  static String shopFileNameFor(OutfitDraft draft) =>
      'shop${draft.itemId}.toml';

  static Future<File> writeShopItem({
    required String gameDir,
    required String modId,
    required OutfitDraft draft,
  }) async {
    final file = File(
      p.join(
        ModsService.modsDir(gameDir),
        modId,
        'entities',
        shopFileNameFor(draft),
      ),
    );
    await file.parent.create(recursive: true);
    await file.writeAsString(shopItemToml(draft));
    return file;
  }

  static String toToml(OutfitDraft draft) {
    final buffer = StringBuffer()
      ..writeln('outfit_id = ${draft.outfitId}')
      ..writeln('character = "${draft.character.tomlValue}"')
      ..writeln('item_id = ${draft.itemId}')
      ..writeln('name = "${_escape(draft.internalName)}"');

    if (!draft.hasAnyMeshRules) return buffer.toString();

    buffer.writeln();
    for (final state in MeshState.values) {
      if (!state.appliesTo(draft.character)) continue;
      final names = draft.meshesFor(state);
      if (names.isEmpty) continue;
      final list = names.map((n) => '"${_escape(n)}"').join(', ');
      buffer.writeln('${state.field} = [$list]');
    }

    return buffer.toString();
  }

  static String fileNameFor(OutfitDraft draft) => 'outfit_${draft.outfitId}.toml';

  static String pathFor(String gameDir, String modId, OutfitDraft draft) {
    return p.join(
      ModsService.modsDir(gameDir),
      modId,
      'entities',
      fileNameFor(draft),
    );
  }

  static String modelDirFor(String gameDir, String modId) {
    return p.join(ModsService.modsDir(gameDir), modId, 'data', 'pl');
  }

  static bool isModelFile(String path) {
    final extension = p.extension(path).toLowerCase();
    return modelExtensions.contains(extension);
  }

  static Map<MeshState, Set<String>> listsFromRules(
    OutfitCharacter character,
    Map<String, MeshRule> rules,
  ) {
    final result = <MeshState, Set<String>>{};

    void add(MeshState state, String name) {
      if (!state.appliesTo(character)) return;
      result.putIfAbsent(state, () => <String>{}).add(name);
    }

    if (rules.values.every((rule) => rule.isDefault)) return result;

    for (final entry in rules.entries) {
      final name = entry.key;
      final rule = entry.value;

      final state = rule.visibility.appliesTo(character)
          ? rule.visibility.state
          : MeshState.base;
      if (state != null) add(state, name);

      if (rule.hideOnSelfDestruct) add(MeshState.selfDestructHide, name);
      if (rule.tintedByHairspray) add(MeshState.hairspray, name);
      if (rule.replacedByWig) add(MeshState.wig, name);
    }

    return result;
  }

  static String siblingPath(String path, String extension) {
    final stem = p.basenameWithoutExtension(path);
    return p.join(p.dirname(path), '$stem$extension');
  }

  static Future<String?> existingSibling(
    String path,
    String extension,
  ) async {
    final candidate = siblingPath(path, extension);
    return await File(candidate).exists() ? candidate : null;
  }

  static OutfitCharacter? characterFromModelPath(String path) {
    final stem = p.basenameWithoutExtension(path).toLowerCase();
    for (final character in OutfitCharacter.values) {
      if (character.modelStems.contains(stem)) return character;
    }
    return null;
  }

  static Future<int> nextFreeOutfitId(
    String gameDir,
    OutfitCharacter character,
  ) async {
    final taken = await takenOutfitIds(gameDir, character);
    var id = character.firstOutfitId;
    while (taken.contains(id)) {
      id++;
    }
    return id;
  }

  static Future<Set<int>> takenOutfitIds(
    String gameDir,
    OutfitCharacter character,
  ) async {
    final root = Directory(ModsService.modsDir(gameDir));
    if (!await root.exists()) return <int>{};

    final needle = '"${character.tomlValue}"';
    final taken = <int>{};

    await for (final entry in root.list(recursive: true, followLinks: false)) {
      if (entry is! File) continue;
      final name = p.basename(entry.path);
      if (!name.startsWith('outfit_') || !name.endsWith('.toml')) continue;

      String content;
      try {
        content = await entry.readAsString();
      } catch (_) {
        continue;
      }

      final id = _outfitIdIn(content);
      if (id == null) continue;
      if (_characterMatches(content, needle)) taken.add(id);
    }
    return taken;
  }

  static bool _characterMatches(String content, String needle) {
    final lower = needle.toLowerCase();
    for (final line in const LineSplitter().convert(content)) {
      final trimmed = line.trim();
      if (!trimmed.startsWith('character')) continue;
      return trimmed.toLowerCase().contains(lower);
    }
    return false;
  }

  static int? _outfitIdIn(String content) {
    for (final line in const LineSplitter().convert(content)) {
      final trimmed = line.trim();
      if (!trimmed.startsWith('outfit_id')) continue;
      final equals = trimmed.indexOf('=');
      if (equals < 0) continue;
      return int.tryParse(trimmed.substring(equals + 1).trim());
    }
    return null;
  }

  static Future<File> write({
    required String gameDir,
    required String modId,
    required OutfitDraft draft,
  }) async {
    final file = File(pathFor(gameDir, modId, draft));
    await file.parent.create(recursive: true);
    await file.writeAsString(toToml(draft));
    return file;
  }

  static Future<List<File>> copyModel({
    required String gameDir,
    required String modId,
    required String sourcePath,
  }) async {
    final targetDir = Directory(modelDirFor(gameDir, modId));
    await targetDir.create(recursive: true);

    final stem = p.basenameWithoutExtension(sourcePath);
    final sourceDir = p.dirname(sourcePath);
    final copied = <File>[];

    for (final extension in modelExtensions) {
      final source = File(p.join(sourceDir, '$stem$extension'));
      if (!await source.exists()) continue;
      final target = p.join(targetDir.path, '$stem$extension');
      copied.add(await source.copy(target));
    }

    return copied;
  }

  static String _escape(String value) =>
      value.replaceAll(r'\', r'\\').replaceAll('"', r'\"');
}
