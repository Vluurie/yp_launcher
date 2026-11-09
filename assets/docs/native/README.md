# NAMS content authoring guide

For normal authoring, use the bundle-style files such as
`item_8500.toml`, `weapon_1500.toml`, and `quest_500.toml`.

## Getting started

- [How to make your first mod in 5 minutes](01_first_mod.md)
- [How to lay out your mod folder](02_mod_folder_layout.md)

## Per-entity walkthroughs

- [How to make an item](03_item.md)
- [How to make a weapon](04_weapon.md)
- [How to patch an existing weapon](30_weapon_patch.md)
- [How to make an outfit](05_outfit.md)
- [How to make an accessory](06_accessory.md)
- [How to make a hairspray](19_hairspray.md)
- [How to make a quest](07_quest.md)
- [Counters and switches for your quest](23_scenario_variables.md)
- [Patching vanilla scripts (NPC dialogue, quest hooks)](24_ruby_patches.md)
- [How to make a mail](08_mail.md)
- [How to make a reward](09_reward.md)
- [How to make voice lines](10_voice_lines.md)
- [How to make a novel](40_novel.md)
- [How to make an archive](41_archive.md)
- [How to show your enemy in Unit Data](42_intel.md)
- [How to add a phase or a subphase](25_phases.md)
- [How to make a graphic adjust area (lighting, fog, color)](26_graphic_adjust.md)
- [How to add your own music](27_custom_music.md)
- [How to add a background sound or silence an existing one](39_ambient_sounds.md)
- [How to add your own sound effect](44_sound_effects.md)
- [How to hide or show parts of the world](28_mesh_visibility.md)
- [How to put effects in a room](29_effect_areas.md)
- [How to play an effect on the player](36_global_effect.md)
- [How to switch an effect type off](38_effect_type_mute.md)
- [How to change one of the effects](46_effect_override.md)
- [How to switch vanilla content off in your own phase](31_hap_groups.md)
- [How to add to the HAP content of a room, phase or quest](47_hap_overlay.md)
- [How to make a custom map](32_custom_maps.md)
- [How to remove objects from the world](37_layout.md)
- [How to add a new game object (entity)](34_custom_entities.md)
- [How to change a menu or HUD layout](45_ui_override.md)
- [How to change the player camera](48_camera_settings.md)

## Cross-cutting topics

- [How to make special effects (buffs, immunities, mesh visibility)](11_special_effects.md)
- [How to translate your mod](12_translate.md)
- [How to configure NAMS (`nams.toml`)](13_nams_toml.md)
- [How to replace textures (packs, mesh picker, hot reload)](33_texture_injection.md)
- [How to turn a mod off without deleting it](14_disabled_mods.md)
- [ReShade, 3DMigoto and DLL mods](35_third_party.md)
- [Load order, default outfits, and reload behavior](20_runtime_configuration.md)
- [The NAMS save file](43_nams_save_file.md)

## When things break

- [How to debug when something goes wrong](17_debugging.md)
- [Cheat sheet](18_cheat_sheet.md)

## Reference data

See the [resources](resources/README.md) folder for vanilla id lists,
shop names, mesh names, EST particle catalog, and the full effect
path registry.
