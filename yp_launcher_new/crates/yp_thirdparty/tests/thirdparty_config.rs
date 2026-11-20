mod common;

use std::collections::{BTreeSet, HashSet};

use common::{temp, write, write_bytes};
use yp_core::ini::IniDoc;
use yp_thirdparty::migoto::shaderfix_names;
use yp_thirdparty::tree;

fn set_key(ini: &str, section: &str, key: &str, value: &str) -> String {
    let mut doc = IniDoc::parse(ini);
    doc.set(Some(section), key, value);
    doc.to_string()
}

fn get_key(ini: &str, section: &str, key: &str) -> Option<String> {
    IniDoc::parse(ini).get(Some(section), key).map(str::to_owned)
}

fn lines_starting_with(text: &str, prefix: &str) -> usize {
    text.lines().filter(|line| line.starts_with(prefix)).count()
}

mod shader_fix_names_map {
    use super::*;

    #[test]
    fn assigns_mod_name_to_new_files_keeps_existing_prunes_gone() {
        let dir = temp("yp_sfn_");

        shaderfix_names::assign(dir.path(), &["aaa.txt", "bbb.txt"], "Bande Desinee");
        let first = shaderfix_names::read(dir.path());
        assert_eq!(first.len(), 2);
        assert_eq!(first["aaa.txt"], "Bande Desinee");
        assert_eq!(first["bbb.txt"], "Bande Desinee");

        shaderfix_names::assign(dir.path(), &["aaa.txt", "ccc.txt"], "Bloom Fix");
        let map = shaderfix_names::read(dir.path());
        assert_eq!(map["aaa.txt"], "Bande Desinee", "existing entry not overwritten by a later mod");
        assert_eq!(map["ccc.txt"], "Bloom Fix");

        let existing: HashSet<String> = ["aaa.txt", "ccc.txt"].iter().map(|name| (*name).to_owned()).collect();
        shaderfix_names::prune(dir.path(), &existing);
        let pruned = shaderfix_names::read(dir.path());
        assert!(!pruned.contains_key("bbb.txt"), "removed file dropped from the map");
        let keys: BTreeSet<_> = pruned.keys().cloned().collect();
        assert_eq!(keys, ["aaa.txt".to_owned(), "ccc.txt".to_owned()].into_iter().collect());
    }

    #[test]
    fn read_returns_empty_for_a_missing_or_malformed_file() {
        let dir = temp("yp_sfn2_");
        assert!(shaderfix_names::read(dir.path()).is_empty());
        write(&dir.path().join(shaderfix_names::FILE_NAME), "{bad");
        assert!(shaderfix_names::read(dir.path()).is_empty());
    }
}

mod file_sizes {
    use super::*;

    #[test]
    fn sizes_are_reported_in_bytes_and_none_for_a_missing_file() {
        let dir = temp("yp_size_");
        write_bytes(&dir.path().join("small"), &[0; 512]);
        write_bytes(&dir.path().join("kb"), &[0; 4096]);
        write_bytes(&dir.path().join("mb"), &vec![0; 2 * 1024 * 1024]);

        assert_eq!(tree::file_size(&dir.path().join("small")), Some(512));
        assert_eq!(tree::file_size(&dir.path().join("kb")), Some(4096));
        assert_eq!(tree::file_size(&dir.path().join("mb")), Some(2 * 1024 * 1024));
        assert_eq!(tree::file_size(&dir.path().join("nope")), None);
    }
}

mod ini_set_key {
    use super::*;

    #[test]
    fn replaces_an_existing_key_in_its_section_leaving_the_rest_intact() {
        let ini = "[Loader]\ntarget = NAMS.exe\n\n[Hunting]\nhunting = 0\nmarking_mode = skip\n";
        let out = set_key(ini, "Hunting", "hunting", "1");
        assert_eq!(get_key(&out, "Hunting", "hunting").as_deref(), Some("1"));
        assert_eq!(get_key(&out, "Hunting", "marking_mode").as_deref(), Some("skip"));
        assert_eq!(get_key(&out, "Loader", "target").as_deref(), Some("NAMS.exe"));
    }

    #[test]
    fn adds_the_key_when_the_section_exists_but_the_key_does_not() {
        let out = set_key("[Hunting]\nhunting = 1\n", "Hunting", "marking_mode", "pink");
        assert_eq!(get_key(&out, "Hunting", "marking_mode").as_deref(), Some("pink"));
        assert_eq!(get_key(&out, "Hunting", "hunting").as_deref(), Some("1"));
    }

    #[test]
    fn appends_a_new_section_when_it_is_missing() {
        let out = set_key("[Loader]\ntarget = NAMS.exe\n", "Rendering", "cache_shaders", "0");
        assert_eq!(get_key(&out, "Rendering", "cache_shaders").as_deref(), Some("0"));
        assert_eq!(get_key(&out, "Loader", "target").as_deref(), Some("NAMS.exe"));
    }

    #[test]
    fn leaves_a_commented_key_as_a_comment_and_adds_the_active_key() {
        let out = set_key("[Hunting]\n;hunting = 1\n", "Hunting", "hunting", "0");
        assert_eq!(get_key(&out, "Hunting", "hunting").as_deref(), Some("0"));
        assert_eq!(lines_starting_with(&out, "hunting = "), 1, "active key must appear exactly once:\n{out}");
        assert!(out.contains(";hunting = 1"));
    }

    #[test]
    fn does_not_duplicate_a_key_when_a_comment_mentions_it() {
        let ini = "[System]\n; hook = all is the safest option\nhook = recommended\nproxy_d3d11 =\n";
        let out = set_key(ini, "System", "hook", "recommended");
        assert_eq!(lines_starting_with(&out, "hook = "), 1, "hook must not be duplicated:\n{out}");
        assert_eq!(get_key(&out, "System", "hook").as_deref(), Some("recommended"));
    }

    #[test]
    fn applying_the_same_key_twice_is_idempotent() {
        let once = set_key("[System]\nload_library_redirect = 1\n", "System", "load_library_redirect", "0");
        let twice = set_key(&once, "System", "load_library_redirect", "0");
        assert_eq!(lines_starting_with(&twice, "load_library_redirect = "), 1);
        assert_eq!(get_key(&twice, "System", "load_library_redirect").as_deref(), Some("0"));
        assert_eq!(once, twice);
    }

    #[test]
    fn does_not_bleed_into_the_next_section() {
        let out = set_key("[Hunting]\nhunting = 0\n[Logging]\nverbose_overlay = 0\n", "Hunting", "hunting", "2");
        assert_eq!(get_key(&out, "Hunting", "hunting").as_deref(), Some("2"));
        assert_eq!(get_key(&out, "Logging", "verbose_overlay").as_deref(), Some("0"));
    }

    #[test]
    fn get_key_returns_none_for_an_absent_section_or_key() {
        let ini = "[Hunting]\nhunting = 1\n";
        assert_eq!(get_key(ini, "Nope", "x"), None);
        assert_eq!(get_key(ini, "Hunting", "nope"), None);
    }
}

mod migoto_config {
    use super::*;
    use yp_thirdparty::migoto::parse_config;
    use yp_thirdparty::{MigotoConfig, MigotoHunting, MigotoMarking};

    #[test]
    fn config_reads_hunting_marking_and_flags() {
        let ini = "[Hunting]\nhunting = 2\nmarking_mode = Pink\nreload_fixes = no_modifiers VK_F10\n\
                   [Logging]\nverbose_overlay = 1\n[Rendering]\ncache_shaders = 0\n[Device]\ncheck_foreground_window = true\n";
        let config = parse_config(ini);
        assert_eq!(config.hunting, MigotoHunting::NoMarking);
        assert_eq!(config.marking_mode, MigotoMarking::Pink);
        assert!(config.verbose_overlay);
        assert!(!config.cache_shaders);
        assert!(config.check_foreground_window);
        assert_eq!(config.reload_fixes_key, "no_modifiers VK_F10");
        assert_eq!(parse_config(""), MigotoConfig::default());
        assert!(parse_config("[Rendering]\ncache_shaders =\n").cache_shaders);
    }

    #[test]
    fn apply_config_writes_every_toggle() {
        let root = temp("yp_migoto_cfg_");
        let game = yp_core::GamePaths::new(root.path());
        let dir = yp_thirdparty::migoto::install_dir(&game);
        write(&dir.join("3dmigoto.dll"), "x");
        write(&dir.join("d3dx.ini"), "[Hunting]\nhunting = 0\n");
        let runtime = yp_thirdparty::MigotoRuntime::for_wine(false);
        let wanted = MigotoConfig {
            hunting: MigotoHunting::On,
            marking_mode: MigotoMarking::Mono,
            verbose_overlay: true,
            cache_shaders: false,
            check_foreground_window: true,
            ..MigotoConfig::default()
        };

        runtime.apply_config(&game, &wanted).unwrap();

        let written = parse_config(&std::fs::read_to_string(dir.join("d3dx.ini")).unwrap());
        assert_eq!(written, wanted);
    }
}
