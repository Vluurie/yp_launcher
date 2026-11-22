#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value as Json, json};
use yp_config::cli::{self, ResetTarget};
use yp_config::{ConfigError, ItemType, Kind, Schema};
use yp_core::{
    AppPaths, CancelToken, ChildProcess, GamePaths, HostEnv, ProcessOutput, ProcessRunner, ProcessSpec, StreamKind,
    SystemRunner,
};
use yp_host::nams_cli::HostNamsCli;
use yp_host::platform::windows::WindowsAdapter;

const BUNDLED_BINS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/bins");
const APPDATA_VAR: &str = "APPDATA";
const SHADOW_RESOLUTION: &str = "lodmod.shadow_resolution";
const GRAPHICS_BACKEND: &str = "nams.graphics_backend";
const FIRST_PLAYTHROUGH: &str = "plugins.yorha_protocol.firstPlaythrough";

struct IsolatedRunner {
    appdata: PathBuf,
}

impl IsolatedRunner {
    fn isolate(&self, spec: &ProcessSpec) -> ProcessSpec {
        spec.clone().env(APPDATA_VAR, self.appdata.to_string_lossy())
    }
}

impl ProcessRunner for IsolatedRunner {
    fn output(&self, spec: &ProcessSpec, timeout: Option<Duration>) -> yp_core::Result<ProcessOutput> {
        SystemRunner.output(&self.isolate(spec), timeout)
    }

    fn stream(
        &self,
        spec: &ProcessSpec,
        cancel: &CancelToken,
        on_chunk: &mut dyn FnMut(StreamKind, &str),
    ) -> yp_core::Result<ProcessOutput> {
        SystemRunner.stream(&self.isolate(spec), cancel, on_chunk)
    }

    fn spawn(&self, spec: &ProcessSpec) -> yp_core::Result<Box<dyn ChildProcess>> {
        SystemRunner.spawn(&self.isolate(spec))
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    cli: HostNamsCli,
    game: PathBuf,
    appdata: PathBuf,
}

impl Fixture {
    fn nams_file(&self, name: &str) -> PathBuf {
        self.game.join("nams").join(name)
    }
}

fn fixture() -> Option<Fixture> {
    let bins = Path::new(BUNDLED_BINS).canonicalize().ok()?;
    if !bins.join(yp_core::consts::NAMS_EXE_NAME).is_file() {
        eprintln!("skipped: no NAMS.exe in {}", bins.display());
        return None;
    }
    let dir = tempfile::tempdir().unwrap();
    let game = dir.path().join("game");
    let appdata = dir.path().join("appdata");
    std::fs::create_dir_all(&game).unwrap();
    std::fs::create_dir_all(&appdata).unwrap();
    let runner: Arc<dyn ProcessRunner> = Arc::new(IsolatedRunner {
        appdata: appdata.clone(),
    });
    let paths = AppPaths {
        runtime_dir: bins.clone(),
        bundled_bins_dir: bins,
        ..AppPaths::for_test(dir.path())
    };
    let cli = HostNamsCli {
        platform: Arc::new(WindowsAdapter::new(HostEnv::current(), runner.clone())),
        runner,
        paths,
        game: GamePaths::new(&game),
    };
    Some(Fixture {
        _dir: dir,
        cli,
        game,
        appdata,
    })
}

fn assert_schema_is_understood(schema: &Schema) {
    assert!(!schema.is_empty(), "schema has no settings");
    for setting in &schema.settings {
        assert_ne!(setting.kind, Kind::Other, "{} has a type the launcher does not know", setting.id);
        assert!(
            schema.category(&setting.category).is_some(),
            "{} is in the unlisted category {}",
            setting.id,
            setting.category
        );
        if setting.kind == Kind::List {
            let item = setting.item_type();
            assert!(
                item.is_some() && item != Some(ItemType::Unknown),
                "{} lists items the launcher cannot read",
                setting.id
            );
        }
        if setting.kind == Kind::Enum {
            assert!(!setting.choices.is_empty(), "{} is an enum without choices", setting.id);
        }
    }
}

#[test]
fn a_fresh_game_folder_is_migrated_once_and_validates() {
    let Some(fixture) = fixture() else { return };
    let first = cli::migrate(&fixture.cli, false).unwrap();
    assert!(!first.up_to_date);
    assert!(first.skipped.is_empty());
    assert!(!first.changes.is_empty());
    assert!(fixture.nams_file("nams.toml").is_file());
    assert!(fixture.nams_file("lodmod.toml").is_file());
    let second = cli::migrate(&fixture.cli, false).unwrap();
    assert!(second.up_to_date, "{second:?}");
    assert!(second.changes.is_empty());
    let report = cli::validate(&fixture.cli).unwrap();
    assert!(report.valid, "{report:?}");
    assert_eq!(report.errors, 0);
}

#[test]
fn a_file_with_a_syntax_error_is_skipped_by_migrate_and_flagged_by_get() {
    let Some(fixture) = fixture() else { return };
    let lodmod = fixture.nams_file("lodmod.toml");
    std::fs::create_dir_all(lodmod.parent().unwrap()).unwrap();
    std::fs::write(&lodmod, "enabled = [\n").unwrap();
    let report = cli::migrate(&fixture.cli, true).unwrap();
    assert!(report.dry_run);
    assert!(
        report.skipped.iter().any(|skipped| skipped.file == "lodmod.toml" && !skipped.detail.is_empty()),
        "{report:?}"
    );
    let values = cli::get(&fixture.cli).unwrap();
    assert!(values.file_error("lodmod.toml").is_some(), "{values:?}");
}

#[test]
fn the_schema_reads_in_every_language_it_offers() {
    let Some(fixture) = fixture() else { return };
    let schema = cli::schema(&fixture.cli, None).unwrap();
    assert_schema_is_understood(&schema);
    assert!(schema.plugins.iter().any(|plugin| plugin.plugin == "yorha_protocol"));
    assert!(!schema.languages.is_empty());
    let ids: Vec<&str> = schema.settings.iter().map(|setting| setting.id.as_str()).collect();
    for lang in &schema.languages {
        let translated = cli::schema(&fixture.cli, Some(lang)).unwrap();
        assert_eq!(&translated.lang, lang);
        assert_schema_is_understood(&translated);
        let translated_ids: Vec<&str> = translated.settings.iter().map(|setting| setting.id.as_str()).collect();
        assert_eq!(translated_ids, ids, "{lang} lists other settings");
    }
}

#[test]
fn every_value_get_returns_and_every_offered_choice_is_accepted_by_set() {
    let Some(fixture) = fixture() else { return };
    cli::migrate(&fixture.cli, false).unwrap();
    let schema = cli::schema(&fixture.cli, None).unwrap();
    let values = cli::get(&fixture.cli).unwrap();
    let mut current = Vec::new();
    for setting in &schema.settings {
        let value = values.get(&setting.id).unwrap_or_else(|| panic!("get has no value for {}", setting.id));
        current.push((setting.id.clone(), value.clone()));
    }
    let report = cli::set(&fixture.cli, &current, true).unwrap();
    assert!(report.dry_run);
    assert!(report.changed.is_empty(), "{:?}", report.changed);
    let offered: Vec<(String, Vec<Json>)> = schema
        .settings
        .iter()
        .map(|setting| {
            let choices = setting.choices.iter().map(|choice| choice.value.clone());
            let presets = setting.presets().iter().map(|preset| preset.value.clone());
            (setting.id.clone(), choices.chain(presets).collect::<Vec<_>>())
        })
        .filter(|(_, values)| !values.is_empty())
        .collect();
    assert!(
        offered
            .iter()
            .any(|(id, values)| id == SHADOW_RESOLUTION && values.contains(&json!(8192)))
    );
    let rounds = offered.iter().map(|(_, values)| values.len()).max().unwrap_or_default();
    for round in 0..rounds {
        let assignments: Vec<(String, Json)> = offered
            .iter()
            .filter_map(|(id, values)| values.get(round).map(|value| (id.clone(), value.clone())))
            .collect();
        let result = cli::set(&fixture.cli, &assignments, true);
        assert!(result.is_ok(), "{assignments:?} was refused: {result:?}");
    }
}

#[test]
fn a_rejected_value_is_reported_and_the_rest_is_written_then_reset() {
    let Some(fixture) = fixture() else { return };
    cli::migrate(&fixture.cli, false).unwrap();
    let assignments = vec![
        (SHADOW_RESOLUTION.to_owned(), json!(4000)),
        (GRAPHICS_BACKEND.to_owned(), json!("d3d12")),
    ];
    let error = cli::set(&fixture.cli, &assignments, false).unwrap_err();
    assert!(matches!(error, ConfigError::Nams { .. }), "{error:?}");
    assert_eq!(error.rejected_ids(), vec![SHADOW_RESOLUTION.to_owned()]);
    let (report, problems) = cli::set_accepted(&fixture.cli, &assignments).unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].id.as_deref(), Some(SHADOW_RESOLUTION));
    assert!(report.needs_restart());
    assert!(report.changed.iter().any(|change| change.id == GRAPHICS_BACKEND && change.new == json!("d3d12")));
    let values = cli::get(&fixture.cli).unwrap();
    assert_eq!(values.get(GRAPHICS_BACKEND), Some(&json!("d3d12")));
    assert_eq!(values.get(SHADOW_RESOLUTION), Some(&json!(2048)));
    let unknown = cli::set(&fixture.cli, &[("nope.missing".to_owned(), json!(1))], true).unwrap_err();
    assert_eq!(unknown.rejected_ids(), vec!["nope.missing".to_owned()]);
    cli::reset(&fixture.cli, &ResetTarget::Category("graphics".to_owned()), true).unwrap();
    cli::reset(&fixture.cli, &ResetTarget::All, true).unwrap();
    let reset = cli::reset(&fixture.cli, &ResetTarget::Ids(vec![GRAPHICS_BACKEND.to_owned()]), false).unwrap();
    assert!(reset.changed.iter().any(|change| change.id == GRAPHICS_BACKEND));
    let values = cli::get(&fixture.cli).unwrap();
    assert_eq!(values.get(GRAPHICS_BACKEND), Some(&json!("d3d11")));
}

#[test]
fn overlay_settings_from_appdata_move_into_the_game_folder() {
    let Some(fixture) = fixture() else { return };
    let legacy_dir = fixture.appdata.join("NAMS");
    std::fs::create_dir_all(&legacy_dir).unwrap();
    std::fs::write(legacy_dir.join("settings.json"), json!({ "firstPlaythrough": false }).to_string()).unwrap();
    cli::migrate(&fixture.cli, false).unwrap();
    assert!(!legacy_dir.join("settings.json").exists());
    let values = cli::get(&fixture.cli).unwrap();
    assert_eq!(values.get(FIRST_PLAYTHROUGH), Some(&Json::Bool(false)));
    let report = cli::set(&fixture.cli, &[(FIRST_PLAYTHROUGH.to_owned(), json!(true))], false).unwrap();
    assert!(report.changed.iter().any(|change| change.id == FIRST_PLAYTHROUGH));
    assert_eq!(cli::get(&fixture.cli).unwrap().get(FIRST_PLAYTHROUGH), Some(&json!(true)));
}
