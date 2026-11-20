use indexmap::IndexMap;
use yp_core::GamePaths;
use yp_core::ini::{IniDoc, IniSyntax};

use crate::error::Result;
use crate::fields::lodmod;

pub const LODMOD_INI_FILE: &str = "LodMod.ini";

pub type LodModIniValues = IndexMap<String, toml::Value>;

pub fn ini_to_toml_key(ini_key: &str) -> Option<&'static str> {
    let key = match ini_key {
        "LODMultiplier" => lodmod::LOD_MULTIPLIER.key(),
        "ShadowResolution" => lodmod::SHADOW_RESOLUTION.key(),
        "ShadowDistanceMultiplier" => lodmod::SHADOW_DISTANCE_MULTIPLIER.key(),
        "ShadowDistanceMinimum" => lodmod::SHADOW_DISTANCE_MINIMUM.key(),
        "ShadowDistanceMaximum" => lodmod::SHADOW_DISTANCE_MAXIMUM.key(),
        "ShadowDistancePSS" => lodmod::SHADOW_DISTANCE_PSS.key(),
        "ShadowFilterStrengthBias" => lodmod::SHADOW_FILTER_STRENGTH_BIAS.key(),
        "ShadowFilterStrengthMinimum" => lodmod::SHADOW_FILTER_STRENGTH_MINIMUM.key(),
        "ShadowFilterStrengthMaximum" => lodmod::SHADOW_FILTER_STRENGTH_MAXIMUM.key(),
        "ShadowModelHQ" => lodmod::SHADOW_MODEL_HQ.key(),
        "ShadowModelForceAll" => lodmod::SHADOW_MODEL_FORCE_ALL.key(),
        "DisableManualCulling" => lodmod::DISABLE_MANUAL_CULLING.key(),
        "DisableVignette" => lodmod::DISABLE_VIGNETTE.key(),
        "DisableHDR" => lodmod::DISABLE_HDR.key(),
        _ => return None,
    };
    Some(key)
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

pub fn parse_ini_value(value: &str) -> toml::Value {
    let lower = value.to_lowercase();
    if lower == "true" {
        return toml::Value::Boolean(true);
    }
    if lower == "false" {
        return toml::Value::Boolean(false);
    }
    if !value.contains('.')
        && let Some(number) = parse_hex_or_decimal(value)
    {
        return toml::Value::Integer(number);
    }
    if let Ok(number) = value.parse::<f64>() {
        return toml::Value::Float(number);
    }
    toml::Value::String(value.to_owned())
}

pub fn parse_lodmod_ini(content: &str) -> LodModIniValues {
    let syntax = lodmod_ini_syntax();
    let doc = IniDoc::parse_with(content, syntax.clone());
    let mut values = LodModIniValues::new();
    for (_, key, value) in doc.entries() {
        if ini_to_toml_key(key).is_none() {
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

pub fn migrate_lodmod_ini(game: &GamePaths, ini_values: &LodModIniValues) -> Result<bool> {
    let path = game.lodmod_toml();
    let raw = yp_core::fs::read_text_opt(&path)?.unwrap_or_default();
    if raw.is_empty() {
        return Ok(false);
    }
    let current = yp_core::toml_io::parse_values(&raw).unwrap_or_default();
    if current
        .get(lodmod::ENABLED.key())
        .and_then(toml::Value::as_bool)
        == Some(true)
    {
        return Ok(false);
    }
    let mut updates = toml::Table::new();
    updates.insert(lodmod::ENABLED.key().to_owned(), toml::Value::Boolean(true));
    for (ini_key, value) in ini_values {
        if let Some(toml_key) = ini_to_toml_key(ini_key) {
            updates.insert(toml_key.to_owned(), value.clone());
        }
    }
    let updated = yp_core::toml_io::update_toml(&raw, &updates)?;
    yp_core::fs::write_text_atomic(&path, &updated)?;
    Ok(true)
}
