use futures::channel::oneshot;
use gpui::{DismissEvent, EventEmitter, ScrollHandle, Task};
use yp_mods::tables::{MUTUALLY_EXCLUSIVE_VARIANT_CATEGORIES, VARIANT_CATEGORY_ORDER};
use yp_mods::{DataCategory, ModVariant};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, ChipSize, ChipTone, Confirm, ListItemSpacing, Modal, ModalFooter,
    ModalHeader, ModalView, TintColor, WindowExt,
};

use crate::labels::variant_category_label;
use crate::layout::shrinkable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantRow {
    Header(DataCategory),
    Item(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantSelection {
    categories: Vec<Option<DataCategory>>,
    groups: Vec<(DataCategory, Vec<usize>)>,
    selected: Vec<usize>,
}

fn is_pick_one(category: Option<DataCategory>) -> bool {
    category.is_some_and(|category| MUTUALLY_EXCLUSIVE_VARIANT_CATEGORIES.contains(&category))
}

impl VariantSelection {
    pub fn new(variants: &[ModVariant]) -> Self {
        let categories: Vec<Option<DataCategory>> =
            variants.iter().map(|variant| variant.category).collect();
        let mut found: Vec<(DataCategory, Vec<usize>)> = Vec::new();
        for (index, category) in categories.iter().enumerate() {
            let key = category.unwrap_or(DataCategory::Other);
            match found.iter_mut().find(|(group, _)| *group == key) {
                Some((_, members)) => members.push(index),
                None => found.push((key, vec![index])),
            }
        }
        let mut groups = Vec::with_capacity(found.len());
        for category in VARIANT_CATEGORY_ORDER {
            if let Some(position) = found.iter().position(|(group, _)| *group == category) {
                groups.push(found.remove(position));
            }
        }
        groups.append(&mut found);
        let mut selection = Self {
            categories,
            groups,
            selected: Vec::new(),
        };
        selection.selected = if selection.is_grouped() {
            selection
                .groups
                .iter()
                .filter(|(category, _)| !MUTUALLY_EXCLUSIVE_VARIANT_CATEGORIES.contains(category))
                .flat_map(|(_, members)| members.iter().copied())
                .collect()
        } else {
            (0..selection.categories.len()).collect()
        };
        selection
    }

    pub fn is_grouped(&self) -> bool {
        self.groups.len() > 1
    }

    pub fn is_exclusive(&self, index: usize) -> bool {
        self.is_grouped()
            && self
                .categories
                .get(index)
                .is_some_and(|category| is_pick_one(*category))
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    pub fn selected(&self) -> &[usize] {
        &self.selected
    }

    pub fn len(&self) -> usize {
        self.selected.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn rows(&self) -> Vec<VariantRow> {
        if !self.is_grouped() {
            return (0..self.categories.len()).map(VariantRow::Item).collect();
        }
        let mut rows = Vec::new();
        for (category, members) in &self.groups {
            rows.push(VariantRow::Header(*category));
            rows.extend(members.iter().copied().map(VariantRow::Item));
        }
        rows
    }

    pub fn toggle(&mut self, index: usize) {
        if index >= self.categories.len() {
            return;
        }
        let was_selected = self.is_selected(index);
        if self.is_exclusive(index) {
            let category = self.categories[index];
            let categories = &self.categories;
            self.selected.retain(|other| categories[*other] != category);
            if !was_selected {
                self.selected.push(index);
            }
        } else if was_selected {
            self.selected.retain(|other| *other != index);
        } else {
            self.selected.push(index);
        }
    }

    pub fn select_all(&mut self) {
        let categories = &self.categories;
        self.selected
            .retain(|index| !is_pick_one(categories[*index]));
        for index in 0..self.categories.len() {
            if !self.is_exclusive(index) && !self.selected.contains(&index) {
                self.selected.push(index);
            }
        }
    }

    pub fn clear(&mut self) {
        self.selected.clear();
    }
}

pub struct VariantDialog {
    variants: Vec<ModVariant>,
    selection: VariantSelection,
    sender: Option<oneshot::Sender<Option<Vec<ModVariant>>>>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

impl VariantDialog {
    fn finish(&mut self, accepted: bool, cx: &mut Context<Self>) {
        let result = accepted.then(|| {
            self.selection
                .selected()
                .iter()
                .filter_map(|index| self.variants.get(*index).cloned())
                .collect()
        });
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(result);
        }
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for VariantDialog {}

impl Focusable for VariantDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for VariantDialog {
    fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.sender.take();
        true
    }
}

impl Render for VariantDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let rows = self.selection.rows();
        let count = i64::try_from(self.selection.len()).unwrap_or(i64::MAX);
        let list = v_flex()
            .w_full()
            .children(rows.into_iter().map(|row| match row {
                VariantRow::Header(category) => {
                    let label = variant_category_label(Some(category), strings).to_uppercase();
                    let label = if MUTUALLY_EXCLUSIVE_VARIANT_CATEGORIES.contains(&category) {
                        strings.mods_ui_dot_join(&label, strings.variant_pick_one_suffix())
                    } else {
                        label
                    };
                    div()
                        .px(sizes.padding_sm)
                        .pt(sizes.padding_md)
                        .pb(sizes.spacing_sm)
                        .child(
                            Label::new(label)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .bold(),
                        )
                        .into_any_element()
                }
                VariantRow::Item(index) => {
                    let Some(variant) = self.variants.get(index) else {
                        return div().into_any_element();
                    };
                    let selected = self.selection.is_selected(index);
                    let icon = match (self.selection.is_exclusive(index), selected) {
                        (true, true) => IconName::RadioButtonChecked,
                        (true, false) => IconName::RadioButtonUnchecked,
                        (false, true) => IconName::CheckBox,
                        (false, false) => IconName::CheckBoxOutlineBlank,
                    };
                    ListItem::new(("variant-row", index))
                        .spacing(ListItemSpacing::Dense)
                        .rounded(true)
                        .start_slot(Icon::new(icon).size(IconSize::Medium).color(if selected {
                            Color::Accent
                        } else {
                            Color::Muted
                        }))
                        .child(shrinkable(Label::new(variant.label.clone()).truncate()))
                        .when(variant.texture_only, |this| {
                            this.end_slot(
                                Chip::new(strings.mod_variant_texture())
                                    .id(("variant-texture", index))
                                    .tone(ChipTone::Muted)
                                    .size(ChipSize::Small),
                            )
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selection.toggle(index);
                            cx.notify();
                        }))
                        .into_any_element()
                }
            }));
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Confirm, _, cx| {
                if !this.selection.is_empty() {
                    this.finish(true, cx);
                }
            }))
            .child(
                Modal::new("mods-variant-dialog", Some(self.scroll_handle.clone()))
                    .header(
                        ModalHeader::new()
                            .headline(strings.mod_variant_dialog_title())
                            .headline_color(Color::Accent)
                            .description(strings.mod_variant_dialog_subtitle()),
                    )
                    .section(
                        h_flex()
                            .gap(sizes.spacing_md)
                            .child(
                                Button::new("variant-select-all", strings.mod_variant_select_all())
                                    .style(ButtonStyle::Subtle)
                                    .size(ButtonSize::Compact)
                                    .color(Color::Accent)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.selection.select_all();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new(
                                    "variant-select-none",
                                    strings.mod_variant_select_none(),
                                )
                                .style(ButtonStyle::Subtle)
                                .size(ButtonSize::Compact)
                                .color(Color::Accent)
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.selection.clear();
                                        cx.notify();
                                    },
                                )),
                            ),
                    )
                    .section(list)
                    .footer(
                        ModalFooter::new().end_slot(
                            h_flex()
                                .gap(sizes.spacing_sm)
                                .child(
                                    Button::new("variant-cancel", strings.button_cancel())
                                        .style(ButtonStyle::Tinted(TintColor::Muted))
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.finish(false, cx)),
                                        ),
                                )
                                .child(
                                    Button::new(
                                        "variant-install",
                                        strings.mod_variant_install_selected(count),
                                    )
                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                    .disabled(self.selection.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| this.finish(true, cx))),
                                ),
                        ),
                    ),
            )
    }
}

pub fn pick_variants(
    variants: Vec<ModVariant>,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<Vec<ModVariant>>> {
    let (sender, receiver) = oneshot::channel();
    window.open_modal(cx, |_, cx| VariantDialog {
        selection: VariantSelection::new(&variants),
        variants,
        sender: Some(sender),
        focus_handle: cx.focus_handle(),
        scroll_handle: ScrollHandle::new(),
    });
    cx.spawn(async move |_| receiver.await.ok().flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use yp_mods::ModKind;

    fn variant(label: &str, category: Option<DataCategory>) -> ModVariant {
        ModVariant {
            sub_path: PathBuf::from(label),
            label: label.to_owned(),
            kind: ModKind::Data,
            texture_only: false,
            category,
        }
    }

    fn mixed() -> Vec<ModVariant> {
        vec![
            variant("Sword A", Some(DataCategory::Weapon)),
            variant("Kimono", Some(DataCategory::Player)),
            variant("Sword B", Some(DataCategory::Weapon)),
            variant("Readme pack", None),
        ]
    }

    #[test]
    fn a_single_category_starts_fully_selected_without_headers() {
        let variants = vec![
            variant("Sword A", Some(DataCategory::Weapon)),
            variant("Sword B", Some(DataCategory::Weapon)),
        ];
        let selection = VariantSelection::new(&variants);
        assert!(!selection.is_grouped());
        assert_eq!(selection.selected(), &[0, 1]);
        assert_eq!(
            selection.rows(),
            vec![VariantRow::Item(0), VariantRow::Item(1)]
        );
        assert!(!selection.is_exclusive(0));
    }

    #[test]
    fn groups_follow_the_category_order_and_skip_pick_one_groups_at_start() {
        let selection = VariantSelection::new(&mixed());
        assert!(selection.is_grouped());
        assert_eq!(
            selection.rows(),
            vec![
                VariantRow::Header(DataCategory::Player),
                VariantRow::Item(1),
                VariantRow::Header(DataCategory::Weapon),
                VariantRow::Item(0),
                VariantRow::Item(2),
                VariantRow::Header(DataCategory::Other),
                VariantRow::Item(3),
            ]
        );
        assert_eq!(selection.selected(), &[1, 3]);
        assert!(selection.is_exclusive(0));
        assert!(!selection.is_exclusive(1));
    }

    #[test]
    fn pick_one_groups_behave_like_radio_buttons_that_can_be_cleared() {
        let mut selection = VariantSelection::new(&mixed());
        selection.toggle(0);
        assert!(selection.is_selected(0));
        selection.toggle(2);
        assert!(!selection.is_selected(0));
        assert!(selection.is_selected(2));
        selection.toggle(2);
        assert!(!selection.is_selected(2));
        assert_eq!(selection.selected(), &[1, 3]);
    }

    #[test]
    fn select_all_leaves_pick_one_groups_empty_and_none_clears() {
        let mut selection = VariantSelection::new(&mixed());
        selection.toggle(0);
        selection.toggle(1);
        selection.select_all();
        assert!(!selection.is_selected(0));
        assert!(!selection.is_selected(2));
        assert!(selection.is_selected(1));
        assert!(selection.is_selected(3));
        selection.clear();
        assert!(selection.is_empty());
        selection.toggle(99);
        assert!(selection.is_empty());
    }
}
