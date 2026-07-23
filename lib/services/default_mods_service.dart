import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

/// How the game activates a boot default.
///
/// The game reads one list per kind and treats each differently, so a mod put
/// in the wrong list either loses its config or fights with it. Adding a kind
/// means adding a value here and nothing else: parsing, writing and the config
/// template all iterate over these values.
enum DefaultKind {
  /// Plain file replacement without a config of its own.
  outfitBare(key: 'outfit_bare', exclusiveGroup: 'outfit'),

  /// Ships an outfit config, so the game resolves its mesh rules and effects
  /// rather than only mounting its files.
  outfitConfig(key: 'outfit_config', exclusiveGroup: 'outfit'),

  /// Animation data such as blinking. Claims no outfit slot and stays active
  /// underneath whatever outfit is worn.
  outfitAnimation(key: 'outfit_animation', exclusiveGroup: null);

  const DefaultKind({required this.key, required this.exclusiveGroup});

  /// Name of this kind's list in default_mods.toml.
  final String key;

  /// Kinds sharing a group compete for the same slot, so only one of them may
  /// be the default for a given model. Null means it competes with nothing.
  final String? exclusiveGroup;

  static DefaultKind? fromKey(String key) {
    for (final k in DefaultKind.values) {
      if (k.key == key) return k;
    }
    return null;
  }

  /// Whether this kind competes with `other` for the same slot.
  bool conflictsWith(DefaultKind other) =>
      exclusiveGroup != null && exclusiveGroup == other.exclusiveGroup;
}

/// One configured boot default.
class DefaultEntry {
  final String path;
  final DefaultKind kind;

  /// Which of the mod's outfits to wear. Null means the one worn without an
  /// item, which is the only choice for a mod that ships a single outfit.
  final int? outfitId;

  const DefaultEntry({
    required this.path,
    required this.kind,
    this.outfitId,
  });
}

class DefaultModsService {
  DefaultModsService._();

  static String _filePath(String gameDir) =>
      path.join(gameDir, 'nams', 'default_mods.toml');

  static String get _defaultFile {
    final buffer = StringBuffer()
      ..writeln('# Mods listed here are active from the moment the game starts,')
      ..writeln('# exactly as if their files were placed in NieRAutomata/data.')
      ..writeln('# Restart the game after editing.')
      ..writeln('#')
      ..writeln('# Paths are relative to the nams/ folder. Prefix matching is used.')
      ..writeln();
    for (final kind in DefaultKind.values) {
      buffer.writeln('${kind.key} = []');
    }
    return buffer.toString();
  }

  static String normalize(String relPath) =>
      relPath.replaceAll('\\', '/').trim().replaceAll(RegExp(r'^/+|/+$'), '');

  static String? toRelative(String gameDir, String absolutePath) {
    final rel = path.relative(absolutePath, from: path.join(gameDir, 'nams'));
    if (rel.startsWith('..')) return null;
    return normalize(rel);
  }

  static Future<List<DefaultEntry>> list(String gameDir) =>
      IsolateService.run(_listSync, gameDir);

  /// Puts `relPath` in `kind`'s list, or removes it when `kind` is null.
  ///
  /// `outfitId` names which of the mod's outfits to wear; null means the one
  /// worn without an item.
  static Future<void> setDefault(
    String gameDir,
    String relPath,
    DefaultKind? kind, {
    int? outfitId,
  }) =>
      IsolateService.run(
        _setSync,
        _SetParams(
          gameDir: gameDir,
          relPath: normalize(relPath),
          kind: kind,
          outfitId: outfitId,
        ),
      );

  /// The entry covering `relPath`, if any.
  static DefaultEntry? entryFor(List<DefaultEntry> entries, String relPath) {
    final rel = normalize(relPath);
    for (final e in entries) {
      if (rel == e.path || rel.startsWith('${e.path}/')) return e;
    }
    return null;
  }

  static bool matches(List<DefaultEntry> entries, String relPath) =>
      entryFor(entries, relPath) != null;

  /// Last segment of a stem entry. `mods/foo/pl/pl000d` -> `pl000d`.
  /// Null for whole-mod entries like `mods/foo`.
  static String? stemOf(String entry) {
    final parts = normalize(entry).split('/');
    if (parts.length < 3) return null;
    return parts.last;
  }

  /// Player outfit models are mutually exclusive - only one may be the default
  /// look at a time. Stems ending in `f` are animation data (blinking and the
  /// like), which layer underneath whatever outfit is worn, so they are exempt.
  static bool isOutfitStem(String stem) {
    final s = stem.toLowerCase();
    if (!s.startsWith('pl') || s.length < 3) return false;
    return !s.endsWith('f');
  }

  /// Kind that fits a model of this mod.
  ///
  /// Animation data is recognised by its stem; everything else depends on
  /// whether the mod ships an outfit config, because that config owns the file
  /// layer and must not be given a second one.
  static DefaultKind kindFor(String stem, {required bool hasOutfitConfig}) {
    if (!isOutfitStem(stem)) return DefaultKind.outfitAnimation;
    return hasOutfitConfig ? DefaultKind.outfitConfig : DefaultKind.outfitBare;
  }

  /// Outfit id to write when a mod is made the default, before the user picks
  /// one. A mod whose only outfits need an item has none to fall back to, so
  /// the single candidate is written straight away.
  static int? initialOutfitId(DefaultKind kind, List<OutfitChoice> choices) {
    if (kind != DefaultKind.outfitConfig) return null;
    if (choices.any((c) => !c.needsItem)) return null;
    final itemOutfits = choices.where((c) => c.needsItem).toList();
    if (itemOutfits.length != 1) return null;
    return itemOutfits.first.outfitId;
  }
}

class _SetParams {
  final String gameDir;
  final String relPath;
  final DefaultKind? kind;
  final int? outfitId;
  const _SetParams({
    required this.gameDir,
    required this.relPath,
    required this.kind,
    this.outfitId,
  });
}

List<DefaultEntry> _listSync(String gameDir) {
  final file = File(DefaultModsService._filePath(gameDir));
  if (!file.existsSync()) return const [];
  return _parse(file.readAsStringSync());
}

List<DefaultEntry> _parse(String raw) {
  final parsed = TomlService.parse(raw);
  final out = <DefaultEntry>[];
  for (final kind in DefaultKind.values) {
    final value = parsed[kind.key];
    if (value is! List) continue;
    for (final item in value) {
      String? rawPath;
      int? outfitId;
      if (item is String) {
        rawPath = item;
      } else if (item is Map) {
        final p = item['path'];
        if (p is String) rawPath = p;
        final id = item['outfit_id'];
        if (id is int) outfitId = id;
      }
      if (rawPath == null) continue;
      final p = DefaultModsService.normalize(rawPath);
      if (p.isEmpty) continue;
      out.add(DefaultEntry(path: p, kind: kind, outfitId: outfitId));
    }
  }
  return out;
}

void _setSync(_SetParams p) {
  final file = File(DefaultModsService._filePath(p.gameDir));
  file.parent.createSync(recursive: true);
  final raw = file.existsSync()
      ? file.readAsStringSync()
      : DefaultModsService._defaultFile;

  final entries = _parse(raw)..removeWhere((e) => e.path == p.relPath);

  final kind = p.kind;
  if (kind != null) {
    final stem = DefaultModsService.stemOf(p.relPath);
    if (stem != null) {
      entries.removeWhere(
        (e) =>
            kind.conflictsWith(e.kind) &&
            DefaultModsService.stemOf(e.path) == stem,
      );
    }
    entries.add(
      DefaultEntry(path: p.relPath, kind: kind, outfitId: p.outfitId),
    );
  }

  final lists = <String, dynamic>{
    for (final k in DefaultKind.values)
      k.key: entries
          .where((e) => e.kind == k)
          .map((e) => e.outfitId == null
              ? e.path
              : {'path': e.path, 'outfit_id': e.outfitId})
          .toList(),
  };
  file.writeAsStringSync(TomlService.updateToml(raw, lists));
}
