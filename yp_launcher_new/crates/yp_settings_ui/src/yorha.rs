use gpui::{Keystroke, ScrollHandle, Subscription};
use serde_json::Value as Json;
use yp_config::{Kind, Setting};
use yp_state::prelude::*;
use yp_state::{ConfigStore, NavigationEvent, SessionEvent};
use yp_ui::prelude::*;
use yp_ui::{IconName, KeyCapture, SettingRow, TwoColumn};

use crate::fields::{Marker, field_id};
use crate::page::{banner, card, column, tab_page};
use crate::schema_page::{self, RowContext, TabScope};

const SCOPE: &str = "yorha";
const FILE: &str = "_internal/cache/settings.json";
pub const TAB: TabScope = TabScope::plugins();
pub const KEYBINDS_GROUP: &str = "keybinds";
const OVERLAY_GROUP: &str = "overlay";
const RANDOMIZER_GROUP: &str = "randomizer";
const CHEATS_GROUP: &str = "cheats";

pub fn group_title(group: &str, cx: &App) -> SharedString {
    let strings = cx.t();
    match group {
        KEYBINDS_GROUP => SharedString::new_static(strings.card_keybinds()),
        OVERLAY_GROUP => SharedString::new_static(strings.card_workspace()),
        RANDOMIZER_GROUP => SharedString::new_static(strings.card_randomizer()),
        CHEATS_GROUP => SharedString::new_static(strings.card_cheats()),
        other => {
            let mut chars = other.chars();
            let first: String = chars
                .next()
                .map(|first| first.to_uppercase().collect())
                .unwrap_or_default();
            SharedString::from(first + chars.as_str())
        }
    }
}

pub fn is_keybind(setting: &Setting) -> bool {
    setting.group.as_deref() == Some(KEYBINDS_GROUP) && setting.kind == Kind::String
}

pub fn groups(settings: &[Setting]) -> Vec<(String, Vec<Setting>)> {
    let mut groups: Vec<(String, Vec<Setting>)> = Vec::new();
    for setting in settings {
        let group = setting.group.clone().unwrap_or_default();
        match groups.iter_mut().find(|(name, _)| *name == group) {
            Some((_, members)) => members.push(setting.clone()),
            None => groups.push((group, vec![setting.clone()])),
        }
    }
    groups
}

pub fn key_label(key: &str) -> Option<String> {
    let mut chars = key.chars();
    if let (Some(single), None) = (chars.next(), chars.next()) {
        return Some(single.to_uppercase().collect());
    }
    if let Some(number) = key.strip_prefix('f')
        && !number.is_empty()
        && number.chars().all(|digit| digit.is_ascii_digit())
    {
        return Some(format!("F{number}"));
    }
    let named = match key {
        "space" => "Space",
        "tab" => "Tab",
        "enter" => "Enter",
        "escape" => "Escape",
        "backspace" => "Backspace",
        "insert" => "Insert",
        "delete" => "Delete",
        "home" => "Home",
        "end" => "End",
        "pageup" => "Page Up",
        "pagedown" => "Page Down",
        "up" => "Arrow Up",
        "down" => "Arrow Down",
        "left" => "Arrow Left",
        "right" => "Arrow Right",
        _ => return None,
    };
    Some(named.to_owned())
}

pub fn keybind_from_keystroke(keystroke: &Keystroke) -> Option<String> {
    let key = key_label(&keystroke.key)?;
    let mut parts: Vec<&str> = Vec::new();
    if keystroke.modifiers.control {
        parts.push("Ctrl");
    }
    if keystroke.modifiers.alt {
        parts.push("Alt");
    }
    if keystroke.modifiers.shift {
        parts.push("Shift");
    }
    parts.push(&key);
    Some(parts.join("+"))
}

pub struct YorhaProtocolTab {
    config: Entity<ConfigStore>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl YorhaProtocolTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let config = stores.config.clone();
        let subscriptions = vec![
            cx.observe(&config, |_, _, cx| cx.notify()),
            cx.subscribe(&stores.session, |this, _, event: &SessionEvent, cx| {
                if matches!(event, SessionEvent::Stopped(_)) {
                    this.config.update(cx, |store, cx| store.reload_if_clean(cx));
                }
            }),
            cx.subscribe(&stores.navigation, |this, _, event: &NavigationEvent, cx| {
                if matches!(
                    event,
                    NavigationEvent::TabChanged {
                        to: TabId::YorhaProtocol,
                        ..
                    }
                ) {
                    this.config.update(cx, |store, cx| store.reload_if_clean(cx));
                }
            }),
        ];
        Self {
            config,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub fn set_keybind(&mut self, id: &str, key: &str, cx: &mut Context<Self>) {
        let value = Json::String(key.to_owned());
        self.config.update(cx, |store, cx| store.set_now(id, value, cx));
    }

    fn keybind_row(&self, setting: &Setting, cx: &mut Context<Self>) -> AnyElement {
        let value = self.config.read(cx).text(&setting.id);
        let id = field_id(SCOPE, &setting.id);
        let setting_id = setting.id.clone();
        SettingRow::new(id.clone(), setting.label.clone())
            .description_opt(Some(SharedString::from(setting.description.clone())).filter(|text| !text.is_empty()))
            .hoverable(false)
            .control(
                KeyCapture::new((id, "capture"), value).on_capture(cx.listener(
                    move |this, keystroke: &Keystroke, _, cx| {
                        if let Some(key) = keybind_from_keystroke(keystroke) {
                            this.set_keybind(&setting_id, &key, cx);
                        }
                    },
                )),
            )
            .into_any_element()
    }

    fn group_card(&self, group: &str, settings: &[Setting], window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let mut context = RowContext::default();
        let mut rows: Vec<AnyElement> = Vec::new();
        for setting in settings {
            let element = if is_keybind(setting) {
                self.keybind_row(setting, cx)
            } else {
                schema_page::field(SCOPE, setting, &self.config, Marker::None, cx).into_any_element()
            };
            rows.extend(context.wrap(setting, element, &self.config, window, cx));
        }
        let store = self.config.read(cx);
        let members: Vec<&Setting> = settings.iter().collect();
        card(schema_page::card_id(SCOPE, group), group_title(group, cx), cx)
            .changed(schema_page::changed_labels(&members, store))
            .children(rows)
            .into_any_element()
    }
}

impl Render for YorhaProtocolTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let store = self.config.read(cx);
        let loading = schema_page::is_loading(store);
        let header = schema_page::header(
            strings.header_yorha_protocol(),
            strings.tooltip_edits_settings_json(),
            Some(FILE),
            None,
            &self.config,
            cx,
        );
        let content = if loading {
            div().into_any_element()
        } else {
            let settings: Vec<Setting> = TAB.settings(self.config.read(cx).schema()).cloned().collect();
            let mut left: Vec<AnyElement> = Vec::new();
            let mut right: Vec<AnyElement> = Vec::new();
            for (index, (group, members)) in groups(&settings).into_iter().enumerate() {
                let card = self.group_card(&group, &members, window, cx);
                if index % 2 == 0 {
                    left.push(card);
                } else {
                    right.push(card);
                }
            }
            let mut sections = schema_page::problems(SCOPE, &TAB, &self.config, cx);
            sections.push(
                banner(IconName::InfoOutline, strings.yorha_protocol_description(), window, cx).into_any_element(),
            );
            sections.push(
                TwoColumn::new(column(left, window, cx), column(right, window, cx))
                    .id("yorha-columns")
                    .into_any_element(),
            );
            column(sections, window, cx).into_any_element()
        };
        tab_page("yorha-scroll", header, &self.scroll, loading, content, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use gpui::Modifiers;

    use super::*;

    fn stroke(key: &str, modifiers: Modifiers) -> Keystroke {
        Keystroke {
            modifiers,
            key: key.into(),
            key_char: None,
        }
    }

    #[test]
    fn keys_use_the_names_the_overlay_writes_itself() {
        assert_eq!(key_label("a").as_deref(), Some("A"));
        assert_eq!(key_label("7").as_deref(), Some("7"));
        assert_eq!(key_label("f5").as_deref(), Some("F5"));
        assert_eq!(key_label("f12").as_deref(), Some("F12"));
        assert_eq!(key_label("insert").as_deref(), Some("Insert"));
        assert_eq!(key_label("pageup").as_deref(), Some("Page Up"));
        assert_eq!(key_label("space").as_deref(), Some("Space"));
        assert_eq!(key_label("fn"), None);
        assert_eq!(key_label("menu"), None);
    }

    #[test]
    fn modifiers_are_written_in_front_of_the_key() {
        assert_eq!(
            keybind_from_keystroke(&stroke("c", Modifiers::shift())).as_deref(),
            Some("Shift+C")
        );
        assert_eq!(
            keybind_from_keystroke(&stroke("f7", Modifiers::none())).as_deref(),
            Some("F7")
        );
        let all = Modifiers {
            control: true,
            alt: true,
            shift: true,
            ..Modifiers::none()
        };
        assert_eq!(
            keybind_from_keystroke(&stroke("x", all)).as_deref(),
            Some("Ctrl+Alt+Shift+X")
        );
        assert_eq!(keybind_from_keystroke(&stroke("menu", Modifiers::none())), None);
    }

    #[test]
    fn plugin_settings_split_into_their_groups_in_schema_order() {
        let schema: yp_config::Schema =
            serde_json::from_value(yp_config::testing::sample_schema()).unwrap();
        let settings: Vec<Setting> = TAB.settings(&schema).cloned().collect();
        let names: Vec<String> = groups(&settings).into_iter().map(|(name, _)| name).collect();
        assert_eq!(names, vec!["keybinds".to_owned(), "overlay".to_owned(), "cheats".to_owned()]);
        let open = schema
            .setting("plugins.yorha_protocol.keybinds.main.yorha_protocol")
            .unwrap();
        assert!(is_keybind(open));
        assert!(!is_keybind(schema.setting("plugins.yorha_protocol.firstPlaythrough").unwrap()));
    }
}
