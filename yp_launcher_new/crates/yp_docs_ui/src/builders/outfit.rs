use std::path::{Path, PathBuf};

use gpui::{
    App, Context, Entity, IntoElement, Render, SharedString, Subscription, Task, Window,
    prelude::*, px,
};
use yp_builders::{item, outfit};
use yp_state::SessionEvent;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::ChipTone;

use super::forms::{
    CreateFailure, ModelProblem, OutfitForm, OutfitProblem, ShopChoice, create_outfit,
    digits_only, outfit_ids, step_value,
};
use super::mesh_tuner::{MeshTuner, MeshTunerEvent};
use super::widgets::{
    ID_FIELD_WIDTH, WIDE_VALUE_WIDTH, bound_input, click, create_row, hint, labelled_row,
    preview_caption, section_title, shop_select, shop_states, stepper, switch_row, text_field,
    toml_preview,
};
use crate::errors::builder_error_text;
use crate::util::{current_game, error_line, open_folder, path_text, pick_file, status_line};

pub struct OutfitBuilder {
    form: OutfitForm,
    name: Entity<InputState>,
    description: Entity<InputState>,
    mod_id: Entity<InputState>,
    outfit_id: Entity<InputState>,
    item_id: Entity<InputState>,
    tuner: Option<Entity<MeshTuner>>,
    drop_error: Option<SharedString>,
    written: Option<PathBuf>,
    error: Option<SharedString>,
    busy: bool,
    create_task: Option<Task<()>>,
    ids_task: Option<Task<()>>,
    pick_task: Option<Task<()>>,
    tuner_subscription: Option<Subscription>,
    _subscriptions: Vec<Subscription>,
}

impl OutfitBuilder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let strings = cx.t();
        let form = OutfitForm::default();
        let (name, name_sub) = bound_input(
            strings.outfit_builder_name_hint(),
            "",
            1,
            false,
            |this: &mut Self, value, _, _| this.form.name = value.to_owned(),
            window,
            cx,
        );
        let (description, description_sub) = bound_input(
            strings.outfit_builder_description_hint(),
            "",
            2,
            false,
            |this: &mut Self, value, _, _| this.form.description = value.to_owned(),
            window,
            cx,
        );
        let (mod_id, mod_sub) = bound_input(
            "",
            &form.mod_id,
            1,
            true,
            |this: &mut Self, value, _, _| this.form.mod_id = value.to_owned(),
            window,
            cx,
        );
        let (outfit_id, outfit_sub) = bound_input(
            "",
            &form.outfit_id.to_string(),
            1,
            true,
            |this: &mut Self, value, window, cx| {
                let digits = digits_only(value);
                if digits != value {
                    let filtered = digits.clone();
                    this.outfit_id
                        .update(cx, |input, cx| input.set_value(filtered, window, cx));
                }
                if let Ok(id) = digits.parse() {
                    this.form.outfit_id = id;
                }
            },
            window,
            cx,
        );
        let (item_id, item_sub) = bound_input(
            "",
            &form.item_id.to_string(),
            1,
            true,
            |this: &mut Self, value, window, cx| {
                let digits = digits_only(value);
                if digits != value {
                    let filtered = digits.clone();
                    this.item_id
                        .update(cx, |input, cx| input.set_value(filtered, window, cx));
                }
                if let Ok(id) = digits.parse() {
                    this.form.item_id = id;
                }
            },
            window,
            cx,
        );
        let session = cx.stores().session.clone();
        let session_sub = cx.subscribe_in(
            &session,
            window,
            |this, _, event: &SessionEvent, window, cx| {
                if matches!(event, SessionEvent::DirectoryChanged { .. }) {
                    this.pick_ids(window, cx);
                }
            },
        );
        let mut this = Self {
            form,
            name,
            description,
            mod_id,
            outfit_id,
            item_id,
            tuner: None,
            drop_error: None,
            written: None,
            error: None,
            busy: false,
            create_task: None,
            ids_task: None,
            pick_task: None,
            tuner_subscription: None,
            _subscriptions: vec![
                name_sub,
                description_sub,
                mod_sub,
                outfit_sub,
                item_sub,
                session_sub,
            ],
        };
        this.pick_ids(window, cx);
        this
    }

    pub fn form(&self) -> &OutfitForm {
        &self.form
    }

    pub fn form_mut(&mut self) -> &mut OutfitForm {
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

    pub fn tuner(&self) -> Option<&Entity<MeshTuner>> {
        self.tuner.as_ref()
    }

    pub fn pick_ids(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            return;
        };
        let character = self.form.character;
        let job = cx.background_spawn(async move { outfit_ids(&game, character) });
        self.ids_task = Some(cx.spawn_in(window, async move |this, cx| {
            let ids = job.await;
            this.update_in(cx, |this, window, cx| {
                this.form.apply_ids(ids);
                let outfit_id = this.form.outfit_id.to_string();
                let item_id = this.form.item_id.to_string();
                this.outfit_id
                    .update(cx, |input, cx| input.set_value(outfit_id, window, cx));
                this.item_id
                    .update(cx, |input, cx| input.set_value(item_id, window, cx));
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn accept_model(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        match self.form.accept_model(path) {
            Ok(character) => {
                self.drop_error = None;
                let model = path.to_path_buf();
                let tuner = cx.new(|cx| MeshTuner::new(model, character, cx));
                self.tuner_subscription = Some(cx.subscribe(
                    &tuner,
                    |this, tuner, _: &MeshTunerEvent, cx| {
                        this.form.mesh_rules = tuner.read(cx).rules().to_vec();
                        cx.notify();
                    },
                ));
                self.tuner = Some(tuner);
                self.pick_ids(window, cx);
            }
            Err(problem) => {
                self.drop_error = Some(SharedString::new_static(match problem {
                    ModelProblem::UnknownModel => strings.outfit_builder_drop_unknown_model(),
                    ModelProblem::NotAModelFile | ModelProblem::NeedsBothFiles => {
                        strings.outfit_builder_drop_invalid()
                    }
                }));
            }
        }
        cx.notify();
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
                OutfitProblem::NeedsGameDir => strings.outfit_builder_needs_game_dir(),
                OutfitProblem::NeedsName => strings.outfit_builder_needs_name(),
                OutfitProblem::NeedsModel => strings.outfit_builder_needs_model(),
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
        let job = cx.background_spawn(async move { create_outfit(&game, &form) });
        self.create_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = job.await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(path) => {
                        this.written = Some(path);
                        this.pick_ids(window, cx);
                    }
                    Err(CreateFailure::IdTaken(taken)) => {
                        this.form.taken_outfit_ids = taken;
                        this.error =
                            Some(SharedString::new_static(cx.t().outfit_builder_id_taken()));
                    }
                    Err(CreateFailure::Build(error)) => {
                        this.error = Some(SharedString::from(builder_error_text(&error, cx.t())));
                    }
                }
                cx.notify();
            })
            .ok();
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
        let selected = self.form.model_stem().map(|stem| {
            h_flex()
                .w_full()
                .gap(sizes.spacing_lg)
                .child(
                    Icon::new(IconName::ViewInAr)
                        .size(IconSize::Medium)
                        .color(Color::Success),
                )
                .child(
                    Label::new(SharedString::from(stem))
                        .size(LabelSize::Small)
                        .color(Color::Success)
                        .semibold()
                        .mono(),
                )
                .child(Chip::new(self.form.character.toml_value()).tone(ChipTone::Accent))
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(hint(strings.outfit_builder_character_detected())),
                )
        });
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                DropZone::new("outfit-model-drop")
                    .title(strings.outfit_builder_drop_title())
                    .hint(strings.outfit_builder_drop_hint())
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

    fn acquisition(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let entity = cx.entity();
        let shop = &self.form.shop;
        let auto_give = {
            let entity = entity.clone();
            switch_row(
                "outfit-auto-give",
                strings.outfit_builder_auto_give(),
                Some(strings.outfit_builder_auto_give_hint()),
                self.form.auto_give,
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.form.auto_give = on;
                        cx.notify();
                    });
                },
                window,
                cx,
            )
        };
        let in_shop = {
            let entity = entity.clone();
            switch_row(
                "outfit-in-shop",
                strings.outfit_builder_in_shop(),
                Some(strings.outfit_builder_in_shop_hint()),
                shop.enabled,
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.form.shop.enabled = on;
                        cx.notify();
                    });
                },
                window,
                cx,
            )
        };
        let shop_fields = shop.enabled.then(|| {
            let (price_min, price_max) = ShopChoice::PRICE_RANGE;
            let price = shop.buy_price;
            let select_entity = entity.clone();
            let toggle_entity = entity.clone();
            let price_stepper = stepper(
                "outfit-buy-price",
                price.to_string(),
                px(WIDE_VALUE_WIDTH),
                (price > price_min).then(|| {
                    click(cx, move |this: &mut Self, _, _| {
                        this.form.shop.buy_price = step_value(
                            this.form.shop.buy_price,
                            -ShopChoice::PRICE_STEP,
                            price_min,
                            price_max,
                        );
                    })
                }),
                (price < price_max).then(|| {
                    click(cx, move |this: &mut Self, _, _| {
                        this.form.shop.buy_price = step_value(
                            this.form.shop.buy_price,
                            ShopChoice::PRICE_STEP,
                            price_min,
                            price_max,
                        );
                    })
                }),
                cx,
            );
            v_flex()
                .w_full()
                .gap(sizes.spacing_md)
                .pl(sizes.padding_lg)
                .child(labelled_row(
                    strings.outfit_builder_shop_name(),
                    Color::Secondary,
                    shop_select("outfit-shop", shop, move |name: &String, _, cx| {
                        let name = name.clone();
                        select_entity.update(cx, |this, cx| {
                            this.form.shop.shop_name = name;
                            cx.notify();
                        });
                    }),
                    window,
                    cx,
                ))
                .child(shop_states(
                    "outfit-shop",
                    shop,
                    move |state, _, cx| {
                        toggle_entity.update(cx, |this, cx| {
                            this.form.shop.toggle_state(state);
                            cx.notify();
                        });
                    },
                    cx.listener(|this, _, _, cx| {
                        this.form.shop.toggle_all_states();
                        cx.notify();
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
            .child(section_title(strings.outfit_builder_acquisition_title()))
            .child(auto_give)
            .child(in_shop)
            .children(shop_fields)
    }

    fn advanced(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let entity = cx.entity();
        let id_hint = if self.form.id_taken() {
            Label::new(strings.outfit_builder_id_taken())
                .size(LabelSize::XSmall)
                .color(Color::Warning)
                .line_height(1.4)
                .into_any_element()
        } else {
            hint(strings.outfit_builder_id_hint()).into_any_element()
        };
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
                    .child(v_flex().flex_none().w(px(ID_FIELD_WIDTH)).child(text_field(
                        strings.outfit_builder_outfit_id(),
                        &self.outfit_id,
                        window,
                        cx,
                    )))
                    .child(v_flex().flex_none().w(px(ID_FIELD_WIDTH)).child(text_field(
                        strings.outfit_builder_item_id(),
                        &self.item_id,
                        window,
                        cx,
                    ))),
            )
            .child(id_hint)
            .child(switch_row(
                "outfit-copy-model",
                strings.outfit_builder_copy_model(),
                Some(strings.outfit_builder_copy_model_hint()),
                self.form.copy_model,
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.form.copy_model = on;
                        cx.notify();
                    });
                },
                window,
                cx,
            ))
    }

    fn preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let draft = self.form.draft();
        let item_draft = self.form.item_draft();
        let shop = draft.sells_in_shop().then(|| {
            toml_preview(
                "outfit-preview-shop",
                outfit::shop_file_name_for(&draft),
                &outfit::shop_item_toml(&draft),
                window,
                cx,
            )
        });
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(preview_caption(strings.outfit_builder_preview(), window, cx))
            .child(toml_preview(
                "outfit-preview-outfit",
                outfit::file_name_for(&draft),
                &outfit::to_toml(&draft),
                window,
                cx,
            ))
            .child(toml_preview(
                "outfit-preview-item",
                item::file_name_for(&item_draft),
                &item::to_toml(&item_draft),
                window,
                cx,
            ))
            .children(shop)
    }
}

impl Render for OutfitBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let written = self.written.as_ref().map(|path| {
            status_line(
                IconName::CheckCircleOutline,
                Color::Success,
                strings.outfit_builder_created(&path_text(path)),
                window,
                cx,
            )
        });
        v_flex()
            .w_full()
            .gap(sizes.padding_md)
            .child(self.model_zone(window, cx))
            .child(text_field(strings.outfit_builder_name(), &self.name, window, cx))
            .child(text_field(
                strings.outfit_builder_description(),
                &self.description,
                window,
                cx,
            ))
            .children(self.tuner.clone())
            .child(self.acquisition(window, cx))
            .child(self.advanced(window, cx))
            .child(self.preview(window, cx))
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(create_row(
                        "outfit",
                        strings.outfit_builder_create(),
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

pub(crate) fn build(window: &mut Window, cx: &mut App) -> gpui::AnyView {
    cx.new(|cx| OutfitBuilder::new(window, cx)).into()
}
