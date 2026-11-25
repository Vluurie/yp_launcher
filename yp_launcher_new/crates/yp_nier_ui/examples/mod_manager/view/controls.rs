use super::ModManager;
use gpui::{App, Context, Entity, IntoElement, PathPromptOptions, Window, div, prelude::*, px};
use std::path::PathBuf;
use yp_nier_ui::{
    GameText, LoadingIcon, MenuChoice, MenuDropZone, MenuDropdown, MenuPanel, MenuProgress,
    MenuSlider, MenuToggle,
};

impl ModManager {
    pub(super) fn display_controls(
        &self,
        owner: Entity<Self>,
        scale: f32,
    ) -> impl IntoElement + use<> {
        let screen = owner.clone();
        div()
            .flex()
            .flex_col()
            .child(
                MenuDropdown::new(
                    "screen-resolution",
                    "Resolution",
                    super::catalog::RESOLUTIONS.map(MenuChoice::new),
                    self.resolution,
                    move |resolution, _, cx| {
                        screen.update(cx, |view, cx| {
                            view.resolution = resolution;
                            cx.notify();
                        })
                    },
                )
                .scrolling()
                .scale(scale),
            )
            .child(
                yp_nier_ui::SettingRow::new("monitor", "Select Monitor")
                    .value("System default")
                    .enabled(false)
                    .scale(scale),
            )
            .child(
                MenuSlider::new(
                    "rotation-speed",
                    "Horizontal Rotation Speed",
                    self.rotation_speed,
                    20,
                    move |speed, _, cx| {
                        owner.update(cx, |view, cx| {
                            view.rotation_speed = speed;
                            cx.notify();
                        })
                    },
                )
                .default_mark(8)
                .scale(scale),
            )
    }

    pub(super) fn mod_controls(
        &self,
        panel: MenuPanel,
        owner: Entity<Self>,
        i: usize,
        s: f32,
    ) -> MenuPanel {
        let toggle = owner.clone();
        let priority = owner.clone();
        let active = self.mods[i].enabled;
        let default = super::catalog::catalog()[i].enabled;
        let changed = active != default || self.priorities[i] != 5;
        let actions = yp_nier_ui::MenuActionMenu::new(
            ("mod-actions", i),
            [
                MenuChoice::new(if active { "Disable" } else { "Enable" }),
                if changed {
                    MenuChoice::new("Restore default")
                } else {
                    MenuChoice::new("Restore default").disabled()
                },
            ],
            move |open| {
                yp_nier_ui::MenuItemRow::new("action-row", "Actions")
                    .icon("archive")
                    .quantity(1 + u32::from(changed))
                    .scale(s)
                    .when(open, |row| row.selected(true).confirmed(true))
            },
            move |action, _, cx| {
                owner.update(cx, |v, cx| {
                    v.mods[i].enabled = if action == 0 { !active } else { default };
                    if action == 1 {
                        v.priorities[i] = 5;
                    }
                    cx.notify();
                })
            },
        )
        .scale(s);
        panel
            .child(
                div().mt(px(16. * s)).child(
                    MenuToggle::new(
                        ("mod-enabled", i),
                        "Enabled",
                        self.mods[i].enabled,
                        move |enabled, _, cx| {
                            toggle.update(cx, |v, cx| {
                                v.mods[i].enabled = enabled;
                                cx.notify();
                            })
                        },
                    )
                    .scale(s),
                ),
            )
            .child(
                MenuSlider::new(
                    ("priority", i),
                    "Priority",
                    self.priorities[i],
                    10,
                    move |value, _, cx| {
                        priority.update(cx, |v, cx| {
                            v.priorities[i] = value;
                            cx.notify();
                        })
                    },
                )
                .enabled(self.mods[i].enabled)
                .scale(s),
            )
            .child(actions)
    }
    pub(super) fn profile_controls(
        &self,
        panel: MenuPanel,
        owner: Entity<Self>,
        s: f32,
    ) -> MenuPanel {
        panel
            .child(
                div().mt(px(16. * s)).child(
                    MenuDropdown::new(
                        "profile-selector",
                        "Profile",
                        [
                            MenuChoice::new(self.profile_name.clone()),
                            MenuChoice::new("All mods"),
                            MenuChoice::new("Unmodified"),
                            if self.previous_profile.is_some() {
                                MenuChoice::new("Previous profile")
                            } else {
                                MenuChoice::new("Previous profile").disabled()
                            },
                        ],
                        self.profile,
                        move |choice, _, cx| {
                            owner.update(cx, |v, cx| {
                                let choice = if choice == 3 {
                                    v.previous_profile.unwrap_or(v.profile)
                                } else {
                                    choice
                                };
                                if choice != v.profile {
                                    v.previous_profile = Some(v.profile);
                                }
                                v.profile = choice;
                                let defaults = super::catalog::catalog();
                                for (i, m) in v.mods.iter_mut().enumerate() {
                                    m.enabled = match choice {
                                        0 => defaults[i].enabled,
                                        1 => true,
                                        _ => false,
                                    };
                                }
                                cx.notify();
                            })
                        },
                    )
                    .scale(s),
                ),
            )
            .child(
                div()
                    .mt(px(8. * s))
                    .flex()
                    .items_center()
                    .gap(px(20. * s))
                    .child(GameText::new("Profile name").scale(0.8 * s))
                    .child(
                        yp_nier_ui::MenuTextField::new("profile-name", self.name_editor.clone())
                            .placeholder("Profile name")
                            .scale(s),
                    ),
            )
    }
    fn receive(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.pending += 1;
        cx.notify();
        let job = cx.background_executor().spawn(async move {
            paths
                .into_iter()
                .map(|path| {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    match path.metadata() {
                        Ok(m) if m.is_dir() => format!("{name}  ·  Folder"),
                        Ok(m) => format!("{name}  ·  {} bytes", m.len()),
                        Err(e) => format!("{name}  ·  {e}"),
                    }
                })
                .collect::<Vec<_>>()
        });
        cx.spawn(async move |weak, cx| {
            let names = job.await;
            let _ = weak.update(cx, |v, cx| {
                v.packages.extend(names);
                v.pending -= 1;
                cx.notify();
            });
        })
        .detach();
    }
    fn browse(owner: &Entity<Self>, files: bool, w: &mut Window, cx: &mut App) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files,
            directories: !files,
            multiple: true,
            prompt: None,
        });
        owner.update(cx, |_, cx| {
            cx.spawn_in(w, async move |weak, cx| {
                if let Ok(Ok(Some(paths))) = paths.await {
                    let _ = weak.update(cx, |v, cx| v.receive(paths, cx));
                }
            })
            .detach();
        });
    }
    pub(super) fn install_detail(&self, owner: Entity<Self>, s: f32) -> MenuPanel {
        let i = self.menu.row_index();
        if i != 0 {
            let icon = LoadingIcon::new(("indicator", i)).scale(s);
            return MenuPanel::new(if i == 1 { "Loading" } else { "Saving" })
                .scale(s)
                .child(
                    div()
                        .flex()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .child(if i == 2 { icon.saving() } else { icon }),
                );
        }
        let files = owner.clone();
        let folder = owner.clone();
        let drop = MenuDropZone::new(
            "packages",
            "Drop a mod archive or folder",
            move |paths, _, cx| {
                owner.update(cx, |v, cx| v.receive(paths.to_vec(), cx));
            },
        )
        .on_browse_files(move |w, cx| Self::browse(&files, true, w, cx))
        .on_browse_folder(move |w, cx| Self::browse(&folder, false, w, cx))
        .scale(s);
        let mut panel = MenuPanel::new("Packages").scale(s).child(drop);
        if self.pending != 0 {
            panel = panel.child(MenuProgress::new("checking", "Reading package details").scale(s));
        }
        if !self.packages.is_empty() {
            panel =
                panel.child(
                    div()
                        .id("received-packages")
                        .min_h_0()
                        .flex_1()
                        .overflow_y_scroll()
                        .children(self.packages.iter().rev().take(5).map(|name| {
                            GameText::new(name.clone()).wrap(912. * s).scale(0.72 * s)
                        })),
                );
        }
        panel
    }
}
