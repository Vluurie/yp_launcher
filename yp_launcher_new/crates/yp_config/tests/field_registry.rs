use std::collections::BTreeSet;

use yp_config::fields::{
    self, AnyField, FieldKind, bindings, content, lodmod, nams, overlay, texture_injection,
};

#[test]
fn registry_sizes_match_the_dart_classes() {
    assert_eq!(nams::ALL.len(), 43);
    assert_eq!(nams::MOUSE.len(), 20);
    assert_eq!(content::fields().count(), 35);
    assert_eq!(bindings::ALL.len(), 35);
    assert_eq!(lodmod::ALL.len(), 51);
    assert_eq!(texture_injection::ALL.len(), 7);
    assert_eq!(overlay::all().len(), 29);
    assert_eq!(fields::all().len(), 43 + 35 + 35 + 51 + 7 + 29);
}

fn assert_unique(name: &str, fields: impl Iterator<Item = &'static dyn AnyField>) {
    let mut seen = BTreeSet::new();
    for field in fields {
        let meta = field.meta();
        assert!(
            seen.insert((meta.section, meta.key)),
            "{name}: {:?} {} twice",
            meta.section,
            meta.key
        );
    }
}

#[test]
fn keys_are_unique_per_file_and_section() {
    assert_unique(
        "nams",
        nams::ALL
            .iter()
            .copied()
            .chain(content::all())
            .chain(bindings::all()),
    );
    assert_unique("lodmod", lodmod::ALL.iter().copied());
    assert_unique("texture_injection", texture_injection::ALL.iter().copied());
    assert_unique("overlay", overlay::all().into_iter());
}

#[test]
fn defaults_respect_ranges_and_allowed_values() {
    for field in fields::all() {
        let meta = field.meta();
        let default = field.default_toml();
        let number = match default {
            toml::Value::Integer(value) => Some(value as f64),
            toml::Value::Float(value) => Some(value),
            _ => None,
        };
        let Some(number) = number else {
            continue;
        };
        if let (Some(min), Some(max)) = (meta.min, meta.max) {
            assert!(
                number >= min && number <= max,
                "{} default {number} outside {min}..{max}",
                meta.key
            );
        }
        if !meta.allowed.is_empty() {
            assert!(
                meta.allowed.contains(&number),
                "{} default {number} not allowed",
                meta.key
            );
        }
    }
}

#[test]
fn kinds_follow_the_dart_types() {
    let kind = |field: &dyn AnyField| field.kind();
    assert_eq!(kind(&nams::SENSITIVITY), FieldKind::Float);
    assert_eq!(kind(&nams::MISC_OPEN_DEBUG_MENU), FieldKind::Int);
    assert_eq!(kind(&nams::MISC_CUSTOM_CURSOR_MENU), FieldKind::Text);
    assert_eq!(kind(&lodmod::SHADOW_BLUR_SCALE), FieldKind::List);
    assert_eq!(kind(&texture_injection::LOAD_ORDER), FieldKind::TextList);
    assert_eq!(kind(&lodmod::FXAA), FieldKind::Bool);
}

#[test]
fn sections_and_metadata_match_the_dart_registry() {
    assert_eq!(bindings::STANDARD_FIRE.section(), Some("mouse.bindings"));
    assert_eq!(
        bindings::STANDARD_FIRE.meta.path(),
        vec!["mouse", "bindings", "standard_fire"]
    );
    assert_eq!(nams::HIDE_SUBTITLE_OVERLAY.section(), Some("cutscene"));
    assert_eq!(lodmod::HIGH_GRIDS_RINGS.meta.allowed, &[1.0, 2.0, 3.0, 4.0]);
    assert!(lodmod::HIGH_GRIDS_RINGS.meta.restart_required);
    assert_eq!(lodmod::SHADOW_CASCADE_RANGE.meta.min, Some(1.05));
    assert_eq!(lodmod::FPS_LIMIT.meta.step, None);
    assert_eq!(lodmod::FPS_LIMIT.meta.max, Some(1000.0));
    assert_eq!(texture_injection::LOAD_ORDER.meta.tooltip, None);
    assert_eq!(overlay::keybinds::OPEN_WORKSPACE.section(), Some("main"));
    assert_eq!(overlay::keybinds::OPEN_WORKSPACE.default_value(), "F1");
    assert_eq!(overlay::workspace::IMPELLER.key(), "impeller");
    assert!(overlay::workspace::IMPELLER.meta.restart_required);
    assert_eq!(lodmod::high_grids_cell_count(3), Some(37));
    assert_eq!(lodmod::high_grids_cell_count(5), None);
    assert_eq!(lodmod::HIGH_GRIDS_ROOMS.len(), 10);
    assert_eq!(
        lodmod::bloom_2017_preset()["bloom_reference_height"].as_integer(),
        Some(900)
    );
}

#[test]
fn content_groups_cover_every_feature_once() {
    let grouped: Vec<&str> = content::GROUPS
        .iter()
        .flat_map(|group| group.fields.iter().map(|field| field.key()))
        .collect();
    assert_eq!(grouped.len(), 35);
    assert_eq!(grouped.iter().collect::<BTreeSet<_>>().len(), 35);
    assert_eq!(
        content::GROUPS.map(|group| group.fields.len()),
        [13, 9, 3, 3, 7]
    );
}
