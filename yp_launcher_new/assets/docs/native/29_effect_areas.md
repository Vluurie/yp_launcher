# How to put effects in a room

A room is one part of the map with a number of its own, such as `0x110`
for the Resistance Camp. You can add an effect file of your own to a
room, or switch on an effect the room already carries. Both can be tied
to a story phase, so a place looks different later on. A second entry
kind switches a room's own effects off.

## How do I add an effect?

Put the entry into `entities\effects.toml` and the effect file into the
mod folder itself, next to `mod.toml` (see
[how to lay out your mod folder](02_mod_folder_layout.md)):

```toml
[[effect_area]]
name = "camp_fire"
room = 0x110
file = "fire.sst"
```

Walk into the Resistance Camp and the effect plays where its file
places it.

## When do I see it?

NAMS writes your entries into the room's effect list while the room
loads. An entry you add or change while you are in the room therefore
shows up **after you leave the room and come back**, not right away.
Only `repeat` and `replaces` of an entry the loaded room already has
change at once.

After the mods are loaded, and after every save of the TOML, the log
says:

```
[EffectInject] <N> effect(s) from mods — re-enter the room to see them
```

When the room loads:

```
[EffectInject] appended <N> binding(s) to <file>, <N> phase bound
[EffectInject] room 0x110: added <N> custom sst file(s) to the eff container
```

In the YorHa Protocol overlay, open the **MAP MANAGER** tab and click
the toolbar button with the tooltip **EFFECT AREAS**. The panel lists
the effects of the room you are in as `Effect <number>`. Entries with
`file` get numbers from 900 up and carry an extra icon.

Saving the TOML while you play reads it again; the log says
`[NAMS] hot register <file>: <N> registered, <N> failed, <N> purged`.
Only `.toml` files in `entities` are read again: after replacing the
effect file itself, save the TOML once more. An entry you take out of
the TOML keeps working until the file is deleted or until the next start.

## Why does nothing appear?

- **No `[EffectInject]` line at all.** The **Effect areas** switch is
  off (`effect_areas` under `[content]` in `nams.toml`, see
  [nams.toml](13_nams_toml.md)); it covers `[[effect_area_mute]]` too
  and takes effect after a restart. Or every entry was refused, see the
  next two points.
- **`[Native] <your toml> [effect_area]: [effect_area][<index>] effect_area '<name>': <path>: <reason>`**
  The effect file could not be used. Either it is not where `file`
  says (the path starts at the mod folder), or it is not an effect
  (`not an effect file (missing EFF magic)`, or a reason ending in
  `not a playable effect and would crash on emit`). A `[Native]` error
  skips every `[[effect_area]]` of that TOML file, not only the broken
  one.
- **`[NAMS] Register failed for effect_area from mod '<mod>': <reason>`**
  (after a save while playing:
  `[NAMS] hot register failed for effect_area '<name>': <reason>`).
  The entry was refused; the reason after the colon says what is wrong.
- **`[EffectInject] <NNN>.sst is not in this room, nothing to bind`**
  The room has no effect with that `sst_no`. Pick one from the
  **EFFECT AREAS** panel.
- **`[EffectInject] '<name>': effect has no MOVE record to place`**
  `pos` cannot move this effect file, so the entry is skipped. Leave
  `pos` out.
- **`[EffectInject] r<room>_EffectArea.bxm not in the container, volumes stay unplaced`**
  The room has no zone list, so `radius` cannot add a zone there.
- **The log looks fine, the effect does not show.** You have not left
  and re-entered the room yet, or the phase does not match: take
  `phase` out for a moment to check whether the effect works at all. A
  misspelled field name is ignored without a log line.

## What goes into an `[[effect_area]]`?

| Field | Needed | What it does |
|---|---|---|
| `name` | yes | A name for the entry, unique across all mods. A second entry with the same name is skipped. The log names the entry by it. |
| `room` | yes | The room number. The **MAP MANAGER** shows the number of the room you are in, such as `Grid (Current: 0x110)` in the Resistance Camp. |
| `file` | this or `sst_no` | Your effect file, an `.sst` or `.est`, as a path from the mod folder (next to `mod.toml`, not inside `entities`). Only this one file is added to the room, nothing else from the container it came from. |
| `sst_no` | this or `file` | The number of an effect the room already has. Switches that effect on, see below. |
| `area_no` | no | The zone the entry waits for, `0` to `63`. `0` is the whole room. Left out, the entry waits for no zone and the phase alone decides. |
| `pos` | no | With `file`: moves the effect to these world coordinates, `[x, y, z]`. With `radius`: the center of the new zone. Left out, the effect stays where its file places it. |
| `radius` | no | Adds a zone of your own, a cylinder around `pos`. Needs `pos` and `area_no`. |
| `height` | no | Height of that cylinder, centered on `pos`. Left out: twice the radius. |
| `replaces` | no | Numbers of the room's effects that are switched off while your entry's phase allows it. A number equal to your own `sst_no` switches your entry off as well. |
| `repeat` | no | An entry plays once. With `repeat` it plays again each time this many frames have passed, `2` to `36000`. Only for an entry without `area_no`. |
| `phase` | no | The story phase the entry plays in. Left out: every phase. |
| `subphases` | no | Only these subphases of `phase`. |
| `not_subphases` | no | Every subphase of `phase` except these. |

`file` and `sst_no` cannot both be set.

## How do I switch on an effect the room already has?

The room's own effects have numbers: the `Effect <number>` entries in
the **EFFECT AREAS** panel. An entry with `sst_no` adds a line for that
number to the room's effect list. The room's own line stays: the room
plays the effect whenever it always did, and your line plays it under
your own condition.

Give your line the `area_no` the room's own line uses, and it waits for
the same zone. **ADD TO MOD** in the panel writes such an entry for the
effect you picked, with its `sst_no` and, if it has one, its `area_no`,
into `nams\mods\<Mod ID>\entities\effects.toml`. **Mod ID** is under
**More options**; left empty it is `effect_editor`.

## How do I give an effect a zone of its own?

`radius` adds a zone to the room: a cylinder around `pos`, `height` tall
and centered on `pos`. It gets your `area_no`, and your entry starts
once the camera is inside it. The effect itself still appears where its
file places it; the zone only decides when it plays. With `sst_no`,
nothing has to be shipped with your mod.

Pick an `area_no` the room does not use yet: a number in use is already
one of the room's zones.

## How does `pos` move my effect file?

An effect file holds placement records, the positions of its parts.
`pos` treats every record that lies 80 or more from the world origin as
a world position: all of them move by the same amount, so the first one
lands on `pos` and the parts keep their layout. Records closer to the
origin are offsets inside the effect and stay. If no record is that far
out, only the first placement record is set to `pos`, and the log says:

```
[EffectInject] '<name>': no world anchor, only the first placement record moves
```

## When does it play?

Without `phase` the entry plays in every phase. A phase is a stage of
the story, given as its group number: `0x100` is route A (see
[how to add a phase](25_phases.md)). `phase` alone covers the whole
phase. `subphases` narrows it to those named points of the phase, and
the entry plays in any one of them; `90_00_AB_Ruined_City` is one of
`0x100`. `not_subphases` is the other way round: the whole phase except
these.

`subphases` and `not_subphases` need a `phase`, because a subphase name
is looked up inside its phase; without one the entry is refused. A name
in both lists is refused too, since the exclusion would win.

## How do I switch a room's own effects off?

`[[effect_area_mute]]` switches off the effects a room carries, except
the numbers you keep.

```toml
[[effect_area_mute]]
name  = "quiet"
room  = 0x110
phase = 0x100
```

This switches every effect of the Resistance Camp's own list off in
phase `0x100`.

| Field | Needed | What it does |
|---|---|---|
| `name` | yes | A name for the rule, unique across all mods. |
| `room` | no | Only this room. Left out: every room. |
| `phase` | no | Only in this phase. Left out: every phase. |
| `subphases` | no | Only these subphases of `phase`. Needs `phase`. |
| `keep` | no | Effect numbers that stay on. Left out: all are switched off. Another rule can still switch a kept number off. |

`[[effect_area]]` entries are never muted, only the room's own effects.
The rule is written into the room's effect list while the room loads,
so a new rule shows after you re-enter the room. After loading, the log
says `[EffectInject] <N> mute rule(s) from mods — re-enter the room to see them`,
and when the room loads
`[EffectInject] muted <N> entr(y/ies) in <file> by phase`.
