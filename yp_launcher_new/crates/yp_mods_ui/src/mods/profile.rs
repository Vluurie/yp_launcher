use gpui::Task;
use yp_mods::ModProfile;
use yp_mods::profiles::{is_valid_name, sanitize_name};
use yp_state::prelude::*;
use yp_state::{PlayState, Stores};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonStyle, ConfirmOptions, ConfirmTone, Msg, MsgKey, PickItem, PickOptions, PromptOptions,
    Select, SelectItem, Spinner, TintColor, Tooltip, TourAnchorExt, confirm, pick_one, prompt_text,
};

use crate::labels::profile_error_key;
use crate::mods::dialogs::has_only_allowed_name_characters;
use crate::mods::tutorial::ANCHOR_PROFILES;
use crate::notify::toast;

pub const PROFILE_NAME_MAX_LENGTH: usize = 48;

pub fn sorted_profile_names(profiles: &[ModProfile]) -> Vec<String> {
    let mut names: Vec<String> = profiles
        .iter()
        .map(|profile| profile.name.clone())
        .collect();
    names.sort_by_key(|name| name.to_lowercase());
    names
}

pub fn delete_candidates(profiles: &[ModProfile], active: &str) -> Vec<String> {
    let mut names: Vec<String> = profiles
        .iter()
        .filter(|profile| profile.name != active)
        .map(|profile| profile.name.clone())
        .collect();
    names.sort();
    names
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileNameError {
    Empty,
    Invalid,
    Collision,
}

impl ProfileNameError {
    pub fn key(self) -> MsgKey {
        match self {
            ProfileNameError::Empty => MsgKey::ModProfileErrorNameEmpty,
            ProfileNameError::Invalid => MsgKey::ModProfileErrorNameInvalid,
            ProfileNameError::Collision => MsgKey::ModProfileErrorNameCollision,
        }
    }
}

pub fn check_profile_name(raw: &str, existing: &[String]) -> Result<String, ProfileNameError> {
    if !has_only_allowed_name_characters(raw) {
        return Err(ProfileNameError::Invalid);
    }
    let sanitized = sanitize_name(raw);
    if sanitized.is_empty() {
        return Err(ProfileNameError::Empty);
    }
    if !is_valid_name(&sanitized) {
        return Err(ProfileNameError::Invalid);
    }
    if existing.contains(&sanitized) {
        return Err(ProfileNameError::Collision);
    }
    Ok(sanitized)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileActions {
    pub game_running: bool,
    pub can_switch: bool,
    pub can_create: bool,
    pub can_rename: bool,
    pub can_delete: bool,
    pub more_than_one: bool,
}

pub fn profile_actions(
    profile_count: usize,
    active_name: &str,
    loading: bool,
    has_game: bool,
    play_state: PlayState,
) -> ProfileActions {
    let game_running = matches!(play_state, PlayState::Running | PlayState::Loading);
    let can_act = !loading && has_game && !game_running;
    let more_than_one = profile_count > 1 && !active_name.is_empty();
    ProfileActions {
        game_running,
        can_switch: can_act,
        can_create: can_act,
        can_rename: can_act && profile_count > 0,
        can_delete: can_act && more_than_one,
        more_than_one,
    }
}

fn prompt_profile_name(
    title: &'static str,
    initial: &str,
    existing: Vec<String>,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<String>> {
    let strings = cx.t();
    let options = PromptOptions::new(title)
        .initial(initial.to_owned())
        .placeholder(strings.mod_profile_new_dialog_hint())
        .max_len(PROFILE_NAME_MAX_LENGTH)
        .validate(move |text| {
            if text.trim().is_empty() {
                return Ok(());
            }
            check_profile_name(text, &existing)
                .map(drop)
                .map_err(|error| SharedString::new_static(error.key().text(strings)))
        });
    prompt_text(options, window, cx)
}

fn report(succeeded: bool, done: Msg, stores: &Stores, cx: &mut App) {
    if succeeded {
        toast(cx, "profile", done, Severity::Success);
        return;
    }
    let key = profile_error_key(stores.profiles.read(cx).error().map(|error| error.as_ref()));
    toast(cx, "profile", key, Severity::Error);
}

fn profile_names(stores: &Stores, cx: &App) -> Vec<String> {
    stores
        .profiles
        .read(cx)
        .profiles()
        .iter()
        .map(|profile| profile.name.clone())
        .collect()
}

pub fn switch_profile(name: String, window: &mut Window, cx: &mut App) {
    let stores = cx.stores().clone();
    if stores.profiles.read(cx).active_name() == name {
        return;
    }
    let switching = stores
        .profiles
        .update(cx, |profiles, cx| profiles.switch_to(&name, cx));
    window
        .spawn(cx, async move |cx| {
            let succeeded = switching.await;
            cx.update(|_, cx| {
                report(
                    succeeded,
                    Msg::ModProfileSwitchedToast { name },
                    &stores,
                    cx,
                )
            })
            .ok();
        })
        .detach();
}

pub fn create_profile(window: &mut Window, cx: &mut App) {
    let stores = cx.stores().clone();
    let existing = profile_names(&stores, cx);
    let title = cx.t().mod_profile_new_dialog_title();
    let asking = prompt_profile_name(title, "", existing, window, cx);
    window
        .spawn(cx, async move |cx| {
            let Some(raw) = asking.await else {
                return;
            };
            let name = sanitize_name(&raw);
            let Ok(creating) = cx.update(|_, cx| {
                stores
                    .profiles
                    .update(cx, |profiles, cx| profiles.create(&name, cx))
            }) else {
                return;
            };
            let succeeded = creating.await;
            cx.update(|_, cx| report(succeeded, Msg::ModProfileCreatedToast { name }, &stores, cx))
                .ok();
        })
        .detach();
}

pub fn rename_profile(window: &mut Window, cx: &mut App) {
    let stores = cx.stores().clone();
    let active = stores.profiles.read(cx).active_name().to_owned();
    let existing: Vec<String> = profile_names(&stores, cx)
        .into_iter()
        .filter(|name| *name != active)
        .collect();
    let title = cx.t().mod_profile_rename_dialog_title();
    let asking = prompt_profile_name(title, &active, existing, window, cx);
    window
        .spawn(cx, async move |cx| {
            let Some(raw) = asking.await else {
                return;
            };
            let name = sanitize_name(&raw);
            if name == active {
                return;
            }
            let Ok(renaming) = cx.update(|_, cx| {
                stores
                    .profiles
                    .update(cx, |profiles, cx| profiles.rename(&active, &name, cx))
            }) else {
                return;
            };
            let succeeded = renaming.await;
            cx.update(|_, cx| report(succeeded, Msg::ModProfileRenamedToast { name }, &stores, cx))
                .ok();
        })
        .detach();
}

pub fn delete_profile(window: &mut Window, cx: &mut App) {
    let stores = cx.stores().clone();
    let strings = cx.t();
    let candidates = {
        let profiles = stores.profiles.read(cx);
        delete_candidates(profiles.profiles(), profiles.active_name())
    };
    if candidates.is_empty() {
        return;
    }
    let items = candidates
        .into_iter()
        .map(|name| PickItem::new(name.clone(), name).icon(IconName::FolderOutlined))
        .collect();
    let picking = pick_one(
        PickOptions::new(strings.mod_profile_delete_dialog_title(), items)
            .cancel_label(strings.button_cancel()),
        window,
        cx,
    );
    window
        .spawn(cx, async move |cx| {
            let Some(name) = picking.await else {
                return;
            };
            let Ok(confirming) = cx.update(|window, cx| {
                let strings = cx.t();
                confirm(
                    ConfirmOptions::new(
                        strings.mod_profile_delete_dialog_title(),
                        strings.mod_profile_delete_dialog_body(&name),
                    )
                    .confirm_label(strings.mod_profile_delete_confirm())
                    .cancel_label(strings.button_cancel())
                    .tone(ConfirmTone::Danger),
                    window,
                    cx,
                )
            }) else {
                return;
            };
            if !confirming.await {
                return;
            }
            let Ok(deleting) = cx.update(|_, cx| {
                stores
                    .profiles
                    .update(cx, |profiles, cx| profiles.delete(&name, cx))
            }) else {
                return;
            };
            let succeeded = deleting.await;
            cx.update(|_, cx| report(succeeded, Msg::ModProfileDeletedToast { name }, &stores, cx))
                .ok();
        })
        .detach();
}

#[derive(Default)]
pub struct ProfileSelector;

yp_ui::impl_into_element!(ProfileSelector);

impl ProfileSelector {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for ProfileSelector {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let stores = cx.stores().clone();
        let (names, active, loading) = {
            let profiles = stores.profiles.read(cx);
            (
                sorted_profile_names(profiles.profiles()),
                profiles.active_name().to_owned(),
                profiles.is_loading(),
            )
        };
        let (has_game, play_state) = {
            let session = stores.session.read(cx);
            (session.game().is_some(), session.play_state())
        };
        let actions = profile_actions(names.len(), &active, loading, has_game, play_state);
        let shown = if names.contains(&active) {
            active
        } else {
            names.first().cloned().unwrap_or_default()
        };
        let locked = strings.mod_profile_locked_running();
        let tooltip = |unlocked: &'static str| {
            Tooltip::text(if actions.game_running {
                locked
            } else {
                unlocked
            })
        };
        let delete_tooltip = if actions.more_than_one {
            strings.mod_profile_delete_button()
        } else {
            strings.mod_profile_error_delete_last()
        };
        h_flex()
            .relative()
            .flex_none()
            .gap(sizes.spacing_md)
            .child(
                Label::new(strings.mod_profile_label())
                    .size(LabelSize::Small)
                    .color(Color::Secondary)
                    .semibold(),
            )
            .when(!names.is_empty(), |this| {
                this.child(
                    Select::<String>::new("mods-profile-select", shown)
                        .items(
                            names
                                .iter()
                                .map(|name| SelectItem::new(name.clone(), name.clone())),
                        )
                        .highlight(true)
                        .min_width(sizes.config_input_width)
                        .max_width(sizes.config_input_width * 2.)
                        .disabled(!actions.can_switch)
                        .on_select(|name: &String, window, cx| {
                            switch_profile(name.clone(), window, cx)
                        }),
                )
            })
            .when(loading, |this| {
                this.child(Spinner::new().size(IconSize::Small))
            })
            .child(
                IconButton::new("mods-profile-new", IconName::Add)
                    .controller_support(false)
                    .bordered(true)
                    .disabled(!actions.can_create)
                    .tooltip(tooltip(strings.mod_profile_new_button()))
                    .on_click(|_, window, cx| create_profile(window, cx)),
            )
            .child(
                IconButton::new("mods-profile-rename", IconName::EditOutlined)
                    .controller_support(false)
                    .bordered(true)
                    .disabled(!actions.can_rename)
                    .tooltip(tooltip(strings.mod_profile_rename_button()))
                    .on_click(|_, window, cx| rename_profile(window, cx)),
            )
            .child(
                IconButton::new("mods-profile-delete", IconName::DeleteOutline)
                    .style(ButtonStyle::Tinted(TintColor::Error))
                    .disabled(!actions.can_delete)
                    .tooltip(tooltip(delete_tooltip))
                    .on_click(|_, window, cx| delete_profile(window, cx)),
            )
            .tour_anchor(ANCHOR_PROFILES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(name: &str) -> ModProfile {
        ModProfile {
            name: name.to_owned(),
            created_at: Default::default(),
        }
    }

    #[test]
    fn profile_names_sort_without_regard_to_case() {
        let profiles = [profile("zeta"), profile("Alpha"), profile("beta")];
        assert_eq!(sorted_profile_names(&profiles), ["Alpha", "beta", "zeta"]);
    }

    #[test]
    fn the_active_profile_is_never_offered_for_deletion() {
        let profiles = [profile("fresh"), profile("default"), profile("heavy")];
        assert_eq!(delete_candidates(&profiles, "default"), ["fresh", "heavy"]);
        assert!(delete_candidates(&[profile("default")], "default").is_empty());
    }

    #[test]
    fn names_are_sanitized_then_checked_for_shape_and_collisions() {
        let existing = vec!["default".to_owned(), "heavy_mods".to_owned()];
        assert_eq!(
            check_profile_name("  Fresh Run ", &existing),
            Ok("fresh_run".to_owned())
        );
        assert_eq!(
            check_profile_name("Heavy Mods", &existing),
            Err(ProfileNameError::Collision)
        );
        assert_eq!(
            check_profile_name("   ", &existing),
            Err(ProfileNameError::Empty)
        );
        assert_eq!(
            check_profile_name("___", &existing),
            Err(ProfileNameError::Empty)
        );
        assert_eq!(
            check_profile_name("mods", &existing),
            Err(ProfileNameError::Invalid)
        );
        assert_eq!(
            check_profile_name("-dash", &existing),
            Err(ProfileNameError::Invalid)
        );
        assert_eq!(
            check_profile_name("caf\u{e9}", &existing),
            Err(ProfileNameError::Invalid)
        );
        assert_eq!(
            check_profile_name("a/b", &existing),
            Err(ProfileNameError::Invalid)
        );
    }

    #[test]
    fn a_running_game_locks_every_profile_action() {
        for state in [PlayState::Running, PlayState::Loading] {
            let actions = profile_actions(3, "default", false, true, state);
            assert!(actions.game_running);
            assert!(!actions.can_switch);
            assert!(!actions.can_create);
            assert!(!actions.can_rename);
            assert!(!actions.can_delete);
        }
    }

    #[test]
    fn deleting_needs_a_second_profile_and_everything_needs_a_game_folder() {
        let single = profile_actions(1, "default", false, true, PlayState::Idle);
        assert!(single.can_create && single.can_rename && single.can_switch);
        assert!(!single.can_delete);
        assert!(!single.more_than_one);

        let several = profile_actions(2, "default", false, true, PlayState::Idle);
        assert!(several.can_delete);

        let loading = profile_actions(2, "default", true, true, PlayState::Idle);
        assert!(!loading.can_create && !loading.can_delete);

        let no_game = profile_actions(2, "default", false, false, PlayState::Idle);
        assert!(!no_game.can_create && !no_game.can_switch);

        let empty = profile_actions(0, "", false, true, PlayState::Idle);
        assert!(empty.can_create);
        assert!(!empty.can_rename);
    }
}
