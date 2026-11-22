use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use yp_builders::outfit::{self, MAX_SHOP_STATE, OUTFIT_ITEM_CATEGORY, SHOP_NAMES};
use yp_builders::weapon::{
    self, EFFECT_PATHS, EffectOp, EffectPath, MAX_EST_ID, MAX_LEVEL, MIN_EST_ID, WeaponEffect,
    WeaponLevel, WeaponUpgrade,
};
use yp_builders::{
    ItemDraft, MeshRule, OutfitCharacter, OutfitDraft, WeaponDraft, WeaponSource, WeaponType,
    item, thumbnail, weapon_convert,
};
use yp_core::GamePaths;

pub const DEFAULT_ITEM_MOD: &str = "my_mod";
pub const DEFAULT_OUTFIT_MOD: &str = "my_outfit";
pub const DEFAULT_WEAPON_MOD: &str = "my_weapon";

pub const THUMBNAIL_MIN_SIDE: i64 = 8;
pub const THUMBNAIL_MAX_SIDE: i64 = 1024;
pub const THUMBNAIL_PREVIEW_SIDE: f32 = 420.;

const DAMAGE_STEP: i64 = 10;
const LEVEL_GROWTH: f64 = 1.4;
const UPGRADE_COST_STEP: i64 = 2500;
const UPGRADE_MIN_COST: i64 = 500;
const EFFECT_STEP: f64 = 0.1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailChoice {
    pub image_path: PathBuf,
    pub width: u32,
    pub height: u32,
}

pub fn snap_to_four(value: i64) -> i64 {
    (value + 2).div_euclid(4) * 4
}

pub fn clamp_side(value: i64) -> u32 {
    let snapped = snap_to_four(value.clamp(THUMBNAIL_MIN_SIDE, THUMBNAIL_MAX_SIDE));
    u32::try_from(snapped).unwrap_or(THUMBNAIL_MIN_SIDE as u32)
}

pub fn preview_scale(width: u32, height: u32) -> f32 {
    let longest = width.max(height) as f32;
    if longest <= THUMBNAIL_PREVIEW_SIDE {
        1.
    } else {
        THUMBNAIL_PREVIEW_SIDE / longest
    }
}

pub fn step_value(value: i64, delta: i64, min: i64, max: i64) -> i64 {
    (value + delta).clamp(min, max)
}

pub fn digits_only(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

pub fn is_texture_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

pub fn texture_name_only(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect()
}

fn or_default<'a>(text: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

fn round_cents(value: f64) -> f64 {
    (value * 100.).round() / 100.
}

#[derive(Debug)]
pub enum CreateFailure {
    IdTaken(BTreeSet<i64>),
    Build(yp_builders::Error),
}

impl From<yp_builders::Error> for CreateFailure {
    fn from(error: yp_builders::Error) -> Self {
        CreateFailure::Build(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemProblem {
    NeedsGameDir,
    NeedsName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemForm {
    pub name: String,
    pub description: String,
    pub mod_id: String,
    pub id: i64,
    pub carry: i64,
    pub sellable: bool,
    pub sell_price: i64,
    pub auto_give: bool,
    pub icon: Option<ThumbnailChoice>,
    pub taken_ids: BTreeSet<i64>,
}

impl Default for ItemForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            mod_id: DEFAULT_ITEM_MOD.to_owned(),
            id: item::FIRST_FREE_ID,
            carry: 1,
            sellable: false,
            sell_price: 100,
            auto_give: true,
            icon: None,
            taken_ids: BTreeSet::new(),
        }
    }
}

impl ItemForm {
    pub const CARRY_RANGE: (i64, i64) = (1, 99);
    pub const SELL_RANGE: (i64, i64) = (1, 99_999);
    pub const SELL_STEP: i64 = 50;

    pub fn id_taken(&self) -> bool {
        self.taken_ids.contains(&self.id)
    }

    pub fn mod_id(&self) -> &str {
        or_default(&self.mod_id, DEFAULT_ITEM_MOD)
    }

    pub fn misctex_name(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|_| thumbnail::texture_name_for(self.id))
    }

    pub fn icon_subtitle(&self) -> String {
        format!("{}.dat / .dtt", thumbnail::misctex_name_for(self.id))
    }

    pub fn draft(&self) -> ItemDraft {
        ItemDraft {
            max_carry: self.carry,
            sellable: self.sellable,
            sell_price: self.sell_price,
            auto_give: self.auto_give,
            misctex_name: self.misctex_name(),
            ..ItemDraft::new(self.name.trim(), self.description.trim(), self.id)
        }
    }

    pub fn set_id_text(&mut self, text: &str) -> bool {
        match text.trim().parse::<i64>() {
            Ok(id) => {
                self.id = id;
                true
            }
            Err(_) => false,
        }
    }

    pub fn apply_ids(&mut self, ids: ItemIds) {
        self.taken_ids = ids.taken;
        self.id = ids.next;
    }

    pub fn problem(&self, has_game_dir: bool) -> Option<ItemProblem> {
        if !has_game_dir {
            Some(ItemProblem::NeedsGameDir)
        } else if self.name.trim().is_empty() {
            Some(ItemProblem::NeedsName)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemIds {
    pub taken: BTreeSet<i64>,
    pub next: i64,
}

pub fn item_ids(game: &GamePaths) -> ItemIds {
    ItemIds {
        taken: item::taken_ids(game),
        next: item::next_free_id(game),
    }
}

pub fn create_item(game: &GamePaths, form: &ItemForm) -> Result<PathBuf, CreateFailure> {
    let taken = item::taken_ids(game);
    if taken.contains(&form.id) {
        return Err(CreateFailure::IdTaken(taken));
    }
    let mod_id = form.mod_id();
    if let Some(icon) = &form.icon {
        thumbnail::create(
            &icon.image_path,
            game,
            mod_id,
            form.id,
            icon.width,
            icon.height,
        )?;
    }
    Ok(item::write(game, mod_id, &form.draft())?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelProblem {
    NotAModelFile,
    UnknownModel,
    NeedsBothFiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutfitProblem {
    NeedsGameDir,
    NeedsName,
    NeedsModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopChoice {
    pub enabled: bool,
    pub shop_name: String,
    pub states: BTreeSet<i64>,
    pub buy_price: i64,
}

impl Default for ShopChoice {
    fn default() -> Self {
        Self {
            enabled: false,
            shop_name: SHOP_NAMES[0].to_owned(),
            states: BTreeSet::from([0]),
            buy_price: 5000,
        }
    }
}

impl ShopChoice {
    pub const PRICE_STEP: i64 = 500;
    pub const PRICE_RANGE: (i64, i64) = (1, 999_999);

    pub fn name(&self) -> Option<String> {
        self.enabled.then(|| self.shop_name.clone())
    }

    pub fn toggle_state(&mut self, state: i64) {
        if !self.states.remove(&state) {
            self.states.insert(state);
        }
    }

    pub fn all_states_selected(&self) -> bool {
        (0..=MAX_SHOP_STATE).all(|state| self.states.contains(&state))
    }

    pub fn toggle_all_states(&mut self) {
        if self.all_states_selected() {
            self.states = BTreeSet::from([0]);
        } else {
            self.states = (0..=MAX_SHOP_STATE).collect();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitIds {
    pub taken: BTreeSet<i64>,
    pub outfit_id: i64,
    pub item_id: i64,
}

pub fn outfit_ids(game: &GamePaths, character: OutfitCharacter) -> OutfitIds {
    OutfitIds {
        taken: outfit::taken_outfit_ids(game, character),
        outfit_id: outfit::next_free_outfit_id(game, character),
        item_id: item::next_free_id(game),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitForm {
    pub name: String,
    pub description: String,
    pub mod_id: String,
    pub character: OutfitCharacter,
    pub outfit_id: i64,
    pub item_id: i64,
    pub model_path: Option<PathBuf>,
    pub copy_model: bool,
    pub auto_give: bool,
    pub shop: ShopChoice,
    pub mesh_rules: Vec<(String, MeshRule)>,
    pub taken_outfit_ids: BTreeSet<i64>,
}

impl Default for OutfitForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            mod_id: DEFAULT_OUTFIT_MOD.to_owned(),
            character: OutfitCharacter::TwoB,
            outfit_id: OutfitCharacter::TwoB.first_outfit_id(),
            item_id: item::FIRST_FREE_ID,
            model_path: None,
            copy_model: true,
            auto_give: true,
            shop: ShopChoice::default(),
            mesh_rules: Vec::new(),
            taken_outfit_ids: BTreeSet::new(),
        }
    }
}

impl OutfitForm {
    pub fn id_taken(&self) -> bool {
        self.taken_outfit_ids.contains(&self.outfit_id)
    }

    pub fn mod_id(&self) -> &str {
        or_default(&self.mod_id, DEFAULT_OUTFIT_MOD)
    }

    pub fn model_stem(&self) -> Option<String> {
        self.model_path.as_deref().and_then(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
    }

    pub fn accept_model(&mut self, path: &Path) -> Result<OutfitCharacter, ModelProblem> {
        if !outfit::is_model_file(path) {
            return Err(ModelProblem::NotAModelFile);
        }
        let character =
            outfit::character_from_model_path(path).ok_or(ModelProblem::UnknownModel)?;
        self.model_path = Some(path.to_path_buf());
        self.mesh_rules.clear();
        self.character = character;
        Ok(character)
    }

    pub fn apply_ids(&mut self, ids: OutfitIds) {
        self.taken_outfit_ids = ids.taken;
        self.outfit_id = ids.outfit_id;
        self.item_id = ids.item_id;
    }

    pub fn set_mesh_names(&mut self, names: &[String]) {
        let previous = std::mem::take(&mut self.mesh_rules);
        self.mesh_rules = names
            .iter()
            .map(|name| {
                let rule = previous
                    .iter()
                    .find(|(known, _)| known == name)
                    .map(|(_, rule)| *rule)
                    .unwrap_or_default();
                (name.clone(), rule)
            })
            .collect();
    }

    pub fn rule(&self, name: &str) -> MeshRule {
        self.mesh_rules
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, rule)| *rule)
            .unwrap_or_default()
    }

    pub fn set_rule(&mut self, name: &str, rule: MeshRule) {
        match self.mesh_rules.iter_mut().find(|(known, _)| known == name) {
            Some(entry) => entry.1 = rule,
            None => self.mesh_rules.push((name.to_owned(), rule)),
        }
    }

    pub fn draft(&self) -> OutfitDraft {
        let rules: Vec<(&str, MeshRule)> = self
            .mesh_rules
            .iter()
            .map(|(name, rule)| (name.as_str(), *rule))
            .collect();
        OutfitDraft {
            model_source_path: self.model_path.clone(),
            meshes: outfit::lists_from_rules(self.character, &rules),
            auto_give: self.auto_give,
            shop_name: self.shop.name(),
            shop_states: self.shop.states.clone(),
            buy_price: self.shop.buy_price,
            ..OutfitDraft::new(
                self.name.trim(),
                self.description.trim(),
                self.character,
                self.outfit_id,
                self.item_id,
            )
        }
    }

    pub fn item_draft(&self) -> ItemDraft {
        ItemDraft {
            auto_give: self.auto_give,
            buy_price: self.shop.enabled.then_some(self.shop.buy_price),
            category: OUTFIT_ITEM_CATEGORY,
            ..ItemDraft::new(self.name.trim(), self.description.trim(), self.item_id)
        }
    }

    pub fn problem(&self, has_game_dir: bool) -> Option<OutfitProblem> {
        if !has_game_dir {
            Some(OutfitProblem::NeedsGameDir)
        } else if self.name.trim().is_empty() {
            Some(OutfitProblem::NeedsName)
        } else if self.model_path.is_none() {
            Some(OutfitProblem::NeedsModel)
        } else {
            None
        }
    }
}

pub fn create_outfit(game: &GamePaths, form: &OutfitForm) -> Result<PathBuf, CreateFailure> {
    let taken = outfit::taken_outfit_ids(game, form.character);
    if taken.contains(&form.outfit_id) {
        return Err(CreateFailure::IdTaken(taken));
    }
    let mod_id = form.mod_id();
    let draft = form.draft();
    item::write(game, mod_id, &form.item_draft())?;
    let file = outfit::write(game, mod_id, &draft)?;
    if form.shop.enabled {
        outfit::write_shop_item(game, mod_id, &draft)?;
    }
    if form.copy_model
        && let Some(model) = &form.model_path
    {
        outfit::copy_model(game, mod_id, model)?;
    }
    Ok(file)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponProblem {
    NeedsGameDir,
    NeedsName,
    NeedsModel,
    NoFreeSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponIds {
    pub taken_obj_ids: BTreeSet<i64>,
    pub id: Option<i64>,
}

pub fn weapon_ids(game: &GamePaths, seed: &str) -> WeaponIds {
    WeaponIds {
        taken_obj_ids: weapon::taken_obj_ids(game),
        id: weapon::next_free_id(game, Some(seed)),
    }
}

pub fn inspect_weapon_model(path: &Path) -> Result<WeaponSource, ModelProblem> {
    if !weapon_convert::is_weapon_file(path) {
        return Err(ModelProblem::NotAModelFile);
    }
    weapon_convert::inspect(path).ok_or(ModelProblem::NeedsBothFiles)
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponForm {
    pub name: String,
    pub description: String,
    pub mod_id: String,
    pub help_short: String,
    pub skill1_name: String,
    pub skill1_desc: String,
    pub skill2_name: String,
    pub skill2_desc: String,
    pub story: String,
    pub upgrades: Vec<WeaponUpgrade>,
    pub effects: Vec<WeaponEffect>,
    pub trait_id: Option<i64>,
    pub est_id_normal: Option<i64>,
    pub est_id_berserk: Option<i64>,
    pub icon: Option<ThumbnailChoice>,
    pub story_image: Option<ThumbnailChoice>,
    pub weapon_type: WeaponType,
    pub id: Option<i64>,
    pub obj_id: Option<i64>,
    pub sell_price: i64,
    pub shop: ShopChoice,
    pub levels: Vec<WeaponLevel>,
    pub taken_obj_ids: BTreeSet<i64>,
    pub model: Option<WeaponSource>,
}

impl Default for WeaponForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            mod_id: DEFAULT_WEAPON_MOD.to_owned(),
            help_short: String::new(),
            skill1_name: String::new(),
            skill1_desc: String::new(),
            skill2_name: String::new(),
            skill2_desc: String::new(),
            story: String::new(),
            upgrades: Vec::new(),
            effects: Vec::new(),
            trait_id: None,
            est_id_normal: None,
            est_id_berserk: None,
            icon: None,
            story_image: None,
            weapon_type: WeaponType::SmallSword,
            id: None,
            obj_id: None,
            sell_price: 500,
            shop: ShopChoice::default(),
            levels: vec![WeaponLevel {
                min_damage: 100,
                max_damage: 140,
            }],
            taken_obj_ids: BTreeSet::new(),
            model: None,
        }
    }
}

impl WeaponForm {
    pub fn mod_id(&self) -> &str {
        or_default(&self.mod_id, DEFAULT_WEAPON_MOD)
    }

    pub fn seed(&self) -> &str {
        self.mod_id.trim()
    }

    fn effective_id(&self) -> i64 {
        self.id.unwrap_or(weapon::FIRST_ID)
    }

    pub fn icon_texture_name(&self) -> String {
        weapon::icon_texture_name_for(self.effective_id())
    }

    pub fn story_texture_name(&self) -> String {
        weapon::story_texture_name_for(self.effective_id())
    }

    pub fn obj_id_label(&self) -> Option<String> {
        self.obj_id.map(|obj_id| format!("0x{obj_id:X}"))
    }

    pub fn draft(&self) -> WeaponDraft {
        let story = self.story.trim();
        WeaponDraft {
            sell_price: self.sell_price,
            levels: self.levels.clone(),
            shop_name: self.shop.name(),
            shop_states: self.shop.states.clone(),
            buy_price: self.shop.buy_price,
            icon_name: self.icon.as_ref().map(|_| self.icon_texture_name()),
            story_texture_suffix: self.story_image.as_ref().and(self.id),
            help_short: self.help_short.trim().to_owned(),
            skill1_name: self.skill1_name.trim().to_owned(),
            skill1_desc: self.skill1_desc.trim().to_owned(),
            skill2_name: self.skill2_name.trim().to_owned(),
            skill2_desc: self.skill2_desc.trim().to_owned(),
            story: if story.is_empty() {
                Vec::new()
            } else {
                vec![story.to_owned()]
            },
            upgrades: self.upgrades.clone(),
            trait_id: self.trait_id,
            effects: self.effects.clone(),
            est_id_normal: self.est_id_normal,
            est_id_berserk: self.est_id_berserk,
            ..WeaponDraft::new(
                self.name.trim(),
                self.description.trim(),
                self.weapon_type,
                self.effective_id(),
                self.obj_id.unwrap_or(self.weapon_type.obj_id_start()),
            )
        }
    }

    pub fn apply_ids(&mut self, ids: WeaponIds) {
        self.taken_obj_ids = ids.taken_obj_ids;
        self.id = ids.id;
        self.allocate_obj_id();
    }

    fn allocate_obj_id(&mut self) {
        self.obj_id =
            weapon::allocate_obj_id(self.weapon_type, &self.taken_obj_ids, Some(self.seed()));
    }

    pub fn accept_model(&mut self, source: WeaponSource) {
        self.weapon_type = source.weapon_type;
        self.model = Some(source);
        self.est_id_normal = None;
        self.est_id_berserk = None;
        self.allocate_obj_id();
    }

    pub fn can_add_level(&self) -> bool {
        (self.levels.len() as i64) < MAX_LEVEL
    }

    pub fn add_level(&mut self) {
        if !self.can_add_level() {
            return;
        }
        let Some(last) = self.levels.last().copied() else {
            return;
        };
        self.levels.push(WeaponLevel {
            min_damage: (last.min_damage as f64 * LEVEL_GROWTH).round() as i64,
            max_damage: (last.max_damage as f64 * LEVEL_GROWTH).round() as i64,
        });
    }

    pub fn set_level(&mut self, index: usize, min_damage: i64, max_damage: i64) {
        if let Some(level) = self.levels.get_mut(index) {
            *level = WeaponLevel {
                min_damage,
                max_damage: max_damage.max(min_damage),
            };
        }
    }

    pub fn step_min_damage(&mut self, index: usize, up: bool) {
        let Some(level) = self.levels.get(index).copied() else {
            return;
        };
        if !up && level.min_damage <= 1 {
            return;
        }
        let delta = if up { DAMAGE_STEP } else { -DAMAGE_STEP };
        self.set_level(index, level.min_damage + delta, level.max_damage);
    }

    pub fn step_max_damage(&mut self, index: usize, up: bool) {
        let Some(level) = self.levels.get(index).copied() else {
            return;
        };
        if !up && level.max_damage <= level.min_damage {
            return;
        }
        let delta = if up { DAMAGE_STEP } else { -DAMAGE_STEP };
        self.set_level(index, level.min_damage, level.max_damage + delta);
    }

    pub fn remove_level(&mut self, index: usize) {
        if self.levels.len() > 1 && index < self.levels.len() {
            self.levels.remove(index);
        }
    }

    pub fn can_add_upgrade(&self) -> bool {
        (self.upgrades.len() as i64) < MAX_LEVEL - 1
    }

    pub fn add_upgrade(&mut self) {
        if !self.can_add_upgrade() {
            return;
        }
        let lv = self.upgrades.len() as i64 + 2;
        self.upgrades.push(WeaponUpgrade {
            lv,
            cost: UPGRADE_COST_STEP * (lv - 1),
            materials: Vec::new(),
        });
    }

    pub fn step_upgrade_cost(&mut self, index: usize, up: bool) {
        let Some(upgrade) = self.upgrades.get_mut(index) else {
            return;
        };
        if up {
            upgrade.cost += UPGRADE_MIN_COST;
        } else if upgrade.cost > UPGRADE_MIN_COST {
            upgrade.cost -= UPGRADE_MIN_COST;
        }
    }

    pub fn remove_last_upgrade(&mut self) {
        self.upgrades.pop();
    }

    pub fn free_effect_paths(&self) -> Vec<EffectPath> {
        EFFECT_PATHS
            .into_iter()
            .filter(|path| !self.effects.iter().any(|effect| effect.path.path == path.path))
            .collect()
    }

    pub fn effect_choices(&self, index: usize) -> Vec<EffectPath> {
        let current = self.effects.get(index).map(|effect| effect.path.path);
        EFFECT_PATHS
            .into_iter()
            .filter(|path| {
                Some(path.path) == current
                    || !self.effects.iter().any(|effect| effect.path.path == path.path)
            })
            .collect()
    }

    pub fn default_levels(&self, path: EffectPath) -> BTreeMap<i64, f64> {
        if !path.per_level {
            return BTreeMap::new();
        }
        let base = if path.op == EffectOp::Add { 0.1 } else { 1.1 };
        (1..=self.levels.len() as i64)
            .map(|level| (level, round_cents(base * level as f64)))
            .collect()
    }

    fn fresh_effect(&self, path: EffectPath, when_player: Option<String>) -> WeaponEffect {
        WeaponEffect {
            levels: self.default_levels(path),
            value: if path.op == EffectOp::Set { 1.0 } else { 1.5 },
            when_player,
            ..WeaponEffect::new(path)
        }
    }

    pub fn add_effect(&mut self) -> bool {
        let Some(path) = self.free_effect_paths().first().copied() else {
            return false;
        };
        let effect = self.fresh_effect(path, None);
        self.effects.push(effect);
        true
    }

    pub fn set_effect_path(&mut self, index: usize, path: EffectPath) {
        let Some(when_player) = self.effects.get(index).map(|effect| effect.when_player.clone())
        else {
            return;
        };
        self.effects[index] = self.fresh_effect(path, when_player);
    }

    pub fn step_effect_level(&mut self, index: usize, level: i64, up: bool) {
        let Some(effect) = self.effects.get_mut(index) else {
            return;
        };
        let current = effect.levels.get(&level).copied().unwrap_or(1.0);
        let next = if up {
            current + EFFECT_STEP
        } else {
            current - EFFECT_STEP
        };
        effect.levels.insert(level, round_cents(next.max(0.)));
    }

    pub fn set_effect_on(&mut self, index: usize, on: bool) {
        if let Some(effect) = self.effects.get_mut(index) {
            effect.value = if on { 1.0 } else { 0.0 };
        }
    }

    pub fn set_effect_when(&mut self, index: usize, player: Option<String>) {
        if let Some(effect) = self.effects.get_mut(index) {
            effect.when_player = player.filter(|player| !player.is_empty());
        }
    }

    pub fn remove_effect(&mut self, index: usize) {
        if index < self.effects.len() {
            self.effects.remove(index);
        }
    }

    pub fn est_id(&self, berserk: bool) -> i64 {
        let draft = self.draft();
        if berserk {
            draft.effective_est_id_berserk()
        } else {
            draft.effective_est_id_normal()
        }
    }

    pub fn step_est_id(&mut self, berserk: bool, up: bool) {
        let current = self.est_id(berserk);
        if (up && current >= MAX_EST_ID) || (!up && current <= MIN_EST_ID) {
            return;
        }
        let next = weapon::clamp_est_id(if up { current + 1 } else { current - 1 }, up);
        if berserk {
            self.est_id_berserk = Some(next);
        } else {
            self.est_id_normal = Some(next);
        }
    }

    pub fn problem(&self, has_game_dir: bool) -> Option<WeaponProblem> {
        if !has_game_dir {
            Some(WeaponProblem::NeedsGameDir)
        } else if self.name.trim().is_empty() {
            Some(WeaponProblem::NeedsName)
        } else if self.model.is_none() {
            Some(WeaponProblem::NeedsModel)
        } else if self.obj_id.is_none() || self.id.is_none() {
            Some(WeaponProblem::NoFreeSlot)
        } else {
            None
        }
    }
}

pub fn create_weapon(game: &GamePaths, form: &WeaponForm) -> Result<PathBuf, yp_builders::Error> {
    let mod_id = form.mod_id();
    let draft = form.draft();
    if let Some(model) = &form.model {
        weapon_convert::convert(model, draft.obj_id, game, mod_id)?;
    }
    let output_dir = thumbnail::output_dir_for(game, mod_id);
    if let Some(icon) = &form.icon {
        thumbnail::create_named(
            &icon.image_path,
            &output_dir,
            &form.icon_texture_name(),
            icon.width,
            icon.height,
        )?;
    }
    if let Some(story) = &form.story_image {
        thumbnail::create_named(
            &story.image_path,
            &output_dir,
            &form.story_texture_name(),
            story.width,
            story.height,
        )?;
    }
    weapon::write(game, mod_id, &draft)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertForm {
    pub source: Option<WeaponSource>,
    pub mod_id: String,
    pub target_obj_id: Option<i64>,
    pub taken: BTreeSet<i64>,
    pub skipped: BTreeSet<i64>,
}

impl Default for ConvertForm {
    fn default() -> Self {
        Self {
            source: None,
            mod_id: DEFAULT_WEAPON_MOD.to_owned(),
            target_obj_id: None,
            taken: BTreeSet::new(),
            skipped: BTreeSet::new(),
        }
    }
}

impl ConvertForm {
    pub fn mod_id(&self) -> &str {
        or_default(&self.mod_id, DEFAULT_WEAPON_MOD)
    }

    fn allocate(&mut self) {
        let Some(source) = &self.source else {
            self.target_obj_id = None;
            return;
        };
        let used: BTreeSet<i64> = self.taken.union(&self.skipped).copied().collect();
        self.target_obj_id =
            weapon::allocate_obj_id(source.weapon_type, &used, Some(self.mod_id.trim()));
    }

    pub fn accept(&mut self, source: WeaponSource, taken: BTreeSet<i64>) {
        self.source = Some(source);
        self.taken = taken;
        self.skipped.clear();
        self.allocate();
    }

    pub fn set_mod_id(&mut self, mod_id: &str) {
        self.mod_id = mod_id.to_owned();
        self.skipped.clear();
        self.allocate();
    }

    pub fn next_slot(&mut self) {
        if let Some(current) = self.target_obj_id {
            self.skipped.insert(current);
        }
        self.allocate();
        if self.target_obj_id.is_none() && !self.skipped.is_empty() {
            self.skipped.clear();
            self.allocate();
        }
    }

    pub fn target_stem(&self) -> Option<String> {
        self.target_obj_id.map(weapon_convert::stem_for_obj_id)
    }

    pub fn target_label(&self) -> Option<String> {
        self.target_obj_id
            .map(|obj_id| format!("{}  ·  0x{obj_id:X}", weapon_convert::stem_for_obj_id(obj_id)))
    }
}
