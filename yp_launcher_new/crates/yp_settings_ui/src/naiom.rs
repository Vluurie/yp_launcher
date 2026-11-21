use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui::{
    FontWeight, KeyDownEvent, ModifiersChangedEvent, MouseButton, MouseDownEvent,
    NavigationDirection, PathPromptOptions, ScrollEvent, ScrollHandle, Stateful, Subscription,
    Task,
};
use serde_json::Value as Json;
use yp_config::{Setting, naiom_keys};
use yp_state::ConfigStore;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    Button, ButtonStyle, Card, CardPadding, FocusNext, FocusPrevious, Icon, IconName, IconSize,
    Label, LabelSize, Select, SelectItem, SettingRow, TintColor, TwoColumn,
};

use crate::binding::{
    Capture, ModifierState, PointerButton, conflict_for, controller_label, cursor_display_name,
    display_binding, display_virtual_key, is_controller_binding, is_valid_cursor_file,
    stored_cursor_path, virtual_key_from_key_down, virtual_key_from_modifiers,
};
use crate::fields::{Marker, field_id};
use crate::page::{column, row_divider, tab_page};
use crate::schema_page::{self, CardPlan, ResetText, RowContext, TabScope};

const SCOPE: &str = "naiom";
const FILE: &str = "naiom.toml";
pub const INPUT_FEATURES_ID: &str = "nams.disable_input_features";
pub const TAB: TabScope = TabScope::roots(&["input"]);
const MOUSE_PROBE: &str = "MOUSE1";

pub fn allows_mouse(setting: &Setting) -> bool {
    setting.keys.iter().any(|key| key == MOUSE_PROBE)
}

pub fn controller_options(setting: &Setting) -> Vec<&'static str> {
    naiom_keys::CONTROLLER_BUTTONS
        .iter()
        .copied()
        .filter(|button| setting.keys.iter().any(|key| key == button))
        .collect()
}

pub fn binding_settings(store: &ConfigStore) -> Vec<Setting> {
    TAB.settings(store.schema())
        .filter(|setting| setting.is_key_binding())
        .cloned()
        .collect()
}

pub fn binding_values(store: &ConfigStore) -> Vec<(String, String)> {
    binding_settings(store)
        .into_iter()
        .map(|setting| {
            let value = store.text(&setting.id);
            (setting.id, value)
        })
        .collect()
}

pub fn conflict_label(id: &str, all: &[(String, String)], store: &ConfigStore) -> Option<SharedString> {
    let other = conflict_for(id, all.iter().map(|(id, value)| (id.as_str(), value.as_str())))?;
    store
        .setting(other)
        .map(|setting| SharedString::from(setting.label.clone()))
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
    focus: HashMap<SharedString, FocusHandle>,
    capturing: Option<SharedString>,
    capture: Capture,
    suppress_click: Option<SharedString>,
    invalid_cursors: HashSet<SharedString>,
    cursor_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl NaiomTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx.stores().config.clone();
        let subscriptions = vec![cx.observe(&config, |_, _, cx| cx.notify())];
        Self {
            config,
            scroll: ScrollHandle::new(),
            focus: HashMap::new(),
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

    pub fn capturing(&self) -> Option<&SharedString> {
        self.capturing.as_ref()
    }

    pub fn capture_state(&self) -> &Capture {
        &self.capture
    }

    pub fn is_cursor_invalid(&self, id: &str) -> bool {
        self.invalid_cursors.iter().any(|invalid| invalid.as_ref() == id)
    }

    fn is_capturing(&self, id: &str) -> bool {
        self.capturing.as_ref().is_some_and(|current| current.as_ref() == id)
    }

    fn write(&self, id: &str, value: Json, cx: &mut App) {
        self.config.update(cx, |store, cx| store.set(id, value, cx));
    }

    fn focus_for(&mut self, id: &SharedString, cx: &mut Context<Self>) -> FocusHandle {
        self.focus
            .entry(id.clone())
            .or_insert_with(|| cx.focus_handle())
            .clone()
    }

    pub fn start_capture(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.focus_for(&id, cx);
        self.capturing = Some(id);
        self.capture = Capture::new();
        self.suppress_click = None;
        window.focus(&handle, cx);
        cx.notify();
    }

    pub fn cancel_capture(&mut self, cx: &mut Context<Self>) {
        if self.capturing.take().is_some() {
            self.capture = Capture::new();
            cx.notify();
        }
    }

    fn finish_binding(&mut self, id: &str, binding: String, cx: &mut Context<Self>) {
        self.capturing = None;
        self.capture = Capture::new();
        self.set_binding(id, &binding, cx);
        cx.notify();
    }

    pub fn set_binding(&mut self, id: &str, binding: &str, cx: &mut Context<Self>) {
        self.write(id, Json::String(binding.to_owned()), cx);
    }

    fn mouse_allowed(&self, id: &str, cx: &App) -> bool {
        self.config.read(cx).setting(id).is_some_and(allows_mouse)
    }

    pub fn capture_key(&mut self, id: &str, state: ModifierState, pressed: &str, cx: &mut Context<Self>) {
        if !self.is_capturing(id) {
            return;
        }
        if let Some(binding) = self.capture.key_down_with(state, pressed) {
            self.finish_binding(id, binding, cx);
        }
    }

    pub fn capture_modifiers(&mut self, id: &str, state: ModifierState, cx: &mut Context<Self>) {
        if !self.is_capturing(id) {
            return;
        }
        match self.capture.modifiers_changed(state) {
            Some(binding) => self.finish_binding(id, binding, cx),
            None => cx.notify(),
        }
    }

    pub fn capture_pointer(&mut self, id: &str, button: PointerButton, cx: &mut Context<Self>) {
        if !self.is_capturing(id) {
            return;
        }
        let allows_mouse = self.mouse_allowed(id, cx);
        if let Some(binding) = self.capture.pointer_down(button, allows_mouse) {
            self.suppress_click = (button == PointerButton::Left).then(|| SharedString::from(id.to_owned()));
            self.finish_binding(id, binding, cx);
        }
    }

    pub fn capture_wheel(&mut self, id: &str, delta_y: f32, cx: &mut Context<Self>) {
        if ControllerMode::is_active(cx) || !self.is_capturing(id) {
            return;
        }
        let allows_mouse = self.mouse_allowed(id, cx);
        if let Some(binding) = self.capture.wheel(delta_y, allows_mouse) {
            self.finish_binding(id, binding, cx);
        }
    }

    pub fn capture_virtual_key(&mut self, id: &str, vk: i64, cx: &mut Context<Self>) {
        if !self.is_capturing(id) {
            return;
        }
        self.capturing = None;
        self.capture = Capture::new();
        self.write(id, Json::from(vk), cx);
        cx.notify();
    }

    pub fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_capture(cx);
        self.invalid_cursors.clear();
        let text = Self::reset_text(cx);
        schema_page::confirm_reset(TAB, &text, &self.config, window, cx);
    }

    fn reset_text(cx: &App) -> ResetText {
        let strings = cx.t();
        ResetText {
            title: SharedString::new_static(strings.naiom_reset_confirm_title()),
            body: SharedString::new_static(strings.naiom_reset_confirm_body()),
        }
    }

    pub fn apply_cursor(&mut self, id: SharedString, path: PathBuf, cx: &mut Context<Self>) {
        let through_wine = cx.env().host.runs_game_through_wine();
        let check_path = path.clone();
        let check = cx.background_spawn(async move { is_valid_cursor_file(&check_path) });
        self.cursor_task = Some(cx.spawn(async move |this, cx| {
            let valid = check.await;
            this.update(cx, |this, cx| {
                if valid {
                    this.invalid_cursors.remove(&id);
                    let stored = stored_cursor_path(&path, through_wine);
                    this.write(&id, Json::String(stored), cx);
                } else {
                    this.invalid_cursors.insert(id);
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn pick_cursor(&mut self, id: SharedString, cx: &mut Context<Self>) {
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
            this.update(cx, |this, cx| this.apply_cursor(id, path, cx)).ok();
        }));
    }

    fn clear_cursor(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.write(&id, Json::String(String::new()), cx);
        self.invalid_cursors.remove(&id);
        cx.notify();
    }

    fn key_button(
        &mut self,
        id: &SharedString,
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
        let capturing = self.is_capturing(id);
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
        let selector = format!("naiom-key-{id}");
        let button = h_flex()
            .id(SharedString::from(selector.clone()))
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
        let button = button.debug_selector(move || selector.clone());
        let handle = self.focus_for(id, cx);
        button.track_focus(&handle.tab_stop(!controller_mode))
    }

    fn binding_row(
        &mut self,
        setting: &Setting,
        all: &[(String, String)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let id = SharedString::from(setting.id.clone());
        let value = all
            .iter()
            .find(|(other, _)| *other == setting.id)
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        let capturing = self.is_capturing(&id);
        let mouse = allows_mouse(setting);
        let conflict = conflict_label(&setting.id, all, self.config.read(cx));
        let controller = is_controller_binding(&value);
        let text = if capturing {
            self.capture
                .pending_label()
                .map(SharedString::from)
                .unwrap_or_else(|| SharedString::new_static(strings.press_key()))
        } else {
            SharedString::from(display_binding(&value, strings))
        };
        let click_id = id.clone();
        let key_id = id.clone();
        let modifier_id = id.clone();
        let pointer_id = id.clone();
        let wheel_id = id.clone();
        let next_id = id.clone();
        let previous_id = id.clone();
        let button = self
            .key_button(
                &id,
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
                if yp_ui::ControllerMode::is_active(cx) || this.is_capturing(&click_id) {
                    return;
                }
                if this.suppress_click.take().as_ref() == Some(&click_id) {
                    return;
                }
                this.start_capture(click_id.clone(), window, cx);
            }))
            .when(capturing, |this| {
                this.on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                    cx.stop_propagation();
                    let state = ModifierState {
                        shift: event.keystroke.modifiers.shift,
                        ctrl: event.keystroke.modifiers.control,
                        alt: event.keystroke.modifiers.alt,
                    };
                    this.capture_key(&key_id, state, &event.keystroke.key, cx);
                }))
                .on_modifiers_changed(cx.listener(move |this, event: &ModifiersChangedEvent, _, cx| {
                    this.capture_modifiers(&modifier_id, modifier_state(event), cx);
                }))
                .on_any_mouse_down(cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.capture_pointer(&pointer_id, pointer_button(event.button), cx);
                }))
                .on_scroll(cx.listener(move |this, event: &ScrollEvent, window, cx| {
                    cx.stop_propagation();
                    let delta = event.delta.pixel_delta(window.line_height());
                    this.capture_wheel(&wheel_id, f32::from(delta.y), cx);
                }))
                .on_action(cx.listener(move |this, _: &FocusNext, _, cx| {
                    this.capture_key(&next_id, ModifierState::default(), "tab", cx);
                }))
                .on_action(cx.listener(move |this, _: &FocusPrevious, _, cx| {
                    let state = ModifierState {
                        shift: true,
                        ..ModifierState::default()
                    };
                    this.capture_key(&previous_id, state, "tab", cx);
                }))
            });
        let detail = if capturing && !mouse {
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
        } else if setting.description.is_empty() {
            div().into_any_element()
        } else {
            Label::new(setting.description.clone())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .line_height(1.3)
                .into_any_element()
        };
        let options = controller_options(setting);
        let picker = (!options.is_empty()).then(|| {
            let select_id = id.clone();
            Select::new(SharedString::from(format!("naiom-controller-{id}")), SharedString::default())
                .trigger_icon(IconName::SportsEsportsOutlined)
                .tooltip(strings.keybind_pick_controller())
                .selected_index(options.iter().position(|option| *option == value))
                .items(
                    options
                        .iter()
                        .map(|option| SelectItem::new(controller_label(option, strings).unwrap_or(option), *option)),
                )
                .on_select(cx.listener(move |this, option: &&'static str, _, cx| {
                    this.cancel_capture(cx);
                    this.set_binding(&select_id, option, cx);
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
                            .child(Label::new(setting.label.clone()).size(LabelSize::Small))
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

    fn virtual_key_row(&mut self, setting: &Setting, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let id = SharedString::from(setting.id.clone());
        let vk = self.config.read(cx).number(&setting.id).map_or(0, |value| value as i64);
        let capturing = self.is_capturing(&id);
        let text = if capturing {
            SharedString::new_static(strings.press_key())
        } else {
            SharedString::from(display_virtual_key(vk, strings))
        };
        let click_id = id.clone();
        let key_id = id.clone();
        let modifier_id = id.clone();
        let tab_id = id.clone();
        let button = self
            .key_button(
                &id,
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
                if !this.is_capturing(&click_id) {
                    this.start_capture(click_id.clone(), window, cx);
                }
            }))
            .when(capturing, |this| {
                this.on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                    cx.stop_propagation();
                    if let Some(vk) = virtual_key_from_key_down(&event.keystroke.key) {
                        this.capture_virtual_key(&key_id, vk, cx);
                    }
                }))
                .on_modifiers_changed(cx.listener(move |this, event: &ModifiersChangedEvent, _, cx| {
                    if let Some(vk) = virtual_key_from_modifiers(modifier_state(event)) {
                        this.capture_virtual_key(&modifier_id, vk, cx);
                    }
                }))
                .on_action(cx.listener(move |this, _: &FocusNext, _, cx| {
                    this.capture_virtual_key(&tab_id, 0x09, cx);
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
                    .child(Label::new(setting.label.clone()).size(LabelSize::Small))
                    .when(!setting.description.is_empty(), |this| {
                        this.child(
                            Label::new(setting.description.clone())
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .line_height(1.3),
                        )
                    }),
            )
            .child(button)
            .into_any_element()
    }

    fn cursor_row(&mut self, setting: &Setting, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let value = self.config.read(cx).text(&setting.id);
        let has_file = !value.is_empty();
        let invalid = self.is_cursor_invalid(&setting.id);
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
        let id = field_id(SCOPE, &setting.id);
        let pick_id = SharedString::from(setting.id.clone());
        let clear_id = pick_id.clone();
        SettingRow::new(id.clone(), setting.label.clone())
            .description_opt(Some(SharedString::from(setting.description.clone())).filter(|text| !text.is_empty()))
            .apply_marker(Marker::RestartOnly.kind(setting.restart_required))
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
                                    .on_click(cx.listener(move |this, _, _, cx| this.pick_cursor(pick_id.clone(), cx))),
                            )
                            .when(has_file, |this| {
                                this.child(
                                    Button::new((id.clone(), "clear"), strings.naiom_cursor_clear())
                                        .style(ButtonStyle::Tinted(TintColor::Muted))
                                        .on_click(cx.listener(move |this, _, _, cx| this.clear_cursor(clear_id.clone(), cx))),
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

    fn rows(
        &mut self,
        settings: &[Setting],
        all: &[(String, String)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut context = RowContext::default();
        let mut rows = Vec::new();
        for setting in settings {
            if setting.id == INPUT_FEATURES_ID {
                continue;
            }
            let element = if setting.is_key_binding() {
                self.binding_row(setting, all, window, cx)
            } else if setting.is_virtual_key() {
                self.virtual_key_row(setting, window, cx)
            } else if setting.is_path() {
                self.cursor_row(setting, window, cx)
            } else {
                schema_page::field(SCOPE, setting, &self.config, Marker::RestartOnly, cx).into_any_element()
            };
            rows.extend(context.wrap(setting, element, &self.config, window, cx));
        }
        rows
    }

    fn card(&mut self, plan: CardPlan, all: &[(String, String)], window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let rows = self.rows(&plan.settings, all, window, cx);
        let advanced = self.rows(&plan.advanced, all, window, cx);
        schema_page::finish_card(plan, rows, advanced, window, cx).into_any_element()
    }

    fn top_banner(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let store = self.config.read(cx);
        let switch = store
            .setting(INPUT_FEATURES_ID)
            .cloned()
            .map(|setting| schema_page::field(SCOPE, &setting, &self.config, Marker::RestartOnly, cx));
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
                    ),
            )
            .when_some(switch, |this, switch| this.child(row_divider(window, cx)).child(switch))
            .into_any_element()
    }
}

impl Render for NaiomTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        if let Some(id) = self.capturing.clone()
            && (yp_ui::ControllerMode::is_active(cx)
                || !self.focus.get(&id).is_some_and(|handle| handle.is_focused(window)))
        {
            self.capturing = None;
            self.capture = Capture::new();
        }
        let store = self.config.read(cx);
        let loading = schema_page::is_loading(store);
        let reset = TAB
            .has_changes(store)
            .then(|| schema_page::reset_button("naiom-reset", TAB, Self::reset_text(cx), &self.config, cx));
        let header = schema_page::header(
            strings.header_naiom(),
            strings.tooltip_edits_naiom(),
            Some(FILE),
            reset,
            &self.config,
            cx,
        );
        let content = if loading {
            div().into_any_element()
        } else {
            let disabled = self.config.read(cx).bool(INPUT_FEATURES_ID);
            let mut sections = schema_page::problems(SCOPE, &TAB, &self.config, cx);
            sections.push(self.top_banner(window, cx));
            if !disabled {
                let all = binding_values(self.config.read(cx));
                let plans = schema_page::plans(SCOPE, &TAB, self.config.read(cx));
                let (bindings, others): (Vec<CardPlan>, Vec<CardPlan>) = plans
                    .into_iter()
                    .filter(|plan| {
                        plan.settings
                            .iter()
                            .chain(&plan.advanced)
                            .any(|setting| setting.id != INPUT_FEATURES_ID)
                    })
                    .partition(|plan| plan.settings.iter().any(Setting::is_key_binding));
                let left: Vec<AnyElement> = others.into_iter().map(|plan| self.card(plan, &all, window, cx)).collect();
                let right: Vec<AnyElement> = bindings.into_iter().map(|plan| self.card(plan, &all, window, cx)).collect();
                sections.push(
                    TwoColumn::new(column(left, window, cx), column(right, window, cx))
                        .id("naiom-columns")
                        .into_any_element(),
                );
            }
            column(sections, window, cx).into_any_element()
        };
        tab_page("naiom-scroll", header, &self.scroll, loading, content, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use yp_config::testing::sample_schema;
    use yp_config::Schema;

    use super::*;

    fn schema() -> Schema {
        serde_json::from_value(sample_schema()).unwrap()
    }

    #[test]
    fn bindings_take_their_mouse_and_controller_rules_from_the_schema() {
        let schema = schema();
        let jump = schema.setting("naiom.bindings.standard_jump").unwrap();
        assert!(allows_mouse(jump));
        let mut keyboard_only = jump.clone();
        keyboard_only.keys.retain(|key| !key.starts_with("MOUSE"));
        assert!(!allows_mouse(&keyboard_only));
        assert!(controller_options(&keyboard_only).is_empty());
    }

    #[test]
    fn conflicts_name_the_other_binding() {
        let all = vec![
            ("naiom.bindings.standard_jump".to_owned(), "SPACE".to_owned()),
            ("naiom.bindings.standard_fire".to_owned(), "SPACE".to_owned()),
            ("naiom.bindings.standard_walk".to_owned(), String::new()),
        ];
        let other = conflict_for(
            "naiom.bindings.standard_jump",
            all.iter().map(|(id, value)| (id.as_str(), value.as_str())),
        );
        assert_eq!(other, Some("naiom.bindings.standard_fire"));
        let none = conflict_for(
            "naiom.bindings.standard_walk",
            all.iter().map(|(id, value)| (id.as_str(), value.as_str())),
        );
        assert_eq!(none, None);
    }
}
