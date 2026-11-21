use indexmap::IndexMap;
use serde_json::Value as Json;
use yp_core::GamePaths;
use yp_core::ini::{IniDoc, IniSyntax};

use crate::cli::{self, NamsCli};
use crate::error::Result;
use crate::reports::Values;

pub const LODMOD_INI_FILE: &str = "LodMod.ini";

pub type LodModIniValues = IndexMap<String, Json>;

pub const ENABLED_ID: &str = "lodmod.enabled";

const INI_KEYS: [(&str, &str); 14] = [
    ("LODMultiplier", "lodmod.lod_multiplier"),
    ("ShadowResolution", "lodmod.shadow_resolution"),
    ("ShadowDistanceMultiplier", "lodmod.shadow_distance_multiplier"),
    ("ShadowDistanceMinimum", "lodmod.shadow_distance_minimum"),
    ("ShadowDistanceMaximum", "lodmod.shadow_distance_maximum"),
    ("ShadowDistancePSS", "lodmod.shadow_distance_pss"),
    ("ShadowFilterStrengthBias", "lodmod.shadow_filter_strength_bias"),
    ("ShadowFilterStrengthMinimum", "lodmod.shadow_filter_strength_minimum"),
    ("ShadowFilterStrengthMaximum", "lodmod.shadow_filter_strength_maximum"),
    ("ShadowModelHQ", "lodmod.shadow_model_hq"),
    ("ShadowModelForceAll", "lodmod.shadow_model_force_all"),
    ("DisableManualCulling", "lodmod.disable_manual_culling"),
    ("DisableVignette", "lodmod.disable_vignette"),
    ("DisableHDR", "lodmod.disable_hdr"),
];

pub fn ini_to_setting_id(ini_key: &str) -> Option<&'static str> {
    INI_KEYS
        .iter()
        .find(|(ini, _)| *ini == ini_key)
        .map(|(_, id)| *id)
}

fn lodmod_ini_syntax() -> IniSyntax {
    IniSyntax {
        separators: vec!['='],
        comment_prefixes: vec![';'],
    }
}

fn parse_hex_or_decimal(text: &str) -> Option<i64> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let magnitude = match digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        Some(hex) if !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit()) => {
            i64::from_str_radix(hex, 16).ok()?
        }
        Some(_) => return None,
        None if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) => {
            digits.parse::<i64>().ok()?
        }
        None => return None,
    };
    Some(if negative { -magnitude } else { magnitude })
}

pub fn parse_ini_value(value: &str) -> Json {
    let lower = value.to_lowercase();
    if lower == "true" {
        return Json::Bool(true);
    }
    if lower == "false" {
        return Json::Bool(false);
    }
    if !value.contains('.')
        && let Some(number) = parse_hex_or_decimal(value)
    {
        return Json::from(number);
    }
    if let Ok(number) = value.parse::<f64>()
        && let Some(number) = serde_json::Number::from_f64(number)
    {
        return Json::Number(number);
    }
    Json::String(value.to_owned())
}

pub fn parse_lodmod_ini(content: &str) -> LodModIniValues {
    let syntax = lodmod_ini_syntax();
    let doc = IniDoc::parse_with(content, syntax.clone());
    let mut values = LodModIniValues::new();
    for (_, key, value) in doc.entries() {
        if ini_to_setting_id(key).is_none() {
            continue;
        }
        let value = syntax.strip_inline_comment(value);
        values.insert(key.to_owned(), parse_ini_value(value));
    }
    values
}

pub fn detect_lodmod_ini(game: &GamePaths) -> Result<Option<LodModIniValues>> {
    let path = game.root().join(LODMOD_INI_FILE);
    let Some(content) = yp_core::fs::read_text_opt(&path)? else {
        return Ok(None);
    };
    let values = parse_lodmod_ini(&content);
    Ok((!values.is_empty()).then_some(values))
}

pub fn lodmod_ini_assignments(ini_values: &LodModIniValues) -> Vec<(String, Json)> {
    let mut assignments = vec![(ENABLED_ID.to_owned(), Json::Bool(true))];
    for (ini_key, value) in ini_values {
        if let Some(id) = ini_to_setting_id(ini_key) {
            assignments.push((id.to_owned(), value.clone()));
        }
    }
    assignments
}

pub fn should_import_lodmod_ini(current: &Values) -> bool {
    current.get(ENABLED_ID).and_then(Json::as_bool) != Some(true)
}

pub fn import_lodmod_ini(cli: &dyn NamsCli, ini_values: &LodModIniValues) -> Result<bool> {
    let current = cli::get(cli)?;
    if !should_import_lodmod_ini(&current) {
        return Ok(false);
    }
    let (report, _) = cli::set_accepted(cli, &lodmod_ini_assignments(ini_values))?;
    Ok(!report.changed.is_empty())
}
