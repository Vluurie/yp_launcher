use std::collections::BTreeMap;

use gpui::{Animation, AnimationExt, App, IntoElement, Window};
use yp_core::consts::APP_VERSION;
use yp_i18n::{Locale, Strings};
use yp_state::prefs::SIDEBAR_SECTION_COLLAPSED;
use yp_state::prelude::*;
use yp_state::{OverlayId, SidebarSection};
use yp_ui::prelude::*;
use yp_ui::{Chip, ChipTone, Sidebar, SidebarGroup, SidebarItem, easing};

use crate::ShellOptions;
use crate::play_button::PlayButton;

pub fn tab_icon(tab: TabId) -> IconName {
    match tab {
        TabId::Launcher => IconName::PlayCircleOutline,
        TabId::LauncherSettings => IconName::SettingsOutlined,
        TabId::Mods => IconName::ExtensionOutlined,
        TabId::Textures => IconName::Texture,
        TabId::Cutscenes => IconName::MovieCreationOutlined,
        TabId::ThirdParty => IconName::AutoFixHigh,
        TabId::Docs => IconName::MenuBookOutlined,
        TabId::Tools => IconName::HandymanOutlined,
        TabId::Nams => IconName::Tune,
        TabId::YorhaProtocol => IconName::DeveloperMode,
        TabId::LodMod => IconName::ImageOutlined,
        TabId::Naiom => IconName::MouseOutlined,
    }
}

pub fn tab_label(tab: TabId, strings: &Strings) -> &'static str {
    match tab {
        TabId::Launcher => strings.tab_launcher(),
        TabId::LauncherSettings => strings.tab_launcher_settings(),
        TabId::Mods => strings.tab_mods(),
        TabId::Textures => strings.tab_textures(),
        TabId::Cutscenes => strings.tab_cutscenes(),
        TabId::ThirdParty => strings.tab_third_party(),
        TabId::Docs => strings.tab_docs(),
        TabId::Tools => strings.tab_tools(),
        TabId::Nams => strings.tab_nams(),
        TabId::YorhaProtocol => strings.tab_yorha_protocol(),
        TabId::LodMod => strings.tab_lod_mod(),
        TabId::Naiom => strings.tab_naiom(),
    }
}

pub fn section_title(section: SidebarSection, strings: &Strings) -> &'static str {
    match section {
        SidebarSection::General => strings.tab_section_general(),
        SidebarSection::Mods => strings.tab_section_mods(),
        SidebarSection::Config => strings.tab_section_config(),
    }
}

pub fn section_key(section: SidebarSection) -> &'static str {
    section_title(section, Strings::for_locale(Locale::En))
}

pub fn is_collapsed(section: SidebarSection, saved: &BTreeMap<String, bool>) -> bool {
    if !section.collapsible() {
        return false;
    }
    saved
        .get(section_key(section))
        .copied()
        .unwrap_or_else(|| section.collapsed_by_default())
}

pub fn toggled(section: SidebarSection, saved: &BTreeMap<String, bool>) -> BTreeMap<String, bool> {
    let mut next = saved.clone();
    if section.collapsible() {
        next.insert(
            section_key(section).to_owned(),
            !is_collapsed(section, saved),
        );
    }
    next
}

pub fn collapsed_sections(cx: &App) -> BTreeMap<String, bool> {
    cx.prefs().get(&SIDEBAR_SECTION_COLLAPSED)
}

pub fn toggle_section(section: SidebarSection, cx: &mut App) {
    let next = toggled(section, &collapsed_sections(cx));
    cx.prefs().set(&SIDEBAR_SECTION_COLLAPSED, &next);
    cx.refresh_windows();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalTabs {
    pub docs: bool,
    pub tools: bool,
}

impl OptionalTabs {
    pub fn read(cx: &App) -> Self {
        let registry = cx.registry();
        let options = cx.try_global::<ShellOptions>();
        Self {
            docs: registry.docs_library
                && registry.overlays.contains(OverlayId::Docs)
                && options.is_none_or(|options| options.docs),
            tools: registry.tabs.contains(TabId::Tools)
                && options.is_none_or(|options| options.tools),
        }
    }

    pub fn shows(self, tab: TabId) -> bool {
        match tab {
            TabId::Docs => self.docs,
            TabId::Tools => self.tools,
            _ => true,
        }
    }
}

pub fn visible_tabs(section: SidebarSection, optional: OptionalTabs) -> Vec<TabId> {
    section
        .tabs()
        .iter()
        .copied()
        .filter(|tab| optional.shows(*tab))
        .collect()
}

fn footer(show_play: bool, show_labels: bool, window: &Window, cx: &App) -> impl IntoElement {
    let sizes = window.sizes(cx);
    let duration = cx.theme().motion.cross;
    v_flex()
        .w_full()
        .gap(sizes.spacing_md)
        .when(show_play, |this| {
            this.child(
                v_flex()
                    .w_full()
                    .child(PlayButton::compact(show_labels))
                    .with_animation(
                        "shell-sidebar-play",
                        Animation::new(duration),
                        |this, delta| this.opacity(easing::ease_out_cubic(delta)),
                    ),
            )
        })
        .child(
            Chip::new(APP_VERSION)
                .id("shell-version")
                .tone(ChipTone::Accent)
                .full_width()
                .truncate()
                .when(!show_labels, |chip| chip.tooltip_text(APP_VERSION)),
        )
}

pub fn render(active: TabId, window: &Window, cx: &App) -> Sidebar {
    let strings = cx.t();
    let sizes = window.sizes(cx);
    let show_labels = sizes.sidebar_labels_visible();
    let saved = collapsed_sections(cx);
    let optional = OptionalTabs::read(cx);
    let mut sidebar = Sidebar::new().id("shell-sidebar");
    for section in SidebarSection::ALL {
        let tabs = visible_tabs(section, optional);
        if tabs.is_empty() {
            continue;
        }
        let mut group = SidebarGroup::new(section_title(section, strings)).count(tabs.len());
        if section.collapsible() {
            group = group
                .collapsible()
                .collapsed(is_collapsed(section, &saved))
                .on_toggle(move |_, _, cx| toggle_section(section, cx));
        }
        for tab in tabs {
            group = group.item(
                SidebarItem::new(
                    ("shell-sidebar-tab", tab.index()),
                    tab_icon(tab),
                    tab_label(tab, strings),
                )
                .active(tab == active)
                .tour_anchor(tab.tour_anchor_id())
                .on_click(move |_, window, cx| {
                    window.dispatch_action(Box::new(OpenTab::new(tab)), cx);
                }),
            );
        }
        sidebar = sidebar.group(group);
    }
    sidebar.footer(footer(
        active != TabId::Launcher,
        show_labels,
        window,
        cx,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_starts_collapsed_and_other_sections_never_collapse() {
        let saved = BTreeMap::new();
        assert!(is_collapsed(SidebarSection::Config, &saved));
        assert!(!is_collapsed(SidebarSection::General, &saved));
        assert!(!is_collapsed(SidebarSection::Mods, &saved));
    }

    #[test]
    fn toggling_flips_only_collapsible_sections() {
        let saved = BTreeMap::new();
        let opened = toggled(SidebarSection::Config, &saved);
        assert_eq!(opened.get("Config"), Some(&false));
        assert!(!is_collapsed(SidebarSection::Config, &opened));
        let closed = toggled(SidebarSection::Config, &opened);
        assert!(is_collapsed(SidebarSection::Config, &closed));
        assert_eq!(toggled(SidebarSection::General, &saved), saved);
    }

    #[test]
    fn saved_state_uses_the_flutter_key() {
        let mut saved = BTreeMap::new();
        saved.insert("Config".to_owned(), false);
        assert!(!is_collapsed(SidebarSection::Config, &saved));
        assert_eq!(section_key(SidebarSection::Config), "Config");
    }

    #[test]
    fn docs_and_tools_rows_need_their_feature() {
        let none = OptionalTabs {
            docs: false,
            tools: false,
        };
        let mods = visible_tabs(SidebarSection::Mods, none);
        assert!(!mods.contains(&TabId::Docs));
        assert!(!mods.contains(&TabId::Tools));
        assert!(mods.contains(&TabId::Mods));
        let docs_only = OptionalTabs {
            docs: true,
            tools: false,
        };
        let mods = visible_tabs(SidebarSection::Mods, docs_only);
        assert!(mods.contains(&TabId::Docs));
        assert!(!mods.contains(&TabId::Tools));
        assert_eq!(visible_tabs(SidebarSection::Config, none).len(), 4);
    }

    #[test]
    fn every_tab_has_a_label_and_an_icon() {
        let strings = Strings::for_locale(Locale::En);
        for tab in TabId::ALL {
            assert!(!tab_label(tab, strings).is_empty());
            let _ = tab_icon(tab);
        }
    }
}
