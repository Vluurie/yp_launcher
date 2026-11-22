# Effect registry paths

The `path` of an `[effect.<name>]` rule has to be one of the values
below. If you typo a path, NAMS rejects the rule with
`unknown path '...'` at boot.


Registry version: `(0, 1)`.

For full usage rules (ops, `value` vs `levels`, conditional `when`
predicates) see [11_special_effects.md](../11_special_effects.md).

## Stat multipliers (`op = "mul"`)

Multiply the underlying engine value. Use values like `1.5` for +50%,
`0.8` for -20%.

| Path | Description | Default op |
|---|---|---|
| `player.gold_gain` | Money from kills / quests | `mul` |
| `player.exp_gain` | Experience gained | `mul` |
| `player.move_speed` | Walk / run speed | `mul` |
| `player.melee_out` | Melee damage you deal | `mul` |
| `player.ranged_out` | Ranged damage you deal | `mul` |
| `player.melee_in_reduction` | Reduce incoming melee damage (additive 0.0–1.0) | `add` |
| `player.ranged_in_reduction` | Reduce incoming ranged damage (additive 0.0–1.0) | `add` |
| `player.pod_cooldown_reduction` | Reduce pod ability cooldown (additive 0.0–1.0) | `add` |
| `player.hacking_level_add` | 9S hacking level bonus (additive integer) | `add` |

The first five paths are pure multipliers; the last four are listed
under "additions" in the tutorial because they're values added to a
base, but they live in the same multiplier-style registry. Use the op
shown in the third column.

## Flag toggles (`op = "set"`, `value = 1.0` for on, `0.0` for off)

Boolean-style player state flags. The rule forces the value to
exactly what you set.

| Path | Description | Default op |
|---|---|---|
| `player.stagger_immune` | Cannot be staggered | `set` |
| `player.electric_immune` | Immune to electric damage | `set` |
| `player.eyemask_hidden` | Hide eye mask | `set` |
| `player.a2_wig_show` | Show A2 wig | `set` |
| `player.block_exp` | Block experience gain | `set` |
| `player.taunt_active` | Taunt mode | `set` |

## Dodge tuning

Values that tune the forward-dodge propulsion / particle behaviour for
each character class.

| Path | Description | Default op |
|---|---|---|
| `player.dodge.propulsion_9s.forward` | 9S forward-dash speed | `mul` |
| `player.dodge.propulsion_default.forward` | 2B / A2 forward-dash speed | `mul` |
| `player.dodge.propulsion_flight_suit.forward` | Flight-suit dash speed | `mul` |
| `player.dodge.particles.forward` | Particle id used during forward dash | `set` |
