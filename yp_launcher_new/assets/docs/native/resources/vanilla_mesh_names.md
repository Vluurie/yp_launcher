# Vanilla mesh node names

The mesh node names the vanilla player models use. These are the names
you put in an outfit's mesh lists, and in the `mesh` field of a
`[[mesh_override]]` block.

Names are **case sensitive** and must match exactly.

Each character has its own model with its own mesh names. A name that
exists on 9S does not necessarily exist on 2B.

## 2B

| Mesh | Used for |
|---|---|
| `Body` | Body, base outfit |
| `Hair` | Hair |
| `Skirt` | Skirt, hidden on self-destruct |
| `Feather` | Feather |
| `Broken` | Damaged body (end of route A) |
| `Eyemask` | Blindfold |
| `Eyelash` | Eyelashes, shown when the blindfold is off |
| `facial_normal` | Face out of combat |
| `facial_serious` | Face in combat |
| `Armor_Body` | Body armour outfit |
| `Armor_Head` | Head armour, full armour outfit only |
| `DLC_Body` | Body, DLC outfit |
| `DLC_Skirt` | Skirt, DLC outfit |
| `DLC_Broken` | Damaged body, DLC outfit |

## 9S

| Mesh | Used for |
|---|---|
| `Body` | Body |
| `Hair` | Hair |
| `Pants` | Pants, hidden on self-destruct |
| `Leg` | Leg |
| `Eyemask` | Blindfold |
| `Eyelash` | Eyelashes, shown when the blindfold is off |
| `facial_normal` | Face out of combat |
| `facial_serious` | Face in combat |
| `mesh_pl0200` | Normal story state |
| `mesh_es0200` | Prologue, heavily damaged |
| `mesh_es0201` | Holy state |
| `mesh_es0202` | Tower state |
| `mesh_es0206` | Missing right arm |

The DLC outfit uses the same set prefixed with `DLC_`: `DLC_Body`,
`DLC_Pants`, `DLC_Leg`, `DLC_mesh_pl0200`, `DLC_mesh_es0200`,
`DLC_mesh_es0201`, `DLC_mesh_es0202`, `DLC_mesh_es0206`. `Hair`,
`Eyemask`, `Eyelash` and the two facial meshes are shared, not
prefixed.

## A2

| Mesh | Used for |
|---|---|
| `Body` | Body |
| `Hair` | Long hair |
| `Cloth` | Cloth, hidden on self-destruct |
| `facial_normal` | Face out of combat |
| `facial_serious` | Face in combat |
| `DLC_Body` | Body, DLC outfit |
| `DLC_Cloth` | Cloth, DLC outfit |

A2 uses no `Eyemask` and no `Eyelash` mesh. Her face still switches
between `facial_normal` and `facial_serious` like the others.

A2's hair works backwards from what you would expect: the long `Hair`
mesh is visible by default and gets **hidden** when the wig accessory
is not equipped. That is what `wig_meshes` lists - vanilla sets it to
`["Hair"]`.

## Other

`Clothing_Sit` is used by seated NPCs, not by the player models.

## Custom models

A custom model can use any mesh names it likes. Names outside the lists
above are accepted and work normally.

## Finding the names in a model

**1. In Blender.** Import the `.wmb` with
[NieR2Blender2NieR](https://github.com/WoefulWolf/NieR2Blender2NieR)
and read the names off the imported objects.

**2. In the launcher.** Drop the model's `.dat` in as a mod and open it
in the 3D model viewer.

**3. In game, live.** Load the mod and open the YP devkit:

- **Entity Manager**
- **Hover Select Entity** in the top left toolbar
- pick your played character
- **Entity Inspector View** → **Meshes**

You get the real list of that model's meshes and can toggle each one on
and off while the game runs, which is the fastest way to work out which
name is which part.
