class AppStrings {
  AppStrings._();

  // App
  static const String appTitle = 'YorHa Protocol Launcher';
 
  // URLs
  static const String githubUrl = 'https://github.com/Vluurie/yp_launcher';
  static const String guideUrl =
      'https://github.com/Vluurie/yp_launcher#readme';
  static const String discordUrl = 'https://discord.gg/Rrr2EH4gBE';
  static const String naoLauncherUrl =
      'https://github.com/TODO/nao-launcher'; // TODO: replace
  static const String cliDocsUrl =
      'https://github.com/TODO/cli-docs'; // TODO: replace

  // Tooltips
  static const String tooltipSourceCode = 'Source Code';
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

  // Game
  static const String gameExeName = 'NierAutomata.exe';
  static const String gameSignature = 'PRJ_028';

  // Launcher files
  static const String launcherDirName = 'YorHaProtocolLauncher';
  static const String launcherExeName = 'launch_nier.exe';
  static const String modloaderDllName = 'modloader.dll';
  static const String yorhaDllName = 'yorha_protocol.dll';
  static const String assetLauncherExe = 'assets/bins/launch_nier.exe';
  static const String assetModloaderDll = 'assets/bins/modloader.dll';
  static const String assetYorhaDll = 'assets/bins/yorha_protocol.dll';

  // CLI arguments
  static const String argModloaderDll = '--modloader-dll';
  static const String argModDll = '--mod-dll';

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
}
