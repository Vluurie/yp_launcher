use std::fs;
use tempfile::tempdir;
use yp_formats::dat::{DatEntry, DatKind, build};
use yp_formats::mot::Motion;
use yp_formats::testing::cube_spec;
use yp_inspector::{ArchivePaths, Camera, MotionSources, RenderStyle, Session, probe};

fn put32(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn rigged_model() -> Vec<u8> {
    let mut spec = cube_spec(&["Outfit"]);
    spec.vertex_flags = 7;
    let mut bytes = spec.build();
    let get =
        |at, source: &[u8]| u32::from_le_bytes(source[at..at + 4].try_into().unwrap()) as usize;
    let vertices = get(get(56, &bytes), &bytes);
    for i in 0..spec.vertices.len() {
        bytes[vertices + i * 28 + 24] = 255;
    }
    let bones = bytes.len();
    bytes.resize(bones + 176, 0);
    for (i, (id, parent)) in [(11u16, u16::MAX), (73, 0)].into_iter().enumerate() {
        let at = bones + 88 * i;
        bytes[at..at + 2].copy_from_slice(&id.to_le_bytes());
        bytes[at + 2..at + 4].copy_from_slice(&parent.to_le_bytes());
        for field in [28, 32, 36, 64, 68, 72] {
            put32(&mut bytes, at + field, 1f32.to_bits());
        }
    }
    let map = bytes.len();
    bytes.extend_from_slice(&1u32.to_le_bytes());
    let sets = bytes.len();
    bytes.extend_from_slice(&((sets + 8) as u32).to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for (at, value) in [
        (40, bones),
        (44, 2),
        (88, map),
        (92, 1),
        (96, sets),
        (100, 1),
    ] {
        put32(&mut bytes, at, value as u32);
    }
    bytes
}

fn translation(bone: i16) -> Vec<u8> {
    let mut bytes = vec![0; 64];
    bytes[..4].copy_from_slice(b"mot\0");
    put32(&mut bytes, 4, 0x20120405);
    bytes[10..12].copy_from_slice(&2u16.to_le_bytes());
    put32(&mut bytes, 12, 44);
    put32(&mut bytes, 16, 1);
    bytes[44..46].copy_from_slice(&bone.to_le_bytes());
    bytes[47] = 1;
    bytes[48..50].copy_from_slice(&2u16.to_le_bytes());
    put32(&mut bytes, 52, 12);
    put32(&mut bytes, 60, 1f32.to_bits());
    bytes
}

#[test]
fn a_shared_motion_bank_animates_the_weighted_vertices_of_an_outfit() {
    let dir = tempdir().unwrap();
    let model = dir.path().join("outfit");
    let shared = dir.path().join("game-data");
    fs::create_dir_all(&model).unwrap();
    fs::create_dir_all(shared.join("pl")).unwrap();
    let dtt = model.join("pl000d.dtt");
    fs::write(
        &dtt,
        build(&[DatEntry::new("pl000d.wmb", rigged_model())], DatKind::Dtt).unwrap(),
    )
    .unwrap();
    let paths = ArchivePaths::new(None, Some(dtt));
    let options = MotionSources {
        mod_root: Some(model.clone()),
        game_data: Some(shared.clone()),
        mod_libraries: vec![
            dir.path().join("nams/mods"),
            dir.path().join("nams/mods_profile_outfits"),
        ],
    };
    let id = probe(&paths).unwrap().candidates[0].id;
    assert_eq!(
        Session::open_with_sources(&paths, id, &options)
            .unwrap()
            .selected_motion(),
        None
    );
    let bank = build(
        &[
            DatEntry::new("pl0000_003f.mot", translation(73)),
            DatEntry::new("pl0000_0031.mot", translation(11)),
        ],
        DatKind::Dat,
    )
    .unwrap();
    let profile_bank = options.mod_libraries[1].join("hair_pack/data/pl");
    fs::create_dir_all(&profile_bank).unwrap();
    fs::write(profile_bank.join("pl000f.dat"), &bank).unwrap();
    let mut session = Session::open_with_sources(&paths, id, &options).unwrap();
    assert_eq!(
        session.motion_names().collect::<Vec<_>>(),
        ["pl0000_003f.mot", "pl0000_0031.mot"]
    );
    assert_eq!(session.selected_motion(), Some(0));
    let sample = |seconds| {
        session
            .render_at(
                seconds,
                &Camera::default(),
                128,
                128,
                &RenderStyle::default(),
            )
            .unwrap()
            .rgba
    };
    assert_ne!(sample(0.), sample(1. / 60.));
    assert_eq!(sample(0.), sample(2. / 60.));
    let mesh = &session.model().meshes[0];
    let pose = session
        .model()
        .skeleton
        .pose(&Motion::parse(&translation(73)).unwrap(), 1. / 60.);
    let p = mesh.positions[0];
    assert_eq!(
        pose.vertex(mesh.weights.first(), p, mesh.normals[0]).0,
        [p[0] + 1., p[1], p[2]]
    );
    assert!(session.select_motion(1));
    fs::write(model.join("idle.mot"), translation(11)).unwrap();
    let local = Session::open_with_sources(&paths, id, &options).unwrap();
    assert_eq!(local.motion_names().next(), Some("idle.mot"));
    fs::write(model.join("bad.mot"), b"mot").unwrap();
    let incompatible = model.join("other.mot");
    fs::write(&incompatible, translation(266)).unwrap();
    let corrupt = build(
        &[DatEntry::new("pl0000_003f.mot", translation(266))],
        DatKind::Dat,
    )
    .unwrap();
    fs::write(shared.join("pl/pl000f.dat"), corrupt).unwrap();
    fs::remove_file(model.join("idle.mot")).unwrap();
    let fallback = Session::open_with_sources(&paths, id, &options).unwrap();
    let selected = fallback.selected_motion().unwrap();
    assert_eq!(
        fallback.motion_names().nth(selected),
        Some("pl0000_003f.mot")
    );
}

#[test]
#[ignore = "requires YP_TEST_MODEL_DTT and YP_TEST_MOTION_LIBRARY pointing to a local outfit and an installed mod library"]
fn installed_outfit_automatically_plays_its_shared_idle() {
    use std::path::PathBuf;
    let model = PathBuf::from(std::env::var_os("YP_TEST_MODEL_DTT").unwrap());
    let library = PathBuf::from(std::env::var_os("YP_TEST_MOTION_LIBRARY").unwrap());
    let paths = ArchivePaths::new(Some(model.with_extension("dat")), Some(model.clone()));
    let sources = MotionSources {
        mod_root: model.parent().map(ToOwned::to_owned),
        mod_libraries: vec![library],
        ..Default::default()
    };
    let candidate = probe(&paths).unwrap().candidates[0].id;
    let session = Session::open_with_sources(&paths, candidate, &sources).unwrap();
    let index = session
        .selected_motion()
        .expect("automatic compatible motion");
    assert_eq!(session.motion_names().nth(index), Some("pl0000_003f.mot"));
    let render = |time| {
        session
            .render_at(time, &Camera::default(), 320, 240, &RenderStyle::default())
            .unwrap()
    };
    assert_ne!(render(0.).rgba, render(1.).rgba);
}

#[test]
fn a_multi_character_outfit_resolves_the_player_bank_for_each_model() {
    let root = tempdir().unwrap();
    let models = root.path().join("outfit");
    let library = root.path().join("mods");
    let banks = library.join("motions/data/pl");
    fs::create_dir_all(&models).unwrap();
    fs::create_dir_all(&banks).unwrap();
    for family in ["pl0000", "pl0200"] {
        let entries = [DatEntry::new(format!("{family}_003f.mot"), translation(73))];
        let name = format!("{}f.dat", &family[..5]);
        fs::write(banks.join(name), build(&entries, DatKind::Dat).unwrap()).unwrap();
    }
    let sources = MotionSources {
        mod_root: Some(models.clone()),
        mod_libraries: vec![library],
        ..Default::default()
    };
    for (stem, idle) in [
        ("pl020d", "pl0200_003f.mot"),
        ("pl010d", "pl0000_003f.mot"),
        ("pl010e", "pl0000_003f.mot"),
    ] {
        let entries = [DatEntry::new(format!("{stem}.wmb"), rigged_model())];
        let path = models.join(format!("{stem}.dtt"));
        fs::write(&path, build(&entries, DatKind::Dtt).unwrap()).unwrap();
        let paths = ArchivePaths::new(None, Some(path));
        let id = probe(&paths).unwrap().candidates[0].id;
        let player = Session::open_with_sources(&paths, id, &sources).unwrap();
        assert_eq!(player.motion_names().collect::<Vec<_>>(), [idle]);
        assert_eq!(player.selected_motion(), Some(0));
    }
}

#[test]
#[ignore = "requires YP_TEST_KIMONO_DIR and YP_TEST_GAME_DATA from an installed game"]
fn kimono_nines_and_a2_animate_the_body_from_packed_game_motions() {
    use std::path::PathBuf;
    let root = PathBuf::from(std::env::var_os("YP_TEST_KIMONO_DIR").unwrap());
    let data = PathBuf::from(std::env::var_os("YP_TEST_GAME_DATA").unwrap());
    let options = MotionSources {
        mod_root: Some(root.clone()),
        game_data: Some(data),
        ..Default::default()
    };
    for (stem, expected, uniform) in [
        ("pl020d", "pl0200_003f.mot", "YoRHa Uniform 2"),
        ("pl010d", "pl0000_003f.mot", "YoRHa Uniform Prototype"),
    ] {
        use yp_mods::inspector_presets::{Character, STATE_EYEMASK, load_character};
        let presets = load_character(&root, Character::for_stem(stem).unwrap());
        let preset = presets
            .iter()
            .find(|p| p.name == uniform)
            .expect("WAX uniform preset");
        let selected_states = [(STATE_EYEMASK.to_owned(), true)].into_iter().collect();
        let selection = yp_inspector::MeshSelection {
            custom_assembly: preset.custom_assembly,
            visible_names: preset
                .visible_mesh_names(&selected_states)
                .into_iter()
                .collect(),
            force_hide: preset.force_hide_meshes.iter().cloned().collect(),
            force_show: preset.force_show_meshes.iter().cloned().collect(),
        };
        let file = root.join("data/pl").join(format!("{stem}.dtt"));
        let paths = ArchivePaths::new(Some(file.with_extension("dat")), Some(file.clone()));
        let id = probe(&paths).unwrap().candidates[0].id;
        let mut session = Session::open_with_sources(&paths, id, &options).unwrap();
        let chosen = session.selected_motion().expect("character idle");
        assert_eq!(session.motion_names().nth(chosen), Some(expected));
        session.apply_preset(&selection);
        let cam = Camera {
            distance: 2.12,
            pitch: -0.05,
            ..Camera::default()
        };
        let style = RenderStyle::default();
        let body = ArchivePaths::new(None, Some(file));
        let body_id = probe(&body).unwrap().candidates[0].id;
        let mut body = Session::open_with_sources(&body, body_id, &options).unwrap();
        assert_eq!(body.cloth_group_count(), 0);
        body.apply_preset(&selection);
        let frame =
            |s: &mut Session, time| s.render_playback(time, &cam, 320, 320, &style).unwrap();
        let first = frame(&mut body, 0.);
        let second = frame(&mut body, 1.);
        let changed = first
            .rgba
            .chunks_exact(4)
            .zip(second.rgba.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        println!(
            "{stem}: {expected}, {} clips, {changed} changing pixels at one second with cloth disabled",
            session.motion_names().count()
        );
        assert!(changed > 100, "{stem}: {changed}");
        if let Some(dir) = std::env::var_os("YP_TEST_KIMONO_CAPTURE") {
            for i in 0..24 {
                let image = frame(&mut session, i as f32 / 12.);
                image::save_buffer(
                    PathBuf::from(&dir).join(format!("kimono-{stem}-idle-{i}.png")),
                    &image.rgba,
                    image.width,
                    image.height,
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }
        }
    }
}

#[test]
#[ignore = "requires YP_TEST_CLOTH_DTT pointing to the Alice outfit"]
fn installed_cloth_moves_without_a_character_motion_and_freezes_on_pause() {
    use std::path::PathBuf;
    let model = PathBuf::from(std::env::var_os("YP_TEST_CLOTH_DTT").unwrap());
    let paths = ArchivePaths::new(Some(model.with_extension("dat")), Some(model));
    let candidate = probe(&paths).unwrap().candidates[0].id;
    let mut session = Session::open(&paths, candidate).unwrap();
    assert!(session.selected_motion().is_none());
    assert_eq!(session.cloth_group_count(), 2);
    assert_eq!(session.cloth_bone_count(), 122);
    let model = session.model();
    let mut archive = yp_formats::DatArchive::open(paths.dat.as_deref().unwrap()).unwrap();
    let mut compare_collisions = |collision: bool| {
        let parameters = archive.read_named("pl000d_0_0_clp.bxm").unwrap().unwrap();
        let shapes = collision.then(|| archive.read_named("pl000d_0_0_clh.bxm").unwrap().unwrap());
        let group =
            yp_formats::cloth::ClothGroup::parse(&parameters, shapes.as_deref(), None).unwrap();
        let mut player = yp_formats::wmb3::ClothPlayer::bind(&model.skeleton, [group]);
        player.pose(&model.skeleton, None, 0.15).unwrap()
    };
    let free = compare_collisions(false);
    let colliding = compare_collisions(true);
    let contact = model
        .meshes
        .iter()
        .filter(|mesh| mesh.lod_index == 0)
        .any(|mesh| {
            mesh.positions.iter().zip(&mesh.weights).any(|(&p, w)| {
                let a = free.vertex(Some(w), p, [0., 1., 0.]).0;
                let b = colliding.vertex(Some(w), p, [0., 1., 0.]).0;
                a.into_iter().zip(b).any(|(a, b)| (a - b).abs() > 0.001)
            })
        });
    assert!(
        contact,
        "body collisions must change the skinned outfit surface"
    );
    let render = |session: &mut Session, time| {
        session
            .render_playback(time, &Camera::default(), 384, 480, &RenderStyle::default())
            .unwrap()
    };
    let rest = render(&mut session, 0.);
    let moved = render(&mut session, 0.15);
    let changed = rest
        .rgba
        .chunks_exact(4)
        .zip(moved.rgba.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 100, "only {changed} pixels changed");
    assert_eq!(moved.rgba, render(&mut session, 0.15).rgba);
    assert_eq!(rest.rgba, render(&mut session, 0.).rgba);
    println!(
        "{} cloth groups, {} bound bones, {changed} changed pixels with no character clip",
        session.cloth_group_count(),
        session.cloth_bone_count()
    );
    if let Some(folder) = std::env::var_os("YP_TEST_CLOTH_CAPTURE") {
        let folder = PathBuf::from(folder);
        for (name, frame) in [("cloth-rest.png", rest), ("cloth-moving.png", moved)] {
            image::save_buffer(
                folder.join(name),
                &frame.rgba,
                frame.width,
                frame.height,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }
}

fn cloth_parameters(bone: &str, up: &str, gravity: &str) -> Vec<u8> {
    let nodes: [(&str, &str, u16, u16); 11] = [
        ("CLOTH", "", 2, 1),
        ("CLOTH_HEADER", "", 3, 3),
        ("CLOTH_WK_LIST", "", 1, 6),
        ("m_Num", "1", 0, 0),
        ("m_GravityVec", gravity, 0, 0),
        ("m_SpdRate", "0.9", 0, 0),
        ("CLOTH_WK", "", 4, 7),
        ("no", bone, 0, 0),
        ("offset", "0 -1 0", 0, 0),
        ("rotLimit", "0.5", 0, 0),
        ("noUp", up, 0, 0),
    ];
    let mut strings = Vec::new();
    let mut data = Vec::new();
    let mut table = Vec::new();
    for (i, (name, value, count, first)) in nodes.iter().enumerate() {
        for v in [*count, *first, 0, i as u16] {
            table.extend(v.to_be_bytes());
        }
        for s in [name, value] {
            let offset = if s.is_empty() {
                u16::MAX
            } else {
                let offset = strings.len() as u16;
                strings.extend(s.as_bytes());
                strings.push(0);
                offset
            };
            data.extend(offset.to_be_bytes());
        }
    }
    let mut bytes = b"BXM\0\0\0\0\0".to_vec();
    for _ in 0..2 {
        bytes.extend((nodes.len() as u16).to_be_bytes());
    }
    bytes.extend((strings.len() as u32).to_be_bytes());
    bytes.extend(table);
    bytes.extend(data);
    bytes.extend(strings);
    bytes
}

#[test]
fn invalid_cloth_disables_the_model_simulation_and_keeps_the_static_preview_available() {
    let dir = tempdir().unwrap();
    let dat = dir.path().join("pl000d.dat");
    let dtt = dir.path().join("pl000d.dtt");
    fs::write(
        &dtt,
        build(&[DatEntry::new("pl000d.wmb", rigged_model())], DatKind::Dtt).unwrap(),
    )
    .unwrap();
    let paths = ArchivePaths::new(Some(dat.clone()), Some(dtt));
    let valid = cloth_parameters("73", "4095", "0.09 0 0");
    let mut bad_tree = valid.clone();
    bad_tree[18..20].copy_from_slice(&0u16.to_be_bytes());
    let render = |s: &mut Session, t| {
        s.render_playback(t, &Camera::default(), 96, 96, &RenderStyle::default())
            .unwrap()
            .rgba
    };
    let open = |entries: Vec<DatEntry>| {
        fs::write(&dat, build(&entries, DatKind::Dat).unwrap()).unwrap();
        let candidate = probe(&paths).unwrap().candidates[0].id;
        Session::open(&paths, candidate).unwrap()
    };
    let good = || DatEntry::new("pl000d_0_0_clp.bxm", valid.clone());
    let mut working = open(vec![good()]);
    let rest = render(&mut working, 0.);
    assert_ne!(rest, render(&mut working, 0.1));
    assert_eq!(rest, render(&mut working, f32::NAN));
    assert_eq!(working.cloth_group_count(), 0);
    assert_eq!(rest, render(&mut working, 0.2));
    for (file, bad) in [
        ("pl000d_0_1_clp.bxm", bad_tree),
        ("pl000d_0_1_clp.bxm", valid[..24].to_vec()),
        ("pl000d_0_1_clp.bxm", cloth_parameters("73", "73", "0 0 0")),
        (
            "pl000d_0_1_clp.bxm",
            cloth_parameters("94", "4095", "0 0 0"),
        ),
        (
            "pl000d_0_1_clp.bxm",
            cloth_parameters("73", "4095", "NaN 0 0"),
        ),
        (
            "pl000d_0_1_clp.bxm",
            cloth_parameters("73", "4095", "0 0 junk"),
        ),
        ("pl000d_0_0_clh.bxm", valid.clone()),
        ("pl000d_0_0_clw.bxm", Vec::new()),
    ] {
        let mut session = open(vec![good(), DatEntry::new(file, bad)]);
        assert_eq!(session.cloth_group_count(), 0);
        assert_eq!(rest, render(&mut session, 0.1));
    }
}

#[test]
fn an_unrepresentable_motion_pose_stops_playback() {
    let dir = tempdir().unwrap();
    let mut motion = translation(73);
    motion.resize(68, 0);
    put32(&mut motion, 16, 2);
    for (at, id) in [(44, 11i16), (56, 73i16)] {
        motion[at..at + 2].copy_from_slice(&id.to_le_bytes());
        motion[at + 2] = 7;
        motion[at + 3] = 0;
        put32(&mut motion, at + 8, 1.0e30f32.to_bits());
    }
    let dtt = dir.path().join("pl000d.dtt");
    fs::write(
        &dtt,
        build(
            &[
                DatEntry::new("pl000d.wmb", rigged_model()),
                DatEntry::new("idle.mot", motion),
            ],
            DatKind::Dtt,
        )
        .unwrap(),
    )
    .unwrap();
    let paths = ArchivePaths::new(None, Some(dtt));
    let mut session = Session::open(&paths, probe(&paths).unwrap().candidates[0].id).unwrap();
    assert!(session.selected_motion().is_some());
    let camera = Camera::default();
    let style = RenderStyle::default();
    let static_frame = session.render(&camera, 64, 64, &style).unwrap();
    assert_eq!(
        session
            .render_playback(0., &camera, 64, 64, &style)
            .unwrap(),
        static_frame
    );
    assert!(session.selected_motion().is_none());
}

#[test]
#[ignore = "requires YP_TEST_SHINOBI_DTT pointing to the Shinobi outfit"]
fn shinobi_forces_reach_the_weighted_skirt_with_repeated_bone_ids() {
    use yp_formats::{DatArchive, cloth::ClothGroup, wmb3::ClothPlayer};
    let dtt = std::path::PathBuf::from(std::env::var_os("YP_TEST_SHINOBI_DTT").unwrap());
    let dat = dtt.with_extension("dat");
    let paths = ArchivePaths::new(Some(dat.clone()), Some(dtt));
    let mut session = Session::open(&paths, probe(&paths).unwrap().candidates[0].id).unwrap();
    assert_eq!(
        (session.cloth_group_count(), session.cloth_bone_count()),
        (2, 81)
    );
    let model = session.model();
    let mut archive = DatArchive::open(&dat).unwrap();
    let mut groups = Vec::new();
    for entry in archive.entries().to_vec() {
        let Some(prefix) = entry.name.strip_suffix("_clp.bxm") else {
            continue;
        };
        let clp = archive.read_entry(&entry).unwrap();
        let clh = archive.read_named(&format!("{prefix}_clh.bxm")).unwrap();
        let clw = archive.read_named(&format!("{prefix}_clw.bxm")).unwrap();
        groups.push(ClothGroup::parse(&clp, clh.as_deref(), clw.as_deref()).unwrap());
    }
    let mesh = model
        .meshes
        .iter()
        .find(|m| m.lod_index == 0 && m.name == "Skirt" && m.positions.iter().any(|v| v[2] < -0.9))
        .unwrap();
    let tip_height = |pose: &yp_formats::wmb3::Pose| {
        let tips: Vec<_> = mesh
            .positions
            .iter()
            .zip(&mesh.weights)
            .filter(|(v, _)| v[2] < -0.9)
            .map(|(&v, w)| pose.vertex(Some(w), v, [0., 1., 0.]).0[1])
            .collect();
        assert!(tips.len() >= 4);
        tips.iter().sum::<f32>() / tips.len() as f32
    };
    let rig = &model.skeleton;
    let mut drops = Vec::new();
    for stride in [1, 4, 15] {
        let mut sim = ClothPlayer::bind(rig, groups.clone());
        let rest = tip_height(&sim.pose(rig, None, 0.).unwrap());
        for step in (stride..=120).step_by(stride) {
            sim.pose(rig, None, step as f32 / 60.).unwrap();
        }
        let settled = tip_height(&sim.pose(rig, None, 2.).unwrap());
        assert_eq!(settled, tip_height(&sim.pose(rig, None, 2.).unwrap()));
        drops.push(rest - settled);
    }
    println!("Rear hem vertical drop after two seconds at three sampling rates: {drops:?}");
    assert!(drops.iter().all(|&d| d > 0.2 && d < 1.2));
    if let Some(bank) = std::env::var_os("YP_TEST_SHARED_IDLE_DAT") {
        let mut bank = DatArchive::open(std::path::Path::new(&bank)).unwrap();
        let idle = Motion::parse(&bank.read_named("pl0000_003f.mot").unwrap().unwrap()).unwrap();
        let normal_pose = rig.try_pose(&idle, 2.).unwrap();
        let mut player = ClothPlayer::bind(rig, groups);
        for tick in 0..120 {
            player.pose(rig, Some(&idle), tick as f32 / 60.).unwrap();
        }
        let with_cloth = player.pose(rig, Some(&idle), 2.).unwrap();
        let drop = tip_height(&normal_pose) - tip_height(&with_cloth);
        println!("Hem drop against the identical character idle pose: {drop}");
        assert!(drop > 0.2 && drop < 1.2);
        if let Some(dir) = std::env::var_os("YP_TEST_CLOTH_CAPTURE") {
            let camera = Camera {
                yaw: std::f32::consts::FRAC_PI_2,
                distance: 1.9,
                pitch: -0.12,
                ..Camera::default()
            };
            for (name, pose) in [
                ("shinobi-idle-without-cloth.png", normal_pose),
                ("shinobi-idle-with-cloth.png", with_cloth),
            ] {
                let mut posed = model.clone();
                for mesh in &mut posed.meshes {
                    for (i, p) in mesh.positions.iter_mut().enumerate() {
                        (*p, mesh.normals[i]) =
                            pose.vertex(mesh.weights.get(i), *p, mesh.normals[i]);
                    }
                }
                let frame = Session::from_model(posed, true)
                    .render(&camera, 520, 480, &RenderStyle::default())
                    .unwrap();
                image::save_buffer(
                    std::path::PathBuf::from(&dir).join(name),
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }
        }
    }
    if let Some(dir) = std::env::var_os("YP_TEST_CLOTH_CAPTURE") {
        let camera = Camera {
            yaw: std::f32::consts::FRAC_PI_2,
            distance: 1.62,
            pitch: -0.1,
            ..Camera::default()
        };
        for step in 0..=60 {
            let frame = session
                .render_playback(
                    step as f32 / 30.,
                    &camera,
                    416,
                    400,
                    &RenderStyle::default(),
                )
                .unwrap();
            if step == 0 || step == 60 {
                let name = if step == 0 {
                    "shinobi-rest.png"
                } else {
                    "shinobi-settled.png"
                };
                image::save_buffer(
                    std::path::PathBuf::from(&dir).join(name),
                    &frame.rgba,
                    frame.width,
                    frame.height,
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }
        }
    }
}
