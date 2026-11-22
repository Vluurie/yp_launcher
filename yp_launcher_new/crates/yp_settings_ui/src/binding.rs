use std::path::Path;

use yp_config::naiom_keys;
use yp_i18n::Strings;

pub const COMBO_SEPARATOR: &str = "+";
pub const PENDING_SUFFIX: &str = "…";
pub const VK_DISABLED: i64 = 0;
pub const VK_SHIFT: i64 = 0x10;
pub const VK_CONTROL: i64 = 0x11;
pub const VK_ALT: i64 = 0x12;

const FUNCTION_KEY_COUNT: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    Shift,
    Ctrl,
    Alt,
}

impl Modifier {
    pub const ALL: [Modifier; 3] = [Modifier::Shift, Modifier::Ctrl, Modifier::Alt];

    pub fn generic(self) -> &'static str {
        match self {
            Modifier::Shift => "SHIFT",
            Modifier::Ctrl => "CTRL",
            Modifier::Alt => "ALT",
        }
    }

    pub fn side(self) -> &'static str {
        match self {
            Modifier::Shift => "LSHIFT",
            Modifier::Ctrl => "LCTRL",
            Modifier::Alt => "LALT",
        }
    }

    pub fn virtual_key(self) -> i64 {
        match self {
            Modifier::Shift => VK_SHIFT,
            Modifier::Ctrl => VK_CONTROL,
            Modifier::Alt => VK_ALT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl ModifierState {
    pub fn is_down(&self, modifier: Modifier) -> bool {
        match modifier {
            Modifier::Shift => self.shift,
            Modifier::Ctrl => self.ctrl,
            Modifier::Alt => self.alt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl PointerButton {
    pub fn key_name(self) -> &'static str {
        match self {
            PointerButton::Left => "MOUSE1",
            PointerButton::Right => "MOUSE2",
            PointerButton::Middle => "MOUSE3",
            PointerButton::Back => "MOUSE4",
            PointerButton::Forward => "MOUSE5",
        }
    }
}

pub fn key_name(key: &str) -> Option<String> {
    let mut chars = key.chars();
    if let (Some(single), None) = (chars.next(), chars.next()) {
        return (single.is_ascii_alphanumeric()).then(|| single.to_ascii_uppercase().to_string());
    }
    if let Some(number) = key.strip_prefix('f')
        && let Ok(number) = number.parse::<u32>()
    {
        return (1..=FUNCTION_KEY_COUNT)
            .contains(&number)
            .then(|| format!("F{number}"));
    }
    let named = match key {
        "space" => "SPACE",
        "end" => "END",
        "up" => "UPARROW",
        "down" => "DOWNARROW",
        "left" => "LEFTARROW",
        "right" => "RIGHTARROW",
        "tab" => "TAB",
        "enter" => "ENTER",
        "backspace" => "BACKSPACE",
        "insert" => "INS",
        "delete" => "DEL",
        "home" => "HOME",
        "pageup" => "PGUP",
        "pagedown" => "PGDN",
        _ => return None,
    };
    Some(named.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Capture {
    held: Vec<Modifier>,
}

impl Capture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn held(&self) -> &[Modifier] {
        &self.held
    }

    fn combo(&self, name: &str) -> String {
        let mut parts: Vec<&str> = self.held.iter().map(|modifier| modifier.generic()).collect();
        parts.push(name);
        parts.join(COMBO_SEPARATOR)
    }

    pub fn pending_label(&self) -> Option<String> {
        if self.held.is_empty() {
            return None;
        }
        Some(self.combo(PENDING_SUFFIX))
    }

    pub fn sync_modifiers(&mut self, state: ModifierState) {
        for modifier in Modifier::ALL {
            let held = self.held.contains(&modifier);
            if state.is_down(modifier) && !held {
                self.held.push(modifier);
            } else if !state.is_down(modifier) && held {
                self.held.retain(|other| *other != modifier);
            }
        }
    }

    pub fn key_down_with(&mut self, state: ModifierState, key: &str) -> Option<String> {
        self.sync_modifiers(state);
        self.key_down(key)
    }

    pub fn key_down(&mut self, key: &str) -> Option<String> {
        if key == "escape" {
            self.held.clear();
            return Some(String::new());
        }
        let name = key_name(key)?;
        let binding = self.combo(&name);
        self.held.clear();
        Some(binding)
    }

    pub fn modifiers_changed(&mut self, state: ModifierState) -> Option<String> {
        for modifier in Modifier::ALL {
            if state.is_down(modifier) && !self.held.contains(&modifier) {
                self.held.push(modifier);
            }
        }
        for modifier in Modifier::ALL {
            if state.is_down(modifier) {
                continue;
            }
            let Some(index) = self.held.iter().position(|held| *held == modifier) else {
                continue;
            };
            if self.held.len() == 1 {
                self.held.clear();
                return Some(modifier.side().to_owned());
            }
            self.held.remove(index);
        }
        None
    }

    pub fn pointer_down(&mut self, button: PointerButton, allows_mouse: bool) -> Option<String> {
        if !allows_mouse {
            return None;
        }
        let binding = self.combo(button.key_name());
        self.held.clear();
        Some(binding)
    }

    pub fn wheel(&mut self, delta_y: f32, allows_mouse: bool) -> Option<String> {
        if !allows_mouse || delta_y == 0. {
            return None;
        }
        let name = if delta_y > 0. {
            "MOUSE_WHEEL_UP"
        } else {
            "MOUSE_WHEEL_DOWN"
        };
        let binding = self.combo(name);
        self.held.clear();
        Some(binding)
    }
}

pub fn controller_label(name: &str, strings: &'static Strings) -> Option<&'static str> {
    Some(match name {
        "CONTROLLER_A" => strings.controller_a(),
        "CONTROLLER_B" => strings.controller_b(),
        "CONTROLLER_X" => strings.controller_x(),
        "CONTROLLER_Y" => strings.controller_y(),
        "CONTROLLER_LB" => strings.controller_lb(),
        "CONTROLLER_RB" => strings.controller_rb(),
        "CONTROLLER_LT" => strings.controller_lt(),
        "CONTROLLER_RT" => strings.controller_rt(),
        "CONTROLLER_LTHUMB" => strings.controller_l_thumb(),
        "CONTROLLER_RTHUMB" => strings.controller_r_thumb(),
        "CONTROLLER_DUP" => strings.controller_d_up(),
        "CONTROLLER_DDOWN" => strings.controller_d_down(),
        "CONTROLLER_DLEFT" => strings.controller_d_left(),
        "CONTROLLER_DRIGHT" => strings.controller_d_right(),
        "CONTROLLER_BACK" => strings.controller_back(),
        "CONTROLLER_START" => strings.controller_start(),
        _ => return None,
    })
}

pub fn is_controller_binding(binding: &str) -> bool {
    naiom_keys::combo_parts(binding)
        .iter()
        .any(|part| naiom_keys::CONTROLLER_BUTTONS.contains(&part.as_str()))
}

pub fn display_binding(binding: &str, strings: &'static Strings) -> String {
    if binding.is_empty() {
        return strings.keybind_unbound().to_owned();
    }
    if !is_controller_binding(binding) {
        return binding.to_owned();
    }
    naiom_keys::combo_parts(binding)
        .iter()
        .map(|part| {
            controller_label(part, strings)
                .map(str::to_owned)
                .unwrap_or_else(|| part.clone())
        })
        .collect::<Vec<_>>()
        .join(COMBO_SEPARATOR)
}

pub fn conflict_for<'a>(
    key: &str,
    bindings: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Option<&'a str> {
    let bindings: Vec<(&str, &str)> = bindings.into_iter().collect();
    let own = bindings
        .iter()
        .find(|(other, _)| *other == key)
        .map(|(_, value)| *value)?;
    if own.is_empty() {
        return None;
    }
    bindings
        .iter()
        .find(|(other, value)| *other != key && *value == own)
        .map(|(other, _)| *other)
}

pub fn virtual_key_of(key: &str) -> Option<i64> {
    let mut chars = key.chars();
    if let (Some(single), None) = (chars.next(), chars.next()) {
        if single.is_ascii_alphanumeric() {
            return Some(i64::from(u32::from(single.to_ascii_uppercase())));
        }
        return Some(match single {
            ';' => 0xBA,
            '=' => 0xBB,
            ',' => 0xBC,
            '-' => 0xBD,
            '.' => 0xBE,
            '/' => 0xBF,
            '`' => 0xC0,
            '[' => 0xDB,
            '\\' => 0xDC,
            ']' => 0xDD,
            '\'' => 0xDE,
            _ => return None,
        });
    }
    if let Some(number) = key.strip_prefix('f')
        && let Ok(number) = number.parse::<u32>()
    {
        return (1..=FUNCTION_KEY_COUNT)
            .contains(&number)
            .then(|| i64::from(0x70 + number - 1));
    }
    Some(match key {
        "backspace" => 0x08,
        "tab" => 0x09,
        "enter" => 0x0D,
        "space" => 0x20,
        "pageup" => 0x21,
        "pagedown" => 0x22,
        "end" => 0x23,
        "home" => 0x24,
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        "insert" => 0x2D,
        "delete" => 0x2E,
        _ => return None,
    })
}

pub fn virtual_key_from_key_down(key: &str) -> Option<i64> {
    if key == "escape" {
        return Some(VK_DISABLED);
    }
    virtual_key_of(key)
}

pub fn virtual_key_from_modifiers(state: ModifierState) -> Option<i64> {
    Modifier::ALL
        .into_iter()
        .find(|modifier| state.is_down(*modifier))
        .map(Modifier::virtual_key)
}

pub fn virtual_key_name(vk: i64) -> Option<String> {
    let fixed = match vk {
        0x08 => "Backspace",
        0x09 => "Tab",
        0x0D => "Enter",
        0x10 => "Shift",
        0x11 => "Ctrl",
        0x12 => "Alt",
        0x13 => "Pause",
        0x14 => "CapsLock",
        0x1B => "Escape",
        0x20 => "Space",
        0x21 => "PageUp",
        0x22 => "PageDown",
        0x23 => "End",
        0x24 => "Home",
        0x25 => "Left",
        0x26 => "Up",
        0x27 => "Right",
        0x28 => "Down",
        0x2D => "Insert",
        0x2E => "Delete",
        0x6A => "Num*",
        0x6B => "Num+",
        0x6D => "Num-",
        0x6E => "Num.",
        0x6F => "Num/",
        0x90 => "NumLock",
        0x91 => "ScrollLock",
        0xBA => ";",
        0xBB => "=",
        0xBC => ",",
        0xBD => "-",
        0xBE => ".",
        0xBF => "/",
        0xC0 => "`",
        0xDB => "[",
        0xDC => "\\",
        0xDD => "]",
        0xDE => "'",
        _ => "",
    };
    if !fixed.is_empty() {
        return Some(fixed.to_owned());
    }
    let code = u32::try_from(vk).ok()?;
    match code {
        0x30..=0x39 | 0x41..=0x5A => char::from_u32(code).map(|key| key.to_string()),
        0x60..=0x69 => Some(format!("Num{}", code - 0x60)),
        0x70..=0x7B => Some(format!("F{}", code - 0x70 + 1)),
        _ => None,
    }
}

pub fn display_virtual_key(vk: i64, strings: &'static Strings) -> String {
    if vk == VK_DISABLED {
        return strings.naiom_key_disabled().to_owned();
    }
    virtual_key_name(vk).unwrap_or_else(|| format!("0x{vk:X}"))
}

pub fn cursor_display_name(stored: &str) -> &str {
    stored
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(stored)
}

pub fn is_valid_cursor_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_lowercase());
    let expected: [u8; 4] = match extension.as_deref() {
        Some("cur") => [0x00, 0x00, 0x02, 0x00],
        Some("ani") => *b"RIFF",
        _ => return false,
    };
    let mut header = [0u8; 4];
    let read = std::fs::File::open(path)
        .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut header));
    read.is_ok() && header == expected
}

pub fn stored_cursor_path(path: &Path, through_wine: bool) -> String {
    let text = path.to_string_lossy();
    if through_wine {
        yp_core::wine_paths::to_wine_path(&text)
    } else {
        text.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use yp_i18n::Locale;

    use super::*;

    fn en() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    fn down(shift: bool, ctrl: bool, alt: bool) -> ModifierState {
        ModifierState { shift, ctrl, alt }
    }

    #[test]
    fn plain_keys_become_naiom_key_names() {
        assert_eq!(key_name("w").as_deref(), Some("W"));
        assert_eq!(key_name("5").as_deref(), Some("5"));
        assert_eq!(key_name("f12").as_deref(), Some("F12"));
        assert_eq!(key_name("f13"), None);
        assert_eq!(key_name("pageup").as_deref(), Some("PGUP"));
        assert_eq!(key_name("up").as_deref(), Some("UPARROW"));
        assert_eq!(key_name("!"), None);
        assert_eq!(key_name("menu"), None);
        for key in ["a", "0", "f1", "space", "end", "tab", "enter", "insert", "delete", "home"] {
            let name = key_name(key).unwrap();
            assert!(naiom_keys::is_key_name(&name), "{name}");
        }
    }

    #[test]
    fn a_key_finishes_the_capture_with_the_held_modifiers() {
        let mut capture = Capture::new();
        assert_eq!(capture.pending_label(), None);
        assert_eq!(capture.modifiers_changed(down(true, false, false)), None);
        assert_eq!(capture.pending_label().as_deref(), Some("SHIFT+…"));
        assert_eq!(capture.modifiers_changed(down(true, true, false)), None);
        assert_eq!(capture.pending_label().as_deref(), Some("SHIFT+CTRL+…"));
        let binding = capture.key_down("e").unwrap();
        assert_eq!(binding, "SHIFT+CTRL+E");
        assert!(naiom_keys::is_valid_binding(&binding));
        assert!(capture.held().is_empty());
    }

    #[test]
    fn modifiers_reported_with_the_key_count_even_without_a_change_event() {
        let mut capture = Capture::new();
        assert_eq!(
            capture.key_down_with(down(true, false, true), "f4").as_deref(),
            Some("SHIFT+ALT+F4")
        );
        let mut capture = Capture::new();
        capture.modifiers_changed(down(false, true, false));
        assert_eq!(
            capture.key_down_with(down(false, false, false), "q").as_deref(),
            Some("Q")
        );
    }

    #[test]
    fn releasing_the_only_modifier_binds_the_modifier_itself() {
        let mut capture = Capture::new();
        capture.modifiers_changed(down(false, false, true));
        assert_eq!(
            capture.modifiers_changed(down(false, false, false)).as_deref(),
            Some("LALT")
        );
        let mut capture = Capture::new();
        capture.modifiers_changed(down(true, true, false));
        assert_eq!(capture.modifiers_changed(down(false, true, false)), None);
        assert_eq!(capture.held(), &[Modifier::Ctrl]);
        assert_eq!(
            capture.modifiers_changed(down(false, false, false)).as_deref(),
            Some("LCTRL")
        );
    }

    #[test]
    fn escape_clears_and_unknown_keys_are_ignored() {
        let mut capture = Capture::new();
        capture.modifiers_changed(down(true, false, false));
        assert_eq!(capture.key_down("menu"), None);
        assert_eq!(capture.held(), &[Modifier::Shift]);
        assert_eq!(capture.key_down("escape").as_deref(), Some(""));
        assert!(capture.held().is_empty());
    }

    #[test]
    fn mouse_input_is_refused_for_keyboard_only_bindings() {
        let mut capture = Capture::new();
        assert_eq!(capture.pointer_down(PointerButton::Left, false), None);
        assert_eq!(capture.wheel(1., false), None);
        assert_eq!(
            capture.pointer_down(PointerButton::Back, true).as_deref(),
            Some("MOUSE4")
        );
        capture.modifiers_changed(down(false, true, false));
        assert_eq!(capture.wheel(3., true).as_deref(), Some("CTRL+MOUSE_WHEEL_UP"));
        assert_eq!(capture.wheel(-3., true).as_deref(), Some("MOUSE_WHEEL_DOWN"));
        assert_eq!(capture.wheel(0., true), None);
        assert!(!naiom_keys::allows_mouse("standard_move_forward"));
        assert!(naiom_keys::allows_mouse("standard_jump"));
    }

    #[test]
    fn controller_buttons_are_shown_with_readable_names() {
        let strings = en();
        for button in naiom_keys::CONTROLLER_BUTTONS {
            assert!(controller_label(button, strings).is_some(), "{button}");
        }
        assert_eq!(display_binding("", strings), strings.keybind_unbound());
        assert_eq!(display_binding("SHIFT+E", strings), "SHIFT+E");
        assert_eq!(
            display_binding("CONTROLLER_A", strings),
            strings.controller_a()
        );
        assert_eq!(
            display_binding("CONTROLLER_LB+CONTROLLER_A", strings),
            format!("{}+{}", strings.controller_lb(), strings.controller_a())
        );
        assert!(is_controller_binding("CONTROLLER_START"));
        assert!(!is_controller_binding("MOUSE1"));
    }

    #[test]
    fn conflicts_name_the_other_binding_with_the_same_value() {
        let bindings = [
            ("standard_jump", "SPACE"),
            ("standard_walk", ""),
            ("standard_fire", "SPACE"),
            ("standard_use", "E"),
            ("standard_light", ""),
        ];
        assert_eq!(conflict_for("standard_jump", bindings), Some("standard_fire"));
        assert_eq!(conflict_for("standard_fire", bindings), Some("standard_jump"));
        assert_eq!(conflict_for("standard_use", bindings), None);
        assert_eq!(conflict_for("standard_walk", bindings), None);
        assert_eq!(conflict_for("missing", bindings), None);
    }

    #[test]
    fn virtual_keys_round_trip_through_their_names() {
        let strings = en();
        assert_eq!(virtual_key_from_key_down("escape"), Some(VK_DISABLED));
        assert_eq!(virtual_key_from_key_down("f1"), Some(0x70));
        assert_eq!(virtual_key_from_key_down("a"), Some(0x41));
        assert_eq!(virtual_key_from_key_down("9"), Some(0x39));
        assert_eq!(virtual_key_from_key_down("`"), Some(0xC0));
        assert_eq!(virtual_key_from_key_down("menu"), None);
        assert_eq!(
            virtual_key_from_modifiers(down(false, true, false)),
            Some(VK_CONTROL)
        );
        assert_eq!(virtual_key_from_modifiers(down(false, false, false)), None);
        assert_eq!(display_virtual_key(0, strings), strings.naiom_key_disabled());
        assert_eq!(display_virtual_key(0x70, strings), "F1");
        assert_eq!(display_virtual_key(0x7B, strings), "F12");
        assert_eq!(display_virtual_key(0x41, strings), "A");
        assert_eq!(display_virtual_key(0x65, strings), "Num5");
        assert_eq!(display_virtual_key(0xC0, strings), "`");
        assert_eq!(display_virtual_key(0xFF, strings), "0xFF");
    }

    #[test]
    fn cursor_files_need_the_right_extension_and_header() {
        let dir = tempfile::tempdir().unwrap();
        let cur = dir.path().join("arrow.CUR");
        std::fs::write(&cur, [0x00, 0x00, 0x02, 0x00, 0x01]).unwrap();
        assert!(is_valid_cursor_file(&cur));
        let ani = dir.path().join("spin.ani");
        std::fs::write(&ani, b"RIFFdata").unwrap();
        assert!(is_valid_cursor_file(&ani));
        let fake = dir.path().join("fake.cur");
        std::fs::write(&fake, b"RIFF").unwrap();
        assert!(!is_valid_cursor_file(&fake));
        let short = dir.path().join("short.ani");
        std::fs::write(&short, b"RI").unwrap();
        assert!(!is_valid_cursor_file(&short));
        let png = dir.path().join("arrow.png");
        std::fs::write(&png, [0x00, 0x00, 0x02, 0x00]).unwrap();
        assert!(!is_valid_cursor_file(&png));
        assert!(!is_valid_cursor_file(&dir.path().join("missing.cur")));
    }

    #[test]
    fn cursor_paths_are_stored_for_the_game_side() {
        assert_eq!(cursor_display_name(r"C:\cursors\arrow.cur"), "arrow.cur");
        assert_eq!(cursor_display_name("Z:/home/me/a.ani"), "a.ani");
        assert_eq!(cursor_display_name("plain.cur"), "plain.cur");
        assert_eq!(
            stored_cursor_path(Path::new("/home/me/a.cur"), true),
            r"Z:\home\me\a.cur"
        );
        assert_eq!(
            stored_cursor_path(Path::new(r"C:\c\a.cur"), false),
            r"C:\c\a.cur"
        );
    }
}
