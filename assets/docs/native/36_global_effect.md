# How to play an effect on the player

`[[global_effect]]` plays an effect on the player for as long as a
condition holds, and fades it out when the condition ends. The effect
comes from a room's own effect container, the archive
`effect\r<room>.eff` that holds the room's numbered effects, or from an
`.est` file in your mod.

## The short version

One block in `entities\effects.toml` of your mod is enough:

```toml
[[global_effect]]
name      = "r200_sst_017"
from_room = 0x200
sst_no    = 17
```

Room `0x200` carries this effect as number 17. With no `room` and no
`phase` it plays on the player everywhere, always. Nothing is copied
into your mod: the bytes are read out of the archive the first time the
effect starts.

## `[[global_effect]]`

| Field | Default | What it does |
|---|---|---|
| `name` | needed | A unique name for the entry, without `:`. It names the effect in the log. |
| `from_room`, `sst_no` | none | An effect a room already carries: the room, and the number the **EFFECT AREAS** editor in the **MAP MANAGER** toolbar shows as `Effect <number>` for that room. |
| `file` | none | An `.est` of your own, as a path from your mod folder, the one with `mod.toml` in it, not from `entities`. The **EFFECT EDITOR** saves an effect into your mod as `effects\<name>.est`. |
| `room` | every room | Only while the player is in this room. |
| `phase` | every phase | Only in this story phase, as a number: `0x200` for `p200`. |
| `subphases` | the whole phase | Only in these subphases. |
| `not_subphases` | none | Not in these subphases. A name in both lists counts as left out. |

Use `file`, or `from_room` with `sst_no`, never both. `subphases` and
`not_subphases` need a `phase`.

## What happens at a change

When the room or the phase stops matching, the effect is faded out:
parts whose file sets fade times keep them, the others end at once. An
entry that starts matching begins right away, no reload needed. An
effect whose file ends on its own is started again as long as the
condition holds.

Saving the TOML restarts every global effect, so a change shows at
once. Replacing only the `.est` is not noticed: save the TOML as well.

## When nothing appears

A failed start says why:

```
[GlobalEffect] '<name>' did not start: <reason>
```

The usual causes:

- With `from_room`: the room has no effect with that number. The reason
  lists the files it does have:
  `global_effect '<name>': effect/r<room>.eff has no <number>.sst, it carries <files>`.
- With `file`: the file is read when the TOML loads, so a wrong path
  shows as
  `[Native] <toml file> [global_effect]: global_effect '<name>': <path>: <reason>`.
  The path starts at your mod folder.
- The phase does not match. Take `phase` out for a moment to check
  whether the effect works at all.
- `global_effects = false` under `[content]` in `nams.toml` skips every
  entry, see [How to configure NAMS](13_nams_toml.md).
