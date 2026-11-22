# How to make an accessory

`entities\accessory_<id>.toml`. An accessory is an item the player puts
on in the accessory slot. With a model, NAMS spawns that model for the
character while the accessory is worn; without one, the accessory acts
only through its mesh overrides and effects. A character can wear
several accessories at once.

```toml
# entities\accessory_<id>.toml
item_id = 8500

[[mesh_override]]
model = "pl2b"
mesh  = "Eyemask"
force = "hide"
```

This accessory has no model. Using item `8500` from the inventory puts
it on, and while 2B wears it, her `Eyemask` mesh (the blindfold) is
hidden. Using the item again takes it off. The item itself is an
ordinary item file, see [How to make an item](03_item.md).

## When it does not work

**The accessory is missing, and the log says**
`[NAMS] Skipping accessory_bundle '<id>' from mod '<mod>' because the item '<item_id>' it depends on (from mod '<unstaged>') is not loaded.`
No loaded mod provides `item_<item_id>.toml`, and `item_id` is not a
base item. Ship the item file.

**`[NAMS] Two mods try to use the same accessory_bundle number '<id>'.`**
Two mods use the same accessory id. Change the id of the mod named
after `Skipped:`.

**`[NAMS] Register failed for accessory_bundle from mod '<mod>': accessory <id> rule '<rule>': <reason>`**
A rule or a `[[mesh_override]]` is broken. See
[§11 How to make special effects](11_special_effects.md).

**The model does not appear.** `filename` is not a name NAMS can read
as an object name, or the files are not at `data\et\<filename>.dat` and
`data\et\<filename>.dtt`.

**A `[[mesh_override]]` does nothing.** The character wears an outfit
without mesh lists, see
[How to make an outfit](05_outfit.md#what-happens-without-mesh-lists).

## Fields

### `id` (auto from filename)

The accessory's id, from `accessory_<id>.toml`. Unique across all mods.
Ids 1 to 19 are the base accessories, and 20 is used as a placeholder
while one is spawned, so pick a higher number.

### `item_id` (required)

The inventory item that puts the accessory on and, used again, takes it
off. Ship a matching `item_<item_id>.toml` (any loaded mod may provide
it). If no loaded mod provides it and it is not a base item, the
accessory is skipped, see above.

### `filename` (optional)

Name of the model file under `data\et\`, without extension: `ete`
followed by three hexadecimal digits. The model goes to
`data\et\<filename>.dat` and `data\et\<filename>.dtt`. `ete000` to
`ete00c` are the base accessory models, do not reuse those.

NAMS registers the object for you, copied from `ete000`. The log shows
it as
`[CustomEntity] 0x<object id> -> <class> (from 0x4e000, lane <lane>, class template)`.
An [entity file](34_custom_entities.md) for the same object takes its
place.

Leave it out for an accessory without a model: nothing is spawned, and
only the mesh overrides and effects below do something. A `filename`
NAMS cannot read as an object name counts as no model.

### `meshes_2b`, `meshes_9s`, `meshes_a2` (optional)

Which meshes of your model are shown when that character wears the
accessory. All other meshes of the model are hidden. When the list for
the wearing character is empty, NAMS shows the model's first mesh for
2B and A2 and its second mesh for 9S.

Mesh names are the names of the meshes inside your model; a name the
model does not have is ignored. See
[Finding the names in a model](resources/vanilla_mesh_names.md#finding-the-names-in-a-model).

## Effects and mesh overrides

Accessories support the same `[effect.*]`, `[[effect_array]]` and
`[[mesh_override]]` blocks as outfits and weapons. See
[§11 How to make special effects](11_special_effects.md). They switch
on when the accessory is put on and off when it is taken off. A save
keeps the accessories each character wears; loading it switches the
effects of the character in control on again, and the others' when
control switches to them.

To make your model replace the character's own hair, hide `Hair` with a
`[[mesh_override]]` and show your model's hair through `meshes_2b`,
`meshes_9s` or `meshes_a2`.

`Eyemask` and `Hair` are vanilla mesh names, see
[Vanilla mesh node names](resources/vanilla_mesh_names.md).

## Changing the file while NAMS runs

Saving the file registers the accessory again. Put it on again to see
the change. A rule you delete from the file stays active until the next
start.
