use std::path::PathBuf;

use futures::channel::oneshot;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render,
    SharedString, Subscription, Task, Window, prelude::*, px,
};
use yp_builders::misctex_scan::FILE_PREFIX;
use yp_builders::thumbnail;
use yp_core::GamePaths;
use yp_ui::prelude::*;
use yp_ui::{ButtonStyle, InputEvent, ModalView, SelectItem, TintColor};

use crate::builders::forms::{is_texture_name, texture_name_only};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconTarget {
    pub mod_id: String,
    pub texture_name: String,
    pub directory: PathBuf,
}

pub fn list_mod_folders(game: &GamePaths) -> Vec<String> {
    let mut ids: Vec<String> = std::fs::read_dir(game.mods_dir())
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    ids.sort();
    ids
}

pub fn icon_file_label(texture_name: &str) -> String {
    format!("{FILE_PREFIX}{texture_name}.dat / .dtt")
}

pub type IconChoice = (String, String);

pub struct IconNameDialog {
    mod_ids: Vec<String>,
    mod_id: String,
    name: Entity<InputState>,
    sender: Option<oneshot::Sender<Option<IconChoice>>>,
    _subscriptions: Vec<Subscription>,
}

impl IconNameDialog {
    fn new(
        mod_ids: Vec<String>,
        sender: oneshot::Sender<Option<IconChoice>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let placeholder = cx.t().tools_icons_new_name_hint();
        let name = cx.new(|cx| InputState::new(window, cx).mono().placeholder(placeholder));
        let subscriptions = vec![cx.subscribe_in(
            &name,
            window,
            |this, input, event: &InputEvent, window, cx| match event {
                InputEvent::Change => {
                    let typed = input.read(cx).value().to_owned();
                    let filtered = texture_name_only(&typed);
                    if filtered != typed {
                        input.update(cx, |input, cx| input.set_value(filtered, window, cx));
                    }
                    cx.notify();
                }
                InputEvent::Submit => this.confirm(cx),
                InputEvent::Focus | InputEvent::Blur => {}
            },
        )];
        window.focus(&name.focus_handle(cx), cx);
        Self {
            mod_id: mod_ids.first().cloned().unwrap_or_default(),
            mod_ids,
            name,
            sender: Some(sender),
            _subscriptions: subscriptions,
        }
    }

    fn texture_name(&self, cx: &App) -> String {
        self.name.read(cx).value().trim().to_owned()
    }

    pub fn is_valid(&self, cx: &App) -> bool {
        !self.mod_id.is_empty() && is_texture_name(&self.texture_name(cx))
    }

    pub fn select_mod(&mut self, mod_id: &str, cx: &mut Context<Self>) {
        if self.mod_ids.iter().any(|known| known == mod_id) {
            self.mod_id = mod_id.to_owned();
            cx.notify();
        }
    }

    fn finish(&mut self, result: Option<IconChoice>, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(result);
        }
        cx.emit(DismissEvent);
    }

    pub fn confirm(&mut self, cx: &mut Context<Self>) {
        if self.is_valid(cx) {
            let result = (self.mod_id.clone(), self.texture_name(cx));
            self.finish(Some(result), cx);
        }
    }
}

impl EventEmitter<DismissEvent> for IconNameDialog {}

impl Focusable for IconNameDialog {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.name.read(cx).focus_handle(cx)
    }
}

impl ModalView for IconNameDialog {
    fn on_before_dismiss(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(None);
        }
        true
    }
}

impl Render for IconNameDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let valid = self.is_valid(cx);
        let name = self.texture_name(cx);
        let field_label = |text: &'static str| {
            Label::new(text)
                .size(LabelSize::XSmall)
                .color(Color::Secondary)
        };
        let mod_field = if self.mod_ids.is_empty() {
            Label::new(strings.tools_icons_new_no_mods())
                .size(LabelSize::XSmall)
                .color(Color::Warning)
                .line_height(1.4)
                .into_any_element()
        } else {
            let entity = cx.entity();
            Select::new("icon-name-mod", SharedString::from(self.mod_id.clone()))
                .items(
                    self.mod_ids
                        .iter()
                        .map(|id| SelectItem::new(SharedString::from(id.clone()), id.clone())),
                )
                .full_width()
                .menu_matches_trigger_width(true)
                .on_select(move |mod_id: &String, _, cx| {
                    entity.update(cx, |this, cx| this.select_mod(mod_id, cx));
                })
                .into_any_element()
        };
        let hint = if name.is_empty() {
            Label::new(strings.tools_icons_new_hint())
                .size(LabelSize::XSmall)
                .color(Color::Muted)
        } else {
            Label::new(icon_file_label(&name))
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .mono()
        };
        Modal::new("icon-name-dialog", None)
            .width(ModalWidth::Custom(px(460.)))
            .header(ModalHeader::new().headline(strings.tools_icons_new_title()))
            .section(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_sm)
                    .child(field_label(strings.tools_icons_new_mod()))
                    .child(mod_field),
            )
            .section(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_sm)
                    .child(field_label(strings.tools_icons_new_name()))
                    .child(Input::new(&self.name).size(InputSize::Small))
                    .child(hint),
            )
            .footer(
                ModalFooter::new().end_slot(
                    h_flex()
                        .gap(sizes.spacing_md)
                        .child(
                            Button::new("icon-name-cancel", strings.cancel_button())
                                .style(ButtonStyle::Tinted(TintColor::Muted))
                                .on_click(cx.listener(|this, _, _, cx| this.finish(None, cx))),
                        )
                        .child(
                            Button::new("icon-name-continue", strings.tools_icons_new_create())
                                .style(ButtonStyle::Tinted(TintColor::Accent))
                                .disabled(!valid)
                                .on_click(cx.listener(|this, _, _, cx| this.confirm(cx))),
                        ),
                ),
            )
    }
}

pub fn pick_icon_target(
    game: GamePaths,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<IconTarget>> {
    let listing_game = game.clone();
    let listing = cx.background_spawn(async move { list_mod_folders(&listing_game) });
    window.spawn(cx, async move |cx| {
        let mod_ids = listing.await;
        let (sender, receiver) = oneshot::channel();
        cx.update(|window, cx| {
            window.open_modal(cx, |window, cx| IconNameDialog::new(mod_ids, sender, window, cx));
        })
        .ok()?;
        let (mod_id, texture_name) = receiver.await.ok().flatten()?;
        let directory = thumbnail::output_dir_for(&game, &mod_id);
        let target_dir = directory.clone();
        cx.background_spawn(async move { std::fs::create_dir_all(&target_dir) })
            .await
            .ok()?;
        Some(IconTarget {
            mod_id,
            texture_name,
            directory,
        })
    })
}

#[cfg(test)]
pub(crate) fn open_for_test(
    mod_ids: Vec<String>,
    window: &mut Window,
    cx: &mut App,
) -> (Option<Entity<IconNameDialog>>, Task<Option<IconChoice>>) {
    let (sender, receiver) = oneshot::channel();
    let dialog =
        window.open_modal(cx, |window, cx| IconNameDialog::new(mod_ids, sender, window, cx));
    (dialog, cx.spawn(async move |_| receiver.await.ok().flatten()))
}
