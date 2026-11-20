use crate::fields::bindings;
use crate::migrate::nams::MOUSE_SECTION;
use crate::templates;

pub const BINDINGS_SECTION: &str = "bindings";

pub fn legacy_mouse_values(nams_raw: Option<&str>) -> toml::Table {
    nams_raw
        .and_then(|raw| yp_core::toml_io::parse_values(raw).ok())
        .and_then(|values| {
            values
                .get(MOUSE_SECTION)
                .and_then(toml::Value::as_table)
                .cloned()
        })
        .unwrap_or_default()
}

pub fn initial_naiom(nams_raw: Option<&str>, evade_key_name: Option<&str>) -> String {
    let mut legacy = legacy_mouse_values(nams_raw);
    if let Some(name) = evade_key_name {
        let evade = bindings::NON_STANDARD_EVADE.key();
        let mut table = legacy
            .get(BINDINGS_SECTION)
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        let bound = table
            .get(evade)
            .and_then(toml::Value::as_str)
            .is_some_and(|current| !current.is_empty());
        if !bound {
            table.insert(evade.to_owned(), toml::Value::String(name.to_owned()));
            legacy.insert(BINDINGS_SECTION.to_owned(), toml::Value::Table(table));
        }
    }
    if legacy.is_empty() {
        return templates::NAIOM.to_owned();
    }
    match yp_core::toml_io::update_toml(templates::NAIOM, &legacy) {
        Ok(text) => text,
        Err(err) => {
            log::warn!("could not carry the old [mouse] settings into naiom.toml: {err}");
            templates::NAIOM.to_owned()
        }
    }
}
