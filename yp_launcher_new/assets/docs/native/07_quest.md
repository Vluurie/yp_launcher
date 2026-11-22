# How to make a quest

A quest needs two files: `entities\quest_<id>.toml` and
`data\quest\q<id>.dat`, the quest's script archive with its
dialogue and the handlers that move the quest on. NAMS has no command
that creates the `.dat`. Without it NAMS still offers the quest, but
nothing moves it past `REQ`. Where both folders sit is in
[How to lay out your mod folder](02_mod_folder_layout.md).

Replace every `<…>` with your own value:

```toml
name = "q<id>"
actor = "<character key>"
category = 0
prompt_on_end = 1
prompt_on_fin = -1
bind = "p<phase group>"

[text.name]
_us = "<quest title>"

[text.clr]
_us = "<text for CLR>"

[text.done]
_us = "<text for DONE>"

[run.1]
_us = "<text for step 1>"

[run.2]
_us = "<text for step 2>"
```

## How does a quest move along?

A **scene state** is a named flag that is either set or not. A quest's
states are scene states named `<name>/<state>`, like `q<id>/REQ`. NAMS
creates them, sets `q<id>/REQ` itself as soon as the quest's conditions
hold (see [When is the quest offered?](#when-is-the-quest-offered)), and
your script sets every state after that. `RUN_N` is **step N** of the
quest.

| State | Created | When it is set |
|---|---|---|
| `REQ` | always | The quest is listed as available. |
| `RUN_N` | for every `[run.N]`; a file without any `[text.*]`, `[run.N]` or `[fail.N]` gets `RUN_1` | The quest is listed as active, with its steps up to N. `RUN_1` also shows the *quest accepted* message. |
| `CLR` | always | The quest is listed as active. |
| `DONE` | always | The quest is listed as completed and shows the message picked by `prompt_on_end`. |
| `END` | with `[text.end]` | The quest is listed as completed. |
| `FIN` | with `[text.fin]` | The quest is listed as completed and shows the message picked by `prompt_on_fin`. |
| `READY` | with `[text.ready]` | The quest is listed as active. |
| `FAIL_N` | for every `[fail.N]` | The quest is listed as active. |
| `BREAK` | always | The quest is left out of the quest list. |

Two more states are named with `_` instead of `/`: `q<id>_Ready`, see
[How do I require items for a step?](#how-do-i-require-items-for-a-step),
and `q<id>_DONE`, which always exists. While `q<id>_DONE` is set, NAMS
does not offer the quest, unless `reset_on_chapter_select` is `true`.

## Which id and name can I use?

The id is the number in the file name. Use a number from 1 to 999 that
no vanilla quest and no other mod uses. `name` must be `q` followed by
the id written with three digits: NAMS offers the quest by its id, and
the state it sets then is `q`, those three digits and `/REQ`. A quest
with any other name is never offered. The quest list keeps only the
first four characters of `name`.

Vanilla quests use these ids: 20, 31, 32, 40, 70–72, 85, 90–92, 95,
100–104, 110, 120–123, 130, 131, 140, 150, 160, 162, 170, 171, 180,
181, 210, 220–222, 240, 250, 290–292, 300, 330, 340, 410, 440, 470,
500, 520, 532, 540, 550, 560–562, 590, 640, 650–652, 660, 680, 720,
770, 800–802, 900.

## When is the quest offered?

NAMS keeps checking every quest and sets `q<id>/REQ` when all of these
hold:

1. One `p` token of `bind` matches the phase group the player is in.
2. Every other token of `bind` holds.
3. Every `[activation]` condition holds.
4. None of the quest's states from the table above is set, and
   `q<id>_DONE` is not set (unless `reset_on_chapter_select` is
   `true`).

`bind` is a comma-separated list of tokens:

- `p<hex>` - a phase group: `p` followed by its number in hex, the way
  [phases](25_phases.md) are named. The quest is offered in any of the
  listed phase groups. **Without a `p` token the quest is never offered.**
- any other token - a scene state that must be set, or, with `!` in
  front, one that must not be set.

Every token that starts with `p` or `P` counts as a phase group, so a
scene state that starts with one of them belongs in `[activation]`.

```toml
bind = "p<phase group>,p<phase group>,!<scene state>"

[activation]
required_states = ["<scene state>"]
forbidden_states = ["<scene state>"]
```

| Field | Condition |
|---|---|
| `required_states` | Every listed scene state is set. |
| `forbidden_states` | None of the listed scene states is set. A quest that is already offered stays listed. |
| `required_quests` | Tested exactly like `required_states`, so write full scene states such as `<quest name>/DONE`, not bare quest names. |

Without `[activation]`, `bind` alone decides.

## Which texts can I add?

Each text block maps a language code from
[How to translate your mod](12_translate.md) to a text. A language
without its own text gets the `_us` text, then the `""` text.

| Block | Text of |
|---|---|
| `[text.name]` | the quest's title |
| `[text.clr]`, `[text.done]`, `[text.end]`, `[text.fin]`, `[text.ready]` | the `CLR`, `DONE`, `END`, `FIN` and `READY` state |
| `[run.N]` | step N (`RUN_N`) |
| `[fail.N]` | failed step N (`FAIL_N`) |

`N` is a whole number and each one is used once; gaps are allowed
(`[run.1]`, `[run.5]`). `END`, `FIN`, `READY`, `RUN_N` and `FAIL_N`
exist only with their block, so give every step your script sets its
`[run.N]`.

## How do I require items for a step?

```toml
[[stage_conditions]]
stage = 1
item_id = 530
quantity = <count>
```

`stage` is the step (`RUN_<stage>`), `item_id` an item and `quantity`
how many of it the player must hold. Several blocks may name the same
step; NAMS reports the first five as that step's required items.
`item_id` must be a [vanilla item](resources/vanilla_item_ids.md), like
530 (Rusted Clump) above, or an item from a loaded mod, otherwise the
quest is skipped.

While `q<id>/RUN_<stage>` is set (steps 1 to 20), NAMS sets
`q<id>_Ready` when the player holds every item of that step in the
needed number, and clears it when not. A step without
`[[stage_conditions]]` counts as met. NAMS updates `q<id>_Ready` each
time it is asked whether the quest can be cleared. NAMS also creates
the state `q<id>_RUN_<stage>_CLEAR` for every step that has conditions.

## How do I see it work?

1. `nams.log` shows `[Native] <mod id>: staged <n> entities` and no
   error that names your file, see [where the logs are](17_debugging.md).
2. Go to a phase group named in `bind`: the quest is listed as
   available.
3. When your script sets `q<id>/RUN_1`, the *quest accepted* message
   shows and the quest is listed as active.
4. When it sets `q<id>/DONE`, the `prompt_on_end` message shows and the
   quest is listed as completed.

## Why is my quest not offered?

**The quest never shows up.** Check:

- `bind` has a `p` token for the phase group you are in.
- `name` is `q` and the id in three digits, and the id is 999 or lower.
- No `bind` token, `[activation]` condition or `q<id>_DONE` blocks it,
  and none of the quest's states is set yet.
- `is_excluded` is `false`, and `is_dlc` is `false` or the DLC is
  installed.
- `custom_quests` is not `false` in the `[content]` table of
  [nams.toml](13_nams_toml.md). With `custom_voice_lines = false`,
  quests with a `voice_bank` are skipped too.

**The log names your quest file.**

```
[Native] <file> [quest_bundle]: parse quest_bundle: <reason>
```

A required field (`name`, `actor`, `category`, `prompt_on_end`,
`prompt_on_fin`) is missing or has the wrong type. Nothing of the file
loads.

```
[NAMS] Skipping quest_bundle '<id>' from mod '<mod id>' because the reward '<reward id>' it depends on (from mod '<unstaged>') is not loaded.
```

No loaded mod has that reward. The same line with `voice_bank '<name>'`
names a missing voice bank, with `item '<item id>'` an unknown
`[[stage_conditions]]` item.

```
[NAMS] Two mods try to use the same quest_bundle number '<id>'.
```

Another mod has a quest with this id. The lines below it name the mod
that keeps it.

```
[NAMS] Register failed for quest_bundle from mod '<mod id>': invalid run num '<key>': <reason>
```

`[run.<key>]` is not a whole number. `invalid fail num` is the same for
`[fail.<key>]`.

## Which fields does the file have?

| Field | Default | What it does |
|---|---|---|
| `id` | number in the file name | The quest's id, 1 to 999. |
| `name` | required | `q` and the id in three digits. All states are named after it. |
| `actor` | required | A character key, see [How do I name a new character?](#how-do-i-name-a-new-character) The quest list keeps the first 18 characters. |
| `category` | required | A number NAMS stores as the quest's category in the quest list. |
| `prompt_on_end` | required | Message when `DONE` is set: `1` quest complete, `2` reported request, `3` repair complete, `-1` none. `0` shows the quest complete message too. |
| `prompt_on_fin` | required | Message when `FIN` is set, same numbers. `0` shows none. |
| `bind` | `""` | Where the quest is offered, see [When is the quest offered?](#when-is-the-quest-offered) |
| `is_excluded` | `false` | `true` leaves the quest out of the quest list and the quest statistics. |
| `is_dlc` | `false` | `true` leaves it out of both unless the DLC is installed. |
| `is_main_story` | `false` | Stored as the quest's main-story flag in the quest list. |
| `owner`, `comment`, `client` | `""` | Free text NAMS stores with the quest. |
| `preserve_state` | `false` | `true` adds every state of the quest to the scene-state filter lists for route A, route B and Chapter Select. |
| `reset_on_chapter_select` | `false` | `true` lets NAMS offer the quest even while `q<id>_DONE` is set. With `preserve_state` it also keeps the quest's states off the Chapter Select list. |
| `chapter` | `""` | Name of a chapter. From this chapter on, the quest counts in each chapter's quest count. Empty: NAMS takes the chapter from `bind`. |
| `route` | `""` | Route the quest counts under in those counts: `a` (or `route_a`), `b` (or `route_b`), `both` (or `ab`, `a+b`). |
| `reward_id` | none | Id of a [reward](09_reward.md). The quest is skipped when no loaded mod has it. Nothing pays the reward out on its own. |
| `voice_bank` | none | Name of a [voice bank](10_voice_lines.md). The quest is skipped when no loaded mod has it. |
| `[activation]`, `[text.*]`, `[run.N]`, `[fail.N]`, `[[stage_conditions]]` | none | See the sections above. |

## How do I name a new character?

`entities\char_name_<key>.toml` adds names for the character key
`<key>`, the part of the file name after `char_name_`. Its `[display]`
table holds one name per language code; NAMS adds the key with these
names to the character names, and a quest points at the key with
`actor` or `client`. The key must be unique across all mods.

```toml
[display]
_us = "<English name>"
"" = "<Japanese name>"
```

The language codes are those of
[How to translate your mod](12_translate.md); `_ct` also works for
Chinese (Traditional) here, and other codes are skipped. Its error
lines are listed there as well.

## How do I start my quest from a vanilla character?

To hook your quest into a vanilla script, change that script with a
Ruby patch, see [Patching vanilla scripts](24_ruby_patches.md).

## How do I count things?

A scene state is only set or not set. If your quest has to count kills
or collected items, or remember several things at once, see
[Counters and switches](23_scenario_variables.md).
