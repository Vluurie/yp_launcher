import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/ini_patch.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_classifier.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';

Directory _temp(String prefix) {
  final dir = Directory.systemTemp.createTempSync(prefix);
  addTearDown(() {
    try {
      dir.deleteSync(recursive: true);
    } catch (_) {}
  });
  return dir;
}

void _write(String filePath, String content) {
  File(filePath)
    ..parent.createSync(recursive: true)
    ..writeAsStringSync(content);
}

String _ini(String dir) =>
    File(p.join(dir, 'ReShade.ini')).readAsStringSync();

void main() {
  group('ReShade.ini search paths', () {
    test('a missing ini gets the effect and texture folders', () {
      final dest = _temp('yp_rs_ini_').path;

      ReShadeRuntime.ensureIniPaths(dest);

      final ini = _ini(dest);
      expect(
        IniPatch.getKey(ini, 'GENERAL', 'EffectSearchPaths'),
        ReShadeRuntime.shaderSearchPath,
      );
      expect(
        IniPatch.getKey(ini, 'GENERAL', 'TextureSearchPaths'),
        ReShadeRuntime.textureSearchPath,
      );
    });

    test('a plain .\\ search path no longer hides reshade-shaders', () {
      final dest = _temp('yp_rs_ini_').path;
      _write(
        p.join(dest, 'ReShade.ini'),
        '[GENERAL]\nEffectSearchPaths=.\\\nTextureSearchPaths=.\\\n',
      );

      ReShadeRuntime.ensureIniPaths(dest);

      final effects = IniPatch.getKey(
        _ini(dest),
        'GENERAL',
        'EffectSearchPaths',
      )!.split(',');
      expect(effects.first, ReShadeRuntime.shaderSearchPath);
    });

    test('a non-recursive reshade-shaders entry becomes recursive', () {
      final dest = _temp('yp_rs_ini_').path;
      Directory(p.join(dest, 'my-fx')).createSync();
      _write(
        p.join(dest, 'ReShade.ini'),
        '[GENERAL]\n'
            'EffectSearchPaths=.\\reshade-shaders\\Shaders,.\\my-fx\\**,'
            '.\\ReShade_shaders\\Shaders,C:\\Old\\Shaders\n',
      );

      ReShadeRuntime.ensureIniPaths(dest);

      expect(
        IniPatch.getKey(_ini(dest), 'GENERAL', 'EffectSearchPaths'),
        '${ReShadeRuntime.shaderSearchPath},.\\my-fx\\**',
      );
    });

    test('a correct ini is left as it is', () {
      final dest = _temp('yp_rs_ini_').path;
      const content =
          '[GENERAL]\n'
          'EffectSearchPaths=.\\reshade-shaders\\Shaders\\**\n'
          'TextureSearchPaths=.\\reshade-shaders\\Textures\\**\n';
      _write(p.join(dest, 'ReShade.ini'), content);

      ReShadeRuntime.ensureIniPaths(dest);

      expect(_ini(dest), content);
    });

    test('add-ons in reshade-addons get an AddonPath', () {
      final dest = _temp('yp_rs_ini_').path;
      _write(p.join(dest, 'reshade-addons', 'Upscaler.addon64'), 'x');

      ReShadeRuntime.ensureIniPaths(dest);

      expect(
        IniPatch.getKey(_ini(dest), 'ADDON', 'AddonPath'),
        ReShadeRuntime.addonSearchPath,
      );
    });

    test('add-ons next to the ReShade DLL keep the default AddonPath', () {
      final dest = _temp('yp_rs_ini_').path;
      _write(p.join(dest, 'Upscaler.addon64'), 'x');
      _write(p.join(dest, 'reshade-addons', 'Other.addon64'), 'x');

      ReShadeRuntime.ensureIniPaths(dest);

      expect(IniPatch.getKey(_ini(dest), 'ADDON', 'AddonPath'), isNull);
    });
  });

  group('importing from the game folder', () {
    test('shader folders named in the game ReShade.ini are found', () {
      final game = _temp('yp_rs_game_').path;
      Directory(p.join(game, 'ReShade_shaders', 'Shaders')).createSync(
        recursive: true,
      );
      Directory(p.join(game, 'reshade-shaders', 'Shaders')).createSync(
        recursive: true,
      );
      _write(
        p.join(game, 'ReShade.ini'),
        '[GENERAL]\n'
            'EffectSearchPaths=.\\ReShade_shaders\\Shaders\\**,'
            '.\\reshade-shaders\\Shaders\\**,.\\,..\\,.\\missing\n',
      );

      final dirs = ReShadeRuntime.referencedSearchDirs(
        game,
        'EffectSearchPaths',
      );

      expect(dirs, hasLength(1));
      expect(
        p.equals(dirs.single, p.join(game, 'ReShade_shaders', 'Shaders')),
        isTrue,
      );
    });
  });

  group('symbolic links', () {
    test('merging copies the content of a linked shader folder', () {
      final root = _temp('yp_rs_link_').path;
      final library = p.join(root, 'library', 'SweetFX');
      _write(p.join(library, 'Clarity.fx'), 'technique Clarity {}');
      final src = p.join(root, 'linked');
      Directory(src).createSync();
      try {
        Link(p.join(src, 'SweetFX')).createSync(library);
      } catch (_) {
        markTestSkipped('this system does not allow creating links');
        return;
      }

      final dest = p.join(root, 'out');
      FileOps.mergeDirectory(src, dest);

      final copied = File(p.join(dest, 'SweetFX', 'Clarity.fx'));
      expect(copied.readAsStringSync(), 'technique Clarity {}');
      expect(
        FileSystemEntity.isLinkSync(p.join(dest, 'SweetFX')),
        isFalse,
        reason: 'the copy is a real folder, not another link',
      );
    });

    test('merging copies what a linked shader file points to', () {
      if (Platform.isWindows) {
        markTestSkipped('Dart creates only directory links on Windows');
        return;
      }
      final root = _temp('yp_rs_link_').path;
      final real = p.join(root, 'library', 'Clarity.fx');
      _write(real, 'technique Clarity {}');
      final src = p.join(root, 'linked');
      Directory(src).createSync();
      try {
        Link(p.join(src, 'Clarity.fx')).createSync(real);
      } catch (_) {
        markTestSkipped('this system does not allow creating symlinks');
        return;
      }

      final dest = p.join(root, 'out');
      FileOps.mergeDirectory(src, dest);

      final copied = File(p.join(dest, 'Clarity.fx'));
      expect(copied.existsSync(), isTrue);
      expect(
        FileSystemEntity.isLinkSync(copied.path),
        isFalse,
        reason: 'the copy is a real file, not another link',
      );
      expect(copied.readAsStringSync(), 'technique Clarity {}');
    });
  });

  group('ReShade add-on archives', () {
    test('an add-on with upscaler DLLs is a ReShade install', () {
      final root = _temp('yp_rs_addon_').path;
      final folder = p.join(root, 'NierASR');
      _write(p.join(folder, 'NierASR.addon64'), 'addon');
      _write(p.join(folder, 'sl.interposer.dll'), 'dll');
      _write(p.join(folder, 'nvngx_dlss.dll'), 'dll');
      _write(p.join(folder, 'd3dcompiler_47.dll'), 'dll');
      _write(p.join(folder, 'dxgi.dll'), 'dll');

      final c = ThirdPartyClassifier.classify(root);
      expect(c.kind, ThirdPartyKind.reshadeWholeInstall);
      expect(c.isAddonBuild, isTrue);

      final companions =
          ReShadeRuntime.addonCompanionDlls([
              File(p.join(folder, 'NierASR.addon64')),
            ]).map((f) => p.basename(f.path)).toList()
            ..sort();
      expect(companions, ['nvngx_dlss.dll', 'sl.interposer.dll']);
    });
  });
}
