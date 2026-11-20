use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{
    FontWeight, KeyDownEvent, ModifiersChangedEvent, MouseButton, MouseDownEvent,
    NavigationDirection, PathPromptOptions, ScrollHandle, ScrollEvent, Stateful,
    Subscription, Task,
};
use yp_config::fields::{bindings, nams};
use yp_config::{ConfigField, ConfigFile, naiom_keys};
use yp_i18n::Strings;
use yp_state::ConfigStore;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    Button, ButtonStyle, Callout, Card, CardPadding, ConfigErrorBanner, ConfirmOptions, ConfirmTone,
    FocusNext, FocusPrevious, Icon, IconName, IconSize, Label, LabelSize, PageHeader,
    PreviewImage, Select, SelectItem, SettingRow, TintColor, TwoColumn, asset_image, confirm,
};

use crate::binding::{
    Capture, ModifierState, PointerButton, conflict_for, controller_label, cursor_display_name,
    display_binding, display_virtual_key, is_controller_binding, is_valid_cursor_file,
    stored_cursor_path, virtual_key_from_key_down, virtual_key_from_modifiers,
};
use crate::errors::{config_failure_text, toml_error_text};
use crate::fields::{
    FieldRef, FieldRow, Marker, ValueSink, bool_of, changed_in, field_id, is_changed, label_of,
    number_of, text_of, tooltip_of, value_in,
};
use crate::page::{card, column, failure_banner, locked, note, reveal, row_divider, tab_page};

const SCOPE: &str = "naiom";

pub const CAMERA_FIELDS: [FieldRef; 7] = [
    &nams::FIX_CAMERA_ACCELERATION,
    &nams::SENSITIVITY,
    &nams::THIRD_PERSON_MODE,
    &nams::THIRD_PERSON_CHAR_FOLLOW,
    &nams::THIRD_PERSON_SMOOTHING,
    &nams::THIRD_PERSON_SENSITIVITY_X,
    &nams::THIRD_PERSON_SENSITIVITY_Y,
];

pub const AIMING_FIELDS: [FieldRef; 7] = [
    &nams::AIM_MODE,
    &nams::AIM_CROSSHAIR,
    &nams::AIM_CROSSHAIR_ALWAYS,
    &nams::AIM_SENSITIVITY,
    &nams::AIM_OUTPUT_MULTIPLIER,
    &nams::AIM_SENSITIVITY_X,
    &nams::AIM_SENSITIVITY_Y,
];

pub const MISC_FIELDS: [FieldRef; 4] = [
    &nams::MISC_DISABLE_POD_PET,
    &nams::MISC_CUSTOM_CURSOR_MENU,
    &nams::MISC_CUSTOM_CURSOR_HACKING,
    &nams::MISC_DISABLE_DEFAULT_CURSOR,
];

pub const MOVEMENT_BINDINGS: [&ConfigField<String>; 7] = [
    &bindings::STANDARD_MOVE_FORWARD,
    &bindings::STANDARD_MOVE_BACKWARD,
    &bindings::STANDARD_MOVE_LEFT,
    &bindings::STANDARD_MOVE_RIGHT,
    &bindings::STANDARD_JUMP,
    &bindings::STANDARD_WALK,
    &bindings::STANDARD_AUTO_RUN,
];

pub const COMBAT_BINDINGS: [&ConfigField<String>; 12] = [
    &bindings::STANDARD_LIGHT_ATTACK,
    &bindings::STANDARD_HEAVY_ATTACK,
    &bindings::STANDARD_FIRE,
    &bindings::STANDARD_PROGRAM,
    &bindings::STANDARD_LOCK_ON,
    &bindings::STANDARD_USE,
    &bindings::STANDARD_SWITCH_WEAPON,
    &bindings::STANDARD_NEXT_PROGRAM,
    &bindings::STANDARD_PREVIOUS_PROGRAM,
    &bindings::STANDARD_SELF_DESTRUCT,
    &bindings::STANDARD_LIGHT,
    &bindings::STANDARD_RESET_CAMERA,
];

pub const NON_STANDARD_BINDINGS: [&ConfigField<String>; 6] = [
    &bindings::NON_STANDARD_EVADE,
    &bindings::NON_STANDARD_AUTO_FIRE,
    &bindings::NON_STANDARD_NEXT_ITEM,
    &bindings::NON_STANDARD_PREVIOUS_ITEM,
    &bindings::NON_STANDARD_USE_ITEM,
    &bindings::NON_STANDARD_POD_PET,
];

pub const MODE_BINDINGS: [&ConfigField<String>; 2] = [
    &bindings::THIRD_PERSON_MODE_TOGGLE,
    &bindings::AIM_MODE_TOGGLE,
];

pub const MENU_BINDINGS: [&ConfigField<String>; 8] = [
    &bindings::STANDARD_MENU_UP,
    &bindings::STANDARD_MENU_DOWN,
    &bindings::STANDARD_MENU_LEFT,
    &bindings::STANDARD_MENU_RIGHT,
    &bindings::STANDARD_MENU_OPEN,
    &bindings::STANDARD_MENU_BACK,
    &bindings::STANDARD_MENU_ENTER,
    &bindings::STANDARD_SHORTCUT_MENU,
];

pub fn binding_value(field: &ConfigField<String>, values: &toml::Table) -> String {
    field
        .meta
        .raw_in(values)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

pub fn binding_values(values: &toml::Table) -> Vec<(&'static str, String)> {
    bindings::ALL
        .iter()
        .map(|field| (field.key(), binding_value(field, values)))
        .collect()
}

pub fn conflict_label(
    field: &ConfigField<String>,
    all: &[(&'static str, String)],
    strings: &'static Strings,
) -> Option<SharedString> {
    let other = conflict_for(
        field.key(),
        all.iter().map(|(key, value)| (*key, value.as_str())),
    )?;
    bindings::ALL
        .iter()
        .find(|candidate| candidate.key() == other)
        .map(|candidate| label_of(*candidate, strings))
}

pub fn has_non_defaults(values: &toml::Table) -> bool {
    nams::MOUSE
        .iter()
        .any(|field| is_changed(*field, &value_in(*field, values)))
        || bindings::ALL
            .iter()
            .any(|field| !binding_value(field, values).is_empty())
}

pub fn reset_values() -> Vec<(&'static str, &'static str, toml::Value)> {
    let mut values: Vec<(&'static str, &'static str, toml::Value)> = nams::MOUSE
        .iter()
        .filter_map(|field| {
            let meta = field.meta();
            meta.section
                .map(|section| (meta.key, section, field.default_toml()))
        })
        .collect();
    values.extend(
        bindings::ALL
            .iter()
            .map(|field| (field.key(), bindings::SECTION, toml::Value::String(String::new()))),
    );
    values
}

fn pointer_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Left,
        MouseButton::Right => PointerButton::Right,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Navigate(NavigationDirection::Back) => PointerButton::Back,
        MouseButton::Navigate(NavigationDirection::Forward) => PointerButton::Forward,
    }
}

fn modifier_state(event: &ModifiersChangedEvent) -> ModifierState {
    ModifierState {
        shift: event.modifiers.shift,
        ctrl: event.modifiers.control,
        alt: event.modifiers.alt,
    }
}

struct KeyFace {
    text: SharedString,
    dimmed: bool,
    controller: bool,
    conflict: bool,
}

pub struct NaiomTab {
    config: Entity<ConfigStore>,
    scroll: ScrollHandle,
    focus: HashMap<&'static str, FocusHandle>,
    capturing: Option<&'static str>,
    capture: Capture,
    suppress_click: Option<&'static str>,
    invalid_cursors: HashSet<&'static str>,
    cursor_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl NaiomTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx.stores().config.clone();
        let subscriptions = vec![cx.observe(&config, |_, _, cx| cx.notify())];
        let mut focus: HashMap<&'static str, FocusHandle> = bindings::ALL
            .iter()
            .map(|field| (field.key(), cx.focus_handle()))
            .collect();
        focus.insert(nams::MISC_OPEN_DEBUG_MENU.key(), cx.focus_handle());
        Self {
            config,
            scroll: ScrollHandle::new(),
            focus,
            capturing: None,
            capture: Capture::new(),
            suppress_click: None,
            invalid_cursors: HashSet::new(),
            cursor_task: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub fn capturing(&self) -> Option<&'static str> {
        self.capturing
    }

    pub fn capture_state(&self) -> &Capture {
        &self.capture
    }

    pub fn is_cursor_invalid(&self, key: &str) -> bool {
        self.invalid_cursors.contains(key)
    }

    fn set_mouse(&self, field: FieldRef, value: toml::Value, cx: &mut App) {
        let meta = *field.meta();
        let Some(section) = meta.section else {
            return;
        };
        self.config.update(cx, |store, cx| {
            store.update_naiom(meta.key, value, section, cx)
        });
    }

    fn sink(&self, field: FieldRef) -> ValueSink {
        let config = self.config.clone();
        let meta = *field.meta();
        Rc::new(move |value, _, cx| {
            let Some(section) = meta.section else {
                return;
            };
            config.update(cx, |store, cx| {
                store.update_naiom(meta.key, value, section, cx)
            });
        })
    }

    pub fn start_capture(&mut self, key: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        self.capturing = Some(key);
        self.capture = Capture::new();
        self.suppress_click = None;
        if let Some(handle) = self.focus.get(key) {
            window.focus(handle, cx);
        }
        cx.notify();
    }

    pub fn cancel_capture(&mut self, cx: &mut Context<Self>) {
        if self.capturing.take().is_some() {
            self.capture = Capture::new();
            cx.notify();
        }
    }

    fn finish_binding(&mut self, key: &'static str, binding: String, cx: &mut Context<Self>) {
        self.capturing = None;
        self.capture = Capture::new();
        self.set_binding(key, &binding, cx);
        cx.notify();
    }

    pub fn set_binding(&mut self, key: &'static str, binding: &str, cx: &mut Context<Self>) {
        self.config.update(cx, |store, cx| {
            store.update_naiom(
                key,
                toml::Value::String(binding.to_owned()),
                bindings::SECTION,
                cx,
            )
        });
    }

    pub fn capture_key(
        &mut self,
        key: &'static str,
        state: ModifierState,
        pressed: &str,
        cx: &mut Context<Self>,
    ) {
        if self.capturing != Some(key) {
            return;
        }
        if let Some(binding) = self.capture.key_down_with(state, pressed) {
            self.finish_binding(key, binding, cx);
        }
    }

    pub fn capture_modifiers(
        &mut self,
        key: &'static str,
        state: ModifierState,
        cx: &mut Context<Self>,
    ) {
        if self.capturing != Some(key) {
            return;
        }
        match self.capture.modifiers_changed(state) {
            Some(binding) => self.finish_binding(key, binding, cx),
            None => cx.notify(),
        }
    }

    pub fn capture_pointer(
        &mut self,
        key: &'static str,
        button: PointerButton,
        cx: &mut Context<Self>,
    ) {
        if self.capturing != Some(key) {
            return;
        }
        if let Some(binding) = self
            .capture
            .pointer_down(button, naiom_keys::allows_mouse(key))
        {
            self.suppress_click = (button == PointerButton::Left).then_some(key);
            self.finish_binding(key, binding, cx);
        }
    }

    pub fn capture_wheel(&mut self, key: &'static str, delta_y: f32, cx: &mut Context<Self>) {
        if ControllerMode::is_active(cx) || self.capturing != Some(key) {
            return;
        }
        if let Some(binding) = self.capture.wheel(delta_y, naiom_keys::allows_mouse(key)) {
            self.finish_binding(key, binding, cx);
        }
    }

    pub fn capture_virtual_key(&mut self, vk: i64, cx: &mut Context<Self>) {
        let field = &nams::MISC_OPEN_DEBUG_MENU;
        if self.capturing != Some(field.key()) {
            return;
        }
        self.capturing = None;
        self.capture = Capture::new();
        self.set_mouse(field, toml::Value::Integer(vk), cx);
        cx.notify();
    }

    pub fn reset_all(&mut self, cx: &mut Context<Self>) {
        self.cancel_capture(cx);
        self.invalid_cursors.clear();
        self.config.update(cx, |store, cx| {
            for (key, section, value) in reset_values() {
                store.update_naiom(key, value, section, cx);
            }
        });
    }

    pub fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        let answer = confirm(
            ConfirmOptions::new(
                strings.naiom_reset_confirm_title(),
                strings.naiom_reset_confirm_body(),
            )
            .confirm_label(strings.lod_mod_reset_confirm_action())
            .cancel_label(strings.cancel_button())
            .tone(ConfirmTone::Danger),
            window,
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await {
                this.update(cx, |this, cx| this.reset_all(cx)).ok();
            }
        })
        .detach();
    }

    pub fn apply_cursor(
        &mut self,
        field: &'static ConfigField<String>,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        let through_wine = cx.env().host.runs_game_through_wine();
        let check_path = path.clone();
        let check = cx.background_spawn(async move { is_valid_cursor_file(&check_path) });
        self.cursor_task = Some(cx.spawn(async move |this, cx| {
            let valid = check.await;
            this.update(cx, |this, cx| {
                if valid {
                    this.invalid_cursors.remove(field.key());
                    let stored = stored_cursor_path(&path, through_wine);
                    this.set_mouse(field, toml::Value::String(stored), cx);
                } else {
                    this.invalid_cursors.insert(field.key());
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn pick_cursor(&mut self, field: &'static ConfigField<String>, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        self.cursor_task = Some(cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |this, cx| this.apply_cursor(field, path, cx))
                .ok();
        }));
    }

    fn clear_cursor(&mut self, field: &'static ConfigField<String>, cx: &mut Context<Self>) {
        self.invalid_cursors.remove(field.key());
        self.set_mouse(field, toml::Value::String(String::new()), cx);
        cx.notify();
    }

    fn row(&self, field: FieldRef, values: &toml::Table, enabled: bool) -> FieldRow {
        FieldRow::toml(SCOPE, field, values)
            .marker(Marker::RestartOnly)
            .disabled(!enabled)
            .sink(self.sink(field))
    }

    fn key_button(
        &self,
        key: &'static str,
        face: KeyFace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<gpui::Div> {
        let KeyFace {
            text,
            dimmed,
            controller,
            conflict,
        } = face;
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let capturing = self.capturing == Some(key);
        let controller_mode = yp_ui::ControllerMode::is_active(cx);
        let accent = theme.colors.accent_primary;
        let border = if capturing {
            accent
        } else if conflict {
            theme.colors.warning
        } else {
            theme.colors.border_medium
        };
        let text_color = if capturing {
            accent
        } else if dimmed {
            theme.colors.text_muted
        } else {
            theme.colors.text_primary
        };
        let hover_border = accent.opacity(theme.alpha.vignette);
        let hover_background = accent.opacity(theme.alpha.subtle);
        let button = h_flex()
            .id(SharedString::from(format!("naiom-key-{key}")))
            .flex_none()
            .justify_center()
            .gap(sizes.spacing_md)
            .min_w(sizes.config_input_width)
            .px(sizes.padding_md)
            .py(sizes.padding_sm)
            .rounded(Sizes::CONTROL_RADIUS)
            .border_1()
            .border_color(border)
            .bg(if capturing {
                accent.opacity(theme.alpha.hover)
            } else {
                theme.colors.input_background
            })
            .text_size(sizes.font_sm)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(text_color)
            .cursor_pointer()
            .when(controller_mode, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!capturing, |this| {
                this.hover(move |style| style.bg(hover_background).border_color(hover_border))
            })
            .when(controller && !capturing, |this| {
                this.child(
                    Icon::new(IconName::SportsEsportsOutlined)
                        .size(IconSize::Small)
                        .color(Color::Accent),
                )
            })
            .child(text);
        #[cfg(test)]
        let button = button.debug_selector(move || format!("naiom-key-{key}"));
        match self.focus.get(key) {
            Some(handle) => button.track_focus(&handle.clone().tab_stop(!controller_mode)),
            None => button,
        }
    }

    fn binding_row(
        &self,
        field: &'static ConfigField<String>,
        all: &[(&'static str, String)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let key = field.key();
        let value = all
            .iter()
            .find(|(other, _)| *other == key)
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        let capturing = self.capturing == Some(key);
        let allows_mouse = naiom_keys::allows_mouse(key);
        let conflict = conflict_label(field, all, strings);
        let controller = is_controller_binding(&value);
        let text = if capturing {
            self.capture
                .pending_label()
                .map(SharedString::from)
                .unwrap_or_else(|| SharedString::new_static(strings.press_key()))
        } else {
            SharedString::from(display_binding(&value, strings))
        };
        let button = self
            .key_button(
                key,
                KeyFace {
                    text,
                    dimmed: value.is_empty(),
                    controller,
                    conflict: conflict.is_some(),
                },
                window,
                cx,
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                if yp_ui::ControllerMode::is_active(cx) {
                    return;
                }
                if this.capturing == Some(key) {
                    return;
                }
                if this.suppress_click.take() == Some(key) {
                    return;
                }
                this.start_capture(key, window, cx);
            }))
            .when(capturing, |this| {
                this.on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                    cx.stop_propagation();
                    let state = ModifierState {
                        shift: event.keystroke.modifiers.shift,
                        ctrl: event.keystroke.modifiers.control,
                        alt: event.keystroke.modifiers.alt,
                    };
                    this.capture_key(key, state, &event.keystroke.key, cx);
                }))
                .on_modifiers_changed(cx.listener(
                    move |this, event: &ModifiersChangedEvent, _, cx| {
                        this.capture_modifiers(key, modifier_state(event), cx);
                    },
                ))
                .on_any_mouse_down(cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.capture_pointer(key, pointer_button(event.button), cx);
                }))
                .on_scroll(cx.listener(move |this, event: &ScrollEvent, window, cx| {
                    cx.stop_propagation();
                    let delta = event.delta.pixel_delta(window.line_height());
                    this.capture_wheel(key, f32::from(delta.y), cx);
                }))
                .on_action(cx.listener(move |this, _: &FocusNext, _, cx| {
                    this.capture_key(key, ModifierState::default(), "tab", cx);
                }))
                .on_action(cx.listener(move |this, _: &FocusPrevious, _, cx| {
                    let state = ModifierState {
                        shift: true,
                        ..ModifierState::default()
                    };
                    this.capture_key(key, state, "tab", cx);
                }))
            });
        let detail = if capturing && !allows_mouse {
            h_flex()
                .gap(sizes.spacing_md)
                .child(
                    Icon::new(IconName::KeyboardOutlined)
                        .size(IconSize::XSmall)
                        .color(Color::Accent),
                )
                .child(
                    Label::new(strings.keybind_mouse_not_supported())
                        .size(LabelSize::XSmall)
                        .color(Color::Accent),
                )
                .into_any_element()
        } else {
            match tooltip_of(field, strings) {
                Some(tooltip) => Label::new(tooltip)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .line_height(1.3)
                    .into_any_element(),
                None => div().into_any_element(),
            }
        };
        let picker = allows_mouse.then(|| {
            let options = naiom_keys::CONTROLLER_BUTTONS;
            Select::new(
                SharedString::from(format!("naiom-controller-{key}")),
                SharedString::default(),
            )
            .trigger_icon(IconName::SportsEsportsOutlined)
            .tooltip(strings.keybind_pick_controller())
            .selected_index(options.iter().position(|option| *option == value))
            .items(options.iter().map(|option| {
                SelectItem::new(
                    controller_label(option, strings).unwrap_or(option),
                    *option,
                )
            }))
            .on_select(cx.listener(move |this, option: &&'static str, _, cx| {
                this.cancel_capture(cx);
                this.set_binding(key, option, cx);
            }))
        });
        v_flex()
            .w_full()
            .py(sizes.spacing_sm)
            .px(sizes.padding_sm)
            .child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(Label::new(label_of(field, strings)).size(LabelSize::Small))
                            .child(detail),
                    )
                    .child(button)
                    .children(picker),
            )
            .when_some(conflict, |this, other| {
                this.child(
                    h_flex()
                        .w_full()
                        .justify_end()
                        .gap(sizes.spacing_md)
                        .pt(sizes.spacing_xs)
                        .child(
                            Icon::new(IconName::WarningAmberRounded)
                                .size(IconSize::XSmall)
                                .color(Color::Warning),
                        )
                        .child(
                            Label::new(strings.keybind_conflict(&other))
                                .size(LabelSize::XSmall)
                                .color(Color::Warning),
                        ),
                )
            })
            .into_any_element()
    }

    fn debug_key_row(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let field = &nams::MISC_OPEN_DEBUG_MENU;
        let key = field.key();
        let vk = field.value_in(values);
        let capturing = self.capturing == Some(key);
        let text = if capturing {
            SharedString::new_static(strings.press_key())
        } else {
            SharedString::from(display_virtual_key(vk, strings))
        };
        let button = self
            .key_button(
                key,
                KeyFace {
                    text,
                    dimmed: vk == 0,
                    controller: false,
                    conflict: false,
                },
                window,
                cx,
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                if this.capturing != Some(key) {
                    this.start_capture(key, window, cx);
                }
            }))
            .when(capturing, |this| {
                this.on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                    cx.stop_propagation();
                    if let Some(vk) = virtual_key_from_key_down(&event.keystroke.key) {
                        this.capture_virtual_key(vk, cx);
                    }
                }))
                .on_modifiers_changed(cx.listener(
                    |this, event: &ModifiersChangedEvent, _, cx| {
                        if let Some(vk) = virtual_key_from_modifiers(modifier_state(event)) {
                            this.capture_virtual_key(vk, cx);
                        }
                    },
                ))
                .on_action(cx.listener(|this, _: &FocusNext, _, cx| {
                    this.capture_virtual_key(0x09, cx);
                }))
            });
        h_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .py(sizes.spacing_sm)
            .px(sizes.padding_sm)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(Label::new(label_of(field, strings)).size(LabelSize::Small))
                    .when_some(tooltip_of(field, strings), |this, tooltip| {
                        this.child(
                            Label::new(tooltip)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .line_height(1.3),
                        )
                    }),
            )
            .child(button)
            .into_any_element()
    }

    fn cursor_row(
        &self,
        field: &'static ConfigField<String>,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let value = text_of(&value_in(field, values));
        let has_file = !value.is_empty();
        let invalid = self.invalid_cursors.contains(field.key());
        let name = if has_file {
            Label::new(cursor_display_name(&value).to_owned())
                .size(LabelSize::Small)
                .color(Color::Secondary)
                .truncate()
        } else {
            Label::new(strings.keybind_unbound())
                .size(LabelSize::Small)
                .color(Color::Muted)
                .italic()
        };
        let id = field_id(SCOPE, field);
        SettingRow::new(id.clone(), label_of(field, strings))
            .description_opt(tooltip_of(field, strings))
            .apply_marker(Marker::RestartOnly.kind(field.meta.restart_required))
            .hoverable(false)
            .below(
                v_flex()
                    .w_full()
                    .pt(sizes.spacing_md)
                    .child(
                        h_flex()
                            .w_full()
                            .gap(sizes.spacing_md)
                            .child(div().flex_1().min_w_0().child(name))
                            .child(
                                Button::new((id.clone(), "pick"), strings.naiom_cursor_pick())
                                    .controller_support(false)
                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.pick_cursor(field, cx)
                                    })),
                            )
                            .when(has_file, |this| {
                                this.child(
                                    Button::new((id.clone(), "clear"), strings.naiom_cursor_clear())
                                        .style(ButtonStyle::Tinted(TintColor::Muted))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.clear_cursor(field, cx)
                                        })),
                                )
                            }),
                    )
                    .when(invalid, |this| {
                        this.child(
                            h_flex()
                                .gap(sizes.spacing_md)
                                .pt(sizes.spacing_md)
                                .child(
                                    Icon::new(IconName::ErrorOutline)
                                        .size(IconSize::XSmall)
                                        .color(Color::Error),
                                )
                                .child(
                                    Label::new(strings.naiom_cursor_invalid())
                                        .size(LabelSize::XSmall)
                                        .color(Color::Error),
                                ),
                        )
                    }),
            )
            .into_any_element()
    }

    fn camera_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let camera_fix = nams::FIX_CAMERA_ACCELERATION.value_in(values);
        let third_person = nams::THIRD_PERSON_MODE.value_in(values);
        card("naiom-camera", strings.card_third_person_camera(), cx)
            .changed(changed_in(&CAMERA_FIELDS, values, strings))
            .child(self.row(&nams::FIX_CAMERA_ACCELERATION, values, true))
            .child(locked(
                camera_fix,
                strings.naiom_needs_camera_fix(),
                self.row(&nams::SENSITIVITY, values, camera_fix),
                window,
                cx,
            ))
            .child(row_divider(window, cx))
            .child(self.row(&nams::THIRD_PERSON_MODE, values, true))
            .child(locked(
                third_person,
                strings.naiom_needs_third_person(),
                v_flex()
                    .w_full()
                    .child(self.row(&nams::THIRD_PERSON_CHAR_FOLLOW, values, third_person))
                    .child(self.row(&nams::THIRD_PERSON_SMOOTHING, values, third_person))
                    .child(self.row(&nams::THIRD_PERSON_SENSITIVITY_X, values, third_person))
                    .child(self.row(&nams::THIRD_PERSON_SENSITIVITY_Y, values, third_person)),
                window,
                cx,
            ))
            .into_any_element()
    }

    fn aiming_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let aim = nams::AIM_MODE.value_in(values);
        let crosshair = nams::AIM_CROSSHAIR.value_in(values);
        let inner = v_flex()
            .w_full()
            .child(self.row(&nams::AIM_CROSSHAIR, values, aim))
            .child(PreviewImage::new(
                "naiom-preview-crosshair",
                asset_image("images/config/crosshair.webp"),
                strings.naiom_crosshair_preview_label(),
            ))
            .child(locked(
                crosshair,
                strings.naiom_needs_crosshair(),
                self.row(&nams::AIM_CROSSHAIR_ALWAYS, values, aim && crosshair),
                window,
                cx,
            ))
            .when(crosshair, |this| {
                this.child(note(
                    strings.naiom_crosshair_note(),
                    IconName::InfoOutline,
                    window,
                    cx,
                ))
            })
            .child(locked(
                !crosshair,
                strings.naiom_crosshair_overrides(),
                v_flex()
                    .w_full()
                    .child(
                        self.row(&nams::AIM_SENSITIVITY, values, aim && !crosshair)
                            .decimals(4),
                    )
                    .child(
                        self.row(&nams::AIM_OUTPUT_MULTIPLIER, values, aim && !crosshair)
                            .decimals(1),
                    ),
                window,
                cx,
            ))
            .child(self.row(&nams::AIM_SENSITIVITY_X, values, aim))
            .child(self.row(&nams::AIM_SENSITIVITY_Y, values, aim));
        card("naiom-aiming", strings.card_pod_aiming(), cx)
            .changed(changed_in(&AIMING_FIELDS, values, strings))
            .child(self.row(&nams::AIM_MODE, values, true))
            .child(locked(
                aim,
                strings.naiom_needs_aim_mode(),
                inner,
                window,
                cx,
            ))
            .into_any_element()
    }

    fn misc_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        card("naiom-misc", strings.card_misc(), cx)
            .changed(changed_in(&MISC_FIELDS, values, strings))
            .child(self.row(&nams::MISC_DISABLE_POD_PET, values, true))
            .child(self.debug_key_row(values, window, cx))
            .child(row_divider(window, cx))
            .child(self.cursor_row(&nams::MISC_CUSTOM_CURSOR_MENU, values, window, cx))
            .child(self.cursor_row(&nams::MISC_CUSTOM_CURSOR_HACKING, values, window, cx))
            .child(self.row(&nams::MISC_DISABLE_DEFAULT_CURSOR, values, true))
            .into_any_element()
    }

    fn binding_rows(
        &self,
        fields: &[&'static ConfigField<String>],
        all: &[(&'static str, String)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        fields
            .iter()
            .map(|field| self.binding_row(field, all, window, cx))
            .collect()
    }

    fn top_banner(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let field = &nams::DISABLE_INPUT_FEATURES;
        let config = self.config.clone();
        let key = field.key();
        let reset = has_non_defaults(values).then(|| {
            Button::new("naiom-reset", strings.lod_mod_reset_button())
                .style(ButtonStyle::Tinted(TintColor::Warning))
                .on_click(cx.listener(|this, _, window, cx| this.confirm_reset(window, cx)))
        });
        let sizes = window.sizes(cx);
        Card::new()
            .padding(CardPadding::Compact)
            .child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(
                        Icon::new(IconName::SportsEsportsOutlined)
                            .size(IconSize::Large)
                            .color(Color::Secondary),
                    )
                    .child(
                        v_flex().flex_1().min_w_0().child(
                            Label::new(strings.naiom_controller_note())
                                .size(LabelSize::XSmall)
                                .color(Color::Secondary)
                                .line_height(1.4),
                        ),
                    )
                    .children(reset),
            )
            .child(row_divider(window, cx))
            .child(
                FieldRow::toml(SCOPE, field, values).on_change(move |value, _, cx| {
                    config.update(cx, |store, cx| {
                        store.update_nams_now(key, value, None, cx)
                    });
                }),
            )
            .into_any_element()
    }
}

impl Render for NaiomTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        if let Some(key) = self.capturing
            && (yp_ui::ControllerMode::is_active(cx)
                || !self.focus.get(key).is_some_and(|handle| handle.is_focused(window)))
        {
            self.capturing = None;
            self.capture = Capture::new();
        }
        let store = self.config.read(cx);
        let loading = store.is_loading();
        let values = store.nams().clone();
        let naiom_error = store
            .error(ConfigFile::Naiom)
            .map(|error| toml_error_text(error, strings));
        let nams_error = store
            .error(ConfigFile::Nams)
            .map(|error| toml_error_text(error, strings));
        let failure = store
            .failure()
            .map(|failure| config_failure_text(failure, strings));
        let path = store.path(ConfigFile::Naiom);
        let header = PageHeader::new(strings.header_naiom());
        let header = match path {
            Some(path) => header.info(strings.tooltip_edits_naiom(), move |_, cx| {
                reveal(path.clone(), true, cx)
            }),
            None => header.info_tooltip(strings.tooltip_edits_naiom()),
        };
        let content = if loading {
            div().into_any_element()
        } else {
            let disabled = nams::DISABLE_INPUT_FEATURES.value_in(&values);
            let mut sections: Vec<AnyElement> = Vec::new();
            if let Some(error) = naiom_error {
                sections.push(
                    ConfigErrorBanner::new(ConfigFile::Naiom.file_name(), error)
                        .into_any_element(),
                );
            }
            if let Some(error) = nams_error {
                sections.push(
                    ConfigErrorBanner::new(ConfigFile::Nams.file_name(), error).into_any_element(),
                );
            }
            if let Some(failure) = failure {
                sections.push(failure_banner("naiom-failure", failure).into_any_element());
            }
            sections.push(self.top_banner(&values, window, cx));
            if !disabled {
                let all = binding_values(&values);
                let tap_evade = nams::MOVEMENT_DISABLE_TAP_EVADE.value_in(&values);
                let evade_unbound = binding_value(&bindings::NON_STANDARD_EVADE, &values).is_empty();
                let left = column(
                    [
                        self.camera_card(&values, window, cx),
                        self.aiming_card(&values, window, cx),
                        self.misc_card(&values, window, cx),
                    ],
                    window,
                    cx,
                );
                let movement = card(
                    "naiom-movement",
                    strings.card_movement_bindings(), cx
                )
                .changed(changed_in(
                    &[&nams::MOVEMENT_DISABLE_TAP_EVADE as FieldRef],
                    &values,
                    strings,
                ))
                .children(self.binding_rows(&MOVEMENT_BINDINGS, &all, window, cx))
                .child(row_divider(window, cx))
                .child(self.row(&nams::MOVEMENT_DISABLE_TAP_EVADE, &values, true))
                .when(tap_evade && evade_unbound, |this| {
                    this.child(
                        Callout::new()
                            .id("naiom-tap-evade-warning")
                            .severity(Severity::Warning)
                            .icon(IconName::WarningAmberRounded)
                            .description(strings.naiom_tap_evade_warning()),
                    )
                });
                let combat =
                    card("naiom-combat", strings.card_combat_bindings(), cx)
                        .children(self.binding_rows(&COMBAT_BINDINGS, &all, window, cx));
                let non_standard = card(
                    "naiom-non-standard",
                    strings.card_non_standard_bindings(), cx
                )
                .children(self.binding_rows(&NON_STANDARD_BINDINGS, &all, window, cx))
                .child(row_divider(window, cx))
                .children(self.binding_rows(&MODE_BINDINGS, &all, window, cx));
                let menu = card("naiom-menu", strings.card_menu_navigation(), cx)
                    .children(self.binding_rows(&MENU_BINDINGS, &all, window, cx));
                let right = column(
                    [
                        Callout::new()
                            .id("naiom-bindings-intro")
                            .severity(Severity::Accent)
                            .icon(IconName::KeyboardOutlined)
                            .description(strings.naiom_bindings_intro())
                            .into_any_element(),
                        movement.into_any_element(),
                        combat.into_any_element(),
                        non_standard.into_any_element(),
                        menu.into_any_element(),
                    ],
                    window,
                    cx,
                );
                sections.push(
                    TwoColumn::new(left, right)
                        .id("naiom-columns")
                        .into_any_element(),
                );
            }
            column(sections, window, cx).into_any_element()
        };
        tab_page("naiom-scroll", header, &self.scroll, loading, content, window, cx)
    }
}

pub fn debug_key_value(values: &toml::Table) -> i64 {
    number_of(&value_in(&nams::MISC_OPEN_DEBUG_MENU, values)).map_or(0, |value| value as i64)
}

pub fn input_features_disabled(values: &toml::Table) -> bool {
    bool_of(&value_in(&nams::DISABLE_INPUT_FEATURES, values))
}

#[cfg(test)]
mod tests {
    use yp_i18n::Locale;

    use super::*;

    fn table(text: &str) -> toml::Table {
        text.parse::<toml::Table>().unwrap()
    }

    #[test]
    fn every_binding_field_is_on_exactly_one_card() {
        let mut keys: Vec<&str> = MOVEMENT_BINDINGS
            .iter()
            .chain(&COMBAT_BINDINGS)
            .chain(&NON_STANDARD_BINDINGS)
            .chain(&MODE_BINDINGS)
            .chain(&MENU_BINDINGS)
            .map(|field| field.key())
            .collect();
        assert_eq!(keys.len(), bindings::ALL.len());
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), bindings::ALL.len());
    }

    #[test]
    fn every_mouse_field_is_on_the_page() {
        let mut keys: Vec<&str> = CAMERA_FIELDS
            .iter()
            .chain(&AIMING_FIELDS)
            .chain(&MISC_FIELDS)
            .map(|field| field.meta().key)
            .collect();
        keys.push(nams::MOVEMENT_DISABLE_TAP_EVADE.key());
        keys.push(nams::MISC_OPEN_DEBUG_MENU.key());
        for field in nams::MOUSE {
            assert!(keys.contains(&field.meta().key), "{}", field.meta().key);
        }
        assert_eq!(keys.len(), nams::MOUSE.len());
    }

    #[test]
    fn bindings_are_read_from_the_nested_table() {
        let values = table(
            "[mouse]\nsensitivity = 2.0\n[mouse.bindings]\nstandard_jump = \"SPACE\"\nstandard_fire = \"SPACE\"\n",
        );
        assert_eq!(binding_value(&bindings::STANDARD_JUMP, &values), "SPACE");
        assert_eq!(binding_value(&bindings::STANDARD_WALK, &values), "");
        let strings = Strings::for_locale(Locale::En);
        let all = binding_values(&values);
        assert_eq!(all.len(), bindings::ALL.len());
        assert_eq!(
            conflict_label(&bindings::STANDARD_JUMP, &all, strings),
            Some(label_of(&bindings::STANDARD_FIRE, strings))
        );
        assert_eq!(conflict_label(&bindings::STANDARD_WALK, &all, strings), None);
    }

    #[test]
    fn reset_is_offered_only_when_something_differs() {
        assert!(!has_non_defaults(&toml::Table::new()));
        assert!(!has_non_defaults(&table("[mouse]\nsensitivity = 2.0\n")));
        assert!(has_non_defaults(&table("[mouse]\nsensitivity = 2.5\n")));
        assert!(has_non_defaults(&table(
            "[mouse.bindings]\nstandard_jump = \"SPACE\"\n"
        )));
        assert!(has_non_defaults(&table("[mouse]\nmisc_open_debug_menu = 112\n")));
    }

    #[test]
    fn reset_writes_every_mouse_default_and_clears_every_binding() {
        let values = reset_values();
        assert_eq!(values.len(), nams::MOUSE.len() + bindings::ALL.len());
        assert!(values.iter().any(|(key, section, value)| {
            *key == "sensitivity" && *section == "mouse" && *value == toml::Value::Float(2.0)
        }));
        assert!(values.iter().any(|(key, section, value)| {
            *key == "standard_jump"
                && *section == "mouse.bindings"
                && *value == toml::Value::String(String::new())
        }));
        assert_eq!(debug_key_value(&table("[mouse]\nmisc_open_debug_menu = 112\n")), 112);
        assert!(input_features_disabled(&table("disable_input_features = true\n")));
    }
}
