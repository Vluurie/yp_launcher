use yp_state::TabId;
use yp_ui::prelude::*;
use yp_ui::{AnchorId, AnchorRegistry, Strings, TourStep};

use crate::markdown::{MarkdownText, parse};

pub const ANCHOR_DROP_ZONE: &str = "mods.drop-zone";
pub const ANCHOR_LIST: &str = "mods.list";
pub const ANCHOR_DETAIL: &str = "mods.detail";
pub const ANCHOR_PROFILES: &str = "mods.profiles";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TutorialKind {
    Ecosystem,
    InstallMod,
    Profiles,
    Supporting,
}

impl TutorialKind {
    pub const ALL: [TutorialKind; 4] = [
        TutorialKind::Ecosystem,
        TutorialKind::InstallMod,
        TutorialKind::Profiles,
        TutorialKind::Supporting,
    ];

    pub fn key(self) -> &'static str {
        match self {
            TutorialKind::Ecosystem => "ecosystem",
            TutorialKind::InstallMod => "install",
            TutorialKind::Profiles => "profiles",
            TutorialKind::Supporting => "supporting",
        }
    }

    pub fn from_key(key: &str) -> Option<TutorialKind> {
        TutorialKind::ALL.into_iter().find(|kind| kind.key() == key)
    }

    pub fn menu_label(self, strings: &Strings) -> &'static str {
        match self {
            TutorialKind::Ecosystem => strings.mods_tutorial_menu_ecosystem(),
            TutorialKind::InstallMod => strings.mods_tutorial_menu_install(),
            TutorialKind::Profiles => strings.mods_tutorial_menu_profiles(),
            TutorialKind::Supporting => strings.mods_tutorial_menu_supporting(),
        }
    }

    pub fn menu_icon(self) -> IconName {
        match self {
            TutorialKind::Ecosystem => IconName::HubOutlined,
            TutorialKind::InstallMod => IconName::DownloadOutlined,
            TutorialKind::Profiles => IconName::LayersOutlined,
            TutorialKind::Supporting => IconName::FavoriteOutline,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepSpec {
    pub anchor: Option<&'static str>,
    pub title: &'static str,
    pub body: &'static str,
}

fn reading(title: &'static str, body: &'static str) -> StepSpec {
    StepSpec {
        anchor: None,
        title,
        body,
    }
}

fn anchored(anchor: &'static str, title: &'static str, body: &'static str) -> StepSpec {
    StepSpec {
        anchor: Some(anchor),
        title,
        body,
    }
}

pub fn step_specs(kind: TutorialKind, strings: &Strings) -> Vec<StepSpec> {
    match kind {
        TutorialKind::Ecosystem => vec![
            reading(
                strings.mods_tutorial_ecosystem_step1_title(),
                strings.mods_tutorial_ecosystem_step1_body(),
            ),
            reading(
                strings.mods_tutorial_ecosystem_step2_title(),
                strings.mods_tutorial_ecosystem_step2_body(),
            ),
            reading(
                strings.mods_tutorial_ecosystem_step3_title(),
                strings.mods_tutorial_ecosystem_step3_body(),
            ),
            reading(
                strings.mods_tutorial_ecosystem_step4_title(),
                strings.mods_tutorial_ecosystem_step4_body(),
            ),
        ],
        TutorialKind::Supporting => vec![
            reading(
                strings.mods_tutorial_supporting_step1_title(),
                strings.mods_tutorial_supporting_step1_body(),
            ),
            reading(
                strings.mods_tutorial_supporting_step2_title(),
                strings.mods_tutorial_supporting_step2_body(),
            ),
            reading(
                strings.mods_tutorial_supporting_step3_title(),
                strings.mods_tutorial_supporting_step3_body(),
            ),
            reading(
                strings.mods_tutorial_supporting_step4_title(),
                strings.mods_tutorial_supporting_step4_body(),
            ),
            reading(
                strings.mods_tutorial_supporting_step5_title(),
                strings.mods_tutorial_supporting_step5_body(),
            ),
        ],
        TutorialKind::InstallMod => vec![
            anchored(
                ANCHOR_DROP_ZONE,
                strings.mods_tutorial_install_step1_title(),
                strings.mods_tutorial_install_step1_body(),
            ),
            reading(
                strings.mods_tutorial_install_step2_title(),
                strings.mods_tutorial_install_step2_body(),
            ),
            anchored(
                ANCHOR_LIST,
                strings.mods_tutorial_install_step3_title(),
                strings.mods_tutorial_install_step3_body(),
            ),
            anchored(
                ANCHOR_DETAIL,
                strings.mods_tutorial_install_step4_title(),
                strings.mods_tutorial_install_step4_body(),
            ),
            anchored(
                TabId::Textures.tour_anchor_id(),
                strings.mods_tutorial_install_step5_title(),
                strings.mods_tutorial_install_step5_body(),
            ),
            anchored(
                TabId::Cutscenes.tour_anchor_id(),
                strings.mods_tutorial_install_step6_title(),
                strings.mods_tutorial_install_step6_body(),
            ),
            reading(
                strings.mods_tutorial_install_step7_title(),
                strings.mods_tutorial_install_step7_body(),
            ),
        ],
        TutorialKind::Profiles => vec![
            anchored(
                ANCHOR_PROFILES,
                strings.mods_tutorial_profiles_step1_title(),
                strings.mods_tutorial_profiles_step1_body(),
            ),
            anchored(
                ANCHOR_PROFILES,
                strings.mods_tutorial_profiles_step2_title(),
                strings.mods_tutorial_profiles_step2_body(),
            ),
            anchored(
                ANCHOR_PROFILES,
                strings.mods_tutorial_profiles_step3_title(),
                strings.mods_tutorial_profiles_step3_body(),
            ),
            reading(
                strings.mods_tutorial_profiles_step4_title(),
                strings.mods_tutorial_profiles_step4_body(),
            ),
        ],
    }
}

pub fn tour_steps(kind: TutorialKind, strings: &Strings) -> Vec<TourStep> {
    step_specs(kind, strings)
        .into_iter()
        .map(|spec| {
            let blocks = parse(spec.body);
            let anchor = spec.anchor.map(AnchorId::from);
            let slot_anchor = anchor.clone();
            let step = TourStep::new(spec.title, spec.body).body_slot(move |_, cx| {
                let reading = slot_anchor
                    .as_ref()
                    .is_none_or(|anchor| AnchorRegistry::get(cx, anchor).is_none());
                MarkdownText::from_blocks(blocks.clone())
                    .reading(reading)
                    .into_any_element()
            });
            match anchor {
                Some(anchor) => step.anchor(anchor),
                None => step,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_ui::Locale;

    #[test]
    fn every_tutorial_has_steps_with_text_in_every_language() {
        for locale in [Locale::En, Locale::De, Locale::Zh, Locale::Th] {
            let strings = Strings::for_locale(locale);
            for kind in TutorialKind::ALL {
                let steps = step_specs(kind, strings);
                assert!(steps.len() >= 4, "{kind:?}");
                for step in steps {
                    assert!(!step.title.is_empty());
                    assert!(!parse(step.body).is_empty(), "{kind:?} {}", step.title);
                }
            }
        }
    }

    #[test]
    fn the_install_tour_points_at_the_tab_and_the_sidebar() {
        let strings = Strings::for_locale(Locale::En);
        let anchors: Vec<Option<&str>> = step_specs(TutorialKind::InstallMod, strings)
            .into_iter()
            .map(|step| step.anchor)
            .collect();
        assert_eq!(
            anchors,
            vec![
                Some(ANCHOR_DROP_ZONE),
                None,
                Some(ANCHOR_LIST),
                Some(ANCHOR_DETAIL),
                Some("sidebar.tab.textures"),
                Some("sidebar.tab.cutscenes"),
                None,
            ]
        );
        assert!(
            step_specs(TutorialKind::Ecosystem, strings)
                .iter()
                .all(|step| step.anchor.is_none())
        );
    }

    #[test]
    fn tutorial_kinds_round_trip_through_their_preview_keys() {
        for kind in TutorialKind::ALL {
            assert_eq!(TutorialKind::from_key(kind.key()), Some(kind));
        }
        assert_eq!(TutorialKind::from_key("nope"), None);
    }
}
