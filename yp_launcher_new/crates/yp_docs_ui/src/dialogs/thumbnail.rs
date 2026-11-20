use std::path::{Path, PathBuf};

use futures::channel::oneshot;
use gpui::{
    App, Context, CursorStyle, DismissEvent, Entity, EventEmitter, ExternalPaths, FocusHandle,
    Focusable, IntoElement, MouseButton, ObjectFit, Pixels, Point, Render, SharedString,
    Subscription, Task, Window, div, img, prelude::*, px,
};
use yp_builders::thumbnail;
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, ButtonStyle, InputEvent, ModalView, Sizes, TintColor};

use crate::builders::forms::{ThumbnailChoice, clamp_side, preview_scale};
use crate::util::{error_line, pick_file, pointer_capture, status_line};

const HANDLE_SIZE: f32 = 16.;
const CANVAS_BORDER: f32 = 1.;
const SIZE_FIELD_WIDTH: f32 = 72.;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailOptions {
    pub subtitle: SharedString,
    pub initial: Option<ThumbnailChoice>,
    pub fixed_size: Option<(u32, u32)>,
    pub size: Option<(u32, u32)>,
}

impl ThumbnailOptions {
    pub fn new(subtitle: impl Into<SharedString>) -> Self {
        Self {
            subtitle: subtitle.into(),
            initial: None,
            fixed_size: None,
            size: None,
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    pub fn initial(mut self, initial: Option<ThumbnailChoice>) -> Self {
        self.initial = initial;
        self
    }

    pub fn fixed_size(mut self, width: u32, height: u32) -> Self {
        self.fixed_size = Some((width, height));
        self
    }

    pub fn starting_size(&self) -> (u32, u32) {
        match (self.fixed_size, self.size, &self.initial) {
            (Some(size), _, _) | (None, Some(size), _) => size,
            (None, None, Some(initial)) => (initial.width, initial.height),
            (None, None, None) => (thumbnail::DEFAULT_WIDTH, thumbnail::DEFAULT_HEIGHT),
        }
    }
}

pub fn first_accepted(paths: &[PathBuf]) -> Option<&Path> {
    paths
        .iter()
        .map(PathBuf::as_path)
        .find(|path| thumbnail::is_accepted(path))
}

pub fn resized(start: (u32, u32), delta: Point<Pixels>, scale: f32) -> (u32, u32) {
    let scale = if scale > 0. { scale } else { 1. };
    let grow = |side: u32, delta: Pixels| {
        clamp_side((f64::from(side) + f64::from(f32::from(delta) / scale)).round() as i64)
    };
    (grow(start.0, delta.x), grow(start.1, delta.y))
}

struct Resize {
    origin: Point<Pixels>,
    size: (u32, u32),
    scale: f32,
}

pub struct ThumbnailDialog {
    options: ThumbnailOptions,
    image_path: Option<PathBuf>,
    width: u32,
    height: u32,
    width_input: Entity<InputState>,
    height_input: Entity<InputState>,
    error: Option<SharedString>,
    resize: Option<Resize>,
    sender: Option<oneshot::Sender<Option<ThumbnailChoice>>>,
    pick_task: Option<Task<()>>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl ThumbnailDialog {
    fn new(
        options: ThumbnailOptions,
        sender: oneshot::Sender<Option<ThumbnailChoice>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (width, height) = options.starting_size();
        let width_input =
            cx.new(|cx| InputState::new(window, cx).mono().default_value(width.to_string()));
        let height_input =
            cx.new(|cx| InputState::new(window, cx).mono().default_value(height.to_string()));
        let subscriptions = vec![
            cx.subscribe_in(&width_input, window, |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Submit | InputEvent::Blur) {
                    this.commit_size(true, window, cx);
                }
            }),
            cx.subscribe_in(&height_input, window, |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Submit | InputEvent::Blur) {
                    this.commit_size(false, window, cx);
                }
            }),
        ];
        Self {
            image_path: options
                .initial
                .as_ref()
                .map(|initial| initial.image_path.clone()),
            options,
            width,
            height,
            width_input,
            height_input,
            error: None,
            resize: None,
            sender: Some(sender),
            pick_task: None,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn image_path(&self) -> Option<&Path> {
        self.image_path.as_deref()
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    fn is_fixed(&self) -> bool {
        self.options.fixed_size.is_some()
    }

    fn sync_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (width, height) = (self.width.to_string(), self.height.to_string());
        self.width_input
            .update(cx, |input, cx| input.set_value(width, window, cx));
        self.height_input
            .update(cx, |input, cx| input.set_value(height, window, cx));
    }

    fn commit_size(&mut self, is_width: bool, window: &mut Window, cx: &mut Context<Self>) {
        let input = if is_width {
            &self.width_input
        } else {
            &self.height_input
        };
        if let Ok(value) = input.read(cx).value().trim().parse::<i64>() {
            let side = clamp_side(value);
            if is_width {
                self.width = side;
            } else {
                self.height = side;
            }
        }
        self.sync_inputs(window, cx);
        cx.notify();
    }

    pub fn set_size(&mut self, width: u32, height: u32, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_fixed() {
            return;
        }
        self.width = clamp_side(i64::from(width));
        self.height = clamp_side(i64::from(height));
        self.sync_inputs(window, cx);
        cx.notify();
    }

    pub fn accept_paths(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        match first_accepted(paths) {
            Some(path) => {
                self.image_path = Some(path.to_path_buf());
                self.error = None;
            }
            None => {
                self.error = Some(SharedString::new_static(cx.t().err_image_unreadable()));
            }
        }
        cx.notify();
    }

    fn browse(&mut self, cx: &mut Context<Self>) {
        let picked = pick_file(cx);
        self.pick_task = Some(cx.spawn(async move |this, cx| {
            if let Some(path) = picked.await {
                this.update(cx, |this, cx| this.accept_paths(&[path], cx)).ok();
            }
        }));
    }

    fn finish(&mut self, choice: Option<ThumbnailChoice>, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(choice);
        }
        cx.emit(DismissEvent);
    }

    pub fn confirm(&mut self, cx: &mut Context<Self>) {
        let Some(image_path) = self.image_path.clone() else {
            return;
        };
        self.finish(
            Some(ThumbnailChoice {
                image_path,
                width: self.width,
                height: self.height,
            }),
            cx,
        );
    }

    fn canvas(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let scale = preview_scale(self.width, self.height);
        let shown_width = px(self.width as f32 * scale);
        let shown_height = px(self.height as f32 * scale);
        let accent = theme.colors.accent_primary;
        let drag_background = accent.opacity(theme.alpha.subtle);
        let hover_border = accent.opacity(theme.alpha.half);
        let handle = px(HANDLE_SIZE);
        let resizing = self.resize.is_some();
        let fixed = self.is_fixed();
        let size = (self.width, self.height);
        let content = match &self.image_path {
            Some(path) => img(path.clone())
                .w(shown_width - px(CANVAS_BORDER * 2.))
                .h(shown_height - px(CANVAS_BORDER * 2.))
                .object_fit(ObjectFit::Fill)
                .into_any_element(),
            None => v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap(sizes.spacing_sm)
                .p(sizes.padding_sm)
                .child(
                    Icon::new(IconName::AddPhotoAlternateOutlined)
                        .size(IconSize::Large)
                        .color(Color::Muted),
                )
                .child(
                    Label::new(strings.thumbnail_drop())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .centered(),
                )
                .into_any_element(),
        };
        div()
            .relative()
            .flex_none()
            .w(shown_width + handle)
            .h(shown_height + handle)
            .child(
                h_flex()
                    .id("thumbnail-canvas")
                    .w(shown_width)
                    .h(shown_height)
                    .overflow_hidden()
                    .rounded(Sizes::CONTROL_RADIUS)
                    .border_1()
                    .border_color(theme.colors.border_medium)
                    .bg(theme.colors.surface_light)
                    .cursor_pointer()
                    .hover(move |style| style.border_color(hover_border))
                    .drag_over::<ExternalPaths>(move |style, _, _, _| {
                        style.bg(drag_background).border_color(accent)
                    })
                    .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                        this.accept_paths(paths.paths(), cx);
                    }))
                    .on_click(cx.listener(|this, _, _, cx| this.browse(cx)))
                    .child(content),
            )
            .when(!fixed, |this| {
                this.child(
                    div()
                        .id("thumbnail-resize")
                        .absolute()
                        .left(shown_width - handle / 2.)
                        .top(shown_height - handle / 2.)
                        .size(handle)
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(3.))
                        .bg(accent)
                        .cursor(CursorStyle::ResizeUpLeftDownRight)
                        .hover(move |style| style.bg(theme.colors.accent_highlight))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                cx.stop_propagation();
                                this.resize = Some(Resize {
                                    origin: event.position,
                                    size,
                                    scale,
                                });
                                cx.notify();
                            }),
                        )
                        .child(
                            Icon::new(IconName::OpenInFull)
                                .size(IconSize::Custom(px(10.)))
                                .color(Color::OnAccent),
                        ),
                )
            })
            .child(pointer_capture(
                resizing,
                cx.listener(|this, event: &gpui::MouseMoveEvent, window, cx| {
                    let Some(resize) = &this.resize else {
                        return;
                    };
                    let delta = Point {
                        x: event.position.x - resize.origin.x,
                        y: event.position.y - resize.origin.y,
                    };
                    let (width, height) = resized(resize.size, delta, resize.scale);
                    if (width, height) != (this.width, this.height) {
                        this.width = width;
                        this.height = height;
                        this.sync_inputs(window, cx);
                        cx.notify();
                    }
                }),
                cx.listener(|this, _: &gpui::MouseUpEvent, _, cx| {
                    this.resize = None;
                    cx.notify();
                }),
            ))
    }

    fn size_row(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let label = Label::new(strings.thumbnail_size())
            .size(LabelSize::XSmall)
            .color(Color::Secondary);
        let hint = |text: SharedString| {
            v_flex().flex_1().min_w_0().child(
                Label::new(text)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .line_height(1.3),
            )
        };
        if self.is_fixed() {
            return h_flex()
                .w_full()
                .gap(sizes.spacing_lg)
                .child(label)
                .child(
                    Label::new(format!("{} \u{d7} {}", self.width, self.height))
                        .size(LabelSize::Small)
                        .color(Color::Accent)
                        .semibold()
                        .mono(),
                )
                .child(hint(SharedString::new_static(strings.thumbnail_size_fixed())));
        }
        let scale = preview_scale(self.width, self.height);
        let note = if scale < 1. {
            SharedString::from(strings.thumbnail_scaled((scale * 100.).round() as i64))
        } else {
            SharedString::new_static(strings.thumbnail_size_hint())
        };
        h_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(label)
            .child(
                Input::new(&self.width_input)
                    .size(InputSize::Small)
                    .width(px(SIZE_FIELD_WIDTH)),
            )
            .child(
                Label::new("\u{d7}")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Input::new(&self.height_input)
                    .size(InputSize::Small)
                    .width(px(SIZE_FIELD_WIDTH)),
            )
            .child(hint(note))
    }
}

impl EventEmitter<DismissEvent> for ThumbnailDialog {}

impl Focusable for ThumbnailDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for ThumbnailDialog {
    fn on_before_dismiss(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(None);
        }
        true
    }
}

impl Render for ThumbnailDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let has_image = self.image_path.is_some();
        div().track_focus(&self.focus_handle).child(
            Modal::new("thumbnail-dialog", None)
                .width(ModalWidth::Custom(px(640.)))
                .header(
                    ModalHeader::new()
                        .icon(Icon::new(IconName::ImageOutlined).color(Color::Accent))
                        .headline(strings.thumbnail_title())
                        .description(self.options.subtitle.clone()),
                )
                .section(
                    h_flex()
                        .w_full()
                        .justify_center()
                        .child(self.canvas(window, cx)),
                )
                .section(self.size_row(window, cx))
                .children(self.error.clone().map(error_line))
                .section(status_line(
                    IconName::BoltOutlined,
                    Color::Muted,
                    strings.thumbnail_hot_reload(),
                    window,
                    cx,
                ))
                .footer(
                    ModalFooter::new().end_slot(
                        h_flex()
                            .gap(sizes.spacing_md)
                            .child(
                                Button::new("thumbnail-cancel", strings.cancel_button())
                                    .style(ButtonStyle::Tinted(TintColor::Muted))
                                    .on_click(cx.listener(|this, _, _, cx| this.finish(None, cx))),
                            )
                            .child(
                                Button::new("thumbnail-use", strings.thumbnail_use())
                                    .style(ButtonStyle::Tinted(if has_image {
                                        TintColor::Success
                                    } else {
                                        TintColor::Muted
                                    }))
                                    .disabled(!has_image)
                                    .on_click(cx.listener(|this, _, _, cx| this.confirm(cx))),
                            ),
                    ),
                ),
        )
    }
}

pub fn pick_thumbnail(
    options: ThumbnailOptions,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<ThumbnailChoice>> {
    let (sender, receiver) = oneshot::channel();
    window.open_modal(cx, |window, cx| ThumbnailDialog::new(options, sender, window, cx));
    cx.spawn(async move |_| receiver.await.ok().flatten())
}

#[cfg(test)]
pub(crate) fn open_for_test(
    options: ThumbnailOptions,
    window: &mut Window,
    cx: &mut App,
) -> (Option<Entity<ThumbnailDialog>>, Task<Option<ThumbnailChoice>>) {
    let (sender, receiver) = oneshot::channel();
    let dialog =
        window.open_modal(cx, |window, cx| ThumbnailDialog::new(options, sender, window, cx));
    (dialog, cx.spawn(async move |_| receiver.await.ok().flatten()))
}
