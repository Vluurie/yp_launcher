import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mods_service.dart';

enum WeaponType { smallSword, largeSword, spear, combatBracer }

extension WeaponTypeInfo on WeaponType {
  int get category => switch (this) {
        WeaponType.smallSword => 0,
        WeaponType.largeSword => 1,
        WeaponType.spear => 2,
        WeaponType.combatBracer => 3,
      };

  int get objIdStart => switch (this) {
        WeaponType.smallSword => 0x30000,
        WeaponType.largeSword => 0x30200,
        WeaponType.spear => 0x30400,
        WeaponType.combatBracer => 0x30600,
      };

  int get objIdEnd => objIdStart + 0x200;

  int get cloneFromObjId => switch (this) {
        WeaponType.smallSword => 0x30003,
        WeaponType.largeSword => 0x30203,
        WeaponType.spear => 0x30400,
        WeaponType.combatBracer => 0x30600,
      };

  int get estIdNormal => switch (this) {
        WeaponType.smallSword => 600,
        WeaponType.largeSword => 610,
        WeaponType.spear => 620,
        WeaponType.combatBracer => 630,
      };

  int get estIdBerserk => switch (this) {
        WeaponType.smallSword => 633,
        WeaponType.largeSword => 634,
        WeaponType.spear => 635,
        WeaponType.combatBracer => 636,
      };
}

class WeaponLevel {
  final int minDamage;
  final int maxDamage;

  const WeaponLevel({required this.minDamage, required this.maxDamage});
}

enum EffectOp { mul, add, set }

class EffectPath {
  final String path;
  final EffectOp op;
  final bool perLevel;

  const EffectPath(this.path, this.op, {this.perLevel = true});
}

class WeaponEffect {
  final EffectPath path;
  final Map<int, double> levels;
  final double value;
  final String? whenPlayer;

  const WeaponEffect({
    required this.path,
    this.levels = const {},
    this.value = 1.0,
    this.whenPlayer,
  });

  String get name {
    final tail = path.path.split('.').last;
    return whenPlayer == null ? tail : '${tail}_${whenPlayer!.toLowerCase()}';
  }
}

class WeaponMaterial {
  final int itemId;
  final int quantity;

  const WeaponMaterial({required this.itemId, required this.quantity});
}

class WeaponUpgrade {
  final int lv;
  final int cost;
  final List<WeaponMaterial> materials;

  const WeaponUpgrade({
    required this.lv,
    required this.cost,
    this.materials = const [],
  });
}

class WeaponDraft {
  final String displayName;
  final String description;
  final WeaponType type;
  final int id;
  final int objId;
  final int sellPrice;
  final List<WeaponLevel> levels;
  final String? shopName;
  final Set<int> shopStates;
  final int buyPrice;
  final String? iconName;
  final List<WeaponUpgrade> upgrades;
  final List<String> story;
  final String skill1Name;
  final String skill1Desc;
  final String skill2Name;
  final String skill2Desc;
  final String helpShort;
  final int? traitId;
  final int? storyTextureSuffix;
  final List<WeaponEffect> effects;
  final int? estIdNormal;
  final int? estIdBerserk;

  const WeaponDraft({
    required this.displayName,
    required this.description,
    required this.type,
    required this.id,
    required this.objId,
    this.sellPrice = 500,
    this.levels = const [],
    this.shopName,
    this.shopStates = const {0},
    this.buyPrice = 5000,
    this.iconName,
    this.upgrades = const [],
    this.story = const [],
    this.skill1Name = '',
    this.skill1Desc = '',
    this.skill2Name = '',
    this.skill2Desc = '',
    this.helpShort = '',
    this.traitId,
    this.storyTextureSuffix,
    this.effects = const [],
    this.estIdNormal,
    this.estIdBerserk,
  });

  bool get sellsInShop => shopName != null && shopName!.isNotEmpty;

  int get effectiveEstIdNormal => estIdNormal ?? type.estIdNormal;

  int get effectiveEstIdBerserk => estIdBerserk ?? type.estIdBerserk;

  WeaponDraft copyWith({
    String? displayName,
    String? description,
    WeaponType? type,
    int? id,
    int? objId,
    int? sellPrice,
    List<WeaponLevel>? levels,
    String? shopName,
    Set<int>? shopStates,
    int? buyPrice,
    bool clearShop = false,
  }) {
    return WeaponDraft(
      displayName: displayName ?? this.displayName,
      description: description ?? this.description,
      type: type ?? this.type,
      id: id ?? this.id,
      objId: objId ?? this.objId,
      sellPrice: sellPrice ?? this.sellPrice,
      levels: levels ?? this.levels,
      shopName: clearShop ? null : (shopName ?? this.shopName),
      shopStates: shopStates ?? this.shopStates,
      buyPrice: buyPrice ?? this.buyPrice,
    );
  }

  String get internalName {
    final buffer = StringBuffer('weapon_');
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
    return name == 'weapon' ? 'weapon_unnamed' : name;
  }
}

class WeaponBuilderService {
  WeaponBuilderService._();

  static const firstId = 1000;
  static const lastId = 1999;
  static const maxLevel = 4;

  static const minEstId = 0;
  static const maxEstId = 665;
  static const blockedEstIds = [435, 436];

  static bool isEstIdAllowed(int id) =>
      id >= minEstId && id <= maxEstId && !blockedEstIds.contains(id);

  static int clampEstId(int id, {required bool up}) {
    var value = id.clamp(minEstId, maxEstId);
    while (blockedEstIds.contains(value)) {
      value += up ? 1 : -1;
      if (value < minEstId || value > maxEstId) return id;
    }
    return value;
  }

  static const effectPaths = [
    EffectPath('player.gold_gain', EffectOp.mul),
    EffectPath('player.exp_gain', EffectOp.mul),
    EffectPath('player.move_speed', EffectOp.mul),
    EffectPath('player.melee_out', EffectOp.mul),
    EffectPath('player.ranged_out', EffectOp.mul),
    EffectPath('player.melee_in_reduction', EffectOp.add),
    EffectPath('player.ranged_in_reduction', EffectOp.add),
    EffectPath('player.pod_cooldown_reduction', EffectOp.add),
    EffectPath('player.hacking_level_add', EffectOp.add),
    EffectPath('player.stagger_immune', EffectOp.set, perLevel: false),
    EffectPath('player.electric_immune', EffectOp.set, perLevel: false),
    EffectPath('player.eyemask_hidden', EffectOp.set, perLevel: false),
    EffectPath('player.a2_wig_show', EffectOp.set, perLevel: false),
    EffectPath('player.block_exp', EffectOp.set, perLevel: false),
    EffectPath('player.taunt_active', EffectOp.set, perLevel: false),
    EffectPath('player.dodge.propulsion_9s.forward', EffectOp.mul),
    EffectPath('player.dodge.propulsion_default.forward', EffectOp.mul),
    EffectPath('player.dodge.propulsion_flight_suit.forward', EffectOp.mul),
    EffectPath('player.dodge.particles.forward', EffectOp.set),
  ];

  static const whenPlayers = [
    '2B',
    '9S',
    'A2',
    'not_2b',
    'not_9s',
    'not_a2',
  ];

  static const vanillaIds = [
    1003, 1013, 1020, 1030, 1040, 1050, 1060, 1070,
    1071, 1080, 1090, 1203, 1213, 1220, 1230, 1240,
    1250, 1260, 1261, 1270, 1400, 1420, 1430, 1440,
    1450, 1460, 1470, 1471, 1480, 1600, 1610, 1620,
    1621, 1630, 1640, 1650, 1875, 1876, 1877,
  ];

  static const vanillaObjIds = [
    0x30003, 0x30013, 0x30020, 0x30030, 0x30040, 0x30050, 0x30060,
    0x30070, 0x30071, 0x30075, 0x30076, 0x30080, 0x30090,
    0x30203, 0x30213, 0x30220, 0x30230, 0x30240, 0x30250, 0x30260,
    0x30261, 0x30270,
    0x30400, 0x30420, 0x30430, 0x30440, 0x30450, 0x30460, 0x30470,
    0x30471, 0x30480,
    0x30600, 0x30610, 0x30620, 0x30621, 0x30630, 0x30640, 0x30650,
    0x30655,
  ];

  static const wpmaskObjIds = [
    0x30072,
    0x30081,
    0x30222,
    0x30432,
    0x30602,
  ];

  static int _hash(String value) {
    var hash = 0x811c9dc5;
    for (final unit in value.toLowerCase().codeUnits) {
      hash = (hash ^ unit) * 0x01000193 & 0x7fffffff;
    }
    return hash;
  }

  static WeaponType? typeFromObjId(int objId) {
    for (final type in WeaponType.values) {
      if (objId >= type.objIdStart && objId < type.objIdEnd) return type;
    }
    return null;
  }

  static bool isObjIdAvailable(int objId) {
    if (typeFromObjId(objId) == null) return false;
    if (vanillaObjIds.contains(objId)) return false;
    if (wpmaskObjIds.contains(objId)) return false;
    return true;
  }

  static int? allocateObjId(
    WeaponType type,
    Set<int> alreadyUsed, {
    String? seed,
  }) {
    final span = type.objIdEnd - type.objIdStart;
    final offset = seed == null || seed.isEmpty ? 0 : _hash(seed) % span;

    for (var step = 0; step < span; step++) {
      final id = type.objIdStart + (offset + step) % span;
      if (!isObjIdAvailable(id)) continue;
      if (alreadyUsed.contains(id)) continue;
      return id;
    }
    return null;
  }

  static String toToml(WeaponDraft draft) {
    final buffer = StringBuffer()
      ..writeln('id = ${draft.id}')
      ..writeln('obj_id = 0x${draft.objId.toRadixString(16).toUpperCase()}')
      ..writeln('category = ${draft.type.category}');

    if (draft.skill1Name.isNotEmpty) {
      buffer.writeln('special1 = ${draft.id * 10 + 1}');
    }
    if (draft.skill2Name.isNotEmpty) {
      buffer.writeln('special2 = ${draft.id * 10 + 2}');
    }
    if (draft.storyTextureSuffix != null) {
      buffer.writeln('story_texture_suffix = ${draft.storyTextureSuffix}');
    }

    buffer
      ..writeln()
      ..writeln('[item]')
      ..writeln('name = "${_escape(draft.internalName)}"')
      ..writeln('max_num = 1')
      ..writeln('no_trash = true')
      ..writeln('die_discard = false')
      ..writeln('buy = ${draft.sellsInShop ? draft.buyPrice : -1}')
      ..writeln('sell = ${draft.sellPrice}');

    final icon = draft.iconName;
    if (icon != null && icon.isNotEmpty) {
      buffer
        ..writeln()
        ..writeln('[icon]')
        ..writeln('misctex_name = "${_escape(icon)}"');
    }

    buffer
      ..writeln()
      ..writeln('[effects]')
      ..writeln('est_id_normal = ${draft.effectiveEstIdNormal}')
      ..writeln('est_id_berserk = ${draft.effectiveEstIdBerserk}')
      ..writeln()
      ..writeln('[text.name]')
      ..writeln('_us = "${_escape(draft.displayName)}"')
      ..writeln()
      ..writeln('[text.help]')
      ..write(_block(draft.description));

    if (draft.helpShort.isNotEmpty) {
      buffer
        ..writeln()
        ..writeln('[text.help_short]')
        ..write(_block(draft.helpShort));
    }

    _writeSkill(buffer, 1, draft.skill1Name, draft.skill1Desc);
    _writeSkill(buffer, 2, draft.skill2Name, draft.skill2Desc);

    for (final page in draft.story) {
      buffer
        ..writeln()
        ..writeln('[[story]]')
        ..write(_block(page));
    }

    for (final upgrade in draft.upgrades) {
      buffer
        ..writeln()
        ..writeln('[[upgrades]]')
        ..writeln('lv = ${upgrade.lv}')
        ..writeln('cost = ${upgrade.cost}');

      if (upgrade.materials.isEmpty) continue;

      buffer.writeln('materials = [');
      for (final material in upgrade.materials) {
        buffer.writeln(
          '  { item_id = ${material.itemId}, '
          'quantity = ${material.quantity} },',
        );
      }
      buffer.writeln(']');
    }

    buffer
      ..writeln()
      ..writeln('[params]')
      ..writeln(
        'clone_from_obj_id = '
        '0x${draft.type.cloneFromObjId.toRadixString(16).toUpperCase()}',
      );

    if (draft.levels.isNotEmpty) {
      buffer.writeln('levels = [');
      for (final level in draft.levels) {
        buffer.writeln(
          '  { min_damage = ${level.minDamage}, '
          'max_damage = ${level.maxDamage} },',
        );
      }
      buffer.writeln(']');
    }

    if (draft.traitId != null) {
      buffer.writeln('trait_id = ${draft.traitId}');
    }

    if (draft.sellsInShop) {
      final states = draft.shopStates.toList()..sort();
      for (final state in states) {
        buffer
          ..writeln()
          ..writeln('[[shops]]')
          ..writeln('shop_name = "${_escape(draft.shopName!)}"')
          ..writeln('state = $state');
      }
    }

    for (final effect in draft.effects) {
      buffer
        ..writeln()
        ..writeln('[effect.${effect.name}]')
        ..writeln('path = "${effect.path.path}"')
        ..writeln('op = "${effect.path.op.name}"');

      if (effect.path.perLevel && effect.levels.isNotEmpty) {
        final keys = effect.levels.keys.toList()..sort();
        final pairs = keys
            .map((lv) => '$lv = ${_number(effect.levels[lv]!)}')
            .join(', ');
        buffer.writeln('levels = { $pairs }');
      } else {
        buffer.writeln('value = ${_number(effect.value)}');
      }

      if (effect.whenPlayer != null) {
        buffer.writeln('when = { player = "${effect.whenPlayer}" }');
      }
    }

    return buffer.toString();
  }

  static void _writeSkill(
    StringBuffer buffer,
    int slot,
    String name,
    String description,
  ) {
    if (name.isEmpty) return;

    buffer
      ..writeln()
      ..writeln('[text.skill_${slot}_name]')
      ..writeln('_us = "${_escape(name)}"');

    if (description.isEmpty) return;

    buffer
      ..writeln()
      ..writeln('[text.skill_${slot}_desc]')
      ..write(_block(description));
  }

  static String _number(double value) =>
      value == value.roundToDouble() ? value.toStringAsFixed(1) : '$value';

  static String _block(String value) {
    final lines = const LineSplitter().convert(value);
    final body = lines.map((line) => line.trimRight()).join('\n');
    if (body.contains(r'\') || body.contains("'''")) {
      return '_us = "${_escape(body).replaceAll('\n', r'\n')}"\n';
    }
    return "_us = '''\n$body'''\n";
  }

  static String iconTextureNameFor(int id) => 'wpi_$id';

  static String storyTextureNameFor(int suffix) => 'ws_$suffix';

  static String fileNameFor(WeaponDraft draft) => 'weapon_${draft.id}.toml';

  static String pathFor(String gameDir, String modId, WeaponDraft draft) {
    return p.join(
      ModsService.modsDir(gameDir),
      modId,
      'entities',
      fileNameFor(draft),
    );
  }

  static Future<Set<int>> takenObjIds(String gameDir) async {
    final root = Directory(ModsService.modsDir(gameDir));
    if (!await root.exists()) return <int>{};

    final taken = <int>{};
    await for (final entry in root.list(recursive: true, followLinks: false)) {
      if (entry is! File) continue;
      final name = p.basename(entry.path);
      if (!name.startsWith('weapon_') || !name.endsWith('.toml')) continue;

      String content;
      try {
        content = await entry.readAsString();
      } catch (_) {
        continue;
      }
      final objId = _objIdIn(content);
      if (objId != null) taken.add(objId);
    }
    return taken;
  }

  static int? _objIdIn(String content) {
    for (final line in const LineSplitter().convert(content)) {
      final trimmed = line.trim();
      if (!trimmed.startsWith('obj_id')) continue;
      final equals = trimmed.indexOf('=');
      if (equals < 0) continue;
      final raw = trimmed.substring(equals + 1).trim();
      if (raw.toLowerCase().startsWith('0x')) {
        return int.tryParse(raw.substring(2), radix: 16);
      }
      return int.tryParse(raw);
    }
    return null;
  }

  static Future<int?> nextFreeId(String gameDir, {String? seed}) async {
    final taken = await takenIds(gameDir);
    return allocateId(taken, seed: seed);
  }

  static int? allocateId(Set<int> alreadyUsed, {String? seed}) {
    const span = lastId - firstId + 1;
    final offset = seed == null || seed.isEmpty ? 0 : _hash(seed) % span;

    for (var step = 0; step < span; step++) {
      final id = firstId + (offset + step) % span;
      if (vanillaIds.contains(id)) continue;
      if (alreadyUsed.contains(id)) continue;
      return id;
    }
    return null;
  }

  static Future<Set<int>> takenIds(String gameDir) async {
    final root = Directory(ModsService.modsDir(gameDir));
    if (!await root.exists()) return <int>{};

    final taken = <int>{};
    await for (final entry in root.list(recursive: true, followLinks: false)) {
      if (entry is! File) continue;
      final name = p.basename(entry.path);
      if (!name.endsWith('.toml')) continue;

      for (final prefix in const ['weapon_', 'item_']) {
        if (!name.startsWith(prefix)) continue;
        final digits = name.substring(prefix.length, name.length - 5);
        final id = int.tryParse(digits);
        if (id != null) taken.add(id);
      }
    }
    return taken;
  }

  static Future<File> write({
    required String gameDir,
    required String modId,
    required WeaponDraft draft,
  }) async {
    final file = File(pathFor(gameDir, modId, draft));
    await file.parent.create(recursive: true);
    await file.writeAsString(toToml(draft));
    return file;
  }

  static String _escape(String value) =>
      value.replaceAll(r'\', r'\\').replaceAll('"', r'\"');
}
