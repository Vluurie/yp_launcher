use serde_json::{Map, Value, json};
use yp_config::fields::overlay::randomizer;
use yp_config::state::OverlayData;

fn data(settings: Value) -> OverlayData {
    OverlayData {
        settings: settings.as_object().cloned().unwrap_or_default(),
        ..OverlayData::default()
    }
}

#[test]
fn missing_workspace_keys_read_as_the_overlay_defaults() {
    let data = data(json!({}));
    assert!(!data.game_keybinds_global());
    assert!(!data.loading_speedup_enabled());
    assert!(data.shaders_enabled());
    assert!(data.sound_enabled());
    assert!(!data.editor_mcp_enabled());
}

#[test]
fn saved_workspace_values_win_over_the_defaults() {
    let data = data(json!({
        "shadersEnabled": false,
        "soundEnabled": false,
        "editorMcpEnabled": true,
    }));
    assert!(!data.shaders_enabled());
    assert!(!data.sound_enabled());
    assert!(data.editor_mcp_enabled());
}

#[test]
fn missing_randomizer_keys_read_as_the_overlay_defaults() {
    let randomizer_values = Map::new();
    assert!(!randomizer::AUTO_START_ON_STARTUP.value_in_json(&randomizer_values));
    for field in [
        &randomizer::RANDOMIZE_GROUND_ENEMIES,
        &randomizer::RANDOMIZE_FLYING_ENEMIES,
        &randomizer::ALLOW_BIG_ENEMIES,
        &randomizer::INCLUDE_DLC_ENEMIES,
    ] {
        assert!(field.value_in_json(&randomizer_values), "{}", field.key());
    }
}
