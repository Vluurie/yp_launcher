# How to configure NAMS (`nams.toml`)

`nams\nams.toml` is created on first launch: the launcher writes it
when it is missing, and so does NAMS at startup. A field left out of
the file counts as its default below, except `skip_startup_logos`,
which counts as `true` when left out.

| Field | Default | Notes |
|---|---|---|
| `disable_content_features` | `false` | Master switch. `true` = every `[content]` field below counts as `false`, and entity files are no longer read again on save. Restart. |
| `disable_texture_injection` | `false` | Loads no replacement textures. Restart. |
| `fix_wind_timer_bug` | `true` | Keeps wind animations moving after the play-time counter reaches its maximum. Live. |
| `disable_debug_hotkeys` | `true` | Ignores the leftover dev hotkeys Shift+3/6/K/L, which reset the render scale and size the render targets for 1600x900. Live. |
| `ingame_settings` | `true` | Adds the NAMS tab and the NAMS rows to the in-game options menu. `false` keeps the vanilla options menu. Restart. |
| `validate_model_data` | `false` | Shows the result of the model data check as a dialog while you play. Live. |
| `validate_scripts` | `false` | Shows script errors as a dialog. |
| `loading_stall_hints` | `true` | Shows hints when the "Loading Map" screen takes long. Live. |
| `disable_plugin_loading` | `false` | Loads no plugin DLLs, the ones named in a mod's `mod.toml` included. Restart. |
| `disable_main_overlay` | `false` | Does not start the main in-game overlay. Restart. |
| `disable_input_features` | `false` | Installs no mouse and keyboard hooks. Restart. |
| `experimental_default_outfits` | `true` in the launcher's file, left out (`false`) in the file NAMS writes | Turns on `default_mods.toml`, see [Load order, default outfits, and reload behavior](20_runtime_configuration.md). Restart. |
| `outfit_swap_visual_effects` | `true` | Plays the visual effects of an outfit change. `false` swaps without them. Live. |
| `disable_reshade_loading` | `false` | Does not load ReShade from `nams\thirdparty\reshade\`. Restart. |
| `disable_3dmigoto_loading` | `false` | Does not load 3DMigoto from `nams\thirdparty\3dmigoto\`. Restart. |
| `disable_graphic_mods_loading` | `false` | Does not load the DLLs in `nams\thirdparty\graphic\`. Restart. |
| `disable_game_mods_loading` | `false` | Does not load the DLLs in `nams\thirdparty\game\`. Restart. |
| `disable_splash_screen` | `false` | Shows no startup splash window. Restart. |
| `skip_startup_logos` | `false` | Skips the logo movies at startup. Restart. |

`nams\thirdparty\` is in your NieRAutomata folder, see
[ReShade, 3DMigoto and DLL mods](35_third_party.md).

`[content]` - every content feature on its own, all `true` by default.
`false` skips the mod entries of that feature and every mod entry that
needs them, e.g. a shop entry selling a modded outfit while
`custom_outfits` is `false`. Restart needed. Without a `[content]` table
the old `content_*` keys still apply.

| Field | What it covers |
|---|---|
| `custom_items` | new items with their icons and texts, items handed out automatically |
| `custom_weapons` | new weapons with their stats, upgrades, texts and story images |
| `weapon_patches` | changes to existing weapons |
| `custom_outfits` | new outfits, the "Default Outfit" item |
| `custom_hairsprays` | new hair sprays |
| `accessories` | new accessories, wearing several accessories at once |
| `player_mesh_swaps` | NAMS puts 2B, 9S and A2 together itself, so outfit and accessory mods can swap, hide or add model parts |
| `mesh_visibility` | hiding or showing single parts of a model |
| `shop_items` | items mods add to shops |
| `item_shortcut_menu` | modded healing items in the quick-item menu |
| `effect_rules` | what modded weapons, outfits, accessories and items do to the player |
| `outfit_selector_npc` | the outfit selection NPC for modded outfits |
| `buddy_outfit_menu` | the "Change outfit" entry when talking to your buddy (experimental) |
| `custom_quests` | new quests with their texts, rewards and conditions |
| `custom_mails` | new mails |
| `custom_voice_lines` | new voice lines |
| `custom_maps` | new map rooms and story phases, their Chapter Select entries and save slot names |
| `map_layouts` | rules for objects placed on the map |
| `novels` | new novels in the Data menu |
| `archives` | new archives in the Data menu |
| `cutscene_replacements` | replaced cutscene videos, moved in-game cutscenes |
| `camera_settings` | player camera settings |
| `script_overlays` | additions and changes to scripts |
| `script_groups` | turning groups of scripts on or off per story phase |
| `ruby_patches` | edits to Ruby scripts |
| `custom_music` | new background music |
| `ambient_sounds` | ambient sounds and crowd voices mods add or mute |
| `custom_sounds` | new sound effects |
| `effect_areas` | effects placed in or removed from rooms |
| `effect_type_mute` | muting whole effect types per room or story phase |
| `global_effects` | effects across a whole room or story phase |
| `lighting_filters` | color and lighting filters for rooms |
| `cloth_physics` | extra cloth physics parts on modded models |
| `custom_texts` | mod texts in menus |
| `ui_overrides` | menu and HUD layouts mods replace |

`[cutscene]` - all `false` by default. All but `enable_h264` are live.

| Field | What it does |
|---|---|
| `enable_h264` | Turns on H264 playback for cutscene videos. Read at startup. A NAMS built without H264 support logs `[CRI] H264 support not available (built without h264 feature)` instead. |
| `hide_subtitle_overlay` | Hides the subtitle overlay during cutscene movies. |
| `keep_subtitle_text` | With `hide_subtitle_overlay`: keeps the caption text and hides only the backdrop behind it. |
| `hide_subtitle_in_events` | With `hide_subtitle_overlay` or `solid_letterbox_bars`: applies during event scenes too, not only movies. |
| `solid_letterbox_bars` | Draws the letterbox as solid black bars. Wins over `hide_subtitle_overlay`. |

Mouse and keyboard settings are in `nams\naiom.toml`. A `[mouse]` table
in `nams.toml` is read only while `naiom.toml` does not exist; NAMS
then writes `naiom.toml` from it.

The file hot-reloads - saving it re-applies the fields marked Live.
A syntax error on save logs
`nams.toml hot-reload parse error (kept previous config): <message>`
and keeps the previous settings. A syntax error at startup shows a
**NAMS - Configuration Error** box and NAMS starts with the defaults.
