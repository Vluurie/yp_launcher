use std::rc::Rc;

use gpui::{Action, App, KeyBinding, KeyBindingContextPredicate, NoAction, actions};
use yp_i18n::Locale;

use crate::i18n::I18n;
use crate::theme;

actions!(yp, [ToggleTheme]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, gpui::Action)]
#[action(namespace = yp, no_json)]
pub struct SetLocale {
    pub locale: Locale,
}

actions!(
    yp_ui,
    [
        Cancel,
        Confirm,
        SelectNext,
        SelectPrevious,
        SelectFirst,
        SelectLast,
        FocusNext,
        FocusPrevious,
    ]
);

pub(crate) const MENU_CONTEXT: &str = "YpMenu";
pub(crate) const MODAL_CONTEXT: &str = "YpModal";
pub(crate) const SHEET_CONTEXT: &str = "YpSheet";
pub(crate) const ROOT_CONTEXT: &str = "YpRoot";
pub(crate) const TOUR_CONTEXT: &str = "YpTour";
pub(crate) const TOAST_HISTORY_CONTEXT: &str = "YpToastHistory";
pub(crate) const ACTIVATION_CONTEXT: &str = "YpKeyboardActivate";

pub fn host_key_binding<A: Action>(
    keystrokes: &str,
    action: A,
    context: Option<&str>,
    cx: &App,
) -> KeyBinding {
    let predicate = context.map(|context| {
        Rc::new(KeyBindingContextPredicate::parse(context).expect("a valid key context"))
    });
    KeyBinding::load(keystrokes, Box::new(action), predicate, false, None, cx)
        .expect("a valid keystroke")
}

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", Cancel, Some(MENU_CONTEXT)),
        KeyBinding::new("enter", Confirm, Some(MENU_CONTEXT)),
        KeyBinding::new("down", SelectNext, Some(MENU_CONTEXT)),
        KeyBinding::new("up", SelectPrevious, Some(MENU_CONTEXT)),
        KeyBinding::new("home", SelectFirst, Some(MENU_CONTEXT)),
        KeyBinding::new("end", SelectLast, Some(MENU_CONTEXT)),
        KeyBinding::new("tab", Cancel, Some(MENU_CONTEXT)),
        KeyBinding::new("shift-tab", Cancel, Some(MENU_CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(MODAL_CONTEXT)),
        KeyBinding::new("enter", Confirm, Some(MODAL_CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(SHEET_CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(TOAST_HISTORY_CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(TOUR_CONTEXT)),
        KeyBinding::new("enter", Confirm, Some(TOUR_CONTEXT)),
        KeyBinding::new("right", SelectNext, Some(TOUR_CONTEXT)),
        KeyBinding::new("left", SelectPrevious, Some(TOUR_CONTEXT)),
        KeyBinding::new("tab", FocusNext, Some(ROOT_CONTEXT)),
        KeyBinding::new("shift-tab", FocusPrevious, Some(ROOT_CONTEXT)),
        KeyBinding::new("enter", NoAction, Some(ACTIVATION_CONTEXT)),
    ]);
    cx.on_action(|_: &ToggleTheme, cx| theme::toggle_theme(cx));
    cx.on_action(|action: &SetLocale, cx| I18n::set(action.locale, cx));
}
