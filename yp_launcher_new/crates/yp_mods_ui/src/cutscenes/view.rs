use std::path::PathBuf;

use gpui::{PathPromptOptions, ScrollHandle, Subscription};
use serde_json::Value as Json;
use yp_cutscenes::CutsceneMod;
use yp_state::Stores;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, Callout, Card, ChipSize, ChipTone, DropProgress, DropZone,
    ListItemSpacing, LoadingState, Msg, PageHeader, ScrollableElement, SettingSwitch, TintColor,
    Tooltip, TwoColumn,
};

use crate::cutscenes::flow::{delete_cutscene, install_from_path};
use crate::cutscenes::model::{
    Codec, CutscenesEvent, CutscenesModel, codec_of, disabled_rel_path, resolution_of,
};
use crate::layout::fill;
use crate::notify::{brief_toast, reveal};

const H264_ID: &str = "nams.cutscene.enable_h264";
const HIDE_SUBTITLE_OVERLAY_ID: &str = "nams.cutscene.hide_subtitle_overlay";
const KEEP_SUBTITLE_TEXT_ID: &str = "nams.cutscene.keep_subtitle_text";
const HIDE_SUBTITLE_IN_EVENTS_ID: &str = "nams.cutscene.hide_subtitle_in_events";
const SOLID_LETTERBOX_BARS_ID: &str = "nams.cutscene.solid_letterbox_bars";

struct TileModel {
    index: usize,
    cutscene: CutsceneMod,
    disabled: bool,
    rel_path: String,
}

pub struct CutscenesView {
    stores: Stores,
    model: Entity<CutscenesModel>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl CutscenesView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let model = cx.new(CutscenesModel::new);
        let subscriptions = vec![
            cx.observe(&model, |_, _, cx| cx.notify()),
            cx.subscribe(&model, |_, _, event: &CutscenesEvent, cx| {
                if *event == CutscenesEvent::ScanFailed {
                    brief_toast(
                        cx,
                        "cutscenes",
                        Msg::ModsUiCutsceneScanFailed,
                        Severity::Error,
                    );
                }
            }),
            cx.observe(&stores.config, |_, _, cx| cx.notify()),
            cx.observe(&stores.disabled_mods, |_, _, cx| cx.notify()),
            cx.observe(&stores.session, |_, _, cx| cx.notify()),
        ];
        Self {
            stores,
            model,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn model(&self) -> &Entity<CutscenesModel> {
        &self.model
    }

    pub fn install(&mut self, source: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.read(cx).is_installing() {
            return;
        }
        install_from_path(self.model.clone(), source, window, cx).detach();
    }

    fn browse(&mut self, files: bool, window: &mut Window, cx: &mut Context<Self>) {
        let picking = cx.prompt_for_paths(PathPromptOptions {
            files,
            directories: !files,
            multiple: false,
            prompt: (!files).then(|| cx.t().select_cutscene_mod_folder().into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = picking.await else {
                return;
            };
            let Some(source) = paths.into_iter().next() else {
                return;
            };
            this.update_in(cx, |this, window, cx| this.install(source, window, cx))
                .ok();
        })
        .detach();
    }

    fn h264_enabled(&self, cx: &App) -> bool {
        self.stores.config.read(cx).bool(H264_ID)
    }

    fn render_switch(&self, id: &'static str, setting_id: &'static str, cx: &mut Context<Self>) -> Option<AnyElement> {
        let config = self.stores.config.read(cx);
        let setting = config.setting(setting_id)?;
        let switch = SettingSwitch::new(id, setting.label.clone(), config.bool(setting_id))
            .description(setting.description.clone())
            .on_change(cx.listener(move |this, value: &bool, _, cx| {
                let value = Json::Bool(*value);
                this.stores
                    .config
                    .update(cx, |config, cx| config.set_now(setting_id, value, cx));
            }));
        Some(switch.into_any_element())
    }

    fn render_subtitle_card(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let strings = cx.t();
        let hide_overlay = self.stores.config.read(cx).bool(HIDE_SUBTITLE_OVERLAY_ID);
        Card::new()
            .title(strings.cutscene_subtitle_card())
            .children(self.render_switch("cutscene-hide-overlay", HIDE_SUBTITLE_OVERLAY_ID, cx))
            .when(hide_overlay, |this| {
                this.children(self.render_switch("cutscene-keep-text", KEEP_SUBTITLE_TEXT_ID, cx))
                    .children(self.render_switch("cutscene-hide-events", HIDE_SUBTITLE_IN_EVENTS_ID, cx))
            })
            .children(self.render_switch("cutscene-solid-bars", SOLID_LETTERBOX_BARS_ID, cx))
    }

    fn render_tile(&self, tile: TileModel, sizes: &Sizes, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let index = tile.index;
        let cutscene = tile.cutscene;
        let disabled = tile.disabled;
        let usm_count = i64::try_from(cutscene.usm_count()).unwrap_or(i64::MAX);
        let folder = cutscene.path.clone();
        let name = Link::new(("cutscene-name", index), cutscene.name.clone())
            .color(if disabled {
                Color::Muted
            } else {
                Color::Default
            })
            .hover_color(Color::Accent)
            .controller_support(false)
            .on_click(move |_, _, cx| reveal(folder.clone(), false, cx));
        let mut chips = h_flex().flex_wrap().gap(sizes.spacing_md).child(
            Chip::new(strings.cutscene_usm_count(usm_count))
                .id(("cutscene-count", index))
                .icon(IconName::CheckCircle)
                .tone(ChipTone::Success)
                .size(ChipSize::Small),
        );
        if let Some((width, height)) = resolution_of(&cutscene) {
            chips = chips.child(
                Chip::new(strings.mods_ui_resolution(i64::from(width), i64::from(height)))
                    .id(("cutscene-resolution", index))
                    .tone(ChipTone::Accent)
                    .size(ChipSize::Small),
            );
        }
        if let Some(codec) = codec_of(&cutscene) {
            let (label, tone) = match codec {
                Codec::H264 => (strings.cutscene_status_h264(), ChipTone::Success),
                Codec::Mpeg2 => (strings.mods_ui_codec_mpeg(), ChipTone::Muted),
            };
            chips = chips.child(
                Chip::new(label)
                    .id(("cutscene-codec", index))
                    .tone(tone)
                    .size(ChipSize::Small),
            );
        }
        if let Some(mod_id) = cutscene.bundled_with_mod_id.clone() {
            chips = chips.child(
                Chip::new(strings.cutscene_bundled_with(&mod_id))
                    .id(("cutscene-bundled", index))
                    .icon(IconName::ExtensionOutlined)
                    .tone(ChipTone::Accent)
                    .size(ChipSize::Small)
                    .tooltip_text(strings.tooltip_open_in_mod_manager())
                    .on_click(move |_, window, cx| {
                        window.dispatch_action(
                            Box::new(OpenTab::selecting(TabId::Mods, mod_id.clone())),
                            cx,
                        );
                    }),
            );
        }
        let rel_path = tile.rel_path;
        let toggle = div()
            .id(("cutscene-toggle-tip", index))
            .tooltip(Tooltip::text(if disabled {
                strings.mod_disabled_tooltip()
            } else {
                strings.mod_enable_tooltip()
            }))
            .child(
                Switch::new(("cutscene-toggle", index), !disabled).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.stores.disabled_mods.update(cx, |store, cx| {
                            store.set_disabled(&rel_path, !disabled, cx);
                        });
                    },
                )),
            );
        let delete = (!cutscene.is_bundled()).then(|| {
            let target = cutscene.clone();
            IconButton::new(("cutscene-delete", index), IconName::DeleteOutline)
                .size(ButtonSize::Compact)
                .style(ButtonStyle::Tinted(TintColor::Error))
                .tooltip(Tooltip::text(strings.button_delete()))
                .on_click(cx.listener(move |this, _, window, cx| {
                    delete_cutscene(this.model.clone(), target.clone(), window, cx).detach();
                }))
        });
        ListItem::new(("cutscene-tile", index))
            .spacing(ListItemSpacing::Sparse)
            .rounded(true)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .items_start()
                    .gap(sizes.spacing_md)
                    .child(name)
                    .child(chips),
            )
            .end_slot(
                h_flex()
                    .gap(sizes.spacing_md)
                    .child(toggle)
                    .children(delete),
            )
            .into_any_element()
    }
}

impl Render for CutscenesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let view = cx.entity().downgrade();
        let h264 = self.h264_enabled(cx);
        let cutscenes_dir = self
            .stores
            .session
            .read(cx)
            .game()
            .map(|game| game.cutscenes_dir());
        let (loading, scan_progress, activity, overrides, tiles) = {
            let model = self.model.read(cx);
            let disabled = self.stores.disabled_mods.read(cx);
            let tiles: Vec<TileModel> = model
                .mods()
                .iter()
                .enumerate()
                .map(|(index, cutscene)| {
                    let rel_path = disabled_rel_path(cutscene);
                    TileModel {
                        index,
                        cutscene: cutscene.clone(),
                        disabled: disabled.is_disabled(&rel_path),
                        rel_path,
                    }
                })
                .collect();
            (
                model.is_loading(),
                model.scan_progress(),
                model.activity().cloned(),
                model.direct_overrides().len(),
                tiles,
            )
        };

        let header = PageHeader::new(strings.header_cutscenes());
        let header = match cutscenes_dir {
            Some(dir) => header.info(strings.tooltip_cutscenes_location(), move |_, cx| {
                reveal(dir.clone(), false, cx);
            }),
            None => header.info_tooltip(strings.tooltip_cutscenes_location()),
        };
        let header = header.actions(
            Chip::new(strings.cutscene_status_h264())
                .id("cutscene-h264-chip")
                .icon(if h264 {
                    IconName::CheckCircle
                } else {
                    IconName::RemoveCircleOutline
                })
                .tone(if h264 {
                    ChipTone::Success
                } else {
                    ChipTone::Muted
                })
                .tooltip_text(strings.cutscene_status_h264_tooltip()),
        );

        if loading {
            let state = match scan_progress {
                Some(progress) => LoadingState::new().message(strings.mods_ui_scan_progress(
                    i64::try_from(progress.scanned).unwrap_or(i64::MAX),
                    i64::try_from(progress.total).unwrap_or(i64::MAX),
                )),
                None => LoadingState::new(),
            };
            return v_flex()
                .size_full()
                .child(header)
                .child(fill(state))
                .into_any_element();
        }

        let menu = cx.theme().id == ThemeId::NierNew;
        let explanation = strings.mods_ui_two_sentences(
            strings.cutscene_how_it_works1(), strings.cutscene_how_it_works2(),
        );
        let path = Label::new(strings.cutscene_structure_path()).size(LabelSize::XSmall).color(Color::Secondary).mono();
        let limit = Label::new(strings.cutscene_folder_name_limit()).size(LabelSize::XSmall).color(Color::Secondary);
        let (banner, guide) = if menu {
            (None, Some(CollapsibleCard::new("cutscene-install-help", strings.menu_install_help())
                .child(Label::new(explanation).size(LabelSize::Small))
                .child(path).child(limit)))
        } else {
            (Some(Callout::new().id("cutscene-banner").severity(Severity::Accent)
                .icon(IconName::InfoOutline).title(strings.cutscene_banner_title())
                .description(explanation).child(path).child(limit)), None)
        };
        let migration =
            (overrides > 0).then(|| {
                Callout::new()
                    .id("cutscene-migration")
                    .severity(Severity::Warning)
                    .title(strings.cutscene_migration_title())
                    .description(strings.cutscene_migration_banner_body(
                        i64::try_from(overrides).unwrap_or(i64::MAX),
                    ))
            });

        let drop_view = view.clone();
        let files_view = view.clone();
        let folder_view = view.clone();
        let drop_zone = DropZone::new("cutscenes-drop-zone")
            .icon(IconName::MovieCreationOutlined)
            .title(strings.drop_cutscene_here())
            .hint(strings.or_click_to_browse())
            .footer(strings.cutscene_movie_hint())
            .busy(activity.map(|activity| {
                let progress = DropProgress::new(activity.text);
                match activity.percent {
                    Some(percent) if percent > 0. => progress.percent(percent),
                    _ => progress,
                }
            }))
            .on_drop_paths(move |paths, window, cx| {
                let Some(source) = paths.first().cloned() else {
                    return;
                };
                drop_view
                    .update(cx, |this, cx| this.install(source, window, cx))
                    .ok();
            })
            .on_browse_files(move |window, cx| {
                files_view
                    .update(cx, |this, cx| this.browse(true, window, cx))
                    .ok();
            })
            .on_browse_folder(move |window, cx| {
                folder_view
                    .update(cx, |this, cx| this.browse(false, window, cx))
                    .ok();
            });

        let tile_count = i64::try_from(tiles.len()).unwrap_or(i64::MAX);
        let installed = Card::new()
            .title(strings.installed_cutscene_mods_count(tile_count))
            .map(|card| {
                if tiles.is_empty() {
                    card.child(
                        div().w_full().py(sizes.padding_lg).child(
                            Label::new(strings.no_cutscene_mods_installed())
                                .color(Color::Muted)
                                .centered(),
                        ),
                    )
                } else {
                    card.child(
                        v_flex().w_full().gap(sizes.spacing_sm).children(
                            tiles
                                .into_iter()
                                .map(|tile| self.render_tile(tile, &sizes, cx)),
                        ),
                    )
                }
            });

        let left = v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(div().child(drop_zone))
            .children(guide)
            .child(self.render_subtitle_card(cx));

        v_flex()
            .size_full()
            .child(header)
            .child(
                div()
                    .id("cutscenes-scroll")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(sizes.padding_md)
                    .p(sizes.content_padding)
                    .children(banner)
                    .children(migration)
                    .child(
                        (if cx.theme().id == ThemeId::NierNew && tile_count > 0 {
                            TwoColumn::new(installed, left).ratio(1.12, 1.)
                        } else {
                            TwoColumn::new(left, installed)
                        })
                        .id("cutscenes-columns"),
                    )
                    .overflow_y_scrollbar(&self.scroll),
            )
            .into_any_element()
    }
}
