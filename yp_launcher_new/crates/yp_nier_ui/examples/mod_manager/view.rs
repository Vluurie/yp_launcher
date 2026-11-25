use super::catalog::{self, Mod};
use gpui::Focusable;
use std::time::Instant;
mod controls;
mod models;
use gpui::{
    Context, Entity, FocusHandle, IntoElement, KeyDownEvent, Render, ScrollStrategy, SharedString,
    UniformListScrollHandle, Window, div, prelude::*, px,
};
use yp_nier_ui::{
    ConfirmDialog, EffectLayer, Focus, GameText, GridNoise, HelpBar, Lens, MenuAction, MenuFrame,
    MenuInput, MenuList, MenuPanel, MenuRow, MenuTab, PauseMenu,
};

pub struct ModManager {
    pub(super) opened_at: Instant,
    closing_at: Option<Instant>,
    focus: FocusHandle,
    menu: PauseMenu,
    mods: Vec<Mod>,
    display: usize,
    scroll: UniformListScrollHandle,
    profile: usize,
    previous_profile: Option<usize>,
    resolution: usize,
    rotation_speed: usize,
    priorities: Vec<usize>,
    packages: Vec<String>,
    pending: usize,
    dialog_answer: Option<bool>,
    model: models::ModelState,
    profile_name: String,
    name_editor: Entity<yp_nier_ui::MenuFieldState>,
}

impl ModManager {
    pub fn selection(&self) -> (usize, Focus) {
        (self.display, self.menu.focus())
    }
    pub fn resolution(&self) -> usize {
        self.resolution
    }
    pub fn rotation_speed(&self) -> usize {
        self.rotation_speed
    }
    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }
    pub fn scroll_offset(&self) -> f32 {
        -f32::from(self.scroll.0.borrow().base_handle.offset().y)
    }
    pub fn settings_values(&self) -> (bool, usize, usize) {
        (self.mods[0].enabled, self.profile, self.priorities[0])
    }
    pub fn page(&self) -> usize {
        self.menu.page_index()
    }
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mods = catalog::catalog();
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let owner = cx.weak_entity();
        window.on_window_should_close(cx, move |w, cx| {
            owner
                .update(cx, |view, cx| {
                    if view.closing_at.is_some() {
                        return true;
                    }
                    view.begin_close(w, cx);
                    false
                })
                .unwrap_or(true)
        });
        let name_editor = cx.new(|cx| yp_nier_ui::MenuFieldState::new("Default", cx));
        cx.subscribe_in(&name_editor, window, |v: &mut Self, field, event, w, cx| {
            match event {
                yp_nier_ui::FieldEvent::Submitted => {
                    let name = field.read(cx).value().trim();
                    if !name.is_empty() {
                        v.profile_name = name.to_owned();
                    }
                }
                yp_nier_ui::FieldEvent::Cancelled => {}
                yp_nier_ui::FieldEvent::Edited => return,
            }
            field.update(cx, |f, cx| f.set_value(&v.profile_name, w, cx));
            v.focus.focus(w, cx);
            cx.notify();
        })
        .detach();
        Self {
            opened_at: Instant::now(),
            closing_at: None,
            name_editor,
            profile_name: "Default".to_owned(),
            focus,
            menu: catalog::navigation(&mods),
            priorities: vec![5; mods.len()],
            profile: 0,
            previous_profile: None,
            resolution: 14,
            rotation_speed: 8,
            packages: Vec::new(),
            pending: 0,
            model: Default::default(),
            dialog_answer: None,
            mods,
            display: 0,
            scroll: Default::default(),
        }
    }

    pub fn input(&mut self, key: MenuInput, window: &mut Window, cx: &mut Context<Self>) {
        if self.closing_at.is_some() {
            window.remove_window();
            return;
        }
        if key == MenuInput::Back && self.menu.page_index() == 2 && self.display != 0 {
            self.display = 0;
            self.menu.select(0);
            cx.notify();
            return;
        }
        if self.menu.focus() == Focus::Dialog {
            if self.dialog_answer.is_none() {
                match key {
                    MenuInput::Confirm => self.dialog_answer = Some(self.menu.affirmative()),
                    MenuInput::Back if !self.menu.affirmative() => self.dialog_answer = Some(false),
                    other => {
                        self.menu.input(other);
                    }
                }
            }
            cx.notify();
            return;
        }
        let action = self.menu.input(key);
        self.apply(action, window, cx);
    }

    fn begin_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.closing_at = Some(Instant::now());
        window
            .spawn(cx, async |cx| {
                cx.background_executor()
                    .timer(MenuFrame::CLOSE_DURATION)
                    .await;
                let _ = cx.update(|w, _| w.remove_window());
            })
            .detach();
        cx.notify();
    }

    fn apply(&mut self, action: Option<MenuAction>, window: &mut Window, cx: &mut Context<Self>) {
        match action {
            Some(MenuAction::Close) => return self.begin_close(window, cx),
            Some(MenuAction::Activate { page, path }) => match (page, path.as_slice()) {
                (0, &[i]) => self.mods[i].enabled = !self.mods[i].enabled,
                (1, &[i]) => self.mods.iter_mut().for_each(|m| m.enabled = i == 0),
                (2, &[i]) => self.display = i,
                (4, &[_]) => self.viewer_action(yp_nier_ui::ViewerAction::Open, cx),
                _ => {}
            },
            None => {}
        }
        if self.menu.focus() == Focus::Rows {
            self.scroll
                .scroll_to_item(self.menu.row_index(), ScrollStrategy::Nearest);
        }
        cx.notify();
    }

    fn tabs(&self, owner: Entity<Self>, scale: f32) -> impl IntoElement + use<> {
        let page = self.menu.page_index();
        let tabs = self.menu.pages().iter().enumerate().map(|(i, p)| {
            let hover = owner.clone();
            let press = owner.clone();
            let tab = MenuTab::new(("page", i), p.label.clone())
                .scale(scale)
                .selected(i == page)
                .expanded(i == page && self.menu.focus() != Focus::Tabs)
                .when_some(p.icon.clone(), |tab, icon| tab.icon(icon))
                .on_hover(move |over, _, cx| {
                    if over {
                        hover.update(cx, |view, cx| {
                            if view.menu.focus() == Focus::Tabs && view.menu.page_index() != i {
                                view.menu.tab(i);
                                cx.notify();
                            }
                        });
                    }
                })
                .on_click(move |_, w, cx| {
                    press.update(cx, |view, cx| {
                        if view.menu.focus() != Focus::Dialog
                            && (view.menu.focus() == Focus::Tabs || view.menu.page_index() != i)
                        {
                            view.menu.tab(i);
                            view.input(MenuInput::Confirm, w, cx);
                        }
                    })
                });
            div()
                .absolute()
                .left(px(211.5 * i as f32 * scale))
                .child(tab)
        });
        div().relative().size_full().children(tabs)
    }

    fn rows(&self, owner: Entity<Self>, scale: f32) -> MenuList {
        let page = self.menu.page_index();
        let count = self.menu.entries().len();
        MenuList::new(("entries", page), count, move |i, _, cx| {
            let view = owner.read(cx);
            let e = &view.menu.entries()[i];
            let hover = owner.clone();
            let click = owner.clone();
            if page == 4 {
                return yp_nier_ui::MenuItemRow::new(("unit", i), e.label.clone())
                    .scale(scale)
                    .icon("panel")
                    .selected(i == view.menu.row_index())
                    .on_hover(move |over, _, cx| {
                        if over {
                            hover.update(cx, |v, cx| {
                                if !v.model.expanded
                                    && v.menu.focus() == Focus::Rows
                                    && v.menu.row_index() != i
                                {
                                    v.menu.select(i);
                                    cx.notify();
                                }
                            });
                        }
                    })
                    .on_click(move |_, w, cx| {
                        click.update(cx, |v, cx| {
                            v.menu.select(i);
                            v.input(MenuInput::Confirm, w, cx);
                        })
                    })
                    .into_any_element();
            }
            MenuRow::new(("entry", i), e.label.clone())
                .scale(scale)
                .enabled(e.enabled)
                .selected(view.menu.focus() != Focus::Tabs && view.menu.row_index() == i)
                .on_hover(move |over, _, cx| {
                    if over {
                        hover.update(cx, |v, cx| {
                            if v.menu.focus() != Focus::Dialog
                                && (v.menu.row_index() != i || v.menu.focus() != Focus::Rows)
                            {
                                v.menu.select(i);
                                cx.notify();
                            }
                        });
                    }
                })
                .on_click(move |_, w, cx| {
                    click.update(cx, |v, cx| {
                        if v.menu.focus() != Focus::Dialog {
                            v.menu.select(i);
                            v.input(MenuInput::Confirm, w, cx);
                        }
                    })
                })
                .into_any_element()
        })
        .item_layout(page == 4)
        .scale(scale)
        .track_scroll(self.scroll.clone())
    }

    fn detail(&self, owner: Entity<Self>, scale: f32) -> impl IntoElement + use<> {
        let i = self.menu.row_index();
        let entry = self
            .menu
            .selected_entry()
            .expect("each preview page has entries");
        if self.menu.page_index() == 3 {
            return self.install_detail(owner, scale).into_any_element();
        }
        if self.menu.page_index() == 2 && i == 0 && self.display == 0 {
            return self.display_controls(owner, scale).into_any_element();
        }
        let count = self.mods.iter().filter(|m| m.enabled).count();
        let detail = match self.menu.page_index() {
            0 => {
                let m = &self.mods[i];
                vec![
                    ("Category", m.category.to_owned()),
                    (
                        "Status",
                        if m.enabled { "Enabled" } else { "Disabled" }.to_owned(),
                    ),
                    ("Load order", format!("{:02}", i + 1)),
                ]
            }
            1 => vec![("Enabled", format!("{count} / {}", self.mods.len()))],
            _ => vec![(
                "Current display",
                self.menu.entries()[self.display].label.to_string(),
            )],
        };
        let fields = detail.into_iter().map(|(label, value)| {
            div()
                .flex()
                .items_center()
                .child(
                    div()
                        .w(px(230. * scale))
                        .child(GameText::new(label).scale(0.72 * scale)),
                )
                .child(GameText::new(value).reveal(false).scale(0.8 * scale))
        });
        let panel = MenuPanel::new(entry.label.clone())
            .scale(scale)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12. * scale))
                    .children(fields),
            )
            .child(
                div()
                    .mt(px(32. * scale))
                    .child(GameText::new(entry.help.clone()).scale(0.72 * scale)),
            );
        match self.menu.page_index() {
            0 => return self.mod_controls(panel, owner, i, scale).into_any_element(),
            1 => {
                return self
                    .profile_controls(panel, owner, scale)
                    .into_any_element();
            }
            _ => {}
        }
        let sample = self.mods.iter().take(3).map(|item| {
            div()
                .flex()
                .justify_between()
                .child(GameText::new(item.name).scale(0.8 * scale))
                .child(
                    GameText::new(if item.enabled { "Enabled" } else { "Disabled" })
                        .scale(0.72 * scale),
                )
        });
        let panel = panel.child(
            div()
                .mt(px(36. * scale))
                .px(px(28. * scale))
                .flex()
                .flex_col()
                .gap(px(14. * scale))
                .children(sample),
        );
        let layer = EffectLayer::new(panel).coordinates(945., 480.);
        match self.display {
            1 => layer.lens(Lens::eye_loupe()),
            2 => layer.grid_noise(GridNoise::default()),
            3 => layer.hacking_static("display-interference", 1),
            4 => layer.lens(Lens::cathode_ray_tube()),
            5 => layer.lens(Lens::chromatic()),
            _ => layer,
        }
        .into_any_element()
    }

    fn dialog(&self, owner: Entity<Self>, scale: f32) -> ConfirmDialog {
        let prompt = self
            .menu
            .selected_entry()
            .and_then(|e| e.confirmation.clone())
            .unwrap_or_default();
        let select = owner.clone();
        ConfirmDialog::new(prompt, move |yes, w, cx| {
            owner.update(cx, |v, cx| {
                if v.menu.focus() == Focus::Dialog {
                    v.dialog_answer = None;
                    v.menu.choose_answer(yes);
                    let action = v.menu.input(MenuInput::Confirm);
                    v.apply(action, w, cx);
                }
            })
        })
        .affirmative(self.menu.affirmative())
        .dismiss(self.dialog_answer)
        .scale(scale)
        .on_select(move |yes, _, cx| {
            select.update(cx, |v, cx| {
                if v.menu.affirmative() != yes {
                    v.menu.choose_answer(yes);
                    cx.notify();
                }
            })
        })
    }
}

impl Render for ModManager {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let tabs = owner.clone();
        let rows = owner.clone();
        let detail = owner.clone();
        let viewer = owner.clone();
        let unit_data = self.menu.page_index() == 4;
        let focus = self.menu.focus();
        let back = owner.clone();
        let confirm = owner.clone();
        let previewing = self.menu.page_index() == 2 && self.display != 0;
        let description = if unit_data {
            "Select unit to view."
        } else if previewing {
            "Preview active. Select Back to restore the menu display."
        } else {
            self.menu.description()
        };
        let help = HelpBar::new(SharedString::from(description.to_owned()))
            .when(unit_data, |bar| {
                bar.prompts(vec![
                    yp_nier_ui::ButtonPrompt::new("up", "Select").with_key("down"),
                    yp_nier_ui::ButtonPrompt::new("confirm", "Model Viewer"),
                    yp_nier_ui::ButtonPrompt::new("back", "Back"),
                ])
            })
            .on_back(move |w, cx| back.update(cx, |v, cx| v.input(MenuInput::Back, w, cx)))
            .on_confirm(move |w, cx| {
                confirm.update(cx, |v, cx| v.input(MenuInput::Confirm, w, cx))
            });
        let frame = MenuFrame::new("manager-frame", self.menu.title().to_owned())
            .opened_at(self.opened_at)
            .when_some(self.closing_at, |frame, start| frame.closing_at(start))
            .header(move |s, cx| tabs.read(cx).tabs(tabs.clone(), s))
            .list(move |s, cx| rows.read(cx).rows(rows.clone(), s))
            .when(!unit_data, |frame| {
                frame.detail(move |s, cx| detail.read(cx).detail(detail.clone(), s))
            })
            .when(unit_data, |frame| {
                frame
                    .subtitle("Unit Data")
                    .foreground(move |s, cx| viewer.read(cx).model_view(viewer.clone(), s))
            })
            .footer(move |_, _| {
                if focus == Focus::Rows {
                    help.vertical()
                } else {
                    help
                }
            })
            .when(focus == Focus::Dialog, |frame| {
                frame.overlay(move |s, cx| owner.read(cx).dialog(owner.clone(), s))
            });
        div()
            .size_full()
            .track_focus(&self.focus)
            .when(self.closing_at.is_some(), |root| {
                root.capture_key_down(|_, w, cx| {
                    w.remove_window();
                    cx.stop_propagation();
                })
                .capture_any_mouse_down(|_, w, cx| {
                    w.remove_window();
                    cx.stop_propagation();
                })
            })
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, w, cx| {
                if view.name_editor.read(cx).focus_handle(cx).is_focused(w) {
                    return;
                }
                let key = match event.keystroke.key.as_str() {
                    "left" | "a" => MenuInput::Left,
                    "right" | "d" => MenuInput::Right,
                    "up" | "w" => MenuInput::Up,
                    "down" | "s" => MenuInput::Down,
                    "enter" => MenuInput::Confirm,
                    "escape" => MenuInput::Back,
                    _ => return,
                };
                view.input(key, w, cx);
                cx.stop_propagation();
            }))
            .child(frame)
    }
}
