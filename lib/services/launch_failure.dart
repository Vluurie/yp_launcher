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

  String friendlyTitle() {
    if (panic) return 'NAMS crashed';
    final c = code;
    if (c == null) return 'Game launch failed';
    return _categoryFor(c).title;
  }

  String friendlyExplanation() {
    if (panic) {
      return 'NAMS hit an unrecoverable error before the game could start. '
          'This is almost always a bug — please share the report below with the maintainer.';
    }
    final c = code;
    if (c == null) return 'The game did not start within 60 seconds and no error was reported.';
    return _categoryFor(c).explanation;
  }

  List<String> hints() {
    if (panic) {
      return const [
        'Copy the full report below and send it to the maintainer.',
        'Try once more after rebooting — sometimes a stale handle clears itself.',
      ];
    }
    final c = code;
    if (c == null) {
      return const [
        'NAMS seems to have spawned but the game window never appeared.',
        'Check Task Manager — is NieRAutomata.exe running but invisible? Kill it and retry.',
        'Make sure no other launcher / DRM tool is holding the exe (FAR, Special K, etc).',
      ];
    }
    final cat = _categoryFor(c);
    return [if (fix != null && fix!.isNotEmpty) fix!, ...cat.extraHints];
  }

  static _Category _categoryFor(int code) {
    switch (code) {
      case 1:
        return const _Category(
          title: 'NAMS reported a failure',
          explanation:
              'A NAMS check failed before the game could run. See the report below for details.',
          extraHints: [
            'Copy the full report below and share it for diagnosis.',
          ],
        );
      case 2:
        return const _Category(
          title: 'NieR:Automata install not found',
          explanation:
              'NAMS could not resolve your NieR:Automata install. The saved path may be wrong, or Steam autodetect failed.',
          extraHints: [
            'Re-pick your game directory in the launcher to refresh the saved path.',
            'Verify game files in Steam (Library → NieR:Automata → Properties → Local Files → Verify).',
          ],
        );
      case 3:
        return const _Category(
          title: 'Could not create a needed folder',
          explanation:
              'NAMS could not create a directory next to NAMS.exe. The install folder may be read-only.',
          extraHints: [
            'Make sure the launcher install folder (where NAMS.exe lives) is writable.',
            'If it is in Program Files or OneDrive-synced, move the launcher to a normal folder or right-click → "Always keep on this device".',
          ],
        );
      case 4:
        return const _Category(
          title: 'Runtime preparation failed',
          explanation:
              'NAMS could not prepare its runtime (game.bin / steam_api64.dll). This is usually a writability or antivirus problem.',
          extraHints: [
            'Add the launcher install folder AND your game folder to your antivirus exclusions, then retry.',
            'Make sure the install folder is writable so the runtime cache can be built.',
          ],
        );
      case 5:
      case 6:
      case 7:
      case 8:
      case 9:
        return const _Category(
          title: 'NAMS host failure',
          explanation:
              'NAMS could not load and start the game host (game.bin). This is usually an environment or corruption issue.',
          extraHints: [
            'Reboot and try again — sometimes a stale handle clears itself.',
            'Add the launcher install folder AND your game folder to your antivirus exclusions.',
            'If it persists, copy the full report and send it to the maintainer.',
          ],
        );
      case 20:
        return const _Category(
          title: 'Steam not running / not logged in',
          explanation:
              'NAMS could not reach a logged-in Steam session. Steam must be running and signed in.',
          extraHints: [
            'Start Steam and sign in, then launch again.',
          ],
        );
      case 21:
        return const _Category(
          title: 'Steam account does not own NieR:Automata',
          explanation:
              'The signed-in Steam account does not own NieR:Automata.',
          extraHints: [
            'Sign into the Steam account that owns NieR:Automata.',
          ],
        );
      case 22:
        return const _Category(
          title: 'Steam check failed',
          explanation:
              'NAMS hit an internal error while verifying Steam ownership.',
          extraHints: [
            'Restart Steam and the launcher, then try again.',
            'If it persists, copy the full report and send it to the maintainer.',
          ],
        );
      case 64:
        return const _Category(
          title: 'Invalid launch arguments',
          explanation:
              'The launcher passed arguments NAMS could not parse. This is a launcher bug.',
          extraHints: [
            'Copy the full report below and send it to the maintainer.',
          ],
        );
      default:
        return const _Category(
          title: 'Game exited unexpectedly',
          explanation:
              'NAMS started the game but it exited with a non-zero code. The game may have crashed.',
          extraHints: [
            'Check the in-app log viewer (nams.log) for the crash details.',
            'Copy the full report below and share it for diagnosis.',
          ],
        );
    }
  }
}

class _Category {
  final String title;
  final String explanation;
  final List<String> extraHints;
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
