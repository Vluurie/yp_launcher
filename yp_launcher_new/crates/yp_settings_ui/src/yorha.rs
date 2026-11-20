use std::rc::Rc;

use gpui::{Keystroke, ScrollHandle, Subscription};
use yp_config::fields::overlay::{cheats, keybinds, randomizer, workspace};
use yp_config::overlay::Settings;
use yp_state::prelude::*;
use yp_state::{NamsSettingsStore, NavigationEvent, SessionEvent};
use yp_ui::prelude::*;
use yp_ui::{IconName, KeyCapture, PageHeader, SettingRow, TwoColumn};

use crate::errors::settings_failure_text;
use crate::fields::{
    FieldRef, FieldRow, ValueSink, bool_of, changed_in_json, field_id, label_of, text_of, to_json,
    tooltip_of, value_in_json,
};
use crate::page::{banner, card, column, failure_banner, muted_text, reveal, tab_page};

const SCOPE: &str = "yorha";

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
    settings: Entity<NamsSettingsStore>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl YorhaProtocolTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let settings = stores.nams_settings.clone();
        let subscriptions = vec![
            cx.observe(&settings, |_, _, cx| cx.notify()),
            cx.subscribe(&stores.session, |this, _, event: &SessionEvent, cx| {
                if matches!(event, SessionEvent::Stopped(_)) {
                    this.settings
                        .update(cx, |store, cx| store.reload_if_clean(cx));
                }
            }),
            cx.subscribe(
                &stores.navigation,
                |this, _, event: &NavigationEvent, cx| {
                    if matches!(
                        event,
                        NavigationEvent::TabChanged {
                            to: TabId::YorhaProtocol,
                            ..
                        }
                    ) {
                        this.settings
                            .update(cx, |store, cx| store.reload_if_clean(cx));
                    }
                },
            ),
        ];
        Self {
            settings,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub fn set_keybind(&mut self, field: &'static yp_config::ConfigField<String>, key: &str, cx: &mut Context<Self>) {
        let Some(section) = field.section() else {
            return;
        };
        let action = field.key();
        self.settings.update(cx, |store, cx| {
            store.update_keybind(section, action, key, cx)
        });
    }

    fn keybind_row(
        &self,
        field: &'static yp_config::ConfigField<String>,
        binds: &Settings,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let value = text_of(&value_in_json(field, binds));
        let id = field_id(SCOPE, field);
        SettingRow::new(id.clone(), label_of(field, strings))
            .description_opt(tooltip_of(field, strings))
            .hoverable(false)
            .control(
                KeyCapture::new((id, "capture"), value).on_capture(cx.listener(
                    move |this, keystroke: &Keystroke, _, cx| {
                        if let Some(key) = keybind_from_keystroke(keystroke) {
                            this.set_keybind(field, &key, cx);
                        }
                    },
                )),
            )
            .into_any_element()
    }

    fn toggle_sink(&self, field: FieldRef) -> ValueSink {
        let settings = self.settings.clone();
        let key = field.meta().key;
        Rc::new(move |value, _, cx| {
            let enabled = bool_of(&value);
            settings.update(cx, |store, cx| store.update_toggle(key, enabled, cx));
        })
    }

    fn cheat_sink(&self, field: FieldRef) -> ValueSink {
        let settings = self.settings.clone();
        let key = field.meta().key;
        Rc::new(move |value, _, cx| {
            let value = to_json(&value);
            settings.update(cx, |store, cx| store.update_cheat(key, value, cx));
        })
    }

    fn randomizer_sink(&self, field: FieldRef) -> ValueSink {
        let settings = self.settings.clone();
        let key = field.meta().key;
        Rc::new(move |value, _, cx| {
            let value = to_json(&value);
            settings.update(cx, |store, cx| store.update_randomizer(key, value, cx));
        })
    }
}

impl Render for YorhaProtocolTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let store = self.settings.read(cx);
        let loading = store.is_loading();
        let dirty = store.is_dirty();
        let root = store.settings().clone();
        let binds = store.keybinds();
        let cheat_values = store.cheats_config();
        let randomizer_values = store.randomizer_config();
        let path = store.path().map(|path| path.to_path_buf());
        let failure = store
            .failure()
            .map(|failure| settings_failure_text(failure, strings));
        let save_store = self.settings.clone();
        let discard_store = self.settings.clone();
        let header = PageHeader::new(strings.header_yorha_protocol())
            .dirty(dirty)
            .save_discard(
                move |_, cx| save_store.update(cx, |store, cx| store.save(cx)),
                move |_, cx| discard_store.update(cx, |store, cx| store.discard(cx)),
            );
        let header = match path {
            Some(path) => header.info(strings.tooltip_edits_settings_json(), move |_, cx| {
                reveal(path.clone(), true, cx)
            }),
            None => header,
        };
        let content = if loading {
            div().into_any_element()
        } else {
            let keybind_rows: Vec<AnyElement> = keybinds::ALL
                .iter()
                .map(|field| self.keybind_row(field, &binds, cx))
                .collect();
            let keybinds_card = card("yorha-keybinds", strings.card_keybinds(), cx)
                .children(keybind_rows);
            let workspace_card = card("yorha-workspace", strings.card_workspace(), cx)
                .changed(changed_in_json(&workspace::ALL, &root, strings))
                .children(workspace::ALL.iter().map(|field| {
                    FieldRow::json(SCOPE, *field, &root).sink(self.toggle_sink(*field))
                }));
            let cheats_card = card("yorha-cheats", strings.card_cheats(), cx)
                .changed(changed_in_json(&cheats::ALL, &cheat_values, strings))
                .children(cheats::ALL.iter().map(|field| {
                    FieldRow::json(SCOPE, *field, &cheat_values).sink(self.cheat_sink(*field))
                }))
                .child(muted_text(strings.cheats_applied_note(), true, window, cx));
            let randomizer_card =
                card("yorha-randomizer", strings.card_randomizer(), cx)
                    .changed(changed_in_json(
                        &randomizer::ALL,
                        &randomizer_values,
                        strings,
                    ))
                    .children(randomizer::ALL.iter().map(|field| {
                        FieldRow::json(SCOPE, *field, &randomizer_values)
                            .sink(self.randomizer_sink(*field))
                    }));
            let left = column(
                [
                    keybinds_card.into_any_element(),
                    workspace_card.into_any_element(),
                ],
                window,
                cx,
            );
            let right = column(
                [
                    cheats_card.into_any_element(),
                    randomizer_card.into_any_element(),
                ],
                window,
                cx,
            );
            let mut sections: Vec<AnyElement> = Vec::new();
            if let Some(failure) = failure {
                sections.push(failure_banner("yorha-failure", failure).into_any_element());
            }
            sections.push(
                banner(
                    IconName::InfoOutline,
                    strings.yorha_protocol_description(),
                    window,
                    cx,
                )
                .into_any_element(),
            );
            sections.push(
                TwoColumn::new(left, right)
                    .id("yorha-columns")
                    .into_any_element(),
            );
            column(sections, window, cx)
            .into_any_element()
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
}
