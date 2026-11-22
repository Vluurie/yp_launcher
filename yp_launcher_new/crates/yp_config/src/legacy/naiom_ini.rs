use std::fmt;

use indexmap::IndexMap;
use yp_core::GamePaths;
use yp_core::ini::{IniDoc, IniSyntax};

use serde_json::Value as Json;

use crate::cli::{self, NamsCli};
use crate::error::Result;
use crate::naiom_keys;
use crate::reports::{Problem, Values};

pub const NAIOM_PREFIX: &str = "naiom.";
pub const BINDINGS_PREFIX: &str = "naiom.bindings.";
const THIRD_PERSON_MODE: &str = "naiom.third_person_mode";
const THIRD_PERSON_CHAR_FOLLOW: &str = "naiom.third_person_char_follow";
const AIM_MODE: &str = "naiom.aim_mode";
const AIM_CROSSHAIR: &str = "naiom.aim_crosshair";
const MISC_DISABLE_DEFAULT_CURSOR: &str = "naiom.misc_disable_default_cursor";
const MISC_OPEN_DEBUG_MENU: &str = "naiom.misc_open_debug_menu";

pub const NAIOM_INI_FILES: [&str; 2] = ["NAIOM.ini", "NAIOM.cfg"];
const UNBOUND: &str = "<unbound>";

pub type NaiomIniValues = IndexMap<String, String>;

const BINDING_MAP: [(&str, &str); 34] = [
    ("standardmoveforward", "standard_move_forward"),
    ("standardmoveleft", "standard_move_left"),
    ("standardmovebackward", "standard_move_backward"),
    ("standardmoveright", "standard_move_right"),
    ("standardjump", "standard_jump"),
    ("standardwalk", "standard_walk"),
    ("standardautorun", "standard_auto_run"),
    ("standardlightattack", "standard_light_attack"),
    ("standardheavyattack", "standard_heavy_attack"),
    ("standardprogram", "standard_program"),
    ("standardlockon", "standard_lock_on"),
    ("standarduse", "standard_use"),
    ("standardselfdestruct", "standard_self_destruct"),
    ("standardlight", "standard_light"),
    ("standardresetcamera", "standard_reset_camera"),
    ("standardmenuup", "standard_menu_up"),
    ("standardmenuleft", "standard_menu_left"),
    ("standardmenudown", "standard_menu_down"),
    ("standardmenuright", "standard_menu_right"),
    ("standardmenuopen", "standard_menu_open"),
    ("standardmenuback", "standard_menu_back"),
    ("standardmenuenter", "standard_menu_enter"),
    ("standardshortcutmenu", "standard_shortcut_menu"),
    ("standardswitchweapon", "standard_switch_weapon"),
    ("standardnextprogram", "standard_next_program"),
    ("standardpreviousprogram", "standard_previous_program"),
    ("standardfire", "standard_fire"),
    ("nonstandardevade", "non_standard_evade"),
    ("nonstandardautofire", "non_standard_auto_fire"),
    ("nonstandardnextitem", "non_standard_next_item"),
    ("nonstandardpreviousitem", "non_standard_previous_item"),
    ("nonstandarduseitem", "non_standard_use_item"),
    ("thirdpersonmodetoggle", "third_person_mode_toggle"),
    ("aimmodetoggle", "aim_mode_toggle"),
];

const FLOAT_MAP: [(&str, &str); 4] = [
    ("thirdpersonsensitivityx", "third_person_sensitivity_x"),
    ("thirdpersonsensitivityy", "third_person_sensitivity_y"),
    ("aimsensitivityx", "aim_sensitivity_x"),
    ("aimsensitivityy", "aim_sensitivity_y"),
];

const BOOL_MAP: [(&str, &str); 2] = [
    ("movementdisabletapevade", "movement_disable_tap_evade"),
    ("miscdisablepodpet", "misc_disable_pod_pet"),
];

fn lookup(map: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    map.iter()
        .find(|(ini, _)| *ini == key)
        .map(|(_, toml)| *toml)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkippedNaiomSetting {
    DebugMenuKey,
    Binding(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedNaiomEntry {
    pub setting: SkippedNaiomSetting,
    pub value: String,
}

fn display_binding_key(toml_key: &str) -> String {
    toml_key
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl fmt::Display for SkippedNaiomEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.setting {
            SkippedNaiomSetting::DebugMenuKey => write!(f, "MiscOpenDebugMenu ({})", self.value),
            SkippedNaiomSetting::Binding(key) => {
                write!(f, "{} ({})", display_binding_key(key), self.value)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NaiomMigration {
    pub migrated: bool,
    pub skipped: Vec<SkippedNaiomEntry>,
}

pub fn parse_naiom_ini(content: &str) -> NaiomIniValues {
    let syntax = IniSyntax::whitespace_separated();
    let doc = IniDoc::parse_with(content, syntax.clone());
    let mut values = NaiomIniValues::new();
    for (_, key, value) in doc.entries() {
        let value = syntax.strip_inline_comment(value);
        let key = key.trim().to_lowercase();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        values.insert(key, value.to_owned());
    }
    values
}

pub fn detect_naiom_ini(game: &GamePaths) -> Result<Option<NaiomIniValues>> {
    for name in NAIOM_INI_FILES {
        let path = game.root().join(name);
        let Some(content) = yp_core::fs::read_text_opt(&path)? else {
            continue;
        };
        let values = parse_naiom_ini(&content);
        if !values.is_empty() {
            return Ok(Some(values));
        }
    }
    Ok(None)
}

fn bool_of(value: &str) -> bool {
    let lower = value.to_lowercase();
    lower == "1" || lower == "true"
}

fn is_unbound(value: &str) -> bool {
    value.is_empty() || value.to_lowercase() == UNBOUND
}

pub fn already_configured(current: &Values) -> bool {
    let flag = |id: &str| current.get(id).and_then(Json::as_bool) == Some(true);
    let any_binding = current.values.iter().any(|(id, value)| {
        id.starts_with(BINDINGS_PREFIX) && value.as_str().is_some_and(|binding| !binding.is_empty())
    });
    flag(THIRD_PERSON_MODE) || flag(AIM_MODE) || any_binding
}

#[derive(Default)]
struct NaiomUpdates {
    assignments: Vec<(String, Json)>,
    skipped: Vec<SkippedNaiomEntry>,
}

impl NaiomUpdates {
    fn set(&mut self, id: &str, value: Json) {
        self.assignments.push((id.to_owned(), value));
    }

    fn apply(&mut self, key: &str, value: &str) {
        if let Some(float_key) = lookup(&FLOAT_MAP, key) {
            if let Ok(number) = value.parse::<f64>()
                && let Some(number) = serde_json::Number::from_f64(number)
            {
                self.set(&format!("{NAIOM_PREFIX}{float_key}"), Json::Number(number));
            }
            return;
        }
        if let Some(bool_key) = lookup(&BOOL_MAP, key) {
            self.set(&format!("{NAIOM_PREFIX}{bool_key}"), Json::Bool(bool_of(value)));
            return;
        }
        match key {
            "thirdpersonmode" => {
                let mode = value.to_lowercase();
                self.set(THIRD_PERSON_MODE, Json::Bool(mode != "disabled"));
                self.set(THIRD_PERSON_CHAR_FOLLOW, Json::Bool(mode == "rawinputcharfollow"));
                return;
            }
            "aimmode" => {
                let mode = value.to_lowercase();
                self.set(AIM_MODE, Json::Bool(mode != "disabled"));
                self.set(AIM_CROSSHAIR, Json::Bool(mode == "crosshair"));
                return;
            }
            "miscenablecustomcursors" => {
                self.set(MISC_DISABLE_DEFAULT_CURSOR, Json::Bool(!bool_of(value)));
                return;
            }
            "miscopendebugmenu" => {
                if is_unbound(value) {
                    return;
                }
                match naiom_keys::key_name_to_vk(value) {
                    Some(vk) => self.set(MISC_OPEN_DEBUG_MENU, Json::from(vk)),
                    None => self.skipped.push(SkippedNaiomEntry {
                        setting: SkippedNaiomSetting::DebugMenuKey,
                        value: value.to_owned(),
                    }),
                }
                return;
            }
            "soundsenabled" | "configreload" => return,
            _ => {}
        }
        let Some(binding_key) = lookup(&BINDING_MAP, key) else {
            return;
        };
        if is_unbound(value) {
            return;
        }
        let normalized = naiom_keys::normalize_binding(value);
        let invalid = !naiom_keys::is_valid_binding(&normalized) || normalized.is_empty();
        let keyboard_only_violation = naiom_keys::combo_contains_non_keyboard(&normalized)
            && !naiom_keys::allows_mouse(binding_key);
        if invalid || keyboard_only_violation {
            self.skipped.push(SkippedNaiomEntry {
                setting: SkippedNaiomSetting::Binding(binding_key),
                value: value.to_owned(),
            });
            return;
        }
        self.set(&format!("{BINDINGS_PREFIX}{binding_key}"), Json::String(normalized));
    }
}

pub fn naiom_ini_assignments(ini_values: &NaiomIniValues) -> (Vec<(String, Json)>, Vec<SkippedNaiomEntry>) {
    let mut updates = NaiomUpdates::default();
    for (key, value) in ini_values {
        updates.apply(key, value);
    }
    (updates.assignments, updates.skipped)
}

fn skipped_of(problem: &Problem, values: &NaiomIniValues) -> Option<SkippedNaiomEntry> {
    let binding_key = problem.id.as_deref()?.strip_prefix(BINDINGS_PREFIX)?;
    let (ini_key, key) = BINDING_MAP.iter().find(|(_, key)| *key == binding_key)?;
    Some(SkippedNaiomEntry {
        setting: SkippedNaiomSetting::Binding(key),
        value: values.get(*ini_key)?.clone(),
    })
}

pub fn import_naiom_ini(cli: &dyn NamsCli, ini_values: &NaiomIniValues) -> Result<NaiomMigration> {
    let current = cli::get(cli)?;
    if already_configured(&current) {
        return Ok(NaiomMigration::default());
    }
    let (assignments, mut skipped) = naiom_ini_assignments(ini_values);
    if assignments.is_empty() {
        return Ok(NaiomMigration {
            migrated: false,
            skipped,
        });
    }
    let (report, rejected) = cli::set_accepted(cli, &assignments)?;
    skipped.extend(rejected.iter().filter_map(|problem| skipped_of(problem, ini_values)));
    Ok(NaiomMigration {
        migrated: !report.changed.is_empty(),
        skipped,
    })
}
