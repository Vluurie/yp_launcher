use std::collections::BTreeSet;
use std::path::PathBuf;

use gpui::{
    Context, Entity, EventEmitter, IntoElement, Render, SharedString, Subscription, Window, div,
    prelude::*, px,
};
use yp_builders::{MeshRule, MeshVisibility, OutfitCharacter};
use yp_i18n::Strings;
use yp_state::inspector::ModelMesh;
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, ButtonStyle, SelectItem, Sizes, Tooltip};

use super::model_view::{ModelView, ModelViewEvent};
use super::widgets::hint;

const VISIBILITY_WIDTH: f32 = 200.;
const NAME_MIN_WIDTH: f32 = 110.;
const PREVIEW_SHARE: f32 = 0.34;
const PORTRAIT: f32 = 3. / 4.;
const LANDSCAPE: f32 = 4. / 3.;

pub const VISIBILITIES: [MeshVisibility; 8] = [
    MeshVisibility::Always,
    MeshVisibility::CombatOnly,
    MeshVisibility::OutOfCombatOnly,
    MeshVisibility::EyemaskOnly,
    MeshVisibility::NoEyemaskOnly,
    MeshVisibility::FeatherOnly,
    MeshVisibility::BrokenOnly,
    MeshVisibility::Never,
];

pub fn visibility_options(character: OutfitCharacter) -> Vec<MeshVisibility> {
    VISIBILITIES
        .into_iter()
        .filter(|visibility| visibility.applies_to(character))
        .collect()
}

pub fn visibility_label(visibility: MeshVisibility, strings: &Strings) -> &'static str {
    match visibility {
        MeshVisibility::Always => strings.outfit_mesh_always(),
        MeshVisibility::CombatOnly => strings.outfit_mesh_combat_only(),
        MeshVisibility::OutOfCombatOnly => strings.outfit_mesh_out_of_combat_only(),
        MeshVisibility::EyemaskOnly => strings.outfit_mesh_eyemask_only(),
        MeshVisibility::NoEyemaskOnly => strings.outfit_mesh_no_eyemask_only(),
        MeshVisibility::FeatherOnly => strings.outfit_mesh_feather_only(),
        MeshVisibility::BrokenOnly => strings.outfit_mesh_broken_only(),
        MeshVisibility::Never => strings.outfit_mesh_never(),
    }
}

pub fn lod0_mesh_names(meshes: &[ModelMesh]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for mesh in meshes.iter().filter(|mesh| mesh.lod_index == 0) {
        if !names.contains(&mesh.name) {
            names.push(mesh.name.clone());
        }
    }
    names
}

pub fn shown_mesh_names(meshes: &[ModelMesh]) -> BTreeSet<String> {
    meshes
        .iter()
        .filter(|mesh| mesh.lod_index == 0 && mesh.visible)
        .map(|mesh| mesh.name.clone())
        .collect()
}

pub fn has_extras(rule: &MeshRule) -> bool {
    rule.hide_on_self_destruct || rule.tinted_by_hairspray || rule.replaced_by_wig
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshTunerEvent {
    Changed,
}

pub struct MeshTuner {
    character: OutfitCharacter,
    view: Entity<ModelView>,
    mesh_names: Vec<String>,
    rules: Vec<(String, MeshRule)>,
    expanded: BTreeSet<String>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<MeshTunerEvent> for MeshTuner {}

impl MeshTuner {
    pub fn new(model: PathBuf, character: OutfitCharacter, cx: &mut Context<Self>) -> Self {
        let view = cx.new(|cx| ModelView::new(model, cx));
        let subscriptions = vec![cx.subscribe(&view, |this, view, event: &ModelViewEvent, cx| {
            if *event == ModelViewEvent::Ready {
                let meshes = view.read(cx).meshes(cx);
                this.set_meshes(lod0_mesh_names(&meshes), cx);
            } else {
                cx.notify();
            }
        })];
        Self {
            character,
            view,
            mesh_names: Vec::new(),
            rules: Vec::new(),
            expanded: BTreeSet::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn view(&self) -> &Entity<ModelView> {
        &self.view
    }

    pub fn mesh_names(&self) -> &[String] {
        &self.mesh_names
    }

    pub fn rules(&self) -> &[(String, MeshRule)] {
        &self.rules
    }

    pub fn rule(&self, name: &str) -> MeshRule {
        self.rules
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, rule)| *rule)
            .unwrap_or_default()
    }

    pub fn set_meshes(&mut self, names: Vec<String>, cx: &mut Context<Self>) {
        let previous = std::mem::take(&mut self.rules);
        self.rules = names
            .iter()
            .map(|name| {
                let rule = previous
                    .iter()
                    .find(|(known, _)| known == name)
                    .map(|(_, rule)| *rule)
                    .unwrap_or_default();
                (name.clone(), rule)
            })
            .collect();
        self.mesh_names = names;
        cx.emit(MeshTunerEvent::Changed);
        cx.notify();
    }

    pub fn set_rule(&mut self, name: &str, rule: MeshRule, cx: &mut Context<Self>) {
        if let Some(entry) = self.rules.iter_mut().find(|(known, _)| known == name) {
            entry.1 = rule;
            cx.emit(MeshTunerEvent::Changed);
            cx.notify();
        }
    }

    fn toggle_shown(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(session) = self.view.read(cx).session().cloned() else {
            return;
        };
        let meshes = session.read(cx).meshes().to_vec();
        let next = !shown_mesh_names(&meshes).contains(name);
        session.update(cx, |session, cx| {
            for mesh in meshes
                .iter()
                .filter(|mesh| mesh.lod_index == 0 && mesh.name == name)
            {
                session.set_mesh_visibility(mesh.id, next, cx);
            }
        });
        cx.notify();
    }

    fn toggle_expanded(&mut self, name: &str, cx: &mut Context<Self>) {
        if !self.expanded.remove(name) {
            self.expanded.insert(name.to_owned());
        }
        cx.notify();
    }

    fn row(
        &self,
        index: usize,
        name: &str,
        shown: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let rule = self.rule(name);
        let expanded = self.expanded.contains(name);
        let marked = has_extras(&rule);
        let character = self.character;
        let key = SharedString::from(name.to_owned());
        let entity = cx.entity();
        let select = {
            let entity = entity.clone();
            let key = key.clone();
            Select::new(
                SharedString::from(format!("mesh-visibility-{index}")),
                visibility_label(rule.visibility, strings),
            )
            .items(
                visibility_options(character)
                    .into_iter()
                    .map(|option| SelectItem::new(visibility_label(option, strings), option)),
            )
            .min_width(px(VISIBILITY_WIDTH))
            .max_width(px(VISIBILITY_WIDTH))
            .on_select(move |visibility: &MeshVisibility, _, cx| {
                let visibility = *visibility;
                entity.update(cx, |this, cx| {
                    let rule = MeshRule {
                        visibility,
                        ..this.rule(&key)
                    };
                    this.set_rule(&key, rule, cx);
                });
            })
        };
        let extra = |id: &'static str,
                     label: &'static str,
                     on: bool,
                     apply: fn(&mut MeshRule, bool)| {
            let entity = entity.clone();
            let key = key.clone();
            Checkbox::new(SharedString::from(format!("mesh-{id}-{index}")), on)
                .label(label)
                .label_size(LabelSize::XSmall)
                .on_click(move |state, _, cx| {
                    let on = state.selected();
                    entity.update(cx, |this, cx| {
                        let mut rule = this.rule(&key);
                        apply(&mut rule, on);
                        this.set_rule(&key, rule, cx);
                    });
                })
        };
        let extras = expanded.then(|| {
            h_flex()
                .w_full()
                .flex_wrap()
                .gap(sizes.spacing_lg)
                .pl(sizes.padding_lg)
                .py(sizes.spacing_sm)
                .child(extra(
                    "self-destruct",
                    strings.outfit_mesh_hide_on_self_destruct(),
                    rule.hide_on_self_destruct,
                    |rule, on| rule.hide_on_self_destruct = on,
                ))
                .child(extra(
                    "hairspray",
                    strings.outfit_mesh_hairspray_tint(),
                    rule.tinted_by_hairspray,
                    |rule, on| rule.tinted_by_hairspray = on,
                ))
                .when(character.has_wig(), |this| {
                    this.child(extra(
                        "wig",
                        strings.outfit_mesh_replaced_by_wig(),
                        rule.replaced_by_wig,
                        |rule, on| rule.replaced_by_wig = on,
                    ))
                })
        });
        let eye_key = key.clone();
        let more_key = key.clone();
        v_flex()
            .w_full()
            .px(sizes.padding_sm)
            .py(sizes.padding_xs)
            .rounded(Sizes::ROW_RADIUS)
            .border_1()
            .border_color(if rule.is_default() {
                theme.colors.border_light
            } else {
                theme.colors.accent_primary.opacity(theme.alpha.accent_border)
            })
            .bg(theme.colors.surface_medium)
            .child(
                h_flex()
                    .w_full()
                    .flex_wrap()
                    .gap(sizes.spacing_md)
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w(px(NAME_MIN_WIDTH))
                            .gap(sizes.spacing_md)
                            .child(
                                IconButton::new(
                                    SharedString::from(format!("mesh-eye-{index}")),
                                    if shown {
                                        IconName::Visibility
                                    } else {
                                        IconName::VisibilityOffOutlined
                                    },
                                )
                                .icon_size(IconSize::Small)
                                .icon_color(if shown {
                                    Color::Secondary
                                } else {
                                    Color::Muted
                                })
                                .tooltip(Tooltip::text(strings.outfit_mesh_preview_toggle()))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.toggle_shown(&eye_key, cx);
                                })),
                            )
                            .child(
                                h_flex()
                                    .id(SharedString::from(format!("mesh-name-{index}")))
                                    .flex_1()
                                    .min_w_0()
                                    .tooltip(Tooltip::text(key.clone()))
                                    .child(
                                        Label::new(key.clone())
                                            .size(LabelSize::XSmall)
                                            .mono()
                                            .truncate(),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .gap(sizes.spacing_md)
                            .child(select)
                            .child(
                                Button::new(
                                    SharedString::from(format!("mesh-more-{index}")),
                                    strings.outfit_mesh_more(),
                                )
                                .style(ButtonStyle::Transparent)
                                .color(if marked { Color::Accent } else { Color::Muted })
                                .end_icon(if expanded {
                                    IconName::ExpandLess
                                } else {
                                    IconName::ExpandMore
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.toggle_expanded(&more_key, cx);
                                })),
                            ),
                    ),
            )
            .children(extras)
    }
}

impl Render for MeshTuner {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let side_by_side = sizes.is_wide();
        let meshes = self.view.read(cx).meshes(cx);
        let shown = shown_mesh_names(&meshes);
        let has_session = self.view.read(cx).session().is_some() && !self.mesh_names.is_empty();
        let viewport = div()
            .relative()
            .w_full()
            .aspect_ratio(if side_by_side { PORTRAIT } else { LANDSCAPE })
            .child(div().absolute().inset_0().child(self.view.clone()));
        let names = self.mesh_names.clone();
        let rows: Vec<_> = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                self.row(index, name, shown.contains(name), window, cx)
                    .into_any_element()
            })
            .collect();
        let list = has_session.then(|| {
            v_flex()
                .w_full()
                .gap(sizes.spacing_sm)
                .child(hint(SharedString::from(
                    strings.outfit_builder_meshes_in_model(names.len() as i64),
                )))
                .children(rows)
        });
        if side_by_side {
            div()
                .flex()
                .flex_row()
                .items_start()
                .w_full()
                .gap(sizes.spacing_lg)
                .child(v_flex().w(gpui::relative(PREVIEW_SHARE)).flex_none().child(viewport))
                .child(v_flex().flex_1().min_w_0().children(list))
        } else {
            div()
                .flex()
                .flex_col()
                .w_full()
                .gap(sizes.spacing_lg)
                .child(viewport)
                .children(list)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh(id: u32, name: &str, lod_index: u32, visible: bool) -> ModelMesh {
        ModelMesh {
            id,
            name: name.to_owned(),
            lod_name: format!("LOD{lod_index}"),
            lod_index,
            visible,
        }
    }

    #[test]
    fn the_list_shows_each_lod0_mesh_once_in_model_order() {
        let meshes = vec![
            mesh(0, "Body", 0, true),
            mesh(1, "Skirt", 0, false),
            mesh(2, "Body", 0, true),
            mesh(3, "Body", 1, true),
            mesh(4, "Low", 1, true),
        ];
        assert_eq!(lod0_mesh_names(&meshes), vec!["Body", "Skirt"]);
        assert_eq!(shown_mesh_names(&meshes), BTreeSet::from(["Body".to_owned()]));
    }

    #[test]
    fn visibility_choices_depend_on_the_character() {
        let two_b = visibility_options(OutfitCharacter::TwoB);
        assert_eq!(two_b.len(), 8);
        let nine_s = visibility_options(OutfitCharacter::NineS);
        assert!(nine_s.contains(&MeshVisibility::EyemaskOnly));
        assert!(!nine_s.contains(&MeshVisibility::BrokenOnly));
        let a2 = visibility_options(OutfitCharacter::A2);
        assert_eq!(
            a2,
            vec![
                MeshVisibility::Always,
                MeshVisibility::CombatOnly,
                MeshVisibility::OutOfCombatOnly,
                MeshVisibility::Never,
            ]
        );
    }

    #[test]
    fn every_visibility_has_its_own_label() {
        let strings = Strings::for_locale(yp_i18n::Locale::En);
        let labels: BTreeSet<&str> = VISIBILITIES
            .into_iter()
            .map(|visibility| visibility_label(visibility, strings))
            .collect();
        assert_eq!(labels.len(), VISIBILITIES.len());
        assert!(!has_extras(&MeshRule::default()));
        assert!(has_extras(&MeshRule {
            tinted_by_hairspray: true,
            ..MeshRule::default()
        }));
    }
}
