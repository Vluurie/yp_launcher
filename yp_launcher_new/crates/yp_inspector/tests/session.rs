use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use yp_formats::dat::{self, DatEntry, DatKind};
use yp_formats::testing::{cube_spec, dds_dxt1_solid, model_archives, texture_index, texture_pair};
use yp_inspector::{
    ArchivePaths, ArchiveSide, Camera, CandidateId, InspectorError, MeshSelection, ProbeWarning,
    RenderStyle, Session, probe,
};

const RED_565: u16 = 0xF800;

struct Fixture {
    _dir: tempfile::TempDir,
    paths: ArchivePaths,
}

fn write_archives(stem: &str, dat: Vec<u8>, dtt: Vec<u8>) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let dat_path = dir.path().join(format!("{stem}.dat"));
    let dtt_path = dir.path().join(format!("{stem}.dtt"));
    std::fs::write(&dat_path, dat).unwrap();
    std::fs::write(&dtt_path, dtt).unwrap();
    Fixture {
        _dir: dir,
        paths: ArchivePaths::new(Some(dat_path), Some(dtt_path)),
    }
}

fn textured_cube(stem: &str, meshes: &[&str]) -> Fixture {
    let archives = model_archives(
        stem,
        cube_spec(meshes).build(),
        &[(0xC0BE, dds_dxt1_solid(8, 8, RED_565))],
    );
    write_archives(stem, archives.dat, archives.dtt)
}

fn open_first(fixture: &Fixture) -> Session {
    let probe = probe(&fixture.paths).unwrap();
    Session::open(&fixture.paths, probe.candidates[0].id).unwrap()
}

fn is_background(pixel: [u8; 4]) -> bool {
    pixel == RenderStyle::default().background
}

#[test]
fn probe_lists_wmb3_models_with_texture_flags() {
    let fixture = textured_cube("wp0003", &["Blade"]);
    let probe = probe(&fixture.paths).unwrap();
    assert!(probe.warnings.is_empty());
    assert_eq!(probe.candidates.len(), 1);
    let candidate = &probe.candidates[0];
    assert_eq!(candidate.name, "wp0003.wmb");
    assert_eq!(
        candidate.id,
        CandidateId {
            side: ArchiveSide::Dtt,
            entry_index: 0
        }
    );
    assert!(candidate.has_textures);
}

#[test]
fn probe_skips_non_wmb3_models_and_flags_missing_textures() {
    let dtt = dat::build(
        &[
            DatEntry::new("em1000.WMB", cube_spec(&["Body"]).build()),
            DatEntry::new("old.wmb", b"WMB4xxxx".to_vec()),
            DatEntry::new("note.txt", b"WMB3".to_vec()),
        ],
        DatKind::Dtt,
    )
    .unwrap();
    let dat = dat::build(&[DatEntry::new("em1000.wta", vec![0; 32])], DatKind::Dat).unwrap();
    let fixture = write_archives("em1000", dat, dtt);
    let probe = probe(&fixture.paths).unwrap();
    let names: Vec<&str> = probe
        .candidates
        .iter()
        .map(|candidate| candidate.name.as_str())
        .collect();
    assert_eq!(names, ["em1000.WMB"]);
    assert!(!probe.candidates[0].has_textures);
}

#[test]
fn probe_keeps_going_when_one_archive_is_unreadable() {
    let fixture = textured_cube("wp0003", &["Blade"]);
    let paths = ArchivePaths::new(
        Some(PathBuf::from("does/not/exist.dat")),
        fixture.paths.dtt.clone(),
    );
    let probe = probe(&paths).unwrap();
    assert_eq!(probe.candidates.len(), 1);
    assert!(matches!(
        &probe.warnings[..],
        [ProbeWarning::ArchiveUnreadable { side: ArchiveSide::Dat, path, .. }] if path == Path::new("does/not/exist.dat")
    ));
}

#[test]
fn probe_errors_are_typed() {
    assert!(matches!(
        probe(&ArchivePaths::default()),
        Err(InspectorError::NoArchives)
    ));
    let missing = ArchivePaths::new(Some(PathBuf::from("a.dat")), Some(PathBuf::from("a.dtt")));
    match probe(&missing) {
        Err(InspectorError::NoReadableArchive { warnings }) => assert_eq!(warnings.len(), 2),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn opening_a_session_summarises_the_model() {
    let fixture = textured_cube("wp0003", &["Blade", "Guard"]);
    let session = open_first(&fixture);
    let summary = session.summary();
    assert_eq!(summary.vertex_count, 16);
    assert_eq!(summary.triangle_count, 12);
    assert_eq!(summary.lod_names, ["LOD0"]);
    let meshes: Vec<(u32, &str, bool)> = summary
        .meshes
        .iter()
        .map(|mesh| (mesh.id, mesh.name.as_str(), mesh.visible))
        .collect();
    assert_eq!(meshes, [(0, "Blade", true), (1, "Guard", true)]);
    assert_eq!(session.model().textures.len(), 1);
    assert_eq!(session.selected_lod(), 0);
}

#[test]
fn open_errors_are_typed() {
    let fixture = textured_cube("wp0003", &["Blade"]);
    let only_dat = ArchivePaths::new(fixture.paths.dat.clone(), None);
    let candidate = CandidateId {
        side: ArchiveSide::Dtt,
        entry_index: 0,
    };
    assert!(matches!(
        Session::open(&only_dat, candidate),
        Err(InspectorError::MissingArchive(ArchiveSide::Dtt))
    ));
    let gone = CandidateId {
        side: ArchiveSide::Dtt,
        entry_index: 9,
    };
    assert!(matches!(
        Session::open(&fixture.paths, gone),
        Err(InspectorError::EntryMissing { index: 9, .. })
    ));
    let not_a_model = CandidateId {
        side: ArchiveSide::Dat,
        entry_index: 0,
    };
    assert!(matches!(
        Session::open(&fixture.paths, not_a_model),
        Err(InspectorError::Format(_))
    ));
}

#[test]
fn player_models_hide_armor_and_variants_by_default() {
    let fixture = textured_cube("pl0000", &["Body", "Armor_Top", "Serious_Face", "Hair"]);
    let session = open_first(&fixture);
    let visible: Vec<bool> = session.meshes().iter().map(|mesh| mesh.visible).collect();
    assert_eq!(visible, [true, false, false, true]);
    assert_eq!(session.default_visible_meshes(), &BTreeSet::from([0, 3]));

    let weapon = textured_cube("wp0003", &["Body", "Armor_Top"]);
    assert!(open_first(&weapon).meshes().iter().all(|mesh| mesh.visible));
}

#[test]
fn screen_textures_are_found_under_the_scr_suffix() {
    let (wta, wtp) = texture_pair(&[(0xC0BE, dds_dxt1_solid(4, 4, RED_565))]);
    let dat = dat::build(&[DatEntry::new("pl0000scr.wta", wta)], DatKind::Dat).unwrap();
    let dtt = dat::build(
        &[
            DatEntry::new("pl0000.wmb", cube_spec(&["Body"]).build()),
            DatEntry::new("pl0000scr.wtp", wtp),
        ],
        DatKind::Dtt,
    )
    .unwrap();
    let fixture = write_archives("pl0000", dat, dtt);
    assert_eq!(open_first(&fixture).model().textures.len(), 1);
}

#[test]
fn combined_wtb_textures_are_preferred() {
    let dds = dds_dxt1_solid(4, 4, RED_565);
    let mut wtb = texture_index(&[(0xC0BE, 64, dds.len() as u32)]);
    wtb.resize(64, 0);
    wtb.extend_from_slice(&dds);
    let dat = dat::build(&[DatEntry::new("bg.bxm", vec![0; 4])], DatKind::Dat).unwrap();
    let dtt = dat::build(
        &[
            DatEntry::new("ba0000.wmb", cube_spec(&["Rock"]).build()),
            DatEntry::new("BA0000.WTB", wtb),
        ],
        DatKind::Dtt,
    )
    .unwrap();
    let fixture = write_archives("ba0000", dat, dtt);
    assert!(probe(&fixture.paths).unwrap().candidates[0].has_textures);
    assert_eq!(open_first(&fixture).model().textures.len(), 1);
}

#[test]
fn visibility_and_lod_changes() {
    let fixture = textured_cube("wp0003", &["Blade", "Guard"]);
    let mut session = open_first(&fixture);
    session.set_mesh_visibility(1, false).unwrap();
    assert_eq!(session.is_mesh_visible(1), Some(false));
    assert_eq!(session.visible_meshes(), BTreeSet::from([0]));
    assert!(matches!(
        session.set_mesh_visibility(7, true),
        Err(InspectorError::MeshMissing(7))
    ));
    assert!(matches!(
        session.set_lod(1),
        Err(InspectorError::LodMissing(1))
    ));
    session.set_lod(0).unwrap();
    assert_eq!(session.show_only(&BTreeSet::from([1])), vec![0, 1]);
    assert_eq!(session.visible_meshes(), BTreeSet::from([1]));
}

#[test]
fn presets_apply_on_top_of_the_default_visibility() {
    let fixture = textured_cube("pl0000", &["Body", "Armor", "Hair"]);
    let mut session = open_first(&fixture);
    session.set_mesh_visibility(2, false).unwrap();
    let selection = MeshSelection {
        custom_assembly: false,
        force_show: BTreeSet::from(["Armor".to_string()]),
        ..MeshSelection::default()
    };
    assert_eq!(session.apply_preset(&selection), BTreeSet::from([0, 1, 2]));
    assert_eq!(session.visible_meshes(), BTreeSet::from([0, 1, 2]));

    let custom = MeshSelection {
        custom_assembly: true,
        visible_names: BTreeSet::from(["Hair".to_string()]),
        ..MeshSelection::default()
    };
    assert_eq!(session.apply_preset(&custom), BTreeSet::from([2]));
    assert_eq!(session.visible_meshes(), BTreeSet::from([2]));
}

#[test]
fn cpu_render_smoke() {
    let fixture = textured_cube("wp0003", &["Blade"]);
    let mut session = open_first(&fixture);
    let style = RenderStyle::default();
    let frame = session.render(&Camera::default(), 64, 48, &style).unwrap();
    assert_eq!(
        (frame.width, frame.height, frame.rgba.len()),
        (64, 48, 64 * 48 * 4)
    );
    let center = frame.pixel(32, 24).unwrap();
    assert!(!is_background(center));
    assert!(
        center[0] > 80 && center[1] < 40 && center[2] < 40,
        "center {center:?}"
    );
    assert!(is_background(frame.pixel(0, 0).unwrap()));
    assert!(is_background(frame.pixel(63, 47).unwrap()));

    let turned = session
        .render(
            &Camera {
                yaw: 0.7,
                pitch: 0.4,
                distance: 3.0,
                pan_x: 4.0,
                pan_y: -2.0,
            },
            64,
            48,
            &style,
        )
        .unwrap();
    assert_ne!(turned, frame);

    session.set_mesh_visibility(0, false).unwrap();
    let hidden = session.render(&Camera::default(), 64, 48, &style).unwrap();
    assert!(
        hidden
            .rgba
            .chunks_exact(4)
            .all(|pixel| is_background(pixel.try_into().unwrap()))
    );
}

#[test]
fn untextured_models_use_the_style_colour() {
    let archives = model_archives("wp0003", cube_spec(&["Blade"]).build(), &[]);
    let fixture = write_archives("wp0003", archives.dat, archives.dtt);
    let session = open_first(&fixture);
    let style = RenderStyle {
        background: [0, 0, 255, 255],
        untextured: [0, 250, 0, 255],
    };
    let frame = session.render(&Camera::default(), 32, 32, &style).unwrap();
    let center = frame.pixel(16, 16).unwrap();
    assert!(
        center[1] > 80 && center[0] == 0 && center[2] == 0,
        "center {center:?}"
    );
    assert_eq!(frame.pixel(0, 0), Some([0, 0, 255, 255]));
}

#[test]
fn render_size_limits_are_typed() {
    let fixture = textured_cube("wp0003", &["Blade"]);
    let session = open_first(&fixture);
    let style = RenderStyle::default();
    assert!(matches!(
        session.render(&Camera::default(), 8, 64, &style),
        Err(InspectorError::RenderTooSmall {
            width: 8,
            height: 64
        })
    ));
    assert!(matches!(
        session.render(&Camera::default(), 4096, 4096, &style),
        Err(InspectorError::RenderTooLarge { .. })
    ));
}

#[test]
#[ignore = "needs YP_EXTRACTED_WP_DIR pointing at an extracted data006.cpk/wp folder"]
fn renders_a_real_weapon() {
    let dir = PathBuf::from(std::env::var_os("YP_EXTRACTED_WP_DIR").expect("YP_EXTRACTED_WP_DIR"));
    let paths = ArchivePaths::new(Some(dir.join("wp0003.dat")), Some(dir.join("wp0003.dtt")));
    let probe = probe(&paths).unwrap();
    assert!(!probe.candidates.is_empty());
    assert!(probe.candidates[0].has_textures);
    let session = Session::open(&paths, probe.candidates[0].id).unwrap();
    assert!(!session.model().textures.is_empty());
    let broadside = Camera {
        yaw: std::f32::consts::FRAC_PI_2,
        pitch: 0.0,
        distance: 1.2,
        ..Camera::INSPECTOR
    };
    let frame = session
        .render(&broadside, 256, 256, &RenderStyle::default())
        .unwrap();
    let drawn: Vec<[u8; 4]> = frame
        .rgba
        .chunks_exact(4)
        .map(|pixel| pixel.try_into().unwrap())
        .filter(|pixel| !is_background(*pixel))
        .collect();
    assert!(drawn.len() > 300, "only {} pixels drawn", drawn.len());
    let mut plain = session.model().clone();
    plain.textures.clear();
    let plain_frame = Session::from_model(plain, false)
        .render(&broadside, 256, 256, &RenderStyle::default())
        .unwrap();
    assert_ne!(plain_frame.rgba, frame.rgba);
}

#[test]
fn probe_leaves_out_models_in_a_layout_the_game_does_not_read() {
    let mut older = cube_spec(&["Body"]).build();
    older[4..8].copy_from_slice(&0x2015_1123_u32.to_le_bytes());
    let dtt = dat::build(
        &[
            DatEntry::new("dbfffb.wmb", older.clone()),
            DatEntry::new("em1000.wmb", cube_spec(&["Body"]).build()),
        ],
        DatKind::Dtt,
    )
    .unwrap();
    let dat = dat::build(&[DatEntry::new("em1000.wta", vec![0; 32])], DatKind::Dat).unwrap();
    let both = write_archives("em1000", dat, dtt);
    let names: Vec<String> = probe(&both.paths)
        .unwrap()
        .candidates
        .into_iter()
        .map(|candidate| candidate.name)
        .collect();
    assert_eq!(names, ["em1000.wmb"]);

    let dtt = dat::build(&[DatEntry::new("dbfffb.wmb", older)], DatKind::Dtt).unwrap();
    let dat = dat::build(&[DatEntry::new("dbfffb.wta", vec![0; 32])], DatKind::Dat).unwrap();
    let only_older = write_archives("dbfffb", dat, dtt);
    let found = probe(&only_older.paths).unwrap();
    assert!(found.candidates.is_empty());
    assert!(found.warnings.is_empty());
}
