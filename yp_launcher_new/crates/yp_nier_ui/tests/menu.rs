use yp_nier_ui::{Focus, MenuAction, MenuEntry, MenuInput, MenuPage, PauseMenu, Theme};

#[test]
fn confirmation_opens_on_no_and_cancel_from_yes_selects_no() {
    let entry = MenuEntry::new("Interface Pack", "Change status").confirm("Change its status?");
    let mut menu = PauseMenu::new(vec![MenuPage::new("Mods", "", vec![entry])]);
    menu.input(MenuInput::Confirm);
    menu.input(MenuInput::Confirm);
    assert_eq!(menu.focus(), Focus::Dialog);
    assert!(!menu.affirmative());
    menu.input(MenuInput::Left);
    assert!(menu.affirmative());
    menu.input(MenuInput::Back);
    assert!(!menu.affirmative());
    assert_eq!(menu.focus(), Focus::Dialog);
    assert_eq!(menu.input(MenuInput::Back), None);
    assert_eq!(menu.focus(), Focus::Rows);
    menu.input(MenuInput::Confirm);
    menu.choose_answer(true);
    assert_eq!(
        menu.input(MenuInput::Confirm),
        Some(MenuAction::Activate {
            page: 0,
            path: vec![0]
        })
    );
}

#[test]
fn keyboard_navigation_skips_disabled_entries_and_restores_nested_focus() {
    let rows = vec![
        MenuEntry::new("Locked", "").disabled(),
        MenuEntry::new("Options", "").children(vec![MenuEntry::new("Display", "")]),
        MenuEntry::new("Backups", ""),
    ];
    let mut menu = PauseMenu::new(vec![MenuPage::new("Mods", "", rows)]);
    menu.input(MenuInput::Down);
    assert_eq!(menu.row_index(), 1);
    menu.input(MenuInput::Up);
    assert_eq!(menu.row_index(), 2);
    menu.input(MenuInput::Down);
    menu.input(MenuInput::Confirm);
    assert_eq!(menu.title(), "Options");
    assert_eq!(
        menu.input(MenuInput::Confirm),
        Some(MenuAction::Activate {
            page: 0,
            path: vec![1, 0]
        })
    );
    menu.input(MenuInput::Back);
    assert_eq!(menu.row_index(), 1);
    menu.input(MenuInput::Back);
    assert_eq!(menu.focus(), Focus::Tabs);
    assert_eq!(menu.input(MenuInput::Back), Some(MenuAction::Close));
}

#[test]
fn theme_supplies_both_menu_glyphs_and_keyboard_sprites() {
    let theme = Theme::bundled();
    assert_eq!(theme.sprite_size("key_043"), Some([40, 40]));
    assert_eq!(theme.sprite_size("key_054"), Some([62, 40]));
    assert_eq!(theme.sprite_size("core_parts_00"), Some([1602, 96]));
    assert!(theme.measure("Mods", 36, 1.)[0] > 50.);
    assert!(theme.measure("設定", 1, 1.)[0] > 40.);
    assert_eq!(
        theme.measure("Menu\nDisplay", 36, 0.5)[1],
        theme.font_height(36)
    );
}
