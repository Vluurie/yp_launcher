import 'dart:io';
import 'dart:typed_data';
import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/wine/steam_vdf.dart';

enum ExeVariant { missing, original, wolfLimitBreak, legacyWindows7, unknown }

class GameDetection {
  GameDetection._();

  static Future<List<String>> detectHDTextures(String gameDir) async {
    final found = <String>[];

    final skRes = Directory(path.join(gameDir, 'SK_Res'));
    if (await skRes.exists() && await _isNonEmpty(skRes)) {
      found.add('SK_Res/');
    }

    return found;
  }

  static Future<bool> _isNonEmpty(Directory dir) async {
    await for (final _ in dir.list(recursive: false)) {
      return true;
    }
    return false;
  }

  static const String wolfLimitBreakSha256 =
      'c9904a39e448d6cb3c98c28a90159cf9314753b0cfea5ceb4e76a12d3308a355';

  /// The legacy Windows 7/8 build of NieRAutomata.exe. NAMS cannot host it
  /// (needs the Windows 10/11 Steam build). Common on Proton/Linux, which
  /// misdetects the OS and downloads the Win7 executable. Values reported by
  /// NAMS itself in its "unsupported build" error.
  static const int legacyWindows7Size = 17786752;
  static const String legacyWindows7Sha1 =
      'f734bef5403c77e29cfc55f57707df19c3655751';

  /// The supported Windows 10/11 Steam build.
  static const int windows10Size = 17837952;
  static const String windows10Sha1 =
      '420cbab7daa1d94a79473c0a6271ef9c30e516e5';

  static const int steam2026Size = 17601536;
  static const String steam2026Sha1 =
      '537be3ea1083114cc1e444b85ac270f2043ac4d6';

  /// Steam build 26534214 (2026-11-09): the 2026-11 exe with Authenticode
  /// same image.
  static const int steam2026SignedSize = 17613568;
  static const String steam2026SignedSha1 =
      'f2fbdbbdddb939a992185dfa9ff404c8e25dd605';

  static const Set<int> supportedExeSizes = {
    windows10Size,
    steam2026Size,
    steam2026SignedSize,
  };

  static const int steam2021TimeDateStamp = 0x60d3a8d2;
  static const int steam2026TimeDateStamp = 0x6ac89519;

  static const String steamAppId = '524220';
  static const String dlcAppId = '580600';

  static const List<GameBuildInfo> knownBuilds = [
    GameBuildInfo(
      build: GameBuild.steam2021,
      label: 'Steam 2021',
      exeSize: windows10Size,
      exeSha1: windows10Sha1,
      timeDateStamp: steam2021TimeDateStamp,
    ),
    GameBuildInfo(
      build: GameBuild.steam2026_11,
      label: 'Steam 2026-11',
      exeSize: steam2026Size,
      exeSha1: steam2026Sha1,
      timeDateStamp: steam2026TimeDateStamp,
    ),
    GameBuildInfo(
      build: GameBuild.steam2026_11,
      label: 'Steam 2026-11 (signed)',
      exeSize: steam2026SignedSize,
      exeSha1: steam2026SignedSha1,
      timeDateStamp: steam2026TimeDateStamp,
    ),
  ];

  static GameBuildInfo? buildOfExe(int size, String sha1) {
    for (final info in knownBuilds) {
      if (info.exeSize == size && info.exeSha1 == sha1) return info;
    }
    return null;
  }

  static GameBuildInfo? buildOfTimeDateStamp(int? stamp) {
    if (stamp == null) return null;
    for (final info in knownBuilds) {
      if (info.timeDateStamp == stamp) return info;
    }
    return null;
  }

  static bool newerThanEveryBuild(int? stamp) =>
      stamp != null && knownBuilds.every((b) => stamp > b.timeDateStamp);

  static int? peTimeDateStamp(List<int> header) {
    final bytes = Uint8List.fromList(header);
    final data = ByteData.sublistView(bytes);
    int? u32(int at) => at >= 0 && at + 4 <= bytes.length
        ? data.getUint32(at, Endian.little)
        : null;
    if (bytes.length < 2 || bytes[0] != 0x4d || bytes[1] != 0x5a) return null;
    final nt = u32(0x3c);
    if (nt == null || u32(nt) != 0x00004550) return null;
    return u32(nt + 8);
  }

  static int? fileTimeDateStamp(String filePath) {
    try {
      final raf = File(filePath).openSync();
      try {
        return peTimeDateStamp(raf.readSync(0x1000));
      } finally {
        raf.closeSync();
      }
    } catch (_) {
      return null;
    }
  }

  static Future<ExeProbe> probeExe(String gameDir) => IsolateService.run(
    _probeExeSync,
    path.join(gameDir, AppStrings.gameExeName),
  );

  static String? steamManifestPath(String gameDir) {
    final common = path.dirname(path.normalize(gameDir));
    final steamapps = path.dirname(common);
    if (path.basename(common).toLowerCase() != 'common') return null;
    if (path.basename(steamapps).toLowerCase() != 'steamapps') return null;
    return path.join(steamapps, 'appmanifest_$steamAppId.acf');
  }

  static SteamAppManifest? readSteamManifest(String gameDir) {
    final manifestPath = steamManifestPath(gameDir);
    if (manifestPath == null) return null;
    final file = File(manifestPath);
    if (!file.existsSync()) return null;
    try {
      return SteamAppManifest.parse(file.readAsStringSync());
    } catch (_) {
      return null;
    }
  }

  static Future<bool> hasDlc(String gameDir) async {
    final manifest = readSteamManifest(gameDir);
    if (manifest != null) return manifest.dlcInstalled(dlcAppId);
    final file = File(path.join(gameDir, 'data', 'data100.cpk'));
    if (!await file.exists()) return false;
    final size = await file.length();
    const lower = (928 - 5) * 1024 * 1024;
    const upper = 937 * 1024 * 1024;
    return size >= lower && size < upper;
  }

  static Future<ExeVariant> detectExeVariant(String gameDir) async {
    final exePath = path.join(gameDir, AppStrings.gameExeName);
    final file = File(exePath);
    if (!await file.exists()) return ExeVariant.missing;

    final size = await file.length();
    if (size == legacyWindows7Size) {
      final sha1Hash = await IsolateService.run(_sha1FileSync, exePath);
      if (sha1Hash == legacyWindows7Sha1) return ExeVariant.legacyWindows7;
    }

    final hash = await IsolateService.run(_hashFileSync, exePath);
    if (hash == null) return ExeVariant.unknown;
    if (hash == wolfLimitBreakSha256) return ExeVariant.wolfLimitBreak;
    return ExeVariant.original;
  }
}

enum GameBuild { steam2021, steam2026_11 }

class GameBuildInfo {
  final GameBuild build;
  final String label;
  final int exeSize;
  final String exeSha1;
  final int timeDateStamp;

  const GameBuildInfo({
    required this.build,
    required this.label,
    required this.exeSize,
    required this.exeSha1,
    required this.timeDateStamp,
  });
}

enum ExeBuildStatus { missing, known, legacyWindows7, modified, newer, unknown }

class ExeProbe {
  final bool exists;
  final int? size;
  final String? sha1;
  final int? timeDateStamp;

  const ExeProbe({
    this.exists = false,
    this.size,
    this.sha1,
    this.timeDateStamp,
  });

  GameBuildInfo? get build => size == null || sha1 == null
      ? null
      : GameDetection.buildOfExe(size!, sha1!);

  GameBuildInfo? get stampBuild =>
      GameDetection.buildOfTimeDateStamp(timeDateStamp);

  ExeBuildStatus get status {
    if (!exists) return ExeBuildStatus.missing;
    if (build != null) return ExeBuildStatus.known;
    if (size == GameDetection.legacyWindows7Size &&
        sha1 == GameDetection.legacyWindows7Sha1) {
      return ExeBuildStatus.legacyWindows7;
    }
    if (stampBuild != null) return ExeBuildStatus.modified;
    if (GameDetection.newerThanEveryBuild(timeDateStamp)) {
      return ExeBuildStatus.newer;
    }
    return ExeBuildStatus.unknown;
  }
}

class SteamAppManifest {
  final String? buildId;
  final String? targetBuildId;
  final String? stateFlags;
  final String? lastUpdated;
  final String? betaKey;
  final Set<String> installedDlc;
  final Set<String> disabledDlc;

  const SteamAppManifest({
    this.buildId,
    this.targetBuildId,
    this.stateFlags,
    this.lastUpdated,
    this.betaKey,
    this.installedDlc = const {},
    this.disabledDlc = const {},
  });

  factory SteamAppManifest.parse(String content) {
    final root = parseVdf(content);
    final state = vdfBlock(root, 'AppState') ?? root;
    final dlc = <String>{};
    final depots = vdfBlock(state, 'InstalledDepots');
    if (depots != null) {
      for (final depot in depots.values) {
        if (depot is! Map) continue;
        final id = vdfString(depot, 'dlcappid');
        if (id != null && id.isNotEmpty) dlc.add(id);
      }
    }
    final userConfig = vdfBlock(state, 'UserConfig');
    final mounted = vdfBlock(state, 'MountedConfig');
    final disabledRaw =
        (mounted == null ? null : vdfString(mounted, 'DisabledDLC')) ??
        (userConfig == null ? null : vdfString(userConfig, 'DisabledDLC')) ??
        '';
    final beta = userConfig == null ? null : vdfString(userConfig, 'BetaKey');
    return SteamAppManifest(
      buildId: vdfString(state, 'buildid'),
      targetBuildId: vdfString(state, 'TargetBuildID'),
      stateFlags: vdfString(state, 'StateFlags'),
      lastUpdated: vdfString(state, 'LastUpdated'),
      betaKey: beta == null || beta.isEmpty ? null : beta,
      installedDlc: dlc,
      disabledDlc: disabledRaw
          .split(',')
          .map((s) => s.trim())
          .where((s) => s.isNotEmpty)
          .toSet(),
    );
  }

  bool dlcInstalled(String appId) =>
      installedDlc.contains(appId) && !disabledDlc.contains(appId);

  bool get updatePending =>
      targetBuildId != null &&
      buildId != null &&
      targetBuildId != '0' &&
      targetBuildId != buildId;

  DateTime? get lastUpdatedAt {
    final seconds = int.tryParse(lastUpdated ?? '');
    if (seconds == null || seconds <= 0) return null;
    return DateTime.fromMillisecondsSinceEpoch(seconds * 1000, isUtc: true);
  }
}

ExeProbe _probeExeSync(String exePath) {
  final file = File(exePath);
  if (!file.existsSync()) return const ExeProbe();
  try {
    final bytes = file.readAsBytesSync();
    return ExeProbe(
      exists: true,
      size: bytes.length,
      sha1: sha1.convert(bytes).toString(),
      timeDateStamp: GameDetection.peTimeDateStamp(
        bytes.length > 0x1000 ? bytes.sublist(0, 0x1000) : bytes,
      ),
    );
  } catch (_) {
    return ExeProbe(exists: true, size: _lengthOrNull(file));
  }
}

int? _lengthOrNull(File file) {
  try {
    return file.lengthSync();
  } catch (_) {
    return null;
  }
}

String? _hashFileSync(String filePath) {
  try {
    final bytes = File(filePath).readAsBytesSync();
    return sha256.convert(bytes).toString();
  } catch (_) {
    return null;
  }
}

String? _sha1FileSync(String filePath) {
  try {
    final bytes = File(filePath).readAsBytesSync();
    return sha1.convert(bytes).toString();
  } catch (_) {
    return null;
  }
}
