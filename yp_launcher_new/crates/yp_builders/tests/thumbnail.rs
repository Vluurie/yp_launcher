use std::path::Path;

use yp_builders::thumbnail;
use yp_core::GamePaths;

fn write_png(path: &Path, width: u32, height: u32) {
    let image = image::RgbaImage::from_fn(width, height, |x, y| image::Rgba([(x * 4) as u8, (y * 4) as u8, 128, 255]));
    image.save(path).unwrap();
}

#[test]
fn accepted_images_by_extension() {
    assert!(thumbnail::is_accepted(Path::new("a/icon.PNG")));
    assert!(thumbnail::is_accepted(Path::new("a/icon.jpeg")));
    assert!(thumbnail::is_accepted(Path::new("a/icon.tga")));
    assert!(!thumbnail::is_accepted(Path::new("a/icon.dds")));
    assert!(!thumbnail::is_accepted(Path::new("a/icon")));
}

#[test]
fn names_follow_the_item_id() {
    assert_eq!(thumbnail::texture_name_for(8500), "item_thumb_8500");
    assert_eq!(thumbnail::misctex_name_for(8500), "misctex_item_thumb_8500");
    let game = GamePaths::new("G");
    assert_eq!(
        thumbnail::output_dir_for(&game, "my_mod"),
        game.mods_dir().join("my_mod").join("data").join("misctex")
    );
}

#[test]
fn a_thumbnail_is_created_previewed_and_removed() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("icon.png");
    write_png(&source, 64, 32);
    let game = GamePaths::new(dir.path().join("game"));

    assert!(!thumbnail::exists(&game, "my_mod", 8500));
    let result = thumbnail::create(
        &source,
        &game,
        "my_mod",
        8500,
        thumbnail::DEFAULT_WIDTH,
        thumbnail::DEFAULT_HEIGHT,
    )
    .unwrap();

    let out_dir = thumbnail::output_dir_for(&game, "my_mod");
    assert_eq!(result.dat_path, out_dir.join("misctex_item_thumb_8500.dat"));
    assert_eq!(result.dtt_path, out_dir.join("misctex_item_thumb_8500.dtt"));
    assert!(thumbnail::exists(&game, "my_mod", 8500));

    let preview = thumbnail::preview(&result.dtt_path).unwrap();
    assert_eq!((preview.width, preview.height), (result.width, result.height));
    assert!(preview.png.starts_with(b"\x89PNG"));

    let scan = yp_builders::misctex_scan::scan(&game);
    assert_eq!(scan.len(), 1);
    assert_eq!(scan[0].texture_name, "item_thumb_8500");

    thumbnail::remove(&game, "my_mod", 8500).unwrap();
    assert!(!thumbnail::exists(&game, "my_mod", 8500));
    assert!(!result.dtt_path.exists());
    thumbnail::remove(&game, "my_mod", 8500).unwrap();
}

#[test]
fn an_unreadable_image_is_a_format_error() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("broken.png");
    std::fs::write(&source, b"not an image").unwrap();

    let result = thumbnail::create_named(&source, dir.path(), "x", 256, 128);

    assert!(matches!(result, Err(yp_builders::Error::Format(_))));
}
