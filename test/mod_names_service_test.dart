import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mod_names_service.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('yp_names_');
  });

  tearDown(() {
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  File namesFile() => File(p.join(gameDir.path, 'nams', 'mod_names.json'));

  test('missing file yields no names', () async {
    expect(await ModNamesService.load(gameDir.path), isEmpty);
  });

  test('a name round-trips through disk', () async {
    await ModNamesService.setName(gameDir.path, 'my_mod', 'Nicer Name');

    expect(await ModNamesService.load(gameDir.path), {'my_mod': 'Nicer Name'});
    expect(namesFile().existsSync(), isTrue);
  });

  test('renaming twice keeps only the latest name', () async {
    await ModNamesService.setName(gameDir.path, 'my_mod', 'First');
    await ModNamesService.setName(gameDir.path, 'my_mod', 'Second');

    expect(await ModNamesService.load(gameDir.path), {'my_mod': 'Second'});
  });

  test('names of other mods survive an unrelated rename', () async {
    await ModNamesService.setName(gameDir.path, 'a', 'Alpha');
    await ModNamesService.setName(gameDir.path, 'b', 'Beta');

    expect(await ModNamesService.load(gameDir.path),
        {'a': 'Alpha', 'b': 'Beta'});
  });

  test('clearing a name drops the entry and removes an empty file', () async {
    await ModNamesService.setName(gameDir.path, 'my_mod', 'Nicer Name');
    await ModNamesService.setName(gameDir.path, 'my_mod', null);

    expect(await ModNamesService.load(gameDir.path), isEmpty);
    expect(namesFile().existsSync(), isFalse,
        reason: 'an empty name store should not linger on disk');
  });

  test('blank names are treated as a reset', () async {
    await ModNamesService.setName(gameDir.path, 'my_mod', 'Nicer Name');
    await ModNamesService.setName(gameDir.path, 'my_mod', '   ');

    expect(await ModNamesService.load(gameDir.path), isEmpty);
  });

  test('corrupt json degrades to no names instead of throwing', () async {
    namesFile().parent.createSync(recursive: true);
    namesFile().writeAsStringSync('{not json');

    expect(await ModNamesService.load(gameDir.path), isEmpty);
  });

  test('an empty game dir is a no-op', () async {
    await ModNamesService.setName('', 'my_mod', 'X');
    expect(await ModNamesService.load(''), isEmpty);
  });
}
