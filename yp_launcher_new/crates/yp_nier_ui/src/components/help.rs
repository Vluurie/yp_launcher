use super::control::mix;
use crate::Theme;
use crate::{INK, LIGHT, PAPER, Rect, Surface};
use gpui::{App, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, relative};
use std::rc::Rc;

type Action = Rc<dyn Fn(&mut Window, &mut App)>;

fn help_bar(s: &mut Surface, text: &str, prompts: &[ButtonPrompt], y: f32) -> Vec<Rect> {
    s.nine_slice(
        "panel",
        Rect::new(48., y - 34., 1508., 74.),
        [20., 20.],
        [10., 10.],
        mix(INK, PAPER, 0.5),
    );
    s.nine_slice(
        "panel",
        Rect::new(45., y - 37., 1508., 74.),
        [20., 20.],
        [10., 10.],
        LIGHT,
    );
    s.sprite("margin", Rect::new(35., y - 45., 50., 90.), INK);
    s.sprite("panel", Rect::new(1522., y + 10., 19.6, 19.6), INK);
    let prompt_width: f32 = prompts
        .iter()
        .map(|p| {
            Theme::bundled().measure(&p.label, 36, 0.75)[0]
                + p.icons
                    .iter()
                    .filter_map(|k| Theme::bundled().sprite_size(k))
                    .map(|d| d[0] as f32 * 0.9 + 3.)
                    .sum::<f32>()
                + 34.
        })
        .sum();
    s.clip(
        Rect::new(94., y - 25., (1408. - prompt_width).max(0.), 50.),
        |s| s.text(text, 36, 0.8, 96., y, INK),
    );
    let mut right = 1515.;
    let mut regions = Vec::with_capacity(prompts.len());
    for prompt in prompts.iter().rev() {
        let end = right;
        let text_width = Theme::bundled().measure(&prompt.label, 36, 0.75)[0];
        right -= text_width;
        s.text(&prompt.label, 36, 0.75, right, y, INK);
        right -= 5.;
        for key in prompt.icons.iter().rev() {
            if let Some([w, h]) = Theme::bundled().sprite_size(key) {
                let width = w as f32 * 0.9;
                right -= width;
                s.sprite(
                    key,
                    Rect::new(right, y - h as f32 * 0.45, width, h as f32 * 0.9),
                    INK,
                );
                right -= 3.;
            }
        }
        regions.push(Rect::new(right, y - 21., end - right, 42.));
        right -= 29.;
    }
    regions.reverse();
    regions
}

#[derive(Clone)]
pub struct ButtonPrompt {
    pub icons: Vec<SharedString>,
    pub label: SharedString,
    action: Option<Action>,
}
impl ButtonPrompt {
    pub fn new(icon: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            icons: vec![icon.into()],
            label: label.into(),
            action: None,
        }
    }
    pub fn with_key(mut self, icon: impl Into<SharedString>) -> Self {
        self.icons.push(icon.into());
        self
    }
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.action = Some(Rc::new(f));
        self
    }
}
#[derive(IntoElement)]
pub struct HelpBar {
    text: SharedString,
    prompts: Vec<ButtonPrompt>,
}
impl HelpBar {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            prompts: vec![
                ButtonPrompt::new("left", "Select").with_key("right"),
                ButtonPrompt::new("confirm", "Confirm"),
                ButtonPrompt::new("back", "Back"),
            ],
        }
    }
    pub fn vertical(mut self) -> Self {
        if let Some(first) = self.prompts.first_mut() {
            *first = ButtonPrompt::new("up", "Select").with_key("down");
        }
        self
    }
    pub fn prompts(mut self, prompts: Vec<ButtonPrompt>) -> Self {
        self.prompts = prompts;
        self
    }
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        if let Some(prompt) = self.prompts.last_mut() {
            prompt.action = Some(Rc::new(f));
        }
        self
    }
    pub fn on_confirm(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        if let Some(prompt) = self.prompts.get_mut(1) {
            prompt.action = Some(Rc::new(f));
        }
        self
    }
}
impl RenderOnce for HelpBar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut s = Surface::new(1600., 100.);
        let regions = help_bar(&mut s, &self.text, &self.prompts, 50.);
        let hits = self
            .prompts
            .into_iter()
            .zip(regions)
            .enumerate()
            .filter_map(|(i, (p, r))| {
                let action = p.action?;
                Some(
                    div()
                        .id(("help-prompt", i))
                        .absolute()
                        .left(relative(r.x / 1600.))
                        .top(relative(r.y / 100.))
                        .w(relative(r.w / 1600.))
                        .h(relative(r.h / 100.))
                        .cursor_pointer()
                        .on_click(move |_, w, cx| action(w, cx)),
                )
            });
        div().relative().size_full().child(s).children(hits)
    }
}
