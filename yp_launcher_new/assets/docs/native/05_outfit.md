# How to make an outfit

**You need the costume model first.** An outfit starts from a player
model you already have: a `.dat` / `.dtt` pair named after the
character's model file, `pl0000` or `pl000d` for 2B, `pl0200` or
`pl020d` for 9S, `pl0100` or `pl010d` for A2.

An outfit entry turns that model into an item the player owns and puts
on. It can also carry mesh rules, so
parts of the model (its meshes) appear and disappear with what the
character is doing. A shop entry, in a file of its own, can sell the
item.

Without an entry, a model mod is a plain folder: the model sits in the
mod's `data\pl\` folder, and the outfit selection NPC (the NPC that
offers modded outfits, `outfit_selector_npc` in
[How to configure NAMS](13_nams_toml.md)) lists it under the mod's
folder name and switches it on. That works and needs no config at all.
Make an outfit when you want the model to be a separate thing the
player owns, rather than a replacement.

## Start here

Drop the model's `.dat` or `.dtt` in below - the model file of a plain
folder mod you already use works fine. Fill in **Outfit name** and
press **Create outfit**.

<!-- yp:outfit-builder -->

## What ends up in the file

For an outfit named `Black Dress`, the builder writes four lines:

```toml
outfit_id = 10001
character = "2B"
item_id = 8000
name = "black_dress"
```

`10001` and `8000` are the first numbers the builder hands out; when
they are taken, it picks the next free ones.

It lives in your mod's `entities\` folder as `outfit_<outfit_id>.toml`,
next to the `item_<item_id>.toml` it points at. The model goes under
`data\pl\`.

When the player uses the item, NAMS switches on the mod's `data\pl\`
folder and the character changes into the model. Using the item again
takes the outfit off: the folder is switched off and the character goes
back to the default outfit. Load a save, open the inventory and use the
item to see it.

Everything below is optional.

## The item

The item is what the player uses to put the outfit on. It is an
ordinary item file - there is no outfit flag to set on it and no
special name it needs. NAMS treats an item as an outfit **because an
outfit row points at its `item_id`**, and nothing else.

For the outfit above, with **Item description** left empty, the builder
writes:

```toml
name = "item_black_dress"
id_type = 0
category = 6
no_trash = true

[auto_give]
quantity = 1

[text.name]
_us = "Black Dress"

[text.help]
_us = '''
'''
```

`name` is the **Outfit name** in lowercase with `item_` in front.
`[text.name]` and `[text.help]` are what the player reads, see
[How to make an item](03_item.md).

### Getting it to the player

`[auto_give]` puts the item straight in the inventory. Pending items are
handed out when a save is loaded, not at startup, so load a save and it
is there.

Drop `[auto_give]` if you would rather the player earn it, and put it in
a shop instead.

The outfit selection NPC offers the outfit to its character whether the
player owns the item or not, see `name` below.

## Selling it in a shop

A shop entry is its own file, not a section of the item. With **Sell it
in a shop** switched on, the builder writes it as `shop<item_id>.toml`:

```toml
# entities\shop8000.toml
[[shop_item]]
shop_name = "shop_RC_Grocery"
item_id = 8000
state = 0
```

Do not start the file name with a kind and an underscore, such as
`item_`: NAMS then reads the whole file as that kind (see
[how to lay out your mod folder](02_mod_folder_layout.md)).

- `shop_name` must be a real shop. The valid names are listed in
  [Vanilla shop names](resources/vanilla_shop_names.md). A name that is
  not one of them is not an error, but the entry is skipped and the log
  says
  `[NAMS] Skipping shop_item '<shop_name>:<item_id>:<state>' from mod '<mod>' because the shop '<shop_name>' it depends on (from mod '<unstaged>') is not loaded.`
- `state` has to be `0` to `6` or the entry is rejected, but NAMS does
  not evaluate it: the item is offered in every story state. The
  builder writes one block per story state you tick; one block with
  `0` is enough.
- Set the price with `buy` on the **item** file. With the shop switched
  on, the builder adds `buy = 5000`, its default **Price**.

The same item can be in several shops - one `[[shop_item]]` block per
entry, all in the same file.

## When it does not work

**The outfit is missing, and the log says**
`[NAMS] Skipping outfit_bundle '<id>' from mod '<mod>' because the item '<item_id>' it depends on (from mod '<unstaged>') is not loaded.`
No loaded mod provides `item_<item_id>.toml`, and `item_id` is not a
base item. Ship the item file.

**`[NAMS] Register failed for outfit_bundle from mod '<mod>': duplicate (outfit_id=<outfit_id>, <character>) — these together must be unique`**
Another outfit of that character already uses this `outfit_id`.
`<character>` reads `TwoB`, `NineS` or `A2`. Pick another number.

**`[NAMS] Two mods try to use the same outfit_bundle number '<id>'.`**
Two mods use the same file number. The outfit of the mod named after
`Skipped:` does not load; change its `id`.

**`` [Native] <file> [outfit_bundle]: parse outfit_bundle: missing field `id` ``**
The file name has no number after the underscore. Write `id = ...` into
the file.

**Using the item does not change the model.** The `outfit_id` is in the
range of the base outfits (see `outfit_id` below), or the mod has no
`data\` folder.

**Parts of the model never switch on or off.** None of the names in the
mesh lists exists in the model; NAMS then changes no mesh at all. Check
the names, see `mesh` below.

**A `[[mesh_override]]` does nothing.** The outfit has no mesh lists,
see [What happens without mesh lists?](#what-happens-without-mesh-lists)

**Changed mesh lists do not show.** NAMS reads mesh lists once at
startup. Restart.

**Buffs are gone after loading a save.** Loading a save does not switch
an outfit's `[effect.*]` rules and `[[mesh_override]]` blocks on. Put
the outfit on again.

A broken effect rule is described in
[§11 How to make special effects](11_special_effects.md).

## The outfit fields

### `id` (auto from filename)
Row id, unique across all mods. The number in the filename
(`outfit_10001.toml` → `10001`) is used unless you override with
`id = ...` inside the file.

If the part after the underscore is not a number
(`outfit_<name>.toml`), it fills in `name` instead of `id`. `id` is
required, so such a file has to carry `id = ...` itself or it fails
with `` missing field `id` ``.

### `outfit_id` (required)
The outfit number the character wears while the outfit is on. The
`outfit_id` of a `[[mesh_override]]` and of an effect rule's `when`
compare against it.

The pair `(outfit_id, character)` must be unique - you can't have two
outfits for 2B with the same `outfit_id`.

Keep it above the base outfits, which use `0` to `3` for 2B and `0` to
`1` for 9S and A2. An `outfit_id` in that range is left out of the
outfit selection NPC's list, and using the item may put on the base
outfit with that number instead of your model.

The builder numbers from 10001 for 2B, 20001 for 9S and 30001 for A2,
and takes the next number that no outfit file of that character in
your mods folder uses. NAMS itself does not reject a low number.

### `character` (required)
Which character wears this outfit. Accepted values: `"2B"`, `"9S"`,
`"A2"`. Case does not matter here - `"2b"` and `"2B"` both parse.

### `item_id` (required)
The inventory item that puts the outfit on. Ship a matching
`item_<item_id>.toml` (any loaded mod may provide it). If no loaded mod
provides it and it is not a base item, the outfit is skipped, see
[When it does not work](#when-it-does-not-work).

### `name` (default `""`)
The name the outfit selection NPC lists the outfit under. An outfit
without a `name` is not listed there. The inventory shows the item's
`[text.name]` instead. The builder writes the **Outfit name** in
lowercase with underscores.

## What happens without mesh lists?

If all mesh-list fields are omitted, NAMS treats the outfit like a
plain folder mod and leaves the meshes to the default assembly. Use
this for a model that already shows the right parts without rules of
its own. `[[mesh_override]]` blocks have no effect on
such a model, the outfit's own included.

Once you add mesh lists, a mesh in `meshes` is shown unless a state
list hides it, so do not list every DLC or armor mesh just because it
exists in the model.

## Showing parts of the model by state

```toml
meshes = ["Body", "Skirt", "Hair"]
combat_meshes = ["facial_serious"]
non_combat_meshes = ["facial_normal"]
eyemask_meshes = ["Eyemask"]
non_eyemask_meshes = ["Eyelash"]
broken_meshes = ["Broken"]
non_broken_meshes = ["Body"]
```

Once any mesh-list field is populated, NAMS assembles the model's
meshes itself while the outfit is worn and starts with all meshes
hidden. `meshes` defines the base visible set. The available state
lists, with the option in the builder's mesh list that writes each:

| Field | Runtime state | Builder option |
|---|---|---|
| `meshes` | Base visible meshes | Always visible |
| `self_destruct_show_meshes` | Shown after self-destruct | - |
| `self_destruct_hide_meshes` | Hidden after self-destruct | Gone after self-destruct |
| `combat_meshes` | Shown in combat, hidden outside | Only in combat |
| `non_combat_meshes` | Shown outside combat, hidden in combat | Only out of combat |
| `eyemask_meshes` | Shown while the eyemask (blindfold) is rendered | Only with blindfold |
| `non_eyemask_meshes` | Shown while the eyemask is not rendered | Only without blindfold |
| `wig_meshes` | Body meshes hidden while A2's wig renders them | Replaced by the wig |
| `broken_meshes` | Shown while the feather is not rendered | Only while damaged |
| `non_broken_meshes` | Shown while the feather is rendered | Only while undamaged |
| `prologue_meshes` | Prologue story state | - |
| `holy_meshes` | Holy story state | - |
| `no_right_meshes` | No-right-side story state | - |
| `tower_meshes` | Tower story state | - |

The eyemask counts as not rendered while Adam's Glasses or the
Camouflage Goggles are worn.

`broken_meshes` / `non_broken_meshes` key off the feather-hidden flag,
which is **not** the same condition as self-destruct - use the
`self_destruct_*` pair for that. The `self_destruct_*` lists also apply
while the feather is hidden, unless the Dress Module has set the
character's dress state to `2`.

When a story state is active and its list is non-empty, that list
replaces `meshes` as the base set. It is not added on top.

When two lists name the same mesh, the one further down this order
wins:
`meshes` or the story list, the feather pair, `self_destruct_hide_meshes`,
`self_destruct_show_meshes`, the eyemask pair, the combat pair,
`wig_meshes`.

If a mesh listed in `non_eyemask_meshes`, `combat_meshes` or
`non_combat_meshes` has a name ending in `Eyelash`, NAMS automatically
derives a `<name>_serious` mesh and shows it in combat only. You do not
need to list it.

`hairspray_meshes` is not a visibility state - see
[Which meshes a hair spray colours](#which-meshes-a-hair-spray-colours)
below. It does count as "a mesh-list field is populated", though: an
outfit that sets only `hairspray_meshes` leaves the default assembly but
gives NAMS no mesh to show or hide, so no part of the model switches
with any state. If you set it, set `meshes` too.

## Forcing one mesh on or off

`[[mesh_override]]` is the same block weapons and accessories use. It
force-shows or force-hides a mesh while the outfit is worn. It is
applied after the state lists, so it wins over them, and it only works
when the outfit has mesh lists.

```toml
[[mesh_override]]
model = "pl2b"
mesh = "Skirt"
force = "show"
outfit_id = 10001
```

### `model` (required)
Which player model the override applies to:

| Value | Character |
|---|---|
| `"pl2b"` or `"2B"` | 2B |
| `"pl9s"` or `"9S"` | 9S |
| `"pla2"` or `"A2"` | A2 |
| `"pla2_wig"` or `"a2_wig"` | A2's wig part |

Unlike `character` above, these are matched exactly - `"2b"` in
lowercase is rejected here.

### `mesh` (required)
The mesh name from the player model. A name the model does not have is
ignored.

The vanilla names, listed per character, are in
[Vanilla mesh node names](resources/vanilla_mesh_names.md). Each
character has a different model - a name that exists on 9S may not
exist on 2B.

For a custom model, or to check what a model really contains, see
[Finding the names in a model](resources/vanilla_mesh_names.md#finding-the-names-in-a-model).

### `force` (required)
What to do with the mesh:

- `"hide"` - force the mesh invisible.
- `"show"` - force the mesh visible.

### `outfit_id` (optional)
Restrict the override to a specific outfit. When set, the override only
applies when the player is wearing the outfit with that `outfit_id`.
Useful when one mod ships multiple outfits and each needs a different
mesh combination.

Leaving `outfit_id` unset does **not** make the override global.
Overrides are only active while the outfit they belong to is worn;
unset means "no extra check beyond that". An override on a
weapon or accessory without `outfit_id` applies whatever outfit is
worn, see [§11 How to make special effects](11_special_effects.md).

## Which meshes a hair spray colours

`hairspray_meshes` lists the meshes NAMS tints when it colours the
character's hair for a hair spray. It shows and hides nothing.

If you omit it, `Hair` is tinted.

## Textures that belong to the outfit

`.dds` files in your mod's `textures\` folder are only active while a
character wears one of the mod's outfits, see
[How to replace textures](33_texture_injection.md).

## Buffs while the outfit is worn

An outfit can carry `[effect.<name>]` tables or `[[effect_array]]`
entries in the outfit file. See
[§11 How to make special effects](11_special_effects.md)
for the full effect rule reference.

They switch on when the outfit is put on, with its item or at the
outfit selection NPC, and off when it is taken off. `levels` and
`when.weapon_level` belong to weapons: on an outfit, a rule that uses
them never applies.
