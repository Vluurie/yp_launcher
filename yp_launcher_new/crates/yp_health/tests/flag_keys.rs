use yp_config::fields::{self, nams};
use yp_config::{AnyField, ConfigFile, FieldKind, templates};
use yp_core::toml_io::parse_values;
use yp_thirdparty::{LoadingFlag, ThirdPartyRuntime};

fn registered(key: &str) -> Vec<&'static dyn AnyField> {
    fields::all()
        .into_iter()
        .filter(|field| field.meta().key == key)
        .collect()
}

#[test]
fn every_third_party_loading_flag_is_a_registered_nams_setting() {
    for flag in LoadingFlag::ALL {
        let matches = registered(flag.key());
        assert_eq!(
            matches.len(),
            1,
            "{} must be registered exactly once",
            flag.key()
        );
        let field = matches[0];
        assert_eq!(field.kind(), FieldKind::Bool, "{}", flag.key());
        assert_eq!(field.meta().section, None, "{}", flag.key());
        assert_eq!(
            field.default_toml(),
            toml::Value::Boolean(false),
            "{}",
            flag.key()
        );
        assert!(
            nams::ALL
                .iter()
                .any(|candidate| candidate.meta().key == flag.key()),
            "{} must live in nams.toml",
            flag.key()
        );
    }
}

#[test]
fn every_loading_flag_ships_switched_on_in_the_nams_template() {
    let template = parse_values(templates::template(ConfigFile::Nams)).unwrap();
    for flag in LoadingFlag::ALL {
        assert_eq!(
            template.get(flag.key()).and_then(toml::Value::as_bool),
            Some(false),
            "{}",
            flag.key()
        );
    }
}

#[test]
fn flags_map_to_the_fields_the_settings_screen_edits() {
    let expected = [
        (LoadingFlag::ReShade, nams::DISABLE_RESHADE_LOADING.key()),
        (LoadingFlag::Migoto, nams::DISABLE_3DMIGOTO_LOADING.key()),
        (LoadingFlag::GameMods, nams::DISABLE_GAME_MODS_LOADING.key()),
        (
            LoadingFlag::GraphicMods,
            nams::DISABLE_GRAPHIC_MODS_LOADING.key(),
        ),
    ];
    assert_eq!(expected.len(), LoadingFlag::ALL.len());
    for (flag, key) in expected {
        assert_eq!(flag.key(), key);
    }
    for runtime in ThirdPartyRuntime::ALL {
        assert!(LoadingFlag::ALL.contains(&runtime.loading_flag()));
    }
}
