import 'package:yp_launcher/l10n/app_localizations.dart';

class LaunchFailure {
  final int? code;
  final bool panic;
  final String headline;
  final String? phase;
  final String? osError;
  final String? cause;
  final String? fix;
  final String rawOutput;
  final String? capturedLogPath;

  const LaunchFailure({
    this.code,
    this.panic = false,
    required this.headline,
    this.phase,
    this.osError,
    this.cause,
    this.fix,
    required this.rawOutput,
    this.capturedLogPath,
  });

  String friendlyTitle(AppLocalizations l) {
    if (panic) return l.failTitlePanic;
    final c = code;
    if (c == null) return l.failTitleUnknown;
    return _categoryFor(c).title(l);
  }

  String friendlyExplanation(AppLocalizations l) {
    if (panic) return l.failExplanationPanic;
    final c = code;
    if (c == null) return l.failExplanationUnknown;
    return _categoryFor(c).explanation(l);
  }

  List<String> hints(AppLocalizations l) {
    if (panic) {
      return [l.failHintPanicShare, l.failHintPanicReboot];
    }
    final c = code;
    if (c == null) {
      return [
        l.failHintUnknownSpawned,
        l.failHintUnknownTaskManager,
        l.failHintUnknownOtherLauncher,
      ];
    }
    final cat = _categoryFor(c);
    return [
      if (fix != null && fix!.isNotEmpty) fix!,
      ...cat.extraHints.map((h) => h(l)),
    ];
  }

  static _Category _categoryFor(int code) {
    switch (code) {
      case 1:
        return _Category(
          title: (l) => l.failTitleNamsFailure,
          explanation: (l) => l.failExplanationNamsFailure,
          extraHints: [(l) => l.failHintShareReport],
        );
      case 2:
        return _Category(
          title: (l) => l.failTitleInstallNotFound,
          explanation: (l) => l.failExplanationInstallNotFound,
          extraHints: [
            (l) => l.failHintRepickDirectory,
            (l) => l.failHintVerifyFiles,
          ],
        );
      case 3:
        return _Category(
          title: (l) => l.failTitleFolderCreate,
          explanation: (l) => l.failExplanationFolderCreate,
          extraHints: [
            (l) => l.failHintWritableFolder,
            (l) => l.failHintProgramFiles,
          ],
        );
      case 4:
        return _Category(
          title: (l) => l.failTitleRuntimePrep,
          explanation: (l) => l.failExplanationRuntimePrep,
          extraHints: [
            (l) => l.failHintAntivirusExclusions,
            (l) => l.failHintWritableCache,
          ],
        );
      case 5:
      case 6:
      case 7:
      case 8:
      case 9:
        return _Category(
          title: (l) => l.failTitleHostFailure,
          explanation: (l) => l.failExplanationHostFailure,
          extraHints: [
            (l) => l.failHintReboot,
            (l) => l.failHintAntivirusExclusions,
            (l) => l.failHintPersistShare,
          ],
        );
      case 20:
        return _Category(
          title: (l) => l.failTitleSteamNotRunning,
          explanation: (l) => l.failExplanationSteamNotRunning,
          extraHints: [(l) => l.failHintStartSteam],
        );
      case 21:
        return _Category(
          title: (l) => l.failTitleSteamNotOwned,
          explanation: (l) => l.failExplanationSteamNotOwned,
          extraHints: [(l) => l.failHintSignInOwner],
        );
      case 22:
        return _Category(
          title: (l) => l.failTitleSteamCheckFailed,
          explanation: (l) => l.failExplanationSteamCheckFailed,
          extraHints: [
            (l) => l.failHintRestartSteam,
            (l) => l.failHintPersistShare,
          ],
        );
      case 64:
        return _Category(
          title: (l) => l.failTitleInvalidArgs,
          explanation: (l) => l.failExplanationInvalidArgs,
          extraHints: [(l) => l.failHintPanicShare],
        );
      default:
        return _Category(
          title: (l) => l.failTitleExitedUnexpectedly,
          explanation: (l) => l.failExplanationExitedUnexpectedly,
          extraHints: [
            (l) => l.failHintCheckLogViewer,
            (l) => l.failHintShareReport,
          ],
        );
    }
  }
}

typedef _L10nString = String Function(AppLocalizations l);

class _Category {
  final _L10nString title;
  final _L10nString explanation;
  final List<_L10nString> extraHints;
  const _Category({
    required this.title,
    required this.explanation,
    required this.extraHints,
  });
}

class LaunchFailureParser {
  static final _errorHeader = RegExp(r'^ERROR\[(\d+)\]\s+(.+)$', multiLine: true);
  static final _panicHeader = RegExp(r'^PANIC\[(\d+)\]\s+(.+)$', multiLine: true);
  static final _phaseLine = RegExp(r'^\s*Phase:\s*(.+)$', multiLine: true);
  static final _osErrorLine = RegExp(r'^\s*OS error:\s*(.+)$', multiLine: true);
  static final _causeLine = RegExp(r'^\s*Likely cause:\s*(.+)$', multiLine: true);
  static final _fixLine = RegExp(r'^\s*Try:\s*(.+)$', multiLine: true);

  static LaunchFailure? parse(
    String combinedOutput, {
    String? capturedLogPath,
  }) {
    if (combinedOutput.trim().isEmpty) return null;
    final panic = _panicHeader.firstMatch(combinedOutput);
    if (panic != null) {
      return LaunchFailure(
        code: int.tryParse(panic.group(1) ?? ''),
        panic: true,
        headline: panic.group(2)?.trim() ?? 'NAMS panicked',
        phase: _phaseLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        osError: _osErrorLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        cause: _causeLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        fix: _fixLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        rawOutput: combinedOutput,
        capturedLogPath: capturedLogPath,
      );
    }
    final err = _errorHeader.firstMatch(combinedOutput);
    if (err != null) {
      return LaunchFailure(
        code: int.tryParse(err.group(1) ?? ''),
        panic: false,
        headline: err.group(2)?.trim() ?? 'NAMS reported a failure',
        phase: _phaseLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        osError: _osErrorLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        cause: _causeLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        fix: _fixLine.firstMatch(combinedOutput)?.group(1)?.trim(),
        rawOutput: combinedOutput,
        capturedLogPath: capturedLogPath,
      );
    }
    return null;
  }
}
