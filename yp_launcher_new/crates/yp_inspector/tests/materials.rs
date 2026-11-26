use std::path::PathBuf;
use yp_formats::wmb3::MaterialBlend;
use yp_inspector::{ArchivePaths, Camera, RenderStyle, Session, probe};

#[test]
fn translucent_shadow_drawn_first_in_the_archive_modulates_the_base_color() {
    use yp_formats::testing::quad_spec;
    use yp_formats::wmb3::{Model, Texture};

    let mut spec = quad_spec();
    spec.materials.push(vec![("g_AlbedoMap".into(), 0xB2)]);
    spec.batches.push(spec.batches[0]);
    spec.lods[0].infos = vec![[0, 0, 1], [0, 0, 0]];
    let mut bytes = spec.build();
    let table = u32::from_le_bytes(bytes[104..108].try_into().unwrap()) as usize;
    let name = bytes.len() as u32;
    bytes.extend_from_slice(b"Multiplication\0");
    bytes[table + 48 + 16..table + 48 + 20].copy_from_slice(&name.to_le_bytes());
    let mut model = Model::parse(&bytes, None).unwrap();
    assert_eq!(model.meshes[0].blend, MaterialBlend::Multiply);
    for (id, rgba) in [(0xA1, [90, 180, 120, 255]), (0xB2, [255, 0, 255, 128])] {
        model.textures.insert(
            id,
            Texture {
                width: 1,
                height: 1,
                rgba: rgba.to_vec(),
            },
        );
    }
    let mut session = Session::from_model(model, false);
    let sample = |session: &Session| {
        session
            .render(&Camera::default(), 80, 64, &RenderStyle::default())
            .unwrap()
            .pixel(39, 35)
            .unwrap()
    };
    let shadowed = sample(&session);
    session.set_mesh_visibility(0, false).unwrap();
    let base = sample(&session);
    assert_eq!(
        shadowed,
        [
            base[0],
            ((u16::from(base[1]) * 127 + 127) / 255) as u8,
            base[2],
            255
        ]
    );
}

#[test]
#[ignore = "requires YP_TEST_EYE_DTT pointing to the Alice outfit"]
fn alice_eye_shadow_preserves_the_iris() {
    let dtt = PathBuf::from(std::env::var_os("YP_TEST_EYE_DTT").unwrap());
    let paths = ArchivePaths::new(Some(dtt.with_extension("dat")), Some(dtt));
    let candidate = probe(&paths).unwrap().candidates[0].id;
    let mut model = Session::open(&paths, candidate).unwrap().model().clone();
    let eyes: Vec<_> = model
        .meshes
        .iter()
        .filter(|m| {
            m.lod_index == 0 && m.name == "facial_normal" && m.albedo_texture_id == Some(0x2fdb97d9)
        })
        .collect();
    let mut low = [f32::INFINITY; 3];
    let mut high = [f32::NEG_INFINITY; 3];
    let iris = eyes
        .iter()
        .find(|m| m.normal_texture_id == Some(0x34a222a2))
        .unwrap();
    for p in &iris.positions {
        for i in 0..3 {
            low[i] = low[i].min(p[i]);
            high[i] = high[i].max(p[i]);
        }
    }
    let shadow = eyes
        .iter()
        .find(|m| m.blend == MaterialBlend::Multiply)
        .unwrap()
        .id;
    model.center = std::array::from_fn(|i| 0.5 * (low[i] + high[i]));
    model.radius = (high[0] - low[0]) * 0.57;
    let mut opaque_model = model.clone();
    opaque_model
        .meshes
        .iter_mut()
        .find(|mesh| mesh.id == shadow)
        .unwrap()
        .blend = MaterialBlend::Opaque;
    let mut session = Session::from_model(model, true);
    let camera = Camera {
        pitch: 0.,
        ..Camera::INSPECTOR
    };
    let render = |s: &Session| {
        s.render(&camera, 560, 360, &RenderStyle::default())
            .unwrap()
    };
    let solid = render(&Session::from_model(opaque_model, true));
    let face = render(&session);
    session.set_mesh_visibility(shadow, false).unwrap();
    let bare = render(&session);
    if let Some(path) = std::env::var_os("YP_TEST_EYE_CAPTURE") {
        let path = PathBuf::from(path);
        image::save_buffer(
            path.with_extension("before.png"),
            &solid.rgba,
            solid.width,
            solid.height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        image::save_buffer(
            path,
            &face.rgba,
            face.width,
            face.height,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
    let lightened = |rgba: &[u8]| {
        rgba.chunks_exact(4)
            .zip(bare.rgba.chunks_exact(4))
            .filter(|(layer, base)| {
                layer[..3]
                    .iter()
                    .zip(&base[..3])
                    .any(|(&a, &b)| a > b.saturating_add(12))
            })
            .count()
    };
    assert!(
        lightened(&solid.rgba) > 100,
        "opaque shading should reproduce the white eye"
    );
    assert!(
        lightened(&face.rgba) < 20,
        "multiplicative shadow brightened the iris"
    );
}

#[test]
#[ignore = "requires YP_TEST_SHINOBI_DTT pointing to the Shinobi outfit"]
fn shinobi_skirt_exterior_and_lining_are_selected_by_facing() {
    let path = PathBuf::from(std::env::var_os("YP_TEST_SHINOBI_DTT").unwrap());
    let paths = ArchivePaths::new(Some(path.with_extension("dat")), Some(path));
    let session = Session::open(&paths, probe(&paths).unwrap().candidates[0].id).unwrap();
    let mut model = session.model().clone();
    model.center = [0., 1.035, -0.60];
    model.radius = 0.32;
    let mut session = Session::from_model(model, true);
    let meshes = session.meshes();
    let fabric = |s: &mut Session, outside: bool, lining: bool| {
        for m in &meshes {
            let batch = &s.model().meshes[m.id as usize];
            let show = m.name == "Skirt"
                && match batch.albedo_texture_id {
                    Some(0x541f36b6) => outside,
                    Some(0x541f36b7) => lining,
                    _ => false,
                };
            s.set_mesh_visibility(m.id, show).unwrap();
        }
    };
    let mut differences = Vec::new();
    for (side, pitch, outside) in [("outside", -1.35, true), ("lining", 1.35, false)] {
        let camera = Camera {
            yaw: 0.22,
            pitch,
            distance: 2.1,
            ..Camera::default()
        };
        let style = RenderStyle::default();
        fabric(&mut session, true, true);
        let both = session.render(&camera, 420, 420, &style).unwrap();
        fabric(&mut session, outside, !outside);
        let expected = session.render(&camera, 420, 420, &style).unwrap();
        let mut wrong = 0;
        let mut covered = 0;
        for (a, b) in both.rgba.chunks_exact(4).zip(expected.rgba.chunks_exact(4)) {
            if b == style.background {
                continue;
            }
            covered += 1;
            if a.iter().zip(b).any(|(&x, &y)| x.abs_diff(y) > 4) {
                wrong += 1;
            }
        }
        if let Some(dir) = std::env::var_os("YP_TEST_CLOTH_CAPTURE") {
            image::save_buffer(
                PathBuf::from(dir).join(format!("shinobi-skirt-{side}.png")),
                &both.rgba,
                both.width,
                both.height,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
        differences.push((side, wrong, covered));
    }
    println!("Mismatches against each isolated material: {differences:?}");
    for (side, wrong, covered) in differences {
        assert!(
            covered > 1000 && wrong * 1000 < covered,
            "{side}: {wrong}/{covered}"
        );
    }
}

#[test]
fn coplanar_opposite_materials_follow_the_viewing_side_in_either_draw_order() {
    use yp_formats::testing::quad_spec;
    use yp_formats::wmb3::{Model, Texture};

    let mut spec = quad_spec();
    spec.indices.extend([0, 2, 1, 0, 3, 2]);
    spec.batches.push([6, 6]);
    spec.lods[0].infos.push([0, 0, 1]);
    spec.materials.push(vec![("g_AlbedoMap".into(), 0xB2)]);
    for reverse in [false, true] {
        let mut model = Model::parse(&spec.build(), None).unwrap();
        if reverse {
            model.meshes.reverse();
        }
        for (id, rgba) in [(0xA1, [220, 0, 0, 255]), (0xB2, [0, 220, 0, 255])] {
            model.textures.insert(
                id,
                Texture {
                    width: 1,
                    height: 1,
                    rgba: rgba.to_vec(),
                },
            );
        }
        let session = Session::from_model(model, false);
        for (yaw, channel) in [(0., 0), (std::f32::consts::PI, 1)] {
            let camera = Camera {
                yaw,
                pitch: 0.23,
                ..Camera::default()
            };
            let style = RenderStyle::default();
            let frame = session.render(&camera, 96, 88, &style).unwrap();
            let surface: Vec<_> = frame
                .rgba
                .chunks_exact(4)
                .filter(|c| *c != style.background)
                .collect();
            assert!(surface.len() > 700);
            assert!(
                surface
                    .iter()
                    .all(|p| p[channel] > 0 && p[1 - channel] == 0)
            );
        }
    }
}
