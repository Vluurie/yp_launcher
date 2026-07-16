/// Pulls the `"path"  "..."` values out of a Steam `libraryfolders.vdf`.
final _vdfPathEntry = RegExp(r'"path"\s+"([^"]+)"');

/// Every library `path` entry in [content], with VDF escaping undone.
List<String> vdfPathEntries(String content) => _vdfPathEntry
    .allMatches(content)
    .map((m) => m.group(1)!.replaceAll(r'\\', r'\').replaceAll(r'\"', '"'))
    .toList();
