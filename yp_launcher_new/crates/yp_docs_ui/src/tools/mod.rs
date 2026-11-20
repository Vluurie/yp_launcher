use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{
    AnyView, Context, Entity, Image, ImageFormat, ImageSource, IntoElement, ObjectFit, Render,
    ScrollHandle, SharedString, Subscription, Task, Window, div, prelude::*, px,
};
use yp_builders::misctex_scan::{self, FILE_PREFIX};
use yp_builders::{MisctexEntry, thumbnail, weapon_convert};
use yp_core::GamePaths;
use yp_core::fs::size_label;
use yp_state::SessionEvent;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    ActiveTheme, ConfirmOptions, ConfirmTone, Sizes, Thumbnail, TintColor, Tooltip, confirm,
};

use crate::builders::forms::{ConvertForm, ThumbnailChoice, inspect_weapon_model};
use crate::builders::weapon::{model_problem_text, model_summary, weapon_preview};
use crate::builders::widgets::{
    bound_input, field_label, hint, labelled_row, text_field, tint_button,
};
use crate::dialogs::icon_name::pick_icon_target;
use crate::dialogs::thumbnail::{ThumbnailOptions, pick_thumbnail};
use crate::errors::builder_error_text;
use crate::util::{clamped, current_game, error_line, pick_file, status_line};

const ICON_THUMB_WIDTH: f32 = 96.;
const ICON_THUMB_HEIGHT: f32 = 48.;
const PREVIEW_ASPECT: f32 = 16. / 9.;
const PREVIEW_MAX_WIDTH: f32 = 520.;
const MODEL_EXTENSION: &str = "dat";
const WEAPON_BUILDER_PAGE: &str = "04_weapon.md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjustTarget {
    pub directory: PathBuf,
    pub texture_name: String,
    pub file_stem: String,
}

pub fn adjust_target(path: &Path) -> Option<AdjustTarget> {
    let is_dat = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(MODEL_EXTENSION));
    let file_stem = path.file_stem()?.to_string_lossy().into_owned();
    let texture_name = file_stem.strip_prefix(FILE_PREFIX)?.to_owned();
    if !is_dat || texture_name.is_empty() {
        return None;
    }
    Some(AdjustTarget {
        directory: path.parent().map(Path::to_path_buf).unwrap_or_default(),
        texture_name,
        file_stem,
    })
}

#[derive(Clone)]
pub struct IconPreview {
    pub image: Arc<Image>,
    pub width: u32,
    pub height: u32,
}

pub struct IconListing {
    pub entries: Vec<MisctexEntry>,
    pub previews: HashMap<PathBuf, IconPreview>,
}

pub fn list_icons(game: &GamePaths) -> IconListing {
    let entries = misctex_scan::scan(game);
    let previews = entries
        .iter()
        .filter_map(|entry| {
            let preview = thumbnail::preview(&entry.dtt_path).ok()?;
            Some((
                entry.dat_path.clone(),
                IconPreview {
                    image: Arc::new(Image::from_bytes(ImageFormat::Png, preview.png)),
                    width: preview.width,
                    height: preview.height,
                },
            ))
        })
        .collect();
    IconListing { entries, previews }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertDone {
    pub stem: String,
    pub obj_id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolCard {
    Icons,
    Weapon,
}

pub struct ToolsView {
    scroll: ScrollHandle,
    icons_open: bool,
    weapon_open: bool,
    icons: Vec<MisctexEntry>,
    previews: HashMap<PathBuf, IconPreview>,
    icon_error: Option<SharedString>,
    icon_busy: bool,
    convert: ConvertForm,
    convert_mod: Entity<InputState>,
    convert_preview: Option<AnyView>,
    convert_drop_error: Option<SharedString>,
    convert_error: Option<SharedString>,
    convert_busy: bool,
    convert_done: Option<ConvertDone>,
    scan_task: Option<Task<()>>,
    icon_task: Option<Task<()>>,
    convert_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl ToolsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let convert = ConvertForm::default();
        let (convert_mod, mod_sub) = bound_input(
            "",
            &convert.mod_id,
            1,
            true,
            |this: &mut Self, value, _, _| this.convert.set_mod_id(value),
            window,
            cx,
        );
        let session = cx.stores().session.clone();
        let session_sub = cx.subscribe_in(
            &session,
            window,
            |this, _, event: &SessionEvent, _, cx| {
                if matches!(event, SessionEvent::DirectoryChanged { .. }) {
                    this.rescan(cx);
                }
            },
        );
        let mut this = Self {
            scroll: ScrollHandle::new(),
            icons_open: true,
            weapon_open: false,
            icons: Vec::new(),
            previews: HashMap::new(),
            icon_error: None,
            icon_busy: false,
            convert,
            convert_mod,
            convert_preview: None,
            convert_drop_error: None,
            convert_error: None,
            convert_busy: false,
            convert_done: None,
            scan_task: None,
            icon_task: None,
            convert_task: None,
            _subscriptions: vec![mod_sub, session_sub],
        };
        this.rescan(cx);
        this
    }

    pub fn viewport_height(&self) -> gpui::Pixels {
        self.scroll.bounds().size.height
    }

    pub fn is_card_open(&self, card: ToolCard) -> bool {
        match card {
            ToolCard::Icons => self.icons_open,
            ToolCard::Weapon => self.weapon_open,
        }
    }

    pub fn set_card_open(&mut self, card: ToolCard, open: bool, cx: &mut Context<Self>) {
        match card {
            ToolCard::Icons => self.icons_open = open,
            ToolCard::Weapon => self.weapon_open = open,
        }
        cx.notify();
    }

    pub fn icons(&self) -> &[MisctexEntry] {
        &self.icons
    }

    pub fn icon_error(&self) -> Option<&SharedString> {
        self.icon_error.as_ref()
    }

    pub fn is_icon_busy(&self) -> bool {
        self.icon_busy
    }

    pub fn convert_form(&self) -> &ConvertForm {
        &self.convert
    }

    pub fn convert_done(&self) -> Option<&ConvertDone> {
        self.convert_done.as_ref()
    }

    pub fn convert_error(&self) -> Option<&SharedString> {
        self.convert_error.as_ref()
    }

    pub fn convert_drop_error(&self) -> Option<&SharedString> {
        self.convert_drop_error.as_ref()
    }

    pub fn rescan(&mut self, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            self.icons.clear();
            self.previews.clear();
            cx.notify();
            return;
        };
        let job = cx.background_spawn(async move { list_icons(&game) });
        self.scan_task = Some(cx.spawn(async move |this, cx| {
            let listing = job.await;
            this.update(cx, |this, cx| {
                this.icons = listing.entries;
                this.previews = listing.previews;
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn write_icon(
        &mut self,
        choice: ThumbnailChoice,
        directory: PathBuf,
        texture_name: String,
        cx: &mut Context<Self>,
    ) {
        self.icon_busy = true;
        self.icon_error = None;
        cx.notify();
        let job = cx.background_spawn(async move {
            thumbnail::create_named(
                &choice.image_path,
                &directory,
                &texture_name,
                choice.width,
                choice.height,
            )
        });
        self.icon_task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| {
                this.icon_busy = false;
                if let Err(error) = result {
                    this.icon_error = Some(SharedString::from(builder_error_text(&error, cx.t())));
                }
                this.rescan(cx);
                cx.notify();
            })
            .ok();
        }));
    }

    fn add_icon(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            return;
        };
        let naming = pick_icon_target(game, window, cx);
        self.icon_task = Some(cx.spawn_in(window, async move |this, cx| {
            let Some(target) = naming.await else {
                return;
            };
            let subtitle = format!(
                "{FILE_PREFIX}{}  \u{b7}  {}",
                target.texture_name, target.mod_id
            );
            let Ok(picking) = cx.update(|window, cx| {
                pick_thumbnail(ThumbnailOptions::new(subtitle), window, cx)
            }) else {
                return;
            };
            let Some(choice) = picking.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.write_icon(choice, target.directory, target.texture_name, cx);
            })
            .ok();
        }));
    }

    fn adjust_icon(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = pick_file(cx);
        self.icon_task = Some(cx.spawn_in(window, async move |this, cx| {
            let Some(path) = picked.await else {
                return;
            };
            let Some(target) = adjust_target(&path) else {
                this.update(cx, |this, cx| {
                    this.icon_error =
                        Some(SharedString::new_static(cx.t().tools_icons_not_misctex()));
                    cx.notify();
                })
                .ok();
                return;
            };
            let subtitle = format!("{}.dat / .dtt", target.file_stem);
            let Ok(picking) = cx.update(|window, cx| {
                pick_thumbnail(ThumbnailOptions::new(subtitle), window, cx)
            }) else {
                return;
            };
            let Some(choice) = picking.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.write_icon(choice, target.directory, target.texture_name, cx);
            })
            .ok();
        }));
    }

    fn replace_icon(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.icons.get(index).cloned() else {
            return;
        };
        let mut options = ThumbnailOptions::new(format!("{}.dat / .dtt", entry.file_stem()));
        if let Some(preview) = self.previews.get(&entry.dat_path) {
            options = options.size(preview.width, preview.height);
        }
        let picking = pick_thumbnail(options, window, cx);
        self.icon_task = Some(cx.spawn(async move |this, cx| {
            let Some(choice) = picking.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.write_icon(
                    choice,
                    entry.directory().to_path_buf(),
                    entry.texture_name.clone(),
                    cx,
                );
            })
            .ok();
        }));
    }

    fn delete_icon(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.icons.get(index).cloned() else {
            return;
        };
        let strings = cx.t();
        let asking = confirm(
            ConfirmOptions::new(
                strings.tools_icons_delete(),
                strings.tools_icons_delete_confirm(&entry.file_stem()),
            )
            .confirm_label(strings.tools_icons_delete())
            .tone(ConfirmTone::Danger),
            window,
            cx,
        );
        self.icon_task = Some(cx.spawn(async move |this, cx| {
            if !asking.await {
                return;
            }
            let job = cx.background_spawn(async move { misctex_scan::remove(&entry) });
            let result = job.await;
            this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.icon_error = Some(SharedString::from(builder_error_text(&error, cx.t())));
                }
                this.rescan(cx);
            })
            .ok();
        }));
    }

    pub fn accept_weapon(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            return;
        };
        let path = path.to_path_buf();
        let job = cx.background_spawn(async move {
            inspect_weapon_model(&path).map(|source| (source, weapon_convert::taken_obj_ids(&game)))
        });
        self.convert_task = Some(cx.spawn_in(window, async move |this, cx| {
            let inspected = job.await;
            this.update_in(cx, |this, window, cx| {
                this.convert_done = None;
                this.convert_error = None;
                this.weapon_open = true;
                match inspected {
                    Ok((source, taken)) => {
                        this.convert_preview = Some(weapon_preview(&source, window, cx));
                        this.convert.accept(source, taken);
                        this.convert_drop_error = None;
                    }
                    Err(problem) => {
                        this.convert_drop_error = Some(SharedString::new_static(
                            model_problem_text(problem, cx.t()),
                        ));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn browse_weapon(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = pick_file(cx);
        self.convert_task = Some(cx.spawn_in(window, async move |this, cx| {
            if let Some(path) = picked.await {
                this.update_in(cx, |this, window, cx| this.accept_weapon(&path, window, cx))
                    .ok();
            }
        }));
    }

    pub fn run_convert(&mut self, cx: &mut Context<Self>) {
        let (Some(game), Some(source), Some(target)) = (
            current_game(cx),
            self.convert.source.clone(),
            self.convert.target_obj_id,
        ) else {
            if self.convert.source.is_some() {
                self.convert_error =
                    Some(SharedString::new_static(cx.t().weapon_builder_no_free_slot()));
                cx.notify();
            }
            return;
        };
        let mod_id = self.convert.mod_id().to_owned();
        self.convert_busy = true;
        self.convert_error = None;
        self.convert_done = None;
        cx.notify();
        let job = cx.background_spawn(async move {
            let converted = weapon_convert::convert(&source, target, &game, &mod_id);
            (converted, weapon_convert::taken_obj_ids(&game))
        });
        self.convert_task = Some(cx.spawn(async move |this, cx| {
            let (converted, taken) = job.await;
            this.update(cx, |this, cx| {
                this.convert_busy = false;
                match converted {
                    Ok(renamed) => {
                        this.convert_done = Some(ConvertDone {
                            stem: renamed.to_stem,
                            obj_id: target,
                        });
                        if let Some(source) = this.convert.source.clone() {
                            this.convert.accept(source, taken);
                        }
                    }
                    Err(error) => {
                        this.convert_error =
                            Some(SharedString::from(builder_error_text(&error, cx.t())));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn icon_row(
        &self,
        index: usize,
        entry: &MisctexEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = cx.theme().colors;
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let preview = self.previews.get(&entry.dat_path);
        let detail = match preview {
            Some(preview) => format!(
                "{}  \u{b7}  {}\u{d7}{}  \u{b7}  {}",
                entry.mod_id,
                preview.width,
                preview.height,
                size_label(entry.total_bytes)
            ),
            None => format!("{}  \u{b7}  {}", entry.mod_id, size_label(entry.total_bytes)),
        };
        let thumb = match preview {
            Some(preview) => Thumbnail::new(
                SharedString::from(format!("tools-icon-thumb-{index}")),
                ImageSource::Image(preview.image.clone()),
            )
            .size(px(ICON_THUMB_WIDTH), px(ICON_THUMB_HEIGHT))
            .fit(ObjectFit::Contain)
            .lightbox(true)
            .into_any_element(),
            None => div()
                .flex_none()
                .w(px(ICON_THUMB_WIDTH))
                .h(px(ICON_THUMB_HEIGHT))
                .flex()
                .items_center()
                .justify_center()
                .rounded(Sizes::ROW_RADIUS)
                .border_1()
                .border_color(colors.border_light)
                .child(
                    Icon::new(IconName::BrokenImageOutlined)
                        .size(IconSize::Medium)
                        .color(Color::Muted),
                )
                .into_any_element(),
        };
        h_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .py(sizes.spacing_md)
            .border_t_1()
            .border_color(colors.border_light)
            .child(thumb)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        Label::new(SharedString::from(entry.file_stem()))
                            .size(LabelSize::Small)
                            .mono()
                            .truncate(),
                    )
                    .child(
                        Label::new(SharedString::from(detail))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .truncate(),
                    ),
            )
            .child(
                tint_button(
                    SharedString::from(format!("tools-icon-replace-{index}")),
                    strings.tools_icons_replace(),
                    TintColor::Accent,
                )
                .disabled(self.icon_busy)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.replace_icon(index, window, cx);
                })),
            )
            .child(
                tint_button(
                    SharedString::from(format!("tools-icon-delete-{index}")),
                    strings.tools_icons_delete(),
                    TintColor::Error,
                )
                .disabled(self.icon_busy)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.delete_icon(index, window, cx);
                })),
            )
    }

    fn icons_card(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let rows: Vec<_> = self
            .icons
            .iter()
            .enumerate()
            .map(|(index, entry)| self.icon_row(index, entry, window, cx).into_any_element())
            .collect();
        let summary = if self.icons.is_empty() {
            hint(strings.tools_icons_empty()).into_any_element()
        } else {
            field_label(SharedString::from(
                strings.tools_icons_count(self.icons.len() as i64),
            ))
            .into_any_element()
        };
        CollapsibleCard::new("tools-icons", strings.tools_icons_title())
            .open(self.icons_open)
            .on_toggle(cx.listener(|this, open: &bool, _, cx| {
                this.set_card_open(ToolCard::Icons, *open, cx);
            }))
            .header_end(
                IconButton::new("tools-icons-refresh", IconName::Refresh)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Muted)
                    .tooltip(Tooltip::text(strings.docs_reload()))
                    .on_click(cx.listener(|this, _, _, cx| this.rescan(cx))),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(
                        h_flex()
                            .w_full()
                            .items_start()
                            .gap(sizes.spacing_md)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(hint(strings.tools_icons_desc())),
                            )
                            .child(
                                div()
                                    .id("tools-icons-hot-reload")
                                    .flex_none()
                                    .tooltip(Tooltip::text(strings.tools_icons_hot_reload()))
                                    .child(
                                        Icon::new(IconName::BoltOutlined)
                                            .size(IconSize::Small)
                                            .color(Color::Success),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap(sizes.spacing_lg)
                            .child(
                                tint_button(
                                    "tools-icons-convert",
                                    strings.tools_icons_convert(),
                                    TintColor::Accent,
                                )
                                .loading(self.icon_busy)
                                .disabled(self.icon_busy)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.add_icon(window, cx);
                                })),
                            )
                            .child(
                                tint_button(
                                    "tools-icons-adjust",
                                    strings.tools_icons_adjust(),
                                    TintColor::Accent,
                                )
                                .disabled(self.icon_busy)
                                .controller_support(false)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.adjust_icon(window, cx);
                                })),
                            ),
                    )
                    .children(self.icon_error.clone().map(error_line))
                    .child(v_flex().w_full().child(summary).children(rows)),
            )
    }

    fn weapon_card(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let entity = cx.entity();
        let browse_entity = entity.clone();
        let details = self.convert.source.as_ref().map(|source| {
            let preview = div()
                .relative()
                .w_full()
                .max_w(px(PREVIEW_MAX_WIDTH))
                .aspect_ratio(PREVIEW_ASPECT)
                .child(div().absolute().inset_0().children(self.convert_preview.clone()));
            let slot = self
                .convert
                .target_label()
                .map(SharedString::from)
                .unwrap_or_else(|| SharedString::new_static(strings.weapon_builder_no_free_slot()));
            let done = self.convert_done.as_ref().map(|done| {
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_sm)
                    .child(status_line(
                        IconName::CheckCircleOutline,
                        Color::Success,
                        strings.weapon_convert_done(&done.stem),
                        window,
                        cx,
                    ))
                    .child(hint(SharedString::from(
                        strings.tools_weapon_convert_next_step(&format!("0x{:X}", done.obj_id)),
                    )))
                    .child(
                        h_flex().child(
                            Button::new(
                                "tools-open-weapon-builder",
                                strings.tools_open_weapon_builder(),
                            )
                            .style(ButtonStyle::Outlined)
                            .size(ButtonSize::Compact)
                            .end_icon(IconName::MenuBookOutlined)
                            .on_click(|_, window, cx| {
                                window.dispatch_action(
                                    Box::new(OpenDocs {
                                        page: Some(WEAPON_BUILDER_PAGE.to_owned()),
                                    }),
                                    cx,
                                );
                            }),
                        ),
                    )
            });
            v_flex()
                .w_full()
                .gap(sizes.spacing_lg)
                .child(model_summary(
                    &source.stem,
                    source.weapon_type,
                    strings.weapon_convert_detected(),
                    window,
                    cx,
                ))
                .child(preview)
                .child(clamped(
                    px(PREVIEW_MAX_WIDTH),
                    v_flex()
                        .w_full()
                        .gap(sizes.spacing_md)
                        .child(text_field(
                            strings.outfit_builder_mod_id(),
                            &self.convert_mod,
                            window,
                            cx,
                        ))
                        .child(labelled_row(
                            strings.weapon_convert_slot(),
                            Color::Secondary,
                            h_flex()
                                .gap(sizes.spacing_lg)
                                .child(
                                    Label::new(slot)
                                        .size(LabelSize::Small)
                                        .color(Color::Accent)
                                        .semibold()
                                        .mono(),
                                )
                                .child(
                                    tint_button(
                                        "tools-weapon-next-slot",
                                        strings.weapon_convert_next_slot(),
                                        TintColor::Muted,
                                    )
                                    .disabled(self.convert_busy)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.convert.next_slot();
                                        cx.notify();
                                    })),
                                ),
                            window,
                            cx,
                        )),
                ))
                .child(
                    h_flex().child(
                        tint_button(
                            "tools-weapon-run",
                            strings.weapon_convert_run(),
                            TintColor::Success,
                        )
                        .loading(self.convert_busy)
                        .disabled(self.convert_busy || self.convert.target_obj_id.is_none())
                        .on_click(cx.listener(|this, _, _, cx| this.run_convert(cx))),
                    ),
                )
                .children(self.convert_error.clone().map(error_line))
                .children(done)
        });
        CollapsibleCard::new(
            "tools-weapon",
            SharedString::from(strings.weapon_convert_title().to_uppercase()),
        )
        .open(self.weapon_open)
        .on_toggle(cx.listener(|this, open: &bool, _, cx| {
            this.set_card_open(ToolCard::Weapon, *open, cx);
        }))
        .child(
            v_flex()
                .w_full()
                .gap(sizes.spacing_lg)
                .child(hint(strings.weapon_convert_desc()))
                .child(
                    DropZone::new("tools-weapon-drop")
                        .title(strings.weapon_convert_drop_title())
                        .hint(strings.weapon_convert_drop_hint())
                        .icon(IconName::FileDownloadOutlined)
                        .on_drop_paths(move |paths, window, cx| {
                            if let Some(path) = paths.first().cloned() {
                                entity.update(cx, |this, cx| this.accept_weapon(&path, window, cx));
                            }
                        })
                        .on_browse_files(move |window, cx| {
                            browse_entity.update(cx, |this, cx| this.browse_weapon(window, cx));
                        }),
                )
                .children(self.convert_drop_error.clone().map(error_line))
                .children(details),
        )
    }

}

impl Render for ToolsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let body = if current_game(cx).is_none() {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(sizes.content_padding)
                .child(
                    Label::new(strings.tools_needs_game_dir())
                        .size(LabelSize::Small)
                        .color(Color::Warning)
                        .centered(),
                )
                .into_any_element()
        } else {
            v_flex()
                .id("tools-content")
                .size_full()
                .gap(sizes.spacing_lg)
                .p(sizes.content_padding)
                .child(self.icons_card(window, cx))
                .child(self.weapon_card(window, cx))
                .overflow_y_scrollbar(&self.scroll)
                .into_any_element()
        };
        v_flex()
            .size_full()
            .bg(colors.background_primary)
            .child(PageHeader::new(SharedString::from(
                strings.tab_tools().to_uppercase(),
            )))
            .child(fill_parent(body))
    }
}
