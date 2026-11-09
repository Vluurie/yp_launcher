import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mods_service.dart';

class ItemDraft {
  final String displayName;
  final String description;
  final int id;
  final int maxCarry;
  final bool sellable;
  final int sellPrice;
  final bool autoGive;
  final String? misctexName;
  final int? buyPrice;
  final int category;

  const ItemDraft({
    required this.displayName,
    required this.description,
    required this.id,
    this.maxCarry = 1,
    this.sellable = false,
    this.sellPrice = 100,
    this.autoGive = true,
    this.misctexName,
    this.buyPrice,
    this.category = 0,
  });

  String get internalName => internalNameFor(displayName);

  ItemDraft copyWith({
    String? displayName,
    String? description,
    int? id,
    int? maxCarry,
    bool? sellable,
    int? sellPrice,
    bool? autoGive,
    String? misctexName,
    int? buyPrice,
    int? category,
    bool clearMisctex = false,
  }) {
    return ItemDraft(
      displayName: displayName ?? this.displayName,
      description: description ?? this.description,
      id: id ?? this.id,
      maxCarry: maxCarry ?? this.maxCarry,
      sellable: sellable ?? this.sellable,
      sellPrice: sellPrice ?? this.sellPrice,
      autoGive: autoGive ?? this.autoGive,
      misctexName: clearMisctex ? null : (misctexName ?? this.misctexName),
      buyPrice: buyPrice ?? this.buyPrice,
      category: category ?? this.category,
    );
  }

  static String internalNameFor(String displayName) {
    final buffer = StringBuffer('item_');
    var lastWasUnderscore = true;

    for (final rune in displayName.toLowerCase().runes) {
      final char = String.fromCharCode(rune);
      final isWord = (rune >= 0x30 && rune <= 0x39) ||
          (rune >= 0x61 && rune <= 0x7a);

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
    return name == 'item' ? 'item_unnamed' : name;
  }
}

class ItemBuilderService {
  ItemBuilderService._();

  static const firstFreeId = 8000;
  static const _idCeiling = 60000;

  static String toToml(ItemDraft draft) {
    final buffer = StringBuffer()
      ..writeln('name = "${_escape(draft.internalName)}"')
      ..writeln('id_type = 0');
    if (draft.category != 0) {
      buffer.writeln('category = ${draft.category}');
    }

    if (draft.maxCarry != 1) {
      buffer.writeln('max_num = ${draft.maxCarry}');
    }
    if (draft.sellable) {
      buffer.writeln('sell = ${draft.sellPrice}');
    } else {
      buffer.writeln('no_trash = true');
    }

    final buy = draft.buyPrice;
    if (buy != null) {
      buffer.writeln('buy = $buy');
    }

    if (draft.autoGive) {
      buffer
        ..writeln()
        ..writeln('[auto_give]')
        ..writeln('quantity = 1');
    }

    final misctex = draft.misctexName;
    if (misctex != null && misctex.isNotEmpty) {
      buffer
        ..writeln()
        ..writeln('[icon]')
        ..writeln('misctex_name = "${_escape(misctex)}"');
    }

    buffer
      ..writeln()
      ..writeln('[text.name]')
      ..writeln('_us = "${_escape(draft.displayName)}"')
      ..writeln()
      ..writeln('[text.help]')
      ..write(_block(draft.description));

    return buffer.toString();
  }

  static String fileNameFor(ItemDraft draft) => 'item_${draft.id}.toml';

  static String pathFor(String gameDir, String modId, ItemDraft draft) {
    return p.join(
      ModsService.modsDir(gameDir),
      modId,
      'entities',
      fileNameFor(draft),
    );
  }

  static Future<int> nextFreeId(String gameDir) async {
    final taken = await takenIds(gameDir);
    var id = firstFreeId;
    while (taken.contains(id) && id < _idCeiling) {
      id++;
    }
    return id;
  }

  static Future<Set<int>> takenIds(String gameDir) async {
    final root = Directory(ModsService.modsDir(gameDir));
    if (!await root.exists()) return <int>{};

    final taken = <int>{};
    await for (final entry in root.list(recursive: true, followLinks: false)) {
      if (entry is! File) continue;
      final id = idFromFileName(p.basename(entry.path));
      if (id != null) taken.add(id);
    }
    return taken;
  }

  static int? idFromFileName(String fileName) {
    if (!fileName.startsWith('item_') || !fileName.endsWith('.toml')) {
      return null;
    }
    final digits = fileName.substring(5, fileName.length - 5);
    return int.tryParse(digits);
  }

  static Future<File> write({
    required String gameDir,
    required String modId,
    required ItemDraft draft,
  }) async {
    final file = File(pathFor(gameDir, modId, draft));
    await file.parent.create(recursive: true);
    await file.writeAsString(toToml(draft));
    return file;
  }

  static String _escape(String value) =>
      value.replaceAll(r'\', r'\\').replaceAll('"', r'\"');

  static String _block(String value) {
    final lines = const LineSplitter().convert(value);
    final body = lines.map((line) => line.trimRight()).join('\n');
    if (body.contains(r'\') || body.contains("'''")) {
      return '_us = "${_escape(body).replaceAll('\n', r'\n')}"\n';
    }
    return "_us = '''\n$body'''\n";
  }
}
