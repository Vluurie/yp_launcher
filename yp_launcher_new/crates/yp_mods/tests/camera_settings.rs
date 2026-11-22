mod support;

use std::path::{Path, PathBuf};

use support::temp;
use yp_core::Error;
use yp_mods::camera::{self, CameraField, CameraFile, CameraSettings, TABLE};

fn write(root: &Path, rel: &str, text: &str) -> PathBuf {
    let path = rel
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    path
}

const CAMERA_FILE: &str = "# Camera tuned for the long dress\n\
[other]\n\
name = \"keep me\" # trailing note\n\
\n\
[player_camera_settings]\n\
# pull the camera back a little\n\
side_offset = 0.25 # shoulder\n\
height_offset = -0.5\n\
distance_offset = 1\n\
pitch_up_limit = 80\n\
\n\
[[mesh_override]]\n\
model = \"pl2b\"\n\
mesh = \"Cape\"\n\
force = \"show\"\n";

#[test]
fn defaults_match_the_game_defaults() {
    assert_eq!(
        CameraSettings::default(),
        CameraSettings {
            side_offset: 0.0,
            height_offset: 0.0,
            distance_offset: 0.0,
            fov_offset: 0.0,
            pitch_up_limit: 72.0,
            pitch_down_limit: 72.0,
        }
    );
    assert_eq!(TABLE, "player_camera_settings");
}

#[test]
fn field_specs_cover_every_setting() {
    let keys: Vec<&str> = CameraField::ALL.iter().map(|field| field.key()).collect();
    assert_eq!(
        keys,
        vec![
            "side_offset",
            "height_offset",
            "distance_offset",
            "fov_offset",
            "pitch_up_limit",
            "pitch_down_limit",
        ]
    );
    let ranges: Vec<(f64, f64, f64, u32)> = CameraField::ALL
        .iter()
        .map(|field| (field.min(), field.max(), field.step(), field.decimals()))
        .collect();
    assert_eq!(
        ranges,
        vec![
            (-3.0, 3.0, 0.05, 2),
            (-2.0, 2.0, 0.05, 2),
            (-5.0, 10.0, 0.05, 2),
            (-25.0, 60.0, 1.0, 0),
            (1.0, 89.0, 1.0, 0),
            (1.0, 89.0, 1.0, 0),
        ]
    );
    let mut settings = CameraSettings::default();
    for (index, field) in CameraField::ALL.into_iter().enumerate() {
        assert_eq!(settings.get(field), field.default_value());
        settings.set(field, index as f64 + 0.5);
    }
    assert_eq!(
        settings,
        CameraSettings {
            side_offset: 0.5,
            height_offset: 1.5,
            distance_offset: 2.5,
            fov_offset: 3.5,
            pitch_up_limit: 4.5,
            pitch_down_limit: 5.5,
        }
    );
}

#[test]
fn finds_the_settings_and_fills_missing_keys_with_defaults() {
    let dir = temp("yp_camera_find_");
    let file = write(dir.path(), "entities/camera.toml", CAMERA_FILE);

    let found = camera::find(dir.path()).unwrap();

    assert_eq!(found.path, file);
    assert_eq!(
        found.settings,
        CameraSettings {
            side_offset: 0.25,
            height_offset: -0.5,
            distance_offset: 1.0,
            fov_offset: 0.0,
            pitch_up_limit: 80.0,
            pitch_down_limit: 72.0,
        }
    );
}

#[test]
fn values_that_are_not_numbers_fall_back_to_the_defaults() {
    let dir = temp("yp_camera_types_");
    write(
        dir.path(),
        "entities/camera.toml",
        "[player_camera_settings]\nside_offset = \"wide\"\nfov_offset = true\npitch_up_limit = [80]\npitch_down_limit = 60.5\n",
    );

    let found = camera::find(dir.path()).unwrap();

    assert_eq!(
        found.settings,
        CameraSettings {
            pitch_down_limit: 60.5,
            ..CameraSettings::default()
        }
    );
}

#[test]
fn an_empty_table_still_counts_and_inline_tables_are_read() {
    let dir = temp("yp_camera_forms_");
    let empty = dir.path().join("empty");
    write(&empty, "entities/camera.toml", "[player_camera_settings]\n");
    let inline = dir.path().join("inline");
    write(
        &inline,
        "entities/camera.toml",
        "player_camera_settings = { fov_offset = 12 }\n",
    );

    assert_eq!(
        camera::find(&empty).unwrap().settings,
        CameraSettings::default()
    );
    assert_eq!(camera::find(&inline).unwrap().settings.fov_offset, 12.0);
}

#[test]
fn only_toml_files_below_an_entities_folder_are_considered() {
    let dir = temp("yp_camera_scope_");
    let root = dir.path().join("mod");
    write(
        &root,
        "camera.toml",
        "[player_camera_settings]\nfov_offset = 1\n",
    );
    write(
        &root,
        "config/camera.toml",
        "[player_camera_settings]\nfov_offset = 2\n",
    );
    write(
        &root,
        "Entities/camera.toml",
        "[player_camera_settings]\nfov_offset = 3\n",
    );
    write(
        &root,
        "wrapper/entities/camera.json",
        "{\"player_camera_settings\": {\"fov_offset\": 4}}",
    );
    write(&root, "wrapper/entities/other.toml", "[weapon]\nid = 5\n");
    write(
        &root,
        "wrapper/entities/list.toml",
        "[[player_camera_settings]]\nfov_offset = 6\n",
    );
    write(
        &root,
        "wrapper/entities/scalar.toml",
        "player_camera_settings = 7\n",
    );
    write(
        &root,
        "wrapper/entities/broken.toml",
        "[player_camera_settings\nfov_offset = 8\n",
    );

    assert_eq!(camera::find(&root), None);
    assert_eq!(camera::find(&dir.path().join("missing")), None);

    let deep = write(
        &root,
        "wrapper/entities/nested/deep/Camera.TOML",
        "[player_camera_settings]\nfov_offset = 9\n",
    );
    assert_eq!(camera::find(&root).unwrap().path, deep);
}

#[test]
fn the_first_match_in_walk_order_wins() {
    let dir = temp("yp_camera_order_");
    let root = dir.path().join("mod");
    write(
        &root,
        "entities/zeta.toml",
        "[player_camera_settings]\nfov_offset = 1\n",
    );
    write(
        &root,
        "entities/Beta/inner.toml",
        "[player_camera_settings]\nfov_offset = 2\n",
    );
    write(&root, "entities/alpha.toml", "[weapon]\nid = 1\n");
    write(
        &root,
        "entities/camera.toml",
        "[player_camera_settings]\nfov_offset = 3\n",
    );

    assert_eq!(camera::find(&root).unwrap().settings.fov_offset, 2.0);

    std::fs::remove_dir_all(root.join("entities").join("Beta")).unwrap();
    assert_eq!(camera::find(&root).unwrap().settings.fov_offset, 3.0);
}

#[test]
fn saving_keeps_comments_and_other_tables() {
    let dir = temp("yp_camera_save_");
    write(dir.path(), "entities/camera.toml", CAMERA_FILE);
    let found = camera::find(dir.path()).unwrap();
    let mut settings = found.settings;
    settings.side_offset = 0.75;
    settings.fov_offset = 15.0;

    camera::save(&found, &settings).unwrap();

    let text = std::fs::read_to_string(&found.path).unwrap();
    assert_eq!(
        text,
        "# Camera tuned for the long dress\n\
[other]\n\
name = \"keep me\" # trailing note\n\
\n\
[player_camera_settings]\n\
# pull the camera back a little\n\
side_offset = 0.75 # shoulder\n\
height_offset = -0.5\n\
distance_offset = 1.0\n\
pitch_up_limit = 80.0\n\
fov_offset = 15.0\n\
pitch_down_limit = 72.0\n\
\n\
[[mesh_override]]\n\
model = \"pl2b\"\n\
mesh = \"Cape\"\n\
force = \"show\"\n"
    );
    assert_eq!(camera::find(dir.path()).unwrap().settings, settings);
}

#[test]
fn saving_the_same_values_twice_leaves_the_file_untouched() {
    let dir = temp("yp_camera_idempotent_");
    write(dir.path(), "entities/camera.toml", CAMERA_FILE);
    let found = camera::find(dir.path()).unwrap();

    camera::save(&found, &found.settings).unwrap();
    let first = std::fs::read_to_string(&found.path).unwrap();
    let stamp = std::fs::metadata(&found.path).unwrap().modified().unwrap();
    camera::save(&found, &found.settings).unwrap();

    assert_eq!(std::fs::read_to_string(&found.path).unwrap(), first);
    assert_eq!(
        std::fs::metadata(&found.path).unwrap().modified().unwrap(),
        stamp
    );
    assert_eq!(
        std::fs::read_dir(found.path.parent().unwrap())
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn saving_keeps_windows_line_endings() {
    let dir = temp("yp_camera_crlf_");
    write(
        dir.path(),
        "entities/camera.toml",
        "# note\r\n[player_camera_settings]\r\nfov_offset = 5.0\r\n",
    );
    let found = camera::find(dir.path()).unwrap();
    let settings = CameraSettings {
        fov_offset: 20.0,
        ..found.settings
    };

    camera::save(&found, &settings).unwrap();

    let text = std::fs::read_to_string(&found.path).unwrap();
    assert!(text.starts_with("# note\r\n[player_camera_settings]\r\nfov_offset = 20.0\r\n"));
    assert!(!text.replace("\r\n", "").contains('\n'));
}

#[test]
fn saving_into_an_inline_table_round_trips() {
    let dir = temp("yp_camera_inline_");
    let path = write(
        dir.path(),
        "entities/camera.toml",
        "# inline form\nplayer_camera_settings = { fov_offset = 12 }\n\n[weapon]\nid = 3\n",
    );
    let found = camera::find(dir.path()).unwrap();
    let settings = CameraSettings {
        side_offset: -1.25,
        ..found.settings
    };

    camera::save(&found, &settings).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("# inline form\nplayer_camera_settings = {"));
    assert!(text.ends_with("\n[weapon]\nid = 3\n"));
    assert_eq!(camera::find(dir.path()).unwrap().settings, settings);
}

#[test]
fn saving_refuses_a_missing_or_broken_file() {
    let dir = temp("yp_camera_errors_");
    let path = write(
        dir.path(),
        "entities/camera.toml",
        "[player_camera_settings]\nfov_offset = 5\n",
    );
    let found = camera::find(dir.path()).unwrap();

    std::fs::write(&path, "[player_camera_settings\nfov_offset = 5\n").unwrap();
    let broken = camera::save(&found, &CameraSettings::default());
    assert!(matches!(
        broken,
        Err(Error::TomlParse {
            path: Some(reported),
            ..
        }) if reported == path
    ));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "[player_camera_settings\nfov_offset = 5\n"
    );

    std::fs::remove_file(&path).unwrap();
    let missing = camera::save(
        &CameraFile {
            path: path.clone(),
            settings: CameraSettings::default(),
        },
        &CameraSettings::default(),
    );
    assert!(missing.is_err_and(|err| err.is_not_found()));
    assert!(!path.exists());
}
