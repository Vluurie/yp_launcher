use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use yp_builders::weapon::{EFFECT_PATHS, EffectOp};
use yp_builders::{MeshRule, MeshVisibility, OutfitCharacter, WeaponType, item, outfit, weapon};
use yp_core::GamePaths;
use yp_formats::testing::{cube_spec, dds_dxt1_solid, model_archives};

use super::forms::*;

pub(crate) fn write_model(dir: &Path, stem: &str, meshes: &[&str]) -> PathBuf {
    std::fs::create_dir_all(dir).expect("model dir");
    let archives = model_archives(
        stem,
        cube_spec(meshes).build(),
        &[(0xC0BE, dds_dxt1_solid(8, 8, 0xF800))],
    );
    let dat = dir.join(format!("{stem}.dat"));
    std::fs::write(&dat, archives.dat).expect("dat");
    std::fs::write(dir.join(format!("{stem}.dtt")), archives.dtt).expect("dtt");
    dat
}

pub(crate) fn write_png(path: &Path, width: u32, height: u32) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("image dir");
    }
    image::RgbaImage::from_pixel(width, height, image::Rgba([200, 120, 40, 255]))
        .save(path)
        .expect("png");
}

fn game(dir: &tempfile::TempDir) -> GamePaths {
    GamePaths::new(dir.path().join("game"))
}

#[test]
fn thumbnail_sides_snap_to_four_inside_the_allowed_range() {
    assert_eq!(snap_to_four(9), 8);
    assert_eq!(snap_to_four(10), 12);
    assert_eq!(snap_to_four(127), 128);
    assert_eq!(clamp_side(-50), 8);
    assert_eq!(clamp_side(3), 8);
    assert_eq!(clamp_side(257), 256);
    assert_eq!(clamp_side(5000), 1024);
    assert_eq!(preview_scale(256, 128), 1.);
    assert_eq!(preview_scale(840, 560), 0.5);
    assert_eq!(preview_scale(100, 1024), 420. / 1024.);
}

#[test]
fn text_filters_keep_digits_and_texture_names_only() {
    assert_eq!(digits_only(" 12a-3 "), "123");
    assert_eq!(texture_name_only("item thumb-85!00_x"), "itemthumb8500_x");
    assert!(is_texture_name("item_thumb_8500"));
    assert!(!is_texture_name(""));
    assert!(!is_texture_name("a b"));
    assert_eq!(step_value(1, -1, 1, 99), 1);
    assert_eq!(step_value(98, 5, 1, 99), 99);
    assert_eq!(step_value(100, 50, 1, 99_999), 150);
}

#[test]
fn a_fresh_item_form_matches_the_flutter_defaults() {
    let form = ItemForm::default();
    assert_eq!(form.mod_id(), DEFAULT_ITEM_MOD);
    assert_eq!(form.id, item::FIRST_FREE_ID);
    assert_eq!(form.carry, 1);
    assert!(form.auto_give);
    assert!(!form.sellable);
    assert_eq!(form.misctex_name(), None);
    assert_eq!(form.problem(false), Some(ItemProblem::NeedsGameDir));
    assert_eq!(form.problem(true), Some(ItemProblem::NeedsName));
    let toml = item::to_toml(&form.draft());
    assert!(toml.contains("name = \"item_unnamed\""));
    assert!(toml.contains("no_trash = true"));
    assert!(toml.contains("[auto_give]"));
}

#[test]
fn item_fields_flow_into_the_draft() {
    let mut form = ItemForm {
        name: "  Rusty Bolt ".into(),
        description: " A bolt. ".into(),
        mod_id: "   ".into(),
        carry: 5,
        sellable: true,
        sell_price: 250,
        auto_give: false,
        ..ItemForm::default()
    };
    assert_eq!(form.problem(true), None);
    assert_eq!(form.mod_id(), DEFAULT_ITEM_MOD);
    assert!(form.set_id_text(" 8123 "));
    assert!(!form.set_id_text("12x"));
    assert!(!form.set_id_text(""));
    assert_eq!(form.id, 8123);
    form.icon = Some(ThumbnailChoice {
        image_path: "icon.png".into(),
        width: 256,
        height: 128,
    });
    assert_eq!(form.misctex_name().as_deref(), Some("item_thumb_8123"));
    assert_eq!(form.icon_subtitle(), "misctex_item_thumb_8123.dat / .dtt");
    let draft = form.draft();
    assert_eq!(draft.display_name, "Rusty Bolt");
    assert_eq!(draft.description, "A bolt.");
    assert_eq!(item::file_name_for(&draft), "item_8123.toml");
    let toml = item::to_toml(&draft);
    assert!(toml.contains("name = \"item_rusty_bolt\""));
    assert!(toml.contains("max_num = 5"));
    assert!(toml.contains("sell = 250"));
    assert!(!toml.contains("[auto_give]"));
    assert!(toml.contains("misctex_name = \"item_thumb_8123\""));

    form.taken_ids = BTreeSet::from([8123]);
    assert!(form.id_taken());
    form.apply_ids(ItemIds {
        taken: BTreeSet::from([8000, 8001]),
        next: 8002,
    });
    assert_eq!(form.id, 8002);
    assert!(!form.id_taken());
}

#[test]
fn creating_an_item_writes_the_file_its_icon_and_refuses_a_taken_id() {
    let dir = tempfile::tempdir().expect("temp dir");
    let game = game(&dir);
    let icon = dir.path().join("src").join("icon.png");
    write_png(&icon, 64, 32);
    let mut form = ItemForm {
        name: "Pebble".into(),
        description: "Small.".into(),
        mod_id: "pebbles".into(),
        icon: Some(ThumbnailChoice {
            image_path: icon,
            width: 64,
            height: 32,
        }),
        ..ItemForm::default()
    };
    form.apply_ids(item_ids(&game));
    assert_eq!(form.id, item::FIRST_FREE_ID);

    let written = create_item(&game, &form).expect("item");
    assert_eq!(written, item::path_for(&game, "pebbles", &form.draft()));
    assert!(std::fs::read_to_string(&written)
        .expect("toml")
        .contains("_us = \"Pebble\""));
    assert!(yp_builders::thumbnail::exists(&game, "pebbles", form.id));

    match create_item(&game, &form) {
        Err(CreateFailure::IdTaken(taken)) => assert!(taken.contains(&form.id)),
        other => panic!("expected a taken id, got {other:?}"),
    }
    assert_eq!(item_ids(&game).next, item::FIRST_FREE_ID + 1);

    let broken = ItemForm {
        id: 8500,
        icon: Some(ThumbnailChoice {
            image_path: dir.path().join("missing.png"),
            width: 64,
            height: 32,
        }),
        ..form.clone()
    };
    assert!(matches!(
        create_item(&game, &broken),
        Err(CreateFailure::Build(yp_builders::Error::Format(_)))
    ));
    assert!(!item::path_for(&game, "pebbles", &broken.draft()).exists());
}

#[test]
fn outfit_models_are_checked_and_set_the_character() {
    let mut form = OutfitForm::default();
    assert_eq!(
        form.accept_model(Path::new("C:/mods/pl000d.png")),
        Err(ModelProblem::NotAModelFile)
    );
    assert_eq!(
        form.accept_model(Path::new("C:/mods/em0000.dat")),
        Err(ModelProblem::UnknownModel)
    );
    assert_eq!(form.model_path, None);
    form.set_rule(
        "Old",
        MeshRule {
            visibility: MeshVisibility::Never,
            ..MeshRule::default()
        },
    );
    assert_eq!(
        form.accept_model(Path::new("C:/mods/PL020D.DTT")),
        Ok(OutfitCharacter::NineS)
    );
    assert_eq!(form.character, OutfitCharacter::NineS);
    assert_eq!(form.model_stem().as_deref(), Some("PL020D"));
    assert!(form.mesh_rules.is_empty());
    assert_eq!(form.problem(false), Some(OutfitProblem::NeedsGameDir));
    assert_eq!(form.problem(true), Some(OutfitProblem::NeedsName));
    form.name = "Dress".into();
    assert_eq!(form.problem(true), None);
    form.model_path = None;
    assert_eq!(form.problem(true), Some(OutfitProblem::NeedsModel));
}

#[test]
fn mesh_rules_follow_the_model_and_shape_the_outfit_file() {
    let mut form = OutfitForm {
        name: "Black Dress".into(),
        ..OutfitForm::default()
    };
    form.set_mesh_names(&["Body".into(), "Skirt".into(), "Mask".into()]);
    assert_eq!(form.mesh_rules.len(), 3);
    assert!(!outfit::to_toml(&form.draft()).contains("meshes"));

    form.set_rule(
        "Skirt",
        MeshRule {
            visibility: MeshVisibility::OutOfCombatOnly,
            hide_on_self_destruct: true,
            ..MeshRule::default()
        },
    );
    form.set_rule(
        "Mask",
        MeshRule {
            visibility: MeshVisibility::EyemaskOnly,
            ..MeshRule::default()
        },
    );
    assert_eq!(form.rule("Skirt").visibility, MeshVisibility::OutOfCombatOnly);
    assert_eq!(form.rule("Unknown"), MeshRule::default());
    let toml = outfit::to_toml(&form.draft());
    assert!(toml.contains("meshes = [\"Body\"]"));
    assert!(toml.contains("non_combat_meshes = [\"Skirt\"]"));
    assert!(toml.contains("eyemask_meshes = [\"Mask\"]"));
    assert!(toml.contains("self_destruct_hide_meshes = [\"Skirt\"]"));

    form.set_mesh_names(&["Skirt".into(), "Cape".into()]);
    assert_eq!(
        form.mesh_rules
            .iter()
            .map(|(name, rule)| (name.as_str(), rule.is_default()))
            .collect::<Vec<_>>(),
        vec![("Skirt", false), ("Cape", true)]
    );
}

#[test]
fn shop_states_toggle_one_by_one_and_all_at_once() {
    let mut shop = ShopChoice::default();
    assert_eq!(shop.name(), None);
    assert_eq!(shop.states, BTreeSet::from([0]));
    shop.toggle_state(3);
    shop.toggle_state(0);
    assert_eq!(shop.states, BTreeSet::from([3]));
    assert!(!shop.all_states_selected());
    shop.toggle_all_states();
    assert!(shop.all_states_selected());
    assert_eq!(shop.states.len(), 7);
    shop.toggle_all_states();
    assert_eq!(shop.states, BTreeSet::from([0]));
    shop.enabled = true;
    assert_eq!(shop.name().as_deref(), Some(outfit::SHOP_NAMES[0]));
}

#[test]
fn creating_an_outfit_writes_outfit_item_shop_and_model() {
    let dir = tempfile::tempdir().expect("temp dir");
    let game = game(&dir);
    let model = write_model(&dir.path().join("src"), "pl000d", &["Body", "Skirt"]);
    let mut form = OutfitForm {
        name: "Black Dress".into(),
        description: "Formal.".into(),
        ..OutfitForm::default()
    };
    assert_eq!(form.accept_model(&model), Ok(OutfitCharacter::TwoB));
    form.apply_ids(outfit_ids(&game, form.character));
    assert_eq!(form.outfit_id, OutfitCharacter::TwoB.first_outfit_id());
    form.shop.enabled = true;
    form.shop.toggle_state(2);
    form.shop.buy_price = 7500;

    let item_toml = item::to_toml(&form.item_draft());
    assert!(item_toml.contains("category = 6"));
    assert!(item_toml.contains("buy = 7500"));

    let written = create_outfit(&game, &form).expect("outfit");
    let draft = form.draft();
    assert_eq!(written, outfit::path_for(&game, DEFAULT_OUTFIT_MOD, &draft));
    let entities = item::entities_dir(&game, DEFAULT_OUTFIT_MOD);
    assert!(entities.join(item::file_name_for(&form.item_draft())).is_file());
    let shop = std::fs::read_to_string(entities.join(outfit::shop_file_name_for(&draft)))
        .expect("shop file");
    assert_eq!(shop.matches("[[shop_item]]").count(), 2);
    let model_dir = outfit::model_dir_for(&game, DEFAULT_OUTFIT_MOD);
    assert!(model_dir.join("pl000d.dat").is_file());
    assert!(model_dir.join("pl000d.dtt").is_file());

    match create_outfit(&game, &form) {
        Err(CreateFailure::IdTaken(taken)) => assert!(taken.contains(&form.outfit_id)),
        other => panic!("expected a taken id, got {other:?}"),
    }
    let next = outfit_ids(&game, OutfitCharacter::TwoB);
    assert_eq!(next.outfit_id, form.outfit_id + 1);
    assert_eq!(
        outfit_ids(&game, OutfitCharacter::A2).outfit_id,
        OutfitCharacter::A2.first_outfit_id()
    );

    let no_copy = OutfitForm {
        mod_id: "plain".into(),
        copy_model: false,
        shop: ShopChoice::default(),
        outfit_id: next.outfit_id,
        item_id: next.item_id,
        ..form.clone()
    };
    create_outfit(&game, &no_copy).expect("second outfit");
    assert!(!outfit::model_dir_for(&game, "plain").exists());
    assert!(!item::entities_dir(&game, "plain")
        .join(outfit::shop_file_name_for(&no_copy.draft()))
        .exists());
}

#[test]
fn weapon_levels_grow_step_and_never_cross() {
    let mut form = WeaponForm::default();
    assert_eq!(form.levels.len(), 1);
    form.add_level();
    assert_eq!(
        (form.levels[1].min_damage, form.levels[1].max_damage),
        (140, 196)
    );
    form.add_level();
    form.add_level();
    assert!(!form.can_add_level());
    form.add_level();
    assert_eq!(form.levels.len(), weapon::MAX_LEVEL as usize);

    form.step_min_damage(0, true);
    form.step_min_damage(0, true);
    form.step_min_damage(0, true);
    form.step_min_damage(0, true);
    form.step_min_damage(0, true);
    assert_eq!((form.levels[0].min_damage, form.levels[0].max_damage), (150, 150));
    form.step_max_damage(0, false);
    assert_eq!(form.levels[0].max_damage, 150);
    form.step_max_damage(0, true);
    assert_eq!(form.levels[0].max_damage, 160);
    form.set_level(0, 1, 5);
    form.step_min_damage(0, false);
    assert_eq!(form.levels[0].min_damage, 1);

    form.remove_level(1);
    assert_eq!(form.levels.len(), 3);
    form.remove_level(9);
    form.remove_level(0);
    form.remove_level(0);
    form.remove_level(0);
    assert_eq!(form.levels.len(), 1);
}

#[test]
fn weapon_upgrades_are_numbered_from_level_two() {
    let mut form = WeaponForm::default();
    form.add_upgrade();
    form.add_upgrade();
    form.add_upgrade();
    assert!(!form.can_add_upgrade());
    form.add_upgrade();
    assert_eq!(
        form.upgrades
            .iter()
            .map(|upgrade| (upgrade.lv, upgrade.cost))
            .collect::<Vec<_>>(),
        vec![(2, 2500), (3, 5000), (4, 7500)]
    );
    form.step_upgrade_cost(0, false);
    form.step_upgrade_cost(0, false);
    form.step_upgrade_cost(0, false);
    form.step_upgrade_cost(0, false);
    form.step_upgrade_cost(0, false);
    assert_eq!(form.upgrades[0].cost, 500);
    form.step_upgrade_cost(0, true);
    assert_eq!(form.upgrades[0].cost, 1000);
    form.remove_last_upgrade();
    assert_eq!(form.upgrades.len(), 2);
}

#[test]
fn weapon_effects_pick_free_paths_and_keep_their_player_filter() {
    let mut form = WeaponForm::default();
    form.add_level();
    assert!(form.add_effect());
    assert!(form.add_effect());
    assert_eq!(form.effects[0].path, EFFECT_PATHS[0]);
    assert_eq!(form.effects[1].path, EFFECT_PATHS[1]);
    assert_eq!(
        form.effects[0].levels.iter().map(|(level, value)| (*level, *value)).collect::<Vec<_>>(),
        vec![(1, 1.1), (2, 2.2)]
    );

    let choices = form.effect_choices(0);
    assert!(choices.contains(&EFFECT_PATHS[0]));
    assert!(!choices.contains(&EFFECT_PATHS[1]));
    assert_eq!(choices.len(), EFFECT_PATHS.len() - 1);
    assert_eq!(form.free_effect_paths().len(), EFFECT_PATHS.len() - 2);

    form.set_effect_when(0, Some("9S".into()));
    let additive = EFFECT_PATHS
        .into_iter()
        .find(|path| path.op == EffectOp::Add && path.per_level)
        .expect("additive path");
    form.set_effect_path(0, additive);
    assert_eq!(form.effects[0].when_player.as_deref(), Some("9S"));
    assert_eq!(form.effects[0].levels[&2], 0.2);
    form.step_effect_level(0, 1, false);
    form.step_effect_level(0, 1, false);
    assert_eq!(form.effects[0].levels[&1], 0.);
    form.step_effect_level(0, 2, true);
    assert_eq!(form.effects[0].levels[&2], 0.3);

    let flag = EFFECT_PATHS
        .into_iter()
        .find(|path| !path.per_level)
        .expect("flag path");
    form.set_effect_path(1, flag);
    assert!(form.effects[1].levels.is_empty());
    assert_eq!(form.effects[1].value, 1.0);
    form.set_effect_on(1, false);
    assert_eq!(form.effects[1].value, 0.0);
    form.set_effect_when(0, Some(String::new()));
    assert_eq!(form.effects[0].when_player, None);

    let toml = weapon::to_toml(&form.draft());
    assert!(toml.contains(&format!("path = \"{}\"", additive.path)));
    form.remove_effect(0);
    form.remove_effect(5);
    assert_eq!(form.effects.len(), 1);

    while form.add_effect() {}
    assert_eq!(form.effects.len(), EFFECT_PATHS.len());
    assert!(form.free_effect_paths().is_empty());
}

#[test]
fn trail_effects_skip_the_blocked_ids_and_stay_in_range() {
    let mut form = WeaponForm::default();
    assert_eq!(form.est_id(false), WeaponType::SmallSword.est_id_normal());
    form.est_id_normal = Some(434);
    form.step_est_id(false, true);
    assert_eq!(form.est_id(false), 437);
    form.step_est_id(false, false);
    assert_eq!(form.est_id(false), 434);
    form.est_id_berserk = Some(weapon::MAX_EST_ID);
    form.step_est_id(true, true);
    assert_eq!(form.est_id(true), weapon::MAX_EST_ID);
    form.est_id_berserk = Some(weapon::MIN_EST_ID);
    form.step_est_id(true, false);
    assert_eq!(form.est_id(true), weapon::MIN_EST_ID);
}

#[test]
fn weapon_ids_follow_the_mod_folder_and_the_model() {
    let dir = tempfile::tempdir().expect("temp dir");
    let game = game(&dir);
    let mut form = WeaponForm::default();
    assert_eq!(form.problem(false), Some(WeaponProblem::NeedsGameDir));
    assert_eq!(form.problem(true), Some(WeaponProblem::NeedsName));
    form.name = "Virtuous Blade".into();
    assert_eq!(form.problem(true), Some(WeaponProblem::NeedsModel));

    let spear = write_model(&dir.path().join("src"), "wp0400", &["Blade"]);
    assert_eq!(
        inspect_weapon_model(&dir.path().join("src").join("pl0000.dat")),
        Err(ModelProblem::NotAModelFile)
    );
    std::fs::write(dir.path().join("src").join("wp0003.dat"), b"x").expect("lonely dat");
    assert_eq!(
        inspect_weapon_model(&dir.path().join("src").join("wp0003.dat")),
        Err(ModelProblem::NeedsBothFiles)
    );
    let source = inspect_weapon_model(&spear).expect("weapon model");
    form.est_id_normal = Some(5);
    form.accept_model(source);
    assert_eq!(form.weapon_type, WeaponType::Spear);
    assert_eq!(form.est_id_normal, None);
    assert_eq!(form.problem(true), Some(WeaponProblem::NoFreeSlot));

    form.apply_ids(weapon_ids(&game, form.seed()));
    assert_eq!(form.problem(true), None);
    let first = form.obj_id.expect("obj id");
    assert_eq!(weapon::type_from_obj_id(first), Some(WeaponType::Spear));
    assert!(form.obj_id_label().expect("label").starts_with("0x304")
        || form.obj_id_label().expect("label").starts_with("0x305"));

    form.mod_id = "another_mod".into();
    form.apply_ids(weapon_ids(&game, form.seed()));
    assert_ne!(form.obj_id, Some(first));
    assert_eq!(form.icon_texture_name(), format!("wpi_{}", form.id.expect("id")));
}

#[test]
fn creating_a_weapon_converts_the_model_and_writes_icons_and_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    let game = game(&dir);
    let model = write_model(&dir.path().join("src"), "wp0003", &["Blade"]);
    let icon = dir.path().join("src").join("icon.png");
    let story = dir.path().join("src").join("story.png");
    write_png(&icon, 64, 64);
    write_png(&story, 800, 560);
    let mut form = WeaponForm {
        name: "Virtuous Blade".into(),
        description: "Sharp.".into(),
        story: " Once upon a time. ".into(),
        skill1_name: "Slash".into(),
        ..WeaponForm::default()
    };
    form.accept_model(inspect_weapon_model(&model).expect("model"));
    form.apply_ids(weapon_ids(&game, form.seed()));
    form.icon = Some(ThumbnailChoice {
        image_path: icon,
        width: 64,
        height: 64,
    });
    form.story_image = Some(ThumbnailChoice {
        image_path: story,
        width: 800,
        height: 560,
    });
    form.shop.enabled = true;

    let draft = form.draft();
    assert_eq!(draft.story, vec!["Once upon a time.".to_owned()]);
    assert_eq!(draft.story_texture_suffix, form.id);
    assert_eq!(draft.icon_name.as_deref(), Some(form.icon_texture_name().as_str()));

    let written = create_weapon(&game, &form).expect("weapon");
    assert_eq!(written, weapon::path_for(&game, DEFAULT_WEAPON_MOD, &draft));
    let stem = yp_builders::weapon_convert::stem_for_obj_id(draft.obj_id);
    let models = yp_builders::weapon_convert::model_dir_for(&game, DEFAULT_WEAPON_MOD);
    assert!(models.join(format!("{stem}.dat")).is_file());
    assert!(models.join(format!("{stem}.dtt")).is_file());
    let textures = yp_builders::thumbnail::output_dir_for(&game, DEFAULT_WEAPON_MOD);
    assert!(textures
        .join(format!("misctex_{}.dat", form.icon_texture_name()))
        .is_file());
    assert!(textures
        .join(format!("misctex_{}.dtt", form.story_texture_name()))
        .is_file());

    let refreshed = weapon_ids(&game, form.seed());
    assert!(refreshed.taken_obj_ids.contains(&draft.obj_id));
    assert_ne!(refreshed.id, form.id);
}

#[test]
fn the_converter_offers_another_slot_and_wraps_around() {
    let dir = tempfile::tempdir().expect("temp dir");
    let model = write_model(&dir.path().join("src"), "wp0603", &["Fist"]);
    let source = inspect_weapon_model(&model).expect("model");
    let mut form = ConvertForm::default();
    assert_eq!(form.target_label(), None);
    form.next_slot();
    assert_eq!(form.target_obj_id, None);

    form.accept(source, BTreeSet::new());
    let first = form.target_obj_id.expect("slot");
    assert_eq!(weapon::type_from_obj_id(first), Some(WeaponType::CombatBracer));
    assert!(weapon::is_obj_id_available(first));
    form.next_slot();
    let second = form.target_obj_id.expect("slot");
    assert_ne!(first, second);
    assert!(form.target_label().expect("label").contains(&form.target_stem().expect("stem")));

    form.set_mod_id("other");
    assert!(form.skipped.is_empty());
    assert_eq!(form.mod_id(), "other");
    form.set_mod_id("  ");
    assert_eq!(form.mod_id(), DEFAULT_WEAPON_MOD);

    let mut seen = BTreeSet::new();
    for _ in 0..600 {
        seen.insert(form.target_obj_id.expect("slot"));
        form.next_slot();
    }
    assert!(seen.len() > 400);
    assert!(form.target_obj_id.is_some());
}
