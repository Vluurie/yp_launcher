use serde_json::json;
use yp_config::fields::overlay::keybinds;
use yp_config::state::OverlayStore;

#[test]
fn a_changed_keybind_is_read_back_instead_of_the_default() {
    let mut store = OverlayStore::new(None);
    for (field, key) in [
        (&keybinds::OPEN_WORKSPACE, "F3"),
        (&keybinds::FREEZE_GAME, "G"),
        (&keybinds::DEV_MODE, "Home"),
    ] {
        store.update_keybind(field.section().unwrap(), field.key(), key);
        let binds = store.data().keybinds();
        assert_eq!(field.value_in_json(&binds), key);
    }
    assert!(store.data().has_unsaved_changes);
}

#[test]
fn keybinds_loaded_from_settings_json_show_their_saved_value() {
    let keybinds_json = json!({
        "main": {"yorha_protocol": "F2"},
        "yorha_protocol": {"free_cam": "K"},
    });
    let binds = keybinds_json.as_object().unwrap();
    assert_eq!(keybinds::OPEN_WORKSPACE.value_in_json(binds), "F2");
    assert_eq!(keybinds::FREE_CAM.value_in_json(binds), "K");
    assert_eq!(keybinds::MAX_SPEED.value_in_json(binds), "F6");
}
