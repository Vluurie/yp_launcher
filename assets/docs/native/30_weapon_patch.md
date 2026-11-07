# How to patch an existing weapon

A weapon patch changes a weapon that already exists - a vanilla one, or
one a mod added. You only write down what you want to be different;
everything else stays as it was.

To give Virtuous Contract more damage, for example, you do not redefine
the weapon, and you do not need any `data\wp` files.

The file is `entities\weapon_patch_<weapon-id>.toml` (see
[mod folder layout](02_mod_folder_layout.md)). The number is the
weapon's **inventory id**, the same one you see in
[vanilla weapon ids](resources/vanilla_weapon_ids.md).

```toml
# entities\weapon_patch_1070.toml - Virtuous Contract
[params]
[[params.level]]
lv = 4
min_damage = 400
max_damage = 520
```

That is a complete patch. It sets the damage range of level 4 of that
weapon.

## What you can change

| Section | Changes |
|---|---|
| `[params]` | the trait, and per level damage, combos, attack speed and the other level values |
| `[effects]` | the attack trail |
| `[effect.<name>]`, `[[effect_array]]` | a rule, e.g. faster dodge while equipped |
| `[[mesh_override]]` | show or hide a mesh |
| `obj_id` | which weapon `[params]` and `[effects]` go to, see below |

What a patch **cannot** do: create an inventory item, add upgrade
levels, texts, an icon or a shop entry. Those only exist on a full
weapon, see [how to make a weapon](04_weapon.md). Sections a patch does
not know are ignored without a message.

## When the patch does nothing

NAMS writes a line to its log when it cannot use a patch. Search the
log ([debugging](17_debugging.md)) for the weapon's inventory id:

| The log says | Fix |
|---|---|
| `Skipping weapon_patch_bundle` ... `because the weapon '...' it depends on (from mod '<unstaged>') is not loaded` | The number in the file name is no vanilla weapon, and no loaded mod adds a weapon with it. Fix the number, or set `obj_id`. |
| `parse weapon_patch_bundle: missing field` | Add the field it names. |
| `parameter level ... is outside 1..=4` | Use `lv` `1` to `4`. |
| `contains parameter level ... more than once` | Write each level once. |
| `is outside 0..=665` | Use an effect id from `0` to `665`. |
| `is blocked because it crashes` | Do not use `435` or `436`. |
| `is outside the player weapon ranges` | Set an `obj_id` inside a weapon class range. |
| `is a reserved wpmask` | Use an `obj_id` that is not a wpmask id. |
| `duplicate weapon_patch obj_id` | Two patches reach the same weapon. Put their changes in one file. |
| `Two mods try to use the same weapon_patch_bundle number` | Another mod patches the same weapon. Of the two, the mod whose folder name sorts second is skipped. |
| `rule '...'` or `mesh_override:` | See [special effects](11_special_effects.md). |
| `[WeaponPatch] Could not resolve obj_id for weapon` | The weapon the patch is for did not load. Look for its own error. |
| `[WeaponPatch] No CallSelector entries matched` or `[WeaponPatch] No WeaponParam entry matched obj_id` | No weapon uses that `obj_id`. Check `obj_id`. |

No line at all:

- The **Weapon changes** switch is off (`weapon_patches` in
  `nams.toml`). Turning it on needs a restart.
- A section name is misspelled.

## `[params]` - damage and handling

One block per level you want to change. Levels you leave out keep the
values they have.

```toml
[params]

[[params.level]]
lv = 1
min_damage = 120
max_damage = 160
atk_speed = 150

[[params.level]]
lv = 4
min_damage = 400
max_damage = 520
```

`lv` is required and must be between **1 and 4**, and each level may
appear only once - NAMS rejects the patch otherwise.

Every other field inside a level is optional:

| Field | Meaning |
|---|---|
| `min_damage`, `max_damage` | damage range |
| `combo_light`, `combo_heavy` | combo values |
| `atk_speed` | attack speed |
| `def_scale` | defence scaling |
| `stagger_scale` | stagger scaling |
| `crit_rate` | critical rate |

`trait_id`, written directly under `[params]`, replaces the weapon's
trait value.

## `[effects]` - the attack trail

```toml
[effects]
est_id_normal = 600
est_id_berserk = 633
```

Write both. Both ids must be within `0..=665`. **435 and 436 are
refused** - they crash NieR:Automata. See
[EST ids](resources/est_id_catalog.md) for what is available.

## Patching by `obj_id`

Normally you leave this out. NAMS then looks the weapon up by its
inventory id, and skips the patch if no such weapon is loaded.

Set `obj_id` to skip that lookup. `[params]` and `[effects]` then go to
whatever weapon uses this `obj_id`; `[effect.*]` and
`[[mesh_override]]` still follow the inventory id in the file name.

```toml
obj_id = 0x30070

[effects]
est_id_normal = 600
est_id_berserk = 633
```

With `[effects]` in the patch, the `obj_id` has to sit inside a weapon
class range and must not be one of the reserved `wpmask` ids; NAMS
refuses the patch otherwise. See
[weapon obj_ids](resources/weapon_obj_ids.md).

## One patch per weapon

Two patches for the same weapon collide. If two mods patch the same
inventory id, the mod whose folder name sorts second is skipped. If two
patches reach the same `obj_id`, only the first one's `[params]` and
`[effects]` load, and the log says `duplicate weapon_patch obj_id`. If
you want several changes on one weapon, put them in the same file.

## Effect rules and mesh overrides

Both work as on a full weapon: they apply while the weapon is in the
weapon set the player is using. `[[effect_array]]` takes the same rules
without a name.

```toml
[effect.faster_dodge]
path = "player.dodge.propulsion_9s.forward"
op = "mul"
levels = { 1 = 1.0, 4 = 2.5 }
when = { player = "9S" }
```

A rule with `levels` is off at the levels it has no value for, here
levels 2 and 3. See [special effects](11_special_effects.md) for the
full rule reference and [outfits](05_outfit.md) for how
`[[mesh_override]]` behaves.
