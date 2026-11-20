# How to make a weapon

## 1. Get a model

A weapon needs a 3D model - a `.dat` and `.dtt` pair. Either build one
in Blender, step by step in
[making a weapon model](resources/weapon_modelling.md), or take the
files from a weapon mod you downloaded.

Keep the name the files came with: `wp` and four hex digits, such as
`wp0003.dat` and `wp0003.dtt`, both in the same folder. The builder
reads the weapon class from that name.

## 2. Drop it into the builder

<!-- yp:weapon-builder -->

## 3. Name it, set the damage, press create

Each row under **UPGRADE LEVELS** is one level, from level 1 up. Give
every level the weapon can reach its own row: a level without a row
gets fixed values, `100` damage among them, instead of the stats the
builder starts from.

The builder picks a free inventory number and a free weapon number for
you. **Create weapon** copies the model to that weapon number and
writes the weapon file into your mod folder, then shows **Created**
with the file's path. This one is a weapon of its own and overwrites
nothing.

## 4. Find it in a shop

NAMS loads the weapon when NieR:Automata starts. With **Sell it in a
shop** ticked, the weapon is in that shop's list. Without a shop entry,
no shop sells it.

## When the weapon does not show up

The builder refuses to write the file and says why:

| The builder says | Fix |
|---|---|
| `That is not a weapon file. Expected something like wp0003.dat.` | Use the name the model came with: `wp` and four hex digits. |
| `Both the .dat and the .dtt must be in the same folder.` | Put the two files side by side. |
| `No free model slot left for this weapon type.` | Every weapon number of that class, or every inventory number, is taken in your mods folder. |

NAMS writes a line to its log when it cannot use a weapon file. Search
the log ([debugging](17_debugging.md)) for the weapon's inventory id:

| The log says | Fix |
|---|---|
| `needs an id between 1000 and 1999` | Give the file a number from 1000 to 1999. |
| `parse weapon_bundle: missing field` | Add the field it names. A missing `id` means the file name does not end in the number. |
| `Skipping weapon_bundle` ... `because the item '...' it depends on (from mod '<unstaged>') is not loaded` | A material `item_id` is neither in [vanilla item ids](resources/vanilla_item_ids.md) nor an item a loaded mod adds. |
| `Skipping weapon_bundle` ... `because the shop '...' it depends on (from mod '<unstaged>') is not loaded` | The `shop_name` is neither in [vanilla shop names](resources/vanilla_shop_names.md) nor a shop a loaded mod defines. |
| `Two mods try to use the same weapon_bundle number` | Another mod uses this inventory id. Of the two, the mod whose folder name sorts second is skipped; pick another number. |
| `duplicate item id` | Another item already uses this inventory id. Pick another number. |
| `duplicate weapon_upgrade for` | Two `[[upgrades]]` blocks have the same `lv`. |
| `duplicate shop_item for` | Two `[[shops]]` blocks name the same shop with the same `state`. Keep one. |
| `rule '...'` or `mesh_override:` | An effect rule or mesh override is wrong, see [special effects](11_special_effects.md). |

No line at all:

- The **New weapons** switch is off (`custom_weapons` in `nams.toml`).
  Turning it on needs a restart.
- A field or section name is misspelled. NAMS ignores names it does not
  know, without a message.

The weapon loads, but:

| You see | Cause |
|---|---|
| No name or description in one language | That language has no text of its own, see `[text]` below. |
| No skill name or text | `special1` or `special2` is `0`. |
| NAMS's placeholder instead of the icon or story picture | The `misctex` file was not found. Check its name. |
| A level drops to `100` damage | That level has no entry in `levels`. |
| Fewer upgrade materials than you listed | A level uses its first five materials only. |

---

# Reference

The builder writes the weapon to `entities\weapon_<id>.toml` in your
mod folder (see [mod folder layout](02_mod_folder_layout.md)). To edit
that file by hand, use the fields below.

## Picking the weapon id (filename id)

The number in the filename (e.g. `weapon_1500.toml` → `1500`) is the
weapon's **inventory id**. It must be between **1000 and 1999**; NAMS
rejects any other number at load time. An `id = ...` line in the file
takes precedence over the file name.

Inside the range, pick a number no vanilla weapon uses - see
[vanilla weapon ids](resources/vanilla_weapon_ids.md) for the taken
ones. NAMS adds no second weapon under a vanilla number. If two mods use
the same number, the mod whose folder name sorts second is skipped.

Shop entries and rewards refer to the weapon by this id.

## Picking the `obj_id` (model / behaviour slot)

The `obj_id` is a **separate** number from the inventory id. The model
files carry it: the builder writes the model for `0x30110` as
`data\wp\wp0110.dat` and `.dtt`. Its range decides the weapon class:

| `obj_id` range | Weapon class | `category` value |
|---|---|---|
| `0x30000` - `0x301FF` | Short sword | `0` |
| `0x30200` - `0x303FF` | Large sword | `1` |
| `0x30400` - `0x305FF` | Spear        | `2` |
| `0x30600` - `0x307FF` | Combat bracer (pod) | `3` |

If your `obj_id` falls outside `0x30000-0x307FF`, the weapon spawns
but never gets combat setup - it floats and does nothing. NAMS does not
check the `obj_id`, so keep it inside one of the four ranges.

Inside the range, two more rules:

1. **Don't reuse a vanilla `obj_id`.** NAMS then skips your `[params]`,
   and the weapon keeps the stats of the vanilla weapon with that
   `obj_id`. The taken slots are listed in
   [weapon obj_ids](resources/weapon_obj_ids.md).
2. **Never use a `wpmask` id** - these are reserved helper objects of
   the four weapon classes: `0x30072`, `0x30081`, `0x30222`,
   `0x30432`, `0x30602`.

Any other number in the right range works, such as `0x30110` for a
small sword. NAMS does not check whether another mod's weapon uses the
same `obj_id`. The builder skips the numbers used in your mods folder
and derives the number from your mod folder name.

## Which `category` goes with the `obj_id`

Write the `category` from the same row of the table above; the builder
always writes the pair from one row. NAMS does not check this for you:
a mismatched pair loads without complaint. `category` is written as the
weapon's class and picks the default `[effects]` pair.

Careful: `category` here means the weapon class, not the item category
you know from `item_<id>.toml`. Weapons keep an item category of `-1`;
this field is a different number with a different meaning.

## Minimal weapon

```toml
# entities\weapon_1500.toml
obj_id   = 0x30110   # small sword range, not vanilla, not wpmask
category = 0         # same row as the obj_id range: 0 = small sword

[item]
name = "weapon_virtuous_blade"
buy  = 5000
sell = 500

[text.name]
_us = "Virtuous Blade"

[text.help]
_us = "A weapon you added with NAMS."

[params]
clone_from_obj_id = 0x30013   # start from Iron Pipe's stats
levels = [{}, {}, {}, {}]     # keep them at all four levels

[[shops]]
shop_name = "shop_PV_Smith"
state     = 0
```

The builder puts the model for this `obj_id` into the same mod as
`data\wp\wp0110.dat` and `.dtt`.

## Top-level fields

### `id` (optional)
The inventory id. Leave it out and NAMS takes the number from the file
name.

### `obj_id` (required)
Hex value, must fall inside one of the four class ranges above, must
not be a vanilla or wpmask id. NAMS checks none of this. See "Picking
the `obj_id`" above for the full rules.

### `category` (required)
Weapon class code. Must match the range `obj_id` falls into:

| Value | Meaning | Required `obj_id` range |
|---|---|---|
| `0` | Short sword         | `0x30000` - `0x301FF` |
| `1` | Large sword         | `0x30200` - `0x303FF` |
| `2` | Spear               | `0x30400` - `0x305FF` |
| `3` | Combat bracer (pod) | `0x30600` - `0x307FF` |

### `special1`, `special2` (default `0`)
The numbers the texts of skill 1 and skill 2 are filed under. At `0`,
NAMS writes no text for that skill, so `[text.skill_1_*]` needs
`special1` and `[text.skill_2_*]` needs `special2`. Use numbers no
other weapon uses: two weapons with the same number show the same skill
text. The builder writes the inventory id followed by `1` and `2`,
such as `15001` and `15002` for weapon `1500`.

### `story_texture_suffix` (optional)
Picture shown on the weapon's story pages; the builder calls it
**Story image**. NAMS loads it from
`data\misctex\misctex_ws_<suffix>.dat`/`.dtt`, so ship that pair in
your mod. The builder uses the weapon's inventory id as the suffix.
When the file does not load, NAMS shows its placeholder picture. Leave
the field unset and NAMS adds no picture.

## `[item]` - inventory side

Every weapon is also an inventory item. The `[item]` block defines that
side, with these fields of an item file. Leave `[item]` out and NAMS
adds the weapon neither to the inventory items nor to the weapon list,
and `[[shops]]` has nothing to sell.

### `name` (required)
Internal name. NAMS finds the weapon for its shop entries by this name,
so use one no other item has. Vanilla weapons use
`weapon_<descriptive_name>`, such as `weapon_japanese_sword` for Faith;
the builder writes `weapon_` and the weapon name in lower case.

### `max_num` (default `1`)
The inventory item's maximum number. The builder writes `1`.

### `no_trash` (default `false`)
Copied into the inventory item. The builder writes `true`.

### `die_discard` (default `false`)
Copied into the inventory item. The builder writes `false`.

### `buy` (default `-1`)
Shop buy price. `-1` = not buyable. Use this together with
`[[shops]]` to put your weapon in a shop's inventory.

### `sell` (default `-1`)
Shop sell price. `-1` = not sellable.

## `[icon]`

Sets the weapon's item icon, the builder's **Inventory icon**. Same as
the item icon block:

```toml
[icon]
misctex_name = "wpi_1500" # the name the builder gives weapon 1500's icon
```

NAMS loads `misctex_<name>.dat`/`.dtt` from the `misctex` folder, so
write the name without the `misctex_` prefix and without extension. To
ship your own, put `misctex_<name>.dat` and `.dtt` into your mod's
`data\misctex\` folder and set `misctex_name = "<name>"`. The builder
does this for you.

Without `[icon]`, NAMS looks for `misctex_item_thumb_<id>`. When no
file loads, it shows its placeholder icon.

## `[text]` - localised text

Localised strings of the weapon:

### `[text.name]`
The weapon's display name.

### `[text.help]`
The weapon's description.

### `[text.help_short]`
Short description.

### `[text.skill_1_name]`, `[text.skill_1_desc]`
First weapon skill name and description. Needs `special1`.

### `[text.skill_2_name]`, `[text.skill_2_desc]`
Second weapon skill name and description. Needs `special2`.

All `[text.*]` blocks accept multiple language codes.

NAMS sets a language's text only from the code for that language.
Chinese and Korean also take `_us` when their own code is missing; the
other languages do not. Write every language you want covered. See
[translations](12_translate.md) for the language codes that exist.

## `[[story]]` - flavour pages

Each `[[story]]` block is one page of the weapon's story. The first
block is page 1, the next page 2, and so on:

```toml
[[story]]
_us = "Recovered from an unidentified YoRHa unit."

[[story]]
_us = "The blade sings when held."

[[story]]
_us = "Command has classified the blade as non-standard equipment."
```

Each entry takes the same language map as `[text.*]`.

## `[params]` - the stats

The simplest way to set up the weapon's combat numbers is to **clone an
existing vanilla weapon** and change only what you want different:

```toml
[params]
clone_from_obj_id = 0x30013
levels = [
  { atk_speed = 150 },
  { atk_speed = 150 },
  { atk_speed = 200 },
  { atk_speed = 200 },
]
```

### `clone_from_obj_id` (optional)
Copy stats from this vanilla weapon as the baseline. This is an
`obj_id`, **not** an inventory id. A vanilla weapon's `obj_id` is `0x30`
followed by the last three digits of its inventory id, such as
`0x30013` for Iron Pipe (`1013`). The DLC weapons are the exception:
`1875` is `0x30075`, `1876` is `0x30076` and `1877` is `0x30655`.

The full list is in [weapon obj_ids](resources/weapon_obj_ids.md); the
matching names are in
[vanilla weapon ids](resources/vanilla_weapon_ids.md).

When set, NAMS starts from that weapon's trait and level values. The
`levels` array below then **overrides** specific fields per upgrade
level. A modded weapon cannot be a clone source: with its `obj_id`, or
a number no weapon has, NAMS starts from the fixed values listed under
`levels`.

### `trait_id` (optional)
The weapon's trait value. Leave it unset to keep the value of the
cloned weapon (or `0` without a clone).

### `levels`
Per-upgrade-level stat overrides. **The position in the array is the
level** - the first entry is level 1, the second level 2, and so on.
There is no `lv` field here; a weapon patch uses one, a weapon file
does not.

Each entry sets only the fields that differ from the cloned baseline.
Available stat fields (all optional, all integers):

| Field | Meaning |
|---|---|
| `min_damage`, `max_damage` | damage range |
| `combo_light`, `combo_heavy` | combo values |
| `atk_speed` | attack speed |
| `def_scale` | defence scaling |
| `stagger_scale` | stagger scaling |
| `crit_rate` | critical rate |

**A level without an entry does not keep the cloned values.** NAMS
gives it `min_damage = 100`, `max_damage = 100`, `combo_light = 8`,
`combo_heavy = 8`, `atk_speed = 100`, `def_scale = 100`,
`stagger_scale = 100`, `crit_rate = 0`, the same values every level
starts from without `clone_from_obj_id`. To keep the cloned values at a
level, give it an entry, even an empty `{}`. Entries past the fourth
are ignored.

## `[effects]` - attack trail

Two effect set ids, the builder's **Attack trail** and **Attack trail
(Berserk)**:

```toml
[effects]
est_id_normal = 600
est_id_berserk = 633
```

Leave the section out entirely and NAMS fills in the pair that matches
your `category`:

| `category` | normal | berserk |
|---|---|---|
| `0` small sword | `600` | `633` |
| `1` large sword | `610` | `634` |
| `2` spear | `620` | `635` |
| `3` combat bracers | `630` | `636` |

With the section, write both fields.

### `est_id_normal`
Effect set used in normal combat mode.

### `est_id_berserk`
Effect set used during Berserk mode.

Both are ids from [the EST id catalogue](resources/est_id_catalog.md),
valid range `0..=665`; `435` and `436` crash NieR:Automata. A weapon
file is not checked: NAMS takes whatever you write. Only weapon patches
refuse numbers outside the range, and the builder skips `435` and
`436`.

## `[[upgrades]]` - upgrade tree

Each block is one upgrade step of the weapon: the level it upgrades
to, its cost and its materials. Without blocks NAMS adds no upgrade
step. One block per level:

```toml
[[upgrades]]
lv = 3
cost = 5000
materials = [
  { item_id = 643, quantity = 3 },
]
```

### `lv` (required)
The level this step upgrades to. Use `2`, `3`, `4`; a block with any
other level is ignored.

### `cost` (required)
What this upgrade step costs. The builder calls it **Cost**.

### `materials` (default: none)
The material items of this step. Each entry has:

- `item_id` - the item id (vanilla or modded). It has to be in
  [vanilla item ids](resources/vanilla_item_ids.md) or be an item a
  loaded mod adds; any other number makes NAMS skip the whole weapon.
- `quantity` - how many of that item.

NAMS uses the first five materials of a level and ignores the rest.

Look the ids up in [vanilla item ids](resources/vanilla_item_ids.md)
rather than typing one from memory: a wrong digit that hits another
vanilla item gives the player a different material without any error.

## `[[shops]]` - shop placement

Decides which shops sell your weapon. Each block creates one shop
entry; add a block for every shop. The weapon needs its `[item]` block
for this.

```toml
[[shops]]
shop_name = "shop_PV_Smith"
state = 0
```

### `shop_name` (required)
The internal name of the shop. The full list is in
[vanilla shop names](resources/vanilla_shop_names.md); a shop a loaded
mod defines works too.

The name has to match exactly: with a name that is neither in that list
nor a shop of a loaded mod, NAMS skips the whole weapon.

### `state` (default `0`)
Stored, but not evaluated: NAMS adds the weapon to the shop no matter
which story state the shop is in. Write `0`. One block per shop is
enough.

Set the price with `buy` in the `[item]` block. A shop list holds at
most 64 entries; when it is full, NAMS adds no more.

## `[effect.*]` and `[[mesh_override]]` - buffs and visuals

Optional. A weapon can apply effect rules or change mesh visibility
while it is in the weapon set the player is using. `[[effect_array]]`
takes the same rules without a name. See
[§11 How to make special effects](11_special_effects.md)
for the full reference. Brief example:

```toml
# gold multiplier 1.20 while equipped
[effect.gold_buff]
path = "player.gold_gain"
op = "mul"
value = 1.20

# Hide 2B's Eyemask mesh while equipped
[[mesh_override]]
model = "pl2b"
mesh = "Eyemask"
force = "hide"
```

## Patching an existing weapon

To change a weapon that already exists, write a patch instead of a full
weapon. See
[how to patch an existing weapon](30_weapon_patch.md).
