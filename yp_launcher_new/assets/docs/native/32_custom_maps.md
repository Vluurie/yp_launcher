# How to make a custom map

A **custom map** is a room NAMS loads under its own grid ids, in its own
phase, without touching any vanilla file.

A map is made of **grids**. A grid is one archive pair: the `.dat` holds
the grid's collision and the layout of the objects standing in it, the
`.dtt` holds the data placed in video memory. A grid id is `g` plus
exactly 5 hex digits, like `g5f200`. A **phase** is named `p` plus 3 hex
digits, and a **subphase** is a step inside it, see
[How to add a phase or a subphase](25_phases.md).

When your phase loads, NAMS hands the phase loader your grid list and
your spawn point. The loader then switches off the streaming overworld,
loads your grids and places the player at `player_pos`.

## What does a finished map mod look like?

```
<your mod>\
  entities\
    maps.toml           <- the [[map]]
  data\
    wd5\
      g5f200.dat        <- collision + object layout
      g5f200.dtt        <- data for video memory
    Enlighten\
      g5f200.enlMeta    <- lighting data, see below
      g5f200.rss
```

The rest of the mod folder is explained in
[How to lay out your mod folder](02_mod_folder_layout.md).

## How do I write the `[[map]]`?

Replace each `<…>` with your own value.

```toml
[[map]]
name = "<map name>"
grids = ["g5f200"]
phase = "<phase, p plus 3 hex digits>"
subphase = "<subphase name>"
player_pos = [<x>, <y>, <z>, <yaw>]
comment = "<note to yourself>"
```

| Field | Required | What it does |
|---|---|---|
| `name` | yes | Your map's id inside NAMS. Pick anything unique. |
| `grids` | yes | The grids to load, 1 to 8 of them. Each one is `g` plus exactly 5 hex digits. |
| `phase` | yes | The phase that shows this map, `p` plus exactly 3 hex digits. |
| `subphase` | no | Without it the map applies to the whole phase. With it, the map only applies while that exact subphase is the current step. 1 to 31 characters. |
| `player_pos` | yes | Where the player spawns: `[x, y, z, yaw]`, with yaw in degrees. |
| `comment` | no | A note to yourself; NAMS does not use it. |

Every grid your mod ships must come as a **pair**: NAMS refuses the map
when your `data` folder has the `.dat` without its `.dtt`, or the `.dtt`
without its `.dat`. A grid that is not in your mod at all is only a
warning, because another mod may provide it.

## Which grid id can I use?

The first digit of a grid id is its layer, and it picks the folder:
`g5f200` lives in `data\wd5\`. Use a `g5xxxx` id: grids of layer 5 are
loaded for a phase instead of being streamed in by distance.

Pick an id no vanilla map uses. The ids vanilla maps use are listed in
[vanilla grid ids](resources/vanilla_grid_ids.md); `g5f200` is not one
of them.

## Where do the Enlighten files go?

**Enlighten** is the lighting data of a grid: a file pair,
`Enlighten\g5f200.enlMeta` and `Enlighten\g5f200.rss`, named after the
grid id. Put both into your mod's `data\Enlighten\` folder.

NAMS points the lighting loader at the mod that has the `.enlMeta` for
your grid and reads the `.rss` from the same folder, so keep the two side
by side. A file with the same path in the vanilla data is used instead
of yours.

## How do I see that it works?

Open the NAMS log, see
[How to debug when something goes wrong](17_debugging.md).

When your phase loads, the log shows the map:

```
[Maps] phase <phase> subphase '<subphase>': map '<map name>' provides rooms ["g5f200"]
```

When a grid's lighting data is read from your mod, the log shows the
folder it came from:

```
[Enlighten] Enlighten/g5f200.enlMeta served from "<path to your mod>\\data"
```

## What if something goes wrong?

**The map is refused when the mods load.** The log has a line starting
with `[NAMS] Register failed for map from mod '<mod id>':`, followed by
one of these:

| Message | Fix |
|---|---|
| `map '<map name>' needs 1 to 8 grids, has <n>` | List 1 to 8 grids. |
| `map '<map name>': grid '<grid>' must start with 'g'` | Write the grid id with its `g`. |
| `map '<map name>': grid '<grid>' must be 'g' followed by exactly 5 hex digits, like g5f100` | Fix the grid id. |
| `map '<map name>': phase '<phase>' must be 'p' followed by exactly 3 hex digits` | Fix `phase`. |
| `map '<map name>': subphase '<subphase>' must be 1..=31 chars` | Shorten or fill in `subphase`. |
| `map '<map name>': wd5\<grid>.dat is in this mod but <grid>.dtt is missing — the .dtt (textures/lighting) is required` | Add the `.dtt` next to the `.dat`. |
| `map '<map name>': wd5\<grid>.dtt is in this mod but <grid>.dat is missing` | Add the `.dat` next to the `.dtt`. |

**A line starts with `[Native] <file> [map]:`.** The `[[map]]` block
itself is malformed: a required field is missing or has the wrong type.
The rest of the line says what is wrong.

**The log says the grid is not in your mod.**

```
[Maps] map '<map name>': wd5\<grid>.dat/.dtt not in this mod's data folder — the game will only load '<grid>' if vanilla or another mod provides it
```

Check the grid id and that the files sit in `data\wd5\` of the mod that
holds the `[[map]]`.

**There is no `[Maps] phase …` line when the phase loads.** `phase` is
not the phase you load into, or `subphase` does not match the current
subphase exactly. Also check that `custom_maps` is not `false` in
[`nams.toml`](13_nams_toml.md).

**The log says `Two mods try to use the same map number '<map name>'`.**
Another mod uses the same `name`; the mod that loads first keeps it.
Give your map a different `name`.

**There is no `[Enlighten] … served from` line for your grid.** The
`.enlMeta` is not in `data\Enlighten\` of your mod, its name does not
match the grid id, or the vanilla data has a file with the same path.

## What else should I know?

- **One map per file.** A file named `map_<map name>.toml` holds one
  map's fields at the top level, without `[[map]]`; the file name gives
  `name`. See [How to lay out your mod folder](02_mod_folder_layout.md).
- **Two maps for one phase.** A map whose `subphase` is the current
  subphase wins over a map without `subphase`.
- **Changes during play.** Saving the file registers the `[[map]]`
  again; the change applies the next time the phase loads. A map you
  delete from the file stays until the next start.
- **Files only for your phase.** Files in `phase_data\<phase>\` are used
  only while that phase is active, see
  [How to lay out your mod folder](02_mod_folder_layout.md).
- **Switch.** `custom_maps = false` in the `[content]` table of
  [`nams.toml`](13_nams_toml.md) skips every `[[map]]`.

## Related

- [How to add a phase or a subphase](25_phases.md)
- [How to make a graphic adjust area](26_graphic_adjust.md)
- [Vanilla grid and room ids](resources/vanilla_grid_ids.md)
