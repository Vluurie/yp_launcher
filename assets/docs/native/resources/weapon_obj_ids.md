# Weapon obj_ids

A weapon's `obj_id` is a **separate** number from its inventory id and
tells the engine which weapon class the 3D model and combat behaviour
belong to. `BehaviorWeapon::vOnSpawn` does a hard range-check on the
`obj_id` to decide which animation/combo/wpmask setup to apply, so
every custom weapon's `obj_id` must:

1. Fall inside one of the four vanilla ranges.
2. Not already be used by a vanilla weapon.
3. Not be one of the engine-internal `wpmask` ids.

If your `obj_id` falls outside these ranges the weapon spawns but
never gets combat setup - it floats and does nothing.


## Weapon class ranges

| `obj_id` range | Weapon class | `category` value |
|---|---|---|
| `0x30000` - `0x301FF` | Small Sword | `0` |
| `0x30200` - `0x303FF` | Large Sword | `1` |
| `0x30400` - `0x305FF` | Spear | `2` |
| `0x30600` - `0x307FF` | Combat Bracer | `3` |

The `category` field in your weapon TOML must agree with the range
your `obj_id` falls into - NAMS rejects the weapon at boot if they
disagree. Each range has ~512 ids; subtracting vanilla and wpmask ids
leaves at least ~400 free slots per category.

## Taken / unavailable

### Vanilla weapon obj_ids

These are extracted from `WeaponInfoTable.bxm.xml` and are already
owned by a shipped weapon. Reusing one means your weapon shares the
exact 3D model and animations of the vanilla weapon at that slot.

#### Small swords

`0x30003`, `0x30013`, `0x30020`, `0x30030`, `0x30040`, `0x30050`,
`0x30060`, `0x30070`, `0x30071`, `0x30075`, `0x30076`, `0x30080`,
`0x30090`

#### Large swords

`0x30203`, `0x30213`, `0x30220`, `0x30230`, `0x30240`, `0x30250`,
`0x30260`, `0x30261`, `0x30270`

#### Spears

`0x30400`, `0x30420`, `0x30430`, `0x30440`, `0x30450`, `0x30460`,
`0x30470`, `0x30471`, `0x30480`

#### Combat bracers

`0x30600`, `0x30610`, `0x30620`, `0x30621`, `0x30630`, `0x30640`,
`0x30650`, `0x30655`

### `wpmask` obj_ids - DO NOT USE

`wpmask` entities are engine-internal helpers spawned during
`vOnSpawn`. **Never** use one as a player-weapon `obj_id` - it will
crash or corrupt the weapon.

| Obj id | Internal name | Class |
|---|---|---|
| `0x30072` | wp0072 | Small sword mask |
| `0x30081` | wp0081 | Small sword mask (unused in XML, engine-reserved) |
| `0x30222` | wp0222 | Large sword mask |
| `0x30432` | wp0432 | Spear mask |
| `0x30602` | wp0602 | Combat bracer mask |

## Picking a free slot

A safe rule: pick an `obj_id` ending in `00` or `10` inside the right
range and well above the vanilla numbers. For a custom small sword:
`0x30100`, `0x30110`, `0x30120` are all good. Large sword: `0x30300`,
`0x30310`. Spear: `0x30500`, `0x30510`. Bracer: `0x30700`, `0x30710`.
