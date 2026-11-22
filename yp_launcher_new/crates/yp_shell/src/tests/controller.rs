use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    div, prelude::*, px, Context, IntoElement, Render, ScrollHandle, TestAppContext, VisualTestContext, Window,
};
use yp_host::controller::{ControllerAction, Direction, CONTROLLER_NAVIGATION};
use yp_state::prelude::*;
use yp_ui::{
    h_flex, v_flex, Button, ButtonCommon, Checkbox, Clickable, ControllerMode, Disableable, IconName, Input,
    InputState, KeyCapture, Link, NumberField, Root, ScrollableElement, Select, SelectItem, Sidebar, SidebarGroup,
    SidebarItem, Slider, ToggleState, WindowExt,
};

use super::{has_modal, open_main, Harness};
use crate::controller;
use crate::directory::{InstallChoice, InstallChoiceDialog, InstallDialogKind};

fn send(action: ControllerAction, cx: &mut VisualTestContext) {
    let handle = cx.update(|window, _| window.window_handle().downcast::<Root>().unwrap());
    controller::dispatch_pending(handle, [action], cx).unwrap();
    cx.run_until_parked();
}

struct Controls {
    clicks: Rc<Cell<usize>>,
    checked: bool,
    selected: usize,
    value: f32,
    scroll: ScrollHandle,
}

impl Render for Controls {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.clicks.clone();
        v_flex().size_full().child(
            div()
                .id("controller-test-scroll")
                .size_full()
                .overflow_y_scrollbar(&self.scroll)
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            Button::new("controller-test-button", "Activate")
                                .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                        )
                        .child(
                            Button::new("controller-test-disabled", "Disabled")
                                .disabled(true)
                                .on_click(|_, _, _| panic!("disabled button activated")),
                        )
                        .child(
                            Checkbox::new("controller-test-checkbox", self.checked)
                                .label("Enabled")
                                .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                                    this.checked = state.selected();
                                    cx.notify();
                                })),
                        )
                        .child(
                            Select::new("controller-test-select", self.selected.to_string())
                                .items(vec![SelectItem::new("First", 0), SelectItem::new("Second", 1)])
                                .selected_index(Some(self.selected))
                                .searchable(true)
                                .on_select(cx.listener(|this, value, _, cx| {
                                    this.selected = *value;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Slider::new("controller-test-slider")
                                .min(0.)
                                .max(10.)
                                .step(1.)
                                .value(self.value)
                                .on_change(cx.listener(|this, value, _, cx| {
                                    this.value = *value;
                                    cx.notify();
                                })),
                        )
                        .child(div().h(px(2000.))),
                ),
        )
    }
}

#[gpui::test]
fn controller_navigates_buttons_checkboxes_menus_and_sliders(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let clicks = Rc::new(Cell::new(0));
    let mut controls = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|_| Controls {
            clicks: clicks.clone(),
            checked: false,
            selected: 0,
            value: 5.,
            scroll: ScrollHandle::new(),
        });
        controls = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    let controls = controls.unwrap();
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(clicks.get(), 1);
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert!(controls.read_with(cx, |controls, _| controls.checked));
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(controls.read_with(cx, |controls, _| controls.selected), 1);
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Move(Direction::Right), cx);
    assert_eq!(controls.read_with(cx, |controls, _| controls.value), 6.);
    send(ControllerAction::Move(Direction::Left), cx);
    assert_eq!(controls.read_with(cx, |controls, _| controls.value), 5.);
    send(ControllerAction::ScrollDown, cx);
    assert!(controls.read_with(cx, |controls, _| controls.scroll.offset().y) < px(0.));
}

#[gpui::test]
fn controller_can_choose_search_all_drives(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let (sender, mut receiver) = futures::channel::oneshot::channel();
    cx.update(|window, cx| {
        window.activate_window();
        window.open_modal(cx, |_, cx| {
            InstallChoiceDialog::new(
                InstallDialogKind::FastResults,
                vec![yp_core::NierInstallation {
                    path: harness.game.root().to_path_buf(),
                    has_data: true,
                }],
                sender,
                cx,
            )
        });
    });
    cx.run_until_parked();
    send(ControllerAction::Move(Direction::Up), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(receiver.try_recv().unwrap(), Some(InstallChoice::DeepSearch));
    assert!(!has_modal(&opened.root, cx));
}

#[gpui::test]
fn controller_searches_drives_and_accepts_the_result(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let install = yp_core::NierInstallation {
        path: harness.game.root().to_path_buf(),
        has_data: true,
    };
    harness.platform.fast_installs.lock().unwrap().push(install.clone());
    harness.platform.deep_installs.lock().unwrap().push(install);
    let stores = harness.init(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    cx.update(|window, _| window.activate_window());
    let selector = cx.update(|window, cx| cx.new(|cx| crate::DirectorySelector::new(window, cx)));
    selector.update_in(cx, |selector, window, cx| selector.run_auto_search(window, cx));
    cx.run_until_parked();
    assert!(has_modal(&opened.root, cx));
    send(ControllerAction::Move(Direction::Up), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(
        harness.platform.deep_searches.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    assert!(has_modal(&opened.root, cx));
    send(ControllerAction::Move(Direction::Up), cx);
    send(ControllerAction::Confirm, cx);
    assert!(!has_modal(&opened.root, cx));
    assert!(stores
        .session
        .read_with(cx, |session, _| session.is_directory_selected()));
    assert!(!stores.session.read_with(cx, |session, _| session.auto_searching()));
}

#[gpui::test]
fn controller_can_select_an_installation_row(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let (sender, mut receiver) = futures::channel::oneshot::channel();
    cx.update(|window, cx| {
        window.activate_window();
        window.open_modal(cx, |_, cx| {
            InstallChoiceDialog::new(
                InstallDialogKind::DeepResults,
                vec![yp_core::NierInstallation {
                    path: harness.game.root().to_path_buf(),
                    has_data: true,
                }],
                sender,
                cx,
            )
        });
    });
    cx.run_until_parked();
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(
        receiver.try_recv().unwrap(),
        Some(InstallChoice::Selected(harness.game.root().to_path_buf()))
    );
    assert!(!has_modal(&opened.root, cx));
}

#[gpui::test]
fn shoulders_cycle_available_tabs_and_back_returns_to_the_launcher(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    send(ControllerAction::NextTab, cx);
    assert_eq!(
        stores.navigation.read_with(cx, |navigation, _| navigation.active()),
        TabId::LauncherSettings
    );
    send(ControllerAction::PreviousTab, cx);
    assert_eq!(
        stores.navigation.read_with(cx, |navigation, _| navigation.active()),
        TabId::Launcher
    );
    send(ControllerAction::NextTab, cx);
    send(ControllerAction::Back, cx);
    assert_eq!(
        stores.navigation.read_with(cx, |navigation, _| navigation.active()),
        TabId::Launcher
    );
}

#[gpui::test]
fn controller_respects_modals_disabled_settings_and_inactive_windows(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    cx.update(|window, cx| {
        let _confirm = yp_ui::confirm(yp_ui::ConfirmOptions::new("Confirm", "Test"), window, cx);
    });
    cx.run_until_parked();
    assert!(has_modal(&opened.root, cx));
    send(ControllerAction::NextTab, cx);
    assert_eq!(
        stores.navigation.read_with(cx, |navigation, _| navigation.active()),
        TabId::Launcher
    );
    send(ControllerAction::Back, cx);
    assert!(!has_modal(&opened.root, cx));
    cx.update(|_, cx| cx.prefs().set(&CONTROLLER_NAVIGATION, &false));
    send(ControllerAction::NextTab, cx);
    assert_eq!(
        stores.navigation.read_with(cx, |navigation, _| navigation.active()),
        TabId::Launcher
    );
    cx.update(|_, cx| cx.prefs().set(&CONTROLLER_NAVIGATION, &true));
    cx.deactivate_window();
    cx.run_until_parked();
    send(ControllerAction::NextTab, cx);
    assert_eq!(
        stores.navigation.read_with(cx, |navigation, _| navigation.active()),
        TabId::Launcher
    );
}

struct OverlayButton(Rc<Cell<usize>>);

impl Render for OverlayButton {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.0.clone();
        div().size_full().child(
            Button::new("controller-overlay-button", "Overlay action")
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
        )
    }
}

#[gpui::test]
fn controller_can_open_message_history_after_a_toast_disappears(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let clicks = Rc::new(Cell::new(0));
    let (root, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|_| OverlayButton(clicks.clone()));
        Root::new(view, window, cx)
    });
    cx.update(|window, cx| {
        window.activate_window();
        let id = window
            .push_toast(yp_ui::Toast::error("Missed failure"), cx)
            .unwrap();
        window.dismiss_toast(&id, cx);
    });
    cx.run_until_parked();
    cx.executor().advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert!(!has_modal(&root, cx));
    assert!(cx.update(|window, _| {
        window
            .context_stack()
            .iter()
            .any(|context| context.contains("YpToastHistory"))
    }));
    assert_eq!(clicks.get(), 0);
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
        Some("Missed failure".to_owned())
    );
    send(ControllerAction::Back, cx);
    assert!(!has_modal(&root, cx));
    assert!(!cx.update(|window, _| {
        window
            .context_stack()
            .iter()
            .any(|context| context.contains("YpToastHistory"))
    }));
}

#[gpui::test]
fn controller_focus_stays_inside_the_top_overlay(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let background_clicks = Rc::new(Cell::new(0));
    let overlay_clicks = Rc::new(Cell::new(0));
    let (_, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|_| OverlayButton(background_clicks.clone()));
        Root::new(view, window, cx)
    });
    cx.update(|window, cx| {
        window.activate_window();
        let overlay = cx.new(|_| OverlayButton(overlay_clicks.clone()));
        window.show_overlay("controller-test-overlay", overlay, cx);
    });
    cx.run_until_parked();
    for _ in 0..4 {
        send(ControllerAction::Move(Direction::Down), cx);
        send(ControllerAction::Confirm, cx);
    }
    assert_eq!(overlay_clicks.get(), 4);
    assert_eq!(background_clicks.get(), 0);
}

struct SidebarControls {
    selected: Rc<Cell<usize>>,
    expanded: Rc<Cell<bool>>,
    icon_only: bool,
}

impl Render for SidebarControls {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let first = self.selected.clone();
        let second = self.selected.clone();
        let third = self.selected.clone();
        let expanded = self.expanded.clone();
        Sidebar::new()
            .icon_only(self.icon_only)
            .group(
                SidebarGroup::new("General")
                    .item(
                        SidebarItem::new("controller-tab-first", IconName::PlayCircleOutline, "Launcher")
                            .on_click(move |_, _, _| first.set(1)),
                    )
                    .item(
                        SidebarItem::new("controller-tab-second", IconName::SettingsOutlined, "Settings")
                            .on_click(move |_, _, _| second.set(2)),
                    ),
            )
            .group(
                SidebarGroup::new("Config")
                    .collapsible()
                    .collapsed(!self.expanded.get())
                    .on_toggle(cx.listener(move |_, _, _, cx| {
                        expanded.set(!expanded.get());
                        cx.notify();
                    }))
                    .item(
                        SidebarItem::new("controller-tab-third", IconName::Tune, "NAMS")
                            .on_click(move |_, _, _| third.set(3)),
                    ),
            )
    }
}

fn check_sidebar_controller_navigation(cx: &mut TestAppContext, icon_only: bool) {
    let harness = Harness::new();
    harness.init(cx);
    let selected = Rc::new(Cell::new(0));
    let expanded = Rc::new(Cell::new(false));
    let (_, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|_| SidebarControls {
            selected: selected.clone(),
            expanded: expanded.clone(),
            icon_only,
        });
        Root::new(view, window, cx)
    });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    for expected in [1, 2] {
        send(ControllerAction::Move(Direction::Down), cx);
        let focused = cx.update(|window, cx| window.focused(cx));
        send(ControllerAction::Move(Direction::Right), cx);
        cx.update(|window, cx| assert_eq!(window.focused(cx), focused));
        send(ControllerAction::Confirm, cx);
        assert_eq!(selected.get(), expected);
    }
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert!(expanded.get());
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(selected.get(), 3);
    cx.update(|_, cx| assert!(yp_ui::FocusVisible::get(cx)));
}

#[gpui::test]
fn controller_can_focus_sidebar_tabs_and_expand_config(cx: &mut TestAppContext) {
    check_sidebar_controller_navigation(cx, false);
}

#[gpui::test]
fn controller_can_focus_icon_only_sidebar_tabs_and_expand_config(cx: &mut TestAppContext) {
    check_sidebar_controller_navigation(cx, true);
}

struct SidebarPage {
    sidebar: gpui::Entity<SidebarControls>,
    clicks: Rc<Cell<usize>>,
    value: f32,
}

impl Render for SidebarPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.clicks.clone();
        h_flex().size_full().child(self.sidebar.clone()).child(
            v_flex()
                .flex_1()
                .child(
                    Button::new("sidebar-page-picker", "Browse files")
                        .controller_support(false)
                        .on_click(|_, _, _| panic!("controller opened a native picker")),
                )
                .child(
                    Button::new("sidebar-page-action", "Page action")
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                )
                .child(
                    Slider::new("sidebar-page-slider")
                        .min(0.)
                        .max(10.)
                        .step(1.)
                        .value(self.value)
                        .on_change(cx.listener(|this, value, _, cx| {
                            this.value = *value;
                            cx.notify();
                        })),
                ),
        )
    }
}

fn check_sidebar_enters_page(cx: &mut TestAppContext, icon_only: bool) {
    let harness = Harness::new();
    harness.init(cx);
    let selected = Rc::new(Cell::new(0));
    let clicks = Rc::new(Cell::new(0));
    let mut page = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        let sidebar = cx.new(|_| SidebarControls {
            selected: selected.clone(),
            expanded: Rc::new(Cell::new(true)),
            icon_only,
        });
        let view = cx.new(|_| SidebarPage {
            sidebar,
            clicks: clicks.clone(),
            value: 5.,
        });
        page = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    for (expected, steps) in [(1, 1), (2, 2), (3, 4)] {
        cx.update(|window, cx| window.blur(cx));
        for _ in 0..steps {
            send(ControllerAction::Move(Direction::Down), cx);
        }
        send(ControllerAction::Confirm, cx);
        assert_eq!(selected.get(), expected);
        send(ControllerAction::Move(Direction::Right), cx);
        send(ControllerAction::Confirm, cx);
        assert_eq!(clicks.get(), expected);
        assert_eq!(selected.get(), expected);
    }
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Move(Direction::Right), cx);
    assert_eq!(page.unwrap().read_with(cx, |page, _| page.value), 6.);
}

#[gpui::test]
fn controller_right_enters_the_page_from_each_sidebar_tab(cx: &mut TestAppContext) {
    check_sidebar_enters_page(cx, false);
}

#[gpui::test]
fn controller_right_enters_the_page_from_each_icon_only_sidebar_tab(cx: &mut TestAppContext) {
    check_sidebar_enters_page(cx, true);
}

struct ControllerAvailability {
    input: gpui::Entity<InputState>,
    clicks: Rc<Cell<usize>>,
    key_captures: Rc<Cell<usize>>,
    value: f64,
}

impl Render for ControllerAvailability {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let forbidden_clicks = self.clicks.clone();
        let folder_clicks = self.clicks.clone();
        let clicks = self.clicks.clone();
        let key_captures = self.key_captures.clone();
        v_flex()
            .size_full()
            .child(Input::new(&self.input))
            .child(
                Button::new("controller-native-picker", "Browse files")
                    .controller_support(false)
                    .tab_index(0)
                    .on_click(move |_, _, _| forbidden_clicks.set(forbidden_clicks.get() + 100)),
            )
            .child(
                Link::new("controller-folder-link", "Open folder")
                    .controller_support(false)
                    .on_click(move |_, _, _| folder_clicks.set(folder_clicks.get() + 100)),
            )
            .child(
                KeyCapture::new("controller-key-capture", "Space")
                    .on_capture(move |_, _, _| key_captures.set(key_captures.get() + 1)),
            )
            .child(
                Button::new("controller-supported", "Supported action")
                    .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
            )
            .child(
                NumberField::new("controller-number", self.value)
                    .min(0.)
                    .max(10.)
                    .steppers(true)
                    .on_change(cx.listener(|this, value, _, cx| {
                        this.value = *value;
                        cx.notify();
                    })),
            )
    }
}

#[gpui::test]
fn controller_skips_keyboard_only_controls_and_keeps_number_steppers_working(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let clicks = Rc::new(Cell::new(0));
    let captures = Rc::new(Cell::new(0));
    let mut controls = None;
    let mut input = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        let text = cx.new(|cx| InputState::new(window, cx).default_value("unchanged"));
        input = Some(text.clone());
        let view = cx.new(|_| ControllerAvailability {
            input: text,
            clicks: clicks.clone(),
            key_captures: captures.clone(),
            value: 5.,
        });
        controls = Some(view.clone());
        Root::new(view, window, cx)
    });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    let input = input.unwrap();
    let controls = controls.unwrap();
    assert!(!input.read_with(cx, |input, _| input.is_disabled()));
    send(ControllerAction::Move(Direction::Down), cx);
    assert!(input.read_with(cx, |input, _| input.is_disabled()));
    cx.update(|_, cx| assert!(ControllerMode::is_active(cx)));
    send(ControllerAction::Confirm, cx);
    assert_eq!(clicks.get(), 1);
    assert_eq!(captures.get(), 0);
    assert_eq!(input.read_with(cx, |input, _| input.value().to_owned()), "unchanged");
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(controls.read_with(cx, |controls, _| controls.value), 4.);
    send(ControllerAction::Move(Direction::Down), cx);
    send(ControllerAction::Confirm, cx);
    assert_eq!(controls.read_with(cx, |controls, _| controls.value), 5.);
    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    cx.update(|_, cx| assert!(!ControllerMode::is_active(cx)));
    assert!(!input.read_with(cx, |input, _| input.is_disabled()));
    send(ControllerAction::Move(Direction::Down), cx);
    cx.simulate_click(gpui::point(px(5.), px(5.)), gpui::Modifiers::none());
    cx.run_until_parked();
    cx.update(|_, cx| assert!(!ControllerMode::is_active(cx)));
    assert!(!input.read_with(cx, |input, _| input.is_disabled()));
}
