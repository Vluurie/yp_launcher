use gpui::{ScrollHandle, Subscription};
use yp_state::ConfigStore;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{Card, CardPadding, Label, LabelSize};

use crate::fields::Marker;
use crate::page::{column, tab_page};
use crate::schema_page::{self, ResetText, TabScope};

const SCOPE: &str = "lodmod";
const FILE: &str = "lodmod.toml";
pub const TAB: TabScope = TabScope::roots(&["graphics"]);

pub struct LodModTab {
    config: Entity<ConfigStore>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl LodModTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx.stores().config.clone();
        let subscriptions = vec![cx.observe(&config, |_, _, cx| cx.notify())];
        Self {
            config,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    fn reset_text(cx: &App) -> ResetText {
        let strings = cx.t();
        ResetText {
            title: SharedString::new_static(strings.lod_mod_reset_confirm_title()),
            body: SharedString::new_static(strings.lod_mod_reset_confirm_body()),
        }
    }

    pub fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = Self::reset_text(cx);
        schema_page::confirm_reset(TAB, &text, &self.config, window, cx);
    }

    fn intro(window: &mut Window, cx: &mut App) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        Card::new()
            .padding(CardPadding::Compact)
            .child(
                Label::new(strings.lod_mod_description())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .line_height(1.4),
            )
            .child(
                v_flex().w_full().pt(sizes.spacing_xs).child(
                    Label::new(strings.comparison_resolution_note())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .italic(),
                ),
            )
            .into_any_element()
    }
}

impl Render for LodModTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let store = self.config.read(cx);
        let loading = schema_page::is_loading(store);
        let reset = TAB
            .has_changes(store)
            .then(|| schema_page::reset_button("lodmod-reset", TAB, Self::reset_text(cx), &self.config, cx));
        let header = schema_page::header(
            strings.header_lod_mod(),
            strings.tooltip_edits_lodmod_toml(),
            Some(FILE),
            reset,
            &self.config,
            cx,
        );
        let content = if loading {
            div().into_any_element()
        } else {
            let mut sections = schema_page::problems(SCOPE, &TAB, &self.config, cx);
            sections.push(Self::intro(window, cx));
            let plans = schema_page::plans(SCOPE, &TAB, self.config.read(cx));
            let mut cards: Vec<AnyElement> = Vec::new();
            for plan in plans {
                let root = TAB.roots.contains(&plan.category.id.as_str());
                let card = schema_page::generic_card(SCOPE, plan, &self.config, Marker::Always, window, cx);
                if root {
                    sections.push(card);
                } else {
                    cards.push(card);
                }
            }
            if !cards.is_empty() {
                sections.push(schema_page::spread(cards, "lodmod-columns", window, cx));
            }
            column(sections, window, cx).into_any_element()
        };
        tab_page("lodmod-scroll", header, &self.scroll, loading, content, window, cx)
    }
}
