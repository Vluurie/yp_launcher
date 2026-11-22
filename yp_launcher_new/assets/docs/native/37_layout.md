# How to remove objects from the world

Every grid places its objects from the layout file inside its `.dat`
(grids are explained in [How to make a custom map](32_custom_maps.md)).
Every grid belongs to a **room**, numbered like `0x100`. `[[layout]]`
tells NAMS which of those objects to leave out, for one room or
everywhere, always or only in a phase.

## How do I remove an object?

Put the rule into a `.toml` file in your mod's `entities` folder, for
example `entities\layout.toml` (see
[How to lay out your mod folder](02_mod_folder_layout.md)):

```toml
[[layout]]
name = "<rule name>"

[[layout.skip]]
object = "bg2043"
```

With this rule no `bg2043` is placed in any room.

## When does it take effect?

Objects are placed when their grid is loaded, and NAMS checks the rules
right before it places each object. A skipped object is never created.

Saving the file during play registers the rules again. They apply to
every object placed after that; objects already standing stay.

To hide parts of a model instead of a whole object, see
[mesh visibility](28_mesh_visibility.md).

## Which fields does a rule have?

| Field | Required | What it does |
|---|---|---|
| `name` | yes | A unique name for the rule. |
| `room` | no | Only objects in grids of this room, for example `0x100`, the Ruined City. Left out: every room. |
| `phase` | no | Only while this phase is the current one, written as a number: the three hex digits of the phase name, `0x…`. Left out: every phase. |
| `subphases` | no | Only in these subphases of `phase`. Needs `phase`. |
| `not_subphases` | no | Never in these subphases of `phase`. Needs `phase`. |

Each `[[layout.skip]]` block is one object that is not placed.

| Field | Required | What it does |
|---|---|---|
| `object` | yes | The object name: two letters and four hex digits. |
| `pos` | no | `[x, y, z]`, a world position. Only placements within `radius` of it are skipped. |
| `radius` | with `pos` | The distance from `pos`, above zero. |

Without `pos`, every placement of the object in the rule's rooms is
skipped. With `pos` and `radius`, one placement can go while the others
of the same object stay.

## What if the object is still there?

Errors are written to the NAMS log, see
[How to debug when something goes wrong](17_debugging.md).

**The rule is refused.** The log has a line starting with
`[NAMS] Register failed for layout from mod '<mod id>':`, or during play
with `[NAMS] hot register failed for layout '<rule name>':`, followed by
one of these:

| Message | Fix |
|---|---|
| `layout needs a name` | Add `name`. |
| `layout '<rule name>' needs at least one skip` | Add a `[[layout.skip]]` block. |
| `layout '<rule name>': '<object>' is not an object name like bg0110` | Fix `object`. |
| `layout '<rule name>': pos needs a radius above zero` | Add `radius`. |
| `layout '<rule name>': radius needs pos` | Add `pos` or remove `radius`. |
| `layout '<rule name>': subphases need a phase` | Add `phase`. |

**A line starts with `[Native] <file> [layout]:`.** A field has the
wrong type, for example `phase` written as a `p` name instead of a
number.

**The log says `Two mods try to use the same layout number '<rule name>'`.**
Another mod uses the same `name`; the mod that loads first keeps it.

**No error, but the object stays.**

- It was placed before the rule was registered.
- The room or the phase does not match. Take `room` and `phase` out for
  a moment to check whether the rule works at all.
- The object is not placed from a grid's layout. Only those objects are
  skipped.
- `map_layouts` is `false` in [`nams.toml`](13_nams_toml.md).

**The object stays away after I deleted the rule.** A rule deleted from
a file stays registered until the next start. Deleting the whole file
takes its rules out at once. An object that was already skipped in a
grid that is still loaded is not placed afterwards.
