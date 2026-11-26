use super::*;
use gpui::{ExternalPaths, canvas};
use yp_ui::{ButtonStyle, ContextMenuEntry};

const SPLIT_WIDTH: Pixels = px(740.);

impl ModsView {
    fn inventory_key(
        &mut self,
        key: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let active = self.selected_id.as_deref();
        let mods: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| match row {
                ListRow::Mod { id, .. } => Some((index, id)),
                ListRow::Header { .. } => None,
            })
            .collect();
        if mods.is_empty() {
            return;
        }
        let current = mods
            .iter()
            .position(|(index, id)| {
                Some(*index) == self.inventory_index && Some(id.as_str()) == active
            })
            .or_else(|| mods.iter().position(|(_, id)| Some(id.as_str()) == active));
        let target = match key.keystroke.key.as_str() {
            "home" => 0,
            "end" => mods.len() - 1,
            "up" => current.map_or(mods.len() - 1, |i| i.saturating_sub(1)),
            "down" => current.map_or(0, |i| (i + 1).min(mods.len() - 1)),
            "enter" | "space" if self.inventory_focus.is_focused(window) => {
                let id = mods[current.unwrap_or(0)].1.clone();
                self.select(Some(id), cx);
                self.compact_detail = true;
                if self.compact(window, cx) {
                    self.compact_detail_focus.focus(window, cx);
                }
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
                return;
            }
            _ => return,
        };
        let (index, id) = mods[target];
        self.select(Some(id.clone()), cx);
        self.compact_detail = false;
        self.inventory_index = Some(index);
        match key.keystroke.key.as_str() {
            "end" if index + 1 == self.rows.len() => self.list.scroll_to_end(),
            "home" => self.list.scroll_to(gpui::ListOffset::default()),
            _ if self.list.bounds_for_item(index).is_none() => {
                self.list.scroll_to(gpui::ListOffset {
                    item_ix: index,
                    offset_in_item: px(0.),
                })
            }
            _ => self.list.scroll_to_reveal_item(index),
        }
        self.inventory_focus.focus(window, cx);
        window.prevent_default();
        cx.stop_propagation();
        cx.notify();
    }

    fn compact(&self, window: &Window, cx: &App) -> bool {
        self.library_width.unwrap_or(window.viewport_size().width)
            < SPLIT_WIDTH * yp_ui::theme::MenuTextSize::factor(cx)
    }

    pub(super) fn library_row(
        &self,
        index: usize,
        mut row: RowModel,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(active) = self.inventory_index
            && active != index
            && matches!(self.rows.get(active), Some(ListRow::Mod { id, .. }) if *id == row.id)
        {
            row.selected = false;
        }
        let colors = &cx.theme().colors;
        let strings = cx.t();
        let bg = if row.selected {
            colors.accent_primary
        } else {
            colors.transparent
        };
        let fg = if row.selected {
            Color::OnAccent
        } else {
            Color::Default
        };
        let name = row.name.clone();
        let id = row.id;
        let group = row.group.map_or(0, |kind| kind as usize + 1);
        let label = Label::new(row.name)
            .size(LabelSize::Small)
            .color(fg)
            .when(row.disabled, |l| l.strikethrough())
            .when(row.selected, |l| l.semibold());
        let status = if row.disabled {
            strings.mod_disabled()
        } else {
            strings.menu_enabled()
        };
        let selector = id.clone();
        h_flex()
            .id(SharedString::from(format!("library-row-{group}-{id}")))
            .debug_selector(move || format!("library-item-{selector}"))
            .role(gpui::Role::ListBoxOption)
            .aria_selected(row.selected)
            .when(row.selected, |el| el.aria_active_descendant())
            .aria_label(format!("{name}, {status}"))
            .keyboard_activate(cx)
            .w_full()
            .min_w_0()
            .min_h(px(36.))
            .gap(px(9.))
            .px(px(10.))
            .py(px(5.))
            .bg(bg)
            .cursor_pointer()
            .when(!row.selected, |el| {
                let hover = colors.surface_medium;
                el.hover(move |s| s.bg(hover))
            })
            .child(
                div()
                    .flex_none()
                    .rounded_full()
                    .bg(colors.background_card)
                    .child(
                        Icon::new(if row.disabled {
                            IconName::RemoveCircleOutline
                        } else {
                            IconName::CheckCircle
                        })
                        .size(IconSize::Custom(px(18.)))
                        .color(if row.disabled {
                            Color::Secondary
                        } else {
                            Color::Success
                        }),
                    ),
            )
            .child(div().flex_1().min_w_0().child(label))
            .when(row.has_warnings, |el| {
                el.child(
                    Icon::new(IconName::WarningAmberRounded)
                        .size(IconSize::Small)
                        .color(fg),
                )
            })
            .when(!row.default_files.is_empty(), |el| {
                el.child(Icon::new(IconName::Star).size(IconSize::Small).color(fg))
            })
            .child(
                Icon::new(IconName::ChevronRight)
                    .size(IconSize::Small)
                    .color(fg),
            )
            .tooltip(Tooltip::text(format!("{name} · {status}")))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select(Some(id.clone()), cx);
                this.inventory_index = Some(index);
                if this.compact(window, cx) {
                    this.compact_detail_focus.focus(window, cx);
                }
                this.compact_detail = true;
                cx.notify();
            }))
            .into_any_element()
    }

    fn install_menu(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let s = cx.t();
        let view = cx.weak_entity();
        PopoverMenu::<ContextMenu>::new("library-install")
            .accessible_label(s.menu_install_mod())
            .anchor(Anchor::TopRight)
            .attach(Anchor::BottomRight)
            .trigger(
                Button::new("library-install-label", s.menu_install_mod())
                    .style(ButtonStyle::Filled)
                    .start_icon(IconName::Add)
                    .end_icon(IconName::KeyboardArrowDown),
            )
            .menu(move |w, cx| {
                let blocked = ControllerMode::is_active(cx);
                let view = view.clone();
                Some(ContextMenu::build(w, cx, move |mut m, _, _| {
                    for (title, files, mode) in [
                        (s.menu_install_archive(), true, ModInstallMode::Guided),
                        (s.menu_install_folder(), false, ModInstallMode::Guided),
                        (s.mod_bulk_install(), false, ModInstallMode::Bulk),
                    ] {
                        let target = view.clone();
                        m = m.item(ContextMenuEntry::new(title).disabled(blocked).handler(
                            move |w, cx| {
                                target.update(cx, |v, cx| v.browse(files, mode, w, cx)).ok();
                            },
                        ));
                    }
                    m = m.item(
                        ContextMenuEntry::new(s.mod_loose_install())
                            .disabled(blocked)
                            .handler(move |w, cx| {
                                view.update(cx, |v, cx| v.browse_loose_folder(w, cx)).ok();
                            }),
                    );
                    m
                }))
            })
    }

    pub(super) fn library(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let s = cx.t();
        let colors = cx.theme().colors.clone();
        let rows = self.sync_rows(cx);
        let mods = self.stores.mods.read(cx);
        let (count, loading, failed) =
            (mods.mods().len(), mods.is_loading(), mods.error().is_some());
        let compact = self.compact(window, cx);
        let filter = self.filter_text(cx);
        let visible = |id: &str| {
            self.rows
                .iter()
                .any(|r| matches!(r,ListRow::Mod{id: visible,..} if visible == id))
        };
        let needs_selection = match self.selected_id.as_deref() {
            Some(id) => !filter.is_empty() && !visible(id) && mods.find(id).is_some(),
            None => self.cleared_selection_filter.as_deref() != Some(filter.as_str()),
        };
        if !compact
            && self.pending_selection.is_none()
            && needs_selection
            && let Some(id) = self.rows.iter().find_map(|r| match r {
                ListRow::Mod { id, .. } => Some(id.clone()),
                _ => None,
            })
        {
            self.select(Some(id), cx);
            self.compact_detail = false;
        }

        let view = cx.weak_entity();
        let tracker = canvas(
            move |bounds, window, cx| {
                let width = bounds.size.width;
                if view
                    .read_with(cx, |v, _| v.library_width != Some(width))
                    .unwrap_or(false)
                {
                    window.defer(cx, move |_, cx| {
                        view.update(cx, |v, cx| {
                            v.library_width = Some(width);
                            cx.notify();
                        })
                        .ok();
                    });
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0();
        let header = PageHeader::new(s.tab_mods()).info_tooltip(s.mod_intro_body());
        let mod_directory = self
            .stores
            .session
            .read(cx)
            .game()
            .map(|game| game.mods_dir());
        let back = compact && self.compact_detail && self.selected_id.is_some();
        let header = if compact {
            h_flex()
                .w_full()
                .min_w_0()
                .flex_wrap()
                .gap(px(6.))
                .py(px(4.))
                .child(
                    div()
                        .id("compact-mod-title")
                        .role(gpui::Role::Heading)
                        .aria_level(1)
                        .aria_label(s.tab_mods())
                        .child(Label::new(s.tab_mods()).semibold()),
                )
                .child(ProfileSelector::new())
                .child(div().flex_1())
                .child(self.install_menu(cx))
                .child(self.render_help_menu(cx))
                .into_any_element()
        } else {
            header
                .actions(ProfileSelector::new())
                .actions(self.install_menu(cx))
                .actions(self.render_help_menu(cx))
                .into_any_element()
        };
        let inventory = v_flex()
            .id("mod-inventory")
            .size_full()
            .min_w_0()
            .min_h_0()
            .gap(px(6.))
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .justify_between()
                    .gap(px(8.))
                    .border_b_2()
                    .border_color(colors.accent_primary)
                    .pb(px(5.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Label::new(s.menu_installed_mods()).semibold()),
                    )
                    .child(
                        Label::new(if filter.is_empty() {
                            count.to_string()
                        } else {
                            s.menu_filtered_mod_count(rows as i64, count as i64)
                        })
                        .size(LabelSize::Small)
                        .color(Color::Secondary),
                    )
                    .when_some(mod_directory, |el, path| {
                        el.child(
                            IconButton::new("library-open-directory", IconName::FolderOpen)
                                .accessible_label(s.menu_open_mods_directory())
                                .tooltip(Tooltip::text(s.menu_open_mods_directory()))
                                .disabled(ControllerMode::is_active(cx))
                                .on_click(move |_, _, cx| reveal(path.clone(), false, cx)),
                        )
                    }),
            )
            .when(count > 0 || !filter.is_empty(), |el| {
                el.child(
                    Input::new(&self.filter)
                        .prefix(Icon::new(IconName::Search).size(IconSize::Small))
                        .cleanable(true),
                )
            })
            .child(
                v_flex()
                    .id("library-selection")
                    .role(gpui::Role::ListBox)
                    .aria_label(s.menu_installed_mods())
                    .aria_keyshortcuts("ArrowUp ArrowDown Home End Enter")
                    .track_focus(&self.inventory_focus)
                    .keyboard_focus(cx)
                    .on_key_down(cx.listener(Self::inventory_key))
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(if rows == 0 && !filter.is_empty() {
                        EmptyState::new(s.docs_no_results())
                            .compact()
                            .into_any_element()
                    } else if count == 0 && !loading && !failed {
                        EmptyState::new(s.mod_list_empty())
                            .icon(IconName::Inventory2Outlined)
                            .description(s.mod_profile_empty_hint())
                            .compact()
                            .into_any_element()
                    } else {
                        self.render_list(rows, loading, failed, cx)
                    })
                    .tour_anchor(ANCHOR_LIST),
            );
        let show_detail = back || !compact && count > 0 && (filter.is_empty() || rows > 0);
        let body = if show_detail {
            let detail = fill(self.render_detail(window, cx)).tour_anchor(ANCHOR_DETAIL);
            if back {
                v_flex()
                    .size_full()
                    .min_h_0()
                    .items_start()
                    .gap(px(4.))
                    .child(
                        div().debug_selector(|| "library-back".to_owned()).child(
                            Button::new("library-back", s.menu_installed_mods())
                                .track_focus(&self.compact_detail_focus)
                                .style(ButtonStyle::Link)
                                .start_icon(IconName::ArrowBack)
                                .on_click(cx.listener(|v, _, window, cx| {
                                    v.compact_detail = false;
                                    v.inventory_focus.focus(window, cx);
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(detail.w_full())
                    .into_any_element()
            } else {
                TwoColumn::new(inventory, v_flex().size_full().child(detail))
                    .id("library-panes")
                    .ratio(0.84, 1.35)
                    .breakpoint(Pixels::ZERO)
                    .fill_height(true)
                    .into_any_element()
            }
        } else {
            inventory.into_any_element()
        };
        let target = cx.weak_entity();
        v_flex()
            .id("mods-workspace")
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .child(tracker)
            .when(!back, |el| el.child(header))
            .child(fill(body).w_full())
            .when(!compact || self.tutorial.is_some(), |el| {
                el.child(
                    h_flex()
                        .w_full()
                        .flex_none()
                        .gap(px(8.))
                        .py(px(5.))
                        .tour_anchor(ANCHOR_DROP_ZONE)
                        .child(
                            Icon::new(IconName::DownloadOutlined)
                                .size(IconSize::Small)
                                .color(Color::Secondary),
                        )
                        .child(
                            div().flex_1().min_w_0().child(
                                Label::new(s.menu_drop_hint())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Secondary),
                            ),
                        ),
                )
            })
            .child(yp_ui::nier::workspace_drop_highlight(
                colors.accent_primary, colors.background_primary, s.drop_to_install(),
            ))
            .on_drop(cx.listener(|v, paths: &ExternalPaths, w, cx| {
                v.install_paths(paths.paths().to_vec(), ModInstallMode::Guided, w, cx)
            }))
            .children(self.tutorial.map(|kind| {
                TourOverlay::new(tour_steps(kind, s))
                    .id(SharedString::from(format!("mods-tutorial-{}", kind.key())))
                    .on_finish(move |_, cx| {
                        target.update(cx, |v, cx| v.close_tutorial(cx)).ok();
                    })
            }))
            .into_any_element()
    }
}
