use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    Anchor, ListAlignment, ListState, PathPromptOptions, ScrollHandle, Subscription, Task,
};
use yp_mods::camera::{CameraField, CameraFile, CameraSettings};
use yp_mods::grouping::ModGroupKind;
use yp_mods::{InstalledMod, ModKind};
use yp_state::prelude::*;
use yp_state::{ModInstallMode, ModInstallRequest, ModsEvent, NavigationEvent, Stores};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, Card, CardPadding, ChipSize, ChipTone, ContextMenu, Divider, DividerColor,
    DropZone, EmptyState, I18n, Input, InputEvent, InputState, ListHeader, ListItemSpacing,
    LoadingState, Msg, PageHeader, PopoverMenu, Tooltip, TourAnchorExt,
    TourOverlay, TwoColumn, VirtualList,
};

use crate::labels::{group_icon, group_label};
use crate::layout::{fill, shrinkable};
use crate::mods::dialogs::{NamingRequest, confirm_uninstall, prompt_mod_name};
use crate::mods::install::{self, Busy};
use crate::mods::list::{
    ListRow, filter_and_sort, has_live_warnings, last_segment, list_rows, mod_rel_path,
    should_show_search,
};
use crate::mods::profile::ProfileSelector;
use crate::mods::tutorial::{
    ANCHOR_DETAIL, ANCHOR_DROP_ZONE, ANCHOR_LIST, TutorialKind, tour_steps,
};
use crate::notify::{reveal, toast};

mod library;

const CAMERA_SAVE_DELAY: Duration = Duration::from_millis(100);

pub(crate) struct CameraEditor {
    pub file: CameraFile,
    pub settings: CameraSettings,
    dirty: bool,
    save: Option<Task<()>>,
}

struct RowModel {
    id: String,
    group: Option<ModGroupKind>,
    name: String,
    version: Option<String>,
    selected: bool,
    disabled: bool,
    shows_data_chip: bool,
    has_warnings: bool,
    default_files: Vec<String>,
}

enum ListEntry {
    Header {
        kind: ModGroupKind,
        count: usize,
        collapsed: bool,
    },
    Mod(RowModel),
}

pub struct ModsView {
    pub(crate) stores: Stores,
    pub(crate) selected_id: Option<String>,
    pending_selection: Option<String>,
    compact_detail: bool,
    inventory_focus: FocusHandle,
    inventory_index: Option<usize>,
    compact_detail_focus: FocusHandle,
    library_width: Option<Pixels>,
    filter: Entity<InputState>,
    collapsed: HashSet<ModGroupKind>,
    list: ListState,
    rows: Rc<Vec<ListRow>>,
    pub(crate) detail_scroll: ScrollHandle,
    tutorial: Option<TutorialKind>,
    pub(crate) previews: Vec<(yp_state::inspector::ArchivePaths, AnyView)>,
    pub(crate) camera: Option<CameraEditor>,
    camera_load: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl ModsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let filter =
            cx.new(|cx| InputState::new(window, cx).placeholder(cx.t().mod_search_placeholder()));
        let mut subscriptions = vec![
            cx.subscribe(&filter, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe(&stores.mods, |this, _, event: &ModsEvent, cx| {
                match event {
                    ModsEvent::Uninstalled { id } if this.selected_id.as_ref() == Some(id) => {
                        this.select(None, cx);
                    }
                    ModsEvent::Changed | ModsEvent::Installed { .. } => {
                        this.resolve_pending_selection(cx);
                        this.drop_missing_selection(cx);
                    }
                    _ => {}
                }
                cx.notify();
            }),
            cx.subscribe(
                &stores.navigation,
                |this, _, event: &NavigationEvent, cx| match event {
                    NavigationEvent::SelectionRequested(TabId::Mods)
                    | NavigationEvent::TabChanged {
                        to: TabId::Mods, ..
                    } => this.take_navigation_selection(cx),
                    _ => {}
                },
            ),
            cx.observe_global::<I18n>(|this, cx| {
                let placeholder = cx.t().mod_search_placeholder();
                this.filter
                    .update(cx, |filter, cx| filter.set_placeholder(placeholder, cx));
            }),
        ];
        subscriptions.push(cx.observe(&stores.profiles, |_, _, cx| cx.notify()));
        subscriptions.push(cx.observe(&stores.disabled_mods, |_, _, cx| cx.notify()));
        subscriptions.push(cx.observe(&stores.default_mods, |_, _, cx| cx.notify()));
        subscriptions.push(cx.observe(&stores.mod_names, |_, _, cx| cx.notify()));
        subscriptions.push(cx.observe(&stores.session, |_, _, cx| cx.notify()));
        let mut view = Self {
            stores,
            selected_id: None,
            pending_selection: None,
            compact_detail: false,
            inventory_focus: cx.focus_handle().tab_stop(true),
            inventory_index: None,
            compact_detail_focus: cx.focus_handle().tab_stop(true),
            library_width: None,
            filter,
            collapsed: HashSet::new(),
            list: ListState::new(0, ListAlignment::Top, Sizes::SELECT_MENU_MAX_HEIGHT),
            rows: Rc::default(),
            detail_scroll: ScrollHandle::new(),
            tutorial: None,
            previews: Vec::new(),
            camera: None,
            camera_load: None,
            _subscriptions: subscriptions,
        };
        view.take_navigation_selection(cx);
        view
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    pub fn filter_text(&self, cx: &App) -> String {
        self.filter.read(cx).value().to_owned()
    }

    pub fn set_filter(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let text = text.to_owned();
        self.filter
            .update(cx, |filter, cx| filter.set_value(text, window, cx));
        cx.notify();
    }

    pub fn is_group_collapsed(&self, kind: ModGroupKind) -> bool {
        self.collapsed.contains(&kind)
    }

    pub fn toggle_group(&mut self, kind: ModGroupKind, cx: &mut Context<Self>) {
        if !self.collapsed.remove(&kind) {
            self.collapsed.insert(kind);
        }
        cx.notify();
    }

    pub fn tutorial(&self) -> Option<TutorialKind> {
        self.tutorial
    }

    pub fn start_tutorial(&mut self, kind: TutorialKind, cx: &mut Context<Self>) {
        self.tutorial = Some(kind);
        cx.notify();
    }

    pub fn close_tutorial(&mut self, cx: &mut Context<Self>) {
        self.tutorial = None;
        cx.notify();
    }

    pub fn select(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        if self.selected_id == id {
            return;
        }
        self.flush_camera(cx);
        self.compact_detail = id.is_some();
        self.inventory_index = None;
        self.selected_id = id;
        self.previews.clear();
        self.camera = None;
        self.camera_load = None;
        self.detail_scroll.set_offset(gpui::point(px(0.), px(0.)));
        self.load_camera(cx);
        cx.notify();
    }

    fn take_navigation_selection(&mut self, cx: &mut Context<Self>) {
        let navigation = self.stores.navigation.clone();
        if let Some(key) =
            navigation.update(cx, |navigation, _| navigation.take_selection(TabId::Mods))
        {
            self.pending_selection = Some(key);
            self.resolve_pending_selection(cx);
        }
    }

    fn resolve_pending_selection(&mut self, cx: &mut Context<Self>) {
        let Some(wanted) = self.pending_selection.clone() else {
            return;
        };
        if self.stores.mods.read(cx).find(&wanted).is_some() {
            self.pending_selection = None;
            self.select(Some(wanted), cx);
        }
    }

    fn drop_missing_selection(&mut self, cx: &mut Context<Self>) {
        let Some(selected) = self.selected_id.clone() else {
            return;
        };
        let missing = {
            let mods = self.stores.mods.read(cx);
            !mods.is_loading() && mods.find(&selected).is_none()
        };
        if missing {
            self.select(None, cx);
        }
    }

    fn load_camera(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected_id.clone() else {
            return;
        };
        let root = match self.stores.mods.read(cx).find(&id) {
            Some(installed) if installed.kind == ModKind::Native => installed.root_path.clone(),
            _ => return,
        };
        self.camera_load = Some(cx.spawn(async move |this, cx| {
            let found = cx
                .background_executor()
                .spawn(async move { yp_mods::camera::find(&root) })
                .await;
            this.update(cx, |this, cx| {
                if this.selected_id.as_deref() != Some(id.as_str()) {
                    return;
                }
                this.camera = found.map(|file| CameraEditor {
                    settings: file.settings,
                    file,
                    dirty: false,
                    save: None,
                });
                cx.notify();
            })
            .ok();
        }));
    }

    pub(crate) fn set_camera_value(
        &mut self,
        field: CameraField,
        value: f32,
        cx: &mut Context<Self>,
    ) {
        let Some(editor) = &mut self.camera else {
            return;
        };
        let scale = 10f64.powi(i32::try_from(field.decimals()).unwrap_or(0));
        let rounded = (f64::from(value) * scale).round() / scale;
        if editor.settings.get(field) == rounded {
            return;
        }
        editor.settings.set(field, rounded);
        editor.dirty = true;
        editor.save = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(CAMERA_SAVE_DELAY).await;
            let Ok(Some((file, settings))) = this.update(cx, |this, _| {
                this.camera.as_mut().map(|editor| {
                    editor.dirty = false;
                    (editor.file.clone(), editor.settings)
                })
            }) else {
                return;
            };
            cx.background_executor()
                .spawn(async move { write_camera(&file, &settings) })
                .await;
        }));
        cx.notify();
    }

    fn flush_camera(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = self.camera.take() else {
            return;
        };
        let CameraEditor {
            file,
            settings,
            dirty,
            ..
        } = editor;
        if dirty {
            cx.background_executor()
                .spawn(async move { write_camera(&file, &settings) })
                .detach();
        }
    }

    pub fn install_paths(
        &mut self,
        paths: Vec<PathBuf>,
        mode: ModInstallMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if paths.is_empty() {
            return;
        }
        let request = ModInstallRequest {
            sources: paths,
            mode,
            texture_packs: false,
        };
        let installing = install::run(request, window, cx);
        cx.spawn(async move |this, cx| {
            let report = installing.await;
            if mode != ModInstallMode::Guided {
                return;
            }
            if let Some(id) = report.installed.last().cloned() {
                this.update(cx, |this, cx| this.select(Some(id), cx)).ok();
            }
        })
        .detach();
    }

    fn browse(
        &mut self,
        files: bool,
        mode: ModInstallMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let picking = cx.prompt_for_paths(PathPromptOptions {
            files,
            directories: !files,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = picking.await else {
                return;
            };
            this.update_in(cx, |this, window, cx| {
                this.install_paths(paths, mode, window, cx)
            })
            .ok();
        })
        .detach();
    }

    fn browse_loose_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picking = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |_, cx| {
            let Ok(Ok(Some(paths))) = picking.await else {
                return;
            };
            let Some(folder) = paths.into_iter().next() else {
                return;
            };
            if let Ok(installing) =
                cx.update(|window, cx| install::run_loose_folder(folder, window, cx))
            {
                installing.await;
            }
        })
        .detach();
    }

    pub(crate) fn set_mod_disabled(
        &mut self,
        mod_id: &str,
        disabled: bool,
        cx: &mut Context<Self>,
    ) {
        let rel_path = mod_rel_path(mod_id);
        self.stores.disabled_mods.update(cx, |store, cx| {
            store.set_disabled(&rel_path, disabled, cx);
        });
    }

    pub(crate) fn rename_mod(
        &mut self,
        mod_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(display_name) = self
            .stores
            .mods
            .read(cx)
            .find(&mod_id)
            .map(|installed| installed.display_name.clone())
        else {
            return;
        };
        let current = self
            .stores
            .mod_names
            .read(cx)
            .custom_name_of(&mod_id)
            .map(str::to_owned)
            .unwrap_or_else(|| display_name.clone());
        let title = cx.t().mod_rename_dialog_title();
        let asking = prompt_mod_name(NamingRequest::new(current).title(title), window, cx);
        let names = self.stores.mod_names.clone();
        cx.spawn(async move |_, cx| {
            let Some(name) = asking.await else {
                return;
            };
            let custom = (name != display_name).then_some(name);
            names
                .update(cx, |names, cx| names.rename(&mod_id, custom.as_deref(), cx));
        })
        .detach();
    }

    pub(crate) fn uninstall_mod(
        &mut self,
        mod_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(installed) = self.stores.mods.read(cx).find(&mod_id).cloned() else {
            return;
        };
        let asking = confirm_uninstall(&installed, window, cx);
        let mods = self.stores.mods.clone();
        cx.spawn_in(window, async move |this, cx| {
            let Some(delete_bundled) = asking.await else {
                return;
            };
            let mut busy = Busy::default();
            if let Ok(message) = cx.update(|_, cx| cx.t().mod_uninstall_busy()) {
                busy.show(message, cx);
            }
            let removing = cx.update(|_, cx| {
                mods.update(cx, |mods, cx| mods.uninstall(&mod_id, delete_bundled, cx))
            });
            let removed = match removing {
                Ok(removing) => removing.await,
                Err(_) => false,
            };
            busy.hide(cx);
            let id = mod_id.clone();
            cx.update(|_, cx| {
                if removed {
                    toast(
                        cx,
                        "mod_uninstalled",
                        Msg::ModUninstalled { id },
                        Severity::Info,
                    );
                } else {
                    toast(
                        cx,
                        "mod_uninstall_fail",
                        Msg::ModsUiUninstallFailed { id },
                        Severity::Error,
                    );
                }
            })
            .ok();
            if removed {
                this.update(cx, |this, cx| {
                    if this.selected_id.as_deref() == Some(mod_id.as_str()) {
                        this.select(None, cx);
                    }
                })
                .ok();
            }
        })
        .detach();
    }

    pub fn visible_rows(&self, cx: &App) -> Vec<ListRow> {
        let filter = self.filter_text(cx);
        let mods = self.stores.mods.read(cx);
        let names = self.stores.mod_names.read(cx);
        let shown = filter_and_sort(mods.mods(), &filter, &|id| {
            names.custom_name_of(id).map(str::to_owned)
        });
        list_rows(&shown, !filter.is_empty(), &self.collapsed)
    }

    fn entry_for(&self, row: &ListRow, cx: &App) -> Option<ListEntry> {
        match row {
            ListRow::Header {
                kind,
                count,
                collapsed,
            } => Some(ListEntry::Header {
                kind: *kind,
                count: *count,
                collapsed: *collapsed,
            }),
            ListRow::Mod { id, group } => {
                let mods = self.stores.mods.read(cx);
                let names = self.stores.mod_names.read(cx);
                let disabled = self.stores.disabled_mods.read(cx);
                let defaults = self.stores.default_mods.read(cx);
                let installed: &InstalledMod = mods.find(id)?;
                let rel_path = mod_rel_path(id);
                Some(ListEntry::Mod(RowModel {
                    group: *group,
                    name: names
                        .custom_name_of(id)
                        .unwrap_or(&installed.display_name)
                        .to_owned(),
                    version: installed
                        .manifest
                        .as_ref()
                        .and_then(|manifest| manifest.version.clone()),
                    selected: self.selected_id.as_deref() == Some(id.as_str()),
                    disabled: disabled.is_disabled(&rel_path),
                    shows_data_chip: installed.kind == ModKind::Native
                        && installed.has_data_overlay(),
                    has_warnings: has_live_warnings(installed, &|rel| disabled.is_disabled(rel)),
                    default_files: defaults
                        .entries_under(&rel_path)
                        .iter()
                        .map(|entry| last_segment(&entry.path).to_owned())
                        .collect(),
                    id: id.clone(),
                }))
            }
        }
    }

    fn sync_rows(&mut self, cx: &App) -> usize {
        let rows = self.visible_rows(cx);
        if *self.rows != rows {
            let same_shape = self.rows.len() == rows.len()
                && self.rows.iter().zip(&rows).all(|(old, new)| {
                    matches!(
                        (old, new),
                        (ListRow::Header { .. }, ListRow::Header { .. })
                            | (ListRow::Mod { .. }, ListRow::Mod { .. })
                    )
                });
            self.rows = Rc::new(rows);
            if !same_shape {
                let top = self.list.logical_scroll_top();
                self.list.reset(self.rows.len());
                self.list.scroll_to(top);
            }
        }
        self.rows.len()
    }

    fn render_index(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let rows = self.rows.clone();
        let Some(entry) = rows.get(index).and_then(|row| self.entry_for(row, cx)) else {
            return div().into_any_element();
        };
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let line = cx.theme().colors.border_light;
        let band = cx.theme().colors.surface_medium;
        match entry {
            ListEntry::Header {
                kind,
                count,
                collapsed,
            } => {
                let menu = cx.theme().id == ThemeId::NierNew;
                let header = v_flex()
                    .debug_selector(move || format!("mod-group-band-{kind:?}"))
                    .w_full()
                    .flex_none()
                    .bg(band)
                    .when(menu, |el| {
                        el.pr(px(10.))
                            .border_t_2()
                            .border_l_4()
                            .border_color(cx.theme().colors.accent_primary)
                    })
                    .when(!menu, |el| el.border_b_1().border_color(line))
                    .child(
                        ListHeader::new(("mods-group", kind as usize), group_label(kind, strings))
                            .count(count)
                            .collapsible(collapsed)
                            .start_slot(
                                Icon::new(group_icon(kind))
                                    .size(IconSize::Small)
                                    .color(Color::Accent),
                            )
                            .on_toggle(cx.listener(move |this, _, _, cx| this.toggle_group(kind, cx))),
                    );
                div()
                    .w_full()
                    .when(menu && index > 0, |el| el.pt(px(12.)))
                    .child(header)
                    .into_any_element()
            }
            ListEntry::Mod(row) => v_flex()
                .w_full()
                .flex_none()
                .border_b_1()
                .border_color(line)
                .when(cx.theme().id == ThemeId::NierNew, |el| el.pr(px(10.)))
                .child(if cx.theme().id == ThemeId::NierNew { self.library_row(index, row, cx) } else { self.render_row(row, &sizes, cx) })
                .into_any_element(),
        }
    }

    fn render_row(&self, row: RowModel, sizes: &Sizes, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let group_key = row.group.map_or(0, |group| group as usize + 1);
        let key = |part: &str| SharedString::from(format!("mod-{part}-{group_key}-{}", row.id));
        let name = Label::new(row.name.clone()).truncate();
        let name = if row.selected { name.semibold() } else { name };
        let name = if row.disabled {
            name.strikethrough().color(Color::Muted)
        } else {
            name
        };
        let toggle_id = row.id.clone();
        let disabled = row.disabled;
        let rename_id = row.id.clone();
        let hover_group = key("hover");
        let rename_group = hover_group.clone();
        let rename = h_flex()
            .flex_none()
            .when(!row.selected, |this| {
                this.invisible()
                    .group_hover(rename_group, |style| style.visible())
            })
            .child(
                IconButton::new(key("rename"), IconName::EditOutlined)
                    .controller_support(false)
                    .size(ButtonSize::Compact)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Muted)
                    .tooltip(Tooltip::text(strings.mod_rename()))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.rename_mod(rename_id.clone(), window, cx);
                    })),
            );
        let end = h_flex()
            .gap(sizes.spacing_md)
            .children(row.version.clone().map(|version| {
                Label::new(version)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
            }))
            .when(row.shows_data_chip, |this| {
                this.child(
                    Chip::new(strings.mod_data_chip())
                        .id(key("data"))
                        .tone(ChipTone::Muted)
                        .size(ChipSize::Small)
                        .tooltip_text(strings.mod_data_chip_tooltip()),
                )
            })
            .when(row.has_warnings, |this| {
                this.child(
                    Icon::new(IconName::WarningAmberRounded)
                        .size(IconSize::Small)
                        .color(Color::Warning),
                )
            })
            .when(!row.default_files.is_empty(), |this| {
                this.child(
                    Chip::new(strings.mod_default_chip())
                        .id(key("default"))
                        .tone(ChipTone::Accent)
                        .size(ChipSize::Small)
                        .tooltip_text(
                            strings.mod_default_row_tooltip(&row.default_files.join(", ")),
                        ),
                )
            })
            .child(rename)
            .child(
                div()
                    .id(key("toggle-tip"))
                    .tooltip(Tooltip::text(if disabled {
                        strings.mod_disabled_tooltip()
                    } else {
                        strings.mod_enable_tooltip()
                    }))
                    .child(Switch::new(key("toggle"), !disabled).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.set_mod_disabled(&toggle_id, !disabled, cx);
                        },
                    ))),
            );
        let select_id = row.id.clone();
        ListItem::new(key("row"))
            .group_name(hover_group)
            .selected(row.selected)
            .spacing(ListItemSpacing::Dense)
            .child(shrinkable(name))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select(Some(select_id.clone()), cx);
            }))
            .end_slot(end)
            .into_any_element()
    }

    fn render_list(
        &self,
        row_count: usize,
        loading: bool,
        load_failed: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        if row_count == 0 {
            if loading {
                return fill(LoadingState::new()).into_any_element();
            }
            if load_failed {
                return fill(
                    EmptyState::new(strings.mods_ui_mods_load_failed())
                        .icon(IconName::ErrorOutline),
                )
                .into_any_element();
            }
            return fill(
                EmptyState::new(strings.mod_list_empty())
                    .icon(IconName::ExtensionOutlined)
                    .description(strings.mod_list_empty_hint()),
            )
            .into_any_element();
        }
        fill(VirtualList::list(
            "mods-list",
            &self.list,
            cx.processor(|this, index: usize, window, cx| this.render_index(index, window, cx)),
        ))
        .into_any_element()
    }

    fn render_help_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let view = cx.entity().downgrade();
        PopoverMenu::<ContextMenu>::new("mods-help-menu")
            .accessible_label(strings.menu_support())
            .anchor(Anchor::TopRight)
            .attach(Anchor::BottomRight)
            .trigger(if cx.theme().id == ThemeId::NierNew {
                Button::new("mods-help", strings.menu_support())
                    .style(ButtonStyle::Link)
                    .end_icon(IconName::KeyboardArrowDown)
                    .into_any_element()
            } else {
                IconButton::new("mods-help", IconName::HelpOutline)
                    .bordered(true)
                    .tooltip(Tooltip::text(strings.mods_tutorial_help_tooltip()))
                    .into_any_element()
            })
            .menu(move |window, cx| {
                let strings = cx.t();
                let view = view.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    TutorialKind::ALL.into_iter().fold(menu, |menu, kind| {
                        let view = view.clone();
                        menu.icon_entry(kind.menu_label(strings), kind.menu_icon(), move |_, cx| {
                            view.update(cx, |this, cx| this.start_tutorial(kind, cx))
                                .ok();
                        })
                    })
                }))
            })
    }
}

fn write_camera(file: &CameraFile, settings: &CameraSettings) {
    if let Err(error) = yp_mods::camera::save(file, settings) {
        log::warn!("could not save {}: {error}", file.path.display());
    }
}

impl Render for ModsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        if cx.theme().id == ThemeId::NierNew {
            return self.library(window, cx);
        }
        let filter = self.filter_text(cx);
        let row_count = self.sync_rows(cx);
        let (mod_count, loading, load_failed) = {
            let mods = self.stores.mods.read(cx);
            (mods.mods().len(), mods.is_loading(), mods.error().is_some())
        };
        let mods_dir = self
            .stores
            .session
            .read(cx)
            .game()
            .map(|game| game.mods_dir());
        let view = cx.entity().downgrade();

        let row_height = sizes.font_md * 1.3 + sizes.padding_md * 2.;
        let header_height = sizes.font_xl * 1.2 + sizes.card_padding_v * 2.;
        let available = sizes.window_height
            - Sizes::TITLE_BAR_HEIGHT
            - header_height
            - sizes.content_padding * 2.
            - row_height;
        let show_search = should_show_search(mod_count, row_height, available, !filter.is_empty());

        let toolbar = h_flex()
            .flex_none()
            .gap(sizes.spacing_md)
            .child(
                IconButton::new("mods-bulk-install", IconName::LibraryAddOutlined)
                    .controller_support(false)
                    .bordered(true)
                    .tooltip(Tooltip::text(strings.mod_bulk_install()))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.browse(false, ModInstallMode::Bulk, window, cx);
                    })),
            )
            .child(
                IconButton::new("mods-loose-install", IconName::NoteAddOutlined)
                    .controller_support(false)
                    .bordered(true)
                    .tooltip(Tooltip::text(strings.mod_loose_install()))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.browse_loose_folder(window, cx);
                    })),
            )
            .child(self.render_help_menu(cx));
        let header = PageHeader::new(strings.header_mods());
        let header = match mods_dir {
            Some(dir) => header.info(strings.mod_intro_body(), move |_, cx| {
                reveal(dir.clone(), false, cx);
            }),
            None => header,
        };
        let header = header
            .actions(ProfileSelector::new())
            .actions(
                div()
                    .h(sizes.icon_lg)
                    .child(Divider::vertical().color(DividerColor::Medium)),
            )
            .actions(toolbar);

        let list = self.render_list(row_count, loading, load_failed, cx);
        let left = v_flex()
            .size_full()
            .gap(sizes.spacing_md)
            .when(show_search, |this| {
                this.child(
                    Input::new(&self.filter)
                        .prefix(
                            Icon::new(IconName::Search)
                                .size(IconSize::Small)
                                .color(Color::Muted),
                        )
                        .cleanable(true),
                )
            })
            .child(
                fill(
                    Card::new()
                        .full_height()
                        .padding(CardPadding::None)
                        .child(list),
                )
                .tour_anchor(ANCHOR_LIST),
            );

        let drop_view = view.clone();
        let files_view = view.clone();
        let folder_view = view.clone();
        let detail = self.render_detail(window, cx);
        let right = v_flex()
            .size_full()
            .gap(sizes.spacing_md)
            .child(
                div()
                    .relative()
                    .flex_none()
                    .child(
                        DropZone::new("mods-drop-zone")
                            .title(strings.drop_mod_here())
                            .hint(strings.drop_mod_here_hint())
                            .on_drop_paths(move |paths, window, cx| {
                                let paths = paths.to_vec();
                                drop_view
                                    .update(cx, |this, cx| {
                                        this.install_paths(
                                            paths,
                                            ModInstallMode::Guided,
                                            window,
                                            cx,
                                        );
                                    })
                                    .ok();
                            })
                            .on_browse_files(move |window, cx| {
                                files_view
                                    .update(cx, |this, cx| {
                                        this.browse(true, ModInstallMode::Guided, window, cx);
                                    })
                                    .ok();
                            })
                            .on_browse_folder(move |window, cx| {
                                folder_view
                                    .update(cx, |this, cx| {
                                        this.browse(false, ModInstallMode::Guided, window, cx);
                                    })
                                    .ok();
                            }),
                    )
                    .tour_anchor(ANCHOR_DROP_ZONE),
            )
            .child(fill(detail).tour_anchor(ANCHOR_DETAIL));

        let finish_view = view.clone();
        v_flex()
            .size_full()
            .child(header)
            .child(fill(
                div().size_full().p(sizes.content_padding).child(
                    TwoColumn::new(left, right)
                        .id("mods-columns")
                        .ratio(6., 5.)
                        .breakpoint(Pixels::ZERO)
                        .fill_height(true),
                ),
            ))
            .children(self.tutorial.map(|kind| {
                TourOverlay::new(tour_steps(kind, strings))
                    .id(SharedString::from(format!("mods-tutorial-{}", kind.key())))
                    .on_finish(move |_, cx| {
                        finish_view
                            .update(cx, |this, cx| this.close_tutorial(cx))
                            .ok();
                    })
            }))
            .into_any_element()
    }
}
