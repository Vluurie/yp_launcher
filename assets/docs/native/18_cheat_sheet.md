# Cheat sheet

The folder layout and `mod.toml` are in
[how to make your first mod](01_first_mod.md) and
[how to lay out your mod folder](02_mod_folder_layout.md).

## Which file name for which kind

An entity file in `entities\` takes one of three forms:

- **`<prefix>_<id>.toml`** - one entity of a bundle kind, its id from
  the file name. A bundle kind holds an entity together with all of
  its parts, such as its texts.
- **List file** - a `.toml` name that does not start with one of the
  prefixes below and `_`. Every top-level `[<kind>]` table is one
  entity, every `[[<kind>]]` block one entry. The bundle kinds of the
  first table work here too, with `id` or `name` written inside.
- **`.json`** - a top-level `kind` field names the kind; an array holds
  entries of one kind. Bundle kinds need `id` or `name` written inside.

"On save" says whether NAMS reads the entity again when you save the
file while NieR:Automata runs. `.json` files always need a restart.

| File | Kind | On save | Page |
|---|---|---|---|
| `accessory_<id>.toml` | `accessory_bundle` | read again | [accessory](06_accessory.md) |
| `archive_<name>.toml` | `archive_bundle` | read again | [archive](41_archive.md) |
| `char_name_<name>.toml` | `char_name_bundle` | restart | [quest](07_quest.md) |
| `entity_<name>.toml` | `entity_bundle` | read again | [custom entities](34_custom_entities.md) |
| `item_<id>.toml` | `item_bundle` | read again | [item](03_item.md) |
| `mail_<id>.toml` | `mail_bundle` | read again | [mail](08_mail.md) |
| `map_<name>.toml` | `map_bundle` | restart | [custom maps](32_custom_maps.md) |
| `novel_<name>.toml` | `novel_bundle` | read again | [novel](40_novel.md) |
| `outfit_<id>.toml` | `outfit_bundle` | read again | [outfit](05_outfit.md) |
| `quest_<id>.toml` | `quest_bundle` | read again | [quest](07_quest.md) |
| `reward_<id>.toml` | `reward_bundle` | read again | [reward](09_reward.md) |
| `voice_bank_<name>.toml` | `voice_bank_bundle` | read again | [voice lines](10_voice_lines.md) |
| `weapon_<id>.toml` | `weapon_bundle` | read again | [weapon](04_weapon.md) |
| `weapon_patch_<id>.toml` | `weapon_patch_bundle` | read again | [weapon patch](30_weapon_patch.md) |

List-file kinds:

| Kind | On save | Page |
|---|---|---|
| `ambient` | read again | [ambient sounds](39_ambient_sounds.md) |
| `ambient_mute` | read again | [ambient sounds](39_ambient_sounds.md) |
| `effect_area` | read again | [effect areas](29_effect_areas.md) |
| `effect_area_mute` | read again | [effect areas](29_effect_areas.md) |
| `effect_override` | read again | [effect override](46_effect_override.md) |
| `effect_type_mute` | read again | [effect type mute](38_effect_type_mute.md) |
| `global_effect` | read again | [global effect](36_global_effect.md) |
| `graphic_adjust` | read again | [graphic adjust](26_graphic_adjust.md) |
| `hairspray` | read again | [hairspray](19_hairspray.md) |
| `hap_group` | read again | [HAP groups](31_hap_groups.md) |
| `hap_overlay` | read again | - |
| `hap_patch` | read again | - |
| `layout` | read again | [layout](37_layout.md) |
| `localized_string` | restart | - |
| `map` | restart | [custom maps](32_custom_maps.md) |
| `mesh_visibility` | read again | [mesh visibility](28_mesh_visibility.md) |
| `music` | read again | [custom music](27_custom_music.md) |
| `phase` | restart | [phases](25_phases.md) |
| `player_camera_settings` | read again | - |
| `ruby_patch` | restart | [Ruby patches](24_ruby_patches.md) |
| `scenario_flag` | restart | [scenario variables](23_scenario_variables.md) |
| `scenario_variable` | restart | [scenario variables](23_scenario_variables.md) |
| `shop` | restart | - |
| `shop_item` | read again | [item](03_item.md) |
| `sound` | restart | [sound effects](44_sound_effects.md) |
| `ui_override` | read again | [UI override](45_ui_override.md) |

The single-row kinds below are what the bundles above are made of.
All of them need a restart:
`accessory`, `accessory_effect`, `accessory_mesh`, `auto_give`,
`item`, `item_icon`, `item_text`, `mail`, `mail_text`, `outfit`,
`outfit_effect`, `outfit_mesh`, `quest`, `quest_fail_text`,
`quest_forbidden_state`, `quest_required_quest`,
`quest_required_state`, `quest_reward`, `quest_run_text`,
`quest_stage_condition`, `quest_text`, `quest_voice_bank`, `reward`,
`reward_item`, `voice_bank`, `voice_clip`, `weapon`, `weapon_effect`,
`weapon_param`, `weapon_param_level`, `weapon_story_texture`,
`weapon_text`, `weapon_upgrade`, `weapon_upgrade_material`.

## Item

File: `item_9999.toml`

```toml
name = "item_test_pebble"
id_type = 0
buy = -1
sell = -1
no_trash = true

[auto_give]
quantity = 1

[text.name]
_us = "Test Pebble"

[text.help]
_us = "A small stone. Proof your mod loaded."
```

## Weapon

File: `weapon_<id>.toml`, with an id from 1000 to 1999.

```toml
obj_id = 0x30100
category = 0
[item]
name = "..."
buy = 5000
[text.name]
_us = "..."
[params]
clone_from_obj_id = 0x30003
[effects]
est_id_normal = 442
est_id_berserk = 442
[[shops]]
shop_name = "shop_RC_Smith"
state = 0
```

## Existing weapon patch

File: `weapon_patch_1070.toml`

```toml
[effects]
est_id_normal = 590
est_id_berserk = 633
```

No weapon definition or `data\wp` files are required.

## Mesh override

In an outfit, accessory, weapon or weapon patch file:

```toml
[[mesh_override]]
model = "pl2b"
mesh = "Skirt"
force = "hide"
```

## Stat buff

An `[effect.<name>]` table in an item, weapon, weapon patch, outfit or
accessory file with `path`, `op` (`mul`, `add` or `set`), and either
`value` or `levels`, see [How to make special effects](11_special_effects.md).

## Disable a mod

`nams\disabled_mods.toml`:

```toml
disabled = ["mods/my_first_mod"]
```
