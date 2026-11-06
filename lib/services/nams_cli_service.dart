import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/verify_result.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';

class NamsTexturePack {
  final String name;
  final String source;
  final String? mod;
  final String? character;
  final bool outfitConditional;
  final bool enabled;
  final int ddsCount;
  final int? loadOrderIndex;
  final String path;

  const NamsTexturePack({
    required this.name,
    required this.source,
    required this.outfitConditional,
    required this.enabled,
    required this.ddsCount,
    required this.path,
    this.mod,
    this.character,
    this.loadOrderIndex,
  });

  factory NamsTexturePack.fromJson(Map<String, dynamic> json) {
    return NamsTexturePack(
      name: json['name'] as String? ?? '',
      source: json['source'] as String? ?? 'standalone',
      mod: json['mod'] as String?,
      character: json['character'] as String?,
      outfitConditional: json['outfit_conditional'] as bool? ?? false,
      enabled: json['enabled'] as bool? ?? true,
      ddsCount: (json['dds_count'] as num?)?.toInt() ?? 0,
      loadOrderIndex: (json['load_order_index'] as num?)?.toInt(),
      path: json['path'] as String? ?? '',
    );
  }
}

class NamsInfo {
  final String? version;
  final String? nierInstallError;
  final bool gameBinPresent;
  final int? gameBinSize;
  final List<({String name, int size})> plugins;

  const NamsInfo({
    this.version,
    this.nierInstallError,
    this.gameBinPresent = false,
    this.gameBinSize,
    this.plugins = const [],
  });

  factory NamsInfo.fromJson(Map<String, dynamic> json) {
    final paths = json['paths'];
    final runtime = json['runtime_files'];
    final gameBin = runtime is Map ? runtime['game_bin'] : null;
    final plugins = json['plugins'];
    return NamsInfo(
      version: json['version'] as String?,
      nierInstallError: paths is Map
          ? paths['nier_install_error'] as String?
          : null,
      gameBinPresent: gameBin is Map && gameBin['present'] == true,
      gameBinSize: gameBin is Map ? (gameBin['size'] as num?)?.toInt() : null,
      plugins: plugins is List
          ? [
              for (final p in plugins.whereType<Map>())
                (
                  name: '${p['name'] ?? '?'}',
                  size: (p['size'] as num?)?.toInt() ?? 0,
                ),
            ]
          : const [],
    );
  }
}

const _verifyTimeout = Duration(seconds: 30);

class NamsCliService {
  NamsCliService._();

  static Future<List<NamsTexturePack>?> texturesList(
    String gameDir, {
    Duration? timeout,
  }) async {
    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final namsExe = paths['namsExe'];
      final launcherDir = paths['launcherDir'];
      if (namsExe == null || !File(namsExe).existsSync()) return null;

      final stdout = await _runForStdout(
        namsExe,
        ['textures', 'list', '--nier-path', gameDir, '--json'],
        workingDirectory: launcherDir,
        timeout: timeout,
      );
      if (stdout == null) return null;
      final decoded = jsonDecode(stdout);
      if (decoded is! Map || decoded['packs'] is! List) return null;
      return (decoded['packs'] as List)
          .whereType<Map>()
          .map((m) => NamsTexturePack.fromJson(Map<String, dynamic>.from(m)))
          .toList();
    } catch (_) {
      return null;
    }
  }

  static Future<NamsInfo?> info(String gameDir, {Duration? timeout}) async {
    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final namsExe = paths['namsExe'];
      if (namsExe == null || !File(namsExe).existsSync()) return null;
      final stdout = await _runForStdout(
        namsExe,
        ['info', '--nier-path', gameDir, '--json'],
        workingDirectory: paths['launcherDir'],
        timeout: timeout,
      );
      if (stdout == null) return null;
      final decoded = jsonDecode(stdout);
      if (decoded is! Map) return null;
      return NamsInfo.fromJson(Map<String, dynamic>.from(decoded));
    } catch (_) {
      return null;
    }
  }

  static Future<String?> _runForStdout(
    String exe,
    List<String> args, {
    String? workingDirectory,
    Duration? timeout,
  }) async {
    if (timeout == null) {
      final result = await Process.run(
        exe,
        args,
        workingDirectory: workingDirectory,
      );
      return result.exitCode == 0 ? result.stdout as String : null;
    }
    final process = await Process.start(
      exe,
      args,
      workingDirectory: workingDirectory,
    );
    final out = StringBuffer();
    const decoder = Utf8Decoder(allowMalformed: true);
    final stdoutDone = process.stdout.transform(decoder).forEach(out.write);
    final stderrDone = process.stderr.drain<void>();
    int code;
    try {
      code = await process.exitCode.timeout(timeout);
    } on TimeoutException {
      process.kill();
      return null;
    }
    await stdoutDone;
    await stderrDone;
    return code == 0 ? out.toString() : null;
  }

  static Future<VerifyOutcome> verify(
    String gameDir,
    AppLocalizations l10n,
  ) async {
    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final namsExe = paths['namsExe'];
      final launcherDir = paths['launcherDir'];
      if (namsExe == null ||
          launcherDir == null ||
          !File(namsExe).existsSync()) {
        return const VerifyOutcome(status: VerifyStatus.error);
      }

      // Build through the adapter so the command runs under the same
      // Proton/Wine runtime as launching. The launch wrapper (gamescope etc.)
      // is deliberately NOT applied: verify is a headless diagnostic and a GUI
      // wrapper would swallow the JSON on stdout.
      final LaunchCommand cmd;
      try {
        cmd = await PlatformAdapter.current.buildNamsCommand(
          namsArgs: namsVerifyArgs,
          namsExe: namsExe,
          gameDir: gameDir,
          gameExe: p.join(gameDir, AppStrings.gameExeName),
          launcherDir: launcherDir,
          l10n: l10n,
        );
      } on LaunchUnavailable {
        return const VerifyOutcome(status: VerifyStatus.noRuntime);
      }

      final process = await Process.start(
        cmd.command,
        cmd.args,
        workingDirectory: cmd.cwd,
        environment: cmd.env,
      );
      const decoder = Utf8Decoder(allowMalformed: true);
      final stdoutText = process.stdout.transform(decoder).join();
      final stderrText = process.stderr.transform(decoder).join();

      final int exitCode;
      try {
        exitCode = await process.exitCode.timeout(_verifyTimeout);
      } on TimeoutException {
        process.kill();
        return const VerifyOutcome(status: VerifyStatus.timedOut);
      }

      if (exitCode == 20 || exitCode == 21 || exitCode == 22) {
        return VerifyOutcome(
          status: VerifyStatus.steamNotRunning,
          exitCode: exitCode,
        );
      }
      if (exitCode == 64) {
        return VerifyOutcome(status: VerifyStatus.error, exitCode: exitCode);
      }

      final stdout = await stdoutText;
      final stderr = await stderrText;
      dynamic decoded;
      try {
        decoded = jsonDecode(stdout);
      } catch (_) {
        decoded = null;
      }
      if (decoded is! Map || decoded['checks'] is! List) {
        return VerifyOutcome(
          status: VerifyStatus.cannotParse,
          exitCode: exitCode,
          raw: '$stdout$stderr',
        );
      }

      final result = VerifyResult.fromJson(Map<String, dynamic>.from(decoded));
      return VerifyOutcome(
        status: result.ok ? VerifyStatus.ok : VerifyStatus.failed,
        result: result,
        exitCode: exitCode,
      );
    } catch (e) {
      return VerifyOutcome(status: VerifyStatus.error, raw: e.toString());
    }
  }
}
