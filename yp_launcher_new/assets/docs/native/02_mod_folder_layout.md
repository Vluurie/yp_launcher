# How to lay out your mod folder

```
nams\mods\my_first_mod\
├── mod.toml           ← optional, but write one
├── entities\          ← your entity files
│   └── item_9999.toml
├── data\              ← optional: replacement files
├── textures\          ← optional: replacement textures
├── cutscenes\         ← optional: bundled cutscene videos
├── phase_data\        ← optional: files for one story phase group
├── effects\           ← optional: effect files
└── ui\                ← optional: UI files
```

- **`mod.toml`** - the mod's id, name and other details. Without it,
  the folder name becomes the mod id, cleaned up the same way as `id`
  below.
- **`entities\`** - the content: `.toml` files, and `.json` files that
  name their kind in a top-level `kind` field. Subfolders allowed; NAMS
  scans recursively.
- **`data\`** - replacement files. The folders inside mirror the base
  data, and a file at `data\<path>` replaces the base file at `<path>`.
  `.cpk` files placed directly in `data\` are mounted as well. A
  `data\pl\` folder (player models) is held back until an outfit of
  the mod is worn; the other folders in `data\` are used from the
  start. See
  [Load order, default outfits, and reload behavior](20_runtime_configuration.md).
- **`textures\`** - `.dds` files named by texture hash, see
  [How to replace textures](33_texture_injection.md).
- **`cutscenes\`** - `.usm` movies under
  `cutscenes\<name>\movie\<file>.usm`. Each one replaces the movie with
  the same file name.
- **`phase_data\`** - one folder per story phase group, named `p` and
  the group's three-digit hex number. Its files are used only while
  that phase group is active, see [phases](25_phases.md).
- **`effects\`** - `.est` effect files. Each one is registered as
  `<mod id>:<file name>`.
- **`ui\`** - `.uid` files, each registered under its file name, see
  [UI overrides](45_ui_override.md).

## Filename → entity kind

NAMS reads the start of an entity's filename to know what it is, and
the rest as its id:

| Filename | Entity kind | Page |
|---|---|---|
| `item_<id>.toml` | item | [item](03_item.md) |
| `weapon_<id>.toml` | weapon | [weapon](04_weapon.md) |
| `weapon_patch_<id>.toml` | existing weapon patch | [weapon patch](30_weapon_patch.md) |
| `outfit_<id>.toml` | outfit | [outfit](05_outfit.md) |
| `accessory_<id>.toml` | accessory | [accessory](06_accessory.md) |
| `quest_<id>.toml` | quest | [quest](07_quest.md) |
| `mail_<id>.toml` | mail | [mail](08_mail.md) |
| `reward_<id>.toml` | reward | [reward](09_reward.md) |
| `voice_bank_<name>.toml` | voice bank | [voice lines](10_voice_lines.md) |
| `char_name_<name>.toml` | character name | [quest](07_quest.md) |
| `map_<name>.toml` | map | [custom maps](32_custom_maps.md) |
| `entity_<name>.toml` | custom entity | [custom entities](34_custom_entities.md) |
| `novel_<name>.toml` | novel | [novel](40_novel.md) |
| `archive_<name>.toml` | archive | [archive](41_archive.md) |

The part after the prefix becomes the entity's `id` when it is a whole
number, and its `name` otherwise. An `id = …` or `name = …` written
inside the file wins over the filename. The rest may contain `_`:
NAMS takes the longest prefix that fits, so `weapon_patch_<id>.toml`
is a weapon patch, never a weapon. For the kinds named by `<name>`,
the part after the prefix must not be only digits.

Any other entity file is a list file: every top-level table names its
kind, and each `[[kind]]` block is one entry. Which kinds are written
this way is in the [cheat sheet](18_cheat_sheet.md). A list file's
name must not start with one of the prefixes above followed by `_`:
such a file is read as that kind instead, unless it contains a
`[<prefix>_bundle]` table, the kind name of that prefix.

## `mod.toml` fields

| Field | Required | Notes |
|---|---|---|
| `id` | yes | Unique mod id: lowercase letters, digits and `_`. Uppercase letters are lowercased and any other character becomes `_`, with a warning in the log. An empty `id` rejects the mod. |
| `display_name` | no | The mod's name in the launcher's **Mod Manager**. |
| `author` | no | Shown in the launcher's mod details, and named in the crash message when a crash is traced to the mod's plugin. |
| `version` | no | Shown in the launcher's mod details. |
| `requires_plugins` | no | Plugins that must be loaded, e.g. `["yorha_protocol"]`. If one is not loaded, the mod's entities and its `data\`, `phase_data\`, `effects\` and `ui\` folders are skipped. |
| `requires` | no | Other mod ids. NAMS loads the mod either way; the launcher's mod details mark the ids that are not installed as missing. |
| `replaces` | no | Mod ids this mod supersedes. NAMS does not act on it. |
| `plugin` | no | Path of a plugin DLL inside the mod folder. NAMS loads it at startup. |
| `nams_min_version` | no | Lowest NAMS version, as `major.minor.patch`. Checked only for `plugin`: an older NAMS does not load the plugin and shows a message box. A value NAMS cannot read counts as met. |
| `issues_url` | no | Named in the crash message when a crash is traced to the mod's plugin. |
| `support_url` | no | Used in that message when `issues_url` is not set. |

Unknown fields make the file invalid: the log shows
`[Native] Invalid <path>: <message>` and the mod is skipped.
