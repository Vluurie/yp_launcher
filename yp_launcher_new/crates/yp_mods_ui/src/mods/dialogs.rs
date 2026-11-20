use futures::channel::oneshot;
use gpui::{DismissEvent, EventEmitter, Task};
use yp_mods::InstalledMod;
use yp_ui::prelude::*;
use yp_ui::{
    ButtonStyle, ChipTone, Confirm, Modal, ModalFooter, ModalHeader, ModalView, ModalWidth,
    PromptOptions, Spinner, TintColor, ToggleState, WindowExt, prompt_text,
};

pub const MOD_NAME_MAX_LENGTH: usize = 64;

pub fn is_allowed_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, ' ' | '_' | '-')
}

pub fn has_only_allowed_name_characters(text: &str) -> bool {
    text.chars().all(is_allowed_name_character)
}

pub fn keep_allowed_name_characters(text: &str) -> String {
    text.chars()
        .filter(|c| is_allowed_name_character(*c))
        .collect()
}

pub struct NamingRequest {
    pub initial: String,
    pub exists_id: Option<String>,
    pub title: Option<SharedString>,
}

impl NamingRequest {
    pub fn new(initial: impl Into<String>) -> Self {
        Self {
            initial: initial.into(),
            exists_id: None,
            title: None,
        }
    }

    pub fn exists(mut self, id: impl Into<String>) -> Self {
        self.exists_id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
}

pub fn prompt_mod_name(
    request: NamingRequest,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<String>> {
    let strings = cx.t();
    let title = request
        .title
        .unwrap_or_else(|| SharedString::new_static(strings.mod_install_needs_name()));
    let invalid = SharedString::new_static(strings.mod_profile_error_name_invalid());
    let mut options = PromptOptions::new(title)
        .initial(keep_allowed_name_characters(&request.initial))
        .max_len(MOD_NAME_MAX_LENGTH)
        .validate(move |text| {
            if has_only_allowed_name_characters(text) {
                Ok(())
            } else {
                Err(invalid.clone())
            }
        });
    if let Some(id) = &request.exists_id {
        options = options.label(strings.mod_install_exists_pick_another(id));
    }
    prompt_text(options, window, cx)
}

pub struct UninstallDialog {
    mod_id: String,
    bundled_packs: Vec<String>,
    delete_bundled: bool,
    sender: Option<oneshot::Sender<Option<bool>>>,
    focus_handle: FocusHandle,
}

impl UninstallDialog {
    fn finish(&mut self, confirmed: bool, cx: &mut Context<Self>) {
        let result = confirmed.then_some(self.delete_bundled && !self.bundled_packs.is_empty());
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(result);
        }
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for UninstallDialog {}

impl Focusable for UninstallDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for UninstallDialog {}

impl Render for UninstallDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let has_bundled = !self.bundled_packs.is_empty();
        let packs = self.bundled_packs.join(", ");
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Confirm, _, cx| this.finish(true, cx)))
            .child(
                Modal::new("mods-uninstall-dialog", None)
                    .header(
                        ModalHeader::new()
                            .headline(strings.mod_uninstall_confirm_title())
                            .headline_color(Color::Error),
                    )
                    .section(div().child(strings.mod_uninstall_confirm_body(&self.mod_id)))
                    .when(has_bundled, |this| {
                        this.section(
                            Checkbox::new("mods-uninstall-bundled", self.delete_bundled)
                                .label(strings.mod_uninstall_also_textures_label(&packs))
                                .label_size(LabelSize::Small)
                                .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                                    this.delete_bundled = state.selected();
                                    cx.notify();
                                })),
                        )
                    })
                    .footer(
                        ModalFooter::new().end_slot(
                            h_flex()
                                .gap(sizes.spacing_sm)
                                .child(
                                    Button::new("mods-uninstall-cancel", strings.action_cancel())
                                        .style(ButtonStyle::Tinted(TintColor::Muted))
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.finish(false, cx)),
                                        ),
                                )
                                .child(
                                    Button::new("mods-uninstall-confirm", strings.mod_uninstall())
                                        .style(ButtonStyle::Tinted(TintColor::Error))
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.finish(true, cx)),
                                        ),
                                ),
                        ),
                    ),
            )
    }
}

pub fn confirm_uninstall(
    installed: &InstalledMod,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<bool>> {
    let (sender, receiver) = oneshot::channel();
    let mod_id = installed.id.clone();
    let bundled_packs = installed.bundled_texture_packs.clone();
    window.open_modal(cx, |_, cx| UninstallDialog {
        delete_bundled: !bundled_packs.is_empty(),
        mod_id,
        bundled_packs,
        sender: Some(sender),
        focus_handle: cx.focus_handle(),
    });
    cx.spawn(async move |_| receiver.await.ok().flatten())
}

pub struct OutfitGuideDialog {
    mod_name: String,
    workspace_key: String,
    has_compat_config: bool,
    focus_handle: FocusHandle,
}

impl OutfitGuideDialog {
    pub fn new(
        mod_name: String,
        workspace_key: String,
        has_compat_config: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            mod_name,
            workspace_key,
            has_compat_config,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<DismissEvent> for OutfitGuideDialog {}

impl Focusable for OutfitGuideDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for OutfitGuideDialog {}

fn guide_way(
    icon: IconName,
    title: &'static str,
    body: SharedString,
    trailing: Option<AnyElement>,
    sizes: &Sizes,
) -> impl IntoElement {
    h_flex()
        .w_full()
        .items_start()
        .gap(sizes.spacing_md)
        .child(
            div()
                .flex_none()
                .pt(sizes.spacing_xs)
                .child(Icon::new(icon).size(IconSize::Medium).color(Color::Accent)),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(sizes.spacing_xs)
                .child(Label::new(title).size(LabelSize::Small).bold())
                .child(
                    Label::new(body)
                        .size(LabelSize::Small)
                        .color(Color::Secondary),
                )
                .children(trailing.map(|trailing| h_flex().pt(sizes.spacing_sm).child(trailing))),
        )
}

impl Render for OutfitGuideDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let npc_link = Chip::new(strings.outfit_guide_npc_link())
            .id("outfit-guide-npc-link")
            .icon(IconName::OpenInNew)
            .tone(ChipTone::Accent)
            .on_click(|_, _, cx| cx.open_url(yp_core::consts::MULTI_OUTFIT_NPC_URL))
            .into_any_element();
        div().track_focus(&self.focus_handle).child(
            Modal::new("mods-outfit-guide", None)
                .width(ModalWidth::Default)
                .header(
                    ModalHeader::new()
                        .icon(Icon::new(IconName::Checkroom).color(Color::Accent))
                        .headline(strings.outfit_guide_title()),
                )
                .section(div().child(strings.outfit_guide_intro(&self.mod_name)))
                .section(
                    v_flex()
                        .w_full()
                        .gap(sizes.spacing_lg)
                        .child(guide_way(
                            IconName::DashboardCustomizeOutlined,
                            strings.outfit_guide_wardrobe_title(),
                            strings
                                .outfit_guide_wardrobe_body(&self.workspace_key)
                                .into(),
                            None,
                            &sizes,
                        ))
                        .child(guide_way(
                            IconName::PersonPinCircleOutlined,
                            strings.outfit_guide_npc_title(),
                            strings.outfit_guide_npc_body().into(),
                            Some(npc_link),
                            &sizes,
                        ))
                        .child(guide_way(
                            IconName::StarOutline,
                            strings.outfit_guide_default_title(),
                            strings.outfit_guide_default_body().into(),
                            None,
                            &sizes,
                        )),
                )
                .when(self.has_compat_config, |this| {
                    this.section(
                        Label::new(strings.outfit_guide_compat_note())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .italic(),
                    )
                })
                .footer(
                    ModalFooter::new().end_slot(
                        Button::new("outfit-guide-done", strings.outfit_guide_done())
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent))),
                    ),
                ),
        )
    }
}

pub struct BusyDialog {
    message: SharedString,
    focus_handle: FocusHandle,
    _owner_watch: Task<()>,
}

impl BusyDialog {
    pub fn new(
        message: impl Into<SharedString>,
        owner_alive: oneshot::Receiver<()>,
        cx: &mut Context<Self>,
    ) -> Self {
        let owner_watch = cx.spawn(async move |this, cx| {
            let _ = owner_alive.await;
            this.update(cx, |_, cx| cx.emit(DismissEvent)).ok();
        });
        Self {
            message: message.into(),
            focus_handle: cx.focus_handle(),
            _owner_watch: owner_watch,
        }
    }

    pub fn message(&self) -> &SharedString {
        &self.message
    }

    pub fn set_message(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
        let message = message.into();
        if self.message != message {
            self.message = message;
            cx.notify();
        }
    }

    pub fn finish(&mut self, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for BusyDialog {}

impl Focusable for BusyDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for BusyDialog {
    fn on_before_dismiss(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        false
    }

    fn dismiss_on_scrim_click(&self) -> bool {
        false
    }

    fn scrim_opacity(&self, cx: &App) -> Option<f32> {
        Some(cx.theme().alpha.overlay)
    }
}

impl Render for BusyDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        div().track_focus(&self.focus_handle).child(
            Modal::new("mods-busy-dialog", None)
                .width(ModalWidth::Default)
                .section(
                    h_flex()
                        .w_full()
                        .gap(sizes.spacing_lg)
                        .py(sizes.padding_sm)
                        .child(Spinner::new().size(IconSize::Large))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(Label::new(self.message.clone()).color(Color::Default)),
                        ),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_names_allow_letters_digits_space_underscore_and_dash_only() {
        assert!(has_only_allowed_name_characters("Kimono Wax_2-b"));
        assert!(has_only_allowed_name_characters(""));
        assert!(!has_only_allowed_name_characters("2B (DLC)"));
        assert!(!has_only_allowed_name_characters("über"));
        assert_eq!(keep_allowed_name_characters("2B (DLC) v1.2"), "2B DLC v12");
    }
}
