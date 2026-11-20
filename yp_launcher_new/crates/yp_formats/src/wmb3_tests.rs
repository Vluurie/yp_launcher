use std::io::Cursor;
use std::path::PathBuf;

use crate::dat::DatArchive;
use crate::error::{FormatError, ModelFault};
use crate::testing::{
    TestLod, TestVertex, cube_spec, dds_dxt1_solid, model_archives, quad_spec, texture_pair,
};
use crate::wmb3::{MAX_TEXTURE_SIDE, Model, TextureFiles, decode_dds_texture, is_wmb3};

fn fault(bytes: &[u8]) -> ModelFault {
    match Model::parse(bytes, None) {
        Err(FormatError::Model(fault)) => fault,
        Err(other) => panic!("expected a model fault, got {other}"),
        Ok(_) => panic!("expected a model fault, got a model"),
    }
}

#[test]
fn parses_a_textured_quad() {
    let bytes = quad_spec().build();
    assert!(is_wmb3(&bytes));
    let model = Model::parse(&bytes, None).unwrap();
    assert_eq!(model.lod_names, ["LOD0"]);
    assert_eq!(model.meshes.len(), 1);
    let mesh = &model.meshes[0];
    assert_eq!(mesh.id, 0);
    assert_eq!(mesh.name, "Body");
    assert_eq!(mesh.lod_name, "LOD0");
    assert_eq!(mesh.lod_index, 0);
    assert_eq!(mesh.positions.len(), 4);
    assert_eq!(mesh.positions[2], [1.0, 1.0, 0.0]);
    assert_eq!(mesh.uvs[1], [1.0, 0.0]);
    assert_eq!(mesh.normals[0], [0.0, 0.0, 1.0]);
    assert_eq!(mesh.indices, [2, 1, 0, 3, 2, 0]);
    assert_eq!(mesh.albedo_texture_id, Some(0xA1));
    assert_eq!(mesh.normal_texture_id, None);
    assert_eq!(model.vertex_count(), 4);
    assert_eq!(model.triangle_count(), 2);
    assert_eq!(model.center, [0.0, 0.0, 0.0]);
    assert!((model.radius - 2.0_f32.sqrt()).abs() < 1e-6);
    assert!(model.textures.is_empty());
    assert_eq!(model.texture_ids().into_iter().collect::<Vec<_>>(), [0xA1]);
}

#[test]
fn batches_are_rebased_to_their_lowest_vertex() {
    let mut spec = quad_spec();
    spec.vertices.insert(
        0,
        TestVertex::new([9.0, 9.0, 9.0], [0.0, 0.0], [0.0, 1.0, 0.0]),
    );
    spec.indices = vec![1, 2, 3, 1, 3, 4];
    let model = Model::parse(&spec.build(), None).unwrap();
    let mesh = &model.meshes[0];
    assert_eq!(mesh.positions.len(), 4);
    assert_eq!(mesh.positions[0], [-1.0, -1.0, 0.0]);
    assert_eq!(mesh.indices, [2, 1, 0, 3, 2, 0]);
}

#[test]
fn wide_indices_and_extended_normals() {
    for flags in [1u32, 4, 5, 7, 10, 11, 12, 14] {
        let mut spec = quad_spec();
        spec.vertex_flags = flags;
        spec.wide_indices = true;
        spec.vertices[0].normal = [0.0, 1.0, 0.0];
        let model = Model::parse(&spec.build(), None)
            .unwrap_or_else(|error| panic!("flags {flags}: {error}"));
        assert_eq!(model.meshes[0].normals[0], [0.0, 1.0, 0.0], "flags {flags}");
        assert_eq!(model.meshes[0].indices, [2, 1, 0, 3, 2, 0], "flags {flags}");
    }
}

#[test]
fn several_lods_and_meshes() {
    let mut spec = cube_spec(&["Body", "Hair", "Armor"]);
    spec.lods.push(TestLod {
        name: String::new(),
        batch_start: 0,
        infos: vec![[0, 7, 3]],
    });
    let model = Model::parse(&spec.build(), None).unwrap();
    assert_eq!(model.lod_names, ["LOD0", "LOD 1"]);
    let names: Vec<(&str, u32)> = model
        .meshes
        .iter()
        .map(|mesh| (mesh.name.as_str(), mesh.lod_index))
        .collect();
    assert_eq!(
        names,
        [("Body", 0), ("Hair", 0), ("Armor", 0), ("Mesh 7", 1)]
    );
    assert_eq!(model.meshes[3].albedo_texture_id, None);
    assert_eq!(model.triangle_count(), 12 + 4);
    let ids: Vec<u32> = model.meshes.iter().map(|mesh| mesh.id).collect();
    assert_eq!(ids, [0, 1, 2, 3]);
}

#[test]
fn degenerate_batches_are_skipped() {
    let mut spec = quad_spec();
    spec.batches = vec![[0, 2], [0, 6]];
    spec.lods[0].infos = vec![[0, 0, 0], [0, 0, 0]];
    let model = Model::parse(&spec.build(), None).unwrap();
    assert_eq!(model.meshes.len(), 1);
}

#[test]
fn decodes_only_the_textures_the_renderer_uses() {
    let mut spec = quad_spec();
    spec.materials[0].push(("g_NormalMap".to_string(), 0xB2));
    let (index, payload) = texture_pair(&[
        (0xA1, dds_dxt1_solid(8, 8, 0xF800)),
        (0xB2, dds_dxt1_solid(4, 4, 0x07E0)),
        (0xFF, dds_dxt1_solid(4, 4, 0x001F)),
    ]);
    let files = TextureFiles { index, payload };
    let model = Model::parse(&spec.build(), Some(&files)).unwrap();
    assert_eq!(model.textures.len(), 1);
    let albedo = &model.textures[&0xA1];
    assert_eq!((albedo.width, albedo.height), (8, 8));
    assert!(
        albedo
            .rgba
            .chunks_exact(4)
            .all(|pixel| pixel[0] > 240 && pixel[3] == 255)
    );
    assert!(!model.textures.contains_key(&0xB2));
    assert!(!model.textures.contains_key(&0xFF));
    assert!(model.texture_ids().contains(&0xB2));
}

#[test]
fn combined_wtb_textures_use_absolute_offsets() {
    let dds = dds_dxt1_solid(4, 4, 0xFFFF);
    let mut wtb = crate::testing::texture_index(&[(0xA1, 0, dds.len() as u32)]);
    let at = wtb.len().div_ceil(16) * 16;
    wtb.resize(at, 0);
    wtb.extend_from_slice(&dds);
    let offsets_at = u32::from_le_bytes(wtb[12..16].try_into().unwrap()) as usize;
    wtb[offsets_at..offsets_at + 4].copy_from_slice(&(at as u32).to_le_bytes());
    let model = Model::parse(&quad_spec().build(), Some(&TextureFiles::combined(wtb))).unwrap();
    assert_eq!(model.textures[&0xA1].width, 4);
}

#[test]
fn broken_texture_files_are_ignored() {
    let files = TextureFiles {
        index: vec![1, 2, 3],
        payload: Vec::new(),
    };
    assert!(
        Model::parse(&quad_spec().build(), Some(&files))
            .unwrap()
            .textures
            .is_empty()
    );
    let (index, _) = texture_pair(&[(0xA1, dds_dxt1_solid(4, 4, 0))]);
    let files = TextureFiles {
        index,
        payload: vec![0; 4],
    };
    assert!(
        Model::parse(&quad_spec().build(), Some(&files))
            .unwrap()
            .textures
            .is_empty()
    );
}

#[test]
fn faults_are_typed() {
    assert_eq!(fault(b"WMB3"), ModelFault::NotWmb3);

    let mut flags = quad_spec();
    flags.vertex_flags = 3;
    assert_eq!(fault(&flags.build()), ModelFault::UnsupportedVertexFlags(3));

    let mut batch = quad_spec();
    batch.lods[0].batch_start = 5;
    assert_eq!(fault(&batch.build()), ModelFault::InvalidBatchIndex);

    let mut group = quad_spec();
    group.lods[0].infos = vec![[2, 0, 0]];
    assert_eq!(fault(&group.build()), ModelFault::InvalidVertexGroupIndex);

    let mut range = quad_spec();
    range.batches = vec![[3, 6]];
    assert_eq!(fault(&range.build()), ModelFault::OutsideEntry);

    let mut vertex = quad_spec();
    vertex.indices = vec![0, 1, 9, 0, 2, 3];
    assert_eq!(fault(&vertex.build()), ModelFault::OutsideEntry);

    let mut empty = quad_spec();
    empty.lods[0].infos.clear();
    assert_eq!(fault(&empty.build()), ModelFault::NoRenderableBatches);

    let mut table = quad_spec().build();
    table[60..64].copy_from_slice(&5000u32.to_le_bytes());
    assert_eq!(fault(&table), ModelFault::TooManyEntries);

    let mut truncated = quad_spec().build();
    let len = truncated.len();
    truncated.truncate(len - 30);
    assert!(Model::parse(&truncated, None).is_err());
}

#[test]
fn archives_from_the_test_helpers_hold_a_model() {
    let archives = model_archives(
        "wp0003",
        quad_spec().build(),
        &[(0xA1, dds_dxt1_solid(4, 4, 0xF800))],
    );
    let mut dtt = DatArchive::from_reader(Cursor::new(archives.dtt)).unwrap();
    let wmb = dtt.read_named("wp0003.wmb").unwrap().unwrap();
    let wtp = dtt.read_named("wp0003.wtp").unwrap().unwrap();
    let mut dat = DatArchive::from_reader(Cursor::new(archives.dat)).unwrap();
    let wta = dat.read_named("wp0003.wta").unwrap().unwrap();
    let model = Model::parse(
        &wmb,
        Some(&TextureFiles {
            index: wta,
            payload: wtp,
        }),
    )
    .unwrap();
    assert_eq!(model.textures.len(), 1);
}

#[test]
#[ignore = "needs YP_EXTRACTED_WP_DIR pointing at an extracted data006.cpk/wp folder"]
fn real_weapon_models_parse_with_textures() {
    let dir = PathBuf::from(std::env::var_os("YP_EXTRACTED_WP_DIR").expect("YP_EXTRACTED_WP_DIR"));
    let mut stems: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dtt"))
        })
        .filter_map(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
        .collect();
    stems.sort();
    let mut parsed = 0;
    for stem in stems {
        let mut dtt = DatArchive::open(&dir.join(format!("{stem}.dtt"))).unwrap();
        let mut dat = DatArchive::open(&dir.join(format!("{stem}.dat"))).unwrap();
        let Some(wmb) = dtt.read_named(&format!("{stem}.wmb")).unwrap() else {
            continue;
        };
        let files = match (
            dat.read_named(&format!("{stem}.wta")).unwrap(),
            dtt.read_named(&format!("{stem}.wtp")).unwrap(),
        ) {
            (Some(index), Some(payload)) => Some(TextureFiles { index, payload }),
            _ => None,
        };
        let model = match Model::parse(&wmb, files.as_ref()) {
            Ok(model) => model,
            Err(FormatError::Model(ModelFault::NoRenderableBatches)) => continue,
            Err(error) => panic!("{stem}: {error}"),
        };
        assert!(model.triangle_count() > 0, "{stem}");
        if files.is_some() {
            assert!(!model.textures.is_empty(), "{stem} decoded no textures");
        }
        parsed += 1;
    }
    assert!(parsed > 10);
}

fn with_smaller_level(mut dds: Vec<u8>, width: u32, height: u32, color_565: u16) -> Vec<u8> {
    let levels = u32::from_le_bytes(dds[28..32].try_into().unwrap()) + 1;
    dds[28..32].copy_from_slice(&levels.to_le_bytes());
    for _ in 0..width.div_ceil(4) * height.div_ceil(4) {
        dds.extend_from_slice(&color_565.to_le_bytes());
        dds.extend_from_slice(&0u16.to_le_bytes());
        dds.extend_from_slice(&[0, 0, 0, 0]);
    }
    dds
}

#[test]
fn large_textures_are_read_from_the_level_that_fits_the_preview() {
    let side = MAX_TEXTURE_SIDE * 4;
    let dds = dds_dxt1_solid(side, side, 0xF800);
    let dds = with_smaller_level(dds, side / 2, side / 2, 0x07E0);
    let dds = with_smaller_level(dds, side / 4, side / 4, 0x001F);
    let dds = with_smaller_level(dds, side / 8, side / 8, 0xFFFF);
    let texture = decode_dds_texture(&dds).unwrap();
    assert_eq!((texture.width, texture.height), (MAX_TEXTURE_SIDE, MAX_TEXTURE_SIDE));
    assert!(
        texture
            .rgba
            .chunks_exact(4)
            .all(|pixel| pixel[0] < 16 && pixel[1] < 16 && pixel[2] > 240 && pixel[3] == 255)
    );
}

#[test]
fn large_textures_without_smaller_levels_are_scaled_down() {
    let side = MAX_TEXTURE_SIDE * 2;
    let texture = decode_dds_texture(&dds_dxt1_solid(side, side / 2, 0xF800)).unwrap();
    assert_eq!((texture.width, texture.height), (MAX_TEXTURE_SIDE, MAX_TEXTURE_SIDE / 2));
    assert!(
        texture
            .rgba
            .chunks_exact(4)
            .all(|pixel| pixel[0] > 240 && pixel[1] < 16 && pixel[3] == 255)
    );
}

#[test]
fn textures_that_already_fit_keep_their_size() {
    let texture = decode_dds_texture(&dds_dxt1_solid(64, 32, 0x07E0)).unwrap();
    assert_eq!((texture.width, texture.height), (64, 32));
}

#[test]
fn a_truncated_level_falls_back_to_the_full_texture() {
    let side = MAX_TEXTURE_SIDE * 2;
    let mut dds = dds_dxt1_solid(side, side, 0xF800);
    dds[28..32].copy_from_slice(&2u32.to_le_bytes());
    let texture = decode_dds_texture(&dds).unwrap();
    assert_eq!((texture.width, texture.height), (MAX_TEXTURE_SIDE, MAX_TEXTURE_SIDE));
    assert!(texture.rgba.chunks_exact(4).all(|pixel| pixel[0] > 240));
}

#[test]
fn textures_decoded_side_by_side_match_the_ones_decoded_alone() {
    let sources: Vec<(u32, Vec<u8>)> = (0..24u32)
        .map(|id| (id, dds_dxt1_solid(8, 8, (id as u16 + 1) * 0x0841)))
        .collect();
    let jobs: Vec<(u32, &[u8])> = sources.iter().map(|(id, dds)| (*id, dds.as_slice())).collect();
    let mut decoded = crate::wmb3::decode_jobs(&jobs);
    decoded.sort_by_key(|(id, _)| *id);
    assert_eq!(decoded.len(), sources.len());
    for ((id, texture), (source_id, dds)) in decoded.iter().zip(&sources) {
        assert_eq!(id, source_id);
        assert_eq!(texture.rgba, decode_dds_texture(dds).unwrap().rgba);
    }
}

#[test]
fn models_in_an_older_layout_are_refused_before_parsing() {
    let current = quad_spec().build();
    assert!(crate::wmb3::is_game_layout(&current));
    assert!(Model::parse(&current, None).is_ok());

    let mut older = current.clone();
    older[4..8].copy_from_slice(&0x2015_1123_u32.to_le_bytes());
    assert!(is_wmb3(&older));
    assert!(!crate::wmb3::is_game_layout(&older));
    assert_eq!(fault(&older), ModelFault::OlderLayout(0x2015_1123));
    assert!(!crate::wmb3::is_game_layout(b"WMB3"));
    assert!(!crate::wmb3::is_game_layout(b"WMB4 "));
}
