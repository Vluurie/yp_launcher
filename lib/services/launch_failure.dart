/// Structured failure information parsed from launch_nier.exe output.
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

  /// User-facing 1-line summary.
  String friendlyTitle() {
    if (panic) return 'launch_nier crashed (panic)';
    final c = code;
    if (c == null) return 'Game launch failed';
    return _categoryFor(c).title;
  }

  String friendlyExplanation() {
    if (panic) {
      return 'launch_nier hit an unrecoverable error before the game could start. '
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
        'launch_nier seems to have spawned but the game window never appeared.',
        'Check Task Manager — is NieRAutomata.exe running but invisible? Kill it and retry.',
        'Make sure no other launcher / DRM tool is holding the exe (FAR, Special K, etc).',
      ];
    }
    final cat = _categoryFor(c);
    return [if (fix != null && fix!.isNotEmpty) fix!, ...cat.extraHints];
  }

  static _Category _categoryFor(int code) {
    switch (code) {
      case 10:
        return const _Category(
          title: 'Bad arguments / working directory',
          explanation:
              'launch_nier could not figure out where to run from. The launcher passed paths it cannot use.',
          extraHints: [
            'Re-pick your game directory in the launcher to refresh the saved path.',
          ],
        );
      case 11:
        return const _Category(
          title: 'Could not create the nierdecrypt working folder',
          explanation:
              'launch_nier needs to create %APPDATA%\\YoRHaProtocolLauncher\\nierdecrypt\\ but Windows refused.',
          extraHints: [
            'Make sure %APPDATA%\\YoRHaProtocolLauncher\\ is writable.',
            'If %APPDATA% is OneDrive-synced, right-click the folder → "Always keep on this device".',
          ],
        );
      case 12:
        return const _Category(
          title: 'modloader.dll missing or unreadable',
          explanation:
              'modloader.dll is gone or zero bytes. Antivirus probably quarantined or wiped it.',
          extraHints: [
            'Open %APPDATA%\\YoRHaProtocolLauncher\\ and verify modloader.dll exists and is several hundred KB.',
            'Add %APPDATA%\\YoRHaProtocolLauncher\\ to your antivirus exclusions, then restart the launcher.',
          ],
        );
      case 13:
        return const _Category(
          title: 'A plugin DLL is missing',
          explanation:
              'One of the --mod-dll files (yorha_protocol or a user plugin) is gone or empty.',
          extraHints: [
            'Open the Plugins tab and see if anything shows as missing.',
            'Restart the launcher — it re-extracts yorha_protocol.dll on startup.',
          ],
        );
      case 20:
        return const _Category(
          title: 'NieRAutomata.exe not found / unreadable',
          explanation:
              'The game executable is missing, zero-byte, or quarantined.',
          extraHints: [
            'Verify game files in Steam (Library → NieR:Automata → Properties → Local Files → Verify).',
            'Add your game directory to your antivirus exclusions.',
          ],
        );
      case 21:
        return const _Category(
          title: 'Decryption step failed',
          explanation:
              'launch_nier could not decrypt the Steam exe. .NET runtime may be missing, or AV blocked the helper.',
          extraHints: [
            'Install the .NET Desktop Runtime (https://dotnet.microsoft.com/download).',
            'Add %APPDATA%\\YoRHaProtocolLauncher\\nierdecrypt\\ to your AV exclusions.',
          ],
        );
      case 30:
        return const _Category(
          title: 'Windows refused to start the game process',
          explanation:
              'CreateProcessW failed. The most common reason is that an antivirus / EDR product blocked the spawn.',
          extraHints: [
            'Try double-clicking NieRAutomata.exe directly. If that also fails, the exe itself is the problem.',
            'Add the launcher folder AND the game folder to your antivirus exclusions.',
            'Make sure NieRAutomata.exe is not already running (Task Manager).',
          ],
        );
      case 31:
        return const _Category(
          title: 'Modloader injection blocked',
          explanation:
              'Windows let the game spawn but blocked launch_nier from injecting modloader.dll. AV / anti-cheat behavior.',
          extraHints: [
            'Disable any anti-cheat or "process protection" feature in your AV temporarily.',
            'Whitelist launch_nier.exe and modloader.dll explicitly.',
          ],
        );
      case 32:
        return const _Category(
          title: 'modloader.dll loaded but reported failure',
          explanation:
              'The modloader started but reported an error during init. See the report below for details.',
          extraHints: [
            'Check the modloader log at %APPDATA%\\YoRHaProtocolLauncher\\nierdecrypt\\logs\\modloader.log',
          ],
        );
      case 40:
        return const _Category(
          title: 'Could not write steam_appid.txt',
          explanation:
              'The nierdecrypt directory rejected a write. AV or read-only attribute.',
          extraHints: [
            'Make sure %APPDATA%\\YoRHaProtocolLauncher\\nierdecrypt\\ is writable.',
            'Add it to your AV exclusions.',
          ],
        );
      case 90:
        return const _Category(
          title: 'launch_nier crashed (panic)',
          explanation:
              'An internal error in launch_nier itself. Please share the report.',
          extraHints: [
            'Copy the full report and send it to the maintainer.',
          ],
        );
      case 99:
      default:
        return const _Category(
          title: 'Unknown launch failure',
          explanation:
              'launch_nier exited with an unrecognised error.',
          extraHints: [
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

/// Parses the structured ERROR[N]/PANIC[N] block emitted by our launch_nier
/// build. Falls back to "no error block found" gracefully.
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
        headline: panic.group(2)?.trim() ?? 'launch_nier panicked',
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
        headline: err.group(2)?.trim() ?? 'launch_nier reported a failure',
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
