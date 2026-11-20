# Making a weapon model in Blender

You start from a vanilla weapon: extract it, swap your model into it,
give it textures and export it.

Credit: the NieR:Modding Discord server, based on MC's original guide.

## Tools you need

**Game files:** [F-SERVO](https://github.com/ArthurHeitmann/F-SERVO) -
extracts the CPK archives and handles the texture files (WTA/WTP/WTB)
as well.

**3D:** [Blender](https://www.blender.org/) 2.8 or newer, plus the
[NieR2Blender2NieR](https://github.com/WoefulWolf/NieR2Blender2NieR)
add-on (N2B2N).

**Textures:** Photoshop with a DDS plugin, or free alternatives
[Paint.NET](https://www.getpaint.net/),
[GIMP](https://www.gimp.org/) or [Photopea](https://photopea.com).
The game only accepts DXT1, DXT3 and DXT5, and every DDS plugin can
write those.

**Mask maps:** [Materialize](https://boundingboxsoftware.com/materialize/)
is the easiest, or [Texture Wizard](https://texturewiz.com/) online.

## 1. Extract the weapons

Open F-SERVO and use its CPK extract on your NieR:Automata `data`
folder. Weapons live in `data006` and `data016` - extract both.

**Extract somewhere outside the game directory**, into a separate
modding folder.

## 2. Import a vanilla weapon

Install the add-on: `Edit -> Preferences -> Add-ons -> Install`, pick
the N2B2N zip, enable it.

Then `File -> Import -> DTT File for Nier Automata` and open a weapon
from the extracted `wp` folder. This guide uses `wp0003.dtt`, the short
sword Faith, because it has a single mesh and no armature - the
simplest case.

A `nier2blender_extracted` folder appears next to your export folder.
That is where the `.dat` and `.dtt` contents get unpacked: `.dat` holds
physics and collisions, `.dtt` holds object data and textures.

The imported weapon looks white because of the viewport shading mode.
Switch between Wireframe, Solid, Material Preview and Rendered with the
spheres in the top right of the 3D view.

## 3. Line up your model

Import your own weapon model and delete the material it came with: open
Material Properties (the red sphere) and press the minus button.

Now scale and rotate your model until it matches the vanilla weapon.

| Key | Action |
|---|---|
| `S` | Scale |
| `R` | Rotate |
| `G` | Move |
| `G` then `X`/`Y`/`Z` | Move along one axis |
| Middle mouse | Orbit camera |
| Shift + middle mouse | Pan camera |
| Ctrl + Tab | Switch mode |

## 4. Make the textures

Each mesh can carry exactly one material, and it has to be a NieR
material. Whatever came with your model will not work - you reuse the
one from the imported vanilla weapon.

Your model probably has a colour texture already. You still need it as
DDS, plus a mask map, plus a normal map if it has none.

### Mask map

In Materialize, load your colour texture (called albedo or diffuse)
into the **Diffuse Map** slot first - everything else is derived from
it.

A mask map packs three different things into the three colour channels:

| Channel | Holds | Meaning |
|---|---|---|
| Red | Metallic | Black = matte, white = metallic reflections |
| Green | Smoothness | Lighter = smoother |
| Blue | Ambient occlusion | Mostly white with grey creases |

Create the metallic, smoothness and AO maps in that order, then assign
them to R, G and B, top to bottom.

Press **Save Property Map** and pick a format like PNG.

### Normal map

If your model already ships a normal map - an almost entirely purple
texture, often named with `N` or `NMR` - skip this.

Otherwise, in Materialize create the **Height Map** first, then the
**Normal Map**, and press `S` to save it.

In Photoshop you can instead use `Filter -> 3D -> Generate Normal Map`.
It handles height poorly, which rarely matters for weapons.

### Sizes and format

Textures must have power-of-two dimensions: 512, 1024, 2048. They do
not have to be square - 1024x2048 is fine.

Use 2048x2048 for albedo and mask if you want clarity, 1024x1024 for a
smaller mod. 1024x1024 is plenty for normal maps.

Save every map as DDS. With no transparency, use **BC1 4bpp Linear**
(also called DXT1) and **enable mipmaps**.

## 5. Swap your model into the NieR mesh

Select the vanilla mesh and enter Edit Mode (`Tab`). Press `A` to
select everything, press `Delete` and choose **Vertices**. The mesh is
now empty but still exists. Leave Edit Mode with `Tab`.

Select your weapon mesh, Ctrl-click the empty NieR mesh, then press
`Ctrl + J` to join yours into it.

Delete any leftover material so the NieR material is the only one on
the mesh.

Every mesh that exists must contain at least one triangle. If you do
not need a mesh, delete the mesh itself rather than emptying it.

After joining, do further size and position edits in **Edit Mode**. If
you change anything in Object Mode, apply the transforms with
`Ctrl + A` or `Object -> Apply -> All Transforms`.

The textures look broken at this point. That is expected - they are
replaced next.

## 6. Export

Open **Output Properties** and scroll to the N2B2N section.

Press **Fetch New Materials** to refresh the exporter's material list.

Now change the texture identifiers. Leaving them unchanged breaks other
mods that use the same identifier.

- Exactly 8 characters.
- Only `0-9` and `a-f`.
- Change only Albedo, Normal and Mask. Leave the rest alone.

Changing the last three characters is enough to make it unique - `123`,
`abc`, `12a` all work.

Then link your textures: click the file button next to each texture
path and pick your file. Afterwards press **Sync Identifiers in
Materials**, then **Sync Blender Materials**.

In the **Nier Automata Export** panel the `.dat` and `.dtt` folders are
already filled in. Leave **Base Name** as the original weapon name.

Pick an export directory. Every step in the export panel must be
highlighted blue. Press **One Click Export All**.

You now have a `.dat` / `.dtt` pair. Drop it into the weapon builder on
[the weapon page](../04_weapon.md) to make it a weapon of its own,
instead of one that overwrites the vanilla weapon you started from.

## Weapons with an armature

Same steps, plus weight painting. The meshes are parented to an
armature, meaning parts of the weapon move in game, and the bones form
a chain.

After joining your weapon into the NieR mesh:

1. Open the Object Data panel (the green triangle). Under **Vertex
   Groups** you will see entries named `bone` plus a number.
2. Select your weapon, click `bone0`, and press `Tab` for Edit Mode.
   Press `A` if nothing is selected.
3. Under the vertex groups there is a blue bar with the value `100`,
   and an **Assign** button under it. Press Assign.

## Weapons with several meshes

Some weapons are split so that different parts can use different
materials. The process is identical, repeated per mesh.

Do not need the extra mesh? Delete it.

Need it? Add a new mesh object, or select the NieR mesh and press
`Shift + D` to duplicate it. If you move it or create it away from the
scene origin, apply all transforms.

## More materials

To reuse a NieR material with different textures, duplicate it:

- **Material Properties tab:** press the Duplicate Material button (two
  overlapping sheets), then rename the copy.
- **Blender File view:** switch the top-right Display Mode from View
  Layer to Blender File, go to Materials, `Ctrl + C` / `Ctrl + V`, then
  rename to drop the `.001` suffix.

To bring in a *different* material - glowing, cloth - open a second
Blender instance, import a NieR model that has it, and copy the whole
mesh over into your project, then delete the mesh. The material stays.
`File -> Append` does the same thing more directly.

## Material properties

### Transparency

Export the DDS as **DXT5** with an alpha channel, then set
`0_xx_g_1BitMask` to `1` in the material's custom properties.

`xx` is a number that differs per model: `0_01_g_1BitMask`,
`0_05_g_1BitMask`. Sometimes the bitmask shows up as `Unknown` instead
- on A2's Hotshader material, `0_05_Unknown` is the bitmask and
`0_04_Unknown` is Decal. The unnumbered `g_1BitMask` further down only
lists which properties the material uses; it does nothing.

Bitmask is strict: a transparent area not surrounded by colour may not
register. Decal has no such requirement but disables shadows and
contact effects and wraps the object in a glass-like sheen. Prefer
bitmask.

### Emission (glow)

Machines and their red eyes use an emission material. You will not see
the glow in Blender - it comes from custom properties.

Import a machine enemy or a mod that uses it, and copy the material
over. It usually has bitmask enabled too, so set `0_01_g_1BitMask` to
`0` if you do not want transparency.

**Intensity** controls the glow from 0 to 10. Useful values are
roughly 1 to 5.

### Animated texture

The Demon's Cry weapon has a material whose texture scrolls vertically.
