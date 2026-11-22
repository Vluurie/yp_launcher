# Resources

## Vanilla id catalogs

- [vanilla_item_ids.md](vanilla_item_ids.md) - all vanilla item ids and
  their human-readable names, grouped by category.
- [vanilla_weapon_ids.md](vanilla_weapon_ids.md) - vanilla weapon
  inventory ids (the filename ids, all inside `1000-1999`).
- [weapon_obj_ids.md](weapon_obj_ids.md) - weapon `obj_id` rules: the
  4 class ranges, the vanilla obj_id list, and the wpmask reservations.
- [vanilla_chip_ids.md](vanilla_chip_ids.md) - leveled chips (base ids
  occupying `base..=base+8`) and unique single-id chips.
- [vanilla_pod_program_ids.md](vanilla_pod_program_ids.md) - pod
  program ids (`skill_act_*` range).
- [vanilla_fish_ids.md](vanilla_fish_ids.md) - fish item ids
  (`fish_*` range).
- [vanilla_achievement_ids.md](vanilla_achievement_ids.md) - vanilla
  achievement ids (two non-contiguous ranges).
- [vanilla_shop_names.md](vanilla_shop_names.md) - the 33 string names
  you can use in `shop_item.shop_name` / `[[shops]].shop_name`.
- [vanilla_grid_ids.md](vanilla_grid_ids.md) - all vanilla map grid ids
  per `wd` layer and the `st` room files, for picking a free custom map
  id.

## Making assets

- [weapon_modelling.md](weapon_modelling.md) - full Blender walkthrough
  for turning a 3D model into a weapon `.dat` / `.dtt` pair: extracting
  the game files, swapping meshes, textures, materials, export.

## Visual / asset references

- [vanilla_mesh_names.md](vanilla_mesh_names.md) - the mesh node names
  of the vanilla player models, listed per character. Use them in an
  outfit's mesh lists and in the `mesh` field of a `[[mesh_override]]`
  block.
- [mesh_toggle_names.md](mesh_toggle_names.md) - the 62 world-mesh
  toggle indices and their names.
- [est_id_catalog.md](est_id_catalog.md) - EST particle effect ids you
  can use in `[effects].est_id_normal` / `[effects].est_id_berserk` on
  weapons.

## Effect rule paths

- [effect_paths.md](effect_paths.md) - the 19 valid `path` values for
  `[effect.<name>]` rules, grouped by op (multipliers / flags / dodge
  tuning).
