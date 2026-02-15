import 'dart:io';
import 'package:path/path.dart' as path;

class NamsConfigService {
  static const _lodmodToml = '''# NAMS LodMod - Visual quality improvements. Restart game after changes.

# Enable/disable all LodMod patches
enabled = false

# 0 = best quality (no LOD pop-in), 1 = vanilla, higher = further LOD distance
lod_multiplier = 0.0

# AO sharpness (0.1-2.0). Set to 2.0 for crisp ambient occlusion
ao_multiplier_width = 1.0
ao_multiplier_height = 1.0

# Shadow texture size. 4096 or 8192 for sharper shadows (vanilla: 2048)
shadow_resolution = 2048

# How far shadows reach. 2.0 = twice as far (vanilla: 1.0)
shadow_distance_multiplier = 1.0
# Advanced: min/max clamp for shadow distance. 0 = off
shadow_distance_minimum = 0.0
shadow_distance_maximum = 0.0
# Advanced: even shadow distribution factor. 0 = off, try 0.5-0.9
shadow_distance_pss = 0.0

# Advanced: shadow softness tweaks. 0 = no change
shadow_filter_strength_bias = 0.0
shadow_filter_strength_minimum = 0.0
shadow_filter_strength_maximum = 0.0

# Better shadow models on trees/foliage. Recommended: true
shadow_model_hq = false
# More objects cast shadows. Recommended: true
shadow_model_force_all = false

# Stops objects popping in/out. Recommended: true
disable_manual_culling = false

# Removes dark edges around the screen. Recommended: true
disable_vignette = false
# Disables the game's fake HDR effect
disable_fake_hdr = false
''';

  static const _textureInjectionToml = '''# NAMS Texture Injection - Controls how modded textures are loaded. Restart game after changes.

# Max texture size to preload into RAM at startup (bigger = longer loading, less stutter)
# 2048 = default, 4096 = preload 4K textures, 16384 = preload everything
preload_max_dimension = 2048

# Preload ALL textures into RAM regardless of size. No more stutter but needs lots of RAM (32GB+)
preload_all = false
''';

  static Future<void> ensureConfigs(String gameDir) async {
    final namsDir = Directory(path.join(gameDir, 'nams'));
    if (!await namsDir.exists()) {
      await namsDir.create(recursive: true);
    }

    await _writeIfMissing(path.join(namsDir.path, 'lodmod.toml'), _lodmodToml);
    await _writeIfMissing(
      path.join(namsDir.path, 'texture_injection.toml'),
      _textureInjectionToml,
    );
  }

  static Future<void> _writeIfMissing(String filePath, String content) async {
    final file = File(filePath);
    if (!await file.exists()) {
      await file.writeAsString(content);
    }
  }
}
