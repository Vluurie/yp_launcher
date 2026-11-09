import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/cutscene_detection_service.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/diagnostics_probes.dart';
import 'package:yp_launcher/services/diagnostics_service.dart';
import 'package:yp_launcher/services/launch_record_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/nams_cli_service.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';

const _dlcBytes = 930 * 1024 * 1024;

void _makeSizedFile(String path, int size) {
  final raf = File(path).openSync(mode: FileMode.write);
  try {
    raf.setPositionSync(size - 1);
    raf.writeByteSync(0);
  } finally {
    raf.closeSync();
  }
}

Directory _fakeGameDir() {
  final dir = Directory.systemTemp.createTempSync('yp_diag_');
  final data = Directory(p.join(dir.path, 'data'))..createSync(recursive: true);
  File(p.join(data.path, 'data000.cpk')).writeAsBytesSync(List.filled(16, 0));
  _makeSizedFile(p.join(data.path, 'data100.cpk'), _dlcBytes);
  File(p.join(data.path, 'hack.dll')).writeAsBytesSync(List.filled(8, 0));
  File(p.join(data.path, 'enemy.dtt')).writeAsBytesSync(List.filled(8, 0));
  final sound = Directory(p.join(data.path, 'sound'))..createSync();
  File(p.join(sound.path, 'bgm.wsp')).writeAsBytesSync(List.filled(8, 0));
  File(p.join(dir.path, AppStrings.gameExeName))
      .writeAsBytesSync(List.filled(GameDetection.windows10Size, 0));
  File(p.join(dir.path, 'steam_api64.dll')).writeAsBytesSync(List.filled(8, 0));
  final nams = Directory(p.join(dir.path, 'nams'))..createSync();
  File(p.join(nams.path, 'nams.toml'))
      .writeAsStringSync('disable_splash_screen = true\n');
  File(p.join(nams.path, 'lodmod.toml')).writeAsStringSync('');
  File(p.join(nams.path, 'texture_injection.toml')).writeAsStringSync('');
  return dir;
}

List<int> _peHeader(int stamp, {int nt = 0x80}) {
  final bytes = Uint8List(0x200);
  final data = ByteData.sublistView(bytes);
  bytes[0] = 0x4d;
  bytes[1] = 0x5a;
  data.setUint32(0x3c, nt, Endian.little);
  data.setUint32(nt, 0x00004550, Endian.little);
  data.setUint32(nt + 8, stamp, Endian.little);
  return bytes;
}

String _manifest({String target = '25901062', String disabledDlc = ''}) => '''
"AppState"
{
	"appid"		"524220"
	"buildid"		"25901062"
	"TargetBuildID"		"$target"
	"StateFlags"		"4"
	"LastUpdated"		"1793892324"
	"InstalledDepots"
	{
		"524221"
		{
			"manifest"		"2909777361921225087"
			"size"		"43732108109"
		}
		"580600"
		{
			"manifest"		"1599661617359645076"
			"size"		"43732108109"
			"dlcappid"		"580600"
		}
	}
	"UserConfig"
	{
		"language"		"english"
		"DisabledDLC"		"$disabledDlc"
	}
}
''';

DiagnosticsReport _bareReport({
  String gameDir = '',
  Map<String, MountFacts?> filesystems = const {},
  List<String> disabledModsEntries = const [],
  List<NamsTexturePack> texturePacks = const [],
  GameIdentity gameIdentity = const GameIdentity(),
  ThirdPartyReport thirdParty = const ThirdPartyReport(),
}) =>
    DiagnosticsReport(
      generatedAt: DateTime.utc(2026, 11, 6),
      gameDir: gameDir,
      systemInfo: const {},
      launcherInfo: {'Game dir': gameDir},
      mods: const [],
      disabledModsEntries: disabledModsEntries,
      texturePacks: texturePacks,
      texturePacksAvailable: texturePacks.isNotEmpty,
      cutsceneMods: const [],
      directOverrides: const [],
      cutsceneDetection: const CutsceneDetection(),
      namsTextures: const [],
      skResTextures: const [],
      waxTextures: const [],
      dataDirContents: const {},
      waxModsRoot: const [],
      skResPacks: const [],
      reshadeContents: const [],
      gameRootExtras: const [],
      filesystems: filesystems,
      gameIdentity: gameIdentity,
      thirdParty: thirdParty,
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  SharedPreferences.setMockInitialValues({});

  late Directory launcherDir;
  setUp(() {
    launcherDir = Directory.systemTemp.createTempSync('yp_diag_launcher_');
    LauncherSetupService.setRuntimeDirForTest(launcherDir.path);
  });
  tearDown(() {
    LauncherSetupService.setRuntimeDirForTest(null);
    try {
      launcherDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  test('vanilla drops list loose files in data/, not cpk or subfolders',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    final paths = r.vanillaDropped.map((f) => f.path).toList();

    expect(paths, contains('data/hack.dll'));
    expect(paths, contains('data/enemy.dtt'));
    expect(paths, isNot(contains('data/data000.cpk')),
        reason: 'core vanilla cpk must not be reported as a drop');
    expect(paths, isNot(contains('data/sound/bgm.wsp')),
        reason: 'vanilla subfolder contents are never scanned');
    expect(
      r.vanillaDropped
          .firstWhere((f) => f.path == 'data/hack.dll')
          .bucket,
      'data-root-loose',
    );
  });

  test('game identity detects DLC and a supported exe', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    expect(r.gameIdentity.hasDlc, isTrue);
    expect(r.gameIdentity.dlcSource, 'size check');
    expect(r.gameIdentity.exeSize, GameDetection.windows10Size);
    expect(r.gameIdentity.buildStatus, ExeBuildStatus.unknown,
        reason: 'right size but no PE header and the wrong sha1');
    expect(r.gameIdentity.exeVariantSupported, isFalse);
    expect(
      r.findings.any((f) =>
          f.level == FindingLevel.error &&
          f.message.contains('not a build NAMS knows')),
      isTrue,
    );
    expect(ExeVariant.values.map((v) => v.name),
        contains(r.gameIdentity.exeVariant));
  });

  test('exe with the 2026 PE stamp but another sha1 is a modified copy',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));
    File(p.join(game.path, AppStrings.gameExeName))
        .writeAsBytesSync(_peHeader(GameDetection.steam2026TimeDateStamp));

    final probe = await GameDetection.probeExe(game.path);
    expect(probe.timeDateStamp, GameDetection.steam2026TimeDateStamp);
    expect(probe.status, ExeBuildStatus.modified);
    expect(probe.stampBuild?.build, GameBuild.steam2026_11);

    final r = await DiagnosticsService.collect(game.path);
    expect(r.gameIdentity.buildDisplay, contains('Steam 2026-11'));
    expect(
      r.findings.first.message,
      allOf(
        contains('build stamp of Steam 2026-11'),
        contains('modified copy'),
      ),
    );
  });

  test('exe newer than every known build asks for a launcher update',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));
    File(p.join(game.path, AppStrings.gameExeName))
        .writeAsBytesSync(_peHeader(GameDetection.steam2026TimeDateStamp + 1));

    final r = await DiagnosticsService.collect(game.path);
    expect(r.gameIdentity.buildStatus, ExeBuildStatus.newer);
    expect(
      r.findings.any((f) => f.message.contains('Update the launcher')),
      isTrue,
    );
  });

  test('known builds map size and sha1 to 2021 and 2026', () {
    expect(
      GameDetection.buildOfExe(
        GameDetection.windows10Size,
        GameDetection.windows10Sha1,
      )?.build,
      GameBuild.steam2021,
    );
    expect(
      GameDetection.buildOfExe(
        GameDetection.steam2026Size,
        GameDetection.steam2026Sha1,
      )?.build,
      GameBuild.steam2026_11,
    );
    expect(
      GameDetection.buildOfExe(
        GameDetection.steam2026Size,
        GameDetection.windows10Sha1,
      ),
      isNull,
    );
    expect(
      GameDetection.buildOfTimeDateStamp(GameDetection.steam2021TimeDateStamp)
          ?.label,
      'Steam 2021',
    );
    expect(GameDetection.peTimeDateStamp([0x4d, 0x5a]), isNull);
    expect(GameDetection.peTimeDateStamp(_peHeader(0x12345678, nt: 0x40)),
        0x12345678);
  });

  test('Steam manifest gives the build id and the DLC depot', () async {
    final library = Directory.systemTemp.createTempSync('yp_diag_lib_');
    addTearDown(() => library.deleteSync(recursive: true));
    final steamapps = Directory(p.join(library.path, 'steamapps'));
    final game = Directory(p.join(steamapps.path, 'common', 'NieRAutomata'))
      ..createSync(recursive: true);
    Directory(p.join(game.path, 'data')).createSync();
    File(p.join(game.path, AppStrings.gameExeName))
        .writeAsBytesSync(_peHeader(GameDetection.steam2021TimeDateStamp));
    File(p.join(game.path, 'NieRAutomataCompat.exe')).writeAsBytesSync([0]);
    File(p.join(game.path, 'Latest_BuildInfo.properties'))
        .writeAsStringSync('Last_SVN_REVISION=7788 \n');
    File(p.join(steamapps.path, 'appmanifest_524220.acf'))
        .writeAsStringSync(_manifest(target: '25901063'));

    expect(await GameDetection.hasDlc(game.path), isTrue,
        reason: 'the DLC depot decides, data100.cpk is absent');

    final r = await DiagnosticsService.collect(game.path);
    final m = r.gameIdentity.steamManifest!;
    expect(m.buildId, '25901062');
    expect(m.updatePending, isTrue);
    expect(r.gameIdentity.dlcSource, 'Steam manifest');
    expect(r.gameIdentity.buildInfo['Last_SVN_REVISION'], '7788');
    expect(r.gameRootExtras, isNot(contains('NieRAutomataCompat.exe')));
    expect(r.gameRootExtras, isNot(contains('Latest_BuildInfo.properties')));
    expect(
      r.findings.any((f) => f.message.contains('update pending')),
      isTrue,
    );

    final disabled = SteamAppManifest.parse(
      _manifest(disabledDlc: GameDetection.dlcAppId),
    );
    expect(disabled.dlcInstalled(GameDetection.dlcAppId), isFalse);
  });

  test('findmnt output flags NTFS game folders', () {
    final fuse = DiagnosticsProbes.parseFindmnt(
      '{"filesystems": [{"target": "/mnt/games", "fstype": "fuseblk", '
      '"options": "rw,relatime"}]}',
    )!;
    expect(fuse.risky, isTrue);
    expect(fuse.target, '/mnt/games');
    final ext4 = DiagnosticsProbes.parseFindmnt(
      '{"filesystems": [{"target": "/", "fstype": "ext4", "options": "rw"}]}',
    )!;
    expect(ext4.risky, isFalse);

    final r = _bareReport(filesystems: {'game folder': fuse});
    final findings = DiagnosticsService.buildFindings(r);
    expect(
      findings.any((f) =>
          f.level == FindingLevel.error && f.message.contains('fuseblk')),
      isTrue,
    );
  });

  test('settings diff covers every key, unknown keys and parse errors',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));
    final nams = p.join(game.path, 'nams');
    File(p.join(nams, 'nams.toml')).writeAsStringSync(
      'disable_splash_screen = true\ndisable_splash_screeen = true\n'
      '[cutscene]\nhd_cutscenes = true\n'
      '[mouse]\nsensitivity = 3.0\n',
    );
    File(p.join(nams, 'naiom.toml')).writeAsStringSync(
      'sensitivity = 2\nthird_person_sensitivity_x = 1.7000000000000002\n',
    );
    File(p.join(nams, 'lodmod.toml')).writeAsStringSync('enabled = = true\n');

    final r = await DiagnosticsService.collect(game.path);
    expect(r.config.unknownKeys.map((d) => d.key),
        ['disable_splash_screeen']);
    expect(r.config.legacyKeys.map((d) => d.key),
        containsAll(['cutscene.hd_cutscenes', 'mouse.sensitivity']));
    expect(
      r.config.legacyKeys.firstWhere((d) => d.key == 'mouse.sensitivity').note,
      contains('naiom.toml'),
    );
    final naiom = r.config.deltas.where((d) => d.file == 'naiom.toml').toList();
    expect(naiom.map((d) => d.key), ['third_person_sensitivity_x'],
        reason: '2 and 2.0 are the same value');
    expect(naiom.single.value, '1.7');
    expect(
      r.findings.any((f) => f.message.contains('disable_splash_screeen')),
      isTrue,
    );
    expect(
      r.findings.any((f) => f.message.contains('hd_cutscenes')),
      isFalse,
      reason: 'old keys NAMS ignores are not findings',
    );
    expect(r.config.parseErrors.keys, contains('lodmod.toml'));
    expect(
      r.findings.any((f) =>
          f.level == FindingLevel.error &&
          f.message.startsWith('lodmod.toml does not parse')),
      isTrue,
    );
    final full = DiagnosticsService.formatFullReport(r);
    expect(full, contains('nams/lodmod.toml (does not parse)'));
    expect(full, isNot(contains('nams/nams.toml\n')),
        reason: 'files that parse are not dumped raw');
  });

  test('last launch, crash dump and hidden mod folders reach the findings',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));
    Directory(p.join(game.path, 'nams', 'mods', '_parked'))
        .createSync(recursive: true);

    final started = DateTime.now().toUtc().subtract(const Duration(seconds: 20));
    await LaunchRecordService.write(
      LaunchRecord(startedAt: started, label: 'NAMS')
          .finished(3, 'boom\nstack line'),
    );
    final logs = Directory(LogService.logsDirectory)..createSync(recursive: true);
    File(p.join(logs.path, 'crashdump_2026-11-06.txt'))
        .writeAsStringSync('EXCEPTION_ACCESS_VIOLATION\nat game.bin+0x1234\n');

    final r = await DiagnosticsService.collect(game.path);
    expect(r.lastLaunch?.exitCode, 3);
    expect(r.logs.crashDumpCount, 1);
    expect(r.logs.newestCrashDump?.lines.first, 'EXCEPTION_ACCESS_VIOLATION');
    expect(r.hiddenModFolders, ['_parked']);
    final messages = r.findings.map((f) => f.message).join('\n');
    expect(messages, contains('exit code 3'));
    expect(messages, contains('crash dump in the last session'));
    expect(messages, contains('start with "_"'));

    final full = DiagnosticsService.formatFullReport(r);
    expect(full, contains('exit code: 3'));
    expect(full, contains('stack line'));
    expect(full, contains('at game.bin+0x1234'));
  });

  test('texture packs disabled in disabled_mods.toml are not shown enabled',
      () {
    final gameDir = p.join(Directory.systemTemp.path, 'NieRAutomata');
    NamsTexturePack pack(String name, {bool enabled = true}) =>
        NamsTexturePack(
          name: name,
          source: 'standalone',
          outfitConditional: false,
          enabled: enabled,
          ddsCount: 1,
          path: p
              .join(gameDir, 'nams', 'inject', 'textures', name)
              .replaceAll('\\', '/'),
        );
    final r = _bareReport(
      gameDir: gameDir,
      disabledModsEntries: const ['inject/textures/Main Menu'],
      texturePacks: [
        pack('Main Menu'),
        pack('Main Menu Extra'),
        pack('Off', enabled: false),
      ],
    );
    expect(DiagnosticsService.packDisabledBy(r, r.texturePacks[0]),
        'disabled_mods.toml');
    expect(DiagnosticsService.packDisabledBy(r, r.texturePacks[1]), isNull,
        reason: 'a prefix only matches whole path segments');
    expect(DiagnosticsService.packDisabledBy(r, r.texturePacks[2]),
        'texture_injection.toml');
    final full = DiagnosticsService.formatFullReport(r);
    expect(full, contains('Main Menu  [standalone]  disabled (disabled_mods.toml)'));
  });

  test('game-root proxy DLLs are flagged, plugin files in data/ are not',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));
    for (final f in ['hack.dll', 'enemy.dtt']) {
      File(p.join(game.path, 'data', f)).deleteSync();
    }
    File(p.join(game.path, 'data', 'nao-nier-settings-0.bin'))
        .writeAsBytesSync([0]);
    File(p.join(game.path, 'dinput8.dll')).writeAsBytesSync([0]);
    File(p.join(game.path, 'd3d11.dll')).writeAsBytesSync([0]);
    Directory(p.join(game.path, 'SK_Res')).createSync();

    final r = await DiagnosticsService.collect(game.path);
    expect(r.vanillaDropped.map((f) => f.path),
        contains('data/nao-nier-settings-0.bin'));
    final messages = r.findings.map((f) => f.message).join('\n');
    expect(messages, isNot(contains('dropped loose')));
    expect(messages, contains('d3d11.dll in the game folder is never loaded'));
    expect(messages, contains('Special K (dinput8.dll in the game folder)'));
    expect(messages, isNot(contains('dinput8.dll in the game folder is never')));
  });

  test('2021-era game DLL mods on the 2026 build are flagged', () {
    ThirdPartyReport withMod(GameModSettings settings) => ThirdPartyReport(
          gameMods: ThirdPartyRuntimeStatus(
            installed: true,
            enabled: true,
            gameModsInfo: GameModsInfo(mods: [
              GameModEntry(fileName: 'celestial.dll', settings: settings),
            ]),
          ),
        );
    const on2026 = GameIdentity(
      buildStatus: ExeBuildStatus.known,
      buildLabel: 'Steam 2026-11',
    );

    final flagged = DiagnosticsService.buildFindings(
      _bareReport(gameIdentity: on2026, thirdParty: withMod(const GameModSettings())),
    );
    expect(flagged.map((f) => f.message).join(),
        contains('celestial.dll'));

    final told2026 = DiagnosticsService.buildFindings(_bareReport(
      gameIdentity: on2026,
      thirdParty: withMod(const GameModSettings(
          sizeOfImage: GameModSettings.steam2026SizeOfImage)),
    ));
    expect(told2026.map((f) => f.message).join(),
        isNot(contains('celestial.dll')));
  });

  test('launch record survives a json round trip', () {
    final start = DateTime.utc(2026, 11, 6, 10);
    final rec = LaunchRecord(startedAt: start, label: 'Proton')
        .finished(0, 'ok');
    final back = LaunchRecord.fromJson(rec.toJson())!;
    expect(back.startedAt, start);
    expect(back.exitCode, 0);
    expect(back.label, 'Proton');
    expect(back.outputTail, 'ok');
    expect(LaunchRecord(startedAt: start).stillRunning, isTrue);
  });

  test('redaction covers Wine-style and forward-slash paths', () {
    final r = _bareReport(gameDir: '/home/alice/Games/NieRAutomata');
    final out = DiagnosticsService.redact(
      r'Z:\home\alice\Games\NieRAutomata\data and '
      'Z:/home/alice/Games/NieRAutomata/nams and '
      '/home/alice/Games/NieRAutomata/logs',
      r,
    );
    expect(out, isNot(contains('alice')));
    expect('<gameDir>'.allMatches(out).length, 3);
  });

  test('username is only redacted inside home paths', () {
    addTearDown(() => DiagnosticsService.userNameForTest = null);
    final r = _bareReport();

    DiagnosticsService.userNameForTest = 'Windows';
    final win = DiagnosticsService.redact(
      r'OS: "Windows 10 Home" at D:\Users\Windows\AppData and C:\Windows\System32',
      r,
    );
    expect(win, contains('"Windows 10 Home"'));
    expect(win, contains(r'D:\Users\<user>\AppData'));
    expect(win, contains(r'C:\Windows\System32'));

    DiagnosticsService.userNameForTest = 'user';
    final linux = DiagnosticsService.redact(
      'user-added files in /home/user/Games and Z:\\home\\user\\x',
      r,
    );
    expect(linux, startsWith('user-added files'));
    expect(linux, contains('/home/<user>/Games'));
    expect(linux, contains(r'Z:\home\<user>\x'));
  });

  test('summary fits a Discord message', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));
    for (var i = 0; i < 60; i++) {
      File(p.join(game.path, 'data', 'loose_file_number_$i.dat'))
          .writeAsBytesSync([0]);
    }
    final r = await DiagnosticsService.collect(game.path);
    final summary = DiagnosticsService.formatSummary(r);
    expect(summary.length, lessThanOrEqualTo(1900));
    expect(summary, contains('Findings:'));
  });

  test('third-party runtimes report not installed for a clean dir', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    expect(r.thirdParty.reshade.installed, isFalse);
    expect(r.thirdParty.migoto.installed, isFalse);
  });

  test('config deltas surface a non-default nams.toml setting', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    final delta = r.configDeltas
        .where((d) => d.key == 'disable_splash_screen')
        .toList();
    expect(delta, hasLength(1));
    expect(delta.first.value, 'true');
    expect(delta.first.defaultValue, 'false');
    expect(delta.first.file, 'nams.toml');
  });

  test('nams health reports missing runtime files and unavailable textures',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    expect(r.namsHealth.missingFiles, contains(AppStrings.namsExeName));
    expect(r.namsHealth.missingFiles, contains(AppStrings.yorhaDllName));
    expect(r.namsHealth.namsExePresent, isFalse);
    expect(r.texturePacksAvailable, isFalse);
    expect(r.recentLogIssues, isEmpty);
  });

  test('empty gameDir returns an all-empty report without throwing', () async {
    final r = await DiagnosticsService.collect('');
    expect(r.vanillaDropped, isEmpty);
    expect(r.mods, isEmpty);
    expect(r.thirdParty.reshade.installed, isFalse);
    expect(r.configDeltas, isEmpty);
    expect(() => DiagnosticsService.formatFullReport(r), returnsNormally);
    expect(() => DiagnosticsService.formatSummary(r), returnsNormally);
  });

  test('full report is English and redacts the launch command path', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final preview = '${game.path}${p.separator}NAMS.exe run '
        '--nier-path ${game.path}';
    final r =
        await DiagnosticsService.collect(game.path, launchCommandPreview: preview);
    final full = DiagnosticsService.formatFullReport(r);

    expect(full, contains('YP Launcher Diagnostics Report'));
    expect(full, contains('Game identity'));
    expect(full, contains('<gameDir>'),
        reason: 'gameDir path must be redacted');
    expect(full, isNot(contains(game.path)),
        reason: 'raw game path must not leak into the report');
  });
}
