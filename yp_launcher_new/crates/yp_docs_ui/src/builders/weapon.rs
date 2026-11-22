use std::path::{Path, PathBuf};

use gpui::{
    AnyView, App, Context, Entity, IntoElement, Render, SharedString, Subscription, Task, Window,
    div, prelude::*, px,
};
use yp_builders::weapon::{self, EffectPath, WHEN_PLAYERS};
use yp_builders::{WeaponSource, WeaponType, thumbnail};
use yp_i18n::Strings;
use yp_state::inspector::ArchivePaths;
use yp_state::prelude::*;
use yp_state::{ModelPreviewRequest, SessionEvent};
use yp_ui::prelude::*;
use yp_ui::{ChipTone, SelectItem, TintColor};

use super::forms::{
    ModelProblem, ShopChoice, WeaponForm, WeaponProblem, create_weapon, inspect_weapon_model,
    step_value, weapon_ids,
};
use super::model_view::ModelView;
use super::widgets::{
    VALUE_WIDTH, WIDE_VALUE_WIDTH, bound_input, click, create_row, hint, labelled_row,
    preview_caption, read_only_field, remove_button, section_title, shop_select, stepper,
    switch_row, text_field, texture_row, tint_button, toml_preview,
};
use crate::dialogs::thumbnail::{ThumbnailOptions, pick_thumbnail};
use crate::errors::builder_error_text;
use crate::util::{current_game, error_line, open_folder, path_text, pick_file, status_line};

const DASH: &str = "-";
const EFFECT_SELECT_WIDTH: f32 = 340.;
const WHEN_SELECT_WIDTH: f32 = 180.;
const LEVEL_LABEL_WIDTH: f32 = 72.;
const UPGRADE_LABEL_WIDTH: f32 = 104.;
const PREVIEW_ASPECT: f32 = 16. / 9.;

pub fn weapon_type_label(weapon_type: WeaponType, strings: &Strings) -> &'static str {
    match weapon_type {
        WeaponType::SmallSword => strings.weapon_type_small_sword(),
        WeaponType::LargeSword => strings.weapon_type_large_sword(),
        WeaponType::Spear => strings.weapon_type_spear(),
        WeaponType::CombatBracer => strings.weapon_type_combat_bracer(),
    }
}

pub fn model_problem_text(problem: ModelProblem, strings: &Strings) -> &'static str {
    match problem {
        ModelProblem::NeedsBothFiles => strings.weapon_convert_needs_both_files(),
        ModelProblem::NotAModelFile | ModelProblem::UnknownModel => {
            strings.weapon_convert_drop_invalid()
        }
    }
}

pub(crate) fn weapon_preview(
    source: &WeaponSource,
    window: &mut Window,
    cx: &mut App,
) -> AnyView {
    let archives = ArchivePaths::new(Some(source.dat_path.clone()), Some(source.dtt_path.clone()));
    if let Some(view) = Registry::model_preview(ModelPreviewRequest { archives }, window, cx) {
        return view;
    }
    let failure = cx.t().weapon_preview_failed();
    let path = source.dat_path.clone();
    cx.new(|cx| ModelView::new(path, cx).failure_text(failure))
        .into()
}

pub(crate) fn model_summary(
    stem: &str,
    weapon_type: WeaponType,
    note: &'static str,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let sizes = window.sizes(cx);
    h_flex()
        .w_full()
        .gap(sizes.spacing_lg)
        .child(
            Icon::new(IconName::Hardware)
                .size(IconSize::Medium)
                .color(Color::Success),
        )
        .child(
            Label::new(SharedString::from(stem.to_owned()))
                .size(LabelSize::Small)
                .color(Color::Success)
                .semibold()
                .mono(),
        )
        .child(Chip::new(weapon_type_label(weapon_type, cx.t())).tone(ChipTone::Accent))
        .child(v_flex().flex_1().min_w_0().child(hint(note)))
}

pub struct WeaponBuilder {
    form: WeaponForm,
    name: Entity<InputState>,
    description: Entity<InputState>,
    mod_id: Entity<InputState>,
    help_short: Entity<InputState>,
    story: Entity<InputState>,
    skill1_name: Entity<InputState>,
    skill1_desc: Entity<InputState>,
    skill2_name: Entity<InputState>,
    skill2_desc: Entity<InputState>,
    preview: Option<AnyView>,
    drop_error: Option<SharedString>,
    written: Option<PathBuf>,
    error: Option<SharedString>,
    busy: bool,
    create_task: Option<Task<()>>,
    ids_task: Option<Task<()>>,
    pick_task: Option<Task<()>>,
    texture_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl WeaponBuilder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let strings = cx.t();
        let form = WeaponForm::default();
        let mut subscriptions = Vec::new();
        let mut input = |placeholder: &'static str,
                         initial: &str,
                         rows: usize,
                         mono: bool,
                         apply: fn(&mut WeaponForm, &str),
                         window: &mut Window,
                         cx: &mut Context<Self>| {
            let (state, subscription) = bound_input(
                placeholder,
                initial,
                rows,
                mono,
                move |this: &mut Self, value, _, _| apply(&mut this.form, value),
                window,
                cx,
            );
            subscriptions.push(subscription);
            state
        };
        let name = input(
            strings.weapon_builder_name_hint(),
            "",
            1,
            false,
            |form, value| form.name = value.to_owned(),
            window,
            cx,
        );
        let description = input(
            strings.weapon_builder_description_hint(),
            "",
            2,
            false,
            |form, value| form.description = value.to_owned(),
            window,
            cx,
        );
        let help_short = input(
            strings.weapon_builder_help_short_hint(),
            "",
            2,
            false,
            |form, value| form.help_short = value.to_owned(),
            window,
            cx,
        );
        let story = input(
            strings.weapon_builder_story_hint(),
            "",
            3,
            false,
            |form, value| form.story = value.to_owned(),
            window,
            cx,
        );
        let skill1_name = input(
            strings.weapon_builder_skill_name_hint(),
            "",
            1,
            false,
            |form, value| form.skill1_name = value.to_owned(),
            window,
            cx,
        );
        let skill1_desc = input(
            strings.weapon_builder_skill_desc_hint(),
            "",
            2,
            false,
            |form, value| form.skill1_desc = value.to_owned(),
            window,
            cx,
        );
        let skill2_name = input(
            strings.weapon_builder_skill_name_hint(),
            "",
            1,
            false,
            |form, value| form.skill2_name = value.to_owned(),
            window,
            cx,
        );
        let skill2_desc = input(
            strings.weapon_builder_skill_desc_hint(),
            "",
            2,
            false,
            |form, value| form.skill2_desc = value.to_owned(),
            window,
            cx,
        );
        let (mod_id, mod_sub) = bound_input(
            "",
            &form.mod_id,
            1,
            true,
            |this: &mut Self, value, window, cx| {
                this.form.mod_id = value.to_owned();
                this.pick_ids(window, cx);
            },
            window,
            cx,
        );
        subscriptions.push(mod_sub);
        let session = cx.stores().session.clone();
        subscriptions.push(cx.subscribe_in(
            &session,
            window,
            |this, _, event: &SessionEvent, window, cx| {
                if matches!(event, SessionEvent::DirectoryChanged { .. }) {
                    this.pick_ids(window, cx);
                }
            },
        ));
        let mut this = Self {
            form,
            name,
            description,
            mod_id,
            help_short,
            story,
            skill1_name,
            skill1_desc,
            skill2_name,
            skill2_desc,
            preview: None,
            drop_error: None,
            written: None,
            error: None,
            busy: false,
            create_task: None,
            ids_task: None,
            pick_task: None,
            texture_task: None,
            _subscriptions: subscriptions,
        };
        this.pick_ids(window, cx);
        this
    }

    pub fn form(&self) -> &WeaponForm {
        &self.form
    }

    pub fn form_mut(&mut self) -> &mut WeaponForm {
        &mut self.form
    }

    pub fn set_name(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = name.to_owned();
        self.name
            .update(cx, |input, cx| input.set_value(value, window, cx));
        self.form.name = name.to_owned();
        cx.notify();
    }

    pub fn set_description(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = text.to_owned();
        self.description
            .update(cx, |input, cx| input.set_value(value, window, cx));
        self.form.description = text.to_owned();
        cx.notify();
    }

    pub fn error(&self) -> Option<&SharedString> {
        self.error.as_ref()
    }

    pub fn drop_error(&self) -> Option<&SharedString> {
        self.drop_error.as_ref()
    }

    pub fn written(&self) -> Option<&PathBuf> {
        self.written.as_ref()
    }

    pub fn pick_ids(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            return;
        };
        let seed = self.form.seed().to_owned();
        let job = cx.background_spawn(async move { weapon_ids(&game, &seed) });
        self.ids_task = Some(cx.spawn_in(window, async move |this, cx| {
            let ids = job.await;
            this.update(cx, |this, cx| {
                this.form.apply_ids(ids);
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn accept_model(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let path = path.to_path_buf();
        let job = cx.background_spawn(async move { inspect_weapon_model(&path) });
        self.pick_task = Some(cx.spawn_in(window, async move |this, cx| {
            let inspected = job.await;
            this.update_in(cx, |this, window, cx| {
                match inspected {
                    Ok(source) => {
                        this.preview = Some(weapon_preview(&source, window, cx));
                        this.form.accept_model(source);
                        this.drop_error = None;
                    }
                    Err(problem) => {
                        this.drop_error = Some(SharedString::new_static(model_problem_text(
                            problem,
                            cx.t(),
                        )));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn browse_model(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = pick_file(cx);
        self.pick_task = Some(cx.spawn_in(window, async move |this, cx| {
            if let Some(path) = picked.await {
                this.update_in(cx, |this, window, cx| this.accept_model(&path, window, cx))
                    .ok();
            }
        }));
    }

    pub fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        let game = current_game(cx);
        if let Some(problem) = self.form.problem(game.is_some()) {
            self.error = Some(SharedString::new_static(match problem {
                WeaponProblem::NeedsGameDir => strings.weapon_builder_needs_game_dir(),
                WeaponProblem::NeedsName => strings.weapon_builder_needs_name(),
                WeaponProblem::NeedsModel => strings.weapon_builder_needs_model(),
                WeaponProblem::NoFreeSlot => strings.weapon_builder_no_free_slot(),
            }));
            cx.notify();
            return;
        }
        let Some(game) = game else {
            return;
        };
        self.busy = true;
        self.error = None;
        self.written = None;
        cx.notify();
        let form = self.form.clone();
        let job = cx.background_spawn(async move { create_weapon(&game, &form) });
        self.create_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = job.await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(path) => {
                        this.written = Some(path);
                        this.form.icon = None;
                        this.form.story_image = None;
                        this.pick_ids(window, cx);
                    }
                    Err(error) => {
                        this.error = Some(SharedString::from(builder_error_text(&error, cx.t())));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn pick_texture(&mut self, story: bool, window: &mut Window, cx: &mut Context<Self>) {
        let name = if story {
            self.form.story_texture_name()
        } else {
            self.form.icon_texture_name()
        };
        let mut options = ThumbnailOptions::new(format!("misctex_{name}.dat / .dtt")).initial(
            if story {
                self.form.story_image.clone()
            } else {
                self.form.icon.clone()
            },
        );
        if story {
            options = options.fixed_size(thumbnail::STORY_WIDTH, thumbnail::STORY_HEIGHT);
        }
        let picking = pick_thumbnail(options, window, cx);
        self.texture_task = Some(cx.spawn(async move |this, cx| {
            if let Some(choice) = picking.await {
                this.update(cx, |this, cx| {
                    if story {
                        this.form.story_image = Some(choice);
                    } else {
                        this.form.icon = Some(choice);
                    }
                    cx.notify();
                })
                .ok();
            }
        }));
    }

    fn open_written_folder(&mut self, cx: &mut Context<Self>) {
        if let Some(folder) = self.written.as_deref().and_then(Path::parent) {
            open_folder(folder, cx);
        }
    }

    fn model_zone(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let entity = cx.entity();
        let browse_entity = entity.clone();
        let selected = self.form.model.as_ref().map(|model| {
            model_summary(
                &model.stem,
                model.weapon_type,
                strings.weapon_builder_model_ready(),
                window,
                cx,
            )
        });
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                DropZone::new("weapon-model-drop")
                    .title(strings.weapon_builder_drop_title())
                    .hint(strings.weapon_builder_drop_hint())
                    .icon(IconName::FileDownloadOutlined)
                    .on_drop_paths(move |paths, window, cx| {
                        if let Some(path) = paths.first().cloned() {
                            entity.update(cx, |this, cx| this.accept_model(&path, window, cx));
                        }
                    })
                    .on_browse_files(move |window, cx| {
                        browse_entity.update(cx, |this, cx| this.browse_model(window, cx));
                    }),
            )
            .children(self.drop_error.clone().map(error_line))
            .children(selected)
    }

    fn levels(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let removable = self.form.levels.len() > 1;
        let rows = self.form.levels.iter().enumerate().map(|(index, level)| {
            let min_stepper = stepper(
                SharedString::from(format!("weapon-level-min-{index}")),
                level.min_damage.to_string(),
                px(VALUE_WIDTH),
                (level.min_damage > 1).then(|| {
                    click(cx, move |this: &mut Self, _, _| {
                        this.form.step_min_damage(index, false);
                    })
                }),
                Some(click(cx, move |this: &mut Self, _, _| {
                    this.form.step_min_damage(index, true);
                })),
                cx,
            );
            let max_stepper = stepper(
                SharedString::from(format!("weapon-level-max-{index}")),
                level.max_damage.to_string(),
                px(VALUE_WIDTH),
                (level.max_damage > level.min_damage).then(|| {
                    click(cx, move |this: &mut Self, _, _| {
                        this.form.step_max_damage(index, false);
                    })
                }),
                Some(click(cx, move |this: &mut Self, _, _| {
                    this.form.step_max_damage(index, true);
                })),
                cx,
            );
            h_flex()
                .w_full()
                .gap(sizes.spacing_md)
                .child(
                    v_flex().flex_none().w(px(LEVEL_LABEL_WIDTH)).child(hint(SharedString::from(
                        strings.weapon_builder_level_number(index as i64 + 1),
                    ))),
                )
                .child(v_flex().flex_1().min_w_0().child(
                    Label::new(strings.weapon_builder_damage())
                        .size(LabelSize::XSmall)
                        .color(Color::Secondary),
                ))
                .child(min_stepper)
                .child(Label::new(DASH).size(LabelSize::Small).color(Color::Muted))
                .child(max_stepper)
                .when(removable, |this| {
                    this.child(remove_button(
                        SharedString::from(format!("weapon-level-remove-{index}")),
                        click(cx, move |this: &mut Self, _, _| {
                            this.form.remove_level(index);
                        }),
                        cx,
                    ))
                })
        });
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                h_flex()
                    .w_full()
                    .child(
                        v_flex()
                            .flex_1()
                            .child(section_title(strings.weapon_builder_levels())),
                    )
                    .when(self.form.can_add_level(), |this| {
                        this.child(
                            tint_button(
                                "weapon-add-level",
                                strings.weapon_builder_add_level(),
                                TintColor::Accent,
                            )
                            .on_click(click(cx, |this: &mut Self, _, _| this.form.add_level())),
                        )
                    }),
            )
            .children(rows)
    }

    fn details(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let skill_name = |slot: i64| SharedString::from(strings.weapon_builder_skill_name(slot));
        let skill_desc = |slot: i64| SharedString::from(strings.weapon_builder_skill_desc(slot));
        v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(section_title(strings.weapon_builder_details()))
            .child(text_field(
                strings.weapon_builder_help_short(),
                &self.help_short,
                window,
                cx,
            ))
            .child(text_field(
                strings.weapon_builder_story(),
                &self.story,
                window,
                cx,
            ))
            .child(text_field(skill_name(1), &self.skill1_name, window, cx))
            .when(!self.form.skill1_name.trim().is_empty(), |this| {
                this.child(text_field(skill_desc(1), &self.skill1_desc, window, cx))
            })
            .child(text_field(skill_name(2), &self.skill2_name, window, cx))
            .when(!self.form.skill2_name.trim().is_empty(), |this| {
                this.child(text_field(skill_desc(2), &self.skill2_desc, window, cx))
            })
    }

    fn textures(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let strings = cx.t();
        let note = |name: String| SharedString::from(format!("misctex_{name}"));
        v_flex()
            .w_full()
            .child(texture_row(
                "weapon-icon",
                strings.weapon_builder_icon(),
                strings.weapon_builder_icon_hint(),
                self.form
                    .icon
                    .as_ref()
                    .map(|icon| (icon, note(self.form.icon_texture_name()))),
                cx.listener(|this, _, window, cx| this.pick_texture(false, window, cx)),
                click(cx, |this: &mut Self, _, _| this.form.icon = None),
                window,
                cx,
            ))
            .child(texture_row(
                "weapon-story",
                strings.weapon_builder_story_icon(),
                strings.weapon_builder_story_icon_hint(),
                self.form
                    .story_image
                    .as_ref()
                    .map(|story| (story, note(self.form.story_texture_name()))),
                cx.listener(|this, _, window, cx| this.pick_texture(true, window, cx)),
                click(cx, |this: &mut Self, _, _| this.form.story_image = None),
                window,
                cx,
            ))
    }

    fn upgrades(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let last = self.form.upgrades.len().saturating_sub(1);
        let rows = self
            .form
            .upgrades
            .iter()
            .enumerate()
            .map(|(index, upgrade)| {
                let cost = stepper(
                    SharedString::from(format!("weapon-upgrade-{index}")),
                    upgrade.cost.to_string(),
                    px(WIDE_VALUE_WIDTH),
                    (upgrade.cost > 500).then(|| {
                        click(cx, move |this: &mut Self, _, _| {
                            this.form.step_upgrade_cost(index, false);
                        })
                    }),
                    Some(click(cx, move |this: &mut Self, _, _| {
                        this.form.step_upgrade_cost(index, true);
                    })),
                    cx,
                );
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(v_flex().flex_none().w(px(UPGRADE_LABEL_WIDTH)).child(hint(
                        SharedString::from(strings.weapon_builder_upgrade_level(upgrade.lv)),
                    )))
                    .child(
                        v_flex().flex_1().min_w_0().child(
                            Label::new(strings.weapon_builder_upgrade_cost())
                                .size(LabelSize::XSmall)
                                .color(Color::Secondary),
                        ),
                    )
                    .child(cost)
                    .child(
                        div()
                            .flex_none()
                            .when(index != last, |this| this.invisible())
                            .child(remove_button(
                                SharedString::from(format!("weapon-upgrade-remove-{index}")),
                                click(cx, |this: &mut Self, _, _| this.form.remove_last_upgrade()),
                                cx,
                            )),
                    )
            })
            .collect::<Vec<_>>();
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                h_flex()
                    .w_full()
                    .child(
                        v_flex()
                            .flex_1()
                            .child(section_title(strings.weapon_builder_upgrades())),
                    )
                    .when(self.form.can_add_upgrade(), |this| {
                        this.child(
                            tint_button(
                                "weapon-add-upgrade",
                                strings.weapon_builder_add_upgrade(),
                                TintColor::Accent,
                            )
                            .on_click(click(cx, |this: &mut Self, _, _| this.form.add_upgrade())),
                        )
                    }),
            )
            .when(rows.is_empty(), |this| {
                this.child(hint(strings.weapon_builder_upgrade_hint()))
            })
            .children(rows)
    }

    fn est_row(
        &self,
        berserk: bool,
        label: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let value = self.form.est_id(berserk);
        let id = if berserk {
            "weapon-est-berserk"
        } else {
            "weapon-est-normal"
        };
        let control = stepper(
            id,
            value.to_string(),
            px(VALUE_WIDTH),
            (value > weapon::MIN_EST_ID).then(|| {
                click(cx, move |this: &mut Self, _, _| {
                    this.form.step_est_id(berserk, false);
                })
            }),
            (value < weapon::MAX_EST_ID).then(|| {
                click(cx, move |this: &mut Self, _, _| {
                    this.form.step_est_id(berserk, true);
                })
            }),
            cx,
        );
        let sizes = window.sizes(cx);
        h_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(
                v_flex().flex_1().min_w_0().child(
                    Label::new(label)
                        .size(LabelSize::XSmall)
                        .color(Color::Secondary),
                ),
            )
            .child(control)
    }

    fn effect_row(
        &self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let effect = &self.form.effects[index];
        let entity = cx.entity();
        let path_entity = entity.clone();
        let when_entity = entity.clone();
        let path_select = Select::new(
            SharedString::from(format!("weapon-effect-path-{index}")),
            effect.path.path,
        )
        .items(
            self.form
                .effect_choices(index)
                .into_iter()
                .map(|path| SelectItem::new(path.path, path)),
        )
        .min_width(px(EFFECT_SELECT_WIDTH))
        .max_width(px(EFFECT_SELECT_WIDTH))
        .searchable(true)
        .on_select(move |path: &EffectPath, _, cx| {
            let path = *path;
            path_entity.update(cx, |this, cx| {
                this.form.set_effect_path(index, path);
                cx.notify();
            });
        });
        let any = strings.weapon_builder_effect_when_any();
        let when_label = effect
            .when_player
            .clone()
            .map(SharedString::from)
            .unwrap_or_else(|| SharedString::new_static(any));
        let when_select = Select::new(
            SharedString::from(format!("weapon-effect-when-{index}")),
            when_label,
        )
        .items(
            std::iter::once(SelectItem::new(any, None::<String>)).chain(
                WHEN_PLAYERS
                    .iter()
                    .map(|player| SelectItem::new(*player, Some((*player).to_owned()))),
            ),
        )
        .min_width(px(WHEN_SELECT_WIDTH))
        .max_width(px(WHEN_SELECT_WIDTH))
        .on_select(move |player: &Option<String>, _, cx| {
            let player = player.clone();
            when_entity.update(cx, |this, cx| {
                this.form.set_effect_when(index, player);
                cx.notify();
            });
        });
        let values = if effect.path.per_level {
            effect
                .levels
                .iter()
                .map(|(level, value)| {
                    let level = *level;
                    h_flex()
                        .w_full()
                        .gap(sizes.spacing_md)
                        .child(v_flex().flex_1().min_w_0().child(hint(SharedString::from(
                            strings.weapon_builder_level_number(level),
                        ))))
                        .child(stepper(
                            SharedString::from(format!("weapon-effect-{index}-level-{level}")),
                            format!("{value:.2}"),
                            px(VALUE_WIDTH),
                            Some(click(cx, move |this: &mut Self, _, _| {
                                this.form.step_effect_level(index, level, false);
                            })),
                            Some(click(cx, move |this: &mut Self, _, _| {
                                this.form.step_effect_level(index, level, true);
                            })),
                            cx,
                        ))
                        .into_any_element()
                })
                .collect::<Vec<_>>()
        } else {
            let entity = entity.clone();
            vec![
                switch_row(
                    SharedString::from(format!("weapon-effect-on-{index}")),
                    strings.weapon_builder_effect_on(),
                    None,
                    effect.value != 0.,
                    move |on, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.form.set_effect_on(index, on);
                            cx.notify();
                        });
                    },
                    window,
                    cx,
                )
                .into_any_element(),
            ]
        };
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(v_flex().flex_1().min_w_0().child(path_select))
                    .child(remove_button(
                        SharedString::from(format!("weapon-effect-remove-{index}")),
                        click(cx, move |this: &mut Self, _, _| {
                            this.form.remove_effect(index);
                        }),
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_sm)
                    .pl(sizes.padding_md)
                    .children(values)
                    .child(labelled_row(
                        strings.weapon_builder_effect_when(),
                        Color::Secondary,
                        when_select,
                        window,
                        cx,
                    )),
            )
    }

    fn effects(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let rows: Vec<_> = (0..self.form.effects.len())
            .map(|index| self.effect_row(index, window, cx).into_any_element())
            .collect();
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                h_flex()
                    .w_full()
                    .child(
                        v_flex()
                            .flex_1()
                            .child(section_title(strings.weapon_builder_effects())),
                    )
                    .when(!self.form.free_effect_paths().is_empty(), |this| {
                        this.child(
                            tint_button(
                                "weapon-add-effect",
                                strings.weapon_builder_add_effect(),
                                TintColor::Accent,
                            )
                            .on_click(click(cx, |this: &mut Self, _, _| {
                                this.form.add_effect();
                            })),
                        )
                    }),
            )
            .child(self.est_row(false, strings.weapon_builder_trail_normal(), window, cx))
            .child(self.est_row(true, strings.weapon_builder_trail_berserk(), window, cx))
            .child(hint(strings.weapon_builder_trail_hint()))
            .when(rows.is_empty(), |this| {
                this.child(hint(strings.weapon_builder_effects_hint()))
            })
            .children(rows)
    }

    fn shop(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let entity = cx.entity();
        let shop = &self.form.shop;
        let toggle_entity = entity.clone();
        let fields = shop.enabled.then(|| {
            let price = shop.buy_price;
            let (price_min, price_max) = ShopChoice::PRICE_RANGE;
            let price_stepper = stepper(
                "weapon-buy-price",
                price.to_string(),
                px(WIDE_VALUE_WIDTH),
                (price > ShopChoice::PRICE_STEP).then(|| {
                    click(cx, move |this: &mut Self, _, _| {
                        this.form.shop.buy_price = step_value(
                            this.form.shop.buy_price,
                            -ShopChoice::PRICE_STEP,
                            price_min,
                            price_max,
                        );
                    })
                }),
                Some(click(cx, move |this: &mut Self, _, _| {
                    this.form.shop.buy_price = step_value(
                        this.form.shop.buy_price,
                        ShopChoice::PRICE_STEP,
                        price_min,
                        price_max,
                    );
                })),
                cx,
            );
            v_flex()
                .w_full()
                .pl(sizes.padding_lg)
                .child(labelled_row(
                    strings.outfit_builder_shop_name(),
                    Color::Secondary,
                    shop_select("weapon-shop", shop, move |name: &String, _, cx| {
                        let name = name.clone();
                        entity.update(cx, |this, cx| {
                            this.form.shop.shop_name = name;
                            cx.notify();
                        });
                    }),
                    window,
                    cx,
                ))
                .child(labelled_row(
                    strings.outfit_builder_buy_price(),
                    Color::Secondary,
                    price_stepper,
                    window,
                    cx,
                ))
        });
        v_flex()
            .w_full()
            .child(switch_row(
                "weapon-in-shop",
                strings.outfit_builder_in_shop(),
                Some(strings.outfit_builder_in_shop_hint()),
                shop.enabled,
                move |on, _, cx| {
                    toggle_entity.update(cx, |this, cx| {
                        this.form.shop.enabled = on;
                        cx.notify();
                    });
                },
                window,
                cx,
            ))
            .children(fields)
    }

    fn advanced(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let dash = SharedString::new_static(DASH);
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                h_flex()
                    .w_full()
                    .items_start()
                    .gap(sizes.spacing_lg)
                    .child(v_flex().flex_1().min_w_0().child(text_field(
                        strings.outfit_builder_mod_id(),
                        &self.mod_id,
                        window,
                        cx,
                    )))
                    .child(v_flex().flex_1().min_w_0().child(read_only_field(
                        strings.weapon_builder_item_id(),
                        self.form
                            .id
                            .map(|id| SharedString::from(id.to_string()))
                            .unwrap_or_else(|| dash.clone()),
                        window,
                        cx,
                    )))
                    .child(v_flex().flex_1().min_w_0().child(read_only_field(
                        strings.weapon_builder_obj_id(),
                        self.form
                            .obj_id_label()
                            .map(SharedString::from)
                            .unwrap_or(dash),
                        window,
                        cx,
                    ))),
            )
            .child(hint(strings.weapon_builder_obj_id_hint()))
    }
}

impl Render for WeaponBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let zone = self.model_zone(window, cx).into_any_element();
        if self.form.model.is_none() {
            return v_flex().w_full().gap(sizes.padding_md).child(zone);
        }
        let has_game = current_game(cx).is_some();
        let draft = self.form.draft();
        let preview_frame = div()
            .relative()
            .w_full()
            .aspect_ratio(PREVIEW_ASPECT)
            .child(div().absolute().inset_0().children(self.preview.clone()));
        let written = self.written.as_ref().map(|path| {
            status_line(
                IconName::CheckCircleOutline,
                Color::Success,
                strings.outfit_builder_created(&path_text(path)),
                window,
                cx,
            )
        });
        let file_preview = toml_preview(
            "weapon-preview",
            weapon::file_name_for(&draft),
            &weapon::to_toml(&draft),
            window,
            cx,
        );
        v_flex()
            .w_full()
            .gap(sizes.padding_md)
            .child(zone)
            .child(preview_frame)
            .child(text_field(strings.weapon_builder_name(), &self.name, window, cx))
            .child(text_field(
                strings.weapon_builder_description(),
                &self.description,
                window,
                cx,
            ))
            .child(self.levels(window, cx))
            .child(self.details(window, cx))
            .when(has_game, |this| this.child(self.textures(window, cx)))
            .child(self.upgrades(window, cx))
            .child(self.effects(window, cx))
            .child(self.shop(window, cx))
            .child(self.advanced(window, cx))
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(preview_caption(strings.outfit_builder_preview(), window, cx))
                    .child(file_preview),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(create_row(
                        "weapon",
                        strings.weapon_builder_create(),
                        strings.outfit_builder_open_folder(),
                        self.busy,
                        self.written.is_some(),
                        cx.listener(|this, _, window, cx| this.create(window, cx)),
                        cx.listener(|this, _, _, cx| this.open_written_folder(cx)),
                        window,
                        cx,
                    ))
                    .children(self.error.clone().map(error_line))
                    .children(written),
            )
    }
}

pub(crate) fn build(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| WeaponBuilder::new(window, cx)).into()
}
