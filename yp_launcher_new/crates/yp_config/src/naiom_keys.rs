pub const LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

pub const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

pub const SPECIAL: [&str; 21] = [
    "SPACE",
    "TAB",
    "ENTER",
    "ESCAPE",
    "BACKSPACE",
    "INS",
    "DEL",
    "HOME",
    "END",
    "PGUP",
    "PGDN",
    "UPARROW",
    "DOWNARROW",
    "LEFTARROW",
    "RIGHTARROW",
    "LSHIFT",
    "RSHIFT",
    "LCTRL",
    "RCTRL",
    "LALT",
    "RALT",
];

pub const FUNCTION_KEYS: [&str; 12] = [
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
];

pub const KEYPAD: [&str; 14] = [
    "KP_0", "KP_1", "KP_2", "KP_3", "KP_4", "KP_5", "KP_6", "KP_7", "KP_8", "KP_9", "KP_PLUS",
    "KP_MINUS", "KP_SLASH", "KP_STAR",
];

pub const MOUSE_BUTTONS: [&str; 7] = [
    "MOUSE1",
    "MOUSE2",
    "MOUSE3",
    "MOUSE4",
    "MOUSE5",
    "MOUSE_WHEEL_UP",
    "MOUSE_WHEEL_DOWN",
];

pub const CONTROLLER_BUTTONS: [&str; 16] = [
    "CONTROLLER_A",
    "CONTROLLER_B",
    "CONTROLLER_X",
    "CONTROLLER_Y",
    "CONTROLLER_LB",
    "CONTROLLER_RB",
    "CONTROLLER_LT",
    "CONTROLLER_RT",
    "CONTROLLER_LTHUMB",
    "CONTROLLER_RTHUMB",
    "CONTROLLER_DUP",
    "CONTROLLER_DDOWN",
    "CONTROLLER_DLEFT",
    "CONTROLLER_DRIGHT",
    "CONTROLLER_BACK",
    "CONTROLLER_START",
];

pub const GENERIC_MODIFIERS: [&str; 3] = ["SHIFT", "CTRL", "ALT"];

pub const SIDE_MODIFIERS: [&str; 6] = ["LSHIFT", "RSHIFT", "LCTRL", "RCTRL", "LALT", "RALT"];

pub const KEYBOARD_ONLY_BINDINGS: [&str; 13] = [
    "standard_move_forward",
    "standard_move_backward",
    "standard_move_left",
    "standard_move_right",
    "standard_auto_run",
    "standard_light",
    "standard_self_destruct",
    "standard_walk",
    "standard_reset_camera",
    "standard_menu_up",
    "standard_menu_down",
    "standard_menu_left",
    "standard_menu_right",
];

pub fn all() -> impl Iterator<Item = &'static str> {
    LETTERS
        .iter()
        .chain(DIGITS.iter())
        .chain(SPECIAL.iter())
        .chain(FUNCTION_KEYS.iter())
        .chain(KEYPAD.iter())
        .chain(MOUSE_BUTTONS.iter())
        .chain(CONTROLLER_BUTTONS.iter())
        .copied()
}

pub fn is_key_name(name: &str) -> bool {
    all().any(|key| key == name)
}

pub fn is_modifier(name: &str) -> bool {
    GENERIC_MODIFIERS.contains(&name) || SIDE_MODIFIERS.contains(&name)
}

pub fn combo_parts(binding: &str) -> Vec<String> {
    binding
        .split('+')
        .map(|part| part.trim().to_uppercase())
        .filter(|part| !part.is_empty())
        .collect()
}

pub fn normalize_binding(binding: &str) -> String {
    combo_parts(binding).join("+")
}

pub fn is_valid_binding(binding: &str) -> bool {
    if binding.is_empty() {
        return true;
    }
    let parts = combo_parts(binding);
    let Some((last, modifiers)) = parts.split_last() else {
        return false;
    };
    is_key_name(last)
        && modifiers
            .iter()
            .all(|part| is_modifier(part) || is_key_name(part))
}

pub fn combo_contains_non_keyboard(binding: &str) -> bool {
    combo_parts(binding).iter().any(|part| {
        MOUSE_BUTTONS.contains(&part.as_str()) || CONTROLLER_BUTTONS.contains(&part.as_str())
    })
}

pub fn is_mouse_button(key_name: &str) -> bool {
    MOUSE_BUTTONS.contains(&key_name.to_uppercase().as_str())
}

pub fn is_valid(key_name: &str) -> bool {
    key_name.is_empty() || is_key_name(&key_name.trim().to_uppercase())
}

pub fn allows_mouse(binding_key: &str) -> bool {
    !KEYBOARD_ONLY_BINDINGS.contains(&binding_key)
}

pub fn key_name_to_vk(key_name: &str) -> Option<u32> {
    let upper = key_name.trim().to_uppercase();
    let mut chars = upper.chars();
    if let (Some(single), None) = (chars.next(), chars.next()) {
        return (single.is_ascii_uppercase() || single.is_ascii_digit())
            .then_some(u32::from(single));
    }
    let named = match upper.as_str() {
        "SPACE" => Some(0x20),
        "END" => Some(0x23),
        "LEFTARROW" => Some(0x25),
        "UPARROW" => Some(0x26),
        "RIGHTARROW" => Some(0x27),
        "DOWNARROW" => Some(0x28),
        "LSHIFT" => Some(0xA0),
        "RSHIFT" => Some(0xA1),
        "LCTRL" => Some(0xA2),
        "RCTRL" => Some(0xA3),
        "LALT" => Some(0xA4),
        "RALT" => Some(0xA5),
        "TAB" => Some(0x09),
        "ENTER" => Some(0x0D),
        "ESCAPE" => Some(0x1B),
        "BACKSPACE" => Some(0x08),
        "INS" => Some(0x2D),
        "DEL" => Some(0x2E),
        "HOME" => Some(0x24),
        "PGUP" => Some(0x21),
        "PGDN" => Some(0x22),
        "KP_PLUS" => Some(0x6B),
        "KP_MINUS" => Some(0x6D),
        "KP_SLASH" => Some(0x6F),
        "KP_STAR" => Some(0x6A),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    if let Some(index) = FUNCTION_KEYS.iter().position(|key| *key == upper) {
        return u32::try_from(index).ok().map(|index| 0x70 + index);
    }
    KEYPAD
        .iter()
        .take(10)
        .position(|key| *key == upper)
        .and_then(|index| u32::try_from(index).ok())
        .map(|index| 0x60 + index)
}

pub fn vk_to_key_name(vk: i64) -> Option<&'static str> {
    let index = |base: i64| usize::try_from(vk - base).ok();
    match vk {
        0x41..=0x5A => index(0x41).map(|i| LETTERS[i]),
        0x30..=0x39 => index(0x30).map(|i| DIGITS[i]),
        0x70..=0x7B => index(0x70).map(|i| FUNCTION_KEYS[i]),
        0x60..=0x69 => index(0x60).map(|i| KEYPAD[i]),
        0x12 | 0xA4 => Some("LALT"),
        0xA5 => Some("RALT"),
        0x09 => Some("TAB"),
        0x0D => Some("ENTER"),
        0x1B => Some("ESCAPE"),
        0x08 => Some("BACKSPACE"),
        0x2D => Some("INS"),
        0x2E => Some("DEL"),
        0x24 => Some("HOME"),
        0x21 => Some("PGUP"),
        0x22 => Some("PGDN"),
        0x6B => Some("KP_PLUS"),
        0x6D => Some("KP_MINUS"),
        0x6F => Some("KP_SLASH"),
        0x6A => Some("KP_STAR"),
        0x20 => Some("SPACE"),
        0x23 => Some("END"),
        0x25 => Some("LEFTARROW"),
        0x26 => Some("UPARROW"),
        0x27 => Some("RIGHTARROW"),
        0x28 => Some("DOWNARROW"),
        0x10 | 0xA0 => Some("LSHIFT"),
        0xA1 => Some("RSHIFT"),
        0x11 | 0xA2 => Some("LCTRL"),
        0xA3 => Some("RCTRL"),
        _ => None,
    }
}
