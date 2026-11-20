use yp_config::fields::lodmod;

fn values(text: &str) -> toml::Table {
    toml::from_str(text).unwrap()
}

#[test]
fn a_broken_toml_parses_to_an_error_instead_of_panicking() {
    let broken = "enabled = \nrings = [[[";
    assert!(yp_core::toml_io::parse_values(broken).is_err());
    assert!(
        yp_core::toml_io::parse_values(broken)
            .unwrap_or_default()
            .is_empty()
    );
}

#[test]
fn wrong_types_fall_back_to_the_default_instead_of_panicking() {
    let values = values(
        "enabled = \"yes\"
shadow_resolution = \"huge\"
lod_multiplier = true
shadow_blur_scale = 42
high_grids = \"not a table\"
",
    );
    assert!(!lodmod::ENABLED.value_in(&values));
    assert_eq!(lodmod::SHADOW_RESOLUTION.value_in(&values), 2048);
    assert_eq!(lodmod::LOD_MULTIPLIER.value_in(&values), 0.0);
    assert_eq!(
        lodmod::SHADOW_BLUR_SCALE.value_in(&values),
        vec![toml::Value::Float(1.0); 4]
    );
    assert!(!lodmod::HIGH_GRIDS_ENABLED.value_in(&values));
    assert_eq!(lodmod::HIGH_GRIDS_RINGS.value_in(&values), 1);
}

#[test]
fn missing_keys_and_empty_values_fall_back_to_the_default() {
    let empty = toml::Table::new();
    assert_eq!(lodmod::SHADOW_RESOLUTION.value_in(&empty), 2048);
    assert_eq!(lodmod::HIGH_GRIDS_RINGS.value_in(&empty), 1);
    assert!(lodmod::HIGH_GRIDS_ROOM_RINGS.value_in(&empty).is_empty());
}

#[test]
fn numbers_are_coerced_across_int_and_float() {
    assert_eq!(
        lodmod::SHADOW_RESOLUTION.value_in(&values("shadow_resolution = 4096.0")),
        4096
    );
    assert_eq!(
        lodmod::LOD_MULTIPLIER.value_in(&values("lod_multiplier = 1")),
        1.0
    );
    assert_eq!(
        lodmod::SHADOW_RESOLUTION.value_in(&values("shadow_resolution = 4096.9")),
        4096
    );
}

#[test]
fn section_values_are_read_out_of_their_table() {
    let values = values("[high_grids]\nenabled = true\nrings = 3\n");
    assert!(lodmod::HIGH_GRIDS_ENABLED.value_in(&values));
    assert_eq!(lodmod::HIGH_GRIDS_RINGS.value_in(&values), 3);
}
