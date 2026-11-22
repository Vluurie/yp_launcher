use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use toml_edit::Value;
use yp_core::GamePaths;

use crate::error::Result;
use crate::item::{entities_dir, mod_file_names, mod_files, numbered_id};
use crate::toml_out::{basic_string, hex_int, inline, multiline_array, slug, text_block, write_file, TomlOut};

pub const FIRST_ID: i64 = 1000;
pub const LAST_ID: i64 = 1999;
pub const MAX_LEVEL: i64 = 4;
pub const MIN_EST_ID: i64 = 0;
pub const MAX_EST_ID: i64 = 665;
pub const BLOCKED_EST_IDS: [i64; 2] = [435, 436];
pub const UNNAMED: &str = "weapon_unnamed";

pub const VANILLA_IDS: [i64; 39] = [
    1003, 1013, 1020, 1030, 1040, 1050, 1060, 1070, 1071, 1080, 1090, 1203, 1213, 1220, 1230, 1240, 1250, 1260, 1261,
    1270, 1400, 1420, 1430, 1440, 1450, 1460, 1470, 1471, 1480, 1600, 1610, 1620, 1621, 1630, 1640, 1650, 1875, 1876,
    1877,
];

pub const VANILLA_OBJ_IDS: [i64; 39] = [
    0x30003, 0x30013, 0x30020, 0x30030, 0x30040, 0x30050, 0x30060, 0x30070, 0x30071, 0x30075, 0x30076, 0x30080, 0x30090,
    0x30203, 0x30213, 0x30220, 0x30230, 0x30240, 0x30250, 0x30260, 0x30261, 0x30270, 0x30400, 0x30420, 0x30430, 0x30440,
    0x30450, 0x30460, 0x30470, 0x30471, 0x30480, 0x30600, 0x30610, 0x30620, 0x30621, 0x30630, 0x30640, 0x30650, 0x30655,
];

pub const WPMASK_OBJ_IDS: [i64; 5] = [0x30072, 0x30081, 0x30222, 0x30432, 0x30602];

pub const WHEN_PLAYERS: [&str; 6] = ["2B", "9S", "A2", "not_2b", "not_9s", "not_a2"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WeaponType {
    SmallSword,
    LargeSword,
    Spear,
    CombatBracer,
}

impl WeaponType {
    pub const ALL: [WeaponType; 4] =
        [WeaponType::SmallSword, WeaponType::LargeSword, WeaponType::Spear, WeaponType::CombatBracer];

    pub const fn category(self) -> i64 {
        match self {
            WeaponType::SmallSword => 0,
            WeaponType::LargeSword => 1,
            WeaponType::Spear => 2,
            WeaponType::CombatBracer => 3,
        }
    }

    pub const fn obj_id_start(self) -> i64 {
        match self {
            WeaponType::SmallSword => 0x30000,
            WeaponType::LargeSword => 0x30200,
            WeaponType::Spear => 0x30400,
            WeaponType::CombatBracer => 0x30600,
        }
    }

    pub const fn obj_id_end(self) -> i64 {
        self.obj_id_start() + 0x200
    }

    pub const fn clone_from_obj_id(self) -> i64 {
        match self {
            WeaponType::SmallSword => 0x30003,
            WeaponType::LargeSword => 0x30203,
            WeaponType::Spear => 0x30400,
            WeaponType::CombatBracer => 0x30600,
        }
    }

    pub const fn est_id_normal(self) -> i64 {
        match self {
            WeaponType::SmallSword => 600,
            WeaponType::LargeSword => 610,
            WeaponType::Spear => 620,
            WeaponType::CombatBracer => 630,
        }
    }

    pub const fn est_id_berserk(self) -> i64 {
        match self {
            WeaponType::SmallSword => 633,
            WeaponType::LargeSword => 634,
            WeaponType::Spear => 635,
            WeaponType::CombatBracer => 636,
        }
    }
}

impl From<WeaponType> for yp_formats::WeaponClass {
    fn from(value: WeaponType) -> Self {
        match value {
            WeaponType::SmallSword => yp_formats::WeaponClass::SmallSword,
            WeaponType::LargeSword => yp_formats::WeaponClass::LargeSword,
            WeaponType::Spear => yp_formats::WeaponClass::Spear,
            WeaponType::CombatBracer => yp_formats::WeaponClass::CombatBracer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WeaponLevel {
    pub min_damage: i64,
    pub max_damage: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectOp {
    Mul,
    Add,
    Set,
}

impl EffectOp {
    pub const fn name(self) -> &'static str {
        match self {
            EffectOp::Mul => "mul",
            EffectOp::Add => "add",
            EffectOp::Set => "set",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectPath {
    pub path: &'static str,
    pub op: EffectOp,
    pub per_level: bool,
}

impl EffectPath {
    const fn leveled(path: &'static str, op: EffectOp) -> Self {
        Self { path, op, per_level: true }
    }

    const fn flat(path: &'static str, op: EffectOp) -> Self {
        Self { path, op, per_level: false }
    }
}

pub const EFFECT_PATHS: [EffectPath; 19] = [
    EffectPath::leveled("player.gold_gain", EffectOp::Mul),
    EffectPath::leveled("player.exp_gain", EffectOp::Mul),
    EffectPath::leveled("player.move_speed", EffectOp::Mul),
    EffectPath::leveled("player.melee_out", EffectOp::Mul),
    EffectPath::leveled("player.ranged_out", EffectOp::Mul),
    EffectPath::leveled("player.melee_in_reduction", EffectOp::Add),
    EffectPath::leveled("player.ranged_in_reduction", EffectOp::Add),
    EffectPath::leveled("player.pod_cooldown_reduction", EffectOp::Add),
    EffectPath::leveled("player.hacking_level_add", EffectOp::Add),
    EffectPath::flat("player.stagger_immune", EffectOp::Set),
    EffectPath::flat("player.electric_immune", EffectOp::Set),
    EffectPath::flat("player.eyemask_hidden", EffectOp::Set),
    EffectPath::flat("player.a2_wig_show", EffectOp::Set),
    EffectPath::flat("player.block_exp", EffectOp::Set),
    EffectPath::flat("player.taunt_active", EffectOp::Set),
    EffectPath::leveled("player.dodge.propulsion_9s.forward", EffectOp::Mul),
    EffectPath::leveled("player.dodge.propulsion_default.forward", EffectOp::Mul),
    EffectPath::leveled("player.dodge.propulsion_flight_suit.forward", EffectOp::Mul),
    EffectPath::leveled("player.dodge.particles.forward", EffectOp::Set),
];

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponEffect {
    pub path: EffectPath,
    pub levels: BTreeMap<i64, f64>,
    pub value: f64,
    pub when_player: Option<String>,
}

impl WeaponEffect {
    pub fn new(path: EffectPath) -> Self {
        Self {
            path,
            levels: BTreeMap::new(),
            value: 1.0,
            when_player: None,
        }
    }

    pub fn name(&self) -> String {
        let tail = self.path.path.rsplit('.').next().unwrap_or(self.path.path);
        match &self.when_player {
            Some(player) => format!("{tail}_{}", player.to_lowercase()),
            None => tail.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WeaponMaterial {
    pub item_id: i64,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WeaponUpgrade {
    pub lv: i64,
    pub cost: i64,
    pub materials: Vec<WeaponMaterial>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponDraft {
    pub display_name: String,
    pub description: String,
    pub weapon_type: WeaponType,
    pub id: i64,
    pub obj_id: i64,
    pub sell_price: i64,
    pub levels: Vec<WeaponLevel>,
    pub shop_name: Option<String>,
    pub shop_states: BTreeSet<i64>,
    pub buy_price: i64,
    pub icon_name: Option<String>,
    pub upgrades: Vec<WeaponUpgrade>,
    pub story: Vec<String>,
    pub skill1_name: String,
    pub skill1_desc: String,
    pub skill2_name: String,
    pub skill2_desc: String,
    pub help_short: String,
    pub trait_id: Option<i64>,
    pub story_texture_suffix: Option<i64>,
    pub effects: Vec<WeaponEffect>,
    pub est_id_normal: Option<i64>,
    pub est_id_berserk: Option<i64>,
}

impl WeaponDraft {
    pub fn new(
        display_name: impl Into<String>,
        description: impl Into<String>,
        weapon_type: WeaponType,
        id: i64,
        obj_id: i64,
    ) -> Self {
        Self {
            display_name: display_name.into(),
            description: description.into(),
            weapon_type,
            id,
            obj_id,
            sell_price: 500,
            levels: Vec::new(),
            shop_name: None,
            shop_states: BTreeSet::from([0]),
            buy_price: 5000,
            icon_name: None,
            upgrades: Vec::new(),
            story: Vec::new(),
            skill1_name: String::new(),
            skill1_desc: String::new(),
            skill2_name: String::new(),
            skill2_desc: String::new(),
            help_short: String::new(),
            trait_id: None,
            story_texture_suffix: None,
            effects: Vec::new(),
            est_id_normal: None,
            est_id_berserk: None,
        }
    }

    pub fn sells_in_shop(&self) -> bool {
        self.shop_name.as_deref().is_some_and(|name| !name.is_empty())
    }

    pub fn effective_est_id_normal(&self) -> i64 {
        self.est_id_normal.unwrap_or(self.weapon_type.est_id_normal())
    }

    pub fn effective_est_id_berserk(&self) -> i64 {
        self.est_id_berserk.unwrap_or(self.weapon_type.est_id_berserk())
    }

    pub fn internal_name(&self) -> String {
        let name = slug(&self.display_name);
        if name.is_empty() { UNNAMED.to_owned() } else { format!("weapon_{name}") }
    }
}

pub fn is_est_id_allowed(id: i64) -> bool {
    (MIN_EST_ID..=MAX_EST_ID).contains(&id) && !BLOCKED_EST_IDS.contains(&id)
}

pub fn clamp_est_id(id: i64, up: bool) -> i64 {
    let mut value = id.clamp(MIN_EST_ID, MAX_EST_ID);
    while BLOCKED_EST_IDS.contains(&value) {
        value += if up { 1 } else { -1 };
        if !(MIN_EST_ID..=MAX_EST_ID).contains(&value) {
            return id;
        }
    }
    value
}

pub fn seed_hash(value: &str) -> i64 {
    let mut hash: i64 = 0x811c_9dc5;
    for unit in value.to_lowercase().encode_utf16() {
        hash = ((hash ^ i64::from(unit)).wrapping_mul(0x0100_0193)) & 0x7fff_ffff;
    }
    hash
}

pub fn type_from_obj_id(obj_id: i64) -> Option<WeaponType> {
    WeaponType::ALL
        .into_iter()
        .find(|weapon_type| (weapon_type.obj_id_start()..weapon_type.obj_id_end()).contains(&obj_id))
}

pub fn is_obj_id_available(obj_id: i64) -> bool {
    type_from_obj_id(obj_id).is_some() && !VANILLA_OBJ_IDS.contains(&obj_id) && !WPMASK_OBJ_IDS.contains(&obj_id)
}

fn seed_offset(seed: Option<&str>, span: i64) -> i64 {
    seed.filter(|seed| !seed.is_empty()).map_or(0, |seed| seed_hash(seed) % span)
}

pub fn allocate_obj_id(weapon_type: WeaponType, already_used: &BTreeSet<i64>, seed: Option<&str>) -> Option<i64> {
    let span = weapon_type.obj_id_end() - weapon_type.obj_id_start();
    let offset = seed_offset(seed, span);
    (0..span)
        .map(|step| weapon_type.obj_id_start() + (offset + step) % span)
        .find(|id| is_obj_id_available(*id) && !already_used.contains(id))
}

pub fn allocate_id(already_used: &BTreeSet<i64>, seed: Option<&str>) -> Option<i64> {
    let span = LAST_ID - FIRST_ID + 1;
    let offset = seed_offset(seed, span);
    (0..span)
        .map(|step| FIRST_ID + (offset + step) % span)
        .find(|id| !VANILLA_IDS.contains(id) && !already_used.contains(id))
}

pub fn to_toml(draft: &WeaponDraft) -> String {
    let mut out = TomlOut::new();
    out.set("id", Value::from(draft.id));
    out.set("obj_id", hex_int(draft.obj_id));
    out.set("category", Value::from(draft.weapon_type.category()));
    if !draft.skill1_name.is_empty() {
        out.set("special1", Value::from(draft.id * 10 + 1));
    }
    if !draft.skill2_name.is_empty() {
        out.set("special2", Value::from(draft.id * 10 + 2));
    }
    if let Some(suffix) = draft.story_texture_suffix {
        out.set("story_texture_suffix", Value::from(suffix));
    }

    out.header(
        &["item"],
        vec![
            ("name", basic_string(&draft.internal_name())),
            ("max_num", Value::from(1)),
            ("no_trash", Value::from(true)),
            ("die_discard", Value::from(false)),
            ("buy", Value::from(if draft.sells_in_shop() { draft.buy_price } else { -1 })),
            ("sell", Value::from(draft.sell_price)),
        ],
    );

    if let Some(icon) = draft.icon_name.as_deref().filter(|icon| !icon.is_empty()) {
        out.header(&["icon"], vec![("misctex_name", basic_string(icon))]);
    }

    out.header(
        &["effects"],
        vec![
            ("est_id_normal", Value::from(draft.effective_est_id_normal())),
            ("est_id_berserk", Value::from(draft.effective_est_id_berserk())),
        ],
    );

    out.header(&["text", "name"], vec![("_us", basic_string(&draft.display_name))]);
    out.header(&["text", "help"], vec![("_us", text_block(&draft.description))]);
    if !draft.help_short.is_empty() {
        out.header(&["text", "help_short"], vec![("_us", text_block(&draft.help_short))]);
    }
    write_skill(&mut out, 1, &draft.skill1_name, &draft.skill1_desc);
    write_skill(&mut out, 2, &draft.skill2_name, &draft.skill2_desc);

    for page in &draft.story {
        out.array_entry("story", true, vec![("_us", text_block(page))]);
    }

    for upgrade in &draft.upgrades {
        let mut entries = vec![("lv", Value::from(upgrade.lv)), ("cost", Value::from(upgrade.cost))];
        if !upgrade.materials.is_empty() {
            let materials = upgrade
                .materials
                .iter()
                .map(|material| {
                    inline(vec![("item_id", Value::from(material.item_id)), ("quantity", Value::from(material.quantity))])
                })
                .collect();
            entries.push(("materials", multiline_array(materials)));
        }
        out.array_entry("upgrades", true, entries);
    }

    let mut params = vec![("clone_from_obj_id", hex_int(draft.weapon_type.clone_from_obj_id()))];
    if !draft.levels.is_empty() {
        let levels = draft
            .levels
            .iter()
            .map(|level| {
                inline(vec![("min_damage", Value::from(level.min_damage)), ("max_damage", Value::from(level.max_damage))])
            })
            .collect();
        params.push(("levels", multiline_array(levels)));
    }
    if let Some(trait_id) = draft.trait_id {
        params.push(("trait_id", Value::from(trait_id)));
    }
    out.header(&["params"], params);

    if draft.sells_in_shop() {
        let shop = draft.shop_name.as_deref().unwrap_or_default();
        for state in &draft.shop_states {
            out.array_entry("shops", true, vec![("shop_name", basic_string(shop)), ("state", Value::from(*state))]);
        }
    }

    for effect in &draft.effects {
        let mut entries = vec![("path", basic_string(effect.path.path)), ("op", basic_string(effect.path.op.name()))];
        if effect.path.per_level && !effect.levels.is_empty() {
            let level_keys: Vec<String> = effect.levels.keys().map(i64::to_string).collect();
            let pairs = level_keys
                .iter()
                .zip(effect.levels.values())
                .map(|(level, value)| (level.as_str(), Value::from(*value)))
                .collect();
            entries.push(("levels", inline(pairs)));
        } else {
            entries.push(("value", Value::from(effect.value)));
        }
        if let Some(player) = &effect.when_player {
            entries.push(("when", inline(vec![("player", basic_string(player))])));
        }
        out.header(&["effect", &effect.name()], entries);
    }

    out.finish()
}

fn write_skill(out: &mut TomlOut, slot: u8, name: &str, description: &str) {
    if name.is_empty() {
        return;
    }
    let name_table = format!("skill_{slot}_name");
    out.header(&["text", &name_table], vec![("_us", basic_string(name))]);
    if description.is_empty() {
        return;
    }
    let desc_table = format!("skill_{slot}_desc");
    out.header(&["text", &desc_table], vec![("_us", text_block(description))]);
}

pub fn icon_texture_name_for(id: i64) -> String {
    format!("wpi_{id}")
}

pub fn story_texture_name_for(suffix: i64) -> String {
    format!("ws_{suffix}")
}

pub fn file_name_for(draft: &WeaponDraft) -> String {
    format!("weapon_{}.toml", draft.id)
}

pub fn path_for(game: &GamePaths, mod_id: &str, draft: &WeaponDraft) -> PathBuf {
    entities_dir(game, mod_id).join(file_name_for(draft))
}

pub fn taken_obj_ids(game: &GamePaths) -> BTreeSet<i64> {
    mod_files(&game.mods_dir())
        .iter()
        .filter(|path| {
            let name = yp_core::fs::file_name_lossy(path);
            name.starts_with("weapon_") && name.ends_with(".toml")
        })
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .filter_map(|text| toml::from_str::<toml::Table>(&text).ok())
        .filter_map(|table| table.get("obj_id").and_then(toml::Value::as_integer))
        .collect()
}

pub fn next_free_id(game: &GamePaths, seed: Option<&str>) -> Option<i64> {
    allocate_id(&taken_ids(game), seed)
}

pub fn taken_ids(game: &GamePaths) -> BTreeSet<i64> {
    mod_file_names(&game.mods_dir())
        .iter()
        .filter_map(|name| numbered_id(name, "weapon_").or_else(|| numbered_id(name, "item_")))
        .collect()
}

pub fn write(game: &GamePaths, mod_id: &str, draft: &WeaponDraft) -> Result<PathBuf> {
    Ok(write_file(&path_for(game, mod_id, draft), &to_toml(draft))?)
}
