# How to make special effects (buffs, immunities, mesh visibility)

Weapons, accessories, and outfits can apply effects while equipped.
Two kinds:

1. **Stat tweaks** (`[effect.*]`) - change numbers and switches on the
   player.
2. **Mesh overrides** (`[[mesh_override]]`) - force pieces of the
   player model to be visible or hidden.

Items and scenario flags carry the same `[effect.*]` rules, see
[Effects on items](#effects-on-items) and
[Effects while a flag is on](#effects-while-a-flag-is-on).

## Stat tweaks (`[effect.*]`)

Each effect rule is a TOML table named `[effect.<rule_name>]`. The
name is yours to choose - it shows up in error messages, so use
descriptive names:

```toml
[effect.stagger_immune]
path = "player.stagger_immune"
op = "set"
value = 1
```

Rules can also be written as `[[effect_array]]` blocks without a name;
error messages call them `_1`, `_2`, … in the order of the file.

A rule has three required parts:

### `path` (required)
Tells NAMS **what** to change. Full list of valid paths in the tables
below. A path NAMS does not know stops the rule from loading with
`unknown path '<path>'`.

### `op` (required)
The **operation** to apply. Three valid values:

| `op` | What it does |
|---|---|
| `"mul"` | Multiplies the value |
| `"add"` | Adds to the value |
| `"set"` | Replaces the value |

Use the op the tables below give for each path. On a switch path, a
`set` or `add` rule with a value other than `0` switches it on.

When several rules change one path, NAMS multiplies by every `mul`
value, then adds every `add` value, and a `set` rule replaces the
result. Two rules of the same weapon, accessory, outfit or item with
the same path and op do not stack: one replaces the other.

### `value` (one of `value` or `levels`)
The **number** to apply. For most rules just set `value = X`. For
weapon rules that scale with upgrade level, use `levels` instead (see
"Per-level" below).

You must set **exactly one** of `value` or `levels`. Setting both, or
neither, makes NAMS reject the rule with an error.

## When do the rules switch on?

| Source | Rules switch on | Rules switch off |
|---|---|---|
| Weapon | when it is in the weapon set in use; again when a weapon of that set, its upgrade level or the character changes | when it leaves the set in use |
| Accessory | when it is put on; again when an outfit item is used, when control switches to its character and when a save is loaded | when it is taken off |
| Outfit | when it is put on, with its item or at the outfit selection NPC | when it is taken off |
| Item | every frame, see [Effects on items](#effects-on-items) | |
| Scenario flag | every frame while the flag is on | when the flag is off |

`when` (see below) is checked at these moments. A weapon's rules are
not checked again when only the outfit changes, and loading a save
does not switch an outfit's rules on: put the outfit on again.

## When a rule does not load or does nothing

A rule NAMS cannot read stops every rule and mesh override of that
weapon, accessory, outfit, item or flag from loading. The log names the
entry and the rule:

```
[NAMS] Register failed for <kind> from mod '<mod>': <entry> rule '<rule>': <reason>
```

While NAMS runs, a changed file reports the same reason as
`[NAMS] hot register failed for <kind> '<key>': <entry> rule '<rule>': <reason>`.
`<entry>` is `weapon <id>`, `weapon_patch <id>`, `accessory <id>`,
`outfit <outfit_id>`, `item <id>` or `scenario_flag '<name>'`.

| `<reason>` | Fix |
|---|---|
| `unknown path '<path>'` | Use a path from the tables below. |
| `unknown op '<op>' (expected mul/add/set)` | Write `mul`, `add` or `set`. |
| `` rule '<rule>': exactly one of `value` or `levels` must be set `` | Keep one of the two. |
| `level key '<key>' must be 1/2/3/4` | Use the keys `1` to `4` in `levels`. |
| `unknown player '<player>' (expected 2B/9S/A2/not_2b/not_9s/not_a2)` | Use one of those values in `when.player`. |

A broken `[[mesh_override]]` reports
`<entry> mesh_override: unknown model '<model>' (expected pl2b/pl9s/pla2/pla2_wig)`
or `<entry> mesh_override: unknown force '<force>' (expected show/hide)`.

A rule that loads but does nothing:

| Rule | Why |
|---|---|
| `levels` on an accessory, outfit, item or flag | `levels` only works on weapons. |
| `when.weapon_level` on anything but a weapon | It never matches. |
| `op = "mul"` on a switch path | Only `set` or `add` with a value other than `0` switches it on. |
| an outfit's rule after loading a save | Put the outfit on again. |

A mesh override that does nothing is covered under
[Mesh overrides](#mesh-overrides-mesh_override).

## Multipliers (use `op = "mul"`)

| Path | Controls |
|---|---|
| `player.gold_gain` | Gold multiplier |
| `player.move_speed` | Movement speed |
| `player.melee_out` | Melee damage you deal |
| `player.ranged_out` | Ranged damage you deal |
| `player.dodge.propulsion_9s.forward` | 9S forward-dodge speed |
| `player.dodge.propulsion_default.forward` | 2B / A2 forward-dodge speed |
| `player.dodge.propulsion_flight_suit.forward` | Flight-suit forward-dodge speed |

The three dodge paths act on the character you control.

## Additions (use `op = "add"`)

| Path | Controls |
|---|---|
| `player.melee_in_reduction` | Reduction of incoming melee damage |
| `player.ranged_in_reduction` | Reduction of incoming ranged damage |
| `player.pod_cooldown_reduction` | Pod cooldown reduction |
| `player.hacking_level_add` | Levels added to an enemy taken over by hacking, rounded to a whole number; the enemy's level stays within `0` to `99` |

## Switches (use `op = "set"`, `value = 1`)

| Path | Effect |
|---|---|
| `player.stagger_immune` | Stagger immunity |
| `player.electric_immune` | Electric immunity |
| `player.eyemask_hidden` | Hides the `Eyemask` mesh and the worn outfit's `eyemask_meshes` |
| `player.a2_wig_show` | A2's wig part acts as if the wig accessory were worn |
| `player.block_exp` | Blocks experience gain |
| `player.taunt_active` | Taunts enemies within 400 units of the player |
| `hud.minimap_hidden` | Sets the mini-map HUD chip of the player and the buddy to level `0` while on |
| `player.dodge.perfect_evade_disabled` | A dodge at the last moment does not start the Perfect Evade slow motion |
| `player.shortcut.vanilla_hidden` | The quick item menu shows only healing items from mods; base healing items stay hidden even when owned |

`player.eyemask_hidden` needs NAMS to assemble the model, like a mesh
override, see [Mesh overrides](#mesh-overrides-mesh_override).

## Special

| Path | Op | Notes |
|---|---|---|
| `player.dodge.particles.forward` | `set` | Particle id shown in addition on a forward dodge of the character you control |

## Per-level (weapons only)

Use `levels` instead of `value` for buffs that scale with the weapon's
upgrade level:

```toml
[effect.stagger_immune]
path = "player.stagger_immune"
op = "set"
levels = { 4 = 1 }
```

The keys are the upgrade levels `1` to `4`. At a level without a key,
the rule does nothing. On an accessory, outfit, item or flag, a rule
with `levels` never applies.

Set **either** `value` **or** `levels`, never both.

## Conditional (`when`)

Restrict the effect to a specific character, weapon level, or outfit:

```toml
[effect.stagger_immune]
path = "player.stagger_immune"
op = "set"
value = 1
when = { player = "9S" }
```

`when` keys (set as many as you need - all conditions must match):

### `player`
Restrict the effect to a specific character or to "anyone except":

- `"2B"`, `"9S"`, `"A2"` - only when playing as that character.
- `"not_2b"`, `"not_9s"`, `"not_a2"` - only when **not** playing that
  character.

The values are matched exactly.

### `weapon_level`
Integer 1-4. The effect only applies when the source weapon is at this
upgrade level. On anything but a weapon, a rule with `weapon_level`
never applies.

```toml
[effect.stagger_immune]
path = "player.stagger_immune"
op = "set"
value = 1
when = { weapon_level = 4 }
```

### `outfit_id`
Integer outfit id. The effect only applies when the character in
control wears the outfit with this `outfit_id`.

With several `when` keys, all of them must match:

```toml
[effect.stagger_immune]
path = "player.stagger_immune"
op = "set"
value = 1
when = { player = "2B", weapon_level = 4, outfit_id = 10001 }
```

This rule applies only when 2B wears outfit 10001 with this weapon at
level 4.

## Effects on items

An item carries the same `[effect.*]` and `[[effect_array]]` rules.
What a rule does depends on whether NAMS counts the item as used up:

| Item | Your rule |
|---|---|
| `id_type = 0` and `category` `0` to `3` | runs once when the item is used, then one item is taken away |
| any other | lasts as long as the player owns at least one |

```toml
# entities\item_8500.toml
name = "item_my_8500"
id_type = 0
category = 6

[effect.stagger_immune]
path = "player.stagger_immune"
op = "set"
value = 1
```

A used-up item only accepts rules on `player.flashlight.charge`, a
number NAMS keeps in the scenario variable whose `path` names it. The
rule changes that variable, capped at its `max`; without such a
variable nothing happens. An item the player keeps rejects that path.
NAMS rejects a rule that cannot work for its item, and the message says
which category would make it work:

- `item <id> rule '<rule>': the player uses this item up, so it can only change a number like '<path>' once - an effect that lasts needs an item the player keeps (category 4, 6 or 7)`
- `item <id> rule '<rule>': '<path>' is changed when an item is used, but the player cannot use this one - give it category 0, 1, 2 or 3`

On an item or a flag, a rule can also carry `phase` and `subphases`
([How to add a phase or a subphase](25_phases.md)). `phase` is the
phase as a number, the hex digits of its name (`p200` is `0x200`).
`subphases` lists subphase names. With `phase` set, the rule only
applies in that phase; with `subphases` set, only in one of those
subphases. Weapons, accessories and outfits ignore both.

### The particle a used item shows

A used-up item with rules shows a particle and can play a sound when it
is used. Set them in the item's own file:

| Field | Default | What it does |
|---|---|---|
| `[use_particle]` `id` | `0x1a4` | Particle id; `0` shows nothing |
| `[use_particle]` `target` | `"player"` | `"player"` or `"pod"` |
| `use_sound` | none | Name of a sound played on the player |

## Effects while a flag is on

A scenario flag ([Counters and switches for your quest](23_scenario_variables.md))
can carry rules too: `[scenario_flag.effect.<rule_name>]` tables or
`[[scenario_flag.effect_array]]` blocks under its `[[scenario_flag]]`
block. The rules last exactly as long as the flag is switched on.

A flag rejects `player.flashlight.charge` with
`scenario_flag '<flag>' rule '<rule>': '<path>' is a number a rule can only change once, so it cannot be held by a flag`.

## Mesh overrides (`[[mesh_override]]`)

Force a piece of the player's 3D model to be visible or hidden while
equipped. Each override is a separate `[[mesh_override]]` block:

```toml
[[mesh_override]]
model = "pl2b"
mesh = "Skirt"
force = "hide"
```

Overrides are applied when NAMS assembles the meshes of the character
you control, after its own mesh rules, so they win over them. NAMS does
not assemble a model from an outfit without mesh lists or from a plain
folder mod; on such a model, overrides do nothing. Overrides also need
`player_mesh_swaps` switched on in [nams.toml](13_nams_toml.md).

### `model` (required)
Which player model the override targets:

| Value | Character |
|---|---|
| `"pl2b"` or `"2B"` | 2B |
| `"pl9s"` or `"9S"` | 9S |
| `"pla2"` or `"A2"` | A2 |
| `"pla2_wig"` or `"a2_wig"` | A2's wig part |

The values are matched exactly. If your mod affects multiple
characters, ship one `[[mesh_override]]` block per model - they don't
auto-cross-apply.

### `mesh` (required)
The mesh name from the player model. A name the model does not have is
ignored. The base models' names are listed per character in
[the mesh name list](resources/vanilla_mesh_names.md); custom player
models can have any mesh names you want.

### `force` (required)
What to do with the named mesh:

- `"hide"` - force invisible.
- `"show"` - force visible.

### `outfit_id` (optional)
Restrict the override to a specific outfit. When set, the override
only applies when the player wears the outfit with that `outfit_id`.

Useful when one mod ships multiple outfits and each needs a different
mesh combination. Without `outfit_id`, the override applies whenever
the source is equipped (regardless of outfit).
