# Load order, defaults, dependencies, and reload behavior

NAMS uses several configuration files under `nams\`. It creates
`mod_load_order.toml` and `disabled_mods.toml` with an empty list when
they are missing.

## `mod_load_order.toml`

```toml
load_order = [
  "mods/my_first_mod",
  "mods/<folder>",
]
```

Paths are relative to `nams\`, and an entry covers the folder it names
and everything inside it. When two enabled mods replace the same file
under `data\`, the one listed later wins. A mod that is not listed
ranks with the last entry; between those, the folder path that sorts
first wins.

This file controls replacement-file priority. It does not decide which
mod owns a duplicated entity id. Mod folders are scanned in sorted
folder-path order, and the first mod to claim an id keeps it.

## `default_mods.toml`

Default outfit mounting is experimental and only runs when this is set
in `nams.toml`:

```toml
experimental_default_outfits = true
```

The launcher writes it as `true`. A `nams.toml` that NAMS creates
leaves it out, which counts as `false`. With the feature on, NAMS
creates a commented `default_mods.toml` when it is missing.

The default file supports:

```toml
outfit_bare = [
  "mods/<folder>",
]

outfit_config = [
  "mods/<folder>",
]

outfit_animation = [
  "mods/<folder>",
]
```

- `outfit_bare` is a player-model file replacement without outfit
  configuration.
- `outfit_config` is a mod with outfit configuration. An entry can
  also be written as `{ path = "mods/<folder>", outfit_id = <outfit id> }`
  to pick which of the mod's outfits is worn; without `outfit_id` it is
  the one worn without an item.
- `outfit_animation` stays active underneath outfit changes and does
  not claim the character's outfit slot.

Entries can name a complete mod folder or a more specific path. Paths
are relative to `nams\`, backslashes are turned into forward slashes,
and an entry covers everything inside the path it names.

## `disabled_mods.toml`

Disabled paths are not mounted or scanned at boot, see
[How to turn a mod off without deleting it](14_disabled_mods.md).

NAMS re-reads these three files when you save them, but mods already
loaded stay loaded and mounted mods keep their order. Restart after
changing enabled mods, `mod.toml` files or the load order.

## `mod.toml` dependencies

What `requires_plugins`, `requires`, `replaces` and `nams_min_version`
do is in [how to lay out your mod folder](02_mod_folder_layout.md).

## Native `data\` layers

Each mod's `data\` folder is mounted as a layer: a folder whose files
replace the base files while the layer is enabled.

- A mod without `data\pl` is mounted enabled.
- A mod with `data\pl` starts with its `pl` folder held back; its other
  folders in `data\` are used from the start.
- Equipping an outfit item of that mod enables the whole layer and
  reloads the player model.
- Using the equipped item again selects outfit `0`, disables the layer,
  and reloads the base player model.
- With `experimental_default_outfits` on, a mod listed in `outfit_bare`
  or `outfit_animation` is mounted enabled from the start.
- `.cpk` files directly inside `data\` are registered separately.

## Reload behavior

| Change | What NAMS does | Reliable action |
|---|---|---|
| `nams.toml` | Re-read on save. Fields read at startup still need a restart. | Follow the field's note in [How to configure NAMS](13_nams_toml.md). |
| `disabled_mods.toml` | List re-read on save. Mods already loaded stay loaded. | Restart. |
| `mod_load_order.toml` | List re-read on save. Mounted mods keep their order. | Restart. |
| `default_mods.toml` | List re-read on save while `experimental_default_outfits` was on at startup. | Restart for the startup mounts. |
| `entities\*.toml` | Read again on save for the kinds marked in the [cheat sheet](18_cheat_sheet.md). | Restart for the other kinds. |
| `entities\*.json` | Not read again. | Restart. |
| `data\` files | Mounted at startup. An outfit change reloads the player model. | Restart unless testing an outfit swap. |
