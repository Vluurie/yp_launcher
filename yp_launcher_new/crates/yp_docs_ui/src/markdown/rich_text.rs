use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, ClipboardItem, ContentMask, CursorStyle, DispatchPhase, ElementId, Entity, FocusHandle, Hsla,
    IntoElement, KeyBinding, MouseButton, MouseMoveEvent, MouseUpEvent, Pixels, Point,
    RenderOnce, SharedString, StyledText, TextAlign, TextLayout, TextRun, UnderlineStyle, Window, canvas,
    prelude::*, px,
};
use yp_ui::input_actions::{Copy, SelectAll};
use yp_ui::{
    ActiveStrings, ActiveTheme, ContextMenu, host_key_binding, ContextMenuEntry, clamp_selection, right_click_menu,
    v_flex, word_range_at,
};

pub(crate) const TEXT_CONTEXT: &str = "YpDocsText";
const MENU_ID: &str = "menu";
const COPY_KEY: &str = "C";
const SELECT_ALL_KEY: &str = "A";

pub type LinkHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;

pub(crate) fn init(cx: &mut App) {
    let context = Some(TEXT_CONTEXT);
    cx.bind_keys([
        host_key_binding("secondary-c", Copy, context, cx),
        KeyBinding::new("ctrl-insert", Copy, context),
        host_key_binding("secondary-a", SelectAll, context, cx),
    ]);
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkRange {
    pub range: Range<usize>,
    pub href: SharedString,
}

pub fn link_index_at(links: &[LinkRange], index: usize) -> Option<usize> {
    links.iter().position(|link| link.range.contains(&index))
}

pub fn restyle_range(
    runs: &[TextRun],
    range: Range<usize>,
    mut restyle: impl FnMut(&mut TextRun),
) -> Vec<TextRun> {
    if range.is_empty() {
        return runs.to_vec();
    }
    let mut result = Vec::with_capacity(runs.len() + 2);
    let mut offset = 0;
    for run in runs {
        let run_range = offset..offset + run.len;
        offset = run_range.end;
        let start = range.start.clamp(run_range.start, run_range.end);
        let end = range.end.clamp(run_range.start, run_range.end);
        if start >= end {
            result.push(run.clone());
            continue;
        }
        if start > run_range.start {
            let mut before = run.clone();
            before.len = start - run_range.start;
            result.push(before);
        }
        let mut inside = run.clone();
        inside.len = end - start;
        restyle(&mut inside);
        result.push(inside);
        if end < run_range.end {
            let mut after = run.clone();
            after.len = run_range.end - end;
            result.push(after);
        }
    }
    result
}

pub fn runs_cover(runs: &[TextRun], text: &str) -> bool {
    let mut offset = 0;
    for run in runs {
        offset += run.len;
        if run.len == 0 || !text.is_char_boundary(offset.min(text.len())) {
            return false;
        }
    }
    offset == text.len()
}

fn reveal_runs(runs: &mut Vec<TextRun>, prefix_len: usize, text_len: usize) -> Option<TextRun> {
    let mut end = 0;
    let edge = runs
        .iter()
        .find(|run| {
            end += run.len;
            prefix_len < end
        })
        .cloned();
    *runs = restyle_range(runs, prefix_len..text_len, |run| {
        run.color.a = 0.;
        run.background_color = None;
        run.underline = None;
        run.strikethrough = None;
    });
    edge
}

struct TextState {
    anchor: usize,
    head: usize,
    selecting: bool,
    pressed_link: Option<usize>,
    hovered_link: Option<usize>,
    focus_handle: FocusHandle,
}

impl TextState {
    fn range(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }
}

pub fn selected_text(text: &str, range: Range<usize>) -> &str {
    text.get(clamp_selection(text, range)).unwrap_or_default()
}

pub fn text_to_copy(text: &str, range: Range<usize>) -> &str {
    let selected = selected_text(text, range);
    if selected.is_empty() { text } else { selected }
}

fn shortcut_hint(key: &str, cx: &App) -> SharedString {
    let strings = cx.t();
    let modifier = if cfg!(target_os = "macos") {
        strings.ui_key_cmd()
    } else {
        strings.ui_key_ctrl()
    };
    SharedString::from(strings.ui_shortcut(modifier, key))
}

fn text_menu(
    state: &Entity<TextState>,
    text: &SharedString,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ContextMenu> {
    let (range, focus_handle) = {
        let state = state.read(cx);
        (state.range(), state.focus_handle.clone())
    };
    window.focus(&focus_handle, cx);
    let copied = text_to_copy(text, range).to_owned();
    let has_text = !text.is_empty();
    let strings = cx.t();
    let copy_hint = shortcut_hint(COPY_KEY, cx);
    let select_all_hint = shortcut_hint(SELECT_ALL_KEY, cx);
    let state = state.clone();
    let end = text.len();
    ContextMenu::build(window, cx, move |menu, _, _| {
        menu.item(
            ContextMenuEntry::new(strings.docs_copy())
                .shortcut(copy_hint)
                .disabled(!has_text)
                .handler(move |_, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                }),
        )
        .item(
            ContextMenuEntry::new(strings.ui_select_all())
                .shortcut(select_all_hint)
                .disabled(!has_text)
                .handler(move |window, cx| {
                    window.focus(&focus_handle, cx);
                    state.update(cx, |state, cx| {
                        state.anchor = 0;
                        state.head = end;
                        state.selecting = false;
                        cx.notify();
                    });
                }),
        )
    })
}

fn index_at(layout: &TextLayout, position: Point<Pixels>) -> usize {
    match layout.index_for_position(position) {
        Ok(index) | Err(index) => index,
    }
}

fn exact_index_at(layout: &TextLayout, position: Point<Pixels>) -> Option<usize> {
    layout.index_for_position(position).ok()
}

fn pointer_capture(
    active: bool,
    on_move: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
    on_up: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            if !active {
                return;
            }
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Capture {
                    on_move(event, window, cx);
                }
            });
            window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Capture {
                    on_up(event, window, cx);
                }
            });
        },
    )
    .absolute()
    .inset_0()
}

pub struct RichBlock {
    id: ElementId,
    text: SharedString,
    runs: Vec<TextRun>,
    links: Vec<LinkRange>,
    on_link: Option<LinkHandler>,
    link_hover: Option<Hsla>,
}

yp_ui::impl_into_element!(RichBlock);

impl RichBlock {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>, runs: Vec<TextRun>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            runs,
            links: Vec::new(),
            on_link: None,
            link_hover: None,
        }
    }

    pub fn links(mut self, links: Vec<LinkRange>, handler: LinkHandler) -> Self {
        self.links = links;
        self.on_link = Some(handler);
        self
    }

    pub fn link_hover_color(mut self, color: Hsla) -> Self {
        self.link_hover = Some(color);
        self
    }
}

impl RenderOnce for RichBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| TextState {
            anchor: 0,
            head: 0,
            selecting: false,
            pressed_link: None,
            hovered_link: None,
            focus_handle: cx.focus_handle(),
        });
        let (selection, selecting, hovered_link, focus_handle) = {
            let state = state.read(cx);
            (
                clamp_selection(&self.text, state.range()),
                state.selecting,
                state.hovered_link,
                state.focus_handle.clone(),
            )
        };
        let focused = focus_handle.is_focused(window);
        let selection_color = if focused {
            theme.colors.accent_primary.opacity(theme.alpha.soft)
        } else {
            theme.colors.accent_primary.opacity(theme.alpha.hover)
        };
        let mut runs = if runs_cover(&self.runs, &self.text) {
            self.runs
        } else {
            Vec::new()
        };
        let hovered = hovered_link.and_then(|index| self.links.get(index).cloned());
        if let Some(link) = &hovered {
            let hover_color = self.link_hover.unwrap_or(theme.colors.accent_highlight);
            runs = restyle_range(&runs, link.range.clone(), |run| {
                run.color = hover_color;
                run.underline = Some(UnderlineStyle {
                    thickness: px(1.),
                    color: Some(hover_color),
                    wavy: false,
                });
            });
        }
        if !runs.is_empty() {
            runs = restyle_range(&runs, selection.clone(), |run| {
                run.background_color = Some(selection_color);
            });
        }
        let edge = yp_ui::nier::page_text_frame(&self.text, window, cx).and_then(|frame| {
            if runs.is_empty() {
                runs.push(window.text_style().to_run(self.text.len()));
            }
            let mut run = reveal_runs(&mut runs, frame.prefix_len, self.text.len())?;
            if window.text_style().text_align != TextAlign::Left
                || frame.glyph.trim().is_empty()
                || self.text[frame.prefix_len..].starts_with(char::is_whitespace)
            {
                return None;
            }
            run.len = frame.glyph.len();
            Some((
                frame.prefix_len,
                SharedString::from(frame.glyph.to_owned()),
                run,
            ))
        });
        let styled = if runs.is_empty() {
            StyledText::new(self.text.clone())
        } else {
            StyledText::new(self.text.clone()).with_runs(runs)
        };
        let layout = styled.layout().clone();
        let text = self.text.clone();
        let links = Rc::new(self.links);
        let on_link = self.on_link;

        let down_state = state.clone();
        let down_layout = layout.clone();
        let down_text = text.clone();
        let down_links = links.clone();
        let down_focus = focus_handle.clone();
        let hover_state = state.clone();
        let hover_layout = layout.clone();
        let hover_links = links.clone();
        let leave_state = state.clone();
        let move_state = state.clone();
        let move_layout = layout.clone();
        let up_state = state.clone();
        let up_layout = layout.clone();
        let up_links = links.clone();
        let out_state = state.clone();
        let copy_state = state.clone();
        let copy_text = text.clone();
        let all_state = state.clone();
        let all_len = text.len();
        let menu_state = state.clone();
        let menu_text = text.clone();
        let menu_id = ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), MENU_ID.into());

        let block = v_flex()
            .id(self.id)
            .key_context(TEXT_CONTEXT)
            .track_focus(&focus_handle)
            .relative()
            .min_w_0()
            .cursor(if hovered.is_some() {
                CursorStyle::PointingHand
            } else {
                CursorStyle::IBeam
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                cx.stop_propagation();
                window.focus(&down_focus, cx);
                let index = index_at(&down_layout, event.position).min(down_text.len());
                let pressed_link = exact_index_at(&down_layout, event.position)
                    .and_then(|index| link_index_at(&down_links, index));
                down_state.update(cx, |state, cx| {
                    state.pressed_link = pressed_link;
                    match event.click_count {
                        2 => {
                            let word = word_range_at(&down_text, index);
                            state.anchor = word.start;
                            state.head = word.end;
                            state.selecting = false;
                        }
                        count if count >= 3 => {
                            state.anchor = 0;
                            state.head = down_text.len();
                            state.selecting = false;
                        }
                        _ => {
                            if event.modifiers.shift {
                                state.head = index;
                            } else {
                                state.anchor = index;
                                state.head = index;
                            }
                            state.selecting = true;
                        }
                    }
                    cx.notify();
                });
            })
            .on_mouse_move(move |event, _, cx| {
                if hover_links.is_empty() {
                    return;
                }
                let link = exact_index_at(&hover_layout, event.position)
                    .and_then(|index| link_index_at(&hover_links, index));
                if hover_state.read(cx).hovered_link != link {
                    hover_state.update(cx, |state, cx| {
                        state.hovered_link = link;
                        cx.notify();
                    });
                }
            })
            .on_hover(move |hovered, _, cx| {
                if !*hovered && leave_state.read(cx).hovered_link.is_some() {
                    leave_state.update(cx, |state, cx| {
                        state.hovered_link = None;
                        cx.notify();
                    });
                }
            })
            .on_mouse_down_out(move |_, _, cx| {
                let has_selection = {
                    let state = out_state.read(cx);
                    state.anchor != state.head
                };
                if has_selection {
                    out_state.update(cx, |state, cx| {
                        state.anchor = 0;
                        state.head = 0;
                        cx.notify();
                    });
                }
            })
            .on_action(move |_: &Copy, _, cx| {
                let range = clamp_selection(&copy_text, copy_state.read(cx).range());
                if !range.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(copy_text[range].to_owned()));
                }
            })
            .on_action(move |_: &SelectAll, _, cx| {
                all_state.update(cx, |state, cx| {
                    state.anchor = 0;
                    state.head = all_len;
                    cx.notify();
                });
            })
            .child(styled)
            .children(edge.map(|(index, glyph, run)| {
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, cx| {
                        if bounds.intersects(&window.content_mask().bounds)
                            && let Some(origin) = layout.position_for_index(index)
                        {
                            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                                let size = window.text_style().font_size.to_pixels(window.rem_size());
                                let line = window.text_system().shape_line(glyph, size, &[run], None);
                                let _ = line.paint(
                                    origin, layout.line_height(), TextAlign::Left, None, window, cx,
                                );
                            });
                        }
                    },
                )
                .absolute()
                .inset_0()
            }))
            .child(pointer_capture(
                selecting,
                move |event, _, cx| {
                    let index = index_at(&move_layout, event.position);
                    if move_state.read(cx).head != index {
                        move_state.update(cx, |state, cx| {
                            state.head = index;
                            cx.notify();
                        });
                    }
                },
                move |event, window, cx| {
                    let released_link = exact_index_at(&up_layout, event.position)
                        .and_then(|index| link_index_at(&up_links, index));
                    let clicked = {
                        let state = up_state.read(cx);
                        (state.anchor == state.head && state.pressed_link == released_link)
                            .then_some(released_link)
                            .flatten()
                    };
                    up_state.update(cx, |state, cx| {
                        state.selecting = false;
                        state.pressed_link = None;
                        cx.notify();
                    });
                    if let Some(index) = clicked
                        && let Some(link) = up_links.get(index)
                        && let Some(handler) = &on_link
                    {
                        handler(&link.href, window, cx);
                    }
                },
            ));
        right_click_menu::<ContextMenu>(menu_id)
            .trigger(block)
            .menu(move |window, cx| text_menu(&menu_state, &menu_text, window, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Font, font};

    fn run(len: usize) -> TextRun {
        TextRun {
            len,
            font: font("Test"),
            color: Hsla::default(),
            background_color: None,
            underline: None,
            strikethrough: None,
        }
    }

    fn marked(runs: &[TextRun]) -> Vec<(usize, bool)> {
        runs.iter()
            .map(|run| (run.len, run.background_color.is_some()))
            .collect()
    }

    fn mark(run: &mut TextRun) {
        run.background_color = Some(Hsla::default());
    }

    #[test]
    fn a_range_inside_one_run_splits_it_in_three() {
        let runs = restyle_range(&[run(10)], 3..6, mark);
        assert_eq!(marked(&runs), vec![(3, false), (3, true), (4, false)]);
    }

    #[test]
    fn typing_masks_only_the_suffix_and_keeps_link_and_selection_offsets() {
        let text = "Aé界";
        let mut link = run("é界".len());
        link.color = gpui::rgb(0x123456).into();
        mark(&mut link);
        let mut runs = vec![run(1), link.clone()];
        let index = "Aé".len();
        let edge = reveal_runs(&mut runs, index, text.len()).unwrap();
        assert_eq!(edge.color, link.color);
        assert!(runs_cover(&runs, text));
        assert_eq!(marked(&runs), vec![(1, false), (2, true), (3, false)]);
        assert_eq!(runs[1].color, link.color);
        assert_eq!(runs[2].color.a, 0.);
        assert_eq!(selected_text(text, 1..text.len()), "é界");
    }

    #[gpui::test]
    fn inline_code_appears_with_its_text_and_is_immediately_visible_with_reduced_motion(
        cx: &mut gpui::TestAppContext,
    ) {
        struct Article;
        impl gpui::Render for Article {
            fn render(&mut self, window: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let prefix = "An article paragraph with inline code at the end. ";
                let mut code = window.text_style().to_run(4);
                code.background_color = Some(gpui::rgb(0x6839bd).into());
                let block = RichBlock::new(
                    "article-text",
                    format!("{prefix}code"),
                    vec![window.text_style().to_run(prefix.len()), code],
                );
                yp_ui::nier::page_text_scope(block, std::time::Instant::now())
            }
        }
        cx.update(yp_ui::init_for_tests);
        let (article, cx) = cx.add_window_view(|_, _| Article);
        cx.run_until_parked();
        let code_visible = |cx: &mut gpui::VisualTestContext| cx.update(|window, _| {
            let color: gpui::Background = gpui::rgb(0x6839bd).into();
            window.painted_quads().iter().any(|quad| quad.background == color)
        });
        assert!(!code_visible(cx));
        article.update(cx, |_, cx| {
            cx.set_global(yp_ui::nier::ReducedMotion(true));
            cx.notify();
        });
        cx.run_until_parked();
        assert!(code_visible(cx));
    }

    #[test]
    fn a_range_across_runs_keeps_every_boundary_and_the_total_length() {
        let runs = restyle_range(&[run(4), run(4), run(4)], 2..10, mark);
        assert_eq!(
            marked(&runs),
            vec![(2, false), (2, true), (4, true), (2, true), (2, false)]
        );
        assert_eq!(runs.iter().map(|run| run.len).sum::<usize>(), 12);
    }

    #[test]
    fn empty_and_out_of_bounds_ranges_change_nothing_or_clamp() {
        assert_eq!(marked(&restyle_range(&[run(5)], 2..2, mark)), vec![(5, false)]);
        assert_eq!(marked(&restyle_range(&[run(5)], 3..99, mark)), vec![(3, false), (2, true)]);
        assert_eq!(marked(&restyle_range(&[run(5)], 0..5, mark)), vec![(5, true)]);
    }

    #[test]
    fn runs_must_cover_the_text_exactly_and_on_char_boundaries() {
        let font: Font = font("Test");
        let text = "aé文";
        let exact = vec![run(1), run(2), run(3)];
        assert!(runs_cover(&exact, text));
        assert!(!runs_cover(&[run(1), run(1), run(4)], text));
        assert!(!runs_cover(&[run(5)], text));
        assert!(!runs_cover(&[run(6), run(0)], text));
        assert!(runs_cover(&[], ""));
        assert_eq!(font.family.as_ref(), "Test");
    }

    #[test]
    fn the_selected_text_is_clamped_to_the_text_and_to_char_boundaries() {
        assert_eq!(selected_text("alpha beta", 6..10), "beta");
        assert_eq!(selected_text("alpha beta", 3..3), "");
        assert_eq!(selected_text("alpha", 2..99), "pha");
        assert_eq!(selected_text("a\u{e9}\u{6587}", 0..3), "a\u{e9}");
        assert_eq!(selected_text("", 0..4), "");
    }

    fn clipboard(cx: &mut gpui::VisualTestContext) -> Option<String> {
        cx.read_from_clipboard().and_then(|item| item.text())
    }

    #[gpui::test]
    fn the_text_menu_copies_the_selection_or_the_block_and_selects_everything(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(yp_ui::init_for_tests);
        let cx = cx.add_empty_window();
        let text = SharedString::from("alpha beta");
        let new_state = |cx: &mut gpui::VisualTestContext| {
            cx.update(|_, cx| {
                cx.new(|cx| TextState {
                    anchor: 0,
                    head: 0,
                    selecting: false,
                    pressed_link: None,
                    hovered_link: None,
                    focus_handle: cx.focus_handle(),
                })
            })
        };
        let state = new_state(cx);

        let menu = cx.update(|window, cx| text_menu(&state, &text, window, cx));
        assert_eq!(menu.read_with(cx, |menu, _| menu.item_count()), 2);
        cx.update(|window, cx| menu.update(cx, |menu, cx| menu.activate(0, window, cx)));
        assert_eq!(clipboard(cx).as_deref(), Some("alpha beta"));

        state.update(cx, |state, _| {
            state.anchor = 10;
            state.head = 6;
        });
        let menu = cx.update(|window, cx| text_menu(&state, &text, window, cx));
        cx.update(|window, cx| menu.update(cx, |menu, cx| menu.activate(0, window, cx)));
        assert_eq!(clipboard(cx).as_deref(), Some("beta"));

        cx.update(|window, cx| menu.update(cx, |menu, cx| menu.activate(1, window, cx)));
        assert_eq!(state.read_with(cx, |state, _| state.range()), 0..10);
        let focused = cx.update(|window, cx| state.read(cx).focus_handle.is_focused(window));
        assert!(focused);

        let empty_state = new_state(cx);
        let empty = SharedString::default();
        let menu = cx.update(|window, cx| text_menu(&empty_state, &empty, window, cx));
        cx.update(|window, cx| {
            menu.update(cx, |menu, cx| {
                menu.activate(0, window, cx);
                menu.activate(1, window, cx);
            });
        });
        assert_eq!(clipboard(cx).as_deref(), Some("beta"));
    }

    #[test]
    fn copying_without_a_selection_takes_the_whole_block() {
        assert_eq!(text_to_copy("alpha beta", 0..0), "alpha beta");
        assert_eq!(text_to_copy("alpha beta", 0..5), "alpha");
        assert_eq!(text_to_copy("", 0..3), "");
    }

    #[test]
    fn links_are_found_by_byte_index() {
        let links = vec![
            LinkRange {
                range: 2..5,
                href: "a.md".into(),
            },
            LinkRange {
                range: 8..9,
                href: "#b".into(),
            },
        ];
        assert_eq!(link_index_at(&links, 1), None);
        assert_eq!(link_index_at(&links, 2), Some(0));
        assert_eq!(link_index_at(&links, 4), Some(0));
        assert_eq!(link_index_at(&links, 5), None);
        assert_eq!(link_index_at(&links, 8), Some(1));
    }
}
