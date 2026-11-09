# How to add a phase or a subphase

A **phase** is the chapter the story is in, like `p100`. Inside it, a
**subphase** marks where in the story you are, like
`90_00_AB_Ruined_City`. A quest binds to a phase with its `bind`, see
[How to make a quest](07_quest.md).

You can add steps to a vanilla phase, or add a whole new phase. Both go
into a `phase.toml` in your `entities` folder.

The filename must not start with one of the prefixes that name an
entity kind, like `quest_` or `item_`: such a file is read as that kind,
see [How to lay out your mod folder](02_mod_folder_layout.md).

## `[[phase]]`

`name` is `p` plus exactly three hex digits, like `p100`. Anything else
is refused when the mod loads, and the log says
`followed by exactly 3 hex digits`.

`subphases` are the names you add. They go after the ones the phase
already has, so existing steps keep their order, and a name the phase
already has is not added again. Each name can be up to 31 characters.
An entry is a name in quotes, or a table with `name` and the fields
listed under "What a subphase entry can carry".

`comment` is a note to yourself.

Only one mod can list a phase. A second mod with the same `name` is
skipped, and the log says
`Two mods try to use the same phase number 'p100'`. Inside one mod,
list each phase once; a second block logs
`lists the same phase number 'p100' twice` and is skipped.

## New phases need a name of `p100` or higher

NAMS refuses a name below `p100`, because objects in such a phase lose
their shadows. The log says
`which is below 0x100 and would leave objects without shadows`.

## What a subphase entry can carry

Write the subphase as a table instead of a bare name to give it more:

| Field | What it does |
|---|---|
| `name` | The subphase name, up to 31 characters. |
| `player_pos` | `[x, y, z, yaw]`, yaw in degrees: where the player is placed in this subphase. |
| `save_name` | The place name a save slot shows for a save made in this subphase, one text per language. |
| `save_image` | The number of the picture that save slot shows. Needs `save_name`. |
| `chapter` | An entry for this subphase in Chapter Select, see below. |

`player_pos` is logged as `[Phases] phase p… subphase '…': player_pos`
when NAMS uses it. A [custom map](32_custom_maps.md) that covers the same
subphase wins over it.

The language keys of `save_name` are `""` for Japanese, `_us`, `_fr`,
`_it`, `_de`, `_es`, `_kr`, `_cn` and `_tw`. Korean and both Chinese
languages show the `_us` text when they have none of their own.

`save_image` is a number from 0 to 999. These are taken by the base
pictures and refused: 0, 10–16, 20, 30–33, 40, 50, 60, 70–72, 90, 100,
110, 120, 121 and 998.

## Starting a subphase from Chapter Select

A subphase with a `chapter` table gets its own entry in Chapter Select,
under **Chapter 18**, named **Custom**. NAMS ships the picture of that
chapter.

| Field | What it does |
|---|---|
| `number` | The entry's place in the chapter, 1 to 99. Each number can be used by one subphase only. |
| `characters` | Who the player can start as: one or two of `2B`, `9S` and `A2`. |
| `title` | The name in the list, one text per language. Required. |
| `help` | The entry's help text, one text per language. |
| `requires` | Scene states. The entry is listed only while every one of them is set. |

`title` and `help` take the same language keys as `save_name`.

A second subphase with a taken `number` is refused with
`chapter number … is already taken`.

## Switching a graphic adjust area per subphase

A phase can decide when a graphic adjust area is allowed to be on. You
name the area by its `FilterName`, so this works for vanilla areas and
for ones you made yourself.

```toml
[[phase]]
name = "p100"

[[phase.filters]]
subphase = "90_00_AB_Ruined_City"
room = 0x110
filter = "camp"
reached = true
```

`subphase` is the step the rule applies to. `room` is the room the area
belongs to, like `0x110` for the Resistance Camp. `filter` is the name
of the area.

By default the rule holds only while that subphase is the current step.
With `reached = true` it holds from that step onwards, which is usually
what you want for a change that should stay. While the rule holds the
area is allowed on, otherwise it is suspended.

A rule only touches the area it names, so everything else in the room
keeps working as before. You can also list only `filters` and no
`subphases`, which adds rules to a vanilla phase without changing it.

## When it does not load

The **New maps and story phases** switch (`custom_maps` in `nams.toml`;
in the launcher, tab **NAMS**, card **CONTENT FEATURES**, group **Story
& world**) must be on; turning it on needs a restart.

A refused block shows up in the log as
`Register failed for phase from mod '…'`, followed by the reason.

## Related

- [How to make a graphic adjust area](26_graphic_adjust.md)
- [How to make a quest](07_quest.md)
