use std::sync::Arc;

use gpui::{
    Bounds, Anchor, DismissEvent, EventEmitter, ListAlignment, ListState,
    PathPromptOptions, Point, RenderImage, ScrollHandle, Subscription, TitlebarOptions,
    UniformListScrollHandle, WindowBounds, WindowOptions, actions, point, size,
};
use yp_ui::prelude::*;
use yp_ui::theme::set_breakpoint_override;
use yp_ui::{
    ApplyKind, Assets, BorderPosition, CaptureFlow, CaptureInput, ChipTone, ConfirmOptions,
    ConfirmTone, ContextMenu, DropProgress, InputEvent, InputState, Locale, ModalView, NumberKind,
    PickItem, PickOptions, PromptOptions, Root, ScrollableElement, SelectItem, SheetSide,
    TintColor, Toast, ToggleOption, TourAnchorExt, TourOverlay, TourStep, VirtualList, WindowExt,
    asset_image, confirm, host_key_binding, h_resizable, pick_many, pick_one, prompt_text, render_image_from_rgba,
    reorder, resizable_panel, right_click_menu,
};

actions!(gallery, [Quit]);

const PAGES: [(&str, IconName); 11] = [
    ("Typography", IconName::ShortText),
    ("Icons", IconName::CategoryOutlined),
    ("Buttons", IconName::Adjust),
    ("Inputs", IconName::KeyboardOutlined),
    ("Menus", IconName::ListAltOutlined),
    ("Containers", IconName::DashboardCustomizeOutlined),
    ("Lists", IconName::LayersOutlined),
    ("Overlays", IconName::FilterNone),
    ("Feedback", IconName::InfoOutline),
    ("Settings", IconName::Tune),
    ("Media", IconName::ImageOutlined),
];

const BLOOM_OLD: &str = "images/config/bloom_2017.webp";
const BLOOM_NEW: &str = "images/config/bloom_2021.webp";
const CROSSHAIR: &str = "images/config/crosshair.webp";

#[derive(Default)]
struct StartupOptions {
    page: usize,
    theme: Option<ThemeId>,
    locale: Option<Locale>,
    breakpoint: Option<Breakpoint>,
    width: Option<f32>,
    height: Option<f32>,
    open: Option<String>,
    hidden: bool,
}

impl StartupOptions {
    fn from_args() -> Self {
        let mut options = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            let Some(value) = args.next() else {
                break;
            };
            match flag.as_str() {
                "--page" => options.page = value.parse().unwrap_or(0),
                "--theme" => options.theme = ThemeId::from_code(&value),
                "--locale" => options.locale = Locale::from_code(&value),
                "--breakpoint" => {
                    options.breakpoint = match value.as_str() {
                        "compact" => Some(Breakpoint::Compact),
                        "medium" => Some(Breakpoint::Medium),
                        "wide" => Some(Breakpoint::Wide),
                        _ => None,
                    }
                }
                "--width" => options.width = value.parse().ok(),
                "--height" => options.height = value.parse().ok(),
                "--open" => options.open = Some(value),
                "--hidden" => options.hidden = value == "true",
                _ => {}
            }
        }
        options
    }
}

fn main() {
    let options = StartupOptions::from_args();
    let assets = Assets::with_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
    gpui_platform::application().with_assets(assets).run(move |cx| {
        yp_ui::init(cx);
        if options.hidden {
            yp_ui::set_link_opener(|_, _| {}, cx);
        }
        if let Some(theme) = options.theme {
            yp_ui::theme::set_theme(theme, cx);
        }
        if let Some(locale) = options.locale {
            yp_ui::I18n::set(locale, cx);
        }
        if options.breakpoint.is_some() {
            set_breakpoint_override(options.breakpoint, cx);
        }
        let start_page = options.page;
        let start_tour = options.open.as_deref() == Some("tour");
        let open = options.open.clone();
        let hidden = options.hidden;
        let window_size = size(
            px(options.width.unwrap_or(1300.)),
            px(options.height.unwrap_or(750.)),
        );
        cx.bind_keys([host_key_binding("secondary-q", Quit, None, cx)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let bounds = Bounds::centered(None, window_size, cx);
        let title = SharedString::new_static(cx.t().ui_gallery_title());
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(title),
                        appears_transparent: cfg!(target_os = "windows"),
                        traffic_light_position: None,
                    }),
                    window_min_size: Some(size(px(600.), px(420.))),
                    show: !hidden,
                    focus: !hidden,
                    ..Default::default()
                },
                |window, cx| {
                    let gallery = cx.new(|cx| Gallery::new(start_page, start_tour, window, cx));
                    let title_bar = cx.new(|_| GalleryTitleBar);
                    cx.new(|cx| {
                        let mut root = Root::new(gallery, window, cx);
                        if cfg!(target_os = "windows") {
                            root.set_title_bar(Some(title_bar.into()), cx);
                        }
                        root
                    })
                },
            )
            .expect("gallery window");
        if let Some(open) = open {
            handle
                .update(cx, |_, window, cx| {
                    window.defer(cx, move |window, cx| open_overlay(&open, window, cx));
                })
                .ok();
        }
        if !hidden {
            cx.activate(true);
        }
    });
}

fn open_overlay(kind: &str, window: &mut Window, cx: &mut App) {
    let strings = cx.t();
    match kind {
        "modal" => {
            window.open_modal(cx, |_, cx| GalleryModal {
                focus_handle: cx.focus_handle(),
            });
        }
        "sheet" => {
            window.open_sheet(cx, |_, cx| GallerySheet {
                focus_handle: cx.focus_handle(),
            });
        }
        "confirm" => confirm(
            ConfirmOptions::new(
                strings.unsaved_changes_title(),
                strings.unsaved_changes_message(),
            )
            .confirm_label(strings.discard())
            .cancel_label(strings.stay())
            .tone(ConfirmTone::Danger),
            window,
            cx,
        )
        .detach(),
        "confirmlong" => confirm(
            ConfirmOptions::new(
                strings.ui_gallery_long_name(),
                strings.ui_gallery_long_text(),
            )
            .confirm_label(strings.discard())
            .cancel_label(strings.stay()),
            window,
            cx,
        )
        .detach(),
        "toastlong" => {
            window.push_toast(Toast::warning(strings.ui_gallery_long_text()), cx);
            window.push_toast(Toast::info(strings.ui_gallery_long_name()), cx);
        }
        "prompt" => prompt_text(
            PromptOptions::new(strings.ui_gallery_open_prompt())
                .label(strings.ui_gallery_sample())
                .initial("yorha")
                .max_len(27),
            window,
            cx,
        )
        .detach(),
        "pick" => pick_one(Gallery::pick_options(cx), window, cx).detach(),
        "pickmany" => pick_many(Gallery::pick_options(cx), window, cx).detach(),
        "toast" => {
            for severity in [
                Severity::Info,
                Severity::Success,
                Severity::Warning,
                Severity::Error,
            ] {
                window.push_toast(
                    Toast::new(strings.ui_gallery_toast()).severity(severity),
                    cx,
                );
            }
        }
        "lightbox" => yp_ui::Lightbox::open(asset_image(BLOOM_NEW), window, cx),
        _ => {}
    }
}

struct GalleryTitleBar;

impl Render for GalleryTitleBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new(cx.t().ui_gallery_title()).icon(IconName::SportsEsports)
    }
}

struct GalleryModal {
    focus_handle: FocusHandle,
}

impl EventEmitter<DismissEvent> for GalleryModal {}

impl Focusable for GalleryModal {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for GalleryModal {}

impl Render for GalleryModal {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        div().track_focus(&self.focus_handle).child(
            Modal::new("gallery-modal", None)
                .header(
                    ModalHeader::new()
                        .icon(Icon::new(IconName::AutoAwesome).color(Color::Accent))
                        .headline(strings.ui_gallery_open_modal())
                        .description(strings.ui_gallery_sample())
                        .show_dismiss_button(true),
                )
                .section(Label::new(strings.language_support_notice()).size(LabelSize::Small))
                .section(Callout::new().description(strings.ui_gallery_tour_second()))
                .footer(
                    ModalFooter::new().end_slot(
                        Button::new("gallery-modal-close", strings.diagnostics_close())
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent))),
                    ),
                ),
        )
    }
}

struct GallerySheet {
    focus_handle: FocusHandle,
}

impl EventEmitter<DismissEvent> for GallerySheet {}

impl Focusable for GallerySheet {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GallerySheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        div().track_focus(&self.focus_handle).size_full().child(
            Sheet::new("gallery-sheet")
                .side(SheetSide::Right)
                .resizable(true)
                .on_dismiss(|window, cx| window.close_sheet(cx))
                .child(
                    PageHeader::new(strings.ui_gallery_open_sheet()).actions(
                        IconButton::new("gallery-sheet-close", IconName::Close)
                            .tooltip(Tooltip::text(strings.tooltip_close()))
                            .on_click(|_, window, cx| window.close_sheet(cx)),
                    ),
                )
                .child(
                    v_flex()
                        .p(sizes.content_padding)
                        .gap(sizes.spacing_lg)
                        .child(Label::new(strings.ui_gallery_sample()))
                        .child(Callout::new().description(strings.ui_drag_to_reorder())),
                ),
        )
    }
}

struct Gallery {
    page: usize,
    content_scroll: ScrollHandle,
    text: Entity<InputState>,
    multiline: Entity<InputState>,
    number: Entity<InputState>,
    checkbox: ToggleState,
    switch: bool,
    radio: usize,
    slider: f32,
    select: usize,
    toggle_group: usize,
    tab: usize,
    key_label: SharedString,
    input_label: SharedString,
    field: SharedString,
    viewport: gpui::Size<Pixels>,
    result: SharedString,
    tour_open: bool,
    sortable: Vec<SharedString>,
    config_collapsed: bool,
    list_collapsed: bool,
    setting_bool: bool,
    setting_number: f64,
    setting_slider: f32,
    setting_select: usize,
    uniform_scroll: UniformListScrollHandle,
    list_state: ListState,
    frame: Option<Arc<RenderImage>>,
    orbit: Point<Pixels>,
    zoom: f32,
    busy: bool,
    animate_background: bool,
    _subscriptions: Vec<Subscription>,
}

fn sample_frame() -> Option<Arc<RenderImage>> {
    let (width, height) = (256u32, 160u32);
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let checker = ((x / 16) + (y / 16)) % 2 == 0;
            let shade = if checker { 60 } else { 30 };
            pixels.push((x * 255 / width) as u8 / 2 + shade);
            pixels.push((y * 255 / height) as u8 / 2 + shade);
            pixels.push(shade * 2);
            pixels.push(255);
        }
    }
    render_image_from_rgba(width, height, pixels)
}

fn block(title: &'static str, content: impl IntoElement) -> Card {
    Card::new().title(title).child(content)
}

impl Gallery {
    fn new(page: usize, tour_open: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let strings = cx.t();
        let text = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(strings.ui_gallery_placeholder())
                .max_length(40)
        });
        let multiline = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(4)
                .max_rows(8)
                .mono()
                .default_value(strings.ui_gallery_multiline())
        });
        let number = cx.new(|cx| InputState::new(window, cx).default_value("12"));
        let subscription = cx.subscribe(&text, |this, input, event: &InputEvent, cx| {
            if *event == InputEvent::Submit {
                this.result = input.read(cx).text();
                cx.notify();
            }
        });
        Self {
            page,
            content_scroll: ScrollHandle::new(),
            text,
            multiline,
            number,
            checkbox: ToggleState::Indeterminate,
            switch: true,
            radio: 0,
            slider: 0.35,
            select: 1,
            toggle_group: 0,
            tab: 0,
            key_label: SharedString::new_static(strings.keybind_unbound()),
            input_label: SharedString::new_static(strings.keybind_unbound()),
            field: SharedString::new_static(strings.ui_gallery_placeholder()),
            viewport: gpui::Size::default(),
            result: SharedString::default(),
            tour_open,
            sortable: Locale::ALL
                .into_iter()
                .map(|locale| SharedString::new_static(locale.display_name()))
                .collect(),
            config_collapsed: false,
            list_collapsed: false,
            setting_bool: true,
            setting_number: 61.,
            setting_slider: 2.,
            setting_select: 0,
            uniform_scroll: UniformListScrollHandle::new(),
            list_state: ListState::new(200, ListAlignment::Top, px(200.)),
            frame: sample_frame(),
            orbit: Point::default(),
            zoom: 1.,
            busy: false,
            animate_background: true,
            _subscriptions: vec![subscription],
        }
    }

    fn show_result(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.result = SharedString::from(cx.t().ui_gallery_result(&value.into()));
        cx.notify();
    }

    fn select_items(cx: &App) -> Vec<SelectItem<usize>> {
        let strings = cx.t();
        vec![
            SelectItem::new(strings.ui_gallery_breakpoint_auto(), 0).icon(IconName::Tune),
            SelectItem::new(strings.ui_gallery_breakpoint_compact(), 1),
            SelectItem::new(strings.ui_gallery_breakpoint_medium(), 2),
            SelectItem::new(strings.ui_gallery_breakpoint_wide(), 3).disabled(true),
        ]
    }

    fn typography(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sample = strings.ui_gallery_sample();
        v_flex()
            .gap_3()
            .child(block(
                "Headline",
                v_flex()
                    .gap_2()
                    .child(Headline::new(strings.app_title()).size(HeadlineSize::Hero))
                    .child(Headline::new(strings.app_title()).size(HeadlineSize::Page)),
            ))
            .child(block(
                "Label",
                v_flex()
                    .gap_1()
                    .child(
                        Label::new(sample)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new(sample)
                            .size(LabelSize::Small)
                            .color(Color::Secondary),
                    )
                    .child(Label::new(sample))
                    .child(
                        Label::new(sample)
                            .size(LabelSize::Large)
                            .color(Color::Accent)
                            .bold(),
                    )
                    .child(
                        Label::new(sample)
                            .size(LabelSize::XLarge)
                            .italic()
                            .underline(),
                    )
                    .child(Label::new(sample).mono().uppercase().color(Color::LogDebug))
                    .child(div().w(px(220.)).child(Label::new(sample).truncate()))
                    .child(div().w(px(220.)).child(Label::new(sample).line_clamp(2)))
                    .child(
                        h_flex()
                            .w(px(340.))
                            .gap_2()
                            .child(
                                Icon::new(IconName::Texture)
                                    .size(IconSize::Small)
                                    .color(Color::Muted),
                            )
                            .child(Label::new(strings.ui_gallery_long_name()).truncate())
                            .child(
                                Chip::new("DATA")
                                    .tone(ChipTone::Muted)
                                    .size(ChipSize::Small),
                            ),
                    )
                    .child(
                        div().w(px(260.)).child(
                            Link::new("gallery-link-long", strings.ui_gallery_long_name())
                                .icon(IconName::Link)
                                .truncate(),
                        ),
                    ),
            ))
            .child(block(
                "HighlightedLabel / Kbd / Link / Divider / SelectableText",
                v_flex()
                    .gap_2()
                    .child(HighlightedLabel::matching(sample, "st"))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Kbd::new("Ctrl"))
                            .child(Kbd::new("Shift"))
                            .child(Kbd::new("P").active(true)),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                Link::new("gallery-link", strings.tooltip_guide())
                                    .icon(IconName::MenuBookOutlined)
                                    .end_icon(IconName::ArrowOutward)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_result("Link", cx);
                                    })),
                            )
                            .child(Divider::vertical().color(DividerColor::Medium))
                            .child(
                                Link::new("gallery-link-plain", strings.tooltip_discord())
                                    .color(Color::Accent)
                                    .underline(),
                            ),
                    )
                    .child(Divider::horizontal())
                    .child(SelectableText::new("gallery-selectable", sample).mono()),
            ))
            .into_any_element()
    }

    fn icons(&mut self, _: &mut Window, _: &mut Context<Self>) -> AnyElement {
        block(
            "Icon / IconName",
            h_flex()
                .flex_wrap()
                .gap_2()
                .children(IconName::ALL.into_iter().enumerate().map(|(index, icon)| {
                    div()
                        .id(("gallery-icon", index))
                        .p_1()
                        .child(
                            Icon::new(icon)
                                .size(IconSize::Large)
                                .color(Color::Secondary),
                        )
                        .tooltip(Tooltip::text(icon.name()))
                })),
        )
        .into_any_element()
    }

    fn buttons(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let styles = [
            ("gallery-filled", ButtonStyle::Filled),
            ("gallery-accent", ButtonStyle::Tinted(TintColor::Accent)),
            ("gallery-success", ButtonStyle::Tinted(TintColor::Success)),
            ("gallery-warning", ButtonStyle::Tinted(TintColor::Warning)),
            ("gallery-error", ButtonStyle::Tinted(TintColor::Error)),
            ("gallery-muted", ButtonStyle::Tinted(TintColor::Muted)),
            ("gallery-primary", ButtonStyle::Tinted(TintColor::Primary)),
            ("gallery-outlined", ButtonStyle::Outlined),
            ("gallery-subtle", ButtonStyle::Subtle),
            ("gallery-transparent", ButtonStyle::Transparent),
            ("gallery-link-style", ButtonStyle::Link),
        ];
        let sizes = [
            ("gallery-size-compact", ButtonSize::Compact),
            ("gallery-size-default", ButtonSize::Default),
            ("gallery-size-medium", ButtonSize::Medium),
            ("gallery-size-large", ButtonSize::Large),
        ];
        v_flex()
            .gap_3()
            .child(
                block(
                    "Button / ButtonStyle",
                    h_flex()
                        .flex_wrap()
                        .gap_2()
                        .children(styles.into_iter().map(|(id, style)| {
                            Button::new(id, strings.button_save())
                                .style(style)
                                .start_icon(IconName::Save)
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.show_result(id, cx)),
                                )
                        })),
                )
                .tour_anchor("gallery-buttons"),
            )
            .child(block(
                "ButtonSize / states",
                v_flex()
                    .gap_2()
                    .child(h_flex().flex_wrap().gap_2().children(sizes.into_iter().map(
                        |(id, size)| {
                            Button::new(id, strings.button_discard())
                                .style(ButtonStyle::Tinted(TintColor::Accent))
                                .size(size)
                                .end_icon(IconName::ArrowForward)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.show_result(id, cx);
                                }))
                        },
                    )))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                Button::new("gallery-disabled", strings.button_cancel())
                                    .style(ButtonStyle::Filled)
                                    .disabled(true),
                            )
                            .child(
                                Button::new("gallery-loading", strings.ui_loading())
                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                    .loading(true),
                            )
                            .child(
                                Button::new("gallery-selected", strings.button_yes())
                                    .style(ButtonStyle::Outlined)
                                    .toggle_state(true)
                                    .selected_label(strings.button_no())
                                    .on_click(|_, _, _| {}),
                            )
                            .child(
                                Button::new("gallery-hero", strings.select_button())
                                    .style(ButtonStyle::Tinted(TintColor::Success))
                                    .size(ButtonSize::Hero)
                                    .start_icon(IconName::PlayArrowRounded)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_result("Hero", cx);
                                    })),
                            ),
                    ),
            ))
            .child(block(
                "IconButton / CopyButton / StepButton",
                h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        IconButton::new("gallery-icon-plain", IconName::Refresh)
                            .tooltip(Tooltip::text(strings.docs_preview()))
                            .on_click(
                                cx.listener(|this, _, _, cx| this.show_result("Refresh", cx)),
                            ),
                    )
                    .child(
                        IconButton::new("gallery-icon-bordered", IconName::FolderOpen)
                            .bordered(true)
                            .on_click(|_, _, _| {}),
                    )
                    .child(
                        IconButton::new("gallery-icon-wide", IconName::DeleteOutline)
                            .shape(IconButtonShape::Wide)
                            .style(ButtonStyle::Tinted(TintColor::Error))
                            .on_click(|_, _, _| {}),
                    )
                    .child(
                        IconButton::new("gallery-icon-toggled", IconName::StarOutline)
                            .selected_icon(IconName::Star)
                            .toggle_state(self.switch)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.switch = !this.switch;
                                cx.notify();
                            })),
                    )
                    .child(IconButton::new("gallery-icon-disabled", IconName::Save).disabled(true))
                    .child(CopyButton::new("gallery-copy", strings.ui_gallery_sample()))
                    .child(
                        CopyButton::new("gallery-copy-label", strings.ui_gallery_sample())
                            .label(strings.copy_command_button()),
                    )
                    .child(
                        StepButton::new("gallery-step-back", IconName::ArrowBack)
                            .on_click(|_, _, _| {}),
                    )
                    .child(StepButton::new(
                        "gallery-step-forward",
                        IconName::ArrowForward,
                    )),
            ))
            .child(block(
                "ToggleButtonGroup / ChoiceCard",
                v_flex()
                    .gap_2()
                    .child(
                        ToggleButtonGroup::new("gallery-toggle-group")
                            .option(strings.tab_mods(), IconName::ExtensionOutlined)
                            .option(strings.tab_textures(), IconName::Texture)
                            .option_with(ToggleOption::new(strings.tab_cutscenes()).count(3))
                            .selected(self.toggle_group)
                            .on_select(cx.listener(|this, index: &usize, _, cx| {
                                this.toggle_group = *index;
                                cx.notify();
                            })),
                    )
                    .child(
                        TwoColumn::new(
                            ChoiceCard::new("gallery-choice-yes", strings.onboarding_first_yes())
                                .description(strings.ui_gallery_sample())
                                .icon(IconName::ShieldOutlined)
                                .selected(self.radio == 0)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.radio = 0;
                                    cx.notify();
                                })),
                            ChoiceCard::new("gallery-choice-no", strings.onboarding_first_no())
                                .description(strings.ui_gallery_sample())
                                .icon(IconName::Visibility)
                                .selected(self.radio == 1)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.radio = 1;
                                    cx.notify();
                                })),
                        )
                        .id("gallery-choice-columns"),
                    ),
            ))
            .into_any_element()
    }

    fn inputs(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        v_flex()
            .gap_3()
            .child(
                block(
                    "Input (single line, multi line) / NumberInput",
                    v_flex()
                        .gap_2()
                        .child(
                            Input::new(&self.text).cleanable(true).prefix(
                                Icon::new(IconName::Search)
                                    .size(IconSize::Small)
                                    .color(Color::Muted),
                            ),
                        )
                        .child(Input::new(&self.text).disabled(true))
                        .child(
                            TextField::new("gallery-text-field", self.field.clone())
                                .placeholder(strings.ui_gallery_placeholder())
                                .max_length(40)
                                .cleanable(true)
                                .on_change(cx.listener(|this, text: &SharedString, _, cx| {
                                    this.field = text.clone();
                                    cx.notify();
                                })),
                        )
                        .child(Input::new(&self.multiline))
                        .child(
                            h_flex()
                                .gap_3()
                                .child(
                                    NumberInput::new(&self.number)
                                        .kind(NumberKind::Int)
                                        .min(0.)
                                        .max(128.)
                                        .steppers(true)
                                        .width(px(160.))
                                        .on_commit(cx.listener(|this, value: &f64, _, cx| {
                                            this.show_result(value.to_string(), cx);
                                        })),
                                )
                                .child(
                                    NumberField::new("gallery-number-field", self.setting_number)
                                        .kind(NumberKind::Float { decimals: 2 })
                                        .on_change(cx.listener(|this, value: &f64, _, cx| {
                                            this.setting_number = *value;
                                            cx.notify();
                                        })),
                                ),
                        ),
                )
                .tour_anchor("gallery-inputs"),
            )
            .child(block(
                "Checkbox / Switch / RadioGroup",
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_4()
                            .child(
                                Checkbox::new("gallery-checkbox", self.checkbox)
                                    .label(strings.launch_option_minimize_on_launch())
                                    .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                                        this.checkbox = *state;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("gallery-checkbox-disabled", true)
                                    .label(strings.button_yes())
                                    .disabled(true),
                            )
                            .child(
                                Switch::new("gallery-switch", self.switch)
                                    .label(strings.button_yes())
                                    .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                                        this.switch = state.selected();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Switch::new("gallery-switch-small", !self.switch)
                                    .size(SwitchSize::Small)
                                    .disabled(true),
                            ),
                    )
                    .child(
                        RadioGroup::new("gallery-radio")
                            .option(strings.onboarding_first_yes())
                            .option_with_description(
                                strings.onboarding_first_no(),
                                strings.ui_gallery_sample(),
                            )
                            .selected(self.radio)
                            .on_change(cx.listener(|this, index: &usize, _, cx| {
                                this.radio = *index;
                                cx.notify();
                            })),
                    ),
            ))
            .child(block(
                "Slider / KeyCapture",
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                Slider::new("gallery-slider")
                                    .value(self.slider)
                                    .step(0.05)
                                    .on_change(cx.listener(|this, value: &f32, _, cx| {
                                        this.slider = *value;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Label::new(format!("{:.2}", self.slider))
                                    .color(Color::Accent)
                                    .mono(),
                            ),
                    )
                    .child(
                        Slider::new("gallery-slider-ticks")
                            .value(2048.)
                            .min(1024.)
                            .max(8192.)
                            .ticks(vec![1024., 2048., 4096., 8192.])
                            .disabled(true),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                KeyCapture::new("gallery-key-capture", self.key_label.clone())
                                    .on_capture(cx.listener(
                                        |this, keystroke: &gpui::Keystroke, _, cx| {
                                            this.key_label =
                                                SharedString::from(keystroke.unparse(cx.platform_info().os));
                                            cx.notify();
                                        },
                                    )),
                            )
                            .child({
                                let view = cx.entity();
                                KeyCapture::new("gallery-key-capture-all", self.input_label.clone())
                                    .capture_modifiers(true)
                                    .capture_mouse(true)
                                    .capture_wheel(true)
                                    .on_input(move |input, _, cx| {
                                        let label = match input {
                                            CaptureInput::Key(keystroke) => keystroke.unparse(cx.platform_info().os),
                                            CaptureInput::Modifiers(modifiers) => {
                                                format!("{modifiers:?}")
                                            }
                                            CaptureInput::Mouse(button) => format!("{button:?}"),
                                            CaptureInput::Wheel(delta) => {
                                                format!("{:.0}", f32::from(delta.y))
                                            }
                                        };
                                        view.update(cx, |this, cx| {
                                            this.input_label = SharedString::from(label);
                                            cx.notify();
                                        });
                                        match input {
                                            CaptureInput::Modifiers(_) => CaptureFlow::Continue,
                                            _ => CaptureFlow::Finish,
                                        }
                                    })
                            }),
                    ),
            ))
            .into_any_element()
    }

    fn menus(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let items = Self::select_items(cx);
        let selected_label = items[self.select].label.clone();
        let entity = cx.entity();
        let menu_entity = entity.clone();
        let context_entity = entity.clone();
        v_flex()
            .gap_3()
            .child(block(
                "Select / SelectMenu",
                h_flex()
                    .flex_wrap()
                    .gap_3()
                    .child(
                        Select::new("gallery-select", selected_label.clone())
                            .items(items.clone())
                            .selected_index(self.select)
                            .highlight(true)
                            .on_select(cx.listener(|this, value: &usize, _, cx| {
                                this.select = *value;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("gallery-select-search", selected_label.clone())
                            .items(items.clone())
                            .selected_index(self.select)
                            .searchable(true)
                            .min_width(px(220.))
                            .menu_matches_trigger_width(true)
                            .on_select(cx.listener(|this, value: &usize, _, cx| {
                                this.select = *value;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("gallery-select-icon", selected_label.clone())
                            .items(items.clone())
                            .trigger_icon(IconName::Tune)
                            .tooltip(strings.tooltip_language())
                            .on_select(cx.listener(|this, value: &usize, _, cx| {
                                this.select = *value;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("gallery-select-disabled", selected_label.clone())
                            .items(items.clone()),
                    )
                    .child(
                        Select::new("gallery-select-long", strings.ui_gallery_long_name())
                            .items(
                                items
                                    .into_iter()
                                    .chain([SelectItem::new(strings.ui_gallery_long_name(), 4)]),
                            )
                            .selected_index(4)
                            .max_width(px(220.))
                            .tooltip(strings.ui_gallery_long_name())
                            .on_select(cx.listener(|this, value: &usize, _, cx| {
                                this.select = (*value).min(3);
                                cx.notify();
                            })),
                    ),
            ))
            .child(block(
                "PopoverMenu / ContextMenu / right_click_menu / Popover",
                h_flex()
                    .flex_wrap()
                    .gap_3()
                    .child(
                        PopoverMenu::<ContextMenu>::new("gallery-popover-menu")
                            .trigger(
                                Button::new(
                                    "gallery-popover-trigger",
                                    strings.mods_tutorial_help_tooltip(),
                                )
                                .style(ButtonStyle::Outlined)
                                .end_icon(IconName::ExpandMore)
                                .tooltip(Tooltip::text(strings.mods_tutorial_help_tooltip())),
                            )
                            .menu(move |window, cx| {
                                let strings = cx.t();
                                let first = menu_entity.clone();
                                let second = menu_entity.clone();
                                Some(ContextMenu::build(window, cx, |menu, _, _| {
                                    menu.header(strings.tab_section_mods())
                                        .icon_entry(
                                            strings.tab_mods(),
                                            IconName::ExtensionOutlined,
                                            move |_, cx| {
                                                first.update(cx, |this, cx| {
                                                    this.show_result("Mods", cx)
                                                });
                                            },
                                        )
                                        .toggleable_entry(
                                            strings.tab_textures(),
                                            true,
                                            move |_, cx| {
                                                second.update(cx, |this, cx| {
                                                    this.show_result("Textures", cx)
                                                });
                                            },
                                        )
                                        .icon_entry(
                                            strings.ui_gallery_long_name(),
                                            IconName::Texture,
                                            |_, _| {},
                                        )
                                        .separator()
                                        .link(strings.tooltip_guide(), "https://example.invalid")
                                }))
                            }),
                    )
                    .child(
                        right_click_menu::<ContextMenu>("gallery-right-click")
                            .trigger(
                                Chip::new(strings.tab_section_general())
                                    .id("gallery-right-click-chip")
                                    .tone(ChipTone::Accent)
                                    .icon(IconName::Mouse),
                            )
                            .menu(move |window, cx| {
                                let strings = cx.t();
                                let target = context_entity.clone();
                                ContextMenu::build(window, cx, |menu, _, _| {
                                    menu.entry(strings.button_delete(), move |_, cx| {
                                        target
                                            .update(cx, |this, cx| this.show_result("Delete", cx));
                                    })
                                    .entry(strings.button_cancel(), |_, _| {})
                                })
                            }),
                    )
                    .child(
                        Popover::new("gallery-popover")
                            .anchor(Anchor::BottomLeft)
                            .attach(Anchor::TopLeft)
                            .trigger(
                                Button::new(
                                    "gallery-popover-button",
                                    strings.launch_options_title(),
                                )
                                .style(ButtonStyle::Tinted(TintColor::Accent))
                                .end_icon(IconName::ExpandLess),
                            )
                            .content(|_, cx| {
                                let strings = cx.t();
                                v_flex()
                                    .gap_2()
                                    .child(
                                        Checkbox::new("gallery-popover-check-1", true)
                                            .label(strings.launch_option_minimize_on_launch()),
                                    )
                                    .child(
                                        Checkbox::new("gallery-popover-check-2", false)
                                            .label(strings.launch_option_prefer_dedicated_gpu())
                                            .label_color(Color::Secondary)
                                            .tooltip(strings.ui_gallery_sample()),
                                    )
                                    .child(
                                        Callout::new()
                                            .id("gallery-popover-callout")
                                            .title(strings.unsaved_changes_title())
                                            .description(strings.ui_gallery_long_text()),
                                    )
                                    .into_any_element()
                            }),
                    )
                    .child(LanguageSelector::new())
                    .child(ThemeToggle::new()),
            ))
            .into_any_element()
    }

    fn containers(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let tabs = [
            strings.tab_launcher(),
            strings.tab_mods(),
            strings.tab_textures(),
        ];
        v_flex()
            .gap_3()
            .child(
                TwoColumn::new(
                    CollapsibleCard::new("gallery-collapsible", strings.card_general())
                        .default_open(true)
                        .changed(vec![
                            SharedString::new_static(strings.tab_mods()),
                            SharedString::new_static(strings.tab_textures()),
                        ])
                        .child(Label::new(strings.ui_gallery_sample()).size(LabelSize::Small))
                        .child(Section::new(strings.tab_section_config()).child(
                            Label::new(strings.ui_gallery_tour_intro()).size(LabelSize::Small),
                        )),
                    Card::new()
                        .title(strings.card_loading())
                        .header_end(
                            Disclosure::new("gallery-disclosure", !self.config_collapsed)
                                .on_toggle(cx.listener(|this, _, _, cx| {
                                    this.config_collapsed = !this.config_collapsed;
                                    cx.notify();
                                })),
                        )
                        .when(!self.config_collapsed, |this| {
                            this.child(
                                DescriptionList::new()
                                    .row(strings.tab_mods(), "12")
                                    .row(strings.tab_textures(), "5")
                                    .item(KeyValueRow::new(strings.tab_cutscenes(), "H264").mono()),
                            )
                        }),
                )
                .id("gallery-container-columns"),
            )
            .child(block(
                "TabBar / Tab / StepDots / EmptyState",
                v_flex()
                    .gap_2()
                    .child(TabBar::new().tabs(tabs.into_iter().enumerate().map(
                        |(index, label)| {
                            Tab::new(("gallery-tab", index))
                                .label(label)
                                .toggle_state(self.tab == index)
                                .count(index + 1)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.tab = index;
                                    cx.notify();
                                }))
                        },
                    )))
                    .child(StepDots::new(6, self.tab + 1))
                    .child(
                        EmptyState::new(strings.ui_nothing_here())
                            .icon(IconName::Inventory2Outlined)
                            .description(strings.ui_gallery_sample())
                            .compact(),
                    ),
            ))
            .child(block(
                "Resizable",
                div().h(px(140.)).child(
                    h_resizable("gallery-resizable")
                        .child(
                            resizable_panel()
                                .size(px(220.))
                                .min(px(120.))
                                .max(px(420.))
                                .child(div().size_full().p_2().child(
                                    Label::new(strings.ui_drag_to_reorder()).size(LabelSize::Small),
                                )),
                        )
                        .child(
                            resizable_panel().child(
                                div().size_full().p_2().child(
                                    Label::new(strings.ui_gallery_sample())
                                        .size(LabelSize::Small)
                                        .color(Color::Muted),
                                ),
                            ),
                        ),
                ),
            ))
            .into_any_element()
    }

    fn lists(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let rows = [
            (strings.tab_mods(), IconName::ExtensionOutlined),
            (strings.tab_textures(), IconName::Texture),
            (strings.tab_cutscenes(), IconName::MovieCreationOutlined),
        ];
        let selected = self.tab;
        v_flex()
            .gap_3()
            .child(block(
                "ListHeader / ListItem / ListSeparator",
                v_flex()
                    .child(
                        ListHeader::new("gallery-list-header", strings.tab_section_mods())
                            .count(rows.len())
                            .collapsible(self.list_collapsed)
                            .on_toggle(cx.listener(|this, _, _, cx| {
                                this.list_collapsed = !this.list_collapsed;
                                cx.notify();
                            })),
                    )
                    .when(!self.list_collapsed, |this| {
                        this.children(rows.into_iter().enumerate().map(|(index, (label, icon))| {
                            ListItem::new(("gallery-list-item", index))
                                .start_slot(
                                    Icon::new(icon).size(IconSize::Small).color(Color::Muted),
                                )
                                .selected(selected == index)
                                .indent_level(index.min(1))
                                .end_slot(
                                    Chip::new("DATA")
                                        .tone(ChipTone::Muted)
                                        .size(ChipSize::Small),
                                )
                                .end_slot_on_hover(
                                    IconButton::new(
                                        ("gallery-list-edit", index),
                                        IconName::EditOutlined,
                                    )
                                    .icon_size(IconSize::Small)
                                    .size(ButtonSize::Compact)
                                    .on_click(|_, _, _| {}),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.tab = index;
                                    cx.notify();
                                }))
                                .child(Label::new(label).size(LabelSize::Small).truncate())
                        }))
                    })
                    .child(ListSeparator::new())
                    .child(
                        ListItem::new("gallery-list-disabled")
                            .disabled(true)
                            .child(Label::new(strings.ui_nothing_here()).size(LabelSize::Small)),
                    )
                    .child(
                        v_flex()
                            .w(px(420.))
                            .child(
                                ListItem::new("gallery-list-long")
                                    .start_slot(
                                        Icon::new(IconName::Texture)
                                            .size(IconSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .end_slot(
                                        Chip::new("DATA")
                                            .tone(ChipTone::Muted)
                                            .size(ChipSize::Small),
                                    )
                                    .on_click(|_, _, _| {})
                                    .child(
                                        Label::new(strings.ui_gallery_long_name())
                                            .size(LabelSize::Small)
                                            .truncate(),
                                    ),
                            )
                            .child(
                                ListItem::new("gallery-list-accent")
                                    .accent(true)
                                    .on_click(|_, _, _| {})
                                    .child(
                                        v_flex()
                                            .min_w_0()
                                            .child(
                                                Label::new(strings.ui_gallery_long_name())
                                                    .size(LabelSize::Small)
                                                    .truncate(),
                                            )
                                            .child(
                                                Label::new(strings.ui_gallery_long_text())
                                                    .size(LabelSize::XSmall)
                                                    .color(Color::Muted)
                                                    .truncate(),
                                            ),
                                    ),
                            ),
                    ),
            ))
            .child(block(
                "SortableList",
                SortableList::new("gallery-sortable", self.sortable.clone())
                    .drag_label(|item: &SharedString| item.clone())
                    .row(|item, index, _, _| {
                        h_flex()
                            .gap_2()
                            .py_1()
                            .child(CountBadge::new(index + 1))
                            .child(Label::new(item.clone()).size(LabelSize::Small))
                            .into_any_element()
                    })
                    .on_reorder(cx.listener_reorder()),
            ))
            .child(block(
                "VirtualList::uniform (10 000 rows) / VirtualList::list",
                h_flex()
                    .gap_3()
                    .h(px(220.))
                    .child(div().flex_1().h_full().child(VirtualList::uniform(
                        "gallery-uniform",
                        10_000,
                        &self.uniform_scroll,
                        |range, _, cx| {
                            let strings = cx.t();
                            range
                                .map(|index| {
                                    ListItem::new(("gallery-uniform-row", index))
                                        .spacing(ListItemSpacing::ExtraDense)
                                        .on_click(|_, _, _| {})
                                        .child(
                                            Label::new(strings.ui_gallery_row(index as i64))
                                                .size(LabelSize::Small),
                                        )
                                })
                                .collect::<Vec<_>>()
                        },
                    )))
                    .child(div().flex_1().h_full().child(VirtualList::list(
                        "gallery-list",
                        &self.list_state,
                        |index, _, cx| {
                            let strings = cx.t();
                            v_flex()
                                .p_1()
                                .child(
                                    Label::new(strings.ui_gallery_row(index as i64))
                                        .size(LabelSize::Small)
                                        .bold(),
                                )
                                .when(index % 3 == 0, |this| {
                                    this.child(
                                        Label::new(strings.ui_gallery_sample())
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    )
                                })
                                .into_any_element()
                        },
                    ))),
            ))
            .into_any_element()
    }

    fn overlays(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        v_flex()
            .gap_3()
            .child(block(
                "Modal / Sheet / Toast / Tooltip / Tour",
                h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        Button::new("gallery-open-modal", strings.ui_gallery_open_modal())
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .on_click(|_, window, cx| {
                                window.open_modal(cx, |_, cx| GalleryModal {
                                    focus_handle: cx.focus_handle(),
                                });
                            }),
                    )
                    .child(
                        Button::new("gallery-open-sheet", strings.ui_gallery_open_sheet())
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .on_click(|_, window, cx| {
                                window.open_sheet(cx, |_, cx| GallerySheet {
                                    focus_handle: cx.focus_handle(),
                                });
                            }),
                    )
                    .children(
                        [
                            ("gallery-toast-info", Severity::Info),
                            ("gallery-toast-accent", Severity::Accent),
                            ("gallery-toast-success", Severity::Success),
                            ("gallery-toast-warning", Severity::Warning),
                            ("gallery-toast-error", Severity::Error),
                        ]
                        .into_iter()
                        .map(|(id, severity)| {
                            Button::new(id, strings.ui_gallery_show_toast())
                                .style(ButtonStyle::Outlined)
                                .color(Color::from(severity))
                                .on_click(move |_, window, cx| {
                                    let message = cx.t().ui_gallery_toast();
                                    window.push_toast(Toast::new(message).severity(severity), cx);
                                })
                        }),
                    )
                    .child(
                        Button::new("gallery-tooltip", strings.tooltip_guide())
                            .style(ButtonStyle::Subtle)
                            .tooltip(Tooltip::with_meta(
                                strings.tooltip_guide(),
                                strings.ui_gallery_sample(),
                            ))
                            .on_click(|_, _, _| {}),
                    )
                    .child(
                        Button::new("gallery-tooltip-long", strings.tooltip_discord())
                            .style(ButtonStyle::Outlined)
                            .tooltip(Tooltip::with_meta(
                                strings.ui_gallery_long_text(),
                                strings.ui_gallery_long_name(),
                            ))
                            .on_click(|_, _, _| {}),
                    )
                    .child(
                        Button::new("gallery-toast-long", strings.ui_gallery_show_toast())
                            .style(ButtonStyle::Tinted(TintColor::Muted))
                            .on_click(|_, window, cx| {
                                let message = cx.t().ui_gallery_long_text();
                                window.push_toast(Toast::warning(message), cx);
                            }),
                    )
                    .child(
                        Button::new("gallery-start-tour", strings.ui_gallery_start_tour())
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.page = 2;
                                this.tour_open = true;
                                cx.notify();
                            })),
                    ),
            ))
            .child(block(
                "confirm / prompt_text / pick_one / pick_many",
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                Button::new("gallery-confirm", strings.ui_gallery_open_confirm())
                                    .style(ButtonStyle::Tinted(TintColor::Warning))
                                    .on_click(cx.listener(|_, _, window, cx| {
                                        let strings = cx.t();
                                        let task = confirm(
                                            ConfirmOptions::new(
                                                strings.unsaved_changes_title(),
                                                strings.unsaved_changes_message(),
                                            )
                                            .confirm_label(strings.discard())
                                            .cancel_label(strings.stay())
                                            .tone(ConfirmTone::Danger),
                                            window,
                                            cx,
                                        );
                                        cx.spawn(async move |this, cx| {
                                            let confirmed = task.await;
                                            this.update(cx, |this, cx| {
                                                this.show_result(confirmed.to_string(), cx);
                                            })
                                            .ok();
                                        })
                                        .detach();
                                    })),
                            )
                            .child(
                                Button::new("gallery-prompt", strings.ui_gallery_open_prompt())
                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                    .on_click(cx.listener(|_, _, window, cx| {
                                        let strings = cx.t();
                                        let invalid = strings.mod_profile_error_name_invalid();
                                        let task = prompt_text(
                                            PromptOptions::new(strings.ui_gallery_open_prompt())
                                                .placeholder(strings.ui_gallery_placeholder())
                                                .max_len(27)
                                                .validate(move |text| {
                                                    if text.contains(' ') {
                                                        Err(SharedString::new_static(invalid))
                                                    } else {
                                                        Ok(())
                                                    }
                                                }),
                                            window,
                                            cx,
                                        );
                                        cx.spawn(async move |this, cx| {
                                            let name = task.await;
                                            this.update(cx, |this, cx| {
                                                this.show_result(name.unwrap_or_default(), cx);
                                            })
                                            .ok();
                                        })
                                        .detach();
                                    })),
                            )
                            .child(
                                Button::new("gallery-pick", strings.ui_gallery_open_pick())
                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                    .on_click(cx.listener(|_, _, window, cx| {
                                        let task = pick_one(Self::pick_options(cx), window, cx);
                                        cx.spawn(async move |this, cx| {
                                            let picked = task.await;
                                            this.update(cx, |this, cx| {
                                                let name = picked.map(Locale::display_name);
                                                this.show_result(name.unwrap_or_default(), cx);
                                            })
                                            .ok();
                                        })
                                        .detach();
                                    })),
                            )
                            .child(
                                Button::new(
                                    "gallery-pick-many",
                                    strings.ui_gallery_open_pick_many(),
                                )
                                .style(ButtonStyle::Tinted(TintColor::Accent))
                                .on_click(cx.listener(
                                    |_, _, window, cx| {
                                        let task = pick_many(Self::pick_options(cx), window, cx);
                                        cx.spawn(async move |this, cx| {
                                            let picked = task.await.unwrap_or_default();
                                            this.update(cx, |this, cx| {
                                                let names = picked
                                                    .into_iter()
                                                    .map(Locale::code)
                                                    .collect::<Vec<_>>()
                                                    .join(", ");
                                                this.show_result(names, cx);
                                            })
                                            .ok();
                                        })
                                        .detach();
                                    },
                                )),
                            ),
                    )
                    .child(
                        Label::new(self.result.clone())
                            .size(LabelSize::Small)
                            .color(Color::Accent),
                    ),
            ))
            .into_any_element()
    }

    fn pick_options(cx: &App) -> PickOptions<Locale> {
        PickOptions::new(
            cx.t().tooltip_language(),
            Locale::ALL
                .into_iter()
                .map(|locale| {
                    PickItem::new(locale.display_name(), locale)
                        .description(locale.code())
                        .icon(IconName::Language)
                        .selected(locale == cx.active_locale())
                })
                .collect(),
        )
        .body(cx.t().language_support_notice())
    }

    fn feedback(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let severities = [
            Severity::Info,
            Severity::Accent,
            Severity::Success,
            Severity::Warning,
            Severity::Error,
        ];
        let tones = [
            ChipTone::Neutral,
            ChipTone::Accent,
            ChipTone::Success,
            ChipTone::Warning,
            ChipTone::Error,
            ChipTone::Muted,
            ChipTone::Info,
        ];
        v_flex()
            .gap_3()
            .child(block(
                "Callout",
                v_flex()
                    .gap_2()
                    .children(severities.into_iter().enumerate().map(|(index, severity)| {
                        Callout::new()
                            .id(("gallery-callout", index))
                            .severity(severity)
                            .title(strings.unsaved_changes_title())
                            .description(strings.ui_gallery_sample())
                            .border_position(if index % 2 == 0 {
                                BorderPosition::All
                            } else {
                                BorderPosition::Left
                            })
                            .pulse(severity == Severity::Warning)
                            .when(severity == Severity::Error, |this| {
                                this.actions_slot(
                                    Button::new("gallery-callout-action", strings.button_discard())
                                        .style(ButtonStyle::Tinted(TintColor::Error))
                                        .on_click(|_, _, _| {}),
                                )
                                .on_dismiss(|_, _| {})
                            })
                    }))
                    .child(
                        Callout::new()
                            .id("gallery-callout-neutral")
                            .neutral()
                            .description(strings.ui_gallery_long_text()),
                    )
                    .child(ConfigErrorBanner::new(
                        "nams.toml",
                        "TOML parse error at line 3, column 9\n  |\n3 | value = \n  |         ^",
                    )),
            ))
            .child(block(
                "Chip / CountBadge / Indicator / ApplyMarker / ChangedBadge",
                v_flex()
                    .gap_2()
                    .child(h_flex().flex_wrap().gap_2().children(
                        tones.into_iter().enumerate().map(|(index, tone)| {
                            Chip::new(strings.tab_mods())
                                .id(("gallery-chip", index))
                                .tone(tone)
                                .icon(IconName::ExtensionOutlined)
                                .outlined(index % 2 == 1)
                                .tooltip_text(strings.ui_gallery_sample())
                                .on_click(|_, _, _| {})
                        }),
                    ))
                    .child(
                        h_flex()
                            .gap_3()
                            .child(CountBadge::new(7))
                            .child(CountBadge::new(128).tone(ChipTone::Accent))
                            .child(Indicator::dot().tone(ChipTone::Success))
                            .child(Indicator::dot().tone(ChipTone::Warning).pulse(true))
                            .child(ApplyMarker::new(ApplyKind::Live).id("gallery-apply-live"))
                            .child(ApplyMarker::new(ApplyKind::Restart).id("gallery-apply-restart"))
                            .child(ChangedBadge::new(
                                "gallery-changed",
                                vec![SharedString::new_static(strings.tab_mods())],
                            )),
                    ),
            ))
            .child(block(
                "Spinner / ProgressBar / LoadingState / BusyOverlay",
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_3()
                            .child(Spinner::new().size(IconSize::Small))
                            .child(Spinner::new())
                            .child(Spinner::new().size(IconSize::Large).color(Color::Success))
                            .child(LoadingState::new().compact()),
                    )
                    .child(ProgressBar::new(
                        "gallery-progress",
                        self.slider * 100.,
                        100.,
                    ))
                    .child(ProgressBar::indeterminate("gallery-progress-indeterminate"))
                    .child(
                        div()
                            .relative()
                            .h(px(120.))
                            .child(LoadingState::new().message(strings.searching_for_nier()))
                            .when(self.busy, |this| {
                                this.child(BusyOverlay::new(strings.mod_install_busy()))
                            }),
                    )
                    .child(
                        Switch::new("gallery-busy", self.busy)
                            .label("BusyOverlay")
                            .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                                this.busy = state.selected();
                                cx.notify();
                            })),
                    ),
            ))
            .into_any_element()
    }

    fn settings(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let items = Self::select_items(cx);
        let selected_label = items[self.setting_select].label.clone();
        TwoColumn::new(
            CollapsibleCard::new("gallery-settings-left", strings.card_general())
                .default_open(true)
                .changed(if self.setting_bool {
                    Vec::new()
                } else {
                    vec![SharedString::new_static(
                        strings.label_disable_plugin_loading(),
                    )]
                })
                .child(
                    SettingSwitch::new(
                        "gallery-setting-switch",
                        strings.label_disable_plugin_loading(),
                        self.setting_bool,
                    )
                    .description(strings.ui_gallery_sample())
                    .apply_marker(ApplyKind::Restart)
                    .changed(!self.setting_bool)
                    .on_change(cx.listener(|this, value: &bool, _, cx| {
                        this.setting_bool = *value;
                        cx.notify();
                    })),
                )
                .child(
                    SettingRow::new("gallery-setting-row", strings.tab_lod_mod())
                        .description(strings.ui_gallery_sample())
                        .description_slot(
                            h_flex()
                                .gap_2()
                                .child(
                                    Chip::new(strings.tab_textures())
                                        .tone(ChipTone::Info)
                                        .size(ChipSize::Small),
                                )
                                .child(
                                    Link::new("gallery-setting-link", strings.tooltip_guide())
                                        .href("https://example.invalid")
                                        .end_icon(IconName::ArrowOutward),
                                ),
                        )
                        .control(
                            Button::new("gallery-setting-button", strings.button_discard())
                                .style(ButtonStyle::Tinted(TintColor::Muted))
                                .on_click(|_, _, _| {}),
                        ),
                )
                .child(PreviewImage::new(
                    "gallery-preview",
                    asset_image(CROSSHAIR),
                    strings.naiom_crosshair_preview_label(),
                ))
                .child(
                    SettingNumber::new(
                        "gallery-setting-number",
                        strings.label_debug_menu_key(),
                        self.setting_number,
                    )
                    .kind(NumberKind::Hex)
                    .min(0.)
                    .description(strings.ui_gallery_tour_intro())
                    .on_change(cx.listener(|this, value: &f64, _, cx| {
                        this.setting_number = *value;
                        cx.notify();
                    })),
                ),
            CollapsibleCard::new("gallery-settings-right", strings.card_loading())
                .default_open(true)
                .child(
                    SettingSlider::new(
                        "gallery-setting-slider",
                        strings.tab_lod_mod(),
                        self.setting_slider,
                    )
                    .min(0.)
                    .max(4.)
                    .step(0.1)
                    .decimals(1)
                    .apply_marker(ApplyKind::Live)
                    .description(strings.ui_gallery_tour_second())
                    .on_change(cx.listener(|this, value: &f32, _, cx| {
                        this.setting_slider = *value;
                        cx.notify();
                    })),
                )
                .child(ImageComparison::new(
                    "gallery-comparison",
                    asset_image(BLOOM_NEW),
                    asset_image(BLOOM_OLD),
                    "2021",
                    "2017",
                ))
                .child(
                    SettingSelect::new(
                        "gallery-setting-select",
                        strings.tooltip_language(),
                        selected_label,
                    )
                    .items(items)
                    .selected_index(self.setting_select)
                    .description(strings.ui_gallery_sample())
                    .apply_marker(ApplyKind::Live)
                    .on_select(cx.listener(|this, value: &usize, _, cx| {
                        this.setting_select = *value;
                        cx.notify();
                    })),
                ),
        )
        .id("gallery-settings-columns")
        .into_any_element()
    }

    fn media(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let busy = self.busy.then(|| {
            DropProgress::new(strings.mod_install_busy())
                .percent(self.slider * 100.)
                .detail(strings.ui_gallery_long_name())
        });
        let viewport = self.viewport;
        v_flex()
            .gap_3()
            .child(block(
                "DropZone",
                DropZone::new("gallery-drop-zone")
                    .title(strings.drop_mod_here())
                    .hint(strings.drop_mod_here_hint())
                    .busy(busy)
                    .accept(|path| path.extension().is_some() || path.is_dir())
                    .on_drop_paths(|paths, window, cx| {
                        let count = i64::try_from(paths.len()).unwrap_or(i64::MAX);
                        let message = cx.t().ui_selected_count(count);
                        window.push_toast(Toast::success(message), cx);
                    })
                    .on_browse_files(|_, cx| {
                        drop(cx.prompt_for_paths(PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: true,
                            prompt: None,
                        }));
                    })
                    .on_browse_folder(|window, cx| {
                        let message = cx.t().drop_zone_browse_folder();
                        window.push_toast(Toast::info(message), cx);
                    }),
            ))
            .child(block(
                "DropZone (narrow) / aspect_ratio / Thumbnail fit",
                h_flex()
                    .items_start()
                    .gap_3()
                    .child(
                        div().w(px(240.)).child(
                            DropZone::new("gallery-drop-zone-narrow")
                                .title(strings.ui_gallery_sample())
                                .hint(strings.drop_mod_here_hint())
                                .on_browse_files(|_, _| {}),
                        ),
                    )
                    .child(
                        div().w(px(240.)).child(
                            DropZone::new("gallery-drop-zone-busy").busy(
                                DropProgress::new(strings.mod_install_busy())
                                    .detail(strings.ui_gallery_long_name()),
                            ),
                        ),
                    )
                    .child(
                        div()
                            .w(px(200.))
                            .aspect_ratio(16. / 9.)
                            .rounded_md()
                            .surface(cx, Surface::Medium)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Label::new("16 : 9").color(Color::Muted).mono()),
                    )
                    .child(
                        Thumbnail::new("gallery-thumb-tall", asset_image(CROSSHAIR))
                            .size(px(70.), px(112.))
                            .fit(gpui::ObjectFit::Contain),
                    )
                    .child(
                        Thumbnail::new("gallery-thumb-wide", asset_image(CROSSHAIR))
                            .size(px(200.), px(60.))
                            .fit(gpui::ObjectFit::Contain),
                    ),
            ))
            .child(
                TwoColumn::new(
                    block(
                        "FrameViewport",
                        div().h(px(220.)).child(
                            FrameViewport::new("gallery-viewport")
                                .frame(self.frame.clone())
                                .loading(self.busy)
                                .on_orbit(cx.listener(|this, delta: &Point<Pixels>, _, cx| {
                                    this.orbit =
                                        point(this.orbit.x + delta.x, this.orbit.y + delta.y);
                                    cx.notify();
                                }))
                                .on_pan(cx.listener(|this, delta: &Point<Pixels>, _, cx| {
                                    this.orbit =
                                        point(this.orbit.x - delta.x, this.orbit.y - delta.y);
                                    cx.notify();
                                }))
                                .on_zoom(cx.listener(|this, delta: &f32, _, cx| {
                                    this.zoom = (this.zoom * (delta * 0.001).exp()).clamp(0.1, 10.);
                                    cx.notify();
                                }))
                                .on_resize(cx.listener(|this, size: &gpui::Size<Pixels>, _, cx| {
                                    this.viewport = *size;
                                    cx.notify();
                                }))
                                .overlay(
                                    div().absolute().right_2().bottom_2().child(
                                        Chip::new(format!(
                                            "{:.0} x {:.0}",
                                            f32::from(viewport.width),
                                            f32::from(viewport.height)
                                        ))
                                        .tone(ChipTone::Muted),
                                    ),
                                )
                                .overlay(
                                    div().absolute().left_2().bottom_2().child(
                                        Chip::new(format!(
                                            "{:.0} / {:.0} / {:.2}",
                                            f32::from(self.orbit.x),
                                            f32::from(self.orbit.y),
                                            self.zoom
                                        ))
                                        .tone(ChipTone::Muted),
                                    ),
                                ),
                        ),
                    ),
                    block(
                        "NierBackground",
                        v_flex()
                            .gap_2()
                            .child(
                                div().h(px(180.)).child(
                                    NierBackground::new()
                                        .animate(self.animate_background)
                                        .child(
                                            div()
                                                .absolute()
                                                .inset_0()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(Headline::new(strings.app_title())),
                                        ),
                                ),
                            )
                            .child(
                                Switch::new("gallery-animate", self.animate_background)
                                    .label("animate")
                                    .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                                        this.animate_background = state.selected();
                                        cx.notify();
                                    })),
                            ),
                    ),
                )
                .id("gallery-media-columns"),
            )
            .child(block(
                "Thumbnail / Lightbox",
                h_flex()
                    .gap_3()
                    .child(
                        Thumbnail::new("gallery-thumb", asset_image(BLOOM_NEW))
                            .size(px(160.), px(90.))
                            .lightbox(true)
                            .tooltip(strings.ui_open_fullscreen()),
                    )
                    .child(
                        Thumbnail::new(
                            "gallery-thumb-missing",
                            asset_image("images/missing.png"),
                        )
                        .size(px(160.), px(90.)),
                    ),
            ))
            .into_any_element()
    }
}

trait ReorderListener {
    fn listener_reorder(&self) -> impl Fn(usize, usize, &mut Window, &mut App) + 'static;
}

impl ReorderListener for Context<'_, Gallery> {
    fn listener_reorder(&self) -> impl Fn(usize, usize, &mut Window, &mut App) + 'static {
        let gallery = self.weak_entity();
        move |from, to, _, cx| {
            gallery
                .update(cx, |this, cx| {
                    if reorder(&mut this.sortable, from, to) {
                        cx.notify();
                    }
                })
                .ok();
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let forced = yp_ui::theme::breakpoint_override(cx);
        let breakpoint_index = match forced {
            None => 0,
            Some(Breakpoint::Compact) => 1,
            Some(Breakpoint::Medium) => 2,
            Some(Breakpoint::Wide) => 3,
        };
        let page = self.page.min(PAGES.len() - 1);
        let content = match page {
            0 => self.typography(window, cx),
            1 => self.icons(window, cx),
            2 => self.buttons(window, cx),
            3 => self.inputs(window, cx),
            4 => self.menus(window, cx),
            5 => self.containers(window, cx),
            6 => self.lists(window, cx),
            7 => self.overlays(window, cx),
            8 => self.feedback(window, cx),
            9 => self.settings(window, cx),
            _ => self.media(window, cx),
        };
        let sidebar = Sidebar::new()
            .group(
                SidebarGroup::new("yp_ui").items(PAGES.into_iter().enumerate().map(
                    |(index, (label, icon))| {
                        SidebarItem::new(("gallery-page", index), icon, label)
                            .active(page == index)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.page = index;
                                this.content_scroll.set_offset(point(px(0.), px(0.)));
                                cx.notify();
                            }))
                    },
                )),
            )
            .footer(Chip::new("yp_ui").tone(ChipTone::Accent));
        let tour = self.tour_open.then(|| {
            TourOverlay::new(vec![
                TourStep::new(
                    strings.ui_gallery_start_tour(),
                    strings.ui_gallery_long_text(),
                ),
                TourStep::new(PAGES[2].0, strings.ui_gallery_tour_second())
                    .anchor("gallery-buttons"),
                TourStep::new(PAGES[0].0, strings.ui_gallery_sample()).anchor("gallery-header"),
            ])
            .on_finish(
                cx.listener(|this, _: &(), _, cx| {
                    this.tour_open = false;
                    cx.notify();
                })
                .into_action(),
            )
        });
        h_flex()
            .size_full()
            .items_start()
            .child(sidebar)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(
                        v_flex().w_full().tour_anchor("gallery-header").child(
                            PageHeader::new(PAGES[page].0)
                                .info_tooltip(strings.ui_gallery_title())
                                .after_title(
                                    Chip::new(format!("{} / {}", page + 1, PAGES.len()))
                                        .tone(ChipTone::Muted)
                                        .size(ChipSize::Small),
                                )
                                .dirty(!self.setting_bool)
                                .save_discard(
                                    cx.listener(|this, _: &(), _, cx| {
                                        this.setting_bool = true;
                                        cx.notify();
                                    })
                                    .into_action(),
                                    cx.listener(|this, _: &(), _, cx| {
                                        this.setting_bool = true;
                                        cx.notify();
                                    })
                                    .into_action(),
                                )
                                .actions(
                                    h_flex()
                                        .gap(sizes.spacing_md)
                                        .child(
                                            ToggleButtonGroup::new("gallery-breakpoint")
                                                .size(ButtonSize::Compact)
                                                .option(strings.ui_gallery_breakpoint_auto(), None)
                                                .option(
                                                    strings.ui_gallery_breakpoint_compact(),
                                                    None,
                                                )
                                                .option(
                                                    strings.ui_gallery_breakpoint_medium(),
                                                    None,
                                                )
                                                .option(strings.ui_gallery_breakpoint_wide(), None)
                                                .selected(breakpoint_index)
                                                .on_select(|index: &usize, _, cx| {
                                                    let forced = match *index {
                                                        1 => Some(Breakpoint::Compact),
                                                        2 => Some(Breakpoint::Medium),
                                                        3 => Some(Breakpoint::Wide),
                                                        _ => None,
                                                    };
                                                    set_breakpoint_override(forced, cx);
                                                }),
                                        )
                                        .child(ThemeToggle::new().id("gallery-header-theme"))
                                        .child(
                                            LanguageSelector::new().id("gallery-header-language"),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        div()
                            .id("gallery-content")
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .p(sizes.content_padding)
                            .child(content)
                            .overflow_y_scrollbar(&self.content_scroll),
                    ),
            )
            .children(tour)
    }
}

trait IntoAction {
    fn into_action(self) -> impl Fn(&mut Window, &mut App) + 'static;
}

impl<F: Fn(&(), &mut Window, &mut App) + 'static> IntoAction for F {
    fn into_action(self) -> impl Fn(&mut Window, &mut App) + 'static {
        move |window, cx| self(&(), window, cx)
    }
}
