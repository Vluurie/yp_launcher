use yp_mods::camera::CameraField;
use yp_mods::defaults::{DefaultKind, initial_outfit_id, kind_for};
use yp_mods::{
    DataArchivePair, DataCategory, DataSummary, InstalledMod, NativeSummary, OutfitChoice,
};
use yp_state::inspector::ArchivePaths;
use yp_state::prelude::*;
use yp_state::{ModelPreviewRequest, OverlayId};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, Card, CardPadding, ChipSize, ChipTone, ConfirmOptions, Divider,
    EmptyState, KeyValueRow, PickItem, PickOptions, ScrollableElement, Select, SelectItem,
    SelectableText, SettingSlider, TintColor, Tooltip, confirm, pick_one,
};

use crate::labels::{
    camera_field_hint, camera_field_label, data_category_label, default_kind_label,
    default_kind_tooltip, kind_badge, kind_doc_page, outfit_choice_label,
};
use crate::layout::fill;
use crate::mods::list::{
    conflict_candidates, conflict_file_count, mod_id_of_entry, mod_rel_path, player_stems,
    stem_rel_path,
};
use crate::mods::view::ModsView;
use crate::notify::reveal;

struct StemModel {
    label: String,
    rel_path: String,
    current: Option<DefaultKind>,
    target: DefaultKind,
    choices: Vec<OutfitChoice>,
    outfit_id: Option<i64>,
}

pub(crate) struct DefaultRequest {
    pub model: String,
    pub mod_name: String,
    pub rel_path: String,
    pub current: Option<DefaultKind>,
    pub target: DefaultKind,
    pub choices: Vec<OutfitChoice>,
}

fn section(title: &'static str) -> Card {
    Card::new().title(title).padding(CardPadding::Compact)
}

fn count(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn short_pack_label(name: &str) -> String {
    yp_mods::pack_name::parse_scoped(name)
        .map(|scoped| scoped.base.trim().to_owned())
        .filter(|base| !base.is_empty())
        .unwrap_or_else(|| name.to_owned())
}

impl ModsView {
    fn mod_label(&self, entry_path: &str, cx: &App) -> String {
        let id = mod_id_of_entry(entry_path);
        if let Some(custom) = self.stores.mod_names.read(cx).custom_name_of(id) {
            return custom.to_owned();
        }
        self.stores
            .mods
            .read(cx)
            .find(id)
            .map(|installed| installed.display_name.clone())
            .unwrap_or_else(|| id.to_owned())
    }

    pub(crate) fn toggle_default(
        &mut self,
        request: DefaultRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let defaults = self.stores.default_mods.clone();
        if request.current.is_some() {
            defaults.update(cx, |store, cx| {
                store.set_default(&request.rel_path, None, None, cx);
            });
            return;
        }
        let outfit_id = initial_outfit_id(request.target, &request.choices);
        let replaced = defaults
            .read(cx)
            .would_replace(&request.rel_path, request.target)
            .map(|entry| entry.path.clone());
        let Some(replaced) = replaced else {
            defaults.update(cx, |store, cx| {
                store.set_default(&request.rel_path, Some(request.target), outfit_id, cx);
            });
            return;
        };
        let strings = cx.t();
        let current = self.mod_label(&replaced, cx);
        let confirming = confirm(
            ConfirmOptions::new(
                strings.mod_default_replace_title(),
                strings.mod_default_replace_body(&request.model, &current, &request.mod_name),
            )
            .confirm_label(strings.mod_default_replace_confirm())
            .cancel_label(strings.button_cancel()),
            window,
            cx,
        );
        cx.spawn(async move |_, cx| {
            if !confirming.await {
                return;
            }
            defaults
                .update(cx, |store, cx| {
                    store.set_default(&request.rel_path, Some(request.target), outfit_id, cx);
                });
        })
        .detach();
    }

    fn resolve_conflicts(
        &mut self,
        mod_id: String,
        candidates: Vec<String>,
        file_count: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let strings = cx.t();
        let items = candidates
            .iter()
            .map(|id| {
                PickItem::new(id.clone(), id.clone())
                    .icon(if *id == mod_id {
                        IconName::Adjust
                    } else {
                        IconName::CircleOutlined
                    })
                    .description(strings.mod_conflict_keep_tooltip(id))
            })
            .collect();
        let picking = pick_one(
            PickOptions::new(strings.mod_conflict_dialog_title(), items)
                .body(strings.mod_conflict_pick_body(count(candidates.len()), count(file_count)))
                .cancel_label(strings.button_cancel()),
            window,
            cx,
        );
        let disabled = self.stores.disabled_mods.clone();
        cx.spawn(async move |_, cx| {
            let Some(winner) = picking.await else {
                return;
            };
            disabled
                .update(cx, |store, cx| {
                    for id in candidates.iter().filter(|id| **id != winner) {
                        store.set_disabled(&mod_rel_path(id), true, cx);
                    }
                });
        })
        .detach();
    }

    pub fn resolve_selected_conflicts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(installed) = self
            .selected_id
            .as_ref()
            .and_then(|id| self.stores.mods.read(cx).find(id).cloned())
        else {
            return;
        };
        let candidates = {
            let disabled = self.stores.disabled_mods.read(cx);
            conflict_candidates(&installed, &|rel| disabled.is_disabled(rel))
        };
        if candidates.len() < 2 {
            return;
        }
        let file_count = conflict_file_count(&installed);
        self.resolve_conflicts(installed.id, candidates, file_count, window, cx);
    }

    fn preview_for(
        &mut self,
        archive: &DataArchivePair,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyView> {
        let key = format!("{}|{}", archive.config_root.display(), archive.stem);
        if let Some((_, view)) = self.previews.iter().find(|(existing, _)| *existing == key) {
            return Some(view.clone());
        }
        let request = ModelPreviewRequest {
            archives: ArchivePaths::new(archive.dat_path.clone(), archive.dtt_path.clone()),
        };
        let view = Registry::model_preview(request, window, cx)?;
        self.previews.push((key, view.clone()));
        Some(view)
    }

    fn stem_models(
        &self,
        installed: &InstalledMod,
        data: &DataSummary,
        cx: &App,
    ) -> Vec<StemModel> {
        let defaults = self.stores.default_mods.read(cx);
        player_stems(data)
            .into_iter()
            .map(|player| {
                let rel_path = stem_rel_path(&installed.id, &player.stem);
                let target = kind_for(&player.stem, data.has_outfit_config);
                let choices = if target == DefaultKind::OutfitConfig {
                    data.outfits_by_stem
                        .get(&player.stem)
                        .cloned()
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                StemModel {
                    label: player.label,
                    current: defaults.kind_of(&rel_path),
                    outfit_id: defaults.outfit_id_of(&rel_path),
                    rel_path,
                    target,
                    choices,
                }
            })
            .collect()
    }

    fn render_stem(
        &self,
        index: usize,
        stem: StemModel,
        mod_name: &str,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let starred = stem.current.is_some();
        let shown = stem.current.unwrap_or(stem.target);
        let label = Label::new(stem.label.clone())
            .size(LabelSize::Small)
            .truncate();
        let label = if starred {
            label.color(Color::AccentSecondary).semibold()
        } else {
            label.color(Color::Secondary)
        };
        let picker = (starred && stem.choices.len() > 1).then(|| {
            let selected = stem
                .choices
                .iter()
                .find(|choice| Some(choice.outfit_id) == stem.outfit_id)
                .map(|choice| outfit_choice_label(choice, strings))
                .unwrap_or_else(|| strings.mod_default_outfit_auto().to_owned());
            let items = std::iter::once(SelectItem::new(strings.mod_default_outfit_auto(), None))
                .chain(stem.choices.iter().map(|choice| {
                    SelectItem::new(outfit_choice_label(choice, strings), Some(choice.outfit_id))
                }));
            let rel_path = stem.rel_path.clone();
            let target = stem.target;
            Select::<Option<i64>>::new(("mod-default-outfit", index), selected)
                .items(items)
                .min_width(sizes.config_input_width)
                .max_width(sizes.config_input_width * 1.6)
                .tooltip(strings.mod_default_outfit_pick_tooltip())
                .on_select(cx.listener(move |this, outfit_id: &Option<i64>, _, cx| {
                    let outfit_id = *outfit_id;
                    this.stores.default_mods.update(cx, |store, cx| {
                        store.set_default(&rel_path, Some(target), outfit_id, cx);
                    });
                }))
        });
        let kind_chip = starred.then(|| {
            Chip::new(default_kind_label(shown, strings))
                .id(("mod-default-kind", index))
                .tone(ChipTone::Accent)
                .size(ChipSize::Small)
                .tooltip_text(default_kind_tooltip(shown, strings))
        });
        let request_parts = (
            stem.label.clone(),
            mod_name.to_owned(),
            stem.rel_path.clone(),
            stem.current,
            stem.target,
            stem.choices.clone(),
        );
        let star = IconButton::new(
            ("mod-default-star", index),
            if starred {
                IconName::Star
            } else {
                IconName::StarOutline
            },
        )
        .size(ButtonSize::Compact)
        .icon_size(IconSize::Small)
        .icon_color(if starred {
            Color::AccentSecondary
        } else {
            Color::Muted
        })
        .tooltip(Tooltip::text(if starred {
            strings.mod_default_tooltip()
        } else {
            default_kind_tooltip(shown, strings)
        }))
        .on_click(cx.listener(move |this, _, window, cx| {
            let (model, mod_name, rel_path, current, target, choices) = request_parts.clone();
            this.toggle_default(
                DefaultRequest {
                    model,
                    mod_name,
                    rel_path,
                    current,
                    target,
                    choices,
                },
                window,
                cx,
            );
        }));
        h_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .pl(sizes.padding_sm)
            .child(div().flex_1().min_w_0().child(label))
            .children(picker.map(|picker| div().flex_none().child(picker)))
            .children(kind_chip.map(|chip| div().flex_none().child(chip)))
            .child(star)
            .into_any_element()
    }

    fn render_native(&self, native: &NativeSummary, sizes: &Sizes, cx: &mut Context<Self>) -> Card {
        let strings = cx.t();
        let registry = cx.registry();
        let docs_available = registry.docs_library && registry.overlays.contains(OverlayId::Docs);
        let mut bundles: Vec<(&String, &usize)> = native.bundles_by_kind.iter().collect();
        bundles.sort_by(|a, b| a.0.cmp(b.0));
        let mut lines: Vec<AnyElement> = bundles
            .into_iter()
            .enumerate()
            .map(|(index, (kind, total))| {
                let text = strings.mods_ui_label_value(kind, &total.to_string());
                match kind_doc_page(kind).filter(|_| docs_available) {
                    Some(page) => Link::new(("mod-kind-doc", index), text)
                        .size(LabelSize::Small)
                        .color(Color::Secondary)
                        .hover_color(Color::Default)
                        .end_icon(IconName::MenuBookOutlined)
                        .on_click(move |_, window, cx| {
                            window.dispatch_action(
                                Box::new(OpenDocs {
                                    page: Some(page.clone()),
                                }),
                                cx,
                            );
                        })
                        .into_any_element(),
                    None => Label::new(text)
                        .size(LabelSize::Small)
                        .color(Color::Secondary)
                        .into_any_element(),
                }
            })
            .collect();
        if lines.is_empty() {
            lines.push(
                Label::new(strings.mod_count_files(count(native.total_entity_files)))
                    .size(LabelSize::Small)
                    .color(Color::Secondary)
                    .into_any_element(),
            );
        }
        section(strings.mod_native_bundles()).child(
            v_flex()
                .w_full()
                .items_start()
                .gap(sizes.spacing_sm)
                .children(lines),
        )
    }

    fn render_camera(&self, sizes: &Sizes, cx: &mut Context<Self>) -> Option<Card> {
        let strings = cx.t();
        let editor = self.camera.as_ref()?;
        let sliders: Vec<AnyElement> = CameraField::ALL
            .into_iter()
            .enumerate()
            .map(|(index, field)| {
                SettingSlider::new(
                    ("mod-camera", index),
                    camera_field_label(field, strings),
                    editor.settings.get(field) as f32,
                )
                .min(field.min() as f32)
                .max(field.max() as f32)
                .step(field.step() as f32)
                .decimals(usize::try_from(field.decimals()).unwrap_or(0))
                .description(camera_field_hint(field, strings))
                .on_change(cx.listener(move |this, value: &f32, _, cx| {
                    this.set_camera_value(field, *value, cx);
                }))
                .into_any_element()
            })
            .collect();
        Some(
            section(strings.mod_camera_settings()).child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_sm)
                    .child(
                        Label::new(strings.mod_camera_settings_hint())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .children(sliders),
            ),
        )
    }

    fn render_data(
        &mut self,
        installed: &InstalledMod,
        data: &DataSummary,
        sizes: &Sizes,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Card {
        let strings = cx.t();
        let mut lines: Vec<AnyElement> = Vec::new();
        if data.has_compat_config {
            lines.push(
                Chip::new(strings.mod_compat_chip())
                    .id("mod-detail-compat")
                    .tone(ChipTone::Muted)
                    .tooltip_text(strings.mod_compat_chip_tooltip())
                    .into_any_element(),
            );
        }
        if !data.players.is_empty() {
            lines.push(
                Label::new(strings.mod_data_player_models())
                    .size(LabelSize::Small)
                    .color(Color::Secondary)
                    .into_any_element(),
            );
            let defaults_enabled = self.stores.default_mods.read(cx).enabled();
            if defaults_enabled {
                let stems = self.stem_models(installed, data, cx);
                for (index, stem) in stems.into_iter().enumerate() {
                    lines.push(self.render_stem(index, stem, &installed.display_name, sizes, cx));
                }
            } else {
                for player in player_stems(data) {
                    lines.push(
                        div()
                            .pl(sizes.padding_sm)
                            .child(
                                Label::new(player.label)
                                    .size(LabelSize::Small)
                                    .color(Color::Secondary),
                            )
                            .into_any_element(),
                    );
                }
            }
        }
        for archive in &data.archives {
            if let Some(preview) = self.preview_for(archive, window, cx) {
                lines.push(div().w_full().child(preview).into_any_element());
            }
        }
        for entry in &data.entries {
            if entry.category == DataCategory::Player && !data.players.is_empty() {
                continue;
            }
            let label = data_category_label(entry.category, &entry.dir_name, strings);
            lines.push(
                Label::new(strings.mods_ui_label_value(
                    &label,
                    &strings.mod_count_files(count(entry.file_count)),
                ))
                .size(LabelSize::Small)
                .color(Color::Secondary)
                .into_any_element(),
            );
        }
        if lines.is_empty() {
            lines.push(
                Label::new(strings.mods_ui_none())
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .into_any_element(),
            );
        }
        section(strings.mod_data_content()).child(
            v_flex()
                .w_full()
                .items_start()
                .gap(sizes.spacing_sm)
                .children(lines),
        )
    }

    fn render_requires(&self, installed: &InstalledMod, sizes: &Sizes, cx: &App) -> Option<Card> {
        let strings = cx.t();
        let manifest = installed.manifest.as_ref()?;
        if manifest.requires.is_empty() && manifest.requires_plugins.is_empty() {
            return None;
        }
        let chips = manifest.requires.iter().enumerate().map(|(index, id)| {
            let missing = installed.requires_missing.contains(id);
            let label = if missing {
                strings.mods_ui_with_note(id, strings.mod_requires_missing())
            } else {
                id.clone()
            };
            Chip::new(label)
                .id(("mod-requires", index))
                .tone(if missing {
                    ChipTone::Error
                } else {
                    ChipTone::Neutral
                })
        });
        Some(
            section(strings.mod_requires_label()).child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .when(!manifest.requires.is_empty(), |this| {
                        this.child(h_flex().flex_wrap().gap(sizes.spacing_md).children(chips))
                    })
                    .when(!manifest.requires_plugins.is_empty(), |this| {
                        this.child(
                            Label::new(strings.mods_ui_label_value(
                                strings.mod_requires_plugins_label(),
                                &manifest.requires_plugins.join(", "),
                            ))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                        )
                    }),
            ),
        )
    }

    fn render_conflicts(
        &self,
        installed: &InstalledMod,
        sizes: &Sizes,
        cx: &mut Context<Self>,
    ) -> Option<Card> {
        let strings = cx.t();
        if installed.conflicts.is_empty() {
            return None;
        }
        let candidates = {
            let disabled = self.stores.disabled_mods.read(cx);
            conflict_candidates(installed, &|rel| disabled.is_disabled(rel))
        };
        if candidates.len() < 2 {
            return None;
        }
        let file_count = conflict_file_count(installed);
        let body = strings.mod_conflict_pick_body(count(candidates.len()), count(file_count));
        let mod_id = installed.id.clone();
        Some(
            section(strings.mod_conflicts_label()).child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(
                        Icon::new(IconName::WarningAmberRounded)
                            .size(IconSize::Small)
                            .color(Color::Warning),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            Label::new(body)
                                .size(LabelSize::Small)
                                .color(Color::Secondary),
                        ),
                    )
                    .child(
                        Button::new("mod-conflict-resolve", strings.mod_conflict_resolve())
                            .style(ButtonStyle::Tinted(TintColor::Warning))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.resolve_conflicts(
                                    mod_id.clone(),
                                    candidates.clone(),
                                    file_count,
                                    window,
                                    cx,
                                );
                            })),
                    ),
            ),
        )
    }

    pub(crate) fn render_detail(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let selected = self
            .selected_id
            .as_ref()
            .and_then(|id| self.stores.mods.read(cx).find(id).cloned());
        let Some(installed) = selected else {
            return Card::new()
                .full_height()
                .child(fill(
                    EmptyState::new(strings.mod_detail_no_selection())
                        .icon(IconName::ExtensionOutlined),
                ))
                .into_any_element();
        };
        let rel_path = mod_rel_path(&installed.id);
        let disabled = self.stores.disabled_mods.read(cx).is_disabled(&rel_path);
        let badge = kind_badge(installed.kind, strings);

        let root_path = installed.root_path.clone();
        let toggle_id = installed.id.clone();
        let uninstall_id = installed.id.clone();
        let actions = h_flex()
            .flex_none()
            .gap(sizes.spacing_md)
            .px(sizes.card_padding_h)
            .py(sizes.card_padding_v)
            .child(
                IconButton::new("mod-detail-open", IconName::FolderOpen)
                    .bordered(true)
                    .tooltip(Tooltip::text(strings.mod_open_folder()))
                    .controller_support(false)
                    .on_click(move |_, _, cx| reveal(root_path.clone(), false, cx)),
            )
            .child(
                IconButton::new(
                    "mod-detail-toggle",
                    if disabled {
                        IconName::ToggleOffOutlined
                    } else {
                        IconName::ToggleOn
                    },
                )
                .style(ButtonStyle::Tinted(if disabled {
                    TintColor::Muted
                } else {
                    TintColor::Success
                }))
                .tooltip(Tooltip::text(if disabled {
                    strings.mod_enable()
                } else {
                    strings.mod_disable()
                }))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_mod_disabled(&toggle_id, !disabled, cx);
                })),
            )
            .child(
                IconButton::new("mod-detail-uninstall", IconName::DeleteOutline)
                    .style(ButtonStyle::Tinted(TintColor::Error))
                    .tooltip(Tooltip::text(strings.mod_uninstall()))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.uninstall_mod(uninstall_id.clone(), window, cx);
                    })),
            );

        let mut sections: Vec<AnyElement> = Vec::new();
        if let Some(manifest) = &installed.manifest {
            let rows = [
                ("mod-detail-author", strings.mod_author(), &manifest.author),
                (
                    "mod-detail-version",
                    strings.mod_version(),
                    &manifest.version,
                ),
            ];
            let rows: Vec<AnyElement> = rows
                .into_iter()
                .filter_map(|(id, label, value)| {
                    let value = value.clone()?;
                    Some(
                        KeyValueRow::with_slot(
                            label,
                            SelectableText::new(id, value)
                                .size(LabelSize::Small)
                                .color(Color::Secondary),
                        )
                        .label_width(sizes.config_input_width)
                        .into_any_element(),
                    )
                })
                .collect();
            if !rows.is_empty() {
                sections.push(v_flex().w_full().children(rows).into_any_element());
            }
        }
        if let Some(native) = &installed.native {
            sections.push(self.render_native(native, &sizes, cx).into_any_element());
            if let Some(camera) = self.render_camera(&sizes, cx) {
                sections.push(camera.into_any_element());
            }
        }
        if let Some(data) = &installed.data {
            sections.push(
                self.render_data(&installed, data, &sizes, window, cx)
                    .into_any_element(),
            );
        }
        if !installed.bundled_texture_packs.is_empty() {
            let chips = installed
                .bundled_texture_packs
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let target = name.clone();
                    Chip::new(short_pack_label(name))
                        .id(("mod-bundled-texture", index))
                        .icon(IconName::Texture)
                        .tone(ChipTone::Accent)
                        .tooltip_text(strings.tooltip_open_in_textures_tab(name))
                        .on_click(move |_, window, cx| {
                            window.dispatch_action(
                                Box::new(OpenTab::selecting(TabId::Textures, target.clone())),
                                cx,
                            );
                        })
                });
            sections.push(
                section(strings.mod_bundled_textures_label())
                    .child(h_flex().flex_wrap().gap(sizes.spacing_md).children(chips))
                    .into_any_element(),
            );
        }
        if !installed.bundled_cutscenes.is_empty() {
            let chips = installed
                .bundled_cutscenes
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let target = name.clone();
                    Chip::new(name.clone())
                        .id(("mod-bundled-cutscene", index))
                        .icon(IconName::MovieCreationOutlined)
                        .tone(ChipTone::Accent)
                        .tooltip_text(strings.tooltip_open_in_cutscenes_tab())
                        .on_click(move |_, window, cx| {
                            window.dispatch_action(
                                Box::new(OpenTab::selecting(TabId::Cutscenes, target.clone())),
                                cx,
                            );
                        })
                });
            sections.push(
                section(strings.mod_bundled_cutscenes_label())
                    .child(h_flex().flex_wrap().gap(sizes.spacing_md).children(chips))
                    .into_any_element(),
            );
        }
        if let Some(requires) = self.render_requires(&installed, &sizes, cx) {
            sections.push(requires.into_any_element());
        }
        if let Some(conflicts) = self.render_conflicts(&installed, &sizes, cx) {
            sections.push(conflicts.into_any_element());
        }

        Card::new()
            .full_height()
            .padding(CardPadding::None)
            .header(
                div()
                    .id("mod-detail-name")
                    .tooltip(Tooltip::text(
                        installed.root_path.to_string_lossy().into_owned(),
                    ))
                    .child(
                        Label::new(installed.display_name.clone())
                            .size(LabelSize::Large)
                            .bold()
                            .truncate(),
                    ),
            )
            .header_end(
                Chip::new(badge.label)
                    .id("mod-detail-kind")
                    .tone(badge.tone)
                    .tooltip_text(badge.tooltip),
            )
            .child(actions)
            .child(Divider::horizontal())
            .child(
                div()
                    .id("mod-detail-scroll")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(sizes.padding_md)
                    .p(sizes.card_padding_h)
                    .children(sections)
                    .overflow_y_scrollbar(&self.detail_scroll),
            )
            .into_any_element()
    }
}
