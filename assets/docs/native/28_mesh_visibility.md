# How to hide, show or recolor parts of the world

A **mesh** is a named part of a loaded model, of a map piece as much as
of an object. With a `[[mesh_visibility]]` block you can hide meshes by
name, keep them shown, or tint them, either everywhere or only in a
certain room or story phase.

## The short version

Put this into a TOML file in your mod's `entities` folder, such as
`entities\meshes.toml` (see
[How to lay out your mod folder](02_mod_folder_layout.md)):

```toml
[[mesh_visibility]]
name   = "no_camp_placeholder"
meshes = ["g11420_dummybuild"]
hide   = true
room   = 0x110
```

While the player is in room `0x110`, the Resistance Camp, every loaded
mesh named `g11420_dummybuild` is hidden.

## Finding mesh names

Open the overlay, go to the **MAP MANAGER** and press **Pick Map Meshes**
in its toolbar. Point at a mesh in the world and click it: the **Mesh
Picker** shows its name with a copy button, its **Object id** and its
**Mesh color**. Two real names:

```
g11420_dummybuild
nolowmap_enkei
```

Names are matched whole and without regard for upper and lower case, so
`NoLowMap_Enkei` and `nolowmap_enkei` are the same thing.

## `[[mesh_visibility]]`

| Field | Default | What it does |
|---|---|---|
| `name` | needed | A unique name for the rule. Error messages name the rule by it. |
| `meshes` | none | The mesh names the rule applies to. One rule can cover several. |
| `object` | any object | Only meshes of the object with this id, the **Object id** the mesh picker shows. Without `meshes`, all meshes of that object. |
| `hide` | `false` | `true` hides the meshes, `false` keeps them shown. |
| `color` | unchanged | Tints the meshes, see below. |
| `receive_shadow` | unchanged | Shadows on the whole object, see below. |
| `room`, `phase`, `subphases` | everywhere | Where and when the rule applies, see below. |

A rule needs `meshes`, an `object`, or both. Every mesh a rule matches is
set to shown or hidden, also by a rule that only sets `color` or
`receive_shadow`: without `hide = true` it is kept shown.

### `color`
Three or four values: red, green, blue and alpha, each `0` or more. They
multiply the colors the mesh draws: `1.0` keeps a channel, lower
darkens, higher brightens. Alpha is `1.0` when only three values are
given. The mesh picker shows a mesh's current values under **Mesh
color**.

### `receive_shadow`
`true` lets shadows darken the object, `false` keeps them off it. Leave
it out to change nothing. It works on the whole object, not on single
meshes, so a rule with `object` and `receive_shadow` needs no `meshes`.

## Deciding where it applies

Leave all of the following out and the rule applies everywhere, all the
time. Set several and they all have to be true at once.

### `room`
Only while the player is in that room.

`0x110` is the Resistance Camp. To find a room number, open the overlay
and look at the **MAP MANAGER**, which shows the current room as
`Grid (Current: 0x110)`.

### `phase` and `subphases`
Only while the story is at that point. `phase` is the story phase as a
number, `0x200` for `p200`; `subphases` narrows it to those subphases.

```toml
[[mesh_visibility]]
name      = "no_camp_placeholder"
meshes    = ["g11420_dummybuild"]
hide      = true
phase     = 0x200
subphases = ["170_00_A_Resistance_Camp_2B"]
```

When a rule stops applying, NAMS stops setting its meshes; it does not
switch them back.

## Hiding something everywhere

Leaving out every condition makes the rule apply in every room and
phase:

```toml
[[mesh_visibility]]
name   = "enkei_off"
meshes = ["nolowmap_enkei"]
hide   = true
```

## When rules overlap

If two rules match the same mesh at the same time, only the one
registered first counts, for `hide` and for `color`. A rule you save
while you play is registered again and counts as the last one.

## Editing while you play

Change `meshes.toml` while you play and save it. The rule is
reloaded and the world updates on the spot, without a restart and
without reloading the area.

For example, save this:

```toml
[[mesh_visibility]]
name   = "enkei_off"
meshes = ["nolowmap_enkei"]
hide   = true
```

then set `hide = false` and save again, and the mesh comes back.

The same works for `color`: change the values and save, and the tint
updates on the spot. Removing the `color` line does not undo a tint; set
it back to the values the mesh picker showed under **Mesh color** before.

Deleting a block from the file does not take the rule out while you
play; it stays until the whole file is deleted. Set `hide = false`
instead.

## When nothing changes

Look in the log for lines starting with `[MeshVisibility]`. One appears
whenever the set of meshes the rules hold changes:

```
[MeshVisibility] room <room>: <meshes> mesh(es) on <objects> object(s)
```

The usual causes:

- The mesh name is spelled differently from the loaded one. Check it
  again in the mesh picker.
- `hide = true` is missing. Without it the rule keeps the meshes shown.
- The room or phase does not match, so the rule never applies. Take one
  condition out at a time to find which.
- The rule was refused when it loaded. The log says why:
  `[NAMS] Register failed for mesh_visibility from mod '<mod id>': <reason>`.
- `mesh_visibility = false` under `[content]` in `nams.toml` switches
  every rule off, see [How to configure NAMS](13_nams_toml.md).
