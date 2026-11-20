mod support;

use support::GameDir;
use yp_config::fields::lodmod;
use yp_config::state::ConfigStore;

#[test]
fn a_broken_lodmod_toml_is_reported_and_never_written_back() {
    let game = GameDir::empty();
    let broken = "enabled = true\nfxaa = false\nfxaa = true\n";
    game.write_nams_file("lodmod.toml", broken);

    let mut store = ConfigStore::new(game.game.clone(), game.host.clone());
    store.load().unwrap();
    assert!(store.data().lodmod_error.is_some());
    assert!(store.data().lodmod_values.is_empty());

    store.update_lodmod(lodmod::FXAA.key(), toml::Value::Boolean(true), None);
    store.save().unwrap();
    assert_eq!(game.read_nams_file("lodmod.toml"), broken);

    store.update_lodmod_live(lodmod::FXAA.key(), toml::Value::Boolean(true), None);
    assert!(!store.flush_lodmod().unwrap());
    assert_eq!(game.read_nams_file("lodmod.toml"), broken);
}

#[test]
fn a_valid_lodmod_toml_still_saves() {
    let game = GameDir::empty();
    game.write_nams_file("lodmod.toml", "enabled = false\nfxaa = false\n");

    let mut store = ConfigStore::new(game.game.clone(), game.host.clone());
    store.load().unwrap();
    assert!(store.data().lodmod_error.is_none());

    store.update_lodmod(lodmod::FXAA.key(), toml::Value::Boolean(true), None);
    store.save().unwrap();
    assert!(game.read_nams_file("lodmod.toml").contains("fxaa = true"));
}

#[test]
fn a_broken_nams_toml_reports_the_error_and_survives_ensure_configs() {
    let game = GameDir::empty();
    let broken = "validate_model_data = \n[mouse\n";
    game.write_nams_file("nams.toml", broken);

    let mut store = ConfigStore::new(game.game.clone(), game.host.clone());
    store.load().unwrap();
    let error = store.data().nams_error.clone().unwrap();
    assert!(error.line.is_some());
    assert_eq!(game.read_nams_file("nams.toml"), broken);
    assert!(store.data().naiom_error.is_none());

    store.update_nams("validate_model_data", toml::Value::Boolean(true), None);
    store.save().unwrap();
    assert_eq!(game.read_nams_file("nams.toml"), broken);
    assert!(
        !store
            .update_nams_now("validate_model_data", toml::Value::Boolean(true), None)
            .unwrap()
    );
}
