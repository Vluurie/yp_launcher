import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/wine/steam_vdf.dart';

const _linuxVdf = '''
"libraryfolders"
{
	"0"
	{
		"path"		"/home/u/.local/share/Steam"
		"apps"
		{
			"524220"		"43881494773"
		}
	}
	"1"
	{
		"path"		"/mnt/games/SteamLibrary"
	}
}
''';

void main() {
  group('vdfPathEntries', () {
    test('returns every path entry in order', () {
      expect(vdfPathEntries(_linuxVdf), [
        '/home/u/.local/share/Steam',
        '/mnt/games/SteamLibrary',
      ]);
    });

    test('unescapes Windows-style backslashes', () {
      expect(
        vdfPathEntries(r'"path"		"C:\\Program Files (x86)\\Steam"'),
        [r'C:\Program Files (x86)\Steam'],
      );
    });

    test('empty content yields nothing', () {
      expect(vdfPathEntries(''), isEmpty);
    });
  });
}
