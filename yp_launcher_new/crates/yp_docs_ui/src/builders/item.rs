use std::path::PathBuf;

use gpui::{
    App, Context, Entity, IntoElement, Render, SharedString, Subscription, Task, Window,
    prelude::*, px,
};
use yp_builders::item;
use yp_state::SessionEvent;
use yp_state::prelude::*;
use yp_ui::prelude::*;

use super::forms::{CreateFailure, ItemForm, ItemProblem, create_item, digits_only, item_ids, step_value};
use super::widgets::{
    ID_FIELD_WIDTH, VALUE_WIDTH, bound_input, click, create_row, hint, labelled_row, preview_caption, stepper,
    switch_row, text_field, texture_row, toml_preview,
};
use crate::dialogs::thumbnail::{ThumbnailOptions, pick_thumbnail};
use crate::errors::builder_error_text;
use crate::util::{current_game, error_line, path_text, reveal_file, status_line};

pub struct ItemBuilder {
    form: ItemForm,
    name: Entity<InputState>,
    description: Entity<InputState>,
    mod_id: Entity<InputState>,
    id: Entity<InputState>,
    written: Option<PathBuf>,
    error: Option<SharedString>,
    busy: bool,
    create_task: Option<Task<()>>,
    ids_task: Option<Task<()>>,
    icon_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl ItemBuilder {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let strings = cx.t();
        let form = ItemForm::default();
        let (name, name_sub) = bound_input(
            strings.item_builder_name_hint(),
            "",
            1,
            false,
            |this: &mut Self, value, _, _| this.form.name = value.to_owned(),
            window,
            cx,
        );
        let (description, description_sub) = bound_input(
            strings.item_builder_description_hint(),
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
        let (id, id_sub) = bound_input(
            "",
            &form.id.to_string(),
            1,
            true,
            |this: &mut Self, value, window, cx| {
                let digits = digits_only(value);
                if digits != value {
                    let filtered = digits.clone();
                    this.id
                        .update(cx, |input, cx| input.set_value(filtered, window, cx));
                }
                this.form.set_id_text(&digits);
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
                    this.pick_id(window, cx);
                }
            },
        );
        let mut this = Self {
            form,
            name,
            description,
            mod_id,
            id,
            written: None,
            error: None,
            busy: false,
            create_task: None,
            ids_task: None,
            icon_task: None,
            _subscriptions: vec![name_sub, description_sub, mod_sub, id_sub, session_sub],
        };
        this.pick_id(window, cx);
        this
    }

    pub fn form(&self) -> &ItemForm {
        &self.form
    }

    pub fn form_mut(&mut self) -> &mut ItemForm {
        &mut self.form
    }

    pub fn name_input(&self) -> &Entity<InputState> {
        &self.name
    }

    pub fn id_input(&self) -> &Entity<InputState> {
        &self.id
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

    pub fn written(&self) -> Option<&PathBuf> {
        self.written.as_ref()
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    pub fn pick_id(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            return;
        };
        let job = cx.background_spawn(async move { item_ids(&game) });
        self.ids_task = Some(cx.spawn_in(window, async move |this, cx| {
            let ids = job.await;
            this.update_in(cx, |this, window, cx| {
                this.form.apply_ids(ids);
                let text = this.form.id.to_string();
                this.id
                    .update(cx, |input, cx| input.set_value(text, window, cx));
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        let game = current_game(cx);
        if let Some(problem) = self.form.problem(game.is_some()) {
            self.error = Some(SharedString::new_static(match problem {
                ItemProblem::NeedsGameDir => strings.item_builder_needs_game_dir(),
                ItemProblem::NeedsName => strings.item_builder_needs_name(),
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
        let job = cx.background_spawn(async move { create_item(&game, &form) });
        self.create_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = job.await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(path) => {
                        this.written = Some(path);
                        this.form.icon = None;
                        this.pick_id(window, cx);
                    }
                    Err(CreateFailure::IdTaken(taken)) => {
                        this.form.taken_ids = taken;
                        this.error =
                            Some(SharedString::new_static(cx.t().item_builder_exists()));
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

    fn pick_icon(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let options =
            ThumbnailOptions::new(self.form.icon_subtitle()).initial(self.form.icon.clone());
        let picking = pick_thumbnail(options, window, cx);
        self.icon_task = Some(cx.spawn(async move |this, cx| {
            if let Some(choice) = picking.await {
                this.update(cx, |this, cx| {
                    this.form.icon = Some(choice);
                    cx.notify();
                })
                .ok();
            }
        }));
    }

    fn show_file(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = &self.written {
            reveal_file(path, cx);
        }
    }
}

impl Render for ItemBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let has_game = current_game(cx).is_some();
        let form = &self.form;
        let draft = form.draft();
        let (carry_min, carry_max) = ItemForm::CARRY_RANGE;
        let (sell_min, sell_max) = ItemForm::SELL_RANGE;
        let carry = form.carry;
        let sell_price = form.sell_price;
        let carry_stepper = stepper(
            "item-carry",
            carry.to_string(),
            px(VALUE_WIDTH),
            (carry > carry_min).then(|| {
                click(cx, move |this: &mut Self, _, _| {
                    this.form.carry = step_value(this.form.carry, -1, carry_min, carry_max);
                })
            }),
            (carry < carry_max).then(|| {
                click(cx, move |this: &mut Self, _, _| {
                    this.form.carry = step_value(this.form.carry, 1, carry_min, carry_max);
                })
            }),
            cx,
        );
        let sell_row = form.sellable.then(|| {
            let control = stepper(
                "item-sell-price",
                sell_price.to_string(),
                px(VALUE_WIDTH),
                (sell_price > sell_min).then(|| {
                    click(cx, move |this: &mut Self, _, _| {
                        this.form.sell_price = step_value(
                            this.form.sell_price,
                            -ItemForm::SELL_STEP,
                            sell_min,
                            sell_max,
                        );
                    })
                }),
                (sell_price < sell_max).then(|| {
                    click(cx, move |this: &mut Self, _, _| {
                        this.form.sell_price = step_value(
                            this.form.sell_price,
                            ItemForm::SELL_STEP,
                            sell_min,
                            sell_max,
                        );
                    })
                }),
                cx,
            );
            v_flex().w_full().pl(sizes.padding_lg).child(labelled_row(
                strings.item_builder_sell_price(),
                Color::Secondary,
                control,
                window,
                cx,
            ))
        });
        let entity = cx.entity();
        let auto_give = {
            let entity = entity.clone();
            switch_row(
                "item-auto-give",
                strings.item_builder_auto_give(),
                Some(strings.item_builder_auto_give_hint()),
                form.auto_give,
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
        let sellable = {
            let entity = entity.clone();
            switch_row(
                "item-sellable",
                strings.item_builder_sellable(),
                None,
                form.sellable,
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.form.sellable = on;
                        cx.notify();
                    });
                },
                window,
                cx,
            )
        };
        let icon_row = has_game.then(|| {
            texture_row(
                "item-icon",
                strings.thumbnail_title(),
                strings.thumbnail_drop_hint(),
                form.icon
                    .as_ref()
                    .map(|icon| (icon, SharedString::new_static(strings.thumbnail_pending()))),
                cx.listener(|this, _, window, cx| this.pick_icon(window, cx)),
                cx.listener(|this, _, _, cx| {
                    this.form.icon = None;
                    cx.notify();
                }),
                window,
                cx,
            )
        });
        let id_hint = if form.id_taken() {
            Label::new(strings.item_builder_exists())
                .size(LabelSize::XSmall)
                .color(Color::Warning)
                .line_height(1.4)
                .into_any_element()
        } else {
            hint(strings.item_builder_id_hint()).into_any_element()
        };
        let preview = toml_preview(
            "item-preview",
            item::file_name_for(&draft),
            &item::to_toml(&draft),
            window,
            cx,
        );
        let written = self.written.as_ref().map(|path| {
            status_line(
                IconName::CheckCircleOutline,
                Color::Success,
                strings.item_builder_created(&path_text(path)),
                window,
                cx,
            )
        });
        v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(text_field(strings.item_builder_name(), &self.name, window, cx))
            .child(text_field(
                strings.item_builder_description(),
                &self.description,
                window,
                cx,
            ))
            .child(
                v_flex()
                    .w_full()
                    .child(labelled_row(
                        strings.item_builder_carry(),
                        Color::Default,
                        carry_stepper,
                        window,
                        cx,
                    ))
                    .child(auto_give)
                    .child(sellable)
                    .children(sell_row)
                    .children(icon_row),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(
                        h_flex()
                            .w_full()
                            .items_start()
                            .gap(sizes.spacing_lg)
                            .child(v_flex().flex_1().min_w_0().child(text_field(
                                strings.item_builder_mod_id(),
                                &self.mod_id,
                                window,
                                cx,
                            )))
                            .child(v_flex().flex_none().w(px(ID_FIELD_WIDTH)).child(text_field(
                                strings.item_builder_id_label(),
                                &self.id,
                                window,
                                cx,
                            ))),
                    )
                    .child(id_hint),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(preview_caption(strings.item_builder_preview(), window, cx))
                    .child(preview),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(create_row(
                        "item",
                        strings.item_builder_create(),
                        strings.item_builder_open_folder(),
                        self.busy,
                        self.written.is_some(),
                        cx.listener(|this, _, window, cx| this.create(window, cx)),
                        cx.listener(|this, _, _, cx| this.show_file(cx)),
                        window,
                        cx,
                    ))
                    .children(self.error.clone().map(error_line))
                    .children(written),
            )
    }
}

pub(crate) fn build(window: &mut Window, cx: &mut App) -> gpui::AnyView {
    cx.new(|cx| ItemBuilder::new(window, cx)).into()
}
