// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'YoRHa Protocol Launcher';

  @override
  String get tooltipLauncherSource => 'Launcher Source Code';

  @override
  String get tooltipNamsSource => 'NAMS Project Source';

  @override
  String get tooltipGuide => 'Guide';

  @override
  String get tooltipDiscord => 'Discord';

  @override
  String get themeToggleToNier => 'Switch to NieR theme';

  @override
  String get themeToggleToDark => 'Switch to dark theme';

  @override
  String get tooltipLanguage => 'Language';

  @override
  String get languageSupportNotice =>
      'Translations are community-made or auto-generated and may be inaccurate. The maintainers speak English only - please ask for help in English.';

  @override
  String get tooltipCopyCommand =>
      'Copy the NAMS command to clipboard so you can paste it into a terminal and start the game manually.';

  @override
  String get notificationCommandCopied =>
      'Launch command copied - paste it into a terminal to start the game manually.';

  @override
  String get notificationCommandNotReady =>
      'Could not build launch command - launcher binaries are not ready yet.';

  @override
  String get textureAutoRecommended => 'Auto (recommended)';

  @override
  String get detectionDlcPresent => 'DLC: present';

  @override
  String get detectionDlcNotDetected => 'DLC: not detected';

  @override
  String get detectionDlcPresentTooltip =>
      'DLC data100.cpk found. Mods that ship DLC-only outfit files (pl000d, pl010d, pl020d) will install as-is.';

  @override
  String get detectionDlcNotDetectedTooltip =>
      'No DLC detected. Mods that ship DLC-only outfit files (pl000d, pl010d, pl020d) will be installed under the non-DLC names (pl0000, pl0100, pl0200) so they show up in-game.';

  @override
  String get detectionExeWolfLimitBreak => 'EXE: Wolf Limit Break';

  @override
  String get detectionExeOriginal => 'EXE: Original';

  @override
  String get detectionExeMissing => 'EXE: missing';

  @override
  String get detectionExeUnrecognised => 'EXE: unrecognised';

  @override
  String get detectionExeUnrecognisedTooltip =>
      'NieRAutomata.exe is present but its hash is not in our known list. NAMS will still run; this is just a heads-up that we have not seen this exact build.';

  @override
  String get detectionExeWolfLimitBreakTooltip =>
      'Wolf Limit Break NieRAutomata.exe detected. NAMS does not need this patch and was never tested against it. The game may still launch, but performance issues, memory crashes, or mod incompatibilities are possible. For full support, restore the original Steam executable (verify game files in Steam).';

  @override
  String get detectionExeLegacyWin7 => 'EXE: Windows 7 build';

  @override
  String get detectionExeLegacyWin7Tooltip =>
      'This is the legacy Windows 7/8 NieRAutomata.exe. NAMS needs the Windows 10/11 Steam build and cannot launch this one. It is common on Proton/Linux, where Steam sometimes downloads the Windows 7 executable.\n\nHow to fix it:\n1. Delete every .exe in your NieRAutomata game folder.\n2. In Steam, set Proton to \'Proton Experimental\' (right click the game > Properties > Compatibility).\n3. In Steam, right click the game > Properties > Installed Files > Verify integrity of game files.\n4. Launch the game once through Steam so it keeps the correct executable, then use the launcher again.';

  @override
  String get launchOptionsTitle => 'LAUNCH OPTIONS';

  @override
  String get launchWrapperTitle => 'LAUNCH WRAPPER (LINUX)';

  @override
  String get launchWrapperDesc =>
      'Prepend a command in front of the game launch, e.g. gamescope or mangohud. The launcher runs the game through Proton, so Steam launch options do not apply here. Leave empty to launch normally. Applies on the next launch.';

  @override
  String get launchWrapperHint => 'gamescope -w 2560 -h 1440 -f --';

  @override
  String get launchWrapperExample =>
      'Examples:\ngamescope -w 2560 -h 1440 -f --\nmangohud\ngamemoderun';

  @override
  String get tabLauncherSettings => 'Troubleshooting';

  @override
  String get troubleWrongExeTitle => 'Wrong game version';

  @override
  String get troubleWrongExeSummary =>
      'This is the legacy Windows 7/8 NieRAutomata.exe. NAMS cannot launch it.';

  @override
  String get troubleMissingFilesTitle => 'Missing launcher files';

  @override
  String troubleMissingFilesSummary(String files) {
    return 'Missing: $files';
  }

  @override
  String get troubleMissingFilesDesc =>
      'These files are part of the portable launcher but are gone. Antivirus (usually Windows Defender) most likely quarantined them. Exclude the launcher folder from your antivirus and restore the files, or re-download the launcher.';

  @override
  String get troubleRecentErrorsTitle => 'Recent NAMS errors';

  @override
  String get troubleFoldersTitle => 'Open folders';

  @override
  String get troubleFoldersDesc =>
      'Quick access to the folders a maintainer may ask you to open.';

  @override
  String get troubleOpenGame => 'Game folder';

  @override
  String get troubleOpenNamsConfig => 'NAMS config';

  @override
  String get troubleOpenLogs => 'Logs';

  @override
  String get troubleOpenBins => 'Launcher folder';

  @override
  String get troubleClearCacheTitle => 'Clear cache';

  @override
  String get troubleClearCacheDesc =>
      'Delete cached state that can go stale: log files, the detection cache, the mod\'s NAMS settings.json, and the runtime extract stamp. Your mods and game files are not touched.';

  @override
  String get troubleClearCacheButton => 'Clear cache';

  @override
  String get troubleClearCacheConfirm =>
      'This deletes logs, the detection cache, the mod\'s in-game settings.json, and forces a runtime re-extract on next start. Mods and game files stay untouched. Continue?';

  @override
  String troubleClearCacheDone(int count) {
    return 'Cache cleared ($count log files removed).';
  }

  @override
  String get verifyInstallTitle => 'INSTALL DIAGNOSTICS';

  @override
  String get verifyInstallDesc =>
      'Run NAMS\'s built-in checks to diagnose why the game may not launch (wrong Windows build, missing Steam files, permissions).';

  @override
  String get verifyInstallButton => 'Verify installation';

  @override
  String get verifyInstallRunning => 'Checking...';

  @override
  String get verifyInstallOk => 'All checks passed.';

  @override
  String get verifyInstallFailed => 'Some checks failed. See details below.';

  @override
  String get verifyNoRuntime =>
      'Cannot verify: no Proton/Wine runtime found for this install.';

  @override
  String get verifySteamNotRunning =>
      'Cannot verify: Steam must be running and own the game.';

  @override
  String get verifyInstallError =>
      'Could not run the check. Make sure a game folder is selected.';

  @override
  String get verifyInstallNoGameDir => 'Select your game folder first.';

  @override
  String get verifyCheckSteamInstall => 'Steam installation';

  @override
  String get verifyCheckNierExe => 'Game executable';

  @override
  String get verifyCheckSteamApi64 => 'Steam API library';

  @override
  String get verifyCheckRuntimeWritable => 'Runtime writable';

  @override
  String get verifyCheckRuntimeCached => 'Runtime library cached';

  @override
  String get launchOptionMinimizeOnLaunch => 'Minimize launcher while playing';

  @override
  String get launchOptionPreferDedicatedGpu => 'Prefer dedicated GPU';

  @override
  String get launchOptionPreferDedicatedGpuTooltip =>
      'Tells the system to run the game on the dedicated graphics card instead of the power-saving one. Only matters on PCs with two GPUs (e.g. gaming laptops).';

  @override
  String get failTitlePanic => 'NAMS crashed';

  @override
  String get failTitleUnknown => 'Game launch failed';

  @override
  String get failExplanationPanic =>
      'NAMS hit an unrecoverable error before the game could start. This is almost always a bug — please share the report below with the maintainer.';

  @override
  String get failHintPanicShare =>
      'Copy the full report below and send it to the maintainer.';

  @override
  String get failHintPanicReboot =>
      'Try once more after rebooting — sometimes a stale handle clears itself.';

  @override
  String get failTitleNamsFailure => 'NAMS reported a failure';

  @override
  String get failExplanationNamsFailure =>
      'A NAMS check failed before the game could run. See the report below for details.';

  @override
  String get failHintShareReport =>
      'Copy the full report below and share it for diagnosis.';

  @override
  String get failTitleInstallNotFound => 'NieR:Automata install not found';

  @override
  String get failExplanationInstallNotFound =>
      'NAMS could not resolve your NieR:Automata install. The saved path may be wrong, or Steam autodetect failed.';

  @override
  String get failHintRepickDirectory =>
      'Re-pick your game directory in the launcher to refresh the saved path.';

  @override
  String get failHintVerifyFiles =>
      'Verify game files in Steam (Library → NieR:Automata → Properties → Local Files → Verify).';

  @override
  String get failTitleFolderCreate => 'Could not create a needed folder';

  @override
  String get failExplanationFolderCreate =>
      'NAMS could not create a directory next to NAMS.exe. The install folder may be read-only.';

  @override
  String get failHintWritableFolder =>
      'Make sure the launcher install folder (where NAMS.exe lives) is writable.';

  @override
  String get failHintProgramFiles =>
      'If it is in Program Files or OneDrive-synced, move the launcher to a normal folder or right-click → \"Always keep on this device\".';

  @override
  String get failTitleRuntimePrep => 'Runtime preparation failed';

  @override
  String get failExplanationRuntimePrep =>
      'NAMS could not prepare its runtime (game.bin / steam_api64.dll). This is usually a writability or antivirus problem.';

  @override
  String get failHintAntivirusExclusions =>
      'Add the launcher install folder AND your game folder to your antivirus exclusions, then retry.';

  @override
  String get failHintWritableCache =>
      'Make sure the install folder is writable so the runtime cache can be built.';

  @override
  String get failTitleHostFailure => 'NAMS host failure';

  @override
  String get failExplanationHostFailure =>
      'NAMS could not load and start the game host (game.bin). This is usually an environment or corruption issue.';

  @override
  String get failHintReboot =>
      'Reboot and try again — sometimes a stale handle clears itself.';

  @override
  String get failHintPersistShare =>
      'If it persists, copy the full report and send it to the maintainer.';

  @override
  String get failTitleSteamNotRunning => 'Steam not running / not logged in';

  @override
  String get failExplanationSteamNotRunning =>
      'NAMS could not reach a logged-in Steam session. Steam must be running and signed in.';

  @override
  String get failHintStartSteam =>
      'Start Steam and sign in, then launch again.';

  @override
  String get failTitleSteamNotOwned =>
      'Steam account does not own NieR:Automata';

  @override
  String get failExplanationSteamNotOwned =>
      'The signed-in Steam account does not own NieR:Automata.';

  @override
  String get failHintSignInOwner =>
      'Sign into the Steam account that owns NieR:Automata.';

  @override
  String get failTitleSteamCheckFailed => 'Steam check failed';

  @override
  String get failExplanationSteamCheckFailed =>
      'NAMS hit an internal error while verifying Steam ownership.';

  @override
  String get failHintRestartSteam =>
      'Restart Steam and the launcher, then try again.';

  @override
  String get failTitleInvalidArgs => 'Invalid launch arguments';

  @override
  String get failExplanationInvalidArgs =>
      'The launcher passed arguments NAMS could not parse. This is a launcher bug.';

  @override
  String get failTitleExitedUnexpectedly => 'Game exited unexpectedly';

  @override
  String get failExplanationExitedUnexpectedly =>
      'NAMS started the game but it exited with a non-zero code. The game may have crashed.';

  @override
  String get failHintCheckLogViewer =>
      'Check the in-app log viewer (nams.log) for the crash details.';

  @override
  String get failHeadlinePanicked => 'NAMS panicked';

  @override
  String get failSectionWhatHappened => 'What happened';

  @override
  String get failSectionReportedByNams => 'Reported by NAMS';

  @override
  String get failSectionTryThis => 'Try this';

  @override
  String get failSectionDiagnosticDetail => 'Diagnostic detail';

  @override
  String get failSectionLaunchManually => 'Launch manually from a terminal';

  @override
  String get failSectionRawOutput => 'Raw output';

  @override
  String get failManualCommandHint =>
      'If the launcher UI keeps failing for you, paste this into a terminal to start the game manually. It is the exact same command the Play button runs.';

  @override
  String get failDetailOs => 'OS';

  @override
  String get failDetailCause => 'Cause';

  @override
  String get failDetailSuggested => 'Suggested';

  @override
  String get failActionCopyReport => 'Copy report';

  @override
  String get failActionOpenLogFile => 'Open log file';

  @override
  String get logDetailOs => 'OS';

  @override
  String get logDetailLocale => 'Locale';

  @override
  String get logNoModsInstalled => 'No mods installed.';

  @override
  String get logSectionSystem => 'System';

  @override
  String get logSectionModsNams => 'Mods (NAMS)';

  @override
  String get logSectionCutscenes => 'Cutscenes';

  @override
  String get logSectionTextures => 'Textures';

  @override
  String get tooltipOpenInModManager => 'Open in Mod Manager';

  @override
  String get tooltipOpenInCutscenesTab => 'Open in Cutscenes tab';

  @override
  String tooltipOpenInTexturesTab(String name) {
    return '$name\n\nOpen in Textures tab';
  }

  @override
  String get actionCancel => 'Cancel';

  @override
  String get tooltipMinimize => 'Minimize';

  @override
  String get tooltipMaximize => 'Maximize';

  @override
  String get tooltipRestore => 'Restore';

  @override
  String get tooltipClose => 'Close';

  @override
  String get infoText => 'Select your game, press play - done.';

  @override
  String get helpPrefix => 'Launcher not working? Try ';

  @override
  String get helpNaoLauncher => 'NAO Launcher';

  @override
  String get helpOr => ' or ';

  @override
  String get helpCommandLine => 'Command Line';

  @override
  String get helpJoinDiscord => '. Join our ';

  @override
  String get helpDiscord => 'Discord';

  @override
  String get helpSuffix => ' for help.';

  @override
  String get noFileSelected => 'No file selected';

  @override
  String get selectButton => 'SELECT';

  @override
  String get filePickerTitle => 'Select Game Executable';

  @override
  String get playButton => 'PLAY';

  @override
  String get stopButton => 'STOP';

  @override
  String get statusReady => 'Ready to launch!';

  @override
  String get statusSelectGame => 'Select your game executable to get started.';

  @override
  String get statusPreparing => 'Preparing launcher files...';

  @override
  String get statusLaunching => 'Launching game...';

  @override
  String get statusRunning => 'Game is running.';

  @override
  String get statusStopped => 'Game stopped.';

  @override
  String get statusStopping => 'Stopping game...';

  @override
  String get errorInvalidExe =>
      'Looks like this is not NieR:Automata. Make sure it\'s the latest Steam version.';

  @override
  String get errorFileNotExist => 'Selected file does not exist';

  @override
  String get errorSelectFailed => 'Failed to select file';

  @override
  String get errorStartFailed => 'Failed to start the game.';

  @override
  String get errorStopFailed => 'Failed to stop the game.';

  @override
  String errorFilesQuarantined(String files) {
    return 'Missing launcher files: $files. This is often caused by antivirus software. We load mods at runtime - standard for game modding but it can trigger false positives. Restore the files from quarantine or re-download the launcher, then add an exclusion for the launcher install folder (the folder containing NAMS.exe).';
  }

  @override
  String get notifyFilesQuarantined =>
      'Missing launcher files detected. This is often caused by antivirus software. We load mods at runtime, which is normal for game modding but may trigger false positives. Restore the files from quarantine or re-download the launcher, then add an exclusion for the launcher install folder (the folder containing NAMS.exe).';

  @override
  String get featureReshade =>
      'ReShade - Already installed? YP picks it up automatically.';

  @override
  String get featureTextures =>
      'HD Textures - Drop textures into nams/inject/textures/ or they get picked up from SK_Res/.';

  @override
  String get featureLodMod =>
      'LOD Mod Ext - Built-in visual tweaks like shadows, details and pop-in. Off by default.';

  @override
  String get tooltipEditConfigs =>
      'Change visual settings without editing files';

  @override
  String get tooltipOpenLogs => 'Open the logs folder in Explorer';

  @override
  String get tooltipCreateShortcut =>
      'Create a desktop shortcut to launch with YoRHa Protocol';

  @override
  String get shortcutName => 'NieR Automata (YoRHa Protocol)';

  @override
  String get shortcutDescription => 'Launch NieR:Automata with YoRHa Protocol';

  @override
  String get notifyShortcutCreated => 'Desktop shortcut created!';

  @override
  String get notifyShortcutFailed => 'Failed to create desktop shortcut.';

  @override
  String get notifyLodModMigrated =>
      'Found your old LodMod.ini settings - imported into lodmod.toml and enabled LodMod Ext.';

  @override
  String get notifyReShadeDetected =>
      'ReShade detected - disabled by default. NAMS already ships a patched native depth-of-field, so ReShade is optional. Re-enable it any time in the NAMS config tab (Disable ReShade Loading → off).';

  @override
  String get notifyNaiomMigrated =>
      'Found your old NAIOM settings - imported into nams.toml. Check the NAIOM tab. You can remove the old NAIOM files (dinput8.dll, NAIOM.ini) from the game folder.';

  @override
  String notifyNaiomSkipped(String entries) {
    return 'Some NAIOM bindings use keys NAMS does not support and were not imported: $entries. Rebind them in the NAIOM tab.';
  }

  @override
  String notifyPlatformUnsupported(String platform) {
    return 'No Windows compatibility layer found on $platform, so the game cannot be started from here. Mods, textures and configs all still work. Install CrossOver and put NieR:Automata in a bottle to enable launching.';
  }

  @override
  String get notifyReShadeIncompatible =>
      'ReShade with addon/ImGui support detected - incompatible. Use standard ReShade without addon support.';

  @override
  String notifyTexturesDetected(String folder) {
    return 'HD textures found in $folder - will load on launch.';
  }

  @override
  String get errorAppDataNotFound => 'APPDATA environment variable not found';

  @override
  String get errorWinePrefixNotFound =>
      'Wine prefix not found. Please set WINEPREFIX environment variable.';

  @override
  String get errorHomeNotFound => 'HOME environment variable not found';

  @override
  String get errorNoWineUser => 'No user directory found in Wine prefix';

  @override
  String errorWineUsersNotFound(String prefix) {
    return 'Wine drive_c/users directory not found in $prefix';
  }

  @override
  String errorPlatformNotSupported(String os) {
    return 'Platform $os is not supported';
  }

  @override
  String errorExeNotFound(String dir) {
    return 'NieRAutomata.exe not found in $dir';
  }

  @override
  String get errorDirNotWritable => 'Launcher folder is read-only';

  @override
  String errorDirNotWritableBody(String dir) {
    return 'The YP Launcher folder cannot be written to:\n$dir\n\nNAMS writes its runtime cache, plugins and logs next to NAMS.exe, so the launcher folder must be writable.\n\nHow to fix it:\n1. Close the launcher.\n2. Move the whole extracted YP Launcher folder out of Program Files (or any protected location) into a normal folder you own - for example Desktop, Documents, or D:\\Games\\YP Launcher.\n3. Start the launcher again from the new location.\n\nAlternative: right-click the launcher .exe and choose \"Run as administrator\" to allow writing in the current location.';
  }

  @override
  String get errorGameDirNotWritable => 'Game folder is read-only';

  @override
  String errorGameDirNotWritableBody(String gameDir, String namsDir) {
    return 'The game folder cannot be written to:\n$gameDir\n\nNAMS writes mods and configs into:\n$namsDir\nbut it cannot create files there. NieR is probably installed under C:\\Program Files (x86)\\Steam, which Windows protects.\n\nHow to fix it (recommended - move the Steam library off Program Files):\n1. Open Steam > Settings > Storage.\n2. Click the drive dropdown > \"Add Drive\" and pick a folder on another drive (for example D:\\SteamLibrary).\n3. Go to your Library, right-click NieR:Automata > Properties > Installed Files > \"Move install folder\", and move it to the new library.\n4. Re-select the game in this launcher and press Play again.\n\nQuick alternative: right-click the launcher .exe and choose \"Run as administrator\" so it can write into Program Files. Moving the library is the cleaner long-term fix.';
  }

  @override
  String get errorNoCompatLayer => 'CrossOver not found';

  @override
  String get errorNoCompatLayerBody =>
      'Running NieR:Automata on this system needs CrossOver, which runs Windows programs on macOS. It was not found in /Applications.\n\nWithout it the launcher can still manage mods, textures and configs - only starting the game is unavailable.\n\nHow to fix it:\n1. Install CrossOver from codeweavers.com.\n2. Install Steam and NieR:Automata inside a CrossOver bottle.\n3. Select NieRAutomata.exe from inside that bottle in this launcher.';

  @override
  String get errorNoCompatLayerLinux => 'No Proton or Wine found';

  @override
  String get errorNoCompatLayerLinuxBody =>
      'Running NieR:Automata on Linux needs Proton (recommended) or Wine, and none was found.\n\nWithout it the launcher can still manage mods, textures and configs - only starting the game is unavailable.\n\nHow to fix it:\n1. In Steam, install a Proton build (Proton Experimental works well). If it is on another drive, the launcher now checks every Steam library.\n2. Make sure you selected NieRAutomata.exe from inside your Steam library (a path containing steamapps/common).\n3. Or set YP_PROTON_PATH to your proton binary before starting the launcher, e.g. YP_PROTON_PATH=\"\$HOME/.steam/steam/steamapps/common/Proton - Experimental/proton\".';

  @override
  String get errorProtonMissing => 'Proton not found';

  @override
  String errorProtonMissingBody(String path) {
    return 'The configured Proton runtime is missing at:\n$path\n\nReinstall Proton through Steam, or set YP_PROTON_PATH to a valid proton binary before starting the launcher.';
  }

  @override
  String get errorNoZDrive => 'The Wine prefix has no Z: drive';

  @override
  String errorNoZDriveBody(String prefix) {
    return 'Wine maps Z: to the host filesystem, which is how the launcher hands NAMS.exe to the game. This prefix is missing dosdevices/z::\n$prefix\n\nThis is a CrossOver default, so the bottle was probably modified. Creating a new bottle and reinstalling the game there is the quickest fix.';
  }

  @override
  String get errorExeOutsidePrefix => 'That executable is not inside a bottle';

  @override
  String errorExeOutsidePrefixBody(String exeName, String path) {
    return 'The launcher starts the game through CrossOver, so $exeName has to live inside a CrossOver bottle:\n$path\n\nInstall the game into a bottle, then select the executable from there.';
  }

  @override
  String get headerNams => 'NAMS';

  @override
  String get headerLodMod => 'LOD MOD EXT';

  @override
  String get headerTextures => 'TEXTURES';

  @override
  String get headerYorhaProtocol => 'YORHA PROTOCOL';

  @override
  String get headerNaiom => 'NAIOM';

  @override
  String get headerCutscenes => 'CUTSCENES';

  @override
  String get tooltipEditsNamsToml => 'Edits nams/nams.toml';

  @override
  String get tooltipEditsLodmodToml => 'Edits nams/lodmod.toml';

  @override
  String get tooltipEditsTextureInjectionToml =>
      'Edits nams/texture_injection.toml';

  @override
  String get tooltipEditsSettingsJson =>
      'Edits nams\\_internal\\cache\\settings.json';

  @override
  String get tooltipEditsNaiom =>
      'Edits the [mouse] settings in nams/nams.toml. Most settings apply after saving; a few need a game restart.';

  @override
  String get tooltipCutscenesLocation =>
      'Mods: nams/cutscenes/<mod_name>/movie/*.usm';

  @override
  String get cardGeneral => 'GENERAL';

  @override
  String get cardLoading => 'LOADING';

  @override
  String get cardHeapOverrides => 'HEAP OVERRIDES';

  @override
  String get cardPerformance => 'PERFORMANCE';

  @override
  String get performanceCardNote =>
      'The first two only help when the CPU is the bottleneck. If your CPU is slower than your GPU, both together can bring up to 20% more performance; on a GPU-bound system they change nothing. The third one is about hitches while the map loads, not about frame rate.';

  @override
  String get cardLevelOfDetail => 'LEVEL OF DETAIL';

  @override
  String get cardAmbientOcclusion => 'AMBIENT OCCLUSION';

  @override
  String get cardPostProcessing => 'POST-PROCESSING';

  @override
  String get cardShadows => 'SHADOWS';

  @override
  String get cardGlobalIllumination => 'GLOBAL ILLUMINATION';

  @override
  String get cardPreloading => 'PRELOADING';

  @override
  String get cardTextureConfig => 'CONFIG';

  @override
  String get cardInstallTextures => 'INSTALL TEXTURES';

  @override
  String get cardHowItWorks => 'HOW IT WORKS';

  @override
  String get cardKeybinds => 'KEYBINDS';

  @override
  String get cardWorkspace => 'WORKSPACE';

  @override
  String get cardCheats => 'CHEATS';

  @override
  String get cardRandomizer => 'RANDOMIZER';

  @override
  String get cardThirdPersonCamera => 'THIRD-PERSON CAMERA';

  @override
  String get cardPodAiming => 'POD / AIMING';

  @override
  String get cardMisc => 'MISC';

  @override
  String get cardMovementBindings => 'MOVEMENT BINDINGS';

  @override
  String get cardCombatBindings => 'COMBAT BINDINGS';

  @override
  String get cardNonStandardBindings => 'NON-STANDARD BINDINGS';

  @override
  String get cardMenuNavigation => 'MENU NAVIGATION';

  @override
  String get cardStructure => 'STRUCTURE';

  @override
  String get buttonSave => 'SAVE';

  @override
  String get buttonDiscard => 'DISCARD';

  @override
  String get buttonCancel => 'CANCEL';

  @override
  String get buttonInstall => 'INSTALL';

  @override
  String get buttonDelete => 'DELETE';

  @override
  String get buttonYes => 'Yes';

  @override
  String get buttonNo => 'No';

  @override
  String get buttonContinue => 'Continue';

  @override
  String get buttonBack => 'Back';

  @override
  String get buttonGetStarted => 'Get Started';

  @override
  String get buttonAutoDetect => 'Auto-detect';

  @override
  String get buttonSelectManually => 'Select manually';

  @override
  String get buttonGoToLauncher => 'Go to launcher';

  @override
  String get unsavedChangesTitle => 'Unsaved changes';

  @override
  String get unsavedChangesMessage => 'You have unsaved changes. Discard them?';

  @override
  String get stay => 'STAY';

  @override
  String get discard => 'DISCARD';

  @override
  String get enterValidNumber => 'Enter a valid number';

  @override
  String get pressKey => 'Press...';

  @override
  String get tabLauncher => 'Launcher';

  @override
  String get tabYorhaProtocol => 'YP Devkit';

  @override
  String get tabNams => 'NAMS';

  @override
  String get tabLodMod => 'LOD Mod Ext';

  @override
  String get tabNaiom => 'NAIOM';

  @override
  String get tabTextures => 'Textures';

  @override
  String get tabMods => 'Mod Manager';

  @override
  String get tabCutscenes => 'Cutscenes';

  @override
  String get tabThirdParty => 'Third Party';

  @override
  String get tabDocs => 'Create Mods';

  @override
  String get tabTools => 'Tools';

  @override
  String get toolsIconsTitle => 'ICONS';

  @override
  String get toolsIconsDesc =>
      'Every misctex icon your mods ship. Replace one to swap the image or change its size - the game reads it by name, so the name has to stay.';

  @override
  String get toolsIconsHotReload =>
      'No restart needed: replace an icon while the game runs, then leave the inventory and open it again. The new image is already there.';

  @override
  String get toolsIconsEmpty =>
      'No icons yet. The item and weapon builders in Create Mods make one, or add one below.';

  @override
  String get toolsIconsAdd => 'Add an icon from an image';

  @override
  String get toolsIconsConvert => 'Convert an image';

  @override
  String get toolsIconsAdjust => 'Adjust an icon';

  @override
  String get toolsIconsNotMisctex =>
      'That is not an icon file. Pick a misctex_*.dat.';

  @override
  String get toolsIconsPick => 'Pick an existing icon';

  @override
  String get toolsIconsNewTitle => 'New icon';

  @override
  String get toolsIconsNewMod => 'Mod folder';

  @override
  String get toolsIconsNewNoMods => 'No mod folders yet. Create a mod first.';

  @override
  String get toolsIconsNewName => 'Texture name';

  @override
  String get toolsIconsNewNameHint => 'item_thumb_8500';

  @override
  String get toolsIconsNewHint => 'The game looks the icon up by this name.';

  @override
  String get toolsIconsNewCreate => 'Continue';

  @override
  String get toolsIconsNew => 'New icon';

  @override
  String get toolsIconsReplace => 'Replace';

  @override
  String get toolsIconsDelete => 'Delete';

  @override
  String toolsIconsDeleteConfirm(String name) {
    return 'Delete $name? The .dat and .dtt files are removed. Any item pointing at it falls back to the placeholder.';
  }

  @override
  String toolsIconsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count icons',
      one: '1 icon',
    );
    return '$_temp0';
  }

  @override
  String get toolsNeedsGameDir =>
      'Select your game folder in the launcher first.';

  @override
  String get docsSearchHint => 'Search the docs';

  @override
  String get docsNoResults => 'Nothing found.';

  @override
  String docsResultCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count results',
      one: '1 result',
    );
    return '$_temp0';
  }

  @override
  String get docsCategoryNative => 'Modding Guide';

  @override
  String get docsCategoryScripting => 'Scripting';

  @override
  String get docsCategoryResources => 'Reference Tables';

  @override
  String get docsCategoryAuthoring => 'Tool Authoring';

  @override
  String get docsOnThisPage => 'On this page';

  @override
  String get docsHome => 'Start here';

  @override
  String get docsHeroTitle => 'Make your own mod';

  @override
  String get docsHeroBody =>
      'New items, weapons, outfits, quests, whole scripted scenes. No coding needed to get started, and everything you need is already on this page.';

  @override
  String get docsHeroCta => 'Your first mod in 5 minutes';

  @override
  String get docsHeroTime => '5 min';

  @override
  String get docsPathBeginner => 'Never modded before';

  @override
  String get docsPathBeginnerBody =>
      'Start with a single new item, then learn how a mod folder is put together.';

  @override
  String get docsPathContent => 'Add things to the game';

  @override
  String get docsPathContentBody =>
      'Items, weapons, outfits, accessories, quests, mail, music and more.';

  @override
  String get docsPathScripting => 'Script the world';

  @override
  String get docsPathScriptingBody =>
      'Make things happen at the right moment: spawns, triggers, cameras, cutscenes.';

  @override
  String get docsSectionGuides => 'Guides';

  @override
  String get docsEnglishOnly =>
      'The documentation is only available in English.';

  @override
  String get docsBackToLauncher => 'Back to launcher';

  @override
  String get docsEdit => 'Edit this page';

  @override
  String docsLinkCopied(String url) {
    return 'Could not open the browser. Link copied: $url';
  }

  @override
  String get docsReload => 'Reload from disk';

  @override
  String get docsSaveFile => 'Save to file';

  @override
  String get docsPreview => 'Refresh preview';

  @override
  String get thumbnailTitle => 'Icon';

  @override
  String get thumbnailDrop => 'Drop an image here';

  @override
  String get thumbnailDropHint =>
      'PNG, JPEG, WebP, BMP or TGA. It gets converted for you.';

  @override
  String get thumbnailSize => 'Size';

  @override
  String get thumbnailSizeHint =>
      'Drag the corner to resize. DXT5, multiples of 4.';

  @override
  String get thumbnailSizeFixed =>
      'Fixed for this image type. Another size overflows its box in game.';

  @override
  String thumbnailScaled(int percent) {
    return 'Preview shown at $percent% - the icon is written at full size.';
  }

  @override
  String get thumbnailCreate => 'Create the icon';

  @override
  String get thumbnailUse => 'Use this image';

  @override
  String get thumbnailPending => 'Ready - written when you create the item.';

  @override
  String get thumbnailHotReload =>
      'Reopen the inventory in-game to see a replaced icon. No restart.';

  @override
  String get thumbnailOpen => 'Set an icon';

  @override
  String get thumbnailChange => 'Change icon';

  @override
  String get thumbnailReplace => 'Replace';

  @override
  String get thumbnailRemove => 'Remove';

  @override
  String get thumbnailDone => 'Icon written next to your item.';

  @override
  String get thumbnailWorking => 'Converting...';

  @override
  String get thumbnailNoItem => 'Create the item first, then give it an icon.';

  @override
  String get errTextureUnreadable =>
      'This icon could not be read. It may be damaged or in an unexpected format.';

  @override
  String get errImageUnreadable =>
      'That file could not be read as an image. Use a PNG, JPEG, WebP, BMP or TGA.';

  @override
  String get errImageSizeOutOfRange =>
      'The size has to be between 4 and 4096 pixels.';

  @override
  String get errImageSizeNotMultipleOfFour =>
      'Width and height have to be multiples of 4.';

  @override
  String get errArchiveEmpty => 'Nothing to pack.';

  @override
  String errArchiveExtensionTooLong(String name) {
    return 'The file extension of $name is longer than three characters.';
  }

  @override
  String get errWeaponNothingToConvert =>
      'That mod has no model or textures to convert. It needs a .dat with a wta and a .dtt with a wmb.';

  @override
  String get errWeaponClassMismatch =>
      'A weapon can only replace one of the same class. A spear cannot become a sword - the animations stay with the weapon it replaces.';

  @override
  String errFileReadFailed(String detail) {
    return 'Could not read the file: $detail';
  }

  @override
  String errFileWriteFailed(String detail) {
    return 'Could not write the file: $detail';
  }

  @override
  String errDirectoryCreateFailed(String detail) {
    return 'Could not create the folder: $detail';
  }

  @override
  String errUnknownCode(int code, String detail) {
    return 'Something went wrong (code $code). $detail';
  }

  @override
  String get itemBuilderTitle => 'Make one right now';

  @override
  String get itemBuilderBody =>
      'Fill this in and the launcher writes the file for you. You can look at what it produced below, and change it by hand later.';

  @override
  String get itemBuilderName => 'Name in the menu';

  @override
  String get itemBuilderNameHint => 'Rusty Bolt';

  @override
  String get itemBuilderDescription => 'Description';

  @override
  String get itemBuilderDescriptionHint => 'A bolt. It has seen better days.';

  @override
  String get itemBuilderModId => 'Mod folder';

  @override
  String get itemBuilderCarry => 'How many the player can carry';

  @override
  String get itemBuilderSellable => 'Can be sold';

  @override
  String get itemBuilderSellPrice => 'Sale price';

  @override
  String get itemBuilderAutoGive => 'Give it to the player automatically';

  @override
  String get itemBuilderAutoGiveHint =>
      'Otherwise you need a shop or a quest reward to hand it out.';

  @override
  String get itemBuilderIdLabel => 'Item id';

  @override
  String get itemBuilderIdHint =>
      'Picked for you. Every item needs one nobody else uses.';

  @override
  String get itemBuilderPreview => 'This is the file that gets written';

  @override
  String get itemBuilderCreate => 'Create the item';

  @override
  String itemBuilderCreated(String path) {
    return 'Written to $path';
  }

  @override
  String get itemBuilderNeedsName => 'Give it a name first.';

  @override
  String get itemBuilderNeedsGameDir =>
      'Select your game folder in the launcher first.';

  @override
  String get itemBuilderExists => 'That id is already taken. Pick another one.';

  @override
  String get itemBuilderOpenFolder => 'Show the file';

  @override
  String get firstModTitle => 'Build it while you read';

  @override
  String get firstModBody =>
      'Each step writes the real file into your game folder. At the end you have a working mod.';

  @override
  String get firstModStep1 => 'Create the mod folder';

  @override
  String get firstModStep1Detail => 'nams\\mods\\my_first_mod\\entities\\';

  @override
  String get firstModEdit => 'Open editor';

  @override
  String get firstModExplainManifest =>
      'This is your mod\'s nameplate. NAMS reads it first to know the mod exists. Change display_name and author to whatever you like - the id has to stay lowercase with no spaces, because it is how other mods refer to yours.';

  @override
  String get firstModExplainItem =>
      'The filename decides what this is: item_9999.toml means an item with id 9999. Rename the file and you change the id. Inside, [text.name] is what the menu shows, [text.help] is the description. Try changing the name before you write it.';

  @override
  String get tomlEditorValid => 'Valid TOML. NAMS will read these:';

  @override
  String get tomlEditorInvalid =>
      'This is not valid TOML yet - NAMS would skip the file.';

  @override
  String get tomlEditorReset => 'Back to the example';

  @override
  String get tomlEditorWrite => 'Write the file';

  @override
  String get firstModStep2 => 'Write mod.toml';

  @override
  String get firstModStep2Detail => 'The nameplate NAMS reads first.';

  @override
  String get firstModStep3 => 'Add the Test Pebble';

  @override
  String get firstModStep3Detail => 'entities\\item_9999.toml';

  @override
  String get firstModStep4 => 'Check that NAMS finds it';

  @override
  String get firstModStep4Detail => 'Runs NAMS verify against your install.';

  @override
  String get firstModDo => 'Do it';

  @override
  String get firstModDone => 'Done';

  @override
  String get firstModRedo => 'Write again';

  @override
  String get firstModVerify => 'Verify';

  @override
  String get firstModOpenFolder => 'Open folder';

  @override
  String get firstModReset => 'Remove this mod again';

  @override
  String get firstModNoGameDir =>
      'Select your game folder in the launcher first.';

  @override
  String get firstModFinished =>
      'Your mod is in place. Restart the game and the Test Pebble is in your inventory.';

  @override
  String firstModFailed(String error) {
    return 'Could not write the files: $error';
  }

  @override
  String get docsCopy => 'Copy';

  @override
  String get docsCopied => 'Copied';

  @override
  String get docsNativeBlurb =>
      'Describe what you want in a text file and NAMS loads it. This is where most mods start.';

  @override
  String get docsScriptingBlurb =>
      'For when things need to happen on cue: spawns, triggers, cameras, whole scripted scenes.';

  @override
  String get docsSectionReference => 'Reference tables';

  @override
  String get docsReferenceBody =>
      'Every id you can look up: items, weapons, chips, effects, meshes.';

  @override
  String get docsBrowseAll => 'Browse all pages';

  @override
  String docsPagesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count pages',
      one: '1 page',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyTitle => 'Third Party Runtimes';

  @override
  String get thirdPartySubtitle =>
      'ReShade and 3DMigoto, loaded by NAMS from a managed folder. No renaming, no DLL conflicts.';

  @override
  String get thirdPartyReShadeHeader => 'RESHADE';

  @override
  String get thirdPartyMigotoHeader => '3DMIGOTO';

  @override
  String get thirdPartyReShadeHowto =>
      'Install ReShade normally into your NieRAutomata folder (pick dxgi as the API). The launcher moves it into NAMS and sets up the paths for you.';

  @override
  String get thirdPartyMigotoHowto =>
      '3DMigoto shader-mod archives are installed through the drop zone at the top. The launcher sets the loader target so NAMS hooks it.';

  @override
  String get thirdPartyGameModsHeader => 'GAME MODS';

  @override
  String get thirdPartyGameModsHowto =>
      'Mod DLLs made for the original game — speedrun timers, trainers and similar tools — are installed through the drop zone at the top. The launcher sets them up to run under NAMS, but it cannot check what a mod actually does: any of them can crash the game. Only add files you trust, and turn one off below if the game starts misbehaving.';

  @override
  String get thirdPartyGameModsNone => 'No game mods installed';

  @override
  String get thirdPartyWaxRejected =>
      'WAX cannot be used with NAMS. Both do the same job, so running them together crashes the game. Your WAX outfits and items work in the Mods tab instead.';

  @override
  String thirdPartyGameModsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count DLLs',
      one: '1 DLL',
      zero: 'No DLLs',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyGameModsDisabledChip => 'Disabled';

  @override
  String get thirdPartyGameModsToggleHint =>
      'Turn this mod off without deleting it.';

  @override
  String get thirdPartyGameModsRemoveOne => 'Delete this mod';

  @override
  String get thirdPartyStatusInstalled => 'Installed';

  @override
  String get thirdPartyStatusNotInstalled => 'Not installed';

  @override
  String get thirdPartyStatusFoundInGame =>
      'Found in your game folder — click Import to set it up for NAMS.';

  @override
  String get thirdPartyEnable => 'Enable';

  @override
  String get thirdPartyImport => 'Import into NAMS';

  @override
  String get thirdPartyRepair => 'Repair';

  @override
  String get thirdPartyRemove => 'Remove';

  @override
  String get thirdPartyGetReShade => 'Get ReShade';

  @override
  String get thirdPartyShadersMissing =>
      'No effects installed yet — add a preset or shader pack, or they won\'t do anything.';

  @override
  String get thirdPartyOpenFolder => 'Open folder';

  @override
  String thirdPartyPresetsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count presets',
      one: '1 preset',
      zero: 'No presets',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyInstallCard => 'INSTALL';

  @override
  String get thirdPartyDropHere =>
      'Drop ReShade presets, 3DMigoto mods or game-mod DLLs here';

  @override
  String get thirdPartyImported => 'Imported into NAMS';

  @override
  String get thirdPartyInstalled => 'Installed';

  @override
  String get thirdPartyInstallFailed => 'Could not install this file';

  @override
  String get thirdPartyRedirectMods =>
      'This is a game-data mod — use the Mod Manager tab.';

  @override
  String get thirdPartyRedirectTextures =>
      'This is a texture pack — use the Textures tab.';

  @override
  String get thirdPartyUnsupported => 'This file type is not supported here.';

  @override
  String get thirdPartyLodModPrompt =>
      'This is a LodMod. NAMS has LodMod Ext built in — import its settings into the LodMod Ext tab?';

  @override
  String get thirdPartyStatusActive => 'Active';

  @override
  String get thirdPartyStatusInactive => 'Installed, disabled in NAMS tab';

  @override
  String get thirdPartyVersion => 'Version';

  @override
  String get thirdPartyAddonBuild => 'Addon build';

  @override
  String get thirdPartyPresets => 'PRESETS';

  @override
  String get thirdPartyNonePresets => 'No presets installed';

  @override
  String get thirdPartyShaderRepos => 'SHADERS';

  @override
  String get thirdPartyNoneShaders => 'No shaders — effects won\'t compile';

  @override
  String thirdPartyShaderCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count effects',
      one: '1 effect',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyAddons => 'ADDONS';

  @override
  String get thirdPartyFiles => 'FILES';

  @override
  String get thirdPartyShaderFixes => 'ShaderFixes';

  @override
  String thirdPartyShaderFixCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count fixes',
      one: '1 fix',
      zero: 'none',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyLoaderTarget => 'Loader target';

  @override
  String get thirdPartyHunting => 'Hunting';

  @override
  String get thirdPartyHuntingOn => 'on';

  @override
  String thirdPartyUpdateTitle(String name) {
    return 'Replace installed $name?';
  }

  @override
  String thirdPartyUpdateBody(String name) {
    return '$name is already installed. The file you dropped is a different build. Replace the installed one with it?';
  }

  @override
  String get thirdPartyUpdateInstalledLabel => 'Installed now';

  @override
  String get thirdPartyUpdateIncomingLabel => 'You dropped';

  @override
  String get thirdPartyUpdateReplace => 'Replace';

  @override
  String get thirdPartyUpdateKeep => 'Keep current';

  @override
  String get thirdPartyUpdateSkipped => 'Kept the installed version.';

  @override
  String get thirdPartyD3dCompilerMissing =>
      'No d3dcompiler_47.dll here. Under Wine, ReShade falls back to Wine\'s stub and effects won\'t compile. Reinstall a ReShade build that ships it, or drop a native d3dcompiler_47.dll into this folder.';

  @override
  String get thirdPartyBanner =>
      'The tools on this tab are third-party software. NAMS only loads them. It does not implement them and cannot change how they behave. Anything you install through them runs on its own, so correctness, compatibility and safety are your responsibility. If something misbehaves, that is on the tool or the mod, not on NAMS.';

  @override
  String get thirdPartyConfigSection => 'SETTINGS';

  @override
  String get thirdPartyRestartRequired => 'applied on next game start';

  @override
  String get thirdPartyReShadePerformanceMode => 'Performance mode';

  @override
  String get thirdPartyReShadePerformanceModeHint =>
      'Skip effect recompilation on launch. Turn off while tuning effects.';

  @override
  String get thirdPartyReShadeShowFps => 'Show FPS';

  @override
  String get thirdPartyReShadeShowFpsHint =>
      'Draw the frame rate in the ReShade overlay corner.';

  @override
  String get thirdPartyReShadeShowClock => 'Show clock';

  @override
  String get thirdPartyReShadeShowClockHint =>
      'Draw the current time in the ReShade overlay corner.';

  @override
  String get thirdPartyReShadeActivePreset => 'Active preset';

  @override
  String get thirdPartyReShadeOverlayKey => 'Overlay key';

  @override
  String get thirdPartyReShadeNoKey => 'unset';

  @override
  String get thirdPartyMigotoHunting => 'Hunting';

  @override
  String get thirdPartyMigotoHuntingOff => 'Off';

  @override
  String get thirdPartyMigotoHuntingOn => 'On';

  @override
  String get thirdPartyMigotoHuntingNoMarking => 'On, no marking';

  @override
  String get thirdPartyMigotoHuntingHint =>
      'Developer mode: cycle shaders to find and dump them. Leave off for normal play.';

  @override
  String get thirdPartyMigotoMarkingMode => 'Marking mode';

  @override
  String get thirdPartyMigotoMarkingModeHint =>
      'How the hunted shader is highlighted in-game: skip (no change), original, pink, or mono.';

  @override
  String get thirdPartyMigotoMarkingSkip => 'Skip';

  @override
  String get thirdPartyMigotoMarkingOriginal => 'Original';

  @override
  String get thirdPartyMigotoMarkingPink => 'Pink';

  @override
  String get thirdPartyMigotoMarkingMono => 'Mono';

  @override
  String get thirdPartyMigotoVerboseOverlay => 'Verbose overlay';

  @override
  String get thirdPartyMigotoVerboseOverlayHint =>
      'Show detailed 3DMigoto status text on screen. Useful while debugging, noisy otherwise.';

  @override
  String get thirdPartyMigotoCacheShaders => 'Cache shaders';

  @override
  String get thirdPartyMigotoCacheShadersHint =>
      'Reuse compiled shaders across launches. Turn off only when debugging fixes.';

  @override
  String get thirdPartyMigotoCheckForegroundWindow => 'Only hook in foreground';

  @override
  String get thirdPartyMigotoCheckForegroundWindowHint =>
      'Only apply the overlay and hotkeys when the game window is focused.';

  @override
  String get thirdPartyConfigApplied => 'Settings saved.';

  @override
  String get thirdPartyConfigNoIni =>
      'No config file yet — install writes one.';

  @override
  String get tabSectionGeneral => 'General';

  @override
  String get tabSectionConfig => 'Config';

  @override
  String get tabSectionMods => 'Mods';

  @override
  String get tabDividerConfig => 'CONFIG';

  @override
  String get tabDividerMods => 'MODS';

  @override
  String get infoBarVersionPrefix => 'YP Launcher ';

  @override
  String get infoBarLogs => 'LOGS';

  @override
  String get infoBarFaq => 'FAQ';

  @override
  String get infoBarWhatsNew => 'WHAT\'S NEW';

  @override
  String get infoBarShortcut => 'SHORTCUT';

  @override
  String get tooltipFaq => 'Do I still need other mods?';

  @override
  String get chipLodModOn => 'LOD MOD EXT ON';

  @override
  String get chipLodModOff => 'LOD MOD EXT OFF';

  @override
  String get chipReShade => 'ReShade';

  @override
  String get chipNoTextures => 'No textures';

  @override
  String get chipNoMods => 'No mods';

  @override
  String get chipNoCutsceneMod => 'No cutscene mod';

  @override
  String chipTexturesCount(String details) {
    return 'Textures ($details)';
  }

  @override
  String chipModsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'mods',
      one: 'mod',
    );
    return '$count $_temp0';
  }

  @override
  String chipInjectedCount(int count) {
    return '$count injected';
  }

  @override
  String get chipSkRes => 'SK_Res';

  @override
  String chipCutsceneMod(int width, int height, String codec) {
    return 'Cutscene Mod (${width}x$height $codec)';
  }

  @override
  String get warningPluginLoadingDisabled =>
      'Plugin loading is disabled - YoRHa Protocol workspace will not load';

  @override
  String get warningReShadeDisabled => 'ReShade auto-loading is disabled';

  @override
  String get warningTextureInjectionDisabled => 'Texture injection is disabled';

  @override
  String get heapOverridesDescription =>
      'Extra memory added on top of NAMS defaults. Parent heaps grow automatically. Only increase if mods need more memory. Restart required.';

  @override
  String get lodModDescription =>
      'Visual quality patches built into NAMS, inspired by Automata-LodMod by emoose and extended well past it - hence the Ext. Removes LOD pop-in, sharpens shadows and ambient occlusion, forces shadow casting on all objects including foliage, disables manual culling so objects don\'t pop in/out, and removes the vignette. On top of that it adds shadow cascades and per-cascade blur, a bloom rewrite, global illumination, FPS uncap with event safeguards, FXAA, render scaling and high-res map grids.';

  @override
  String get comparisonResolutionNote =>
      'All comparison images on this tab were captured at 2560x1440.';

  @override
  String get namsDescription =>
      'NAMS is the mod loader that hosts NieR:Automata. It launches the game inside its own process, loads your mods at runtime, and applies these engine settings. These options control validation, content features, plugin/ReShade/3DMigoto loading, and memory heap sizes.';

  @override
  String get yorhaProtocolDescription =>
      'YP Devkit (YorHa Protocol) is a modding and developer plugin loaded by NAMS. Its full toolkit — hitbox/light/bone visualizers, a freecam and Photo Mode, entity spawning, the HAP script tree, an item spawner, and more — lives in an in-game overlay you open with F1. This tab is a convenient place to adjust its keybinds, workspace toggles, cheats, and enemy randomizer from the launcher; all of it can also be changed in-game.';

  @override
  String get dropModelModHere => 'Drop model mod folder or archive here';

  @override
  String get dropToInstall => 'Drop to install';

  @override
  String get orClickToBrowse => 'or click to browse';

  @override
  String get mixedModDetected => 'Mixed Mod Detected';

  @override
  String get noOutfitsInstalled => 'No outfits installed yet';

  @override
  String get defaultNoMod => 'Default (no mod)';

  @override
  String get clearAllStartupOutfits => 'Clear all startup outfits';

  @override
  String get removeOutfitTitle => 'Remove outfit?';

  @override
  String get noModelFoundError =>
      'No model found. Needs pl000d.dat/.dtt (2B), pl010d.dat/.dtt (A2), or pl020d.dat/.dtt (9S).';

  @override
  String get unsupportedArchiveFormat => 'Unsupported archive format.';

  @override
  String get extractingArchive => 'Extracting archive...';

  @override
  String extractingArchivePercent(int percent) {
    return 'Extracting - $percent%';
  }

  @override
  String extractingArchivePercentFile(int percent, String file) {
    return 'Extracting $percent% - $file';
  }

  @override
  String get cutsceneScanningArchive => 'Scanning archive for movie folder...';

  @override
  String cutsceneCopyingFile(int current, int total, String name) {
    return 'Copying $current/$total - $name';
  }

  @override
  String cutsceneCopyingFileBytes(
    int current,
    int total,
    String name,
    String mbDone,
    String mbTotal,
  ) {
    return 'Copying $current/$total - $name ($mbDone / $mbTotal MB)';
  }

  @override
  String get failedToExtractArchive => 'Failed to extract archive.';

  @override
  String get extractFailedDiskFull =>
      'Extraction failed: not enough free space on the temp drive. Free up disk space and try again.';

  @override
  String get textureDeleteConfirmTitle => 'Delete texture pack?';

  @override
  String textureDeleteConfirmBody(String name) {
    return 'Permanently remove $name from nams/inject/textures/? This cannot be undone.';
  }

  @override
  String get textureDeleteLabel => 'Delete';

  @override
  String get busyDeletingTexture => 'Deleting texture pack...';

  @override
  String get busyDeletingCutscene => 'Deleting cutscene mod...';

  @override
  String get busyCloseTitle => 'Operation in progress';

  @override
  String get busyCloseBody =>
      'The launcher is busy installing, deleting or extracting files. Closing now can leave broken or partial files on disk. Wait until the current operation finishes, or force-close anyway.';

  @override
  String get busyCloseForce => 'Force close';

  @override
  String extractFailedDetail(String detail) {
    return 'Extraction failed: $detail';
  }

  @override
  String get noOutfitsInstalledNotif => 'No outfits installed.';

  @override
  String get dialogSelectModFolder => 'Select Model Mod Folder';

  @override
  String get dialogNameOutfitShown =>
      'This name will be shown in-game when swapping outfits.';

  @override
  String get dropTextureHere => 'Drop texture folder or archive here';

  @override
  String get installedToTextures => 'Installed to: nams/inject/textures/';

  @override
  String get texturesPackNameTitle => 'Name this pack';

  @override
  String get installingTextures => 'Installing textures...';

  @override
  String get textureDropAnalyzing => 'Analyzing dropped files...';

  @override
  String get textureDropNoDds =>
      'No .dds textures found in this drop. Only .dds files are supported.';

  @override
  String get cutsceneDropAnalyzing => 'Analyzing dropped files...';

  @override
  String get cutsceneDropNoUsm => 'No .usm cutscene files found in this drop.';

  @override
  String get modDropAnalyzing => 'Analyzing dropped files...';

  @override
  String get modDropNotAMod =>
      'This doesn\'t look like a NAMS mod. Drop a folder/archive that contains entities/, wax/, data/, or a mod.toml.';

  @override
  String get modDropMisroutedCutscenes =>
      'This looks like a standalone cutscene mod - drop it on the Cutscenes tab instead. Bundled cutscenes belong inside a mod that already ships entities/ or mod.toml.';

  @override
  String modSideInstalledTextures(String names) {
    return 'Also installed bundled texture pack(s) into nams/inject/textures/: $names';
  }

  @override
  String modLooseUnpairedWarning(String names) {
    return 'Installed, but some files are missing their vanilla pair (.dat/.dtt): $names. The mod may not work fully.';
  }

  @override
  String get modBundledTexturesLabel => 'Bundled textures';

  @override
  String get modBundledCutscenesLabel => 'Bundled cutscenes';

  @override
  String textureBundledWithMod(String modId) {
    return 'Bundled with mod: $modId';
  }

  @override
  String modUninstallAlsoTexturesLabel(String names) {
    return 'Also delete bundled texture pack(s): $names';
  }

  @override
  String get noTextures => 'No textures';

  @override
  String get noTexturesInstalled => 'No textures installed';

  @override
  String get textureConflictHint =>
      'Both mods are loaded. They change some of the same things. Put the one you care about more at the top.';

  @override
  String get noConflictsFound =>
      'No conflicts found. All mods load independently.';

  @override
  String get selectTextureFiles => 'Select Texture Files or Archives';

  @override
  String get noTextureFilesFound => 'No texture files found (.dds, .png, .tga)';

  @override
  String get cheatsAppliedNote => 'Applied once on game start.';

  @override
  String get wipLabel => 'WIP';

  @override
  String get dropCutsceneHere => 'Drop cutscene mod folder or archive here';

  @override
  String get cutsceneMovieHint =>
      'Mod must contain a movie/ folder with .usm files';

  @override
  String get cutsceneNamingTitle => 'Name this cutscene mod';

  @override
  String get removeCutsceneModTitle => 'Remove cutscene mod?';

  @override
  String get noCutsceneModsInstalled => 'No cutscene mods installed yet';

  @override
  String get cutsceneHowItWorks1 =>
      'Custom cutscenes load from nams/cutscenes/ instead of data/movie/.';

  @override
  String get cutsceneHowItWorks2 =>
      'If a custom file is missing or broken, the original plays as fallback.';

  @override
  String get cutsceneHowItWorks3 =>
      'Your original game files are never touched - mods load from a separate location.';

  @override
  String get cutsceneStructurePath =>
      'nams/cutscenes/<mod_name>/movie/<filename>.usm';

  @override
  String get cutsceneFolderNameLimit =>
      'Folder names are limited to 27 characters.';

  @override
  String get cutsceneMigrationTitle =>
      'Custom cutscene files detected in data/movie/';

  @override
  String cutsceneMigrationBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'files',
      one: 'file',
    );
    return 'Found $count custom cutscene $_temp0 directly in data/movie/. These overwrite the originals permanently. Next time, install cutscene mods here instead - if a custom file fails to load, the original plays as fallback.';
  }

  @override
  String get noMovieFolderFound => 'No movie/ folder with .usm files found.';

  @override
  String get noUsmFilesFound => 'No .usm files found in the mod.';

  @override
  String get onboardingWelcomeTitle => 'Welcome to YoRHa Protocol';

  @override
  String get onboardingWelcomeSubtitle =>
      'One launcher. All your mods. New NieR content.\n\nDrag-and-drop mods, mid-game outfit switching, new quests, and a built-in devkit - without configuration nightmares.';

  @override
  String get onboardingSelectTitle => 'Select your NieR:Automata installation';

  @override
  String get onboardingSearchingDrives => 'Scanning all drives...';

  @override
  String get onboardingSearchingNier => 'Searching for NieR:Automata...';

  @override
  String get onboardingSelectInstallation => 'Select Installation';

  @override
  String get onboardingFirstPlaythroughTitle =>
      'Is this your first playthrough?';

  @override
  String get onboardingFirstPlaythroughHint => 'We\'ll hide spoilers if so.';

  @override
  String get onboardingFirstYes => 'Yes - hide spoiler features';

  @override
  String get onboardingFirstNo => 'No - show everything';

  @override
  String get onboardingMigrationAskTitle => 'Modded NieR before?';

  @override
  String get onboardingMigrationAskBody =>
      'We\'ll pick up your old setup automatically.';

  @override
  String get onboardingMigrationYes => 'Yes';

  @override
  String get onboardingMigrationNo => 'No';

  @override
  String get onboardingMigrationResultsTitle => 'What we found';

  @override
  String get onboardingMigrationResultsBody => '';

  @override
  String get onboardingMigrationNothingFound =>
      'Nothing detected. You\'re clean.';

  @override
  String get onboardingMigrationActionReshadeKept =>
      'Disabled - NAMS has native DoF. Re-enable in NAMS config if you want it.';

  @override
  String get onboardingMigrationActionReshadeIncompatible =>
      'Addon/ImGui version - replace or remove.';

  @override
  String get onboardingMigrationActionTextures => 'Will load automatically.';

  @override
  String get onboardingMigrationActionLodMod => 'LodMod.ini imported.';

  @override
  String get onboardingMigrationActionSkRes => 'Picked up automatically.';

  @override
  String get onboardingMigrationActionNaiom =>
      'Your NAIOM settings are imported into NAMS automatically. Afterwards you can delete the old NAIOM files (dinput8.dll, NAIOM.ini).';

  @override
  String get onboardingMigrationActionCutscenes => 'Already integrated.';

  @override
  String get onboardingMigrationActionExistingMods =>
      'Already installed - kept as-is.';

  @override
  String onboardingMigrationLabelExistingMods(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count mods in nams/mods/',
      one: '1 mod in nams/mods/',
    );
    return '$_temp0';
  }

  @override
  String get onboardingModInstallAskTitle => 'Install a mod before launching?';

  @override
  String get onboardingModInstallAskBody =>
      'Got a .zip or folder? We\'ll handle it.\n(Textures and cutscenes have their own tabs - install those later.)';

  @override
  String get onboardingModInstallYes => 'Yes, install a mod now';

  @override
  String get onboardingModInstallSkip => 'No, skip for now';

  @override
  String get onboardingModInstallTitle => 'Add your first mod';

  @override
  String get onboardingModInstallBody => 'Supports .zip, .7z, and folders.';

  @override
  String get onboardingModInstallSubBody => 'Game files stay safe.';

  @override
  String get onboardingModInstallBusy => 'Installing…';

  @override
  String get onboardingModInstallInspecting => 'Inspecting mod…';

  @override
  String onboardingModInstallExtractingPercent(int percent) {
    return 'Extracting… $percent%';
  }

  @override
  String onboardingModInstallExtractingFile(int percent, String file) {
    return 'Extracting $percent% - $file';
  }

  @override
  String get onboardingModInstallNotAMod =>
      'That does not look like a valid NAMS mod. Make sure it is a .zip / .7z file with a mod.toml or a recognised data layout.';

  @override
  String get onboardingOutfitHintHeader => 'To wear it';

  @override
  String get onboardingOutfitHintCompat => 'Buy or equip from your inventory.';

  @override
  String get onboardingOutfitHintData =>
      'F1 in-game → wardrobe icon (top-left).';

  @override
  String get onboardingOutfitHintMultiOutfit =>
      'Or use Multi-Outfit NPC at Resistance Camp.';

  @override
  String get onboardingOutfitHintMultiOutfitLink => 'Get Multi-Outfit NPC';

  @override
  String get onboardingOutfitHintMultiOutfitUrl =>
      'https://www.nexusmods.com/nierautomata/mods/789';

  @override
  String onboardingModInstallFailed(String reason) {
    return 'Install failed: $reason';
  }

  @override
  String get onboardingModInstallNoGameDir =>
      'Game directory not selected yet. Go back and pick your install first.';

  @override
  String get onboardingModInstallInstalledHeader => 'Installed:';

  @override
  String get onboardingModInstallSkipButton => 'Skip';

  @override
  String get onboardingModInstallDoneButton => 'Done';

  @override
  String get onboardingModInstallPickerTitle =>
      'Select a mod (.zip, .7z) or open a folder';

  @override
  String get onboardingModInstallFolderPickerTitle => 'Select a mod folder';

  @override
  String get onboardingReadyTitle => 'You\'re all set!';

  @override
  String get onboardingCreateShortcut => 'Create desktop shortcut';

  @override
  String get onboardingFirstPlaythroughSpoilerFree =>
      'First playthrough (spoiler-free)';

  @override
  String get onboardingFullAccess => 'Full access';

  @override
  String get detectionReShade => 'ReShade';

  @override
  String get detectionHDTextures => 'HD Textures';

  @override
  String get detectionLodMod => 'LOD Mod (Automata-LodMod)';

  @override
  String get detectionSkRes => 'Special K Textures (SK_Res)';

  @override
  String get detectionNaiom => 'NAIOM';

  @override
  String get detectionCutscenes => 'Cutscene Mods (nams/cutscenes)';

  @override
  String get detectionCustomMovie => 'Custom cutscenes in data/movie/';

  @override
  String get detectionDetected => 'Detected';

  @override
  String get detectionReShadeIncompatible => 'Incompatible (addon/ImGui)';

  @override
  String get detectionNotFound => 'Not found';

  @override
  String get detectionNoneFound => 'None found';

  @override
  String get detectionLodModMigrated => 'Found - migrated into NAMS';

  @override
  String get detectionSkResAuto => 'Found - loaded automatically';

  @override
  String get detectionNaiomPending =>
      'Found - settings are imported automatically';

  @override
  String get detectionNoneInstalled => 'None installed';

  @override
  String get detectionCustomMovieHint =>
      'Found - consider using nams/cutscenes/ instead for safe fallback';

  @override
  String get detectionInstalled => 'Installed';

  @override
  String get detectionCustomFilesDetected => 'Custom files detected';

  @override
  String get detectionMigratedIntoNams => 'Migrated into NAMS';

  @override
  String get detectionLoadedAutomatically => 'Loaded automatically';

  @override
  String get detectionMigrationComingSoon => 'Settings imported automatically';

  @override
  String get detectionNotSet => 'Not set';

  @override
  String get labelGame => 'Game';

  @override
  String get labelMode => 'Mode';

  @override
  String get labelDlc => 'DLC';

  @override
  String get labelNoDlc => 'No DLC';

  @override
  String get searchingForNier => 'Searching for NieR:Automata...';

  @override
  String get autoButton => 'AUTO';

  @override
  String get nierFound => 'NieR:Automata Found';

  @override
  String get selectInstallation => 'Select your installation:';

  @override
  String get notYourGame => 'Not your game?';

  @override
  String get searchAllDrives => 'Search all drives';

  @override
  String get selectManually => 'Select manually';

  @override
  String get notFoundTitle => 'Not Found';

  @override
  String get notFoundMessage =>
      'Could not find NieR:Automata via Steam. Want to scan all drives? This may take up to 30 seconds.';

  @override
  String get scanDrives => 'Scan drives';

  @override
  String get confirmInstallation => 'Is this the correct installation?';

  @override
  String get cancelButton => 'Cancel';

  @override
  String get noSelectManually => 'No, select manually';

  @override
  String get yesButton => 'Yes';

  @override
  String get installationsFoundTitle => 'NieR:Automata Installations Found';

  @override
  String get validInstallations => 'Valid installations (with data folder):';

  @override
  String get withoutDataFolder => 'Without data folder:';

  @override
  String get noLogEntries => 'No log entries found';

  @override
  String get noLogMatches => 'No log entries match your search';

  @override
  String get logViewerTitle => 'LOG VIEWER';

  @override
  String get logSearchPlaceholder => 'Search level / module / message...';

  @override
  String get logLiveBadge => 'LIVE';

  @override
  String get logAutoScroll => 'Auto-scroll to newest';

  @override
  String get entriesSuffix => 'entries';

  @override
  String get scrollToBottom => 'Scroll to bottom';

  @override
  String get openLogsFolder => 'Open logs folder';

  @override
  String get diagnosticsButton => 'Generate diagnostics';

  @override
  String get diagnosticsSubtitle =>
      'Collect a full report of your install (game files, mods, ReShade/3DMigoto, settings) to share when asking for help.';

  @override
  String get copyCommandTitle => 'Manual launch command';

  @override
  String get copyCommandDesc =>
      'If the game will not start from the launcher, copy the NAMS command and run it in a terminal. NAMS prints why it failed there, which is the fastest way to find the cause.';

  @override
  String get copyCommandButton => 'Copy command';

  @override
  String get diagnosticsTitle => 'Diagnostics summary';

  @override
  String get diagnosticsCollecting => 'Collecting diagnostics...';

  @override
  String get diagnosticsCopy => 'Copy summary';

  @override
  String get diagnosticsCopied => 'Summary copied to clipboard';

  @override
  String get diagnosticsSaveFull => 'Save full report';

  @override
  String diagnosticsSavedAt(String path) {
    return 'Full report saved to $path';
  }

  @override
  String get modsTutorialBack => 'Back';

  @override
  String get modsTutorialNext => 'Next';

  @override
  String get modsTutorialFinish => 'Got it';

  @override
  String get modsTutorialSkip => 'Skip';

  @override
  String get modsTutorialMenuEcosystem => 'NAMS & the launcher';

  @override
  String get modsTutorialMenuInstall => 'How to install a mod';

  @override
  String get modsTutorialMenuProfiles => 'How profiles work';

  @override
  String get modsTutorialMenuSupporting => 'Supporting the ecosystem';

  @override
  String get modsTutorialSupportingStep1Title =>
      'An ecosystem, not a mod manager';

  @override
  String get modsTutorialSupportingStep1Body =>
      '**NAMS has been a 3+ year project.** What started as a single modloader grew into a whole platform that **mods**, **plugins**, and tools are now built on top of:\n\n- **Mods** are content additions - outfits, custom quests, new weapons, textures. They use NAMS\'s content system but don\'t run their own code.\n- **Plugins** are programs that run alongside the game and can extend the modloader itself - things like the multiplayer mod, debug overlays, or new game systems. They\'re written in code, loaded by NAMS at startup.\n- The launcher you\'re using right now is **not a plugin** - it\'s a separate app that prepares your mods and configs before launch.\n\nThe mods you see today aren\'t running in spite of an obstacle course - they\'re running on top of years of foundational work that exists specifically so you don\'t have to redo it.\n\nThis section is about **how that platform stays alive** and what it means for you - whether you\'re just playing, dipping into modding, or thinking about contributing something back.\n\nThe next pages start with the parts that apply to **everyone**, then go into more technical territory at the end. Skip ahead whenever you\'ve heard enough - none of it is required reading.';

  @override
  String get modsTutorialSupportingStep2Title => 'Content without code';

  @override
  String get modsTutorialSupportingStep2Body =>
      '**You don\'t need to be a programmer to add to this ecosystem.**\n\nNAMS has a content system that already supports - and is steadily growing to support more - declarative content additions:\n\n- **Custom quests** added on top of the vanilla story.\n- **New weapons and items** with their own behavior, no overrides.\n- **Accessories** in **new slots**, not replacing existing ones.\n- **Custom cutscenes** that live in `nams/cutscenes/` or are bundled inside a mod (e.g. a custom quest shipping its own cinematics) - the original cutscenes are **never replaced**, new ones simply load alongside.\n\nThe through-line: **additive, not replacive.** Vanilla content is preserved; modded content is layered on top. That means a modeler with no programming experience can build accessories, weapons, characters, and ship them as additions - without overwriting anything, without breaking saves, without conflicts with other mods doing the same.\n\nThis surface is going to **expand** over time. Every release adds more declarative hooks so non-programmers can do more.';

  @override
  String get modsTutorialSupportingStep3Title =>
      'Ways to contribute - not just code';

  @override
  String get modsTutorialSupportingStep3Body =>
      'Making content (the previous page) is one way to give back. It\'s far from the only one - and a lot of the things that keep an ecosystem healthy don\'t involve making mods at all. Here\'s what actually moves the needle:\n\n- **Write a guide.** \"How I built X with NAMS\", \"how I debugged Y\", \"the five things I wish I\'d known\". Most of the gaps in onboarding right now are documentation gaps.\n- **Report a real bug, well.** A reproducible repro, a log, the diagnostics report from the Logs panel. That\'s worth more than ten \"it doesn\'t work\" tickets.\n- **Model.** Accessories, weapons, characters, props. NAMS\'s content system loads those as **additions** - new slots, no overrides - so a modeler with no programming experience can ship work that just slots into everyone else\'s loadouts without conflicts.\n- **Translate.** The launcher is localized. If your language isn\'t there yet and you\'d use it, the strings are in `lib/l10n/` and a PR is welcome.\n- **Test on weird hardware.** Steam Deck, AMD GPUs, ultrawide setups, multi-monitor, controllers nobody owns. Bugs that only show up on rare configurations stay hidden until someone reports them.\n- **Just answer questions in the Discord.** Helping the next new person is contribution. Most of what survives in any ecosystem long-term is the culture of the people who showed up early.\n- **Reverse-engineer one game function and give the API back.** *(more on this on the next page if you\'re curious)* - for the technically-inclined, this is the highest-leverage contribution there is.\n\n### Closing thought (so far)\n\nA lot of free time, personal investment, and stubborn fleiß went into making this platform exist - always with the mindset of **enabling other people to start doing something**. If one of those bullet points feels doable, you\'re already most of the way there. The Discord (**YoRHa Continuum**) is the place to ask.\n\nThe next two pages get more technical - read on if you\'re curious about how plugins coexist and how new game APIs end up in NAMS, or stop here if you\'ve heard enough.';

  @override
  String get modsTutorialSupportingStep4Title => 'Plugins coexist - by design';

  @override
  String get modsTutorialSupportingStep4Body =>
      '*The next two pages are more technical and aimed at people thinking about building plugins. Skip if not your thing.*\n\nA defining feature of the NAMS platform is that **plugins can run together at the same time**, in the same game session, without fighting each other.\n\n**Multiplayer Mod by Jasper** is one of the biggest things to happen in the NAMS ecosystem and is still actively maintained - huge respect to that work. The YP devkit and the MP plugin **both load at once**, both work, both render their own UI on top of the game. That\'s not a happy accident - that\'s what NAMS\'s plugin host was built for.\n\nSo if you ship a plugin that follows the platform\'s expectations, **it can coexist with everything else already running** - your plugin, the MP plugin, the YP devkit, future plugins from people you\'ll never meet. You don\'t have to compete for hooks or fight load order; the platform mediates.\n\nThere are still refactorings happening every month to reduce edges where one plugin can accidentally break another. It\'s a moving target - but the direction is clear and the work is ongoing.';

  @override
  String get modsTutorialSupportingStep5Title =>
      'You stand on free reverse-engineering';

  @override
  String get modsTutorialSupportingStep5Body =>
      'Most of the engine APIs you\'d need to build a serious plugin already exist in NAMS - and **they exist because someone reverse-engineered the game** to solve their own problem and contributed the result back.\n\nThe game is closed-source. Every API in NAMS that lets you read or write some game state is the result of someone tracing functions, finding offsets, validating behavior. That\'s a lot of free work, and it lives in NAMS specifically so the **next** plugin author doesn\'t have to redo it.\n\nWhen you build on NAMS, you inherit all of that. You don\'t start from `LoadLibrary`; you start from APIs that someone already wrestled into existence - and the next person who needs the API you reverse-engineered gets the same gift.\n\n### Why this is the highest-leverage contribution\n\nIf you ever do this - even once - you\'ve permanently saved every future plugin author the same work. That\'s the asymmetry. A guide helps a few people read it; an API in NAMS helps everyone who\'ll ever need that capability, forever. The ecosystem grows on the back of people who reverse-engineer one thing for themselves and leave the result behind for everyone.';

  @override
  String get modsTutorialEcosystemStep1Title => 'Why this all exists';

  @override
  String get modsTutorialEcosystemStep1Body =>
      'Modding NieR has historically been painful. Mods that work fine on their own start fighting the moment you stack a few of them - different DLL hooks (`dxgi`, `d3d11`, `dinput8`) compete for the same slot, the wrong wrapper wins the load order, and the game silently crashes on boot. People with 5–10 mods spend more time bisecting than playing.\n\nFor a long time the only answer was *manual installs only*: drop loose `.dat`/`.dtt` files into `data/`, hand-edit configs, never use a mod manager. That works for one or two mods, but it overwrites real game files and leaves no record of what changed. Tools like Vortex didn\'t help - they don\'t understand NieR\'s quirks.\n\n**NAMS exists to fix this at the modloader level**, and **this launcher exists to give NAMS a friendly face on top.**';

  @override
  String get modsTutorialEcosystemStep2Title => 'What NAMS does';

  @override
  String get modsTutorialEcosystemStep2Body =>
      '**NAMS is the modloader.** It\'s not a proxy DLL hijacking `dxgi.dll` or `d3d11.dll` like older tools did - that\'s the very mechanism that caused the conflicts in the first place.\n\nInstead, NAMS runs as its own host process: it loads NieR:Automata as a library inside that process (the game\'s exe transformed into a loadable `game.bin`) and initializes the modloader before the game starts. Nothing is injected into another process - NAMS *is* the process the game runs in, with full control over what\'s about to load.\n\nFrom there, NAMS does two big things:\n\n**1. Reimplements the features other tools used to provide** - LodMod, Limit Break, texture injection, fast loading - natively, in one coordinated layer. Mods plug into NAMS APIs instead of fighting over which DLL hook gets called first.\n\n**2. Provides a virtual file system (VFS):**\n\n- Every mod lives in its own folder under `nams/mods/<modId>/` - never overwriting real game files.\n- At runtime NAMS overlays active mods into the engine\'s view of `data/`.\n- Your vanilla `data/*.cpk` and `NieRAutomata.exe` are **never modified**, so launching unmodded through Steam still works exactly as before.\n\nMods declare what they change in a manifest. NAMS validates and loads them in a defined order, so you finally get **clean enable/disable per mod** and **knowable conflict detection** - neither was possible with the old drop-files-into-`data` approach.\n\n### How this fits together\n\nThis launcher is **not** built directly on top of NieR:Automata. It doesn\'t reverse-engineer the game, hook engine functions, or know anything about `.dat`/`.dtt` formats on its own. The order is:\n\n1. **NieR:Automata** - the game, untouched.\n2. **NAMS** - the modloader, designed first to make scalable modding possible at all (engine reimplementation, VFS, plugin host, content framework).\n3. **This launcher** - built on top of NAMS as the helper. It reads NAMS\'s TOML configs, writes into NAMS\'s folder layout, and talks to NAMS\'s APIs. That\'s it.\n\nThe practical consequence: NAMS is the load-bearing layer. The launcher is just a friendly UI in front of it, and could be replaced by a different UI (or the command line) without your mods caring.\n\n### And it has been\n\nThis isn\'t theoretical. **The NAO Launcher by Rustukun** is a separate launcher built on the same foundation - a different UI, different design choices, talks to the same NAMS underneath. Your mods, your `nams/mods/<modId>/` folders, your `disabled_mods.toml` - all of it works the same regardless of which launcher you\'re using.\n\nThat\'s the proof that NAMS is the platform and any launcher (this one, NAO, a future one nobody\'s written yet) is just a frontend choice. Pick whichever fits your workflow; your mod library doesn\'t have to move.';

  @override
  String get modsTutorialEcosystemStep3Title =>
      'What this launcher adds - and what\'s different';

  @override
  String get modsTutorialEcosystemStep3Body =>
      'NAMS handles loading. The launcher handles **everything around it** - install, organisation, troubleshooting:\n\n- **Mod Manager** - drag-and-drop install of NAMS-format mods, automatic layout normalization (wax/SK_Res wrappers, bundled assets), manifest inspection, conflict warnings.\n- **Textures** - manage standalone texture packs and `load_order` priority without hand-editing TOML.\n- **Cutscenes** - install HD cutscene mods, auto-detect codec (H264 vs MPEG-2), wire up the right NAMS settings.\n- **Profiles** - keep multiple mod loadouts side-by-side, switch with one click, without copying or losing state.\n- **Diagnostics** - full report of what\'s installed where, what\'s leftover from old installs, what NAMS sees vs what\'s actually on disk.\n\n### Why we built this\n\n**Nothing against manual installs.** Dropping one outfit\'s `.dat`/`.dtt` files into the right `data/` subfolder works fine for one or two mods. The launcher is built for the scale beyond that.\n\nThe NAMS ecosystem now supports things like:\n\n- **30+ outfit mods** with multi-outfit swapping per character.\n- **20+ custom quests** added on top of the vanilla story.\n- **10+ new weapons** with their own behaviour.\n- Plus textures, cutscenes, plugins, balance changes…\n\nManaging that by hand isn\'t a philosophical preference - it\'s **just not possible**. You can\'t track which file came from which mod, you can\'t enable or disable a single mod cleanly, you can\'t tell why something broke. At scale, manual modding hits a hard wall - and the NAMS ecosystem has been past that wall for a while.\n\n### How this differs from NAMH and Vortex\n\nIf you\'ve been around the NieR scene a while, you remember how previous mod managers ended:\n\n- **NAMH** (NieR Automata Mod Helper) became unmaintained, broke games in unrecoverable ways, hit \"program in use\" lock issues, and the standard recovery was *uninstall the game, reinstall, do it manually.* It worked by **directly replacing files in `data/`** - once a NAMH install went sideways, there was no clean way back.\n- **Vortex** never properly understood NieR\'s file structure. Its virtual file system makes assumptions that don\'t match how the game actually loads content, so installs would silently misplace files.\n\nThis launcher is built differently. The defining choices:\n\n1. **No file replacement, ever.** Mods live in `nams/mods/<modId>/` and get overlaid into the engine\'s view at runtime via NAMS\'s VFS. Vanilla `data/` is never touched. There\'s no \"unrecoverable state\" because nothing in the real game ever changed.\n2. **Every action is reversible.** Uninstall a mod → its folder and bundled assets are removed cleanly. Disable a mod → an entry in `disabled_mods.toml` and NAMS skips it. No hidden state, no irreversible writes.\n3. **Profiles instead of one global state.** Past managers committed everything to a single configuration. Profiles let you keep multiple loadouts side-by-side and switch atomically - nothing to corrupt, nothing to lose.\n4. **Built on a maintained modloader.** NAMH\'s death came from the modloader story being uncertain. NAMS is the foundation everything here is built on, and the launcher follows its updates.\n\nIf this launcher ever stops being maintained, your mods are still just files on disk that NAMS reads - you don\'t end up locked out of your own install.';

  @override
  String get modsTutorialEcosystemStep4Title => 'Where to go next';

  @override
  String get modsTutorialEcosystemStep4Body =>
      'If you\'ve never installed a mod here before:\n\n- **How to install a mod** - walks through the install flow tab-by-tab.\n- **How profiles work** - explains separate loadouts and when to use them.\n\nFind both in this same help menu (the **?** icon you used to open this).\n\n**The short version:** drop archives onto the right tab (Mod Manager for character/data mods, Textures for standalone texture packs, Cutscenes for HD cutscenes), hit Play, look at Logs if something breaks. The launcher tries to do the right thing automatically - if you disagree with a choice it made, every action is reversible from the UI.';

  @override
  String get modsTutorialHelpTooltip => 'Tutorials & help';

  @override
  String get modsTutorialInstallStep1Title => 'Drop your mod here';

  @override
  String get modsTutorialInstallStep1Body =>
      'This is the **Mod Manager** tab - it\'s where character, outfit, and other gameplay mods get installed.\n\nDrag a `.zip`, `.7z`, or `.rar` from Nexus onto this drop zone (or click to browse). The launcher unpacks it, checks the layout, and puts it in the right place. You don\'t need to extract anything yourself.\n\n**Good to know:** your vanilla game files stay untouched. Mods live in a separate `nams/` folder, so you can always launch the unmodded game through Steam if you want.';

  @override
  String get modsTutorialInstallStep2Title =>
      'Trying to install a WAX mod? Read this first';

  @override
  String get modsTutorialInstallStep2Body =>
      '**WAX mods do work here** - NAMS reimplements WAX up to a tested compatibility version. Most mods on Nexus that target that version or older will install and run normally.\n\n### How compatibility works\n\nNAMS is validated against a specific WAX version. Anything WAX shipped up to and including that version: supported. Anything WAX adds in a **newer** version after that: not automatically - that\'s a new feature on the WAX side that has to be reimplemented from scratch on the NAMS side.\n\n### What happens when WAX adds something new\n\nWhen WAX ships a new feature in a future version, it gets evaluated against NAMS\'s roadmap:\n\n- **In scope** - if the feature fits where NAMS is already heading, it gets implemented and a future NAMS update will support mods using it.\n- **Out of scope** - NAMS has its own content extensions to focus on (custom quests, custom world maps, custom plugin chips, expanded modding APIs, and more). Reimplementing every WAX feature isn\'t the priority. Some niche WAX features may simply not get NAMS equivalents.\n\n**This isn\'t a slight against WAX.** They\'re separate projects with separate goals. NAMS isn\'t trying to be a drop-in WAX replacement - it\'s its own platform that happens to be **broadly compatible** with WAX because most users want their existing mod libraries to keep working.\n\n### This pattern is normal\n\nThis kind of split is **how every modded-game ecosystem evolves**, not a NieR-specific weirdness. Concrete example: **Skyrim Legendary Edition (LE)** and **Skyrim Special Edition (SE)** are forks of the same engine. SE is broadly compatible with LE mods, but not 100% - some LE mods need conversion, and a handful never got ported because they relied on engine quirks SE changed. The Skyrim community didn\'t treat that as a flaw; it just became part of how the ecosystem worked. Same story with **OpenMW vs vanilla Morrowind**, **Java vs Bedrock Minecraft**, **KSP1 vs KSP2 mods**, etc.\n\nWhenever a platform reimplements another platform\'s behavior, you get a compatibility envelope - most of it works, edges don\'t. That\'s the deal across every modded-game scene that\'s been around long enough to fork.\n\n### Best practice if you\'re not sure\n\n1. **Create a fresh empty profile** (see *How profiles work* in the help menu) and switch to it.\n2. **Drop the WAX mod alone** into that profile. Nothing else.\n3. **Press Play.** Works? Install it into your real profile.\n4. **Doesn\'t work?** The mod is probably using a feature past NAMS\'s tested WAX version, or one that NAMS chose not to reimplement.\n\n### What to expect\n\n- If you have features X, Y, and Z working in NAMS and the WAX mod you want needs feature W that isn\'t supported - and you can live without W - you\'ve still got X, Y, Z working fine alongside it.\n- If feature W is essential to the mod and NAMS doesn\'t have it, you\'re picking between WAX (gets you W but loses NAMS\'s other benefits) or NAMS (everything else, but not W).\n\n**And don\'t forget the other side of the trade-off:** sticking with WAX means missing out on the **NAMS-exclusive mods** that don\'t run on WAX at all - multi-outfit swapping per character, custom quests, and the broader plugin ecosystem (Multiplayer Mod, the YP devkit, future plugins). Those depend on NAMS APIs that WAX doesn\'t have. So the choice isn\'t \"NAMS minus W\" vs \"WAX with W\" - it\'s \"NAMS-ecosystem minus W\" vs \"WAX with W minus everything NAMS-exclusive.\"\n\nThat\'s a real trade-off, and it\'s yours to make. We\'re not the right people to ask whether a specific WAX-exclusive feature will get NAMS support - that\'s an ecosystem-roadmap question, best directed at the YoRHa Continuum Discord.';

  @override
  String get modsTutorialInstallStep3Title => 'Your installed mods';

  @override
  String get modsTutorialInstallStep3Body =>
      'Every mod you installed shows up here.\n\n**Toggle on the right** - enable or disable the mod. Disabling keeps it installed but tells the modloader to skip it on next launch.\n\n**Game crashing on boot?** Disable half your mods, launch, repeat. The toggles let you bisect quickly.\n\n**Warning badges** mark mods that conflict with each other (two mods replacing the same outfit, etc.) - that\'s the usual reason a game won\'t reach the title screen.';

  @override
  String get modsTutorialInstallStep4Title => 'Mod details';

  @override
  String get modsTutorialInstallStep4Body =>
      'Click any mod in the list to see its details here: author, version, what it changes, conflicts with your other mods, and any **bundled texture packs or cutscenes** that came with it.\n\nIf a mod won\'t work, the most common reasons show up here - *requires a newer NAMS version* or *conflicts with another mod*. Both are visible **before** you press Play.\n\nThe **Uninstall** button cleans the mod up properly, including its bundled extras.';

  @override
  String get modsTutorialInstallStep5Title =>
      'Standalone texture mods → Textures tab';

  @override
  String get modsTutorialInstallStep5Body =>
      '**Texture-only mods** (HD upscale packs, color reskins) don\'t get installed here. They live in their own tab.\n\nClick **Textures** in the sidebar to install those. Drag `.zip` archives in the same way - the launcher knows what it\'s looking at.\n\n**Note:** if a character mod *bundles* its own textures, those install automatically with the mod. You only use the Textures tab for **standalone** texture packs.';

  @override
  String get modsTutorialInstallStep6Title => 'Cutscene mods → Cutscenes tab';

  @override
  String get modsTutorialInstallStep6Body =>
      '**Cutscene mods** (HD cutscenes, replacement videos) go in their own tab too.\n\nClick **Cutscenes** in the sidebar to install those.\n\n**Same rule as textures:** if a character mod bundles cutscenes, they install automatically - you only use the Cutscenes tab for **standalone** cutscene packs.';

  @override
  String get modsTutorialInstallStep7Title => 'Press Play';

  @override
  String get modsTutorialInstallStep7Body =>
      'Head back to the **Launcher** tab and press **PLAY**. The modloader reads your mods fresh on every game start, so you don\'t need to restart this launcher.\n\n### If the game crashes\n\nOpen **Logs** (bottom-right) - the modloader\'s output usually names the broken mod. Come back here and disable it.\n\n### Still broken with everything disabled?\n\nIf you (or a previous mod manager) ever dropped loose `.dat` / `.dtt` files directly into `<gameDir>/data/`, the engine still picks those up - and the modloader can\'t see or disable them. That\'s the kind of mess this launcher specifically avoids: every mod stays isolated in `nams/mods/<modId>/` instead of overwriting real game files.\n\nOpen **Logs → Diagnostics** and check the *Vanilla data/ overlay* section. Anything listed there is leftover from an old install - move those folders out of `data/` and your game is back to a clean state.';

  @override
  String get modsTutorialProfilesStep1Title => 'What profiles are for';

  @override
  String get modsTutorialProfilesStep1Body =>
      'Profiles let you keep multiple separate mod loadouts side by side.\n\nFor example:\n\n- A **main** profile with the mods you actually play with.\n- A **test** profile for trying out anything new.\n\nIf a sketchy new mod breaks your game, just switch back to **main** and you\'re playing again. Your loadouts never mix.\n\n**Important:** mods you\'re not actively using aren\'t deleted - they\'re just parked, ready to come back when you switch.';

  @override
  String get modsTutorialProfilesStep2Title => 'Create a profile';

  @override
  String get modsTutorialProfilesStep2Body =>
      'Click **NEW** in the profile bar, type a name, confirm.\n\nThe launcher creates a fresh empty profile and switches to it. Your previous profile\'s mods stay safe on disk - they\'re not gone, they\'re just parked.\n\nNow you can install whatever you want into this new profile without touching your other loadouts.';

  @override
  String get modsTutorialProfilesStep3Title => 'Switch, rename, delete';

  @override
  String get modsTutorialProfilesStep3Body =>
      '**Switch** - pick any profile from the dropdown. Your mod list flips over instantly.\n\n**Rename** - change a profile\'s name without losing anything.\n\n**Delete** - permanently remove an inactive profile (you can\'t delete the active one or the last remaining one).\n\nThe whole switch happens in one step - if anything goes wrong it rolls back automatically, so you can\'t end up in a broken state.';

  @override
  String get modsTutorialProfilesStep4Title =>
      'What follows the profile, what doesn\'t';

  @override
  String get modsTutorialProfilesStep4Body =>
      '**Per-profile** (changes when you switch):\n\n- Your installed mods\n- Which mods are enabled or disabled\n- Textures that came bundled with a mod\n\n**Shared between all profiles** (never changes):\n\n- Standalone texture packs you installed via the Textures tab\n- Cutscene mods\n- Plugins\n- All your launcher settings\n\nSo profiles only flip what\'s actually mod-specific. Your global setup follows you everywhere.';

  @override
  String get platformUnsupportedTitle => 'Cannot launch on this platform';

  @override
  String get platformUnsupportedLinux =>
      'NieR:Automata is a Windows game, so it needs a compatibility layer to run on Linux.\n\nInstall Steam with Proton (the game runs fine under Proton), or install CrossOver/Wine. Once a runtime is present, the launcher can start the game.\n\nNative Linux without a translation layer cannot run the game.';

  @override
  String get platformUnsupportedMacos =>
      'NieR:Automata is a Windows game. Run the launcher through CrossOver/Wine - that has worked in the past, though it has not been re-tested recently. Native macOS without a translation layer cannot run the game.\n\nIf you did get it working somehow, please use the command line directly instead of this launcher.';

  @override
  String get playDisabledTooltip => 'Launch unavailable on this platform';

  @override
  String get diagnosticsClose => 'Close';

  @override
  String get diagnosticsSectionDataOverlay =>
      'Vanilla data/ overlay (manual drops)';

  @override
  String get diagnosticsSectionExternalLegacy => 'External / legacy';

  @override
  String get diagnosticsSectionGameRootExtras =>
      'Game root extras (non-vanilla)';

  @override
  String get diagnosticsSectionGameIdentity => 'Game identity';

  @override
  String get diagnosticsSectionNamsHealth => 'NAMS health';

  @override
  String get diagnosticsSectionReshade => 'ReShade';

  @override
  String get diagnosticsSectionMigoto => '3DMigoto';

  @override
  String get diagnosticsSectionTexturePacks => 'Texture packs';

  @override
  String get diagnosticsSectionVanillaDrops => 'Files dropped in vanilla data/';

  @override
  String get diagnosticsSectionNonDefault => 'Non-default settings';

  @override
  String get diagnosticsSectionRecentIssues => 'Recent NAMS issues';

  @override
  String diagnosticsExeVariant(String variant) {
    return 'exe: $variant';
  }

  @override
  String get diagnosticsExeUnsupported => 'unsupported build';

  @override
  String get diagnosticsDlcPresent => 'DLC';

  @override
  String get diagnosticsGameRunning => 'running';

  @override
  String get diagnosticsNamsPresent => 'NAMS.exe';

  @override
  String diagnosticsMissingFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count missing files',
      one: '1 missing file',
    );
    return '$_temp0';
  }

  @override
  String get diagnosticsInstalled => 'installed';

  @override
  String get diagnosticsEnabled => 'enabled';

  @override
  String get diagnosticsShadersMissing => 'shaders missing';

  @override
  String get diagnosticsTexturePacksUnavailable =>
      'NAMS texture query unavailable';

  @override
  String get diagnosticsExtraFile => 'extra';

  @override
  String diagnosticsFileCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count files',
      one: '1 file',
    );
    return '$_temp0';
  }

  @override
  String diagnosticsMoreItems(int count) {
    return '... $count more';
  }

  @override
  String get refreshButton => 'Refresh';

  @override
  String get tabModloaderLabel => 'Modloader';

  @override
  String get tabYorhaLabel => 'YoRHa Protocol';

  @override
  String get configEditorTitle => 'CONFIG EDITOR';

  @override
  String get changelogTitle => 'WHAT\'S NEW';

  @override
  String get tipDragTextures =>
      'Drag texture mods directly into the Textures tab';

  @override
  String get tipHdCutscenes =>
      'HD cutscene mods are auto-detected and configured';

  @override
  String get tipLodModPreviews =>
      'LOD Mod Ext settings come with before/after preview images';

  @override
  String get tipFaqButton =>
      'Use the FAQ button to see which mods YoRHa Protocol replaces';

  @override
  String get tipReShadeAuto =>
      'ReShade is auto-detected - no manual config needed';

  @override
  String get tipFreecam =>
      'YoRHa Protocol includes a built-in freecam with save slots';

  @override
  String get tipCustomQuests => 'Custom quests are coming soon - stay tuned';

  @override
  String get sectionNams => 'NAMS';

  @override
  String get sectionTextureInjection => 'TEXTURE INJECTION';

  @override
  String get sectionLodMod => 'LOD MOD EXT';

  @override
  String get sectionLevelOfDetail => 'LEVEL OF DETAIL';

  @override
  String get sectionAmbientOcclusion => 'AMBIENT OCCLUSION';

  @override
  String get sectionShadows => 'SHADOWS';

  @override
  String get sectionPostProcessing => 'POST-PROCESSING';

  @override
  String get labelValidateModelData => 'Validate Model Data';

  @override
  String get tooltipValidateModelData =>
      'The game validates model data when loading. Normally it silently fails and continues with broken data, which can cause invisible models or glitches. When enabled, NAMS surfaces the validation result as a dialog so you can see exactly which model failed and why.';

  @override
  String get labelPreloadMaxDimension => 'Preload Max Dimension';

  @override
  String get tooltipPreloadMaxDimension =>
      'Max texture size to preload into RAM at startup. 2048 = default, 4096 = preload up to 4K textures, 16384 = preload everything. Higher = longer loading but less stutter in-game.';

  @override
  String get labelPreloadAllTextures => 'Preload All Textures';

  @override
  String get tooltipPreloadAllTextures =>
      'Preload ALL textures into RAM regardless of size. Eliminates all texture pop-in stutter but needs 32GB+ RAM and makes startup significantly slower.';

  @override
  String get labelEnableLodMod => 'Enable LodMod Ext';

  @override
  String get tooltipEnableLodMod =>
      'Master toggle for all LodMod Ext visual patches/rewrites.';

  @override
  String get labelLodMultiplier => 'LOD Multiplier';

  @override
  String get tooltipLodMultiplier =>
      'Controls when models switch to their lower detail levels. 0 = LODs disabled (best quality, no pop-in). 0.75 = the lower detail levels take over closer to the camera, which costs less GPU; the distance at which models vanish stays vanilla. 1 = vanilla. 10 = high detail far away, helps reduce AO bleed.';

  @override
  String get lodMultiplierOptionQuality => '0 - Best quality';

  @override
  String get lodMultiplierOptionPerformance => '0.75 - Performance';

  @override
  String get lodMultiplierOptionVanilla => '1 - Vanilla';

  @override
  String get lodMultiplierOptionFar => '10 - Detail far away';

  @override
  String get labelDisableManualCulling => 'Disable Manual Culling';

  @override
  String get tooltipDisableManualCulling =>
      'Prevents models/geometry from randomly disappearing at certain distances or camera angles. Fixes things like the mall interior vanishing after crossing the bridge, buildings outside camp disappearing, etc. Rare ugly LOD models that would show up are filtered out.';

  @override
  String get labelDisableCameraCulling => 'Disable Camera Culling';

  @override
  String get tooltipDisableCameraCulling =>
      'Stops grass, bushes and small objects from disappearing and reappearing while you move the camera. The map hides some of them whenever the camera is in certain spots, because from there they were thought to be out of sight. With a longer view distance, for example with High-Res Map Grids or a larger Object Spawn Range, they are no longer out of sight, so you see them pop in and out. Can cost frame rate in areas with a lot of grass. Takes effect live.';

  @override
  String get labelLayoutSpawnRange => 'Object Spawn Range';

  @override
  String get tooltipLayoutSpawnRange =>
      'How far away from you grass, bushes, rubble and other small map objects appear. 100 m = vanilla. Lower values save frame rate, but these objects then appear closer to you. Higher values fill in the landscape further out, but cost frame rate in areas with a lot of grass. Applies to objects that have not appeared yet; objects that are already there stay until that part of the map unloads.';

  @override
  String get labelLayoutSpawnEverything => 'Spawn All Map Objects';

  @override
  String get tooltipLayoutSpawnEverything =>
      'Places every patch of grass, bush and small map object as soon as its part of the map is loaded, no matter how far away you are. Ignores Object Spawn Range. The objects still appear one after another, so a freshly loaded area fills in gradually. Costs frame rate in areas with a lot of grass.';

  @override
  String get labelAoWidth => 'AO Width';

  @override
  String get tooltipAoWidth =>
      'Multiplier for AO horizontal resolution. Vanilla AO runs at 1/4 screen res. 2.0 = half screen res (crisper AO but heavy). 1.5 is a good balance. Range: 0.1 - 2.0. Setting only one axis to 2 can be a lighter alternative.';

  @override
  String get labelAoHeight => 'AO Height';

  @override
  String get tooltipAoHeight =>
      'Multiplier for AO vertical resolution. Vanilla AO runs at 1/4 screen res. 2.0 = half screen res (crisper AO but heavy). 1.5 is a good balance. Range: 0.1 - 2.0. Both at 2.0 can cost ~10 FPS in worst case.';

  @override
  String get labelShadowResolution => 'Shadow Resolution';

  @override
  String get tooltipShadowResolution =>
      'Shadow map texture size. Higher = sharper shadows but heavier on GPU. 2048 = vanilla, 4096 = good upgrade, 8192 = very sharp. Must be power of 2. Sharpness depends on both resolution and distance (larger distance = more area to fit, so quality decreases).';

  @override
  String get labelDistanceMultiplier => 'Distance Multiplier';

  @override
  String get tooltipDistanceMultiplier =>
      'Multiplies the per-scene shadow draw distance. 2.0 = shadows visible twice as far. Vanilla: 1.0. Disable Min/Max below for this to work properly, or use them to restrict the range this multiplier sets.';

  @override
  String get labelDistanceMinimum => 'Distance Minimum';

  @override
  String get tooltipDistanceMinimum =>
      'Minimum shadow draw distance clamp. 0 = off (no minimum). Setting to ~70 with 8192 resolution matches vanilla quality while greatly increasing shadow distance.';

  @override
  String get labelDistanceMaximum => 'Distance Maximum';

  @override
  String get tooltipDistanceMaximum =>
      'Maximum shadow draw distance clamp. 0 = off (no maximum). Only worth setting if the default game distances cause performance issues.';

  @override
  String get labelDistancePss => 'Distance PSS';

  @override
  String get tooltipDistancePss =>
      'Enables PSS shadow distribution for more even shadow quality. 0 = off. Good values: 0.5 - 0.9. Looks great in some areas but may appear blurry in others. Should be set much larger than other distance values (~1500 for large open areas).';

  @override
  String get labelFilterStrengthBias => 'Filter Strength Bias';

  @override
  String get tooltipFilterStrengthBias =>
      'Adjusts shadow blur filter strength per-scene. 0 = off. -1 = sharper shadows. Positive = softer. Different areas use different strengths (forest = softer). Can be combined with Min/Max to restrict the range.';

  @override
  String get labelFilterStrengthMin => 'Filter Strength Min';

  @override
  String get tooltipFilterStrengthMin =>
      'Forces a minimum shadow filter strength across all areas. 0 = off. Game default varies per scene (usually ~4). Use to prevent shadows from being too sharp in any area.';

  @override
  String get labelFilterStrengthMax => 'Filter Strength Max';

  @override
  String get tooltipFilterStrengthMax =>
      'Forces a maximum shadow filter strength across all areas. 0 = off. Game default varies per scene (usually ~4). Use to prevent shadows from being too blurry in any area.';

  @override
  String get labelHqShadowModels => 'HQ Shadow Models';

  @override
  String get tooltipHqShadowModels =>
      'Uses real-time HQ models for shadows instead of static LQ models. Tree shadows will sway with the wind instead of being frozen. Experimental - works well in city ruins, could cause issues in rare areas.';

  @override
  String get labelForceAllShadowModels => 'Force All Shadow Models';

  @override
  String get tooltipForceAllShadowModels =>
      'Forces all models to cast shadows, including small objects like rocks and grass. Experimental - may rarely cause invisible models to cast shadows. No issues noticed so far.';

  @override
  String get labelDisableVignette => 'Disable Vignette';

  @override
  String get tooltipDisableVignette =>
      'Removes the dark vignette effect on screen edges. Some loading screens may still have it baked into textures.';

  @override
  String get configAppliesOnRestart => 'Applies on restart';

  @override
  String get configAppliesLive => 'Applies instantly (live)';

  @override
  String get dropZoneBrowseFolder => 'Or pick a folder';

  @override
  String get labelGiEnabled => 'Enable Global Illumination';

  @override
  String get tooltipGiEnabled =>
      'What FAR calls global illumination: every frame a per-pixel pass walks all loaded reflection cubemaps to pick the nearest ones. With this on, only the cubemaps nearest to the camera stay in that walk, the rest are skipped. FAR\'s version cuts the list in load order, so cubemaps around you can be dropped and nearby surfaces go dull and dark. This one keeps the ones closest to you, so what you see up close keeps its light and reflections while the pass still gets cheaper. Only far reflections get a coarser cubemap. Takes effect live.';

  @override
  String get labelGiWorkgroupSize => 'Reflection cubemaps kept';

  @override
  String get tooltipGiWorkgroupSize =>
      'How many reflection cubemaps nearest to the camera stay in the per-pixel walk. 128 = everything vanilla keeps, 64/32/16 = progressively cheaper, far reflections get coarser.';

  @override
  String get labelGiMinLightExtent => 'GI Min Light Extent';

  @override
  String get tooltipGiMinLightExtent =>
      'Cull small distant lights from GI. 0.0 = no culling (all lights contribute), 0.5 = aggressive culling. Range 0.0-1.0.';

  @override
  String get cardExperimental => 'EXPERIMENTAL';

  @override
  String get cardBloom => 'BLOOM';

  @override
  String configParseErrorTitle(String file) {
    return '$file could not be read';
  }

  @override
  String get configParseErrorBody =>
      'The file is not valid TOML. The launcher is showing default values and will not write to this file until it parses again. Fix it in a text editor and save - the launcher picks it up on its own.';

  @override
  String get subheadingHighGrids => 'HIGH-RES MAP GRIDS';

  @override
  String get highGridsWipTitle => 'Work in progress';

  @override
  String get highGridsWipBody =>
      'This feature is not finished. The culling rules for the higher ring counts still have to be written, so far grids can show map parts that belong somewhere else.';

  @override
  String get highGridsWipContribute =>
      'Contributions welcome: the rule lists below are the part that needs filling in. Meshes and grid ids can be picked directly in-game with the YP Devkit (Map Manager -> Mesh Picker) and written straight into a rule.';

  @override
  String get subheadingFrameRate => 'FRAME RATE';

  @override
  String get subheadingRendering => 'RENDERING';

  @override
  String get bloomReferenceVanilla => 'Vanilla';

  @override
  String get bloom2017Button => 'Set 2017 bloom';

  @override
  String get bloom2017Tooltip =>
      'Sets Bloom Reference Height to 900, the 2017 release\'s bloom footprint. Bloom then keeps that footprint at any resolution instead of scaling with it and shimmering at high res. Leaves Bloom Blur Width alone. Applies live.';

  @override
  String get labelBloomReferenceHeight => 'Bloom Reference Height';

  @override
  String get tooltipBloomReferenceHeight =>
      'Fixes bloom shimmering at high resolutions. 0 = vanilla 2021 (the bloom pyramid scales with your resolution, which makes bright edges shimmer). 900 = the 2017 bloom footprint at any resolution. Range 360-4320.';

  @override
  String get labelBloomKernelReferenceHeight => 'Bloom Blur Width';

  @override
  String get tooltipBloomKernelReferenceHeight =>
      'Reference height the blur width is scaled against. 0 = vanilla (the blur shrinks relative to the screen at high resolutions). Any other value keeps the blur width fixed to that height at any resolution. Keeps render targets at full size, so it can be used instead of Bloom Reference Height. Range 360-4320.';

  @override
  String get labelBloomExtraBlur => 'Extra Bloom Blur';

  @override
  String get tooltipBloomExtraBlur =>
      'Extra blur rounds that widen and brighten the glow, in 0.1 steps. 0 = vanilla.';

  @override
  String get bloomDropLevelsOff => 'Off';

  @override
  String get bloomDropLevelsWidest => 'Widest layer';

  @override
  String get bloomDropLevelsTwoWidest => 'Two widest layers';

  @override
  String get labelBloomDropCoarseLevels => 'Remove Wide Bloom Layers';

  @override
  String get tooltipBloomDropCoarseLevels =>
      'Removes the shimmery, flickering bloom layers while keeping the small glow around light sources. 0 = vanilla, 1 = removes the widest layer, 2 = removes the two widest layers. Based on Void\'s Bloom Fix.';

  @override
  String get labelHighGridsEnabled => 'Enable High-Res Map Grids';

  @override
  String get tooltipHighGridsEnabled =>
      'Keeps more of the map loaded in high resolution around you. Vanilla only keeps 7 grids loaded and renders everything else as low-res terrain. Costs memory, loading time and frame rate, since the extra grids are also rendered. Work in progress: the culling rules are incomplete. Turning this on also turns on Disable Manual Culling, and turning it off turns that back off. It also turns on Disable Camera Culling if it is not on yet.';

  @override
  String get labelHighGridsRings => 'Grid Rings';

  @override
  String get tooltipHighGridsRings =>
      'How many hex rings of high-res grids are kept loaded around you. 1 = vanilla (7 grids), 2 = 19, 3 = 37, 4 = 61. Each grid costs roughly 100 MB of heap, so higher values need much more memory and load longer. They also cost frame rate, since all of that geometry is drawn. 4 is the tested value.';

  @override
  String get labelHighGridsRoomRings => 'Per-Room Ring Overrides';

  @override
  String get tooltipHighGridsRoomRings =>
      'Use fewer rings in specific rooms. Closed areas like the Flooded City show map parts from elsewhere when too much is loaded around you. Cannot exceed the global ring count.';

  @override
  String get labelHighGridsBlockedInRoom => 'Grids Blocked In Room';

  @override
  String get tooltipHighGridsBlockedInRoom =>
      'Grids that never load while you are in a room. Use when a specific grid is visible from somewhere it was never meant to be seen from. The grid id is the low 4 hex digits of a grid number, so it matches in every story phase.';

  @override
  String get labelHighGridsBlockedFromGrid => 'Grids Blocked From Grid';

  @override
  String get tooltipHighGridsBlockedFromGrid =>
      'Grids that never load while you stand on one specific grid. Use when blocking the grid for the whole room would be too broad.';

  @override
  String get labelContentEffectAreas => 'Effect areas';

  @override
  String get tooltipContentEffectAreas =>
      'Effects a mod places in a room. Changes show after re-entering the room.';

  @override
  String get labelSkipStartupLogos => 'Skip startup logos';

  @override
  String get tooltipSkipStartupLogos =>
      'Skip the Platinum Games / Square Enix logo movies while the game loads. Loading itself is untouched, and the game already lets you skip them with a button press.';

  @override
  String get labelShadowCascades => 'Shadow cascades';

  @override
  String get tooltipShadowCascades =>
      '4 = vanilla, 8 adds four far cascades (experimental). With 8 the shadow atlas is tiled 4x4, so each cascade gets a quarter of the shadow resolution - raise it to compensate.';

  @override
  String get labelShadowCascadeRange => 'Cascade range';

  @override
  String get tooltipShadowCascadeRange =>
      'How far past the vanilla shadow distance the four extra cascades reach.';

  @override
  String get labelShadowBlurScale => 'Shadow blur per cascade';

  @override
  String get tooltipShadowBlurScale =>
      'Multiplier on the game\'s own blur, near to far. 1.0 = unchanged, 0.5 = half the blur for crisper contact shadows. Applies live.';

  @override
  String get labelDisableHdr => 'Disable HDR';

  @override
  String get tooltipDisableHdr =>
      'Play in SDR even on an HDR display, the same thing Special K\'s HideHDRSupport does. Turn this on if HDR looks washed out to you. Applies live, but the swap chain rebuild can take a moment and the frame rate may drop until it is done; if it does not apply, restart the game.';

  @override
  String get labelRenderScale => 'Render scale';

  @override
  String get tooltipRenderScale =>
      'Internal resolution (experimental). 2.0 renders the game at twice the display resolution and scales it back down, removing most aliasing and shimmer. 0.5 renders at half the display resolution and scales it up: softer picture, a quarter of the GPU work, for weak GPUs. GPU load follows the square of the factor. Works in every display mode. Applies live, but the swap chain rebuild can take a moment and the frame rate may drop until it is done; if it does not apply, restart the game.';

  @override
  String get labelFxaa => 'FXAA';

  @override
  String get tooltipFxaa =>
      'Cheap edge smoothing. Use with the in-game anti-aliasing set to Off. Applies live.';

  @override
  String get labelMsaaPrepassFix => 'MSAA black dot fix';

  @override
  String get tooltipMsaaPrepassFix =>
      'Fixes the black dots on leaves, grass and hair when the in-game anti-aliasing (MSAA) is on. No cost. Applies live.';

  @override
  String get labelMsaaShadowMaskFix => 'MSAA bright rim fix';

  @override
  String get tooltipMsaaShadowMaskFix =>
      'Removes the bright rim around shadowed characters and objects when the in-game anti-aliasing (MSAA) is on. No cost. Applies live.';

  @override
  String get subheadingMsaaPerformance =>
      'MSAA Performance Fixup (experimental)';

  @override
  String get labelMsaaPerPixel => 'Shade materials per pixel';

  @override
  String get tooltipMsaaPerPixel =>
      'With the in-game anti-aliasing (MSAA) on, the game shades every material once per sample, so 8x MSAA shades every model eight times. With this on, materials shade once per pixel and MSAA only smooths the edges. Needs the MSAA bright rim fix. Applies live.';

  @override
  String get labelMsaaPerPixelTerrain => 'Also the terrain materials';

  @override
  String get tooltipMsaaPerPixelTerrain =>
      'The biggest part of the gain. Wet ground can flicker bright for a moment, stronger than normal. If that does not bother you, this is the largest MSAA performance gain. Applies live.';

  @override
  String get labelConstantBufferDedup => 'Skip unchanged shader parameters';

  @override
  String get tooltipConstantBufferDedup =>
      'Increases performance. Every draw reads a small block of parameters — transforms, material settings, light values. Vanilla re-sends all of them to the graphics card every frame, several thousand uploads, even when nothing in them changed. Vanilla already contains the comparison that would skip an unchanged block, but it never reaches it. About half the uploads turn out to be redundant. Applies live.';

  @override
  String get labelConstantBufferUploadOnBind =>
      'Upload shader parameters only when drawn';

  @override
  String get tooltipConstantBufferUploadOnBind =>
      'Increases performance. Vanilla sends every shader parameter block to the graphics card each frame, including the ones nothing on screen uses — roughly four out of five. With this on, a block is only sent when a draw actually binds it. Applies live.';

  @override
  String get labelDecreaseStutterDuringGridLoading =>
      'Less stutter while the map loads';

  @override
  String get tooltipDecreaseStutterDuringGridLoading =>
      'Smoother map loading. When a new part of the map loads around you, vanilla builds everything for it in one go, which shows as a hitch every time. With this on, that work is spread over several frames and prepared in the background, so most of those hitches disappear. Work in progress, more improvements will follow in future updates. You will still notice small hitches here and there while the map loads, for example when effects or new objects appear. Do not expect a higher frame rate, faster loading, or stutter-free loading with 4K texture packs. Applies live.';

  @override
  String get labelAoFadeFix => 'AO through walls fix';

  @override
  String get tooltipAoFadeFix =>
      'Stops ambient occlusion (AO) from showing through walls and jumping when a model switches LOD. During a LOD cross-fade the game draws both LOD versions solid into the AO depth, so AO appears for geometry you can barely see. This skips that extra draw. LOD multiplier 0 also hides the bug because there are no LOD switches at all; this fix removes the cause and keeps LODs on. No cost. Applies live.';

  @override
  String get labelHighGridsHiddenMeshes => 'Hidden meshes';

  @override
  String get tooltipHighGridsHiddenMeshes =>
      'Low-detail stand-in meshes to hide once enough rings are loaded. Find the names with the mesh picker in the YP Devkit. Grid 0x0 applies to every grid.';

  @override
  String get labelTextureHotReload => 'Hot reload';

  @override
  String get tooltipTextureHotReload =>
      'Watch the texture folders and apply changed .dds files while the game runs. Same-size changes update instantly.';

  @override
  String get gridRuleMesh => 'Mesh';

  @override
  String get gridRuleFromRings => 'From rings';

  @override
  String shadowBlurCascade(int n) {
    return 'Cascade $n';
  }

  @override
  String get gridRuleRoom => 'Room';

  @override
  String get gridRuleRings => 'Rings';

  @override
  String get gridRuleGrid => 'Grid';

  @override
  String get gridRuleFromGrid => 'Standing on';

  @override
  String get gridRuleAdd => 'Add rule';

  @override
  String get gridRuleEmpty => 'No rules - every grid loads normally.';

  @override
  String highGridsCellCount(int rings, int cells) {
    String _temp0 = intl.Intl.pluralLogic(
      rings,
      locale: localeName,
      other: '$rings rings - $cells grids',
      one: '1 ring - 7 grids (vanilla)',
    );
    return '$_temp0';
  }

  @override
  String get lodModResetButton => 'Reset to defaults';

  @override
  String get lodModResetConfirmTitle => 'Reset LodMod Ext settings?';

  @override
  String get lodModResetConfirmBody =>
      'This will reset every LodMod Ext field on this tab to its default value. Your current values will be overwritten. Continue?';

  @override
  String get lodModResetConfirmAction => 'Reset';

  @override
  String get lodModResetToast => 'LodMod Ext settings reset to defaults';

  @override
  String get experimentalWarningBody =>
      'These are experimental. Think twice before using them on a first playthrough, so they do not affect your experience.';

  @override
  String get labelFpsUncapInMenus => 'Uncap FPS in menus / loading';

  @override
  String get tooltipFpsUncapInMenus =>
      'Removes the 60 FPS lock during menus and loading screens. Loading feels faster and menu animations get smoother. Safe: gameplay is unaffected. Applies live.';

  @override
  String get labelFpsUncapInGameplay => 'Uncap FPS in gameplay';

  @override
  String get tooltipFpsUncapInGameplay =>
      'Removes the 60 FPS lock during gameplay. WARNING: NieR:Automata\'s physics, animations, and cutscene timing are tied to the 60 FPS lock. Uncapping causes broken physics (jumping height, dodge i-frames), animation speed changes, audio desync in cutscenes, and softlocks in scripted sequences. Use only if you know exactly what trade-offs you\'re accepting. Applies live.';

  @override
  String get labelFpsLimit => 'FPS Limit';

  @override
  String get labelFpsCapInHacking => 'Vanilla cap in hacking';

  @override
  String get tooltipFpsCapInHacking =>
      'Drop back to the vanilla 60 FPS cap while the hacking minigame runs, even with the gameplay uncap on. The minigame\'s timing assumes 60 FPS. Applies live.';

  @override
  String get labelFpsCapInEvents => 'Vanilla cap in events';

  @override
  String get tooltipFpsCapInEvents =>
      'Drop back to the vanilla 60 FPS cap while a cutscene, event, movie or caption is running. Event scripts assume 60 FPS and can soft-lock otherwise. Applies live.';

  @override
  String get tooltipFpsLimit =>
      'FPS cap applied when uncap is active. 0 = unlimited. Otherwise 1-1000 (NAMS clamps values out of range). 30 sets the same frame time the game\'s own 30 fps mode uses. While uncap is on the wait is a busy-wait, so the lower the cap, the longer a CPU core spins each frame. Tip: capping at half your monitor\'s refresh rate gives smoother motion than vanilla 60 (e.g. 72 on 144Hz, 82 on 165Hz, 120 on 240Hz).';

  @override
  String get tutorialValidateModel =>
      'Shows you when a mod\'s model is broken instead of silently failing.';

  @override
  String get labelValidateScripts => 'Validate Scripts';

  @override
  String get tooltipValidateScripts =>
      'Show script errors as dialog instead of silently crashing.';

  @override
  String get previewValidationDialog => 'VALIDATION DIALOG';

  @override
  String get previewScriptErrorDialog => 'SCRIPT ERROR DIALOG';

  @override
  String get labelLoadingStallHints => 'Loading Stall Hints';

  @override
  String get tooltipLoadingStallHints =>
      'Show escalating hints when the loading screen takes too long. Helps identify missing or deleted mod files.';

  @override
  String get labelFixWindTimerBug => 'Fix Wind Timer Bug';

  @override
  String get tooltipFixWindTimerBug =>
      'Fixes a vanilla bug where wind animation stops after max playtime. Uses the game\'s speed factor instead.';

  @override
  String get labelDisablePluginLoading => 'Disable Plugin Loading';

  @override
  String get tooltipDisablePluginLoading =>
      'Skip loading all plugin DLLs (e.g. YoRHa Protocol workspace). All NAMS engine features still work.';

  @override
  String get labelDisableContentFeatures => 'Disable Content Features';

  @override
  String get tooltipDisableContentFeatures =>
      'Master switch for all content-layer features. When on, NAMS runs as a pure engine layer (mouse fixes, validation, heap tuning, crash fixes) without any item / weapon / outfit / quest / accessory mod support. Useful for benchmarking or isolating engine vs content issues.';

  @override
  String get labelContentItems => 'Items / Weapons / Shops';

  @override
  String get tooltipContentItems =>
      'Custom items, weapons, outfits and shop entries. Disable to play without any item-related mods. Requires restart.';

  @override
  String get labelContentAccessories => 'Accessories';

  @override
  String get tooltipContentAccessories =>
      'Custom accessories (face masks, lunar tear, masamune mask, etc.) and the accessory equip / unequip flow. Disable to play without accessory mods. Requires restart.';

  @override
  String get labelContentAssembleMeshes => 'Player Meshes';

  @override
  String get tooltipContentAssembleMeshes =>
      'Custom player meshes (mesh swaps, hair / outfit / mask overrides). Disable to render the vanilla player models unchanged. Requires restart.';

  @override
  String get labelContentQuestIntegration => 'Quests / Mails / Voice';

  @override
  String get tooltipContentQuestIntegration =>
      'Custom quests, custom mails, custom voice lines, and the quest UI integration that activates them. Disable to play without quest mods. Requires restart.';

  @override
  String get labelContentEffectsApplier => 'Effect Rules';

  @override
  String get tooltipContentEffectsApplier =>
      'Per-frame application of weapon / outfit effect rules to player stats (damage multipliers, dodge tweaks, immunities, etc.).';

  @override
  String get labelContentEquipTracker => 'Equip Tracker';

  @override
  String get tooltipContentEquipTracker =>
      'Detection of weapon equip / unequip changes. Drives effect rules and equip-time SDK callbacks.';

  @override
  String get labelContentMcd => 'Custom Text';

  @override
  String get tooltipContentMcd =>
      'In-game text customization (custom item names, descriptions, dialogue strings supplied by mods).';

  @override
  String get labelContentBuddyRubySelector =>
      'Buddy Outfit Selector (experimental)';

  @override
  String get tooltipContentBuddyRubySelector =>
      'Patches the global buddy conversation script to add a \"Change outfit\" entry that lists modded buddy outfits. Disable if the patched conversation script causes instability or interferes with other script mods.';

  @override
  String get cardContentFeatures => 'CONTENT FEATURES';

  @override
  String get contentFeaturesDescription =>
      'Per-feature toggles for the content layer. All default to ON. Useful for narrowing a problem down to a specific subsystem. Requires a game restart.';

  @override
  String get labelDisableReShadeLoading => 'Disable ReShade Loading';

  @override
  String get tooltipDisableReShadeLoading =>
      'Skip automatic ReShade DLL detection from the reshade/ folder and does not load it anymore.';

  @override
  String get labelDisable3dmigotoLoading => 'Disable 3DMigoto Loading';

  @override
  String get tooltipDisable3dmigotoLoading =>
      'Skip loading the 3DMigoto runtime from thirdparty/3dmigoto/. Turn off to stop loading shader mods.';

  @override
  String get labelDisableGameModsLoading => 'Disable Game Mod Loading';

  @override
  String get tooltipDisableGameModsLoading =>
      'Stop loading the mods listed in the Third Party tab under Game Mods, such as speedrun timers and trainers. Turn this on to play without them without having to delete anything.';

  @override
  String get labelDisableTextureInjection => 'Disable Texture Injection';

  @override
  String get tooltipDisableTextureInjection =>
      'Skip texture injection from the mods folder. Useful for isolating issues or if you don\'t want to use texture mods even though they are installed.';

  @override
  String get labelHideSubtitleOverlay => 'Hide Subtitle Overlay';

  @override
  String get tooltipHideSubtitleOverlay =>
      'Hide the black bars and caption overlay shown during cutscenes. Only affects cutscenes - menus, item descriptions and normal dialogue stay untouched.';

  @override
  String get labelKeepSubtitleText => 'Keep Subtitle Text';

  @override
  String get tooltipKeepSubtitleText =>
      'Remove only the dark backdrop behind the captions and keep the text itself readable. Requires Hide Subtitle Overlay.';

  @override
  String get labelHideSubtitleInEvents =>
      'Also Hide In Event Scenes (Experimental)';

  @override
  String get tooltipHideSubtitleInEvents =>
      'EXPERIMENTAL. Also apply during in-engine event scenes, not just pre-rendered movies. May miss some scenes or hide text you wanted to keep - turn off if anything looks wrong. Requires Hide Subtitle Overlay.';

  @override
  String get labelOutfitSwapVisualEffects => 'Outfit Swap Visual Effects';

  @override
  String get tooltipOutfitSwapVisualEffects =>
      'Play the visual effects during an outfit hot-swap: the pod spawn-in blinder animation, the curtain, and the hacking-screen glitch filter. Turn off for an instant, effect-free swap - the model still reloads. Takes effect immediately, no restart needed.';

  @override
  String get labelExperimentalDefaultOutfits =>
      'Default Outfits (experimental)';

  @override
  String get tooltipExperimentalDefaultOutfits =>
      'Lets you mark installed outfit mods as active from the moment the game starts, as if their files were placed in the game\'s data folder. When on, the mod details panel shows a star button per player model to set it as the boot default. Off by default while the feature stabilizes. Requires a game restart.';

  @override
  String get labelSolidLetterboxBars => 'Solid Letterbox Bars';

  @override
  String get tooltipSolidLetterboxBars =>
      'Draw the cutscene letterbox as opaque black bars of equal height, instead of the vanilla translucent and uneven ones (76 top, 154 bottom). Works on its own, and takes precedence over Hide Subtitle Overlay when both are on.';

  @override
  String get labelDisableSplashScreen => 'Disable Splash Screen';

  @override
  String get tooltipDisableSplashScreen =>
      'Skip the startup splash window shown while the game loads. The original game revealed its window before it was ready, causing resize and flicker artifacts; NAMS finished the splash so the window is only revealed once ready. Turning this on brings those vanilla startup artifacts back.';

  @override
  String get labelDisableDebugHotkeys => 'Ignore Developer Hotkeys';

  @override
  String get tooltipDisableDebugHotkeys =>
      'The retail build still contains the developer hotkeys Shift+3, Shift+6, Shift+K and Shift+L. They reset the render scale and force every render target to a fixed 1600x900 instead of your display resolution, which rebuilds the whole post-processing chain and leaves the image looking wrong until you restart. On by default. Turn off to get the vanilla debug keys back.';

  @override
  String get labelDisableInputFeatures => 'Disable Input Features';

  @override
  String get tooltipDisableInputFeatures =>
      'Turn off the whole mouse and keyboard layer. The game keeps its stock input handling and everything below is ignored. Requires restart.';

  @override
  String get tooltipValidateModelDataSettings =>
      'Surfaces model validation errors as a dialog instead of silent failure.';

  @override
  String get heapDefault => 'Default';

  @override
  String get heapPlus16MB => '+16 MB';

  @override
  String get heapPlus32MB => '+32 MB';

  @override
  String get heapPlus64MB => '+64 MB';

  @override
  String get heapPlus128MB => '+128 MB';

  @override
  String get heapPlus256MB => '+256 MB';

  @override
  String heapCustomMB(String mb) {
    return '+$mb MB';
  }

  @override
  String get heapScriptEngine => 'Script Engine';

  @override
  String get heapScriptEngineDesc => 'For complex script mods (mRuby / HAP).';

  @override
  String get heapPlayerModels => 'Player Models';

  @override
  String get heapPlayerModelsDesc => 'For large player model replacement mods.';

  @override
  String get heapPlayerTextures => 'Player Textures';

  @override
  String get heapPlayerTexturesDesc => 'For high-res player texture mods.';

  @override
  String get heapEnemyBgModels => 'Enemy/BG Models';

  @override
  String get heapEnemyBgModelsDesc => 'For enemy or environment model mods.';

  @override
  String get heapEnemyBgTextures => 'Enemy/BG Textures';

  @override
  String get heapEnemyBgTexturesDesc =>
      'For high-res enemy/environment textures.';

  @override
  String get tutorialLodModEnable =>
      'Turn this on for better visuals. This is the most impactful setting.';

  @override
  String get tutorialLodModShadowRes =>
      'Higher values mean sharper shadows. 8192 is recommended.';

  @override
  String get tutorialLodModComparison =>
      'Click any comparison to see the difference fullscreen.';

  @override
  String get comparisonVanilla => 'VANILLA';

  @override
  String get comparisonDefaultEnabled => 'DEFAULT (ENABLED)';

  @override
  String get comparisonAo05x => 'AO 0.5x';

  @override
  String get comparisonAo20x => 'AO 2.0x';

  @override
  String get comparisonVignetteOn => 'VIGNETTE ON';

  @override
  String get comparisonVignetteOff => 'VIGNETTE OFF';

  @override
  String get comparison2048 => '2048';

  @override
  String get comparison8192 => '8192';

  @override
  String get comparisonCubemaps16 => '16 cubemaps';

  @override
  String get comparisonCubemaps128 => '128 cubemaps';

  @override
  String get comparisonDefault => 'DEFAULT';

  @override
  String get comparison20x => '2.0x';

  @override
  String get comparisonPssMinus5 => 'PSS -5.0';

  @override
  String get comparisonBiasMinus5 => 'BIAS -5.0';

  @override
  String get comparisonOff => 'OFF';

  @override
  String get comparisonOn => 'ON';

  @override
  String get comparisonBloom2021 => '2021';

  @override
  String get comparisonBloom2017 => '2017';

  @override
  String get comparisonHighGrids7 => '7 grids (vanilla)';

  @override
  String get comparisonHighGrids61 => '61 grids (4 rings)';

  @override
  String get comparisonCascades4 => '4 cascades (vanilla)';

  @override
  String get comparisonCascades8 => '8 cascades';

  @override
  String get comparison30 => '3.0';

  @override
  String get comparisonHqForceAll => 'HQ + FORCE ALL';

  @override
  String get tutorialKeybind =>
      'Click to change the keybind. Press any key to assign it.';

  @override
  String get tutorialDamageMultiplier =>
      'Tweak gameplay - stack damage, enable infinite HP, and more.';

  @override
  String get labelOpenWorkspace => 'Open Workspace';

  @override
  String get tooltipOpenWorkspace =>
      'Open the YoRHa Protocol workspace in-game.';

  @override
  String get labelFreezeGame => 'Freeze Game';

  @override
  String get tooltipFreezeGame =>
      'Freeze/unfreeze the game. Useful for screenshots and frame stepping.';

  @override
  String get labelMaxSpeed => 'Max Speed';

  @override
  String get tooltipMaxSpeed =>
      'Toggle maximum game speed for fast travel or testing.';

  @override
  String get labelFreeCam => 'Free Cam';

  @override
  String get tooltipFreeCam =>
      'Toggle free camera. Full keyboard/mouse and controller support.';

  @override
  String get labelPhaseJump => 'Phase Jump';

  @override
  String get tooltipPhaseJump =>
      'Jump to a preselected game phase/scene. Set the target in-game.';

  @override
  String get labelToggleInput => 'Toggle Input';

  @override
  String get tooltipToggleInput =>
      'Toggle game input on/off while the workspace is open.';

  @override
  String get labelAdvanceFrame => 'Advance Frame';

  @override
  String get tooltipAdvanceFrame =>
      'Step one frame forward while the game is frozen. Hold to advance faster.';

  @override
  String get labelDevMode => 'Dev Mode';

  @override
  String get tooltipDevMode =>
      'Toggle developer mode. Enables penetration/stress test buttons and debug tools.';

  @override
  String get labelWarpSave1 => 'Save Warp 1';

  @override
  String get tooltipWarpSave1 =>
      'Save the current position and rotation to warp slot 1.';

  @override
  String get labelWarpGoto1 => 'Warp to 1';

  @override
  String get tooltipWarpGoto1 =>
      'Teleport to the position saved in warp slot 1.';

  @override
  String get labelWarpSave2 => 'Save Warp 2';

  @override
  String get tooltipWarpSave2 =>
      'Save the current position and rotation to warp slot 2.';

  @override
  String get labelWarpGoto2 => 'Warp to 2';

  @override
  String get tooltipWarpGoto2 =>
      'Teleport to the position saved in warp slot 2.';

  @override
  String get labelGlobalKeybinds => 'Global Keybinds';

  @override
  String get tooltipGlobalKeybinds =>
      'Keybinds work while the workspace is closed.';

  @override
  String get labelLoadingSpeedup => 'Loading Speedup';

  @override
  String get tooltipLoadingSpeedup => 'Faster loading screens.';

  @override
  String get labelShaders => 'Shaders';

  @override
  String get tooltipShaders => 'Workspace shaders. Toggle off for performance.';

  @override
  String get labelSound => 'Sound';

  @override
  String get tooltipSound => 'Workspace UI interaction sound.';

  @override
  String get labelImpeller => 'Impeller Renderer (experimental)';

  @override
  String get tooltipImpeller =>
      'Render the overlay with Flutter\'s Impeller renderer instead of Skia. Off by default: several users hit crashes with it. Try it, and if the game is stable you can keep it for a faster overlay when you press F1. With it on, images in the overlay do not render, apart from the world map card. Takes effect on the next game start.';

  @override
  String get labelDamageMultiplier => 'Damage Multiplier';

  @override
  String get tooltipDamageMultiplier => '2.0 = double damage.';

  @override
  String get labelSyncEnemyLevels => 'Sync Enemy Levels';

  @override
  String get tooltipSyncEnemyLevels => 'Match enemy levels to yours.';

  @override
  String get labelInfiniteHealth => 'Infinite Health';

  @override
  String get tooltipInfiniteHealth => 'Take no damage.';

  @override
  String get labelInfiniteJump => 'Infinite Jump';

  @override
  String get tooltipInfiniteJump => 'Unlimited mid-air jumps.';

  @override
  String get labelNoPodCooldown => 'No Pod Cooldown';

  @override
  String get tooltipNoPodCooldown => 'Pod programs have no cooldown.';

  @override
  String get labelInfiniteAirDash => 'Infinite Air Dash';

  @override
  String get tooltipInfiniteAirDash => 'Unlimited mid-air dashes.';

  @override
  String get labelAutoStart => 'Auto Start';

  @override
  String get tooltipAutoStart => 'Auto-start randomizer on game launch.';

  @override
  String get labelGroundEnemies => 'Ground Enemies';

  @override
  String get tooltipGroundEnemies => 'Randomize ground-based spawns.';

  @override
  String get labelFlyingEnemies => 'Flying Enemies';

  @override
  String get tooltipFlyingEnemies => 'Randomize flying spawns.';

  @override
  String get labelAllowBigEnemies => 'Allow Big Enemies';

  @override
  String get tooltipAllowBigEnemies => 'Allow large enemies.';

  @override
  String get labelIncludeDlcEnemies => 'Include DLC Enemies';

  @override
  String get tooltipIncludeDlcEnemies => 'Include DLC enemies.';

  @override
  String get tutorialCameraAccel => 'Removes mouse acceleration for 1:1 input.';

  @override
  String get tutorialWipBanner =>
      'These features are coming in future NAMS updates.';

  @override
  String get labelFixCameraAcceleration => 'Fix Camera Acceleration';

  @override
  String get tooltipFixCameraAcceleration =>
      'Linear 1:1 mouse movement. Removes deadzone and acceleration curve from camera rotation.';

  @override
  String get labelSensitivity => 'Sensitivity';

  @override
  String get tooltipSensitivity =>
      'Camera sensitivity multiplier. Higher = faster rotation. 2.0 is a good default.';

  @override
  String get labelAimSensitivity => 'Aim Sensitivity';

  @override
  String get tooltipAimSensitivity =>
      'Aim sensitivity for top-down/side-scroll. 0.001 for ~3500 DPI, 0.003 for ~800 DPI.';

  @override
  String get labelAimOutputMultiplier => 'Aim Output Multiplier';

  @override
  String get tooltipAimOutputMultiplier =>
      'Raw multiplier for crosshair movement speed after normalization. Higher = faster crosshair. Most users won\'t need to change this.';

  @override
  String get labelDisablePodPet => 'Disable Pod Pet';

  @override
  String get tooltipDisablePodPet =>
      'Disable the pod petting animation triggered by mouse movement. Recommended.';

  @override
  String get labelDebugMenuKey => 'Debug Menu Key';

  @override
  String get tooltipDebugMenuKey =>
      'Opens the debug menu accessible after clearing the game. Usually requires a controller - this binding makes it possible with keyboard.';

  @override
  String get labelThirdPersonMode => 'Third-Person Camera Fix';

  @override
  String get tooltipThirdPersonMode =>
      'Raw mouse input for the third-person camera. Smooth, direct camera control that ignores the in-game mouse settings.';

  @override
  String get labelThirdPersonCharFollow => 'Camera Follows Character';

  @override
  String get tooltipThirdPersonCharFollow =>
      'Keep the game\'s automatic camera-follow while moving, like on a controller.';

  @override
  String get labelThirdPersonSmoothing => 'Camera Smoothing';

  @override
  String get tooltipThirdPersonSmoothing =>
      'How much the camera eases toward the mouse. 0 = it follows the mouse at once, 1 = the game\'s own smoothing.';

  @override
  String get labelThirdPersonSensX => 'Horizontal Sensitivity';

  @override
  String get tooltipThirdPersonSensX =>
      'Left/right camera speed. Negative value inverts the direction.';

  @override
  String get labelThirdPersonSensY => 'Vertical Sensitivity';

  @override
  String get tooltipThirdPersonSensY =>
      'Up/down camera speed. Negative value inverts the direction.';

  @override
  String get labelAimMode => 'Fix Pod Aiming';

  @override
  String get tooltipAimMode =>
      'Remove clamp and deadzone from pod/mech aiming in top-down and side-scroll views.';

  @override
  String get labelAimCrosshair => 'Crosshair Mode';

  @override
  String get tooltipAimCrosshair =>
      'Aim by pointing: the pod aims at a crosshair that follows your mouse, like a twin-stick shooter. The crosshair is built from the game\'s own UI elements, so it looks and feels like it was always part of NieR:Automata. Recommended.';

  @override
  String get labelAimCrosshairAlways => 'Always Show Crosshair';

  @override
  String get tooltipAimCrosshairAlways =>
      'Keep the crosshair visible even when not firing. Off = only shown while shooting.';

  @override
  String get naiomNeedsCrosshair => 'Turn on Crosshair Mode to use this';

  @override
  String get labelAimSensX => 'Aim Horizontal Sensitivity';

  @override
  String get tooltipAimSensX =>
      'Left/right aim speed multiplier. Negative value inverts the direction.';

  @override
  String get labelAimSensY => 'Aim Vertical Sensitivity';

  @override
  String get tooltipAimSensY =>
      'Up/down aim speed multiplier. Negative value inverts the direction.';

  @override
  String get labelDisableTapEvade => 'Disable Tap Evade';

  @override
  String get tooltipDisableTapEvade =>
      'Stop double-tapping movement keys from dodging. Only useful together with a dedicated Evade key.';

  @override
  String get labelCustomCursorMenu => 'Menu Cursor';

  @override
  String get tooltipCustomCursorMenu =>
      'Custom mouse cursor for the menus (.cur or .ani file). Empty = the bundled default cursor.';

  @override
  String get labelCustomCursorHacking => 'Hacking Cursor';

  @override
  String get tooltipCustomCursorHacking =>
      'Custom cursor for the hacking minigame. Empty = same as the menu cursor.';

  @override
  String get labelDisableDefaultCursor => 'Keep System Cursor';

  @override
  String get tooltipDisableDefaultCursor =>
      'Don\'t use the bundled cursor - keep the normal Windows cursor unless you picked your own file above.';

  @override
  String get labelBindMoveForward => 'Move Forward';

  @override
  String get tooltipBindMoveForward => 'Same as the in-game binding.';

  @override
  String get labelBindMoveBackward => 'Move Backward';

  @override
  String get tooltipBindMoveBackward => 'Same as the in-game binding.';

  @override
  String get labelBindMoveLeft => 'Move Left';

  @override
  String get tooltipBindMoveLeft => 'Same as the in-game binding.';

  @override
  String get labelBindMoveRight => 'Move Right';

  @override
  String get tooltipBindMoveRight => 'Same as the in-game binding.';

  @override
  String get labelBindJump => 'Jump';

  @override
  String get tooltipBindJump => 'Same as the in-game binding.';

  @override
  String get labelBindWalk => 'Walk';

  @override
  String get tooltipBindWalk => 'Hold to walk slowly.';

  @override
  String get labelBindAutoRun => 'Auto-Run';

  @override
  String get tooltipBindAutoRun =>
      'Keep running without holding the movement keys.';

  @override
  String get labelBindLightAttack => 'Light Attack';

  @override
  String get tooltipBindLightAttack => 'Same as the in-game binding.';

  @override
  String get labelBindHeavyAttack => 'Heavy Attack';

  @override
  String get tooltipBindHeavyAttack => 'Same as the in-game binding.';

  @override
  String get labelBindFire => 'Fire / Pod Dash';

  @override
  String get tooltipBindFire =>
      'Fires the pod. Together with Jump it performs the pod dash - also while Auto-Fire is on.';

  @override
  String get labelBindProgram => 'Use Program';

  @override
  String get tooltipBindProgram => 'Use the pod / flying-unit program.';

  @override
  String get labelBindLockOn => 'Lock-On';

  @override
  String get tooltipBindLockOn => 'Lock onto the current target.';

  @override
  String get labelBindUse => 'Use / Interact';

  @override
  String get tooltipBindUse => 'Same as the in-game binding.';

  @override
  String get labelBindSelfDestruct => 'Self-Destruct';

  @override
  String get tooltipBindSelfDestruct => 'Same as the in-game binding.';

  @override
  String get labelBindLight => 'Toggle Light';

  @override
  String get tooltipBindLight => 'Same as the in-game binding.';

  @override
  String get labelBindResetCamera => 'Reset Camera';

  @override
  String get tooltipBindResetCamera => 'Same as the in-game binding.';

  @override
  String get labelBindSwitchWeapon => 'Switch Weapon Set';

  @override
  String get tooltipBindSwitchWeapon => 'Cycle the equipped weapon sets.';

  @override
  String get labelBindNextProgram => 'Next Program';

  @override
  String get tooltipBindNextProgram => 'Select the next pod program.';

  @override
  String get labelBindPreviousProgram => 'Previous Program';

  @override
  String get tooltipBindPreviousProgram => 'Select the previous pod program.';

  @override
  String get labelBindMenuUp => 'Menu Up';

  @override
  String get tooltipBindMenuUp => 'Navigate up in menus.';

  @override
  String get labelBindMenuDown => 'Menu Down';

  @override
  String get tooltipBindMenuDown => 'Navigate down in menus.';

  @override
  String get labelBindMenuLeft => 'Menu Left';

  @override
  String get tooltipBindMenuLeft => 'Navigate left in menus.';

  @override
  String get labelBindMenuRight => 'Menu Right';

  @override
  String get tooltipBindMenuRight => 'Navigate right in menus.';

  @override
  String get labelBindMenuOpen => 'Open Menu';

  @override
  String get tooltipBindMenuOpen => 'Open the system menu.';

  @override
  String get labelBindMenuBack => 'Menu Back / Close';

  @override
  String get tooltipBindMenuBack =>
      'Go back in menus, or close them at the top level.';

  @override
  String get labelBindMenuEnter => 'Menu Enter / Skip Dialog';

  @override
  String get tooltipBindMenuEnter =>
      'Enter the selected sub-menu or skip dialog.';

  @override
  String get labelBindShortcutMenu => 'Shortcut Menu';

  @override
  String get tooltipBindShortcutMenu => 'Open the shortcut menu.';

  @override
  String get labelBindEvade => 'Evade (dedicated key)';

  @override
  String get tooltipBindEvade =>
      'Dodge in the current movement direction with a single key - no double-tap needed.';

  @override
  String get labelBindAutoFire => 'Auto-Fire Toggle';

  @override
  String get tooltipBindAutoFire =>
      'Toggle continuous pod fire on/off, so you don\'t have to hold the fire button.';

  @override
  String get labelBindNextItem => 'Next Item';

  @override
  String get tooltipBindNextItem =>
      'Switch to the next quick-item instantly. Works silently in the background - no item menu appears in-game, that\'s intended.';

  @override
  String get labelBindPreviousItem => 'Previous Item';

  @override
  String get tooltipBindPreviousItem =>
      'Switch to the previous quick-item instantly. Works silently in the background - no item menu appears in-game, that\'s intended.';

  @override
  String get labelBindUseItem => 'Use Item';

  @override
  String get tooltipBindUseItem =>
      'Use the selected quick-item instantly. Works silently in the background - no item menu appears in-game, that\'s intended.';

  @override
  String get labelBindThirdPersonToggle => 'Camera Fix Toggle';

  @override
  String get tooltipBindThirdPersonToggle =>
      'Turn the third-person camera fix on/off while playing.';

  @override
  String get labelBindAimToggle => 'Aim Fix Toggle';

  @override
  String get tooltipBindAimToggle =>
      'Turn the pod aiming fix on/off while playing.';

  @override
  String get keybindUnbound => 'Not bound';

  @override
  String keybindConflict(String other) {
    return 'Also used by: $other';
  }

  @override
  String get keybindMouseNotSupported =>
      'Mouse buttons don\'t work for this action - it needs a keyboard key.';

  @override
  String get naiomResetConfirmTitle => 'Reset NAIOM settings?';

  @override
  String get naiomResetConfirmBody =>
      'This will reset every camera, aiming, cursor and keybind setting on this tab to its default value. Nothing is written until you press Save, so you can still discard afterwards. Continue?';

  @override
  String get naiomControllerNote =>
      'Playing with a controller? These settings are designed for mouse and keyboard, but some of them - especially the camera and aiming fixes - also affect controller input. If you switch back to playing on a controller, disable those settings first to restore the original gamepad feel.';

  @override
  String get cardCheatEngine => 'CHEAT ENGINE';

  @override
  String get cheatTableConvertDesc =>
      'Got a Cheat Engine table (.CT) that does not work with NAMS? Fix it here. The fixed copy is saved next to your original file.';

  @override
  String get cheatTableConvertButton => 'Fix cheat table...';

  @override
  String cheatTableConvertSuccess(String file) {
    return 'Fixed! Saved as $file';
  }

  @override
  String get cheatTableConvertNone =>
      'This table already works with NAMS - nothing to fix.';

  @override
  String get cheatTableConvertError =>
      'Could not fix this table. Make sure the file is a valid .CT file.';

  @override
  String get naiomBetaBadge => 'BETA';

  @override
  String get naiomRestartBadge => 'RESTART';

  @override
  String get naiomRestartTooltip => 'Takes effect after restarting the game.';

  @override
  String get naiomNeedsCameraFix =>
      'Turn on Fix Camera Acceleration to use this';

  @override
  String get naiomNeedsThirdPerson =>
      'Turn on Third-Person Camera Fix to use this';

  @override
  String get naiomNeedsAimMode => 'Turn on Fix Pod Aiming to use this';

  @override
  String get naiomCrosshairOverrides =>
      'Not used while Crosshair Mode is on - the crosshair has its own speed';

  @override
  String get naiomThirdPersonRestartNote =>
      'Turning this ON needs a game restart. Turning it OFF works while playing.';

  @override
  String get naiomTapEvadeWarning =>
      'No Evade key is bound! With Tap Evade disabled and no dedicated Evade key you cannot dodge at all. Bind an Evade key under Non-Standard Actions.';

  @override
  String get naiomCrosshairNote =>
      'The crosshair only shows during normal top-down / side-scroll gameplay with mouse input. If it\'s not visible somewhere, that\'s usually normal - not a bug.';

  @override
  String get naiomBindingsIntro =>
      'Extra keys on top of the game\'s own controls - the original keys keep working. Changes apply after saving, no restart needed.';

  @override
  String get naiomCrosshairPreviewLabel => 'Crosshair Mode in-game';

  @override
  String get naiomCursorPick => 'Choose file...';

  @override
  String get naiomCursorClear => 'Remove';

  @override
  String get naiomCursorInvalid =>
      'Not a valid cursor file - needs a real .cur or .ani file';

  @override
  String get naiomLiveBadge => 'LIVE';

  @override
  String get naiomLiveTooltip =>
      'Applies after saving - no game restart needed.';

  @override
  String get labelPreloadMaxDimensionShort => 'Preload Max Dimension';

  @override
  String get tooltipPreloadMaxDimensionShort =>
      '0 = disabled (pure streaming), 2048 = default, 4096 = 4K textures, 16384 = everything.';

  @override
  String get labelPreloadAllTexturesShort => 'Preload All Textures';

  @override
  String get tooltipPreloadAllTexturesShort =>
      'Preload ALL textures. No stutter but needs 32GB+ RAM.';

  @override
  String get labelVramBudget => 'VRAM Budget (MB)';

  @override
  String get tooltipVramBudget =>
      'How much GPU memory the texture mod system may use. Pick a value to set a hard cap - e.g. 8192 means \"never use more than 8 GB for modded textures\", 16384 means \"never use more than 16 GB\". Auto (recommended) skips the cap and uses what your GPU actually has available.';

  @override
  String get labelStreamingEnabled => 'Background Loading';

  @override
  String get tooltipStreamingEnabled =>
      'Loads textures in the background while you play. Prevents freezes and stuttering when new areas load in. Turn off only if you have issues - without it, the game may briefly freeze when loading new textures.';

  @override
  String get labelLoadOnlyRelevant => 'Load Only Relevant';

  @override
  String get tooltipLoadOnlyRelevant =>
      'For huge packs (400+ files), only load textures matching a curated priority list - saves VRAM and loading time. Small packs (clothing, weapons) are always loaded in full. Turn on if you use a massive pack and want to save memory.';

  @override
  String get tutorialDropTextures =>
      'Drag texture mods here to install them. Zip files are extracted automatically.';

  @override
  String get tutorialLoadOrder =>
      'If mods overlap, drag to reorder. Top = highest priority.';

  @override
  String get textureOverlapLabel => 'OVERLAP';

  @override
  String tooltipTextureOverlap(String mods) {
    return 'Changes same textures as: $mods. The one higher in the list (closer to HIGHEST) is what you see in-game.';
  }

  @override
  String get tooltipFolderNotFound =>
      'Folder not found in nams/inject/textures/';

  @override
  String get priorityHighest => 'HIGHEST';

  @override
  String get priorityMedium => 'MEDIUM';

  @override
  String get priorityLowest => 'LOWEST';

  @override
  String nameOutfitTitle(String character) {
    return 'Name this outfit ($character)';
  }

  @override
  String get outfitNameHint => 'Outfit name';

  @override
  String installedTextureCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'files',
      one: 'file',
    );
    return 'Installed $count texture $_temp0';
  }

  @override
  String installationFailed(String error) {
    return 'Installation failed: $error';
  }

  @override
  String removedItem(String name) {
    return 'Removed \"$name\"';
  }

  @override
  String get tutorialStarIcon =>
      'Click the star to set a default outfit that loads on game start.';

  @override
  String installedOutfitsCount(int count) {
    return 'INSTALLED OUTFITS ($count)';
  }

  @override
  String get tooltipDlcDetected =>
      'DLC detected (data100.cpk). Model files use DLC naming (pl000d).';

  @override
  String get tooltipNoDlcDetected =>
      'No DLC detected. Model files will be renamed to non-DLC naming (pl0000).';

  @override
  String installConfirmMod(String name, String character) {
    return 'Install \"$name\" ($character)?';
  }

  @override
  String installedOutfit(String name, String character) {
    return 'Installed \"$name\" ($character)';
  }

  @override
  String get crossInstallTextures =>
      'This mod also contains texture files. Install them to nams/inject/textures/?';

  @override
  String alsoInstalledTextures(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'files',
      one: 'file',
    );
    return 'Also installed $count texture $_temp0';
  }

  @override
  String get clearedAllStartupOutfits => 'Cleared all startup outfits';

  @override
  String get clearedStartupOutfit => 'Cleared startup outfit';

  @override
  String setStartupOutfit(String name) {
    return 'Set \"$name\" as startup outfit';
  }

  @override
  String get tutorialDropCutscenes =>
      'Drop cutscene mod archives here. Supports .zip, .7z, and .rar.';

  @override
  String get tutorialInstalledCutscenes =>
      'Your installed cutscene mods. Custom cutscenes load from here instead of data/movie/.';

  @override
  String get selectCutsceneModFolder => 'Select Cutscene Mod Folder';

  @override
  String cutsceneNamingHint(int max) {
    return 'Max $max characters. This becomes the folder name in nams/cutscenes/.';
  }

  @override
  String cutsceneNameTooLong(int max) {
    return 'Name must be $max characters or fewer.';
  }

  @override
  String get preparingInstall => 'Preparing...';

  @override
  String installedCutsceneMod(String name, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'files',
      one: 'file',
    );
    return 'Installed \"$name\" ($count USM $_temp0)';
  }

  @override
  String deleteCutsceneConfirm(String name) {
    return 'Delete \"$name\" and all its files?';
  }

  @override
  String installedCutsceneModsCount(int count) {
    return 'INSTALLED CUTSCENE MODS ($count)';
  }

  @override
  String cutsceneUsmCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'files',
      one: 'file',
    );
    return '$count USM $_temp0';
  }

  @override
  String cutsceneMatchCount(int matching, int total) {
    return '$matching/$total match originals';
  }

  @override
  String tooltipMissingOriginals(String files) {
    return 'Files not matching originals: $files';
  }

  @override
  String get cutsceneMismatchHint =>
      'Some files don\'t match original cutscene names. Missing files will fall back to the original cutscenes.';

  @override
  String cutsceneMigrationBannerBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'files',
      one: 'file',
    );
    return 'Found $count custom cutscene $_temp0 directly in data/movie/. These overwrite the originals permanently. Next time, install cutscene mods here instead - if a custom file fails to load, the original plays as fallback.';
  }

  @override
  String hardwareInfoLabel(int ram, String gpu) {
    return '${ram}GB RAM | $gpu';
  }

  @override
  String hardwareInfoRamOnly(int ram) {
    return '${ram}GB RAM';
  }

  @override
  String texturesScanResult(int count, int sizeMB, int maxDim) {
    return '$count texture files, ${sizeMB}MB total, max ${maxDim}px';
  }

  @override
  String recommendedSettings(int dim, String allLabel) {
    return 'Recommended: Preload $dim, Preload All $allLabel';
  }

  @override
  String get applyRecommended => 'APPLY';

  @override
  String get settingsMatchRecommended =>
      'Your settings match the recommendation';

  @override
  String get reasonNoTextures => 'No textures installed';

  @override
  String reasonFitsInMemory(int ramGB, int textureSizeMB) {
    return '${ramGB}GB RAM, ${textureSizeMB}MB textures - fits in memory, preload everything for zero stutter';
  }

  @override
  String reasonExceedsRam(int ramGB, int estimatedGB) {
    return '${ramGB}GB RAM, ~${estimatedGB}GB estimated texture memory - preloading all will freeze or crash your system. Use a low preload dimension or remove some texture packs.';
  }

  @override
  String reasonTooLargeForAll(int ramGB, int textureSizeMB) {
    return '${ramGB}GB RAM, ${textureSizeMB}MB textures - too large to preload all, preload up to 4K on demand';
  }

  @override
  String reasonMediumRam(int ramGB) {
    return '${ramGB}GB RAM - preload up to 4K, larger textures load on demand';
  }

  @override
  String reasonLowRam(int ramGB) {
    return '${ramGB}GB RAM - preload small textures only to save memory';
  }

  @override
  String get analyzingHardware => 'Analyzing hardware and textures...';

  @override
  String texturesBloatWarning(int total, int relevant, int excess) {
    return 'This mod has $total textures but only $relevant are visually relevant (based on GPUnity\'s curated reference set). The remaining $excess textures add load time and RAM usage with no visible benefit.';
  }

  @override
  String cleanUnneededTextures(int count) {
    return 'REMOVE $count UNNEEDED';
  }

  @override
  String cleanedTextures(int deleted, int kept) {
    return 'Removed $deleted unneeded textures, kept $kept';
  }

  @override
  String get confirmCleanTextures => 'Remove unneeded textures?';

  @override
  String confirmCleanTexturesBody(int count, String sizeMB) {
    return 'This will permanently delete $count texture files ($sizeMB MB) from this mod folder.';
  }

  @override
  String get confirmCleanTexturesDetail1 =>
      'Only textures matching the GPUnity curated reference set will be kept';

  @override
  String get confirmCleanTexturesDetail2 =>
      'This only affects the selected mod folder, not other installed mods';

  @override
  String get confirmCleanTexturesDetail3 =>
      'This cannot be undone - drop the mod again to restore removed files';

  @override
  String get texturesBloatDialogTitle => 'Unnecessary textures detected';

  @override
  String texturesBloatDialogBody(int total, int relevant, int excess) {
    return 'This texture pack has $total files but only $relevant match the GPUnity curated reference set. The remaining $excess textures are likely unnecessary.';
  }

  @override
  String get texturesBloatPoint1 =>
      'Much longer game startup - the engine loads every texture at launch';

  @override
  String get texturesBloatPoint2 =>
      'Random stutter and frame drops - the game streams textures that add no visual benefit';

  @override
  String get texturesBloatPoint3 =>
      'High RAM usage - up to several GB wasted on textures you cannot see';

  @override
  String get texturesBloatPoint4 =>
      'Some AI-upscaled textures may contain artifacts or corruption';

  @override
  String get texturesBloatPoint5 =>
      'Almost no visual difference - most are tiny UI elements, particle effects, etc.';

  @override
  String get texturesBloatRecommendation =>
      'Removing them is safe and recommended for better performance.';

  @override
  String get texturesBloatKeepAll => 'Keep all';

  @override
  String texturesBloatRemoveUnneeded(int count) {
    return 'Remove unneeded ($count)';
  }

  @override
  String get texturesProgressExtracting => 'Extracting archive...';

  @override
  String get texturesProgressCopying => 'Copying files...';

  @override
  String get texturesProgressAnalyzing => 'Analyzing textures...';

  @override
  String get texturesAnalyzingSetup => 'Analyzing your texture setup...';

  @override
  String get texturesBusyMessage =>
      'Please wait - texture installation in progress';

  @override
  String texturesInstallProgress(
    int files,
    int totalFiles,
    int mb,
    int totalMb,
  ) {
    return 'Installing: $files/$totalFiles files - $mb/$totalMb MB';
  }

  @override
  String texturesAnalyzeProgress(int scanned, int total) {
    return 'Analyzing: $scanned/$total textures';
  }

  @override
  String get cleaningTextures => 'Removing unneeded textures...';

  @override
  String get textureMergeTitle => 'Add to existing or install new?';

  @override
  String get textureMergeDescription =>
      'You already have texture mods installed. Do you want to add these files to an existing mod or install as a new one?';

  @override
  String get textureMergeNewMod => 'Install as new mod';

  @override
  String textureMergeAddTo(String name) {
    return 'Add to: $name';
  }

  @override
  String get cutsceneMergeTitle => 'Add to existing or install new?';

  @override
  String get cutsceneMergeDescription =>
      'You already have cutscene mods installed. Multi-part cutscene packs should be merged into the same mod.';

  @override
  String get cutsceneMergeNewMod => 'Install as new mod';

  @override
  String cutsceneMergeAddTo(String name) {
    return 'Add to: $name';
  }

  @override
  String get headerMods => 'MODS';

  @override
  String cutsceneBundledWith(String modId) {
    return 'Bundled with $modId';
  }

  @override
  String get cutsceneSubtitleCard => 'SUBTITLES & BLACK BARS';

  @override
  String get cutsceneStatusHd => 'HD';

  @override
  String get cutsceneStatusHdTooltip =>
      '[cutscene] hd_cutscenes in nams.toml - must be true for HD cutscene mods to load.';

  @override
  String get cutsceneStatusH264 => 'H264';

  @override
  String get cutsceneStatusH264Tooltip =>
      '[cutscene] enable_h264 in nams.toml - must be true to play H264-encoded cutscenes.';

  @override
  String get modIntroTitle =>
      'Powered by NAMS - your data/ folder stays untouched';

  @override
  String get modIntroBody =>
      'NAMS loads mods from nams/mods/ through a virtual file system on top of the original game data, so nothing is ever copied or overwritten in data/. Mods can be turned on or off at any time without reinstalling, multiple outfits can coexist for the same character, and uninstalling a mod just removes its folder - the vanilla game is always intact underneath.';

  @override
  String get modListEmpty => 'No mods installed';

  @override
  String get modListEmptyHint =>
      'Drop a mod folder or archive into the box above to install.';

  @override
  String get modSearchPlaceholder => 'Search mods…';

  @override
  String get modFilterAll => 'All';

  @override
  String get modCollapseAll => 'Collapse all groups';

  @override
  String get modExpandAll => 'Expand all groups';

  @override
  String get modBulkInstall => 'Bulk install from folder';

  @override
  String modBulkInstallBusy(int done, int total, String name) {
    return 'Installing $done of $total: $name';
  }

  @override
  String get modBulkInstallScanning => 'Scanning folder for mod archives…';

  @override
  String get modBulkInstallNone =>
      'No mod archives (.zip / .7z / .rar) found in that folder.';

  @override
  String modBulkInstallDone(int installed, int total) {
    return 'Installed $installed of $total mods.';
  }

  @override
  String get modLooseInstall => 'Install loose files from folder';

  @override
  String get modLooseInstallScanning => 'Scanning folder for loose game files…';

  @override
  String get modLooseInstallNone =>
      'No loose game files (.dat / .dtt) found in that folder.';

  @override
  String modLooseInstallBusy(int count) {
    return 'Installing $count loose files…';
  }

  @override
  String modLooseInstallProgress(int done, int total) {
    return 'Copying $done of $total files…';
  }

  @override
  String get modLooseInstallFinalizing => 'Placing files into the mod…';

  @override
  String modLooseInstallDone(int count, String id) {
    return 'Installed $count loose files into $id.';
  }

  @override
  String get modGroup2b => '2B OUTFITS';

  @override
  String get modGroup9s => '9S OUTFITS';

  @override
  String get modGroupA2 => 'A2 OUTFITS';

  @override
  String get modGroupOtherOutfits => 'OTHER OUTFITS';

  @override
  String get modGroupWeapons => 'WEAPONS';

  @override
  String get modGroupAccessories => 'ACCESSORIES';

  @override
  String get modGroupItems => 'ITEMS';

  @override
  String get modGroupEnemies => 'ENEMIES';

  @override
  String get modGroupWorldProps => 'WORLD PROPS';

  @override
  String get modGroupModelVariants => 'MODEL VARIANTS';

  @override
  String get modGroupMaps => 'MAPS / STAGES';

  @override
  String get modGroupUi => 'UI / FONTS';

  @override
  String get modGroupMisc => 'MISC TEXTURES';

  @override
  String get modGroupArchives => 'CPK ARCHIVES';

  @override
  String get modGroupEffects => 'EFFECTS';

  @override
  String get modGroupScripting => 'SCRIPTS';

  @override
  String get modGroupLocalization => 'TEXT & LOCALIZATION';

  @override
  String get modGroupCutscenes => 'CUTSCENES';

  @override
  String get modGroupAudio => 'AUDIO';

  @override
  String get modGroupTextures => 'TEXTURES';

  @override
  String get modGroupNative => 'NATIVE MODS';

  @override
  String get modGroupOther => 'OTHER';

  @override
  String get modGroupMixed => 'MIXED CONTENT';

  @override
  String get modGroupWax => 'WAX COMPACT';

  @override
  String get modGroupMultiHint =>
      'This mod replaces models for several characters, so it is listed under each of them.';

  @override
  String get modGroupMixedHint =>
      'This mod changes several kinds of content at once. Click it to see everything it includes and which categories it touches.';

  @override
  String get modRename => 'Rename';

  @override
  String get modRenameDialogTitle => 'Rename mod';

  @override
  String get modRenameReset => 'Reset to original name';

  @override
  String get dropModHere => 'Drop mod here';

  @override
  String get dropModHereHint => 'or click to browse';

  @override
  String get modKindNative => 'NATIVE';

  @override
  String get modKindNativeTooltip =>
      'NAMS mod with an entities/ folder. Defines new items, weapons, outfits, accessories, quests etc. via TOML bundles.';

  @override
  String get modKindData => 'DATA';

  @override
  String get modKindDataTooltip =>
      'The classic mod format - same files that would normally go into NieRAutomata/data/, just managed under nams/mods/ instead keeping original data dir clean';

  @override
  String get textureOutfitLinkedTitle => 'Outfit-linked textures';

  @override
  String get textureOutfitLinkedSubtitle =>
      'These textures live inside their mod folder and load only while that outfit is equipped. NAMS hot-swaps them when you change outfits in-game.';

  @override
  String textureOutfitLinkedEntry(int count) {
    return '$count textures - active only with this outfit';
  }

  @override
  String get modKindTexture => 'TEXTURES';

  @override
  String get modKindTextureTooltip =>
      'A texture pack. Its .dds files were installed to nams/inject/textures/ and are managed from the Textures tab.';

  @override
  String get modKindUnknown => 'UNKNOWN';

  @override
  String get modKindUnknownTooltip =>
      'The launcher couldn\'t recognise this folder as a valid mod.';

  @override
  String get modCompatChip => 'wax compat';

  @override
  String get modCompatChipTooltip =>
      ' NAMS reads these too for compatibility with existing wax mods.';

  @override
  String get modDataChip => '+data';

  @override
  String get modDataChipTooltip =>
      'Ships a data/ overlay alongside its metadata. Models, textures, sounds etc. live here.';

  @override
  String get modDetailNoSelection => 'Select a mod to see details';

  @override
  String get modAuthor => 'Author';

  @override
  String get modVersion => 'Version';

  @override
  String get modRootPath => 'Path';

  @override
  String get modNativeBundles => 'Native bundles';

  @override
  String get modDataContent => 'Data content';

  @override
  String get modDataPlayerModels => 'Player models';

  @override
  String get modCameraSettings => 'Camera';

  @override
  String get modCameraSettingsHint =>
      'Changes are saved to the mod and apply in game while it runs.';

  @override
  String get modCameraSideOffset => 'Side offset';

  @override
  String get modCameraSideOffsetHint =>
      'Meters. Positive moves the camera right, negative left.';

  @override
  String get modCameraHeightOffset => 'Height offset';

  @override
  String get modCameraHeightOffsetHint =>
      'Meters. Positive moves the camera up, negative down.';

  @override
  String get modCameraDistanceOffset => 'Distance offset';

  @override
  String get modCameraDistanceOffsetHint =>
      'Meters. Positive moves the camera away from the player, negative closer.';

  @override
  String get modCameraFovOffset => 'Field of view offset';

  @override
  String get modCameraFovOffsetHint =>
      'Degrees. Positive widens the view, negative zooms in.';

  @override
  String get threeDInspectorOpen => 'Open 3D inspector';

  @override
  String get threeDInspectorClose => 'Close 3D inspector';

  @override
  String get threeDInspectorResetCamera => 'Reset camera';

  @override
  String get threeDInspectorControls =>
      'Left drag: orbit · Right drag: pan · Wheel: zoom · Esc: close';

  @override
  String get threeDInspectorVertices => 'vertices';

  @override
  String get threeDInspectorTriangles => 'triangles';

  @override
  String get threeDInspectorLod => 'LOD';

  @override
  String get threeDInspectorMeshes => 'Meshes';

  @override
  String get threeDInspectorExpand => 'Expand 3D inspector';

  @override
  String get threeDInspectorOutfit => 'Outfit';

  @override
  String get threeDInspectorVariant => 'Model variant';

  @override
  String get threeDInspectorVariantNormal => 'Normal';

  @override
  String get threeDInspectorVariantDamaged => 'Damaged';

  @override
  String get threeDInspectorVariantBase => 'Base';

  @override
  String get threeDInspectorVariantSelfDestruct => 'Self-destruct';

  @override
  String get threeDInspectorVariantDamagedLeft => 'Broken left';

  @override
  String get threeDInspectorVariantDamagedRight => 'Broken right';

  @override
  String get threeDInspectorVariantDamagedHoles => 'Broken holes';

  @override
  String get threeDInspectorVariantDamaged2bHand => 'Broken 2B hand';

  @override
  String get threeDInspectorVariantArmor => 'Armor';

  @override
  String get threeDInspectorVariantDlc => 'DLC';

  @override
  String get threeDInspectorVariantDlcDamaged => 'DLC damaged';

  @override
  String get threeDInspectorVariantDlcDamagedLeft => 'DLC broken left';

  @override
  String get threeDInspectorVariantDlcDamagedRight => 'DLC broken right';

  @override
  String get threeDInspectorVariantDlcDamagedHoles => 'DLC broken holes';

  @override
  String get threeDInspectorVariantDlcDamaged2bHand => 'DLC broken 2B hand';

  @override
  String get threeDInspectorVariantBerserk => 'Berserk';

  @override
  String threeDInspectorOutfitNumber(int id) {
    return 'Outfit $id';
  }

  @override
  String get threeDInspectorStates => 'States';

  @override
  String get threeDInspectorStateSelfDestructed => 'Self-destructed / broken';

  @override
  String get threeDInspectorStateEyemask => 'Eyemask';

  @override
  String get threeDInspectorStateCombat => 'Combat';

  @override
  String get threeDInspectorStatePrologue => 'Prologue';

  @override
  String get threeDInspectorStateHoly => 'Holy';

  @override
  String get threeDInspectorStateNoRight => 'Right side lost';

  @override
  String get threeDInspectorStateTower => 'Tower';

  @override
  String get threeDInspectorStateWig => 'Wig replaces body hair';

  @override
  String threeDInspectorMissingMeshes(int count) {
    return '$count configured mesh(es) not found';
  }

  @override
  String get modRequiresLabel => 'Requires';

  @override
  String get modRequiresPluginsLabel => 'Requires plugins';

  @override
  String get modRequiresMissing => 'missing';

  @override
  String get modConflictsLabel => 'Conflicts';

  @override
  String get modLoadOrderHint =>
      'These mods replace the same files. Drag to reorder - top wins.';

  @override
  String get modConflictKeep => 'KEEP THIS';

  @override
  String get modConflictResolve => 'RESOLVE';

  @override
  String get modConflictDialogTitle => 'Which mod should win?';

  @override
  String modConflictKeepTooltip(String id) {
    return 'Keep $id and disable the others';
  }

  @override
  String modConflictPickBody(int mods, int files) {
    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$files files',
      one: 'file',
    );
    return '$mods enabled mods replace the same $_temp0. Pick the one to keep - the others get disabled.';
  }

  @override
  String modConflictOverlapFile(String otherId, String file) {
    return '$otherId also ships $file';
  }

  @override
  String get modOpenFolder => 'Open folder';

  @override
  String get modEnable => 'Enable';

  @override
  String get modDisable => 'Disable';

  @override
  String get modDisabled => 'Disabled';

  @override
  String get modDisabledTooltip =>
      'This mod is disabled. NAMS will not load it on next game start. Enable it to load again - no need to delete and reinstall.';

  @override
  String get modEnableTooltip =>
      'Mod is loaded by NAMS. Click to disable without removing the files.';

  @override
  String get modDefaultTooltip =>
      'Active from game start, as if its files were in NieRAutomata/data. Click to turn off.';

  @override
  String get modSetDefaultTooltip =>
      'Make this mod active from game start, without copying anything into NieRAutomata/data.';

  @override
  String get modSetDefaultOutfitTooltip =>
      'Wear this from game start, without copying anything into NieRAutomata/data. Replaces whichever outfit is currently the default - only one can be.';

  @override
  String get modDefaultChip => 'DEFAULT';

  @override
  String get modDefaultKindOutfitBare => 'outfit';

  @override
  String get modDefaultKindOutfitConfig => 'outfit + config';

  @override
  String get modDefaultKindOutfitAnimation => 'animation';

  @override
  String get modDefaultKindOutfitBareTooltip =>
      'Replaces the model files directly. Only one outfit can be the default at a time.';

  @override
  String get modDefaultKindOutfitConfigTooltip =>
      'This mod ships an outfit config, so its mesh rules and effects load with it. Only one outfit can be the default at a time.';

  @override
  String get modDefaultKindOutfitAnimationTooltip =>
      'Animation data, not an outfit. Stays active underneath whatever outfit you wear.';

  @override
  String get modDefaultReplaceTitle => 'Replace the default?';

  @override
  String modDefaultReplaceBody(String model, String current, String next) {
    return '$model is currently worn from game start by \"$current\".\n\nMaking \"$next\" the default removes that, since only one mod can dress a character at a time.';
  }

  @override
  String get modDefaultReplaceConfirm => 'Replace';

  @override
  String get modDefaultOutfitAuto => 'Default outfit';

  @override
  String get modDefaultOutfitPickTooltip =>
      'This mod ships several outfits. Pick the one you want to wear from game start. \"Default outfit\" is the one worn without an item.';

  @override
  String modDefaultRowTooltip(String files) {
    return 'Active from game start: $files';
  }

  @override
  String get modDisableNotice => 'Disabled - takes effect on next game start.';

  @override
  String get modEnableNotice => 'Enabled - takes effect on next game start.';

  @override
  String get modUninstall => 'Uninstall';

  @override
  String get modUninstallConfirmTitle => 'Uninstall mod?';

  @override
  String modUninstallConfirmBody(String id) {
    return 'This will permanently delete the mod folder \"$id\".';
  }

  @override
  String get modProfileLabel => 'Profile';

  @override
  String get modProfileNewButton => 'New';

  @override
  String get modProfileRenameButton => 'Rename';

  @override
  String get modProfileDeleteButton => 'Delete';

  @override
  String get modProfileNewDialogTitle => 'New profile';

  @override
  String get modProfileNewDialogHint => 'Profile name (letters, numbers, _ -)';

  @override
  String get modProfileRenameDialogTitle => 'Rename profile';

  @override
  String get modProfileDeleteDialogTitle => 'Delete profile?';

  @override
  String modProfileDeleteDialogBody(String name) {
    return 'Permanently remove the folder mods_profile_$name/ and any bundled texture packs from this profile. This cannot be undone.';
  }

  @override
  String get modProfileDeleteConfirm => 'Delete';

  @override
  String get modProfileErrorNameEmpty => 'Name required';

  @override
  String get modProfileErrorNameInvalid => 'Use letters, numbers, _ or - only';

  @override
  String get modProfileErrorNameCollision =>
      'A profile with this name already exists';

  @override
  String get modProfileErrorDeleteActive =>
      'Switch to another profile before deleting this one';

  @override
  String get modProfileErrorDeleteLast =>
      'Cannot delete the only remaining profile';

  @override
  String get modProfileErrorTargetMissing =>
      'Profile folder is missing on disk';

  @override
  String get modProfileErrorFsBusy =>
      'Filesystem is busy. Close the game and retry.';

  @override
  String get modProfileLockedRunning =>
      'Stop the game before changing profiles.';

  @override
  String get modProfileEmptyHint => 'Empty profile - drop a mod to get started';

  @override
  String modProfileSwitchedToast(String name) {
    return 'Switched to profile $name';
  }

  @override
  String modProfileCreatedToast(String name) {
    return 'Created and switched to profile $name';
  }

  @override
  String modProfileDeletedToast(String name) {
    return 'Deleted profile $name';
  }

  @override
  String modProfileRenamedToast(String name) {
    return 'Renamed profile to $name';
  }

  @override
  String get modInstallNeedsName => 'Name this mod';

  @override
  String modInstallExistsPickAnother(String id) {
    return 'A mod named \"$id\" already exists. Pick a different name.';
  }

  @override
  String get modInspectBusy => 'Inspecting mod…';

  @override
  String get modInstallBusy => 'Installing mod…';

  @override
  String get modVariantDialogTitle => 'Choose what to install';

  @override
  String get modVariantDialogSubtitle =>
      'This archive contains multiple options. Pick the ones you want.';

  @override
  String get modOutfitChoiceDialogTitle => 'Choose what to install';

  @override
  String get modOutfitChoiceDialogSubtitle =>
      'Tick everything you want. Each item installs as its own mod. If an outfit ships textures, they come along and you can fine-tune which sets it uses later in the Textures tab.';

  @override
  String get variantCatPlayer => 'Outfits';

  @override
  String get variantCatWeapon => 'Weapons';

  @override
  String get variantCatAccessory => 'Accessories';

  @override
  String get variantCatEnemy => 'Enemies';

  @override
  String get variantCatModelVariant => 'Model variants';

  @override
  String get variantCatItem => 'Items';

  @override
  String get variantCatWorldProp => 'World props';

  @override
  String get variantCatMap => 'Maps';

  @override
  String get variantCatEffects => 'Effects';

  @override
  String get variantCatScripting => 'Scripting';

  @override
  String get variantCatLocalization => 'Localization';

  @override
  String get variantCatUi => 'UI';

  @override
  String get variantCatCutscenes => 'Cutscenes';

  @override
  String get variantCatAudio => 'Audio';

  @override
  String get variantCatMisc => 'Misc';

  @override
  String get variantCatOther => 'Other';

  @override
  String get variantPickOneSuffix => 'pick one';

  @override
  String get modVariantSelectAll => 'Select all';

  @override
  String get modVariantSelectNone => 'None';

  @override
  String get modVariantInstall => 'Install';

  @override
  String modVariantInstallSelected(int count) {
    return 'Install $count';
  }

  @override
  String get modVariantTexture => 'textures';

  @override
  String modVariantInstalledToast(int count) {
    return 'Installed $count option(s)';
  }

  @override
  String get modUninstallBusy => 'Uninstalling mod…';

  @override
  String modInstalled(String id) {
    return 'Installed: $id';
  }

  @override
  String modInstallFailed(String reason) {
    return 'Install failed: $reason';
  }

  @override
  String get modInstallReasonUnknownDrop =>
      'Unknown drop - the folder doesn\'t match any supported mod layout.';

  @override
  String get modInstallReasonUnsupportedNasa =>
      'This is a NASA mod (contains sadfutago.cpk), which this launcher does not support.';

  @override
  String get modInstallReasonInvalidMixed =>
      'Invalid layout - a mod can\'t mix entities and wax-style configs.';

  @override
  String get modInstallReasonNativeEmpty =>
      'No entity files found in entities/.';

  @override
  String get modInstallReasonDataEmpty => 'No recognisable content found.';

  @override
  String get modInstallReasonArchiveExtractFailed =>
      'Couldn\'t extract the archive.';

  @override
  String get modInstallReasonMoveFailed =>
      'Couldn\'t move the files into nams/mods/.';

  @override
  String get modInstallReasonTextureOnly =>
      'This is a texture pack (only .dds files). Install it from the Textures tab instead.';

  @override
  String modUninstalled(String id) {
    return 'Removed: $id';
  }

  @override
  String modCountFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count files',
      one: '1 file',
    );
    return '$_temp0';
  }

  @override
  String get weaponConvertTitle => 'Weapon converter';

  @override
  String get weaponConvertDesc =>
      'Drop a downloaded weapon mod in. It normally overwrites a vanilla weapon; the launcher moves it to a free slot instead, so nothing is replaced.';

  @override
  String get weaponConvertDropTitle => 'Drop a weapon mod';

  @override
  String get weaponConvertDropHint =>
      'Drop the .dat or .dtt here, or click to pick one. Both files must sit in the same folder.';

  @override
  String get weaponConvertDropInvalid =>
      'That is not a weapon file. Expected something like wp0003.dat.';

  @override
  String get weaponConvertNeedsBothFiles =>
      'Both the .dat and the .dtt must be in the same folder.';

  @override
  String get weaponConvertDetected =>
      'Weapon class read from the file name. The new slot keeps that class - animations come with it.';

  @override
  String get weaponConvertNextSlot => 'Another slot';

  @override
  String get weaponConvertRun => 'Move to a free slot';

  @override
  String weaponConvertDone(String stem) {
    return 'Written as $stem';
  }

  @override
  String weaponConvertNextStep(String objId) {
    return 'The model is in place. To make it a weapon of its own, create one with obj_id $objId using the builder above.';
  }

  @override
  String get weaponPreviewFailed =>
      'This model cannot be shown here. That does not stop the conversion.';

  @override
  String get weaponBuilderTitle => 'Weapon builder';

  @override
  String get weaponBuilderDesc =>
      'Fill in a name and some damage numbers; the launcher picks a free weapon slot and writes the file.';

  @override
  String get weaponBuilderDropTitle => 'Drop your weapon model';

  @override
  String get weaponBuilderDropHint => 'The .dat / .dtt pair of your weapon.';

  @override
  String get weaponBuilderNeedsModel => 'Drop your weapon model in first.';

  @override
  String get weaponBuilderModelReady =>
      'Becomes a weapon of its own, so it overwrites nothing.';

  @override
  String get weaponBuilderName => 'Weapon name';

  @override
  String get weaponBuilderNameHint =>
      'Shown in the inventory, e.g. Virtuous Blade';

  @override
  String get weaponBuilderDescription => 'Description';

  @override
  String get weaponBuilderDescriptionHint =>
      'Shown when the player looks at the weapon';

  @override
  String get weaponBuilderType => 'Weapon type';

  @override
  String get weaponTypeSmallSword => 'Small sword';

  @override
  String get weaponTypeLargeSword => 'Large sword';

  @override
  String get weaponTypeSpear => 'Spear';

  @override
  String get weaponTypeCombatBracer => 'Combat bracers';

  @override
  String get weaponBuilderLevels => 'UPGRADE LEVELS';

  @override
  String get weaponBuilderAddLevel => 'Add level';

  @override
  String weaponBuilderLevelNumber(int n) {
    return 'Level $n';
  }

  @override
  String get weaponBuilderDamage => 'Damage';

  @override
  String get weaponBuilderItemId => 'Inventory number';

  @override
  String get weaponBuilderObjId => 'Weapon number';

  @override
  String get weaponBuilderObjIdHint =>
      'Both numbers are derived from the mod folder name, so a different name gives a different weapon number. That keeps your released mod from colliding with someone else\'s. Rename the folder if you want other numbers.';

  @override
  String get weaponBuilderNeedsGameDir =>
      'Pick your NieR:Automata folder first.';

  @override
  String get weaponBuilderNeedsName => 'Give the weapon a name.';

  @override
  String get weaponBuilderNoFreeSlot =>
      'No free model slot left for this weapon type.';

  @override
  String get weaponBuilderCreate => 'Create weapon';

  @override
  String get weaponBuilderEffects => 'SPECIAL EFFECTS';

  @override
  String get weaponBuilderAddEffect => 'Add effect';

  @override
  String get weaponBuilderEffectsHint =>
      'Optional. Effects change how the game plays while the weapon is equipped - damage, speed, dash, immunities.';

  @override
  String get weaponBuilderTrailNormal => 'Attack trail';

  @override
  String get weaponBuilderTrailBerserk => 'Attack trail (Berserk)';

  @override
  String get weaponBuilderTrailHint =>
      'Any effect from 0 to 665. 590-641 are the weapon trails; other values give you anything else in the game. 435 and 436 crash and are skipped.';

  @override
  String get weaponBuilderEffectWhen => 'Only for';

  @override
  String get weaponBuilderEffectWhenAny => 'Everyone';

  @override
  String get weaponBuilderEffectOn => 'Active';

  @override
  String get weaponBuilderIcon => 'Inventory icon';

  @override
  String get weaponBuilderIconHint => 'Shown in the inventory and in shops.';

  @override
  String get weaponBuilderStoryIcon => 'Story image';

  @override
  String get weaponBuilderStoryIconHint =>
      'Shown next to the weapon story text.';

  @override
  String get weaponBuilderDetails => 'MENU TEXTS';

  @override
  String get weaponBuilderHelpShort => 'Short description';

  @override
  String get weaponBuilderHelpShortHint =>
      'One line, shown in tight menu spots';

  @override
  String get weaponBuilderStory => 'Weapon story';

  @override
  String get weaponBuilderStoryHint =>
      'The story text shown on the weapon page';

  @override
  String weaponBuilderSkillName(int n) {
    return 'Skill $n name';
  }

  @override
  String weaponBuilderSkillDesc(int n) {
    return 'Skill $n effect';
  }

  @override
  String get weaponBuilderSkillNameHint =>
      'Leave empty if the weapon has no skill';

  @override
  String get weaponBuilderSkillDescHint => 'What the skill does';

  @override
  String get weaponBuilderTraitId => 'Trait';

  @override
  String get weaponBuilderTraitNone => 'None';

  @override
  String get weaponBuilderUpgrades => 'UPGRADE COST';

  @override
  String get weaponBuilderAddUpgrade => 'Add upgrade';

  @override
  String weaponBuilderUpgradeLevel(int n) {
    return 'To level $n';
  }

  @override
  String get weaponBuilderUpgradeCost => 'Cost';

  @override
  String get weaponBuilderUpgradeHint =>
      'Without upgrade entries the weapon cannot be upgraded at the blacksmith.';

  @override
  String get outfitBuilderTitle => 'Outfit builder';

  @override
  String get outfitBuilderDesc =>
      'Drop a player model in, pick which meshes show in which state, and the launcher writes the outfit and its item for you.';

  @override
  String get outfitBuilderDropTitle => 'Drop a player model';

  @override
  String get outfitBuilderDropHint =>
      'Drop a .dat or .dtt player model here, or click to pick one.';

  @override
  String get outfitBuilderDropInvalid => 'That is not a .dat or .dtt file.';

  @override
  String get outfitBuilderDropUnknownModel =>
      'That model is not a known player model. Expected one of pl0000, pl000d (2B), pl0200, pl020d (9S), pl0100, pl010d (A2).';

  @override
  String get outfitBuilderCharacterDetected =>
      'Character read from the model file. Drop a different model to change it.';

  @override
  String get outfitBuilderNoModelInArchive =>
      'No model was found in that archive.';

  @override
  String outfitBuilderMeshesInModel(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count meshes in this model',
      one: '1 mesh in this model',
    );
    return '$_temp0';
  }

  @override
  String get outfitBuilderAcquisitionTitle => 'HOW THE PLAYER GETS IT';

  @override
  String get outfitBuilderAutoGive => 'Put it straight in the inventory';

  @override
  String get outfitBuilderAutoGiveHint =>
      'Added when a save is loaded, so it is there without having to find it.';

  @override
  String get outfitBuilderInShop => 'Sell it in a shop';

  @override
  String get outfitBuilderInShopHint =>
      'Writes a shop entry so the player can buy it.';

  @override
  String get outfitBuilderShopName => 'Shop';

  @override
  String get shopStateRouteABeforeRuinsCollapse =>
      'Route A, before the Ruined City collapses';

  @override
  String get shopStateRouteAAfterRuinsCollapse =>
      'Route A, after the Ruined City collapses';

  @override
  String get shopStateRouteBBeforeRuinsCollapse =>
      'Route B, before the Ruined City collapses';

  @override
  String get shopStateRouteBAfterRuinsCollapse =>
      'Route B, after the Ruined City collapses';

  @override
  String get shopStateRouteC => 'Route C/D, or endings A and B reached';

  @override
  String get shopStateEndingCOrD => 'Ending C or D reached';

  @override
  String get shopStateEndingCAndD => 'Endings C and D both reached';

  @override
  String get outfitBuilderShopStateAll => 'All';

  @override
  String get outfitBuilderShopState => 'Story state';

  @override
  String get outfitBuilderShopStateHint =>
      'Which point in the story the shop offers it from. 0 is the start.';

  @override
  String get outfitBuilderBuyPrice => 'Price';

  @override
  String get outfitMeshMore => 'Options';

  @override
  String get outfitMeshPreviewToggle =>
      'Show or hide this mesh in the preview only';

  @override
  String get logWrapOn => 'Wrap long lines';

  @override
  String get logWrapOff => 'Cut long lines off';

  @override
  String get outfitMeshAlways => 'Always visible';

  @override
  String get outfitMeshCombatOnly => 'Only in combat';

  @override
  String get outfitMeshOutOfCombatOnly => 'Only out of combat';

  @override
  String get outfitMeshEyemaskOnly => 'Only with blindfold';

  @override
  String get outfitMeshNoEyemaskOnly => 'Only without blindfold';

  @override
  String get outfitMeshFeatherOnly => 'Only while undamaged';

  @override
  String get outfitMeshBrokenOnly => 'Only while damaged';

  @override
  String get outfitMeshNever => 'Never show';

  @override
  String get outfitMeshHideOnSelfDestruct => 'Gone after self-destruct';

  @override
  String get outfitMeshHairsprayTint => 'Coloured by hairspray';

  @override
  String get outfitMeshReplacedByWig => 'Replaced by the wig';

  @override
  String get outfitBuilderName => 'Outfit name';

  @override
  String get outfitBuilderNameHint => 'Shown in the wardrobe, e.g. Black Dress';

  @override
  String get outfitBuilderDescription => 'Item description';

  @override
  String get outfitBuilderDescriptionHint =>
      'Shown when the player looks at the item';

  @override
  String get outfitBuilderModId => 'Mod folder';

  @override
  String get outfitBuilderOutfitId => 'Outfit id';

  @override
  String get outfitBuilderItemId => 'Item id';

  @override
  String get outfitBuilderIdHint =>
      'Both ids are picked for you and only need changing if you want a specific number.';

  @override
  String get outfitBuilderIdTaken =>
      'That outfit id is already used for this character.';

  @override
  String get outfitBuilderCopyModel => 'Copy the model into the mod';

  @override
  String get outfitBuilderCopyModelHint =>
      'Puts the .dat and .dtt in data/pl/ so the mod is complete on its own.';

  @override
  String get outfitBuilderNeedsGameDir =>
      'Pick your NieR:Automata folder first.';

  @override
  String get outfitBuilderNeedsName => 'Give the outfit a name.';

  @override
  String get outfitBuilderNeedsModel => 'Drop a player model in first.';

  @override
  String get outfitBuilderCreate => 'Create outfit';

  @override
  String outfitBuilderCreated(String path) {
    return 'Created $path';
  }

  @override
  String get outfitBuilderOpenFolder => 'Open folder';

  @override
  String get outfitBuilderPreview => 'Will be written as';

  @override
  String get outfitStateBase => 'Base';

  @override
  String get outfitStateBaseHint =>
      'Meshes visible in the normal state. Once you tick anything anywhere, everything starts hidden and only what you pick is shown.';

  @override
  String get outfitStateCombat => 'In combat';

  @override
  String get outfitStateCombatHint =>
      'Shown while fighting, on top of the base set. Vanilla puts the combat face here.';

  @override
  String get outfitStateNonCombat => 'Out of combat';

  @override
  String get outfitStateNonCombatHint =>
      'Shown while not fighting. Vanilla puts the neutral face here.';

  @override
  String get outfitStateEyemask => 'Blindfold on';

  @override
  String get outfitStateEyemaskHint =>
      'Shown while the blindfold is worn. Vanilla lists the Eyemask mesh here.';

  @override
  String get outfitStateNonEyemask => 'Blindfold off';

  @override
  String get outfitStateNonEyemaskHint =>
      'Shown when the blindfold is off, for example with the goggles equipped. Vanilla lists the eyelashes here.';

  @override
  String get outfitStateSelfDestructShow => 'Self-destruct shows';

  @override
  String get outfitStateSelfDestructShowHint =>
      'Shown after the character self-destructs.';

  @override
  String get outfitStateSelfDestructHide => 'Self-destruct hides';

  @override
  String get outfitStateSelfDestructHideHint =>
      'Hidden after the character self-destructs. Vanilla hides the skirt here.';

  @override
  String get outfitStateBroken => 'Feather hidden';

  @override
  String get outfitStateBrokenHint =>
      'Shown while the feather is not drawn. This is a separate state from self-destruct.';

  @override
  String get outfitStateNonBroken => 'Feather shown';

  @override
  String get outfitStateNonBrokenHint =>
      'Shown while the feather is drawn, the normal case.';

  @override
  String get outfitStateWig => 'Wig';

  @override
  String get outfitStateWigHint =>
      'Meshes the wig renders instead of the body. They get hidden while the wig is on. Vanilla lists Hair here.';

  @override
  String get outfitStateHairspray => 'Hairspray tint';

  @override
  String get outfitStateHairsprayHint =>
      'Meshes a hairspray colours. These are tinted, not shown or hidden. Leave empty to use Hair.';

  @override
  String get outfitStatePrologue => 'Prologue';

  @override
  String get outfitStatePrologueHint =>
      'Replaces the base set during the prologue, when 9S is heavily damaged.';

  @override
  String get outfitStateHoly => 'Holes';

  @override
  String get outfitStateHolyHint =>
      'Replaces the base set while 9S is full of holes.';

  @override
  String get outfitStateNoRight => 'No right arm';

  @override
  String get outfitStateNoRightHint =>
      'Replaces the base set after 9S loses his right arm.';

  @override
  String get outfitStateTower => 'Tower';

  @override
  String get outfitStateTowerHint => 'Replaces the base set in the tower.';
}
