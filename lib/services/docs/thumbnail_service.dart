import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/services/rust_bridge.dart';
import 'package:yp_launcher/src/rust/api/texture.dart';

class ThumbnailService {
  ThumbnailService._();

  static const defaultWidth = 256;
  static const defaultHeight = 128;

  static const storyWidth = 800;
  static const storyHeight = 560;

  static const acceptedExtensions = <String>[
    '.png',
    '.jpg',
    '.jpeg',
    '.webp',
    '.bmp',
    '.tga',
  ];

  static bool isAccepted(String path) {
    final extension = p.extension(path).toLowerCase();
    return acceptedExtensions.contains(extension);
  }

  static String textureNameFor(int itemId) => 'item_thumb_$itemId';

  static String outputDirFor(String gameDir, String modId) {
    return p.join(ModsService.modsDir(gameDir), modId, 'data', 'misctex');
  }

  static String misctexNameFor(int itemId) =>
      'misctex_${textureNameFor(itemId)}';

  static Future<MiscTexResult> create({
    required String sourceImagePath,
    required String gameDir,
    required String modId,
    required int itemId,
    int width = defaultWidth,
    int height = defaultHeight,
  }) {
    return createNamed(
      sourceImagePath: sourceImagePath,
      outputDir: outputDirFor(gameDir, modId),
      textureName: textureNameFor(itemId),
      width: width,
      height: height,
    );
  }

  static Future<MiscTexResult> createNamed({
    required String sourceImagePath,
    required String outputDir,
    required String textureName,
    int width = defaultWidth,
    int height = defaultHeight,
  }) async {
    await RustBridge.initialize();
    return createItemThumbnail(
      sourceImagePath: sourceImagePath,
      outputDir: outputDir,
      textureName: textureName,
      width: width,
      height: height,
    );
  }

  static Future<TexturePreview> preview(String dttPath) async {
    await RustBridge.initialize();
    return readMisctexPreview(dttPath: dttPath);
  }

  static Future<bool> exists({
    required String gameDir,
    required String modId,
    required int itemId,
  }) {
    final path = p.join(
      outputDirFor(gameDir, modId),
      '${misctexNameFor(itemId)}.dat',
    );
    return File(path).exists();
  }

  static Future<void> remove({
    required String gameDir,
    required String modId,
    required int itemId,
  }) async {
    final dir = outputDirFor(gameDir, modId);
    final stem = misctexNameFor(itemId);
    for (final extension in const ['.dat', '.dtt']) {
      final file = File(p.join(dir, '$stem$extension'));
      if (await file.exists()) await file.delete();
    }
  }
}
