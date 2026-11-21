pub mod model;

use std::path::PathBuf;

use futures::channel::oneshot;
use gpui::{Global, PathPromptOptions, ScrollHandle, Subscription, Task};
use yp_core::GamePaths;
use yp_state::{ConfigStore, TexturePackOutcome};
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    BorderPosition, BusyOverlay, ButtonSize, ButtonStyle, Callout, Card, Chip, ChipSize, ChipTone,
    DropProgress, DropZone, Icon, IconButton, IconName, IconSize, Label,
    LabelSize, Link, SortableList, Switch, SwitchSize, TintColor, Tooltip, TwoColumn,
};

use crate::extras::MANAGED_TEXTURE_IDS;
use crate::fields::Marker;
use crate::page::{column, reveal, tab_page};
use crate::schema_page::{self, TabScope};
use model::{InstallOutcome, PackRow, Priority, TexturesEvent, TexturesModel};

const SCOPE: &str = "textures";
const INJECT_FOLDER_LABEL: &str = "nams/inject/textures/";
const SK_RES_FOLDER_LABEL: &str = "SK_Res/";
const WAX_FOLDER_LABEL: &str = "wax/mods/";



const FILE: &str = "texture_injection.toml";
pub const TAB: TabScope = TabScope::roots(&["textures"]).managed(&MANAGED_TEXTURE_IDS);

pub struct SharedTextures(pub Entity<TexturesModel>);

impl Global for SharedTextures {}

pub fn shared_model(cx: &mut App) -> Entity<TexturesModel> {
    if let Some(shared) = cx.try_global::<SharedTextures>() {
        return shared.0.clone();
    }
    let model = cx.new(TexturesModel::new);
    cx.set_global(SharedTextures(model.clone()));
    model
}

fn browse(model: Entity<TexturesModel>, directories: bool, window: &mut Window, cx: &mut App) {
    let strings = cx.t();
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: !directories,
        directories,
        multiple: !directories,
        prompt: Some(SharedString::new_static(strings.select_texture_files())),
    });
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            model
                .update_in(cx, |model, window, cx| model.install(paths, window, cx))
                .ok();
        })
        .detach();
}

pub fn install_zone(model: &Entity<TexturesModel>, cx: &App) -> DropZone {
    let strings = cx.t();
    let progress = model.read(cx).install_phase().map(|phase| {
        let progress = DropProgress::new(phase.text(strings));
        match phase.percent() {
            Some(percent) => progress.percent(percent),
            None => progress,
        }
    });
    let drop_model = model.clone();
    let files_model = model.clone();
    let folder_model = model.clone();
    DropZone::new("textures-drop-zone")
        .title(strings.drop_texture_here())
        .hint(strings.or_click_to_browse())
        .icon(IconName::Texture)
        .footer(strings.installed_to_textures())
        .busy(progress)
        .on_drop_paths(move |paths, window, cx| {
            let paths = paths.to_vec();
            drop_model.update(cx, |model, cx| model.install(paths, window, cx));
        })
        .on_browse_files(move |window, cx| browse(files_model.clone(), false, window, cx))
        .on_browse_folder(move |window, cx| browse(folder_model.clone(), true, window, cx))
}

pub fn install_packs(
    paths: Vec<PathBuf>,
    window: &mut Window,
    cx: &mut App,
) -> Task<TexturePackOutcome> {
    let model = shared_model(cx);
    let ready = {
        let model = model.read(cx);
        !paths.is_empty() && !model.is_installing() && model.game().is_some()
    };
    if !ready {
        return Task::ready(TexturePackOutcome::Failed);
    }
    let (sender, receiver) = oneshot::channel();
    let mut sender = Some(sender);
    let subscription = cx.subscribe(&model, move |_, event: &TexturesEvent, _| {
        if let TexturesEvent::InstallFinished(outcome) = event
            && let Some(sender) = sender.take()
        {
            let _ = sender.send(outcome.clone());
        }
    });
    model.update(cx, |model, cx| model.install(paths, window, cx));
    cx.spawn(async move |_| {
        let outcome = receiver.await;
        drop(subscription);
        match outcome {
            Ok(InstallOutcome::Installed(_)) => TexturePackOutcome::Installed,
            Ok(InstallOutcome::Cancelled) => TexturePackOutcome::Cancelled,
            _ => TexturePackOutcome::Failed,
        }
    })
}

fn folder_link(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    folder: PathBuf,
) -> gpui::Div {
    let name: SharedString = name.into();
    div().min_w_0().truncate().child(
        Link::new(id, name.clone())
            .size(LabelSize::XSmall)
            .color(Color::Secondary)
            .tooltip(name)
            .controller_support(false)
            .on_click(move |_, _, cx| reveal(folder.clone(), false, cx)),
    )
}

fn pack_row(
    row: &PackRow,
    index: usize,
    model: &Entity<TexturesModel>,
    game: &GamePaths,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let strings = cx.t();
    let theme = cx.theme().clone();
    let sizes = window.sizes(cx);
    let accent = theme.colors.accent_primary;
    let hover_background = accent.opacity(theme.alpha.faint);
    let hover_border = accent.opacity(theme.alpha.soft);
    let name = row.name.clone();
    let folder = game.inject_textures_dir().join(&row.name);
    let toggle_model = model.clone();
    let toggle_name = name.clone();
    let delete_model = model.clone();
    let delete_name = name.clone();
    let enabled = !row.disabled;
    let title = if row.exists {
        folder_link(("texture-pack-name", index), name.clone(), folder).into_any_element()
    } else {
        div()
            .min_w_0()
            .child(
                Label::new(name.clone())
                    .size(LabelSize::Small)
                    .color(Color::Warning)
                    .truncate(),
            )
            .into_any_element()
    };
    h_flex()
        .id(("texture-pack-row", index))
        .w_full()
        .gap(sizes.spacing_md)
        .mb(sizes.padding_xs)
        .px(sizes.padding_md)
        .py(sizes.padding_sm)
        .rounded(sizes.border_radius)
        .border_1()
        .border_color(theme.colors.border_light)
        .bg(theme.colors.surface_light)
        .hover(move |style| style.bg(hover_background).border_color(hover_border))
        .child(
            h_flex()
                .flex_1()
                .min_w_0()
                .gap(sizes.spacing_md)
                .when(row.disabled, |this| this.opacity(theme.alpha.half))
                .child(
                    Icon::new(IconName::Folder)
                        .size(IconSize::Small)
                        .color(if row.exists {
                            Color::Muted
                        } else {
                            Color::Warning
                        }),
                )
                .child(div().flex_1().min_w_0().child(title))
                .when_some(row.bundled_with.clone(), |this, mod_id| {
                    let target = mod_id.clone();
                    this.child(
                        IconButton::new(("texture-pack-mod", index), IconName::ExtensionOutlined)
                            .icon_size(IconSize::Small)
                            .icon_color(Color::Accent)
                            .size(ButtonSize::Compact)
                            .tooltip(Tooltip::text(strings.texture_bundled_with_mod(&mod_id)))
                            .on_click(move |_, window, cx| {
                                window.dispatch_action(
                                    Box::new(OpenTab::selecting(TabId::Mods, target.clone())),
                                    cx,
                                );
                            }),
                    )
                })
                .when(!row.conflicts.is_empty(), |this| {
                    this.child(
                        Chip::new(strings.texture_overlap_label())
                            .id(("texture-pack-overlap", index))
                            .icon(IconName::WarningAmber)
                            .tone(ChipTone::Warning)
                            .size(ChipSize::Small)
                            .tooltip_text(
                                strings.tooltip_texture_overlap(&row.conflicts.join(", ")),
                            ),
                    )
                })
                .when_some(row.priority, |this, priority| {
                    this.child(
                        Chip::new(match priority {
                            Priority::Highest => strings.priority_highest(),
                            Priority::Lowest => strings.priority_lowest(),
                        })
                        .tone(ChipTone::Accent)
                        .size(ChipSize::Small),
                    )
                })
                .when(!row.exists, |this| {
                    this.child(
                        div()
                            .id(("texture-pack-missing", index))
                            .child(
                                Icon::new(IconName::WarningAmber)
                                    .size(IconSize::Small)
                                    .color(Color::Warning),
                            )
                            .tooltip(Tooltip::text(strings.tooltip_folder_not_found())),
                    )
                }),
        )
        .child(
            div()
                .id(("texture-pack-toggle", index))
                .flex_none()
                .tooltip(Tooltip::text(if enabled {
                    strings.mod_enable_tooltip()
                } else {
                    strings.mod_disabled_tooltip()
                }))
                .child(
                    Switch::new(("texture-pack-switch", index), enabled)
                        .size(SwitchSize::Small)
                        .on_click(move |state, _, cx| {
                            let enabled = state.selected();
                            toggle_model.update(cx, |model, cx| {
                                model.set_pack_enabled(&toggle_name, enabled, cx)
                            });
                        }),
                ),
        )
        .when(row.bundled_with.is_none(), |this| {
            this.child(
                IconButton::new(("texture-pack-delete", index), IconName::Close)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Error)
                    .size(ButtonSize::Compact)
                    .style(ButtonStyle::Tinted(TintColor::Error))
                    .tooltip(Tooltip::text(strings.texture_delete_label()))
                    .on_click(move |_, window, cx| {
                        let name = delete_name.clone();
                        delete_model.update(cx, |model, cx| model.delete(name, window, cx));
                    }),
            )
        })
        .into_any_element()
}

pub struct TexturesTab {
    model: Entity<TexturesModel>,
    config: Entity<ConfigStore>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl TexturesTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let model = shared_model(cx);
        let subscriptions = vec![
            cx.observe(&model, |_, _, cx| cx.notify()),
            cx.observe(&stores.config, |_, _, cx| cx.notify()),
            cx.observe(&stores.mods, |_, _, cx| cx.notify()),
            cx.observe(&stores.disabled_mods, |_, _, cx| cx.notify()),
        ];
        Self {
            model,
            config: stores.config.clone(),
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn model(&self) -> &Entity<TexturesModel> {
        &self.model
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    fn config_cards(&self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let plans = schema_page::plans(SCOPE, &TAB, self.config.read(cx));
        plans
            .into_iter()
            .map(|plan| schema_page::generic_card(SCOPE, plan, &self.config, Marker::RestartOnly, window, cx))
            .collect()
    }

    fn inject_card(
        &self,
        game: &GamePaths,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let model = self.model.read(cx);
        let rows = model.rows(cx);
        let listing = model.listing().clone();
        let count = i64::try_from(listing.nams.len()).unwrap_or(i64::MAX);
        let mut card = Card::new().title(strings.textures_ui_folder_count(INJECT_FOLDER_LABEL, count));
        if !listing.conflicts.is_empty() {
            card = card.child(
                div().w_full().pb(sizes.spacing_md).child(
                    Callout::new()
                        .id("textures-conflict-hint")
                        .severity(Severity::Accent)
                        .icon(IconName::SwapVert)
                        .border_position(BorderPosition::None)
                        .description(strings.texture_conflict_hint()),
                ),
            );
        } else if rows.len() > 1 {
            card = card.child(
                div().pb(sizes.spacing_md).child(
                    Label::new(strings.no_conflicts_found())
                        .size(LabelSize::XSmall)
                        .color(Color::Success),
                ),
            );
        }
        if !rows.is_empty() {
            let row_model = self.model.clone();
            let row_game = game.clone();
            let reorder_model = self.model.clone();
            return card
                .child(
                    SortableList::new("textures-load-order", rows)
                        .drag_label(|row: &PackRow| SharedString::from(row.name.clone()))
                        .row(move |row, index, window, cx| {
                            pack_row(row, index, &row_model, &row_game, window, cx)
                        })
                        .on_reorder(move |from, to, _, cx| {
                            reorder_model.update(cx, |model, cx| model.reorder(from, to, cx));
                        }),
                )
                .into_any_element();
        }
        let texture_dir = game.inject_textures_dir();
        for (index, name) in listing.nams.iter().enumerate() {
            let delete_model = self.model.clone();
            let delete_name = name.clone();
            card = card.child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .pb(sizes.padding_xs)
                    .child(
                        Icon::new(IconName::Image)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(div().flex_1().min_w_0().child(folder_link(
                        ("texture-loose-name", index),
                        name.clone(),
                        texture_dir.join(name),
                    )))
                    .child(
                        IconButton::new(("texture-loose-delete", index), IconName::Close)
                            .icon_size(IconSize::Small)
                            .icon_color(Color::Error)
                            .size(ButtonSize::Compact)
                            .style(ButtonStyle::Tinted(TintColor::Error))
                            .tooltip(Tooltip::text(strings.texture_delete_label()))
                            .on_click(move |_, window, cx| {
                                let name = delete_name.clone();
                                delete_model
                                    .update(cx, |model, cx| model.delete(name, window, cx));
                            }),
                    ),
            );
        }
        card.into_any_element()
    }

    fn external_card(
        id: &'static str,
        folder_label: &'static str,
        priority: &'static str,
        entries: &[String],
        root: PathBuf,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let count = i64::try_from(entries.len()).unwrap_or(i64::MAX);
        Card::new()
            .title(strings.textures_ui_folder_count(folder_label, count))
            .header_end(
                Chip::new(priority)
                    .tone(ChipTone::Muted)
                    .size(ChipSize::Small),
            )
            .children(entries.iter().enumerate().map(|(index, name)| {
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .pb(sizes.padding_xs)
                    .child(
                        Icon::new(IconName::Image)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(div().flex_1().min_w_0().child(folder_link(
                        (id, index),
                        name.clone(),
                        root.join(name),
                    )))
            }))
            .into_any_element()
    }

    fn outfit_card(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let packs = self.model.read(cx).outfit_packs().to_vec();
        let count = i64::try_from(packs.len()).unwrap_or(i64::MAX);
        Card::new()
            .header(
                h_flex()
                    .gap(sizes.spacing_sm)
                    .child(
                        Icon::new(IconName::Checkroom)
                            .size(IconSize::Small)
                            .color(Color::Accent),
                    )
                    .child(
                        Label::new(strings.textures_ui_folder_count(
                            strings.texture_outfit_linked_title(),
                            count,
                        ))
                        .size(LabelSize::Small)
                        .color(Color::Accent)
                        .bold(),
                    ),
            )
            .child(
                div().pb(sizes.spacing_md).child(
                    Label::new(strings.texture_outfit_linked_subtitle())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .line_height(1.3),
                ),
            )
            .children(packs.into_iter().enumerate().map(|(index, pack)| {
                let name = match &pack.character {
                    Some(character) => {
                        strings.textures_ui_name_with_owner(&pack.name, character)
                    }
                    None => pack.name.clone(),
                };
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .pb(sizes.padding_xs)
                    .child(
                        Icon::new(IconName::Link)
                            .size(IconSize::Small)
                            .color(Color::Accent),
                    )
                    .child(div().flex_1().min_w_0().child(folder_link(
                        ("texture-outfit-pack", index),
                        name,
                        PathBuf::from(&pack.path),
                    )))
                    .child(
                        Label::new(strings.texture_outfit_linked_entry(pack.dds_count))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
            }))
            .into_any_element()
    }
}

impl Render for TexturesTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let store = self.config.read(cx);
        let loading = schema_page::is_loading(store);
        let header = schema_page::header(
            strings.header_textures(),
            strings.tooltip_edits_texture_injection_toml(),
            Some(FILE),
            None,
            &self.config,
            cx,
        );
        let model = self.model.read(cx);
        let deleting = model.is_deleting();
        let listing_loading = model.is_loading();
        let has_textures = model.has_any_textures();
        let listing = model.listing().clone();
        let has_outfit_packs = !model.outfit_packs().is_empty();
        let game = model.game().cloned();
        let content = if loading {
            div().into_any_element()
        } else {
            let mut left_sections = vec![
                Card::new()
                    .title(strings.card_install_textures())
                    .child(install_zone(&self.model, cx))
                    .into_any_element(),
            ];
            left_sections.extend(self.config_cards(window, cx));
            let left = column(left_sections, window, cx);
            let mut right: Vec<AnyElement> = Vec::new();
            if let (false, Some(game)) = (listing_loading, game.as_ref()) {
                if !has_textures {
                    right.push(
                        div()
                            .w_full()
                            .py(sizes.padding_xl)
                            .flex()
                            .justify_center()
                            .child(
                                Label::new(strings.no_textures_installed())
                                    .color(Color::Muted)
                                    .italic(),
                            )
                            .into_any_element(),
                    );
                }
                if !listing.nams.is_empty() {
                    right.push(self.inject_card(game, window, cx));
                }
                if has_outfit_packs {
                    right.push(self.outfit_card(window, cx));
                }
                if !listing.sk.is_empty() {
                    right.push(Self::external_card(
                        "texture-sk-entry",
                        SK_RES_FOLDER_LABEL,
                        strings.priority_medium(),
                        &listing.sk,
                        game.special_k_textures_dir().join("NieRAutomata.exe"),
                        window,
                        cx,
                    ));
                }
                if !listing.wax.is_empty() {
                    right.push(Self::external_card(
                        "texture-wax-entry",
                        WAX_FOLDER_LABEL,
                        strings.priority_highest(),
                        &listing.wax,
                        game.wax_mods_dir(),
                        window,
                        cx,
                    ));
                }
            }
            let right = column(right, window, cx);
            let mut sections = schema_page::problems(SCOPE, &TAB, &self.config, cx);
            sections.push(
                TwoColumn::new(left, right)
                    .id("textures-columns")
                    .into_any_element(),
            );
            column(sections, window, cx).into_any_element()
        };
        tab_page("textures-scroll", header, &self.scroll, loading, content, window, cx)
            .when(deleting, |this| {
                this.child(BusyOverlay::new(strings.busy_deleting_texture()))
            })
    }
}
