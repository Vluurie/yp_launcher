# How to make a graphic adjust area

A **graphic adjust area** is a box in the world. While the player stands
inside it, the room switches to the look you picked for it: colors, fog,
lights, shadows and depth of field. Each look of a room has a number.

The easiest way to make one is the in-game editor. Open the overlay, go
to the **MAP MANAGER** and press **GRAPHIC ADJUST** in its toolbar.

## Using the editor

1. Walk to the spot you want, then press **NEW AREA**.
2. Give it a name and a size, then confirm with **CREATE**. The name
   takes 1 to 31 letters, digits, `_` and `-`. The area starts with the
   look that is active right now. The world reloads once and your area
   appears in the list, already selected.
3. Change the look with the sliders in the **GAD Manager**, and pick
   which look the area uses with the **Filter ID**, **Fog ID** and
   **Grading ID** steppers.
4. Drag the corner handles to shape the box, or type the numbers.
5. Press **SAVE AS MOD**.

The editor writes into the mod folder named in the **Mod ID** field,
`nams\mods\gad_editor\` unless you type another name there. **NEW AREA**
writes too, not only **SAVE AS MOD**. `graphics.toml` and the look file
go into the mod's `entities\` folder, and a new `mod.toml` into the mod
folder itself, replacing the one that is there. You can hand the folder
around like any other mod.

## What the editor writes

`graphics.toml` holds one `[[graphic_adjust]]` block per area, with the
values the area has in the editor:

| Line | What the editor writes |
|---|---|
| `room`, `name` | the room the area is in, and its name |
| `filter_id`, `fog_id`, `grading_id` | the area's ids; **NEW AREA** takes them from the look active at that moment |
| `priority` | **NEW AREA**: one above the highest priority in the room, at most `0xff` |
| `transition_frame` | **NEW AREA**: `1` |
| `sky_id` | **NEW AREA**: the skybox of the look active at that moment; **SAVE AS MOD** writes it empty |
| `gad` | `<name>.gad.xml` |
| `[graphic_adjust.volume]` | the box, see below |

**NEW AREA** always writes the look file `<name>.gad.xml` next to
`graphics.toml`. **SAVE AS MOD** writes it, and the `gad` line, only
with **Save current look as well** ticked. That file holds the actual
colors, fog, lights, shadows and depth of field. Without the `gad` line
the area uses the room's own look with its `filter_id`.

## `[[graphic_adjust]]`

| Field | Default | What it does |
|---|---|---|
| `room` | needed | The room the area belongs to, like `0x110` for the Resistance Camp. The **MAP MANAGER** shows the current room as `Grid (Current: 0x110)`. |
| `all_rooms` | `false` | `true` adds the area to every room; `room` can then be left out. |
| `name` | needed | The `FilterName`, 1 to 31 characters. Phases refer to areas by this name, and it has to be unique inside the room. |
| `filter_id` | needed | Which look the area uses, up to `0xff`. |
| `fog_id`, `grading_id` | `filter_id` | Which fog and which color grading the area uses. |
| `sky_id` | none | The skybox: `bga000`, `bga001` or `bga010`. |
| `priority` | `20` | When areas of different rooms are active at once, the one with the higher number wins. |
| `transition_frame` | `300` | How many frames the blend into the look takes. |
| `gad` | none | The look file, next to the TOML. The look number inside it must equal `filter_id`. |
| `volume` | none | The box, see below. |
| `fill_room` | `false` | `true` adds the area without a box; `volume` is then ignored. |
| `always_on` | `false` | `true` makes the area active without the player inside it, while its room is loaded. |
| `phase`, `subphases` | always | Limit the area to a story step, see below. |
| `core` | `false` | See *Changing the textures of a look*. |

## `[graphic_adjust.volume]`

`points` are the four ground corners as `[x, z]` in world coordinates,
not offsets. Order them so they go around the box the same way the
editor writes them; the editor turns a list that runs the other way
around before it saves. Reversed order makes the box test inside out,
and the area never turns on.

`top` and `bottom` are the height range. The player counts as inside
when their height is at or above `bottom` and below `top`.

`trans` is the center of the box. The editor fills it in for you.

A `volume` needs all four lines, and exactly four `points`.

## Turning it on only in certain story steps

By default the area is active whenever the player is inside it. `phase`
and `subphases` limit that to a story step: `phase` is the story phase as
a number, `0x200` for `p200`, and `subphases` narrows it to those
subphases. `subphases` needs a `phase`.

```toml
phase     = 0x200
subphases = ["170_00_A_Resistance_Camp_2B"]
```

Outside them the area is skipped when the active look is picked. A phase
can also switch an area on and off by its name, see
[How to add a phase or a subphase](25_phases.md).

## Changing the textures of a look

A row without `volume` and without `fill_room` adds no area. It gives
look `filter_id` other textures wherever that look shows in the row's
room, also when one of the room's own areas turns it on. Such a row
needs `fog_id` or `grading_id`.

`grading_id` picks the color grading, `fog_id` the fog textures. One you
leave out stays as it is. The textures come from the room's own set
first, then from any other loaded room's. The editor shows the ids each
area of a loaded room uses.

Some looks are set by a script instead of an area. They read their
textures from the core set, and `core = true` changes those; leave
`room` out. A `core` row takes no `all_rooms`, `volume` or `fill_room`.

`phase` and `subphases` work here too. `gad`, `sky_id` and `always_on`
need an area.

## Changing an area later

Saving `graphics.toml` takes effect right away for rows without an area,
and for `phase`, `subphases` and `always_on` of an area. Everything else
of an area, its look file included, is read when its room loads; the
editor reloads the world for you. Replacing only the `.gad.xml` is not
noticed: save the TOML as well.

## When it does not show up

**The area never turns on.** The player has to be inside the box. Turn
on **Show volumes** in the editor to see it, and use **Focus camera on
area** to fly the camera to it.

If another area in the same room covers the same spot, the first one in
the room wins; the editor lists a room's areas in that order. Move your
box, shrink the other one, or switch the other one off with **Toggle
on/off** to check.

**The look file is not used.** When the room loads it, the log says:

```
[GadInject] fed gad '<name>' (No 0x<filter_id>) into room 0x<room>
```

A look file whose number differs from `filter_id` is refused:

```
[Native] <toml file> [graphic_adjust]: graphic_adjust '<name>': Work/No is <number> but filter_id is <filter_id> — they must match
```

**Nothing from `graphics.toml` loads.** A refused block is named in the
log, with the reason:

```
[NAMS] Register failed for graphic_adjust from mod '<mod id>': <reason>
```

`lighting_filters = false` under `[content]` in `nams.toml` skips every
block, see [How to configure NAMS](13_nams_toml.md). Where the log is:
[How to debug](17_debugging.md).
