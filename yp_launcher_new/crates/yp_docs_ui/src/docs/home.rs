use std::rc::Rc;

use gpui::{
    AnyElement, App, IntoElement, Pixels, ScrollHandle, SharedString, Window, div, linear_color_stop,
    linear_gradient, prelude::*, px,
};
use yp_docs::{DocPage, pretty_name};
use yp_i18n::Locale;
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, ButtonStyle, ChipTone};

use super::model::{FIRST_MOD_PAGE, find_by_file_name};
use crate::util::clamped;

const MAX_WIDTH: f32 = 1000.;
const TEXT_MAX_WIDTH: f32 = 620.;
const STACK_BELOW: f32 = 680.;
const TWO_COLUMNS_BELOW: f32 = 520.;
const THREE_COLUMNS_BELOW: f32 = 820.;
const HERO_ANGLE: f32 = 135.;

pub const NATIVE_README: &str = "README.md";
pub const SCRIPTING_README: &str = "scripting/README.md";

pub const NATIVE_GUIDES: [(&str, IconName); 8] = [
    ("03_item.md", IconName::CategoryOutlined),
    ("04_weapon.md", IconName::GavelOutlined),
    ("05_outfit.md", IconName::CheckroomOutlined),
    ("06_accessory.md", IconName::AutoAwesomeOutlined),
    ("07_quest.md", IconName::FlagOutlined),
    ("27_custom_music.md", IconName::MusicNoteOutlined),
    ("12_translate.md", IconName::TranslateOutlined),
    ("18_cheat_sheet.md", IconName::BoltOutlined),
];

pub const SCRIPTING_GUIDES: [(&str, IconName); 9] = [
    (SCRIPTING_README, IconName::MenuBookOutlined),
    ("01_hap.md", IconName::ListAltOutlined),
    ("00_hap_system.md", IconName::SchemaOutlined),
    ("03_getting_started.md", IconName::EditOutlined),
    ("02_action.md", IconName::PlayCircleOutline),
    ("04_scripts.md", IconName::Code),
    ("_signals.md", IconName::PodcastsOutlined),
    ("_layouts.md", IconName::GridViewOutlined),
    ("scripting.md", IconName::Functions),
];

pub type OpenFile = Rc<dyn Fn(&str, &mut Window, &mut App)>;
pub type BrowseAll = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideTile {
    pub file_name: &'static str,
    pub icon: IconName,
    pub title: String,
}

pub fn guide_tiles(pages: &[DocPage], entries: &[(&'static str, IconName)]) -> Vec<GuideTile> {
    entries
        .iter()
        .filter_map(|(file_name, icon)| {
            let page = find_by_file_name(pages, file_name)?;
            let title = if page.title.is_empty() {
                pretty_name(file_name)
            } else {
                page.title.clone()
            };
            Some(GuideTile {
                file_name,
                icon: *icon,
                title,
            })
        })
        .collect()
}

pub fn tile_columns(width: f32) -> u16 {
    if width < TWO_COLUMNS_BELOW {
        2
    } else if width < THREE_COLUMNS_BELOW {
        3
    } else {
        4
    }
}

fn open_handler(
    open: &OpenFile,
    file_name: impl Into<SharedString>,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static {
    let open = open.clone();
    let file_name = file_name.into();
    move |_, window, cx| open(&file_name, window, cx)
}

fn hero(open: &OpenFile, window: &mut Window, cx: &mut App) -> gpui::Div {
    let theme = cx.theme().clone();
    let sizes = window.sizes(cx);
    let strings = cx.t();
    let light = theme.appearance.is_light();
    let accent = theme.colors.accent_primary;
    let from = accent.opacity(if light {
        theme.alpha.soft
    } else {
        theme.alpha.hover
    });
    let to = theme.colors.surface_medium.opacity(if light {
        theme.alpha.glass
    } else {
        theme.alpha.overlay
    });
    let english_only = (cx.active_locale() != Locale::En).then(|| {
        h_flex()
            .gap(sizes.spacing_md)
            .child(
                Icon::new(IconName::InfoOutline)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Label::new(strings.docs_english_only())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
    });
    v_flex()
        .w_full()
        .gap(sizes.spacing_lg)
        .p(sizes.padding_xl)
        .rounded(sizes.border_radius * 2.)
        .border_1()
        .border_color(accent.opacity(theme.alpha.accent_border))
        .bg(linear_gradient(
            HERO_ANGLE,
            linear_color_stop(from, 0.),
            linear_color_stop(to, 1.),
        ))
        .child(
            Label::new(strings.docs_hero_title())
                .size(LabelSize::XLarge)
                .bold(),
        )
        .child(
            clamped(
                px(TEXT_MAX_WIDTH),
                Label::new(strings.docs_hero_body())
                    .size(LabelSize::Small)
                    .color(Color::Secondary)
                    .line_height(1.5),
            ),
        )
        .child(
            h_flex().child(
                ButtonLike::new("docs-hero-start")
                    .style(ButtonStyle::Filled)
                    .size(ButtonSize::Large)
                    .on_click(open_handler(open, FIRST_MOD_PAGE))
                    .child(
                        Icon::new(IconName::PlayArrowRounded)
                            .size(IconSize::Medium)
                            .color(Color::OnAccent),
                    )
                    .child(Text::new(strings.docs_hero_cta()))
                    .child(
                        div()
                            .px(sizes.chip_padding_h)
                            .py(sizes.chip_padding_v)
                            .rounded(sizes.font_md)
                            .bg(theme.colors.button_text.opacity(theme.alpha.line))
                            .text_size(sizes.font_xs)
                            .child(Text::new(strings.docs_hero_time())),
                    ),
            ),
        )
        .children(english_only)
}

fn path_cards(open: &OpenFile, stacked: bool, window: &mut Window, cx: &mut App) -> AnyElement {
    let sizes = window.sizes(cx);
    let strings = cx.t();
    let cards = [
        (
            "docs-path-beginner",
            IconName::RocketLaunchOutlined,
            strings.docs_path_beginner(),
            strings.docs_path_beginner_body(),
            FIRST_MOD_PAGE,
        ),
        (
            "docs-path-content",
            IconName::Inventory2Outlined,
            strings.docs_path_content(),
            strings.docs_path_content_body(),
            NATIVE_README,
        ),
        (
            "docs-path-scripting",
            IconName::MovieFilterOutlined,
            strings.docs_path_scripting(),
            strings.docs_path_scripting_body(),
            SCRIPTING_README,
        ),
    ]
    .map(|(id, icon, title, body, file_name)| {
        div().flex_1().min_w_0().flex().child(
            ChoiceCard::new(id, title)
                .description(body)
                .icon(icon)
                .on_click(open_handler(open, file_name)),
        )
    });
    if stacked {
        v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .children(cards)
            .into_any_element()
    } else {
        div()
            .flex()
            .flex_row()
            .w_full()
            .gap(sizes.spacing_lg)
            .children(cards)
            .into_any_element()
    }
}

#[allow(clippy::too_many_arguments)]
fn category_block(
    id: &'static str,
    title: &'static str,
    blurb: &'static str,
    icon: IconName,
    tiles: Vec<GuideTile>,
    columns: u16,
    open: &OpenFile,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    if tiles.is_empty() {
        return None;
    }
    let sizes = window.sizes(cx);
    let grid = div()
        .grid()
        .grid_cols(columns)
        .w_full()
        .gap(sizes.spacing_lg)
        .children(tiles.into_iter().enumerate().map(|(index, tile)| {
            ButtonLike::new(SharedString::from(format!("{id}-tile-{index}")))
                .style(ButtonStyle::Outlined)
                .size(ButtonSize::Large)
                .full_width()
                .on_click(open_handler(open, tile.file_name))
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .gap(sizes.spacing_lg)
                        .child(
                            Icon::new(tile.icon)
                                .size(IconSize::Medium)
                                .color(Color::Secondary)
                                .group_hover_color(
                                    ButtonLike::hover_group(&gpui::ElementId::Name(
                                        SharedString::from(format!("{id}-tile-{index}")),
                                    )),
                                    Color::Accent,
                                ),
                        )
                        .child(
                            v_flex().flex_1().min_w_0().child(
                                Label::new(SharedString::from(tile.title))
                                    .size(LabelSize::XSmall)
                                    .weight(gpui::FontWeight::MEDIUM)
                                    .line_clamp(2),
                            ),
                        ),
                )
        }));
    Some(
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .child(
                h_flex()
                    .gap(sizes.spacing_lg)
                    .child(Icon::new(icon).size(IconSize::Medium).color(Color::Accent))
                    .child(Label::new(title).size(LabelSize::Small).bold().uppercase()),
            )
            .child(
                clamped(
                    px(TEXT_MAX_WIDTH),
                    Label::new(blurb)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .line_height(1.5),
                ),
            )
            .child(v_flex().w_full().pt(sizes.spacing_md).child(grid))
            .into_any_element(),
    )
}

fn reference_card(
    pages: &[DocPage],
    open: &OpenFile,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let resources: Vec<&DocPage> = pages.iter().filter(|page| page.is_resource()).collect();
    if resources.is_empty() {
        return None;
    }
    let sizes = window.sizes(cx);
    let strings = cx.t();
    Some(
        Card::new()
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(
                        h_flex()
                            .gap(sizes.spacing_lg)
                            .child(
                                Icon::new(IconName::TableChartOutlined)
                                    .size(IconSize::Medium)
                                    .color(Color::Accent),
                            )
                            .child(
                                Label::new(strings.docs_section_reference())
                                    .size(LabelSize::Default)
                                    .bold(),
                            ),
                    )
                    .child(
                        Label::new(strings.docs_reference_body())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .line_height(1.5),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .flex_wrap()
                            .gap(sizes.spacing_md)
                            .pt(sizes.spacing_sm)
                            .children(resources.into_iter().enumerate().map(|(index, page)| {
                                Chip::new(SharedString::from(page.title.clone()))
                                    .id(SharedString::from(format!("docs-reference-{index}")))
                                    .tone(ChipTone::Neutral)
                                    .on_click(open_handler(open, page.file_name.clone()))
                            })),
                    ),
            )
            .into_any_element(),
    )
}

pub(crate) fn render_home(
    pages: &[DocPage],
    content_width: Pixels,
    scroll: &ScrollHandle,
    open: OpenFile,
    browse_all: BrowseAll,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div> {
    let sizes = window.sizes(cx);
    let strings = cx.t();
    let width = f32::from(content_width).min(MAX_WIDTH);
    let columns = tile_columns(width);
    let native = category_block(
        "docs-native",
        strings.docs_category_native(),
        strings.docs_native_blurb(),
        IconName::Inventory2Outlined,
        guide_tiles(pages, &NATIVE_GUIDES),
        columns,
        &open,
        window,
        cx,
    );
    let scripting = category_block(
        "docs-scripting",
        strings.docs_category_scripting(),
        strings.docs_scripting_blurb(),
        IconName::MovieFilterOutlined,
        guide_tiles(pages, &SCRIPTING_GUIDES),
        columns,
        &open,
        window,
        cx,
    );
    let reference = reference_card(pages, &open, window, cx);
    let browse_label = SharedString::from(format!(
        "{}  \u{b7}  {}",
        strings.docs_browse_all(),
        strings.docs_pages_count(pages.len() as i64)
    ));
    div()
        .id("docs-home")
        .size_full()
        .p(sizes.content_padding)
        .child(
            v_flex()
                .max_w(px(MAX_WIDTH))
                .mx_auto()
                .gap(sizes.padding_lg)
                .child(hero(&open, window, cx))
                .child(path_cards(&open, width < STACK_BELOW, window, cx))
                .children(native)
                .children(scripting)
                .children(reference)
                .child(
                    h_flex().w_full().justify_center().py(sizes.spacing_md).child(
                        Link::new("docs-browse-all", browse_label)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .hover_color(Color::Accent)
                            .on_click(move |_, window, cx| browse_all(window, cx)),
                    ),
                ),
        )
        .overflow_y_scrollbar(scroll)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(path: &str, title: &str) -> DocPage {
        let relative = path.strip_prefix("assets/docs/").expect("asset path");
        DocPage {
            path: path.to_owned(),
            category: relative.split('/').next().expect("category").to_owned(),
            title: title.to_owned(),
            file_name: relative.rsplit('/').next().expect("file").to_owned(),
        }
    }

    #[test]
    fn tiles_appear_only_for_pages_that_exist_and_use_their_titles() {
        let pages = vec![
            page("assets/docs/native/03_item.md", "Add an item"),
            page("assets/docs/native/05_outfit.md", ""),
            page("assets/docs/scripting/README.md", "Scripting"),
        ];
        let native = guide_tiles(&pages, &NATIVE_GUIDES);
        assert_eq!(
            native
                .iter()
                .map(|tile| (tile.file_name, tile.title.as_str()))
                .collect::<Vec<_>>(),
            vec![("03_item.md", "Add an item"), ("05_outfit.md", "Outfit")]
        );
        let scripting = guide_tiles(&pages, &SCRIPTING_GUIDES);
        assert_eq!(scripting.len(), 1);
        assert_eq!(scripting[0].file_name, SCRIPTING_README);
        assert!(guide_tiles(&[], &NATIVE_GUIDES).is_empty());
    }

    #[test]
    fn the_tile_grid_narrows_with_the_content() {
        assert_eq!(tile_columns(400.), 2);
        assert_eq!(tile_columns(520.), 3);
        assert_eq!(tile_columns(819.), 3);
        assert_eq!(tile_columns(1000.), 4);
    }

    #[test]
    fn the_bundled_docs_have_the_pages_the_home_links_to() {
        let mut library = yp_docs::DocsLibrary::new(crate::markdown::tests::docs_dir());
        let pages = library.load_index().expect("index").to_vec();
        for file_name in [FIRST_MOD_PAGE, NATIVE_README, SCRIPTING_README] {
            assert!(
                find_by_file_name(&pages, file_name).is_some(),
                "{file_name} is missing"
            );
        }
        assert_eq!(guide_tiles(&pages, &NATIVE_GUIDES).len(), NATIVE_GUIDES.len());
        assert_eq!(
            guide_tiles(&pages, &SCRIPTING_GUIDES).len(),
            SCRIPTING_GUIDES.len()
        );
    }
}
