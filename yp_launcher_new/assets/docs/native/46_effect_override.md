# How to change one of the effects

Effects are kept in **banks**, and each effect in a bank has a number.
`[[effect_override]]` makes one number of one bank play an `.est` file
from your mod instead of its own.

## The short version

The easiest way is the **EFFECT EDITOR** in the overlay. Pick the effect
in its **PLAYING NOW** list and edit it where it runs, then pick your mod
and press **SAVE**. The editor writes two files into the mod, named
after the bank and the number with three digits:

- `entities\effect_override_<bank>_<number>.toml`
- `effects\override_<bank>_<number>.est`

For effect 79 of the `core` bank, the one the editor's **Lens flare**
template starts from, in a mod with the id `my_mod`, the TOML reads:

```toml
[[effect_override]]
name = "my_mod.core_079"
bank = "core"
no   = 79
file = "effects/override_core_079.est"
```

## `[[effect_override]]`

| Field | Default | What it does |
|---|---|---|
| `name` | needed | A unique name for the row. |
| `bank` | needed | The bank the effect belongs to, see below. |
| `no` | needed | The effect's number in the bank. `0` to `999` are the bank's `.est` files, `1000` to `1999` its `.sst` files: 1000 plus the file's number. |
| `file` | needed | The `.est`, as a path from your mod folder, the one with `mod.toml` in it. |
| `room` | every room | Use the file only while the player is in this room. |
| `phase` | every phase | Use the file only in this story phase, as a number: `0x200` for `p200`. |
| `subphases` | the whole phase | Only in these subphases. Needs a `phase`. |
| `not_subphases` | none | Not in these subphases. Needs a `phase`. |

Added to a row, these lines use the file through phase `p200`, except
in that one subphase:

```toml
phase         = 0x200
not_subphases = ["170_00_A_Resistance_Camp_2B"]
```

`bank` names, in any case:

| `bank` | The bank of |
| --- | --- |
| `core` | the core effects |
| `r200` | a room: `r` and its three hex digits |
| `p200` | a story phase: `p` and its three hex digits |
| `g11420` | a map grid: `g` and its number |
| `pl0200` | an object: its name |
| `ev` and four hex digits | an event; a room or phase name can come first |

## When it applies

A bank that loads while the row matches plays your file. When the room
or the story moves in or out of the row's range, loaded banks switch
over on the next frame. An effect that is already playing keeps the file
it started with, and plays yours from its next start on. If two rows
match the same number of the same bank, the one registered first counts.

Saving the TOML switches loaded banks to the new file on the next frame.
Replacing only the `.est` is not noticed: save the TOML as well.

## When your file does not play

- A wrong row is refused when the TOML loads, with the reason:
  `[Native] <toml file> [effect_override]: <reason>`, such as
  `effect_override '<name>': '<bank>' is no bank name`.
- The effect is still running from before. Start it again.
- `room` or `phase` do not match where you stand.
- The bank has no effect with that number, or that effect cannot be
  swapped. The log does not report these two cases.
