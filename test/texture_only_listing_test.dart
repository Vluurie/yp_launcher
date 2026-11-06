import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/mods_service.dart';

void main() {
  test('a pack of dds files with a readme is a texture pack', () {
    expect(
      ModsService.isTextureOnlyListing([
        'Read me.txt',
        'SK_Res',
        r'SK_Res\desktop.ini',
        r'SK_Res\inject',
        r'SK_Res\inject\textures',
        r'SK_Res\inject\textures\04ad8a8f.dds',
        r'SK_Res\inject\textures\0a2b11c3.dds',
      ]),
      isTrue,
    );
  });

  test('dds next to game files is a mod', () {
    expect(
      ModsService.isTextureOnlyListing([
        'Kimono/data/pl/pl000d.dat',
        'Kimono/data/pl/pl000d.dtt',
        'Kimono/textures/pl000d_0.dds',
      ]),
      isFalse,
    );
  });

  test('dds next to a cpk is a mod', () {
    expect(
      ModsService.isTextureOnlyListing(['data100.cpk', 'textures/a.dds']),
      isFalse,
    );
  });

  test('dds next to mod.toml, entities or wax is a mod', () {
    expect(
      ModsService.isTextureOnlyListing(['mod.toml', 'textures/a.dds']),
      isFalse,
    );
    expect(
      ModsService.isTextureOnlyListing([
        'Sword/entities/weapons/sword.toml',
        'Sword/textures/a.dds',
      ]),
      isFalse,
    );
    expect(
      ModsService.isTextureOnlyListing([
        'Dress/wax/mods/Dress/outfits/dress.json',
        'Dress/textures/a.dds',
      ]),
      isFalse,
    );
  });

  test('an archive without dds is not a texture pack', () {
    expect(ModsService.isTextureOnlyListing(['readme.txt']), isFalse);
    expect(ModsService.isTextureOnlyListing(const []), isFalse);
  });
}
