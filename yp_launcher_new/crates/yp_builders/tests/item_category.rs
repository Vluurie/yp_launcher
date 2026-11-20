use yp_builders::item;
use yp_builders::outfit::OUTFIT_ITEM_CATEGORY;
use yp_builders::ItemDraft;

#[test]
fn an_outfit_item_carries_the_category_nams_uses_for_its_own() {
    let draft = ItemDraft {
        category: OUTFIT_ITEM_CATEGORY,
        ..ItemDraft::new("Black Dress", "A dress", 9100)
    };

    let toml = item::to_toml(&draft);

    assert!(toml.contains("category = 6"));
    assert!(toml.contains("id_type = 0"));
}

#[test]
fn a_plain_item_writes_no_category_line() {
    assert!(!item::to_toml(&ItemDraft::new("Nut", "A nut", 8000)).contains("category"));
}
