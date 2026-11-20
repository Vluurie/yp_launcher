use yp_config::naiom_keys::{
    CONTROLLER_BUTTONS, FUNCTION_KEYS, KEYBOARD_ONLY_BINDINGS, MOUSE_BUTTONS, all, allows_mouse,
    combo_contains_non_keyboard, combo_parts, is_mouse_button, is_valid, is_valid_binding,
    key_name_to_vk, normalize_binding, vk_to_key_name,
};

#[test]
fn the_key_list_has_every_group_once() {
    let keys: Vec<&str> = all().collect();
    assert_eq!(keys.len(), 106);
    let unique: std::collections::BTreeSet<&str> = keys.iter().copied().collect();
    assert_eq!(unique.len(), keys.len());
}

#[test]
fn function_keys_f1_to_f12_are_valid_and_map_to_vk_codes() {
    for (index, key) in FUNCTION_KEYS.iter().enumerate() {
        assert!(is_valid(key));
        assert!(is_valid_binding(key));
        let vk = key_name_to_vk(key).unwrap();
        assert_eq!(vk, 0x70 + u32::try_from(index).unwrap());
        assert_eq!(vk_to_key_name(i64::from(vk)), Some(*key));
    }
    assert!(!is_valid("F13"));
}

#[test]
fn keyboard_keys_round_trip_through_vk_codes() {
    for key in all() {
        if MOUSE_BUTTONS.contains(&key)
            || CONTROLLER_BUTTONS.contains(&key)
            || key.starts_with("KP_") && key.len() > 4
        {
            continue;
        }
        let vk = key_name_to_vk(key).unwrap_or_else(|| panic!("{key} has no VK code"));
        assert_eq!(vk_to_key_name(i64::from(vk)), Some(key), "{key}");
    }
    assert_eq!(key_name_to_vk("kp_plus"), Some(0x6B));
    assert_eq!(key_name_to_vk(" k "), Some(0x4B));
    assert_eq!(key_name_to_vk("MOUSE1"), None);
    assert_eq!(key_name_to_vk("CAPSLOCK"), None);
    assert_eq!(key_name_to_vk("é"), None);
}

#[test]
fn generic_virtual_keys_map_to_their_left_side() {
    assert_eq!(vk_to_key_name(0x10), Some("LSHIFT"));
    assert_eq!(vk_to_key_name(0x11), Some("LCTRL"));
    assert_eq!(vk_to_key_name(0x12), Some("LALT"));
    assert_eq!(vk_to_key_name(0x52), Some("R"));
    assert_eq!(vk_to_key_name(19), None);
    assert_eq!(vk_to_key_name(-1), None);
}

#[test]
fn bindings_need_a_real_key_last_and_modifiers_before_it() {
    assert!(is_valid_binding(""));
    assert!(is_valid_binding("CTRL+X"));
    assert!(is_valid_binding("lctrl + x"));
    assert!(is_valid_binding("SHIFT+CONTROLLER_A"));
    assert!(is_valid_binding("A+B"));
    assert!(!is_valid_binding("X+CTRL"));
    assert!(!is_valid_binding("SHIFT"));
    assert!(!is_valid_binding("+"));
    assert!(!is_valid_binding("CAPSLOCK"));
    assert_eq!(combo_parts(" lctrl + x +"), vec!["LCTRL", "X"]);
    assert_eq!(normalize_binding("lctrl+x"), "LCTRL+X");
}

#[test]
fn mouse_and_controller_buttons_are_not_keyboard_keys() {
    assert!(combo_contains_non_keyboard("MOUSE2"));
    assert!(combo_contains_non_keyboard("ctrl+controller_lb"));
    assert!(!combo_contains_non_keyboard("CTRL+X"));
    assert!(is_mouse_button("mouse_wheel_up"));
    assert!(!is_mouse_button("CONTROLLER_A"));
    for key in CONTROLLER_BUTTONS {
        assert!(is_valid(key));
    }
}

#[test]
fn keyboard_only_bindings_reject_mouse_buttons() {
    assert_eq!(KEYBOARD_ONLY_BINDINGS.len(), 13);
    assert!(!allows_mouse("standard_walk"));
    assert!(!allows_mouse("standard_menu_up"));
    assert!(allows_mouse("standard_fire"));
    assert!(allows_mouse("non_standard_auto_fire"));
}
