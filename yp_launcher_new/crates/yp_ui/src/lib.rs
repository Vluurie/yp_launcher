pub mod actions;
pub mod assets;
pub mod components;
mod erased;
pub mod i18n;
pub mod prelude;
pub mod styled_ext;
pub mod theme;
pub mod traits;

pub use actions::{
    Cancel, Confirm, FocusNext, FocusPrevious, SelectFirst, SelectLast, SelectNext, SelectPrevious,
    SetLocale, ToggleTheme, host_key_binding,
};
pub use assets::{Assets, EmbeddedFile, asset_image, load_fonts};
pub use components::*;
pub use erased::ErasedComponent;
pub use i18n::{ActiveStrings, I18n};
pub use styled_ext::{StyledExt, fill_parent, h_flex, v_flex};
pub use theme::{
    ActiveTheme, Alpha, Appearance, Breakpoint, ButtonHover, Color, Elevation, GlobalTheme, Motion,
    NavHover, ResponsiveExt, Scaled, Severity, Sizes, StatusColor, StatusColors, Surface, Theme,
    ThemeColors, ThemeElevation, ThemeFonts, ThemeId, ThemeSizes, ThemeStyle, easing,
};
pub use traits::{
    ActionHandler, ButtonCommon, ClickHandler, Clickable, Disableable, FixedWidth,
    SelectableButton, ToggleState, Toggleable, TooltipBuilder, ValueHandler, VisibleOnHover,
};
pub use yp_i18n::{Locale, Msg, MsgKey, Strings};

#[doc(hidden)]
pub use gpui as __gpui;

use gpui::App;

pub fn init(cx: &mut App) {
    if let Err(error) = assets::load_fonts(cx) {
        log::error!("could not load bundled fonts: {error}");
    }
    init_without_fonts(cx);
}

pub fn init_for_tests(cx: &mut App) {
    init_without_fonts(cx);
}

fn init_without_fonts(cx: &mut App) {
    theme::init(cx);
    i18n::init(cx);
    actions::init(cx);
    components::init(cx);
}
