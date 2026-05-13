class AppStrings {
  AppStrings._();

  // App
  static const String appTitle = 'YoRHa Protocol Launcher';

  // URLs
  static const String githubUrl = 'https://github.com/Vluurie/yp_launcher';
  static const String namsGitlabUrl =
      'https://gitlab.yasupa.de/nams/nams-rs/-/tree/master/mods/yorha_protocol?ref_type=heads';
  static const String guideUrl =
      'https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YoRHa_Protocol_Documentation.md';
  static const String discordUrl = 'https://discord.gg/Z5spWtF8qs';
  static const String naoLauncherUrl =
      'https://www.nexusmods.com/nierautomata/mods/772?tab=files';
  static const String cliDocsUrl =
      'https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YorHa_Protocol_Documentation.md#how-to-start-via-command-line';

  // Tooltips
  static const String tooltipLauncherSource = 'Launcher Source Code';
  static const String tooltipNamsSource = 'NAMS Project Source';
  static const String tooltipGuide = 'Guide';
  static const String tooltipDiscord = 'Discord';
  static const String tooltipMinimize = 'Minimize';
  static const String tooltipMaximize = 'Maximize';
  static const String tooltipRestore = 'Restore';
  static const String tooltipClose = 'Close';

  // Info & help text
  static const String infoText = 'Select your game, press play \u2014 done.';
  static const String helpPrefix = 'Launcher not working? Try ';
  static const String helpNaoLauncher = 'NAO Launcher';
  static const String helpOr = ' or ';
  static const String helpCommandLine = 'Command Line';
  static const String helpJoinDiscord = '. Join our ';
  static const String helpDiscord = 'Discord';
  static const String helpSuffix = ' for help.';

  // Directory selector
  static const String noFileSelected = 'No file selected';
  static const String selectButton = 'SELECT';
  static const String filePickerTitle = 'Select Game Executable';
  static const String allowedExtension = 'exe';

  // Play button
  static const String playButton = 'PLAY';
  static const String stopButton = 'STOP';

  // Status messages
  static const String statusReady = 'Ready to launch!';
  static const String statusSelectGame =
      'Select your game executable to get started.';
  static const String statusPreparing = 'Preparing launcher files...';
  static const String statusLaunching = 'Launching game...';
  static const String statusRunning = 'Game is running.';
  static const String statusStopped = 'Game stopped.';
  static const String statusStopping = 'Stopping game...';

  // Error messages
  static const String errorInvalidExe =
      "Looks like this is not NieR:Automata. Make sure it's the latest Steam version.";
  static const String errorFileNotExist = 'Selected file does not exist';
  static const String errorSelectFailed = 'Failed to select file';
  static const String errorStartFailed = 'Failed to start the game.';
  static const String errorStopFailed = 'Failed to stop the game.';

  // Preferences keys
  static const String prefKeyDirectory = 'nier_directory';
  static const String prefKeyMinimizeOnLaunch = 'minimize_on_launch';

  // Game
  static const String gameExeName = 'NierAutomata.exe';
  static const String gameSignature = 'PRJ_028';

  // Launcher files
  static const String launcherDirName = 'YoRHaProtocolLauncher';
  static const String launcherExeName = 'launch_nier.exe';
  static const String modloaderDllName = 'modloader.dll';
  static const String yorhaDllName = 'yorha_protocol.dll';
  static const String assetLauncherExe = 'assets/bins/launch_nier.exe';
  static const String assetModloaderDll = 'assets/bins/modloader.dll';
  static const String assetYorhaDll = 'assets/bins/yorha_protocol.dll';

  // CLI arguments
  static const String argModloaderDll = '--modloader-dll';
  static const String argModDll = '--mod-dll';

  // Feature info
  static const String featureReshade =
      'ReShade \u2014 Already installed? YP picks it up automatically.';
  static const String featureTextures =
      'HD Textures \u2014 Drop textures into nams/inject/textures/ or they get picked up from SK_Res/.';
  static const String featureLodMod =
      'LOD Mod \u2014 Built-in visual tweaks like shadows, details and pop-in. Off by default.';

  // Tooltips for action buttons
  static const String tooltipEditConfigs =
      'Change visual settings without editing files';
  static const String tooltipOpenLogs = 'Open the logs folder in Explorer';

  // Platform error messages
  static const String errorAppDataNotFound =
      'APPDATA environment variable not found';
  static const String errorWinePrefixNotFound =
      'Wine prefix not found. Please set WINEPREFIX environment variable.';
  static const String errorHomeNotFound = 'HOME environment variable not found';
  static const String errorNoWineUser =
      'No user directory found in Wine prefix';

  static String errorWineUsersNotFound(String prefix) =>
      'Wine drive_c/users directory not found in $prefix';

  static String errorPlatformNotSupported(String os) =>
      'Platform $os is not supported';

  static String errorExeNotFound(String dir) =>
      '${AppStrings.gameExeName} not found in $dir';

  static String errorFilesQuarantined(String files) =>
      'Missing launcher files: $files. This is often caused by antivirus software. We use DLL injection to load mods at runtime \u2014 this is standard for game modding but can trigger false positives. Restore the files from quarantine or re-download the launcher, then add an exclusion for the launcher install folder and %APPDATA%/YoRHaProtocolLauncher/.';

  static const String notifyFilesQuarantined =
      'Missing launcher files detected. This is often caused by antivirus software. We use DLL injection to load mods, which is normal for game modding but may trigger false positives. Restore the files from quarantine or re-download the launcher, then add an exclusion for the launcher install folder and %APPDATA%/YoRHaProtocolLauncher/.';

  // Notification banners
  static const String notifyLodModMigrated =
      'Found your old LodMod.ini settings \u2014 imported into lodmod.toml and enabled LodMod.';
  static const String notifyReShadeDetected =
      'ReShade detected \u2014 will load automatically on launch.';
  static const String notifyReShadeIncompatible =
      'ReShade with addon/ImGui support detected \u2014 incompatible. Use standard ReShade without addon support.';
  static String notifyTexturesDetected(String folder) =>
      'HD textures found in $folder \u2014 will load on launch.';
  static const String notifyWolfLimitBreakDetected =
      'Wolf Limit Break NieRAutomata.exe detected. NAMS does not need this patch and was never tested against it. The game may still launch, but performance issues, memory crashes, or mod incompatibilities are possible. For full support, restore the original Steam executable (verify game files in Steam).';
  static String notifyPlatformUnsupported(String platform) =>
      'YoRHa Protocol Launcher is currently a Windows-only build. Detected: $platform. You can keep going, but this OS is not tested or supported by the author right now. Cross-platform support (Linux/Wine, macOS via CrossOver) was working in the past and will be revisited — expect bugs until then.';

  // Desktop shortcut
  static const String tooltipCreateShortcut =
      'Create a desktop shortcut to launch with YoRHa Protocol';
  static const String shortcutName = 'NieR Automata (YoRHa Protocol)';
  static const String shortcutDescription =
      'Launch NieR:Automata with YoRHa Protocol';
  static const String notifyShortcutCreated = 'Desktop shortcut created!';
  static const String notifyShortcutFailed =
      'Failed to create desktop shortcut.';

  static const String appVersion = 'v2.0.8-beta';

  // Tab headers
  static const String headerNams = 'NAMS';
  static const String headerLodMod = 'LOD MOD';
  static const String headerTextures = 'TEXTURES';
  static const String headerYorhaProtocol = 'YORHA PROTOCOL';
  static const String headerNaiom = 'NAIOM';
  static const String headerCutscenes = 'CUTSCENES';

  // Header tooltip messages
  static const String tooltipEditsNamsToml = 'Edits nams/nams.toml';
  static const String tooltipEditsLodmodToml = 'Edits nams/lodmod.toml';
  static const String tooltipEditsTextureInjectionToml =
      'Edits nams/texture_injection.toml';
  static const String tooltipEditsSettingsJson =
      'Edits %APPDATA%\\NAMS\\settings.json';
  static const String tooltipEditsNaiom =
      'Edits nams/nams.toml [mouse] section. NAIOM features being integrated into YP.';
  static const String tooltipCutscenesLocation =
      'Mods: nams/cutscenes/<mod_name>/movie/*.usm';

  // Card titles
  static const String cardGeneral = 'GENERAL';
  static const String cardLoading = 'LOADING';
  static const String cardHeapOverrides = 'HEAP OVERRIDES';
  static const String cardLevelOfDetail = 'LEVEL OF DETAIL';
  static const String cardAmbientOcclusion = 'AMBIENT OCCLUSION';
  static const String cardPostProcessing = 'POST-PROCESSING';
  static const String cardShadows = 'SHADOWS';
  static const String cardPreloading = 'PRELOADING';
  static const String cardInstallTextures = 'INSTALL TEXTURES';
  static const String cardHowItWorks = 'HOW IT WORKS';
  static const String cardKeybinds = 'KEYBINDS';
  static const String cardWorkspace = 'WORKSPACE';
  static const String cardCheats = 'CHEATS';
  static const String cardRandomizer = 'RANDOMIZER';
  static const String cardThirdPersonCamera = 'THIRD-PERSON CAMERA';
  static const String cardPodAiming = 'POD / AIMING';
  static const String cardMisc = 'MISC';
  static const String cardMovementBindings = 'MOVEMENT BINDINGS';
  static const String cardCombatBindings = 'COMBAT BINDINGS';
  static const String cardNonStandardBindings = 'NON-STANDARD BINDINGS';
  static const String cardMenuNavigation = 'MENU NAVIGATION';
  static const String cardStructure = 'STRUCTURE';

  // Button labels
  static const String buttonSave = 'SAVE';
  static const String buttonDiscard = 'DISCARD';
  static const String buttonCancel = 'CANCEL';
  static const String buttonInstall = 'INSTALL';
  static const String buttonDelete = 'DELETE';
  static const String buttonYes = 'Yes';
  static const String buttonNo = 'No';
  static const String buttonContinue = 'Continue';
  static const String buttonBack = 'Back';
  static const String buttonGetStarted = 'Get Started';
  static const String buttonAutoDetect = 'Auto-detect';
  static const String buttonSelectManually = 'Select manually';
  static const String buttonGoToLauncher = 'Go to launcher';

  // Settings view
  static const String heapOverridesDescription =
      'Extra memory added on top of NAMS defaults. Parent heaps grow automatically. Only increase if mods need more memory. Restart required.';

  // LodMod view
  static const String lodModDescription =
      'Visual quality patches built into NAMS, inspired by Automata-LodMod by emoose. Removes LOD pop-in, sharpens shadows and ambient occlusion, forces shadow casting on all objects including foliage, disables manual culling so objects don\'t pop in/out, and removes the vignette.';

  static const String dropModelModHere =
      'Drop model mod folder or archive here';
  static const String dropToInstall = 'Drop to install';
  static const String orClickToBrowse = 'or click to browse';
  static const String mixedModDetected = 'Mixed Mod Detected';
  static const String noOutfitsInstalled = 'No outfits installed yet';
  static const String defaultNoMod = 'Default (no mod)';
  static const String clearAllStartupOutfits = 'Clear all startup outfits';
  static const String removeOutfitTitle = 'Remove outfit?';
  static const String noModelFoundError =
      'No model found. Needs pl000d.dat/.dtt (2B), pl010d.dat/.dtt (A2), or pl020d.dat/.dtt (9S).';
  static const String unsupportedArchiveFormat = 'Unsupported archive format.';
  static const String extractingArchive = 'Extracting archive...';
  static const String failedToExtractArchive = 'Failed to extract archive.';
  static const String noOutfitsInstalledNotif = 'No outfits installed.';
  static const String dialogSelectModFolder = 'Select Model Mod Folder';
  static const String dialogNameOutfitShown =
      'This name will be shown in-game when swapping outfits.';

  // Textures view
  static const String dropTextureHere = 'Drop texture folder or archive here';
  static const String installedToTextures =
      'Installed to: nams/inject/textures/';
  static const String installingTextures = 'Installing textures...';
  static const String noTextures = 'No textures';
  static const String noTexturesInstalled = 'No textures installed';
  static const String textureConflictHint =
      'Both mods are loaded. They change some of the same things. Put the one you care about more at the bottom.';
  static const String noConflictsFound =
      'No conflicts found. All mods load independently.';
  static const String selectTextureFiles = 'Select Texture Files or Archives';
  static const String noTextureFilesFound =
      'No texture files found (.dds, .png, .tga)';

  // YoRHa Protocol view
  static const String cheatsAppliedNote = 'Applied once on game start.';

  // NAIOM view
  static const String wipLabel = 'WIP';
  static const String todoLabel = 'TODO';
  static const String naiomWipBanner =
      'NAIOM (NieR Automata Input Overhaul Mod) features are being integrated natively into NAMS. '
      'Active settings work now via nams.toml. Greyed-out features are planned for a future update.';

  // Cutscenes view
  static const String dropCutsceneHere =
      'Drop cutscene mod folder or archive here';
  static const String cutsceneMovieHint =
      'Mod must contain a movie/ folder with .usm files';
  static const String cutsceneNamingTitle = 'Name this cutscene mod';
  static const String removeCutsceneModTitle = 'Remove cutscene mod?';
  static const String noCutsceneModsInstalled =
      'No cutscene mods installed yet';
  static const String cutsceneHowItWorks1 =
      'Custom cutscenes load from nams/cutscenes/ instead of data/movie/.';
  static const String cutsceneHowItWorks2 =
      'If a custom file is missing or broken, the original plays as fallback.';
  static const String cutsceneHowItWorks3 =
      'Your original game files are never touched \u2014 mods load from a separate location.';
  static const String cutsceneStructurePath =
      'nams/cutscenes/<mod_name>/movie/<filename>.usm';
  static const String cutsceneFolderNameLimit =
      'Folder names are limited to 27 characters.';
  static const String cutsceneMigrationTitle =
      'Custom cutscene files detected in data/movie/';
  static const String cutsceneMismatchHint =
      'Some files don\'t match original cutscene names. Missing files will fall back to the original cutscenes.';
  static const String noMovieFolderFound =
      'No movie/ folder with .usm files found.';
  static const String noUsmFilesFound = 'No .usm files found in the mod.';

  // Onboarding wizard
  static const String onboardingWelcomeTitle = 'Welcome to YoRHa Protocol';
  static const String onboardingWelcomeSubtitle =
      'The all-in-one mod launcher for NieR:Automata.\nNo manual file management \u2014 just play.';
  static const String onboardingSelectTitle =
      'Select your NieR:Automata installation';
  static const String onboardingSearchingDrives = 'Scanning all drives...';
  static const String onboardingSearchingNier =
      'Searching for NieR:Automata...';
  static const String onboardingSelectInstallation = 'Select Installation';
  static const String onboardingFirstPlaythroughTitle =
      'Is this your first playthrough?';
  static const String onboardingFirstPlaythroughHint =
      'If yes, certain advanced features will be hidden\nuntil you progress further.';
  static const String onboardingFirstYes = 'Yes \u2014 hide spoiler features';
  static const String onboardingFirstNo = 'No \u2014 show everything';
  static const String onboardingReadyTitle = "You're all set!";
  static const String onboardingCreateShortcut = 'Create desktop shortcut';
  static const String onboardingFirstPlaythroughSpoilerFree =
      'First playthrough (spoiler-free)';
  static const String onboardingFullAccess = 'Full access';

  // Detection labels
  static const String detectionReShade = 'ReShade';
  static const String detectionHDTextures = 'HD Textures';
  static const String detectionLodMod = 'LOD Mod (Automata-LodMod)';
  static const String detectionSkRes = 'Special K Textures (SK_Res)';
  static const String detectionNaiom = 'NAIOM';
  static const String detectionCutscenes = 'Cutscene Mods (nams/cutscenes)';
  static const String detectionCustomMovie = 'Custom cutscenes in data/movie/';
  static const String detectionDetected = 'Detected';
  static const String detectionNotFound = 'Not found';
  static const String detectionNoneFound = 'None found';
  static const String detectionLodModMigrated =
      'Found \u2014 migrated into NAMS';
  static const String detectionSkResAuto = 'Found \u2014 loaded automatically';
  static const String detectionNaiomPending =
      'Found \u2014 not yet migrated, coming in a future update';
  static const String detectionNoneInstalled = 'None installed';
  static const String detectionCustomMovieHint =
      'Found \u2014 consider using nams/cutscenes/ instead for safe fallback';
  static const String detectionInstalled = 'Installed';
  static const String detectionCustomFilesDetected = 'Custom files detected';
  static const String detectionMigratedIntoNams = 'Migrated into NAMS';
  static const String detectionLoadedAutomatically = 'Loaded automatically';
  static const String detectionMigrationComingSoon =
      'Found \u2014 migration coming soon';
  static const String detectionNotSet = 'Not set';

  // Shared labels
  static const String labelGame = 'Game';
  static const String labelMode = 'Mode';
  static const String labelDlc = 'DLC';
  static const String labelNoDlc = 'No DLC';

  // Directory selector dialogs
  static const String searchingForNier = 'Searching for NieR:Automata...';
  static const String autoButton = 'AUTO';
  static const String nierFound = 'NieR:Automata Found';
  static const String selectInstallation = 'Select your installation:';
  static const String notYourGame = 'Not your game?';
  static const String searchAllDrives = 'Search all drives';
  static const String selectManually = 'Select manually';
  static const String notFoundTitle = 'Not Found';
  static const String notFoundMessage =
      'Could not find NieR:Automata via Steam. Want to scan all drives? This may take up to 30 seconds.';
  static const String scanDrives = 'Scan drives';
  static const String confirmInstallation = 'Is this the correct installation?';
  static const String cancelButton = 'Cancel';
  static const String noSelectManually = 'No, select manually';
  static const String yesButton = 'Yes';
  static const String installationsFoundTitle =
      'NieR:Automata Installations Found';
  static const String validInstallations =
      'Valid installations (with data folder):';
  static const String withoutDataFolder = 'Without data folder:';

  // Log panel
  static const String noLogEntries = 'No log entries found';
  static const String logViewerTitle = 'LOG VIEWER';
  static const String entriesSuffix = 'entries';
  static const String scrollToBottom = 'Scroll to bottom';
  static const String openLogsFolder = 'Open logs folder';
  static const String refreshButton = 'Refresh';
  static const String tabModloaderId = 'modloader';
  static const String tabModloaderLabel = 'Modloader';
  static const String tabYorhaId = 'yorha';
  static const String tabYorhaLabel = 'YoRHa Protocol';

  // Config panel
  static const String configEditorTitle = 'CONFIG EDITOR';
  static const String saveButton = 'SAVE';
  static const String discardButton = 'DISCARD';

  // Dialogs
  static const String unsavedChangesTitle = 'Unsaved changes';
  static const String unsavedChangesMessage =
      'You have unsaved changes. Discard them?';
  static const String stay = 'STAY';
  static const String discard = 'DISCARD';

  // Validation
  static const String enterValidNumber = 'Enter a valid number';

  // Keybind
  static const String pressKey = 'Press...';

  // Tab labels
  static const String tabLauncher = 'LAUNCHER';
  static const String tabYorhaProtocol = 'YORHA PROTOCOL';
  static const String tabNams = 'NAMS';
  static const String tabLodMod = 'LOD MOD';
  static const String tabNaiom = 'NAIOM';
  static const String tabTextures = 'TEXTURES';
  static const String tabCutscenes = 'CUTSCENES';
  static const String tabDividerConfig = 'CONFIG';
  static const String tabDividerMods = 'MODS';

  // Info bar
  static const String infoBarVersionPrefix = 'YP Launcher ';
  static const String infoBarLogs = 'LOGS';
  static const String infoBarFaq = 'FAQ';
  static const String infoBarWhatsNew = "WHAT'S NEW";
  static const String infoBarShortcut = 'SHORTCUT';
  static const String tooltipFaq = 'Do I still need other mods?';

  // Detection chip labels
  static const String chipLodModOn = 'LOD MOD ON';
  static const String chipLodModOff = 'LOD MOD OFF';
  static const String chipReShade = 'ReShade';
  static const String chipNoTextures = 'No textures';
  static const String chipNoOutfits = 'No outfits';
  static const String chipNoCutsceneMod = 'No cutscene mod';
  static String chipTexturesCount(String details) => 'Textures ($details)';
  static String chipOutfitsCount(int count) =>
      '$count outfit${count > 1 ? 's' : ''}';
  static String chipInjectedCount(int count) => '$count injected';
  static const String chipSkRes = 'SK_Res';
  static String chipCutsceneMod(int width, int height, String codec) =>
      'Cutscene Mod (${width}x$height $codec)';

  // Warning messages
  static const String warningPluginLoadingDisabled =
      'Plugin loading is disabled \u2014 YoRHa Protocol workspace will not load';
  static const String warningReShadeDisabled =
      'ReShade auto-loading is disabled';
  static const String warningTextureInjectionDisabled =
      'Texture injection is disabled';

  // Changelog dialog
  static const String changelogTitle = "WHAT'S NEW";

  static const List<String> tips = [
    'Drag texture mods directly into the Textures tab',
    'Set a default outfit in Garderobe \u2014 it loads automatically on game start',
    'HD cutscene mods are auto-detected and configured',
    'LOD Mod settings come with before/after preview images',
    'Use the FAQ button to see which mods YoRHa Protocol replaces',
    'ReShade is auto-detected \u2014 no manual config needed',
    'YoRHa Protocol includes a built-in freecam with save slots',
    'Custom quests are coming soon \u2014 stay tuned',
  ];
}
