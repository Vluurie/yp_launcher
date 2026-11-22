use std::path::PathBuf;

use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::OutfitChoice;
use yp_mods::defaults::{
    self, DefaultEntry, DefaultKind, entry_for, initial_outfit_id, is_outfit_stem, kind_for,
    stem_of,
};
use yp_mods::state::DefaultModsData;

struct Fixture {
    _dir: TempDir,
    game: GamePaths,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("yp_defaults_")
            .tempdir()
            .unwrap();
        let game = GamePaths::new(dir.path());
        Self { _dir: dir, game }
    }

    fn file(&self) -> PathBuf {
        self.game.default_mods_toml()
    }

    fn raw(&self) -> String {
        std::fs::read_to_string(self.file()).unwrap()
    }

    fn write_raw(&self, text: &str) {
        std::fs::create_dir_all(self.file().parent().unwrap()).unwrap();
        std::fs::write(self.file(), text).unwrap();
    }

    fn star(&self, rel: &str, kind: DefaultKind) {
        defaults::set_default(&self.game, rel, Some(kind), None).unwrap();
    }

    fn star_outfit(&self, rel: &str, kind: DefaultKind, outfit_id: Option<i64>) {
        defaults::set_default(&self.game, rel, Some(kind), outfit_id).unwrap();
    }

    fn unstar(&self, rel: &str) {
        defaults::set_default(&self.game, rel, None, None).unwrap();
    }

    fn entries(&self) -> Vec<DefaultEntry> {
        defaults::list(&self.game).unwrap()
    }

    fn paths(&self) -> Vec<String> {
        self.entries().into_iter().map(|entry| entry.path).collect()
    }

    fn kind_of(&self, rel: &str) -> Option<DefaultKind> {
        entry_for(&self.entries(), rel).map(|entry| entry.kind)
    }

    fn outfit_of(&self, rel: &str) -> Option<i64> {
        entry_for(&self.entries(), rel).and_then(|entry| entry.outfit_id)
    }
}

fn contains_all(haystack: &[String], needles: &[&str]) -> bool {
    needles
        .iter()
        .all(|needle| haystack.iter().any(|item| item == needle))
}

fn choice(outfit_id: i64, name: &str, needs_item: bool) -> OutfitChoice {
    OutfitChoice {
        outfit_id,
        name: Some(name.to_owned()),
        needs_item,
    }
}

#[test]
fn missing_file_yields_no_defaults() {
    let fx = Fixture::new();
    assert!(fx.entries().is_empty());
}

#[test]
fn a_whole_mod_round_trips_through_disk() {
    let fx = Fixture::new();
    fx.star("mods/my_mod", DefaultKind::OutfitBare);
    assert_eq!(fx.paths(), vec!["mods/my_mod"]);
    assert!(fx.file().is_file());
}

#[test]
fn unstarring_drops_the_entry() {
    let fx = Fixture::new();
    fx.star("mods/my_mod", DefaultKind::OutfitBare);
    fx.unstar("mods/my_mod");
    assert!(fx.entries().is_empty());
}

#[test]
fn starring_the_same_entry_twice_does_not_duplicate_it() {
    let fx = Fixture::new();
    fx.star("mods/my_mod", DefaultKind::OutfitBare);
    fx.star("mods/my_mod", DefaultKind::OutfitBare);
    assert_eq!(fx.paths(), vec!["mods/my_mod"]);
}

#[test]
fn backslashes_are_normalized_to_forward_slashes() {
    let fx = Fixture::new();
    fx.star(r"mods\my_mod", DefaultKind::OutfitBare);
    assert_eq!(fx.paths(), vec!["mods/my_mod"]);
}

#[test]
fn unrelated_mods_survive_a_star() {
    let fx = Fixture::new();
    fx.star("mods/a", DefaultKind::OutfitBare);
    fx.star("mods/b", DefaultKind::OutfitBare);
    assert!(contains_all(&fx.paths(), &["mods/a", "mods/b"]));
}

#[test]
fn kinds_each_round_trip_through_their_own_list() {
    let fx = Fixture::new();
    fx.star("mods/bare/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/blink/pl/pl000f", DefaultKind::OutfitAnimation);
    assert_eq!(
        fx.kind_of("mods/bare/pl/pl000d"),
        Some(DefaultKind::OutfitBare)
    );
    assert_eq!(
        fx.kind_of("mods/blink/pl/pl000f"),
        Some(DefaultKind::OutfitAnimation)
    );
}

#[test]
fn kinds_a_config_outfit_is_stored_apart_from_a_bare_one() {
    let fx = Fixture::new();
    fx.star("mods/configured/pl/pl000d", DefaultKind::OutfitConfig);
    let raw = fx.raw();
    assert!(
        raw.contains("outfit_config = [\"mods/configured/pl/pl000d\"]"),
        "{raw}"
    );
    assert!(raw.contains("outfit_bare = []"), "{raw}");
}

#[test]
fn kinds_re_starring_under_a_different_kind_moves_the_entry() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig);
    assert_eq!(fx.paths(), vec!["mods/alpha/pl/pl000d"]);
    assert_eq!(
        fx.kind_of("mods/alpha/pl/pl000d"),
        Some(DefaultKind::OutfitConfig)
    );
}

#[test]
fn kinds_unstarring_finds_the_entry_whichever_list_it_is_in() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig);
    fx.unstar("mods/alpha/pl/pl000d");
    assert!(fx.entries().is_empty());
}

#[test]
fn outfit_choice_round_trips_through_disk() {
    let fx = Fixture::new();
    fx.star_outfit("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig, Some(2));
    assert_eq!(fx.outfit_of("mods/alpha/pl/pl000d"), Some(2));
}

#[test]
fn outfit_no_choice_means_the_outfit_worn_without_an_item() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig);
    assert_eq!(fx.outfit_of("mods/alpha/pl/pl000d"), None);
}

#[test]
fn outfit_chosen_is_written_as_a_table_plain_as_a_string() {
    let fx = Fixture::new();
    fx.star_outfit("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig, Some(3));
    fx.star("mods/beta/pl/pl020d", DefaultKind::OutfitConfig);
    let raw = fx.raw();
    assert!(
        raw.contains("{ path = \"mods/alpha/pl/pl000d\", outfit_id = 3 }"),
        "{raw}"
    );
    assert!(raw.contains("\"mods/beta/pl/pl020d\""), "{raw}");
}

#[test]
fn outfit_re_picking_replaces_the_previous_choice() {
    let fx = Fixture::new();
    fx.star_outfit("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig, Some(1));
    fx.star_outfit("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig, Some(2));
    assert_eq!(fx.paths(), vec!["mods/alpha/pl/pl000d"]);
    assert_eq!(fx.outfit_of("mods/alpha/pl/pl000d"), Some(2));
}

#[test]
fn outfit_clearing_the_choice_returns_to_the_item_less_outfit() {
    let fx = Fixture::new();
    fx.star_outfit("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig, Some(2));
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig);
    assert_eq!(fx.outfit_of("mods/alpha/pl/pl000d"), None);
}

#[test]
fn outfit_a_hand_written_table_entry_is_read_back() {
    let fx = Fixture::new();
    fx.write_raw("outfit_config = [{ path = \"mods/alpha/pl/pl000d\", outfit_id = 2 }]\n");
    assert_eq!(fx.paths(), vec!["mods/alpha/pl/pl000d"]);
    assert_eq!(fx.outfit_of("mods/alpha/pl/pl000d"), Some(2));
    assert_eq!(
        fx.kind_of("mods/alpha/pl/pl000d"),
        Some(DefaultKind::OutfitConfig)
    );
}

#[test]
fn outfit_strings_and_tables_mix_in_one_list() {
    let fx = Fixture::new();
    fx.write_raw(
        "outfit_config = [\"mods/plain/pl/pl000d\", { path = \"mods/picked/pl/pl020d\", outfit_id = 1 }]\n",
    );
    assert_eq!(fx.outfit_of("mods/plain/pl/pl000d"), None);
    assert_eq!(fx.outfit_of("mods/picked/pl/pl020d"), Some(1));
}

#[test]
fn outfit_a_table_without_a_path_is_skipped() {
    let fx = Fixture::new();
    fx.write_raw("outfit_config = [{ outfit_id = 2 }]\n");
    assert!(fx.entries().is_empty());
}

#[test]
fn outfit_the_choice_survives_an_unrelated_write() {
    let fx = Fixture::new();
    fx.star_outfit("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig, Some(2));
    fx.star("mods/beta/pl/pl020d", DefaultKind::OutfitBare);
    assert_eq!(fx.outfit_of("mods/alpha/pl/pl000d"), Some(2));
}

#[test]
fn outfit_a_lone_item_outfit_is_written_when_the_mod_becomes_the_default() {
    let fx = Fixture::new();
    let choices = [choice(71900, "Divergent", true)];
    let id = initial_outfit_id(DefaultKind::OutfitConfig, &choices);
    fx.star_outfit("mods/black_wax/pl/pl020d", DefaultKind::OutfitConfig, id);
    assert_eq!(fx.outfit_of("mods/black_wax/pl/pl020d"), Some(71900));
    assert!(
        fx.raw()
            .contains("{ path = \"mods/black_wax/pl/pl020d\", outfit_id = 71900 }")
    );
}

#[test]
fn initial_outfit_id_an_item_outfit_is_picked_when_it_is_the_only_one() {
    assert_eq!(
        initial_outfit_id(
            DefaultKind::OutfitConfig,
            &[choice(71900, "Divergent", true)]
        ),
        Some(71900)
    );
}

#[test]
fn initial_outfit_id_an_item_less_outfit_is_left_to_the_game() {
    assert_eq!(
        initial_outfit_id(
            DefaultKind::OutfitConfig,
            &[choice(5, "Replacement", false), choice(6, "Extra", true)]
        ),
        None
    );
}

#[test]
fn initial_outfit_id_several_item_outfits_are_left_for_the_user_to_pick() {
    assert_eq!(
        initial_outfit_id(
            DefaultKind::OutfitConfig,
            &[choice(1, "One", true), choice(2, "Two", true)]
        ),
        None
    );
}

#[test]
fn initial_outfit_id_no_outfits_means_nothing_to_write() {
    assert_eq!(initial_outfit_id(DefaultKind::OutfitConfig, &[]), None);
}

#[test]
fn initial_outfit_id_other_kinds_never_carry_an_outfit_id() {
    assert_eq!(
        initial_outfit_id(DefaultKind::OutfitBare, &[choice(71900, "Divergent", true)]),
        None
    );
}

#[test]
fn stem_exclusivity_starring_a_stem_in_another_mod_releases_the_previous_owner() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/beta/pl/pl000d", DefaultKind::OutfitBare);
    assert_eq!(fx.paths(), vec!["mods/beta/pl/pl000d"]);
}

#[test]
fn stem_exclusivity_a_config_outfit_displaces_a_bare_one() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/beta/pl/pl000d", DefaultKind::OutfitConfig);
    assert_eq!(fx.paths(), vec!["mods/beta/pl/pl000d"]);
    assert_eq!(
        fx.kind_of("mods/beta/pl/pl000d"),
        Some(DefaultKind::OutfitConfig)
    );
}

#[test]
fn stem_exclusivity_a_bare_outfit_displaces_a_config_one() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitConfig);
    fx.star("mods/beta/pl/pl000d", DefaultKind::OutfitBare);
    assert_eq!(fx.paths(), vec!["mods/beta/pl/pl000d"]);
    assert_eq!(
        fx.kind_of("mods/beta/pl/pl000d"),
        Some(DefaultKind::OutfitBare)
    );
}

#[test]
fn stem_exclusivity_an_outfit_and_an_animation_coexist() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/beta/pl/pl000f", DefaultKind::OutfitAnimation);
    assert!(contains_all(
        &fx.paths(),
        &["mods/alpha/pl/pl000d", "mods/beta/pl/pl000f"]
    ));
}

#[test]
fn stem_exclusivity_two_animations_for_the_same_stem_do_not_displace_each_other() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000f", DefaultKind::OutfitAnimation);
    fx.star("mods/beta/pl/pl000f", DefaultKind::OutfitAnimation);
    assert!(contains_all(
        &fx.paths(),
        &["mods/alpha/pl/pl000f", "mods/beta/pl/pl000f"]
    ));
}

#[test]
fn stem_exclusivity_each_character_keeps_its_own_default_outfit() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/beta/pl/pl020d", DefaultKind::OutfitConfig);
    fx.star("mods/gamma/pl/pl010d", DefaultKind::OutfitBare);
    assert!(contains_all(
        &fx.paths(),
        &[
            "mods/alpha/pl/pl000d",
            "mods/beta/pl/pl020d",
            "mods/gamma/pl/pl010d"
        ]
    ));
}

#[test]
fn stem_exclusivity_an_outfit_star_does_not_disturb_a_starred_animation() {
    let fx = Fixture::new();
    fx.star("mods/blink/pl/pl000f", DefaultKind::OutfitAnimation);
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/beta/pl/pl000d", DefaultKind::OutfitConfig);
    let paths = fx.paths();
    assert!(contains_all(
        &paths,
        &["mods/blink/pl/pl000f", "mods/beta/pl/pl000d"]
    ));
    assert!(!paths.contains(&"mods/alpha/pl/pl000d".to_owned()));
}

#[test]
fn stem_exclusivity_different_stems_within_one_mod_coexist() {
    let fx = Fixture::new();
    fx.star("mods/alpha/pl/pl000d", DefaultKind::OutfitBare);
    fx.star("mods/alpha/pl/pl000f", DefaultKind::OutfitAnimation);
    assert!(contains_all(
        &fx.paths(),
        &["mods/alpha/pl/pl000d", "mods/alpha/pl/pl000f"]
    ));
}

#[test]
fn stem_exclusivity_whole_mod_entries_are_not_treated_as_stems() {
    let fx = Fixture::new();
    fx.star("mods/alpha", DefaultKind::OutfitBare);
    fx.star("mods/beta", DefaultKind::OutfitBare);
    assert!(contains_all(&fx.paths(), &["mods/alpha", "mods/beta"]));
}

#[test]
fn default_kind_outfit_kinds_compete_animation_does_not() {
    assert!(DefaultKind::OutfitBare.conflicts_with(DefaultKind::OutfitConfig));
    assert!(DefaultKind::OutfitConfig.conflicts_with(DefaultKind::OutfitBare));
    assert!(!DefaultKind::OutfitBare.conflicts_with(DefaultKind::OutfitAnimation));
    assert!(!DefaultKind::OutfitAnimation.conflicts_with(DefaultKind::OutfitAnimation));
}

#[test]
fn default_kind_every_kind_has_a_distinct_toml_key() {
    let mut keys: Vec<&str> = DefaultKind::ALL.iter().map(|kind| kind.key()).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), DefaultKind::ALL.len());
}

#[test]
fn default_kind_from_key_round_trips_every_kind() {
    for kind in DefaultKind::ALL {
        assert_eq!(DefaultKind::from_key(kind.key()), Some(kind));
    }
    assert_eq!(DefaultKind::from_key("nope"), None);
}

#[test]
fn kind_for_an_animation_stem_is_an_animation_regardless_of_config() {
    assert_eq!(kind_for("pl000f", false), DefaultKind::OutfitAnimation);
    assert_eq!(kind_for("pl000f", true), DefaultKind::OutfitAnimation);
}

#[test]
fn kind_for_an_outfit_stem_follows_whether_the_mod_ships_a_config() {
    assert_eq!(kind_for("pl000d", false), DefaultKind::OutfitBare);
    assert_eq!(kind_for("pl000d", true), DefaultKind::OutfitConfig);
}

#[test]
fn stem_of_reads_the_last_segment_of_a_stem_entry() {
    assert_eq!(stem_of("mods/alpha/pl/pl000d").as_deref(), Some("pl000d"));
}

#[test]
fn stem_of_is_null_for_a_whole_mod_entry() {
    assert_eq!(stem_of("mods/alpha"), None);
}

#[test]
fn stem_of_tolerates_backslashes_and_stray_slashes() {
    assert_eq!(
        stem_of(r"/mods\alpha\wp\wp0100/").as_deref(),
        Some("wp0100")
    );
}

#[test]
fn is_outfit_stem_player_model_stems_are_outfits() {
    assert!(is_outfit_stem("pl000d"));
    assert!(is_outfit_stem("pl0000"));
    assert!(is_outfit_stem("pl020d"));
}

#[test]
fn is_outfit_stem_stems_ending_in_f_are_animations() {
    assert!(!is_outfit_stem("pl000f"));
    assert!(!is_outfit_stem("pl002f"));
    assert!(!is_outfit_stem("pl010f"));
}

#[test]
fn is_outfit_stem_non_player_stems_are_not_outfits() {
    assert!(!is_outfit_stem("wp0100"));
    assert!(!is_outfit_stem("q451"));
}

#[test]
fn is_outfit_stem_case_does_not_matter() {
    assert!(is_outfit_stem("PL000D"));
    assert!(!is_outfit_stem("PL000F"));
}

fn bare(path: &str) -> DefaultEntry {
    DefaultEntry::new(path, DefaultKind::OutfitBare)
}

#[test]
fn matches_an_exact_path() {
    assert!(defaults::matches(&[bare("mods/a")], "mods/a"));
}

#[test]
fn matches_a_parent_entry_covers_nested_paths() {
    assert!(defaults::matches(&[bare("mods/a")], "mods/a/pl/pl000d"));
}

#[test]
fn matches_a_sibling_with_a_shared_name_prefix_does_not_match() {
    assert!(!defaults::matches(&[bare("mods/a")], "mods/ab"));
}

#[test]
fn matches_an_unrelated_path_does_not_match() {
    assert!(!defaults::matches(&[bare("mods/a")], "mods/b"));
}

#[test]
fn matches_entry_for_reports_which_kind_covers_the_path() {
    let entries = [
        bare("mods/a"),
        DefaultEntry::new("mods/c", DefaultKind::OutfitConfig),
    ];
    assert_eq!(
        entry_for(&entries, "mods/c/pl/pl000d").map(|entry| entry.kind),
        Some(DefaultKind::OutfitConfig)
    );
    assert_eq!(entry_for(&entries, "mods/z"), None);
}

#[test]
fn comments_in_the_file_survive_a_write() {
    let fx = Fixture::new();
    fx.write_raw("# keep me\noutfit_bare = []\n");
    fx.star("mods/my_mod", DefaultKind::OutfitBare);
    assert!(fx.raw().contains("# keep me"));
}

#[test]
fn a_list_this_build_does_not_know_is_left_alone() {
    let fx = Fixture::new();
    fx.write_raw("outfit_bare = []\nfuture_kind = [\"mods/x\"]\n");
    fx.star("mods/my_mod", DefaultKind::OutfitBare);
    assert!(fx.raw().contains("future_kind"));
}

#[test]
fn a_malformed_list_degrades_to_no_defaults_instead_of_throwing() {
    let fx = Fixture::new();
    fx.write_raw("outfit_bare = \"not a list\"\n");
    assert!(fx.entries().is_empty());
}

fn data(entries: Vec<DefaultEntry>) -> DefaultModsData {
    DefaultModsData {
        entries,
        default_outfits_enabled: true,
    }
}

#[test]
fn would_replace_names_the_mod_currently_holding_the_stem() {
    let d = data(vec![bare("mods/alpha/pl/pl000d")]);
    let hit = d.would_replace("mods/beta/pl/pl000d", DefaultKind::OutfitBare);
    assert_eq!(
        hit.map(|entry| entry.path.as_str()),
        Some("mods/alpha/pl/pl000d")
    );
}

#[test]
fn would_replace_a_different_stem_is_not_replaced() {
    let d = data(vec![bare("mods/alpha/pl/pl020d")]);
    assert_eq!(
        d.would_replace("mods/beta/pl/pl000d", DefaultKind::OutfitBare),
        None
    );
}

#[test]
fn would_replace_an_animation_is_not_replaced_by_an_animation() {
    let d = data(vec![DefaultEntry::new(
        "mods/blink/pl/pl000f",
        DefaultKind::OutfitAnimation,
    )]);
    assert_eq!(
        d.would_replace("mods/beta/pl/pl000f", DefaultKind::OutfitAnimation),
        None
    );
}

#[test]
fn would_replace_a_config_outfit_replaces_a_bare_one() {
    let d = data(vec![bare("mods/alpha/pl/pl000d")]);
    let hit = d.would_replace("mods/beta/pl/pl000d", DefaultKind::OutfitConfig);
    assert_eq!(
        hit.map(|entry| entry.path.as_str()),
        Some("mods/alpha/pl/pl000d")
    );
}

#[test]
fn would_replace_re_starring_the_same_entry_replaces_nothing() {
    let d = data(vec![bare("mods/alpha/pl/pl000d")]);
    assert_eq!(
        d.would_replace("mods/alpha/pl/pl000d", DefaultKind::OutfitBare),
        None
    );
}

#[test]
fn would_replace_a_whole_mod_entry_has_no_stem_to_compete_over() {
    let d = data(vec![bare("mods/alpha")]);
    assert_eq!(d.would_replace("mods/beta", DefaultKind::OutfitBare), None);
}

#[test]
fn entries_under_lists_the_mod_and_its_stems() {
    let d = data(vec![
        bare("mods/foo"),
        bare("mods/foo/pl/pl000d"),
        bare("mods/foobar"),
    ]);
    let under: Vec<&str> = d
        .entries_under("mods/foo")
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    assert_eq!(under, vec!["mods/foo", "mods/foo/pl/pl000d"]);
}

#[test]
fn entries_in_one_list_do_not_leak_into_another() {
    let fx = Fixture::new();
    fx.write_raw("outfit_bare = [\"mods/a\"]\noutfit_config = [\"mods/b\"]\noutfit_animation = [\"mods/c\"]\n");
    assert_eq!(fx.kind_of("mods/a"), Some(DefaultKind::OutfitBare));
    assert_eq!(fx.kind_of("mods/b"), Some(DefaultKind::OutfitConfig));
    assert_eq!(fx.kind_of("mods/c"), Some(DefaultKind::OutfitAnimation));
}

#[test]
fn a_fresh_file_starts_from_the_commented_template() {
    let fx = Fixture::new();
    fx.star("mods/a", DefaultKind::OutfitAnimation);
    let raw = fx.raw();
    assert!(raw.starts_with("# Mods listed here are active from the moment the game starts,"));
    assert!(raw.contains("outfit_animation = [\"mods/a\"]"));
    assert!(raw.contains("outfit_config = []"));
}
