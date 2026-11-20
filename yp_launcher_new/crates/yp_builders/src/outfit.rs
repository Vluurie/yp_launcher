use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use toml_edit::Value;
use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::error::Result;
use crate::item::{entities_dir, mod_files};
use crate::toml_out::{basic_string, inline_array, slug, write_file, TomlOut};

pub const MODEL_EXTENSIONS: [&str; 2] = [".dat", ".dtt"];
pub const OUTFIT_ITEM_CATEGORY: i64 = 6;
pub const MAX_SHOP_STATE: i64 = 6;
pub const UNNAMED: &str = "unnamed_outfit";

pub const SHOP_STATE_KEYS: [&str; 7] = [
    "routeABeforeRuinsCollapse",
    "routeAAfterRuinsCollapse",
    "routeBBeforeRuinsCollapse",
    "routeBAfterRuinsCollapse",
    "routeC",
    "endingCOrD",
    "endingCAndD",
];

pub const SHOP_NAMES: [&str; 19] = [
    "shop_RC_Grocery",
    "shop_RC_Grocery_small",
    "shop_PV_Grocery",
    "shop_PV_UniqueShop",
    "shop_AP_Grocery",
    "shop_CS_Grocery",
    "shop_DC_Grocery",
    "shop_DO_Grocery",
    "shop_DP_Grocery",
    "shop_FC_Grocery",
    "shop_GP_Grocery",
    "shop_PC_Grocery",
    "shop_PC_Grocery_first",
    "shop_VF_Grocery",
    "shop_BK_Grocery",
    "shop_CD_Grocery",
    "shop_CF_Grocery",
    "shop_Emile",
    "shop_rareEmile",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OutfitCharacter {
    TwoB,
    NineS,
    A2,
}

impl OutfitCharacter {
    pub const ALL: [OutfitCharacter; 3] = [OutfitCharacter::TwoB, OutfitCharacter::NineS, OutfitCharacter::A2];

    pub const fn toml_value(self) -> &'static str {
        match self {
            OutfitCharacter::TwoB => "2B",
            OutfitCharacter::NineS => "9S",
            OutfitCharacter::A2 => "A2",
        }
    }

    pub const fn model_stems(self) -> [&'static str; 2] {
        match self {
            OutfitCharacter::TwoB => ["pl000d", "pl0000"],
            OutfitCharacter::NineS => ["pl020d", "pl0200"],
            OutfitCharacter::A2 => ["pl010d", "pl0100"],
        }
    }

    pub const fn first_outfit_id(self) -> i64 {
        match self {
            OutfitCharacter::TwoB => 10001,
            OutfitCharacter::NineS => 20001,
            OutfitCharacter::A2 => 30001,
        }
    }

    pub const fn has_eyemask(self) -> bool {
        !matches!(self, OutfitCharacter::A2)
    }

    pub const fn has_wig(self) -> bool {
        matches!(self, OutfitCharacter::A2)
    }

    pub const fn has_story_states(self) -> bool {
        matches!(self, OutfitCharacter::NineS)
    }

    pub const fn has_broken_states(self) -> bool {
        matches!(self, OutfitCharacter::TwoB)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MeshState {
    Base,
    Combat,
    NonCombat,
    Eyemask,
    NonEyemask,
    SelfDestructShow,
    SelfDestructHide,
    Broken,
    NonBroken,
    Wig,
    Hairspray,
    Prologue,
    Holy,
    NoRight,
    Tower,
}

impl MeshState {
    pub const ALL: [MeshState; 15] = [
        MeshState::Base,
        MeshState::Combat,
        MeshState::NonCombat,
        MeshState::Eyemask,
        MeshState::NonEyemask,
        MeshState::SelfDestructShow,
        MeshState::SelfDestructHide,
        MeshState::Broken,
        MeshState::NonBroken,
        MeshState::Wig,
        MeshState::Hairspray,
        MeshState::Prologue,
        MeshState::Holy,
        MeshState::NoRight,
        MeshState::Tower,
    ];

    pub const fn field(self) -> &'static str {
        match self {
            MeshState::Base => "meshes",
            MeshState::Combat => "combat_meshes",
            MeshState::NonCombat => "non_combat_meshes",
            MeshState::Eyemask => "eyemask_meshes",
            MeshState::NonEyemask => "non_eyemask_meshes",
            MeshState::SelfDestructShow => "self_destruct_show_meshes",
            MeshState::SelfDestructHide => "self_destruct_hide_meshes",
            MeshState::Broken => "broken_meshes",
            MeshState::NonBroken => "non_broken_meshes",
            MeshState::Wig => "wig_meshes",
            MeshState::Hairspray => "hairspray_meshes",
            MeshState::Prologue => "prologue_meshes",
            MeshState::Holy => "holy_meshes",
            MeshState::NoRight => "no_right_meshes",
            MeshState::Tower => "tower_meshes",
        }
    }

    pub const fn applies_to(self, character: OutfitCharacter) -> bool {
        match self {
            MeshState::Eyemask | MeshState::NonEyemask => character.has_eyemask(),
            MeshState::Wig => character.has_wig(),
            MeshState::Broken | MeshState::NonBroken => character.has_broken_states(),
            MeshState::Prologue | MeshState::Holy | MeshState::NoRight | MeshState::Tower => character.has_story_states(),
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeshVisibility {
    #[default]
    Always,
    CombatOnly,
    OutOfCombatOnly,
    EyemaskOnly,
    NoEyemaskOnly,
    FeatherOnly,
    BrokenOnly,
    Never,
}

impl MeshVisibility {
    pub const fn state(self) -> Option<MeshState> {
        match self {
            MeshVisibility::Always => Some(MeshState::Base),
            MeshVisibility::CombatOnly => Some(MeshState::Combat),
            MeshVisibility::OutOfCombatOnly => Some(MeshState::NonCombat),
            MeshVisibility::EyemaskOnly => Some(MeshState::Eyemask),
            MeshVisibility::NoEyemaskOnly => Some(MeshState::NonEyemask),
            MeshVisibility::FeatherOnly => Some(MeshState::NonBroken),
            MeshVisibility::BrokenOnly => Some(MeshState::Broken),
            MeshVisibility::Never => None,
        }
    }

    pub const fn applies_to(self, character: OutfitCharacter) -> bool {
        match self {
            MeshVisibility::EyemaskOnly | MeshVisibility::NoEyemaskOnly => character.has_eyemask(),
            MeshVisibility::FeatherOnly | MeshVisibility::BrokenOnly => character.has_broken_states(),
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MeshRule {
    pub visibility: MeshVisibility,
    pub hide_on_self_destruct: bool,
    pub tinted_by_hairspray: bool,
    pub replaced_by_wig: bool,
}

impl MeshRule {
    pub fn is_default(&self) -> bool {
        *self == MeshRule::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MeshNames(Vec<String>);

impl MeshNames {
    pub fn insert(&mut self, name: &str) {
        if !self.0.iter().any(|known| known == name) {
            self.0.push(name.to_owned());
        }
    }

    pub fn names(&self) -> &[String] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|known| known == name)
    }

    pub fn sorted(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.0.iter().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}

impl<S: AsRef<str>> FromIterator<S> for MeshNames {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        let mut names = MeshNames::default();
        for name in iter {
            names.insert(name.as_ref());
        }
        names
    }
}

pub type MeshLists = BTreeMap<MeshState, MeshNames>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitDraft {
    pub display_name: String,
    pub description: String,
    pub character: OutfitCharacter,
    pub outfit_id: i64,
    pub item_id: i64,
    pub model_source_path: Option<PathBuf>,
    pub meshes: MeshLists,
    pub auto_give: bool,
    pub shop_name: Option<String>,
    pub shop_states: BTreeSet<i64>,
    pub buy_price: i64,
}

impl OutfitDraft {
    pub fn new(
        display_name: impl Into<String>,
        description: impl Into<String>,
        character: OutfitCharacter,
        outfit_id: i64,
        item_id: i64,
    ) -> Self {
        Self {
            display_name: display_name.into(),
            description: description.into(),
            character,
            outfit_id,
            item_id,
            model_source_path: None,
            meshes: MeshLists::new(),
            auto_give: true,
            shop_name: None,
            shop_states: BTreeSet::from([0]),
            buy_price: 5000,
        }
    }

    pub fn sells_in_shop(&self) -> bool {
        self.shop_name.as_deref().is_some_and(|name| !name.is_empty())
    }

    pub fn meshes_for(&self, state: MeshState) -> &[String] {
        self.meshes.get(&state).map(MeshNames::names).unwrap_or_default()
    }

    pub fn has_any_mesh_rules(&self) -> bool {
        self.meshes.values().any(|names| !names.is_empty())
    }

    pub fn internal_name(&self) -> String {
        let name = slug(&self.display_name);
        if name.is_empty() { UNNAMED.to_owned() } else { name }
    }
}

pub fn shop_item_toml(draft: &OutfitDraft) -> String {
    let mut out = TomlOut::new();
    for (index, state) in draft.shop_states.iter().enumerate() {
        out.array_entry(
            "shop_item",
            index > 0,
            vec![
                ("shop_name", basic_string(draft.shop_name.as_deref().unwrap_or_default())),
                ("item_id", Value::from(draft.item_id)),
                ("state", Value::from(*state)),
            ],
        );
    }
    out.finish()
}

pub fn shop_file_name_for(draft: &OutfitDraft) -> String {
    format!("shop{}.toml", draft.item_id)
}

pub fn write_shop_item(game: &GamePaths, mod_id: &str, draft: &OutfitDraft) -> Result<PathBuf> {
    Ok(write_file(&entities_dir(game, mod_id).join(shop_file_name_for(draft)), &shop_item_toml(draft))?)
}

pub fn to_toml(draft: &OutfitDraft) -> String {
    let mut out = TomlOut::new();
    out.set("outfit_id", Value::from(draft.outfit_id));
    out.set("character", basic_string(draft.character.toml_value()));
    out.set("item_id", Value::from(draft.item_id));
    out.set("name", basic_string(&draft.internal_name()));
    if !draft.has_any_mesh_rules() {
        return out.finish();
    }
    let mut first = true;
    for state in MeshState::ALL {
        if !state.applies_to(draft.character) {
            continue;
        }
        let names = draft.meshes_for(state);
        if names.is_empty() {
            continue;
        }
        out.set(state.field(), inline_array(names.iter().map(|name| basic_string(name)).collect()));
        if first {
            out.gap_before_key(state.field());
            first = false;
        }
    }
    out.finish()
}

pub fn file_name_for(draft: &OutfitDraft) -> String {
    format!("outfit_{}.toml", draft.outfit_id)
}

pub fn path_for(game: &GamePaths, mod_id: &str, draft: &OutfitDraft) -> PathBuf {
    entities_dir(game, mod_id).join(file_name_for(draft))
}

pub fn model_dir_for(game: &GamePaths, mod_id: &str) -> PathBuf {
    game.mods_dir().join(mod_id).join("data").join("pl")
}

pub fn is_model_file(path: &Path) -> bool {
    MODEL_EXTENSIONS.contains(&dotted_extension(path).as_str())
}

pub(crate) fn dotted_extension(path: &Path) -> String {
    path.extension().map(|ext| format!(".{}", ext.to_string_lossy().to_lowercase())).unwrap_or_default()
}

pub fn lists_from_rules(character: OutfitCharacter, rules: &[(&str, MeshRule)]) -> MeshLists {
    let mut result = MeshLists::new();
    if rules.iter().all(|(_, rule)| rule.is_default()) {
        return result;
    }
    let mut add = |state: MeshState, name: &str| {
        if state.applies_to(character) {
            result.entry(state).or_default().insert(name);
        }
    };
    for (name, rule) in rules {
        let state = if rule.visibility.applies_to(character) { rule.visibility.state() } else { Some(MeshState::Base) };
        if let Some(state) = state {
            add(state, name);
        }
        if rule.hide_on_self_destruct {
            add(MeshState::SelfDestructHide, name);
        }
        if rule.tinted_by_hairspray {
            add(MeshState::Hairspray, name);
        }
        if rule.replaced_by_wig {
            add(MeshState::Wig, name);
        }
    }
    result
}

pub fn sibling_path(path: &Path, extension: &str) -> PathBuf {
    let stem = path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
    path.with_file_name(format!("{stem}{extension}"))
}

pub fn existing_sibling(path: &Path, extension: &str) -> Option<PathBuf> {
    Some(sibling_path(path, extension)).filter(|candidate| candidate.is_file())
}

pub fn character_from_model_path(path: &Path) -> Option<OutfitCharacter> {
    let stem = windows_stem(path).to_lowercase();
    OutfitCharacter::ALL.into_iter().find(|character| character.model_stems().contains(&stem.as_str()))
}

fn windows_stem(path: &Path) -> String {
    let text = path.to_string_lossy();
    let name = text.rsplit(['/', '\\']).next().unwrap_or(&text);
    match name.rfind('.') {
        Some(dot) if dot > 0 => name[..dot].to_owned(),
        _ => name.to_owned(),
    }
}

pub fn next_free_outfit_id(game: &GamePaths, character: OutfitCharacter) -> i64 {
    let taken = taken_outfit_ids(game, character);
    let mut id = character.first_outfit_id();
    while taken.contains(&id) {
        id += 1;
    }
    id
}

pub fn taken_outfit_ids(game: &GamePaths, character: OutfitCharacter) -> BTreeSet<i64> {
    mod_files(&game.mods_dir())
        .iter()
        .filter(|path| {
            let name = yp_core::fs::file_name_lossy(path);
            name.starts_with("outfit_") && name.ends_with(".toml")
        })
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .filter_map(|text| toml::from_str::<toml::Table>(&text).ok())
        .filter(|table| {
            table
                .get("character")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| value.eq_ignore_ascii_case(character.toml_value()))
        })
        .filter_map(|table| table.get("outfit_id").and_then(toml::Value::as_integer))
        .collect()
}

pub fn write(game: &GamePaths, mod_id: &str, draft: &OutfitDraft) -> Result<PathBuf> {
    Ok(write_file(&path_for(game, mod_id, draft), &to_toml(draft))?)
}

pub fn copy_model(game: &GamePaths, mod_id: &str, source_path: &Path) -> Result<Vec<PathBuf>> {
    let target_dir = model_dir_for(game, mod_id);
    std::fs::create_dir_all(&target_dir).io(IoOp::Create, &target_dir)?;
    let mut copied = Vec::new();
    for extension in MODEL_EXTENSIONS {
        let source = sibling_path(source_path, extension);
        if !source.is_file() {
            continue;
        }
        let target = target_dir.join(yp_core::fs::file_name_lossy(&source));
        std::fs::copy(&source, &target).io(IoOp::Copy, &target)?;
        copied.push(target);
    }
    Ok(copied)
}
