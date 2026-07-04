import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[Locale('en')];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'YoRHa Protocol Launcher'**
  String get appTitle;

  /// No description provided for @tooltipLauncherSource.
  ///
  /// In en, this message translates to:
  /// **'Launcher Source Code'**
  String get tooltipLauncherSource;

  /// No description provided for @tooltipNamsSource.
  ///
  /// In en, this message translates to:
  /// **'NAMS Project Source'**
  String get tooltipNamsSource;

  /// No description provided for @tooltipGuide.
  ///
  /// In en, this message translates to:
  /// **'Guide'**
  String get tooltipGuide;

  /// No description provided for @tooltipDiscord.
  ///
  /// In en, this message translates to:
  /// **'Discord'**
  String get tooltipDiscord;

  /// No description provided for @tooltipMinimize.
  ///
  /// In en, this message translates to:
  /// **'Minimize'**
  String get tooltipMinimize;

  /// No description provided for @tooltipMaximize.
  ///
  /// In en, this message translates to:
  /// **'Maximize'**
  String get tooltipMaximize;

  /// No description provided for @tooltipRestore.
  ///
  /// In en, this message translates to:
  /// **'Restore'**
  String get tooltipRestore;

  /// No description provided for @tooltipClose.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get tooltipClose;

  /// No description provided for @infoText.
  ///
  /// In en, this message translates to:
  /// **'Select your game, press play — done.'**
  String get infoText;

  /// No description provided for @helpPrefix.
  ///
  /// In en, this message translates to:
  /// **'Launcher not working? Try '**
  String get helpPrefix;

  /// No description provided for @helpNaoLauncher.
  ///
  /// In en, this message translates to:
  /// **'NAO Launcher'**
  String get helpNaoLauncher;

  /// No description provided for @helpOr.
  ///
  /// In en, this message translates to:
  /// **' or '**
  String get helpOr;

  /// No description provided for @helpCommandLine.
  ///
  /// In en, this message translates to:
  /// **'Command Line'**
  String get helpCommandLine;

  /// No description provided for @helpJoinDiscord.
  ///
  /// In en, this message translates to:
  /// **'. Join our '**
  String get helpJoinDiscord;

  /// No description provided for @helpDiscord.
  ///
  /// In en, this message translates to:
  /// **'Discord'**
  String get helpDiscord;

  /// No description provided for @helpSuffix.
  ///
  /// In en, this message translates to:
  /// **' for help.'**
  String get helpSuffix;

  /// No description provided for @noFileSelected.
  ///
  /// In en, this message translates to:
  /// **'No file selected'**
  String get noFileSelected;

  /// No description provided for @selectButton.
  ///
  /// In en, this message translates to:
  /// **'SELECT'**
  String get selectButton;

  /// No description provided for @filePickerTitle.
  ///
  /// In en, this message translates to:
  /// **'Select Game Executable'**
  String get filePickerTitle;

  /// No description provided for @playButton.
  ///
  /// In en, this message translates to:
  /// **'PLAY'**
  String get playButton;

  /// No description provided for @stopButton.
  ///
  /// In en, this message translates to:
  /// **'STOP'**
  String get stopButton;

  /// No description provided for @statusReady.
  ///
  /// In en, this message translates to:
  /// **'Ready to launch!'**
  String get statusReady;

  /// No description provided for @statusSelectGame.
  ///
  /// In en, this message translates to:
  /// **'Select your game executable to get started.'**
  String get statusSelectGame;

  /// No description provided for @statusPreparing.
  ///
  /// In en, this message translates to:
  /// **'Preparing launcher files...'**
  String get statusPreparing;

  /// No description provided for @statusLaunching.
  ///
  /// In en, this message translates to:
  /// **'Launching game...'**
  String get statusLaunching;

  /// No description provided for @statusRunning.
  ///
  /// In en, this message translates to:
  /// **'Game is running.'**
  String get statusRunning;

  /// No description provided for @statusStopped.
  ///
  /// In en, this message translates to:
  /// **'Game stopped.'**
  String get statusStopped;

  /// No description provided for @statusStopping.
  ///
  /// In en, this message translates to:
  /// **'Stopping game...'**
  String get statusStopping;

  /// No description provided for @errorInvalidExe.
  ///
  /// In en, this message translates to:
  /// **'Looks like this is not NieR:Automata. Make sure it\'s the latest Steam version.'**
  String get errorInvalidExe;

  /// No description provided for @errorFileNotExist.
  ///
  /// In en, this message translates to:
  /// **'Selected file does not exist'**
  String get errorFileNotExist;

  /// No description provided for @errorSelectFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed to select file'**
  String get errorSelectFailed;

  /// No description provided for @errorStartFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed to start the game.'**
  String get errorStartFailed;

  /// No description provided for @errorStopFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed to stop the game.'**
  String get errorStopFailed;

  /// No description provided for @errorFilesQuarantined.
  ///
  /// In en, this message translates to:
  /// **'Missing launcher files: {files}. This is often caused by antivirus software. We load mods at runtime — standard for game modding but it can trigger false positives. Restore the files from quarantine or re-download the launcher, then add an exclusion for the launcher install folder (the folder containing NAMS.exe).'**
  String errorFilesQuarantined(String files);

  /// No description provided for @notifyFilesQuarantined.
  ///
  /// In en, this message translates to:
  /// **'Missing launcher files detected. This is often caused by antivirus software. We load mods at runtime, which is normal for game modding but may trigger false positives. Restore the files from quarantine or re-download the launcher, then add an exclusion for the launcher install folder (the folder containing NAMS.exe).'**
  String get notifyFilesQuarantined;

  /// No description provided for @featureReshade.
  ///
  /// In en, this message translates to:
  /// **'ReShade — Already installed? YP picks it up automatically.'**
  String get featureReshade;

  /// No description provided for @featureTextures.
  ///
  /// In en, this message translates to:
  /// **'HD Textures — Drop textures into nams/inject/textures/ or they get picked up from SK_Res/.'**
  String get featureTextures;

  /// No description provided for @featureLodMod.
  ///
  /// In en, this message translates to:
  /// **'LOD Mod — Built-in visual tweaks like shadows, details and pop-in. Off by default.'**
  String get featureLodMod;

  /// No description provided for @tooltipEditConfigs.
  ///
  /// In en, this message translates to:
  /// **'Change visual settings without editing files'**
  String get tooltipEditConfigs;

  /// No description provided for @tooltipOpenLogs.
  ///
  /// In en, this message translates to:
  /// **'Open the logs folder in Explorer'**
  String get tooltipOpenLogs;

  /// No description provided for @tooltipCreateShortcut.
  ///
  /// In en, this message translates to:
  /// **'Create a desktop shortcut to launch with YoRHa Protocol'**
  String get tooltipCreateShortcut;

  /// No description provided for @shortcutName.
  ///
  /// In en, this message translates to:
  /// **'NieR Automata (YoRHa Protocol)'**
  String get shortcutName;

  /// No description provided for @shortcutDescription.
  ///
  /// In en, this message translates to:
  /// **'Launch NieR:Automata with YoRHa Protocol'**
  String get shortcutDescription;

  /// No description provided for @notifyShortcutCreated.
  ///
  /// In en, this message translates to:
  /// **'Desktop shortcut created!'**
  String get notifyShortcutCreated;

  /// No description provided for @notifyShortcutFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed to create desktop shortcut.'**
  String get notifyShortcutFailed;

  /// No description provided for @notifyLodModMigrated.
  ///
  /// In en, this message translates to:
  /// **'Found your old LodMod.ini settings — imported into lodmod.toml and enabled LodMod.'**
  String get notifyLodModMigrated;

  /// No description provided for @notifyReShadeDetected.
  ///
  /// In en, this message translates to:
  /// **'ReShade detected — disabled by default. NAMS already ships a patched native depth-of-field, so ReShade is optional. Re-enable it any time in the NAMS config tab (Disable ReShade Loading → off).'**
  String get notifyReShadeDetected;

  /// No description provided for @notifyReShadeIncompatible.
  ///
  /// In en, this message translates to:
  /// **'ReShade with addon/ImGui support detected — incompatible. Use standard ReShade without addon support.'**
  String get notifyReShadeIncompatible;

  /// No description provided for @notifyTexturesDetected.
  ///
  /// In en, this message translates to:
  /// **'HD textures found in {folder} — will load on launch.'**
  String notifyTexturesDetected(String folder);

  /// No description provided for @errorAppDataNotFound.
  ///
  /// In en, this message translates to:
  /// **'APPDATA environment variable not found'**
  String get errorAppDataNotFound;

  /// No description provided for @errorWinePrefixNotFound.
  ///
  /// In en, this message translates to:
  /// **'Wine prefix not found. Please set WINEPREFIX environment variable.'**
  String get errorWinePrefixNotFound;

  /// No description provided for @errorHomeNotFound.
  ///
  /// In en, this message translates to:
  /// **'HOME environment variable not found'**
  String get errorHomeNotFound;

  /// No description provided for @errorNoWineUser.
  ///
  /// In en, this message translates to:
  /// **'No user directory found in Wine prefix'**
  String get errorNoWineUser;

  /// No description provided for @errorWineUsersNotFound.
  ///
  /// In en, this message translates to:
  /// **'Wine drive_c/users directory not found in {prefix}'**
  String errorWineUsersNotFound(String prefix);

  /// No description provided for @errorPlatformNotSupported.
  ///
  /// In en, this message translates to:
  /// **'Platform {os} is not supported'**
  String errorPlatformNotSupported(String os);

  /// No description provided for @errorExeNotFound.
  ///
  /// In en, this message translates to:
  /// **'NierAutomata.exe not found in {dir}'**
  String errorExeNotFound(String dir);

  /// No description provided for @errorDirNotWritable.
  ///
  /// In en, this message translates to:
  /// **'Launcher folder is read-only'**
  String get errorDirNotWritable;

  /// No description provided for @errorDirNotWritableBody.
  ///
  /// In en, this message translates to:
  /// **'The YP Launcher folder cannot be written to:\n{dir}\n\nNAMS writes its runtime cache, plugins and logs next to NAMS.exe, so the launcher folder must be writable.\n\nHow to fix it:\n1. Close the launcher.\n2. Move the whole extracted YP Launcher folder out of Program Files (or any protected location) into a normal folder you own — for example Desktop, Documents, or D:\\Games\\YP Launcher.\n3. Start the launcher again from the new location.\n\nAlternative: right-click the launcher .exe and choose \"Run as administrator\" to allow writing in the current location.'**
  String errorDirNotWritableBody(String dir);

  /// No description provided for @errorGameDirNotWritable.
  ///
  /// In en, this message translates to:
  /// **'Game folder is read-only'**
  String get errorGameDirNotWritable;

  /// No description provided for @errorGameDirNotWritableBody.
  ///
  /// In en, this message translates to:
  /// **'The game folder cannot be written to:\n{gameDir}\n\nNAMS writes mods and configs into:\n{namsDir}\nbut it cannot create files there. NieR is probably installed under C:\\Program Files (x86)\\Steam, which Windows protects.\n\nHow to fix it (recommended — move the Steam library off Program Files):\n1. Open Steam > Settings > Storage.\n2. Click the drive dropdown > \"Add Drive\" and pick a folder on another drive (for example D:\\SteamLibrary).\n3. Go to your Library, right-click NieR:Automata > Properties > Installed Files > \"Move install folder\", and move it to the new library.\n4. Re-select the game in this launcher and press Play again.\n\nQuick alternative: right-click the launcher .exe and choose \"Run as administrator\" so it can write into Program Files. Moving the library is the cleaner long-term fix.'**
  String errorGameDirNotWritableBody(String gameDir, String namsDir);

  /// No description provided for @headerNams.
  ///
  /// In en, this message translates to:
  /// **'NAMS'**
  String get headerNams;

  /// No description provided for @headerLodMod.
  ///
  /// In en, this message translates to:
  /// **'LOD MOD'**
  String get headerLodMod;

  /// No description provided for @headerTextures.
  ///
  /// In en, this message translates to:
  /// **'TEXTURES'**
  String get headerTextures;

  /// No description provided for @headerYorhaProtocol.
  ///
  /// In en, this message translates to:
  /// **'YORHA PROTOCOL'**
  String get headerYorhaProtocol;

  /// No description provided for @headerNaiom.
  ///
  /// In en, this message translates to:
  /// **'NAIOM'**
  String get headerNaiom;

  /// No description provided for @headerCutscenes.
  ///
  /// In en, this message translates to:
  /// **'CUTSCENES'**
  String get headerCutscenes;

  /// No description provided for @tooltipEditsNamsToml.
  ///
  /// In en, this message translates to:
  /// **'Edits nams/nams.toml'**
  String get tooltipEditsNamsToml;

  /// No description provided for @tooltipEditsLodmodToml.
  ///
  /// In en, this message translates to:
  /// **'Edits nams/lodmod.toml'**
  String get tooltipEditsLodmodToml;

  /// No description provided for @tooltipEditsTextureInjectionToml.
  ///
  /// In en, this message translates to:
  /// **'Edits nams/texture_injection.toml'**
  String get tooltipEditsTextureInjectionToml;

  /// No description provided for @tooltipEditsSettingsJson.
  ///
  /// In en, this message translates to:
  /// **'Edits %APPDATA%\\NAMS\\settings.json'**
  String get tooltipEditsSettingsJson;

  /// No description provided for @tooltipEditsNaiom.
  ///
  /// In en, this message translates to:
  /// **'Edits nams/nams.toml [mouse] section. NAIOM features being integrated into YP.'**
  String get tooltipEditsNaiom;

  /// No description provided for @tooltipCutscenesLocation.
  ///
  /// In en, this message translates to:
  /// **'Mods: nams/cutscenes/<mod_name>/movie/*.usm'**
  String get tooltipCutscenesLocation;

  /// No description provided for @cardGeneral.
  ///
  /// In en, this message translates to:
  /// **'GENERAL'**
  String get cardGeneral;

  /// No description provided for @cardLoading.
  ///
  /// In en, this message translates to:
  /// **'LOADING'**
  String get cardLoading;

  /// No description provided for @cardHeapOverrides.
  ///
  /// In en, this message translates to:
  /// **'HEAP OVERRIDES'**
  String get cardHeapOverrides;

  /// No description provided for @cardLevelOfDetail.
  ///
  /// In en, this message translates to:
  /// **'LEVEL OF DETAIL'**
  String get cardLevelOfDetail;

  /// No description provided for @cardAmbientOcclusion.
  ///
  /// In en, this message translates to:
  /// **'AMBIENT OCCLUSION'**
  String get cardAmbientOcclusion;

  /// No description provided for @cardPostProcessing.
  ///
  /// In en, this message translates to:
  /// **'POST-PROCESSING'**
  String get cardPostProcessing;

  /// No description provided for @cardShadows.
  ///
  /// In en, this message translates to:
  /// **'SHADOWS'**
  String get cardShadows;

  /// No description provided for @cardGlobalIllumination.
  ///
  /// In en, this message translates to:
  /// **'GLOBAL ILLUMINATION'**
  String get cardGlobalIllumination;

  /// No description provided for @cardPreloading.
  ///
  /// In en, this message translates to:
  /// **'PRELOADING'**
  String get cardPreloading;

  /// No description provided for @cardTextureConfig.
  ///
  /// In en, this message translates to:
  /// **'CONFIG'**
  String get cardTextureConfig;

  /// No description provided for @cardInstallTextures.
  ///
  /// In en, this message translates to:
  /// **'INSTALL TEXTURES'**
  String get cardInstallTextures;

  /// No description provided for @cardHowItWorks.
  ///
  /// In en, this message translates to:
  /// **'HOW IT WORKS'**
  String get cardHowItWorks;

  /// No description provided for @cardKeybinds.
  ///
  /// In en, this message translates to:
  /// **'KEYBINDS'**
  String get cardKeybinds;

  /// No description provided for @cardWorkspace.
  ///
  /// In en, this message translates to:
  /// **'WORKSPACE'**
  String get cardWorkspace;

  /// No description provided for @cardCheats.
  ///
  /// In en, this message translates to:
  /// **'CHEATS'**
  String get cardCheats;

  /// No description provided for @cardRandomizer.
  ///
  /// In en, this message translates to:
  /// **'RANDOMIZER'**
  String get cardRandomizer;

  /// No description provided for @cardThirdPersonCamera.
  ///
  /// In en, this message translates to:
  /// **'THIRD-PERSON CAMERA'**
  String get cardThirdPersonCamera;

  /// No description provided for @cardPodAiming.
  ///
  /// In en, this message translates to:
  /// **'POD / AIMING'**
  String get cardPodAiming;

  /// No description provided for @cardMisc.
  ///
  /// In en, this message translates to:
  /// **'MISC'**
  String get cardMisc;

  /// No description provided for @cardMovementBindings.
  ///
  /// In en, this message translates to:
  /// **'MOVEMENT BINDINGS'**
  String get cardMovementBindings;

  /// No description provided for @cardCombatBindings.
  ///
  /// In en, this message translates to:
  /// **'COMBAT BINDINGS'**
  String get cardCombatBindings;

  /// No description provided for @cardNonStandardBindings.
  ///
  /// In en, this message translates to:
  /// **'NON-STANDARD BINDINGS'**
  String get cardNonStandardBindings;

  /// No description provided for @cardMenuNavigation.
  ///
  /// In en, this message translates to:
  /// **'MENU NAVIGATION'**
  String get cardMenuNavigation;

  /// No description provided for @cardStructure.
  ///
  /// In en, this message translates to:
  /// **'STRUCTURE'**
  String get cardStructure;

  /// No description provided for @buttonSave.
  ///
  /// In en, this message translates to:
  /// **'SAVE'**
  String get buttonSave;

  /// No description provided for @buttonDiscard.
  ///
  /// In en, this message translates to:
  /// **'DISCARD'**
  String get buttonDiscard;

  /// No description provided for @buttonCancel.
  ///
  /// In en, this message translates to:
  /// **'CANCEL'**
  String get buttonCancel;

  /// No description provided for @buttonInstall.
  ///
  /// In en, this message translates to:
  /// **'INSTALL'**
  String get buttonInstall;

  /// No description provided for @buttonDelete.
  ///
  /// In en, this message translates to:
  /// **'DELETE'**
  String get buttonDelete;

  /// No description provided for @buttonYes.
  ///
  /// In en, this message translates to:
  /// **'Yes'**
  String get buttonYes;

  /// No description provided for @buttonNo.
  ///
  /// In en, this message translates to:
  /// **'No'**
  String get buttonNo;

  /// No description provided for @buttonContinue.
  ///
  /// In en, this message translates to:
  /// **'Continue'**
  String get buttonContinue;

  /// No description provided for @buttonBack.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get buttonBack;

  /// No description provided for @buttonGetStarted.
  ///
  /// In en, this message translates to:
  /// **'Get Started'**
  String get buttonGetStarted;

  /// No description provided for @buttonAutoDetect.
  ///
  /// In en, this message translates to:
  /// **'Auto-detect'**
  String get buttonAutoDetect;

  /// No description provided for @buttonSelectManually.
  ///
  /// In en, this message translates to:
  /// **'Select manually'**
  String get buttonSelectManually;

  /// No description provided for @buttonGoToLauncher.
  ///
  /// In en, this message translates to:
  /// **'Go to launcher'**
  String get buttonGoToLauncher;

  /// No description provided for @unsavedChangesTitle.
  ///
  /// In en, this message translates to:
  /// **'Unsaved changes'**
  String get unsavedChangesTitle;

  /// No description provided for @unsavedChangesMessage.
  ///
  /// In en, this message translates to:
  /// **'You have unsaved changes. Discard them?'**
  String get unsavedChangesMessage;

  /// No description provided for @stay.
  ///
  /// In en, this message translates to:
  /// **'STAY'**
  String get stay;

  /// No description provided for @discard.
  ///
  /// In en, this message translates to:
  /// **'DISCARD'**
  String get discard;

  /// No description provided for @enterValidNumber.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid number'**
  String get enterValidNumber;

  /// No description provided for @pressKey.
  ///
  /// In en, this message translates to:
  /// **'Press...'**
  String get pressKey;

  /// No description provided for @tabLauncher.
  ///
  /// In en, this message translates to:
  /// **'Launcher'**
  String get tabLauncher;

  /// No description provided for @tabYorhaProtocol.
  ///
  /// In en, this message translates to:
  /// **'YP Devkit'**
  String get tabYorhaProtocol;

  /// No description provided for @tabNams.
  ///
  /// In en, this message translates to:
  /// **'NAMS'**
  String get tabNams;

  /// No description provided for @tabLodMod.
  ///
  /// In en, this message translates to:
  /// **'LOD Mod'**
  String get tabLodMod;

  /// No description provided for @tabNaiom.
  ///
  /// In en, this message translates to:
  /// **'NAIOM'**
  String get tabNaiom;

  /// No description provided for @tabTextures.
  ///
  /// In en, this message translates to:
  /// **'Textures'**
  String get tabTextures;

  /// No description provided for @tabMods.
  ///
  /// In en, this message translates to:
  /// **'Mod Manager'**
  String get tabMods;

  /// No description provided for @tabCutscenes.
  ///
  /// In en, this message translates to:
  /// **'Cutscenes'**
  String get tabCutscenes;

  /// No description provided for @tabSectionGeneral.
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get tabSectionGeneral;

  /// No description provided for @tabSectionConfig.
  ///
  /// In en, this message translates to:
  /// **'Config'**
  String get tabSectionConfig;

  /// No description provided for @tabSectionMods.
  ///
  /// In en, this message translates to:
  /// **'Mods'**
  String get tabSectionMods;

  /// No description provided for @tabDividerConfig.
  ///
  /// In en, this message translates to:
  /// **'CONFIG'**
  String get tabDividerConfig;

  /// No description provided for @tabDividerMods.
  ///
  /// In en, this message translates to:
  /// **'MODS'**
  String get tabDividerMods;

  /// No description provided for @infoBarVersionPrefix.
  ///
  /// In en, this message translates to:
  /// **'YP Launcher '**
  String get infoBarVersionPrefix;

  /// No description provided for @infoBarLogs.
  ///
  /// In en, this message translates to:
  /// **'LOGS'**
  String get infoBarLogs;

  /// No description provided for @infoBarFaq.
  ///
  /// In en, this message translates to:
  /// **'FAQ'**
  String get infoBarFaq;

  /// No description provided for @infoBarWhatsNew.
  ///
  /// In en, this message translates to:
  /// **'WHAT\'S NEW'**
  String get infoBarWhatsNew;

  /// No description provided for @infoBarShortcut.
  ///
  /// In en, this message translates to:
  /// **'SHORTCUT'**
  String get infoBarShortcut;

  /// No description provided for @tooltipFaq.
  ///
  /// In en, this message translates to:
  /// **'Do I still need other mods?'**
  String get tooltipFaq;

  /// No description provided for @chipLodModOn.
  ///
  /// In en, this message translates to:
  /// **'LOD MOD ON'**
  String get chipLodModOn;

  /// No description provided for @chipLodModOff.
  ///
  /// In en, this message translates to:
  /// **'LOD MOD OFF'**
  String get chipLodModOff;

  /// No description provided for @chipReShade.
  ///
  /// In en, this message translates to:
  /// **'ReShade'**
  String get chipReShade;

  /// No description provided for @chipNoTextures.
  ///
  /// In en, this message translates to:
  /// **'No textures'**
  String get chipNoTextures;

  /// No description provided for @chipNoMods.
  ///
  /// In en, this message translates to:
  /// **'No mods'**
  String get chipNoMods;

  /// No description provided for @chipNoCutsceneMod.
  ///
  /// In en, this message translates to:
  /// **'No cutscene mod'**
  String get chipNoCutsceneMod;

  /// No description provided for @chipTexturesCount.
  ///
  /// In en, this message translates to:
  /// **'Textures ({details})'**
  String chipTexturesCount(String details);

  /// No description provided for @chipModsCount.
  ///
  /// In en, this message translates to:
  /// **'{count} {count, plural, =1{mod} other{mods}}'**
  String chipModsCount(int count);

  /// No description provided for @chipInjectedCount.
  ///
  /// In en, this message translates to:
  /// **'{count} injected'**
  String chipInjectedCount(int count);

  /// No description provided for @chipSkRes.
  ///
  /// In en, this message translates to:
  /// **'SK_Res'**
  String get chipSkRes;

  /// No description provided for @chipCutsceneMod.
  ///
  /// In en, this message translates to:
  /// **'Cutscene Mod ({width}x{height} {codec})'**
  String chipCutsceneMod(int width, int height, String codec);

  /// No description provided for @warningPluginLoadingDisabled.
  ///
  /// In en, this message translates to:
  /// **'Plugin loading is disabled — YoRHa Protocol workspace will not load'**
  String get warningPluginLoadingDisabled;

  /// No description provided for @warningReShadeDisabled.
  ///
  /// In en, this message translates to:
  /// **'ReShade auto-loading is disabled'**
  String get warningReShadeDisabled;

  /// No description provided for @warningTextureInjectionDisabled.
  ///
  /// In en, this message translates to:
  /// **'Texture injection is disabled'**
  String get warningTextureInjectionDisabled;

  /// No description provided for @heapOverridesDescription.
  ///
  /// In en, this message translates to:
  /// **'Extra memory added on top of NAMS defaults. Parent heaps grow automatically. Only increase if mods need more memory. Restart required.'**
  String get heapOverridesDescription;

  /// No description provided for @lodModDescription.
  ///
  /// In en, this message translates to:
  /// **'Visual quality patches built into NAMS, inspired by Automata-LodMod by emoose. Removes LOD pop-in, sharpens shadows and ambient occlusion, forces shadow casting on all objects including foliage, disables manual culling so objects don\'t pop in/out, and removes the vignette.'**
  String get lodModDescription;

  /// No description provided for @dropModelModHere.
  ///
  /// In en, this message translates to:
  /// **'Drop model mod folder or archive here'**
  String get dropModelModHere;

  /// No description provided for @dropToInstall.
  ///
  /// In en, this message translates to:
  /// **'Drop to install'**
  String get dropToInstall;

  /// No description provided for @orClickToBrowse.
  ///
  /// In en, this message translates to:
  /// **'or click to browse'**
  String get orClickToBrowse;

  /// No description provided for @mixedModDetected.
  ///
  /// In en, this message translates to:
  /// **'Mixed Mod Detected'**
  String get mixedModDetected;

  /// No description provided for @noOutfitsInstalled.
  ///
  /// In en, this message translates to:
  /// **'No outfits installed yet'**
  String get noOutfitsInstalled;

  /// No description provided for @defaultNoMod.
  ///
  /// In en, this message translates to:
  /// **'Default (no mod)'**
  String get defaultNoMod;

  /// No description provided for @clearAllStartupOutfits.
  ///
  /// In en, this message translates to:
  /// **'Clear all startup outfits'**
  String get clearAllStartupOutfits;

  /// No description provided for @removeOutfitTitle.
  ///
  /// In en, this message translates to:
  /// **'Remove outfit?'**
  String get removeOutfitTitle;

  /// No description provided for @noModelFoundError.
  ///
  /// In en, this message translates to:
  /// **'No model found. Needs pl000d.dat/.dtt (2B), pl010d.dat/.dtt (A2), or pl020d.dat/.dtt (9S).'**
  String get noModelFoundError;

  /// No description provided for @unsupportedArchiveFormat.
  ///
  /// In en, this message translates to:
  /// **'Unsupported archive format.'**
  String get unsupportedArchiveFormat;

  /// No description provided for @extractingArchive.
  ///
  /// In en, this message translates to:
  /// **'Extracting archive...'**
  String get extractingArchive;

  /// No description provided for @extractingArchivePercent.
  ///
  /// In en, this message translates to:
  /// **'Extracting — {percent}%'**
  String extractingArchivePercent(int percent);

  /// No description provided for @extractingArchivePercentFile.
  ///
  /// In en, this message translates to:
  /// **'Extracting {percent}% — {file}'**
  String extractingArchivePercentFile(int percent, String file);

  /// No description provided for @cutsceneScanningArchive.
  ///
  /// In en, this message translates to:
  /// **'Scanning archive for movie folder...'**
  String get cutsceneScanningArchive;

  /// No description provided for @cutsceneCopyingFile.
  ///
  /// In en, this message translates to:
  /// **'Copying {current}/{total} — {name}'**
  String cutsceneCopyingFile(int current, int total, String name);

  /// No description provided for @cutsceneCopyingFileBytes.
  ///
  /// In en, this message translates to:
  /// **'Copying {current}/{total} — {name} ({mbDone} / {mbTotal} MB)'**
  String cutsceneCopyingFileBytes(
    int current,
    int total,
    String name,
    String mbDone,
    String mbTotal,
  );

  /// No description provided for @failedToExtractArchive.
  ///
  /// In en, this message translates to:
  /// **'Failed to extract archive.'**
  String get failedToExtractArchive;

  /// No description provided for @extractFailedDiskFull.
  ///
  /// In en, this message translates to:
  /// **'Extraction failed: not enough free space on the temp drive. Free up disk space and try again.'**
  String get extractFailedDiskFull;

  /// No description provided for @textureDeleteConfirmTitle.
  ///
  /// In en, this message translates to:
  /// **'Delete texture pack?'**
  String get textureDeleteConfirmTitle;

  /// No description provided for @textureDeleteConfirmBody.
  ///
  /// In en, this message translates to:
  /// **'Permanently remove {name} from nams/inject/textures/? This cannot be undone.'**
  String textureDeleteConfirmBody(String name);

  /// No description provided for @textureDeleteLabel.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get textureDeleteLabel;

  /// No description provided for @busyDeletingTexture.
  ///
  /// In en, this message translates to:
  /// **'Deleting texture pack...'**
  String get busyDeletingTexture;

  /// No description provided for @busyDeletingCutscene.
  ///
  /// In en, this message translates to:
  /// **'Deleting cutscene mod...'**
  String get busyDeletingCutscene;

  /// No description provided for @busyDeletingPlugin.
  ///
  /// In en, this message translates to:
  /// **'Deleting plugin...'**
  String get busyDeletingPlugin;

  /// No description provided for @pluginValidatingInstalling.
  ///
  /// In en, this message translates to:
  /// **'Validating and installing plugin...'**
  String get pluginValidatingInstalling;

  /// No description provided for @busyCloseTitle.
  ///
  /// In en, this message translates to:
  /// **'Operation in progress'**
  String get busyCloseTitle;

  /// No description provided for @busyCloseBody.
  ///
  /// In en, this message translates to:
  /// **'The launcher is busy installing, deleting or extracting files. Closing now can leave broken or partial files on disk. Wait until the current operation finishes, or force-close anyway.'**
  String get busyCloseBody;

  /// No description provided for @busyCloseForce.
  ///
  /// In en, this message translates to:
  /// **'Force close'**
  String get busyCloseForce;

  /// No description provided for @extractFailedDetail.
  ///
  /// In en, this message translates to:
  /// **'Extraction failed: {detail}'**
  String extractFailedDetail(String detail);

  /// No description provided for @noOutfitsInstalledNotif.
  ///
  /// In en, this message translates to:
  /// **'No outfits installed.'**
  String get noOutfitsInstalledNotif;

  /// No description provided for @dialogSelectModFolder.
  ///
  /// In en, this message translates to:
  /// **'Select Model Mod Folder'**
  String get dialogSelectModFolder;

  /// No description provided for @dialogNameOutfitShown.
  ///
  /// In en, this message translates to:
  /// **'This name will be shown in-game when swapping outfits.'**
  String get dialogNameOutfitShown;

  /// No description provided for @dropTextureHere.
  ///
  /// In en, this message translates to:
  /// **'Drop texture folder or archive here'**
  String get dropTextureHere;

  /// No description provided for @installedToTextures.
  ///
  /// In en, this message translates to:
  /// **'Installed to: nams/inject/textures/'**
  String get installedToTextures;

  /// No description provided for @installingTextures.
  ///
  /// In en, this message translates to:
  /// **'Installing textures...'**
  String get installingTextures;

  /// No description provided for @textureDropAnalyzing.
  ///
  /// In en, this message translates to:
  /// **'Analyzing dropped files...'**
  String get textureDropAnalyzing;

  /// No description provided for @textureDropNoDds.
  ///
  /// In en, this message translates to:
  /// **'No .dds textures found in this drop. Only .dds files are supported.'**
  String get textureDropNoDds;

  /// No description provided for @cutsceneDropAnalyzing.
  ///
  /// In en, this message translates to:
  /// **'Analyzing dropped files...'**
  String get cutsceneDropAnalyzing;

  /// No description provided for @cutsceneDropNoUsm.
  ///
  /// In en, this message translates to:
  /// **'No .usm cutscene files found in this drop.'**
  String get cutsceneDropNoUsm;

  /// No description provided for @modDropAnalyzing.
  ///
  /// In en, this message translates to:
  /// **'Analyzing dropped files...'**
  String get modDropAnalyzing;

  /// No description provided for @modDropNotAMod.
  ///
  /// In en, this message translates to:
  /// **'This doesn\'t look like a NAMS mod. Drop a folder/archive that contains entities/, wax/, data/, or a mod.toml.'**
  String get modDropNotAMod;

  /// No description provided for @modDropMisroutedCutscenes.
  ///
  /// In en, this message translates to:
  /// **'This looks like a standalone cutscene mod — drop it on the Cutscenes tab instead. Bundled cutscenes belong inside a mod that already ships entities/ or mod.toml.'**
  String get modDropMisroutedCutscenes;

  /// No description provided for @modSideInstalledTextures.
  ///
  /// In en, this message translates to:
  /// **'Also installed bundled texture pack(s) into nams/inject/textures/: {names}'**
  String modSideInstalledTextures(String names);

  /// No description provided for @modBundledTexturesLabel.
  ///
  /// In en, this message translates to:
  /// **'Bundled textures'**
  String get modBundledTexturesLabel;

  /// No description provided for @modBundledCutscenesLabel.
  ///
  /// In en, this message translates to:
  /// **'Bundled cutscenes'**
  String get modBundledCutscenesLabel;

  /// No description provided for @textureBundledWithMod.
  ///
  /// In en, this message translates to:
  /// **'Bundled with mod: {modId}'**
  String textureBundledWithMod(String modId);

  /// No description provided for @modUninstallAlsoTexturesLabel.
  ///
  /// In en, this message translates to:
  /// **'Also delete bundled texture pack(s): {names}'**
  String modUninstallAlsoTexturesLabel(String names);

  /// No description provided for @noTextures.
  ///
  /// In en, this message translates to:
  /// **'No textures'**
  String get noTextures;

  /// No description provided for @noTexturesInstalled.
  ///
  /// In en, this message translates to:
  /// **'No textures installed'**
  String get noTexturesInstalled;

  /// No description provided for @textureConflictHint.
  ///
  /// In en, this message translates to:
  /// **'Both mods are loaded. They change some of the same things. Put the one you care about more at the top.'**
  String get textureConflictHint;

  /// No description provided for @noConflictsFound.
  ///
  /// In en, this message translates to:
  /// **'No conflicts found. All mods load independently.'**
  String get noConflictsFound;

  /// No description provided for @selectTextureFiles.
  ///
  /// In en, this message translates to:
  /// **'Select Texture Files or Archives'**
  String get selectTextureFiles;

  /// No description provided for @noTextureFilesFound.
  ///
  /// In en, this message translates to:
  /// **'No texture files found (.dds, .png, .tga)'**
  String get noTextureFilesFound;

  /// No description provided for @cheatsAppliedNote.
  ///
  /// In en, this message translates to:
  /// **'Applied once on game start.'**
  String get cheatsAppliedNote;

  /// No description provided for @wipLabel.
  ///
  /// In en, this message translates to:
  /// **'WIP'**
  String get wipLabel;

  /// No description provided for @todoLabel.
  ///
  /// In en, this message translates to:
  /// **'TODO'**
  String get todoLabel;

  /// No description provided for @naiomWipBanner.
  ///
  /// In en, this message translates to:
  /// **'NAIOM (NieR Automata Input Overhaul Mod) features are being integrated natively into NAMS. Active settings work now via nams.toml. Greyed-out features are planned for a future update.'**
  String get naiomWipBanner;

  /// No description provided for @dropCutsceneHere.
  ///
  /// In en, this message translates to:
  /// **'Drop cutscene mod folder or archive here'**
  String get dropCutsceneHere;

  /// No description provided for @cutsceneMovieHint.
  ///
  /// In en, this message translates to:
  /// **'Mod must contain a movie/ folder with .usm files'**
  String get cutsceneMovieHint;

  /// No description provided for @cutsceneNamingTitle.
  ///
  /// In en, this message translates to:
  /// **'Name this cutscene mod'**
  String get cutsceneNamingTitle;

  /// No description provided for @removeCutsceneModTitle.
  ///
  /// In en, this message translates to:
  /// **'Remove cutscene mod?'**
  String get removeCutsceneModTitle;

  /// No description provided for @noCutsceneModsInstalled.
  ///
  /// In en, this message translates to:
  /// **'No cutscene mods installed yet'**
  String get noCutsceneModsInstalled;

  /// No description provided for @cutsceneHowItWorks1.
  ///
  /// In en, this message translates to:
  /// **'Custom cutscenes load from nams/cutscenes/ instead of data/movie/.'**
  String get cutsceneHowItWorks1;

  /// No description provided for @cutsceneHowItWorks2.
  ///
  /// In en, this message translates to:
  /// **'If a custom file is missing or broken, the original plays as fallback.'**
  String get cutsceneHowItWorks2;

  /// No description provided for @cutsceneHowItWorks3.
  ///
  /// In en, this message translates to:
  /// **'Your original game files are never touched — mods load from a separate location.'**
  String get cutsceneHowItWorks3;

  /// No description provided for @cutsceneStructurePath.
  ///
  /// In en, this message translates to:
  /// **'nams/cutscenes/<mod_name>/movie/<filename>.usm'**
  String get cutsceneStructurePath;

  /// No description provided for @cutsceneFolderNameLimit.
  ///
  /// In en, this message translates to:
  /// **'Folder names are limited to 27 characters.'**
  String get cutsceneFolderNameLimit;

  /// No description provided for @cutsceneMigrationTitle.
  ///
  /// In en, this message translates to:
  /// **'Custom cutscene files detected in data/movie/'**
  String get cutsceneMigrationTitle;

  /// No description provided for @cutsceneMigrationBody.
  ///
  /// In en, this message translates to:
  /// **'Found {count} custom cutscene {count, plural, =1{file} other{files}} directly in data/movie/. These overwrite the originals permanently. Next time, install cutscene mods here instead — if a custom file fails to load, the original plays as fallback.'**
  String cutsceneMigrationBody(int count);

  /// No description provided for @noMovieFolderFound.
  ///
  /// In en, this message translates to:
  /// **'No movie/ folder with .usm files found.'**
  String get noMovieFolderFound;

  /// No description provided for @noUsmFilesFound.
  ///
  /// In en, this message translates to:
  /// **'No .usm files found in the mod.'**
  String get noUsmFilesFound;

  /// No description provided for @onboardingWelcomeTitle.
  ///
  /// In en, this message translates to:
  /// **'Welcome to YoRHa Protocol'**
  String get onboardingWelcomeTitle;

  /// No description provided for @onboardingWelcomeSubtitle.
  ///
  /// In en, this message translates to:
  /// **'One launcher. All your mods. New NieR content.\n\nDrag-and-drop mods, mid-game outfit switching, new quests, and a built-in devkit — without configuration nightmares.'**
  String get onboardingWelcomeSubtitle;

  /// No description provided for @onboardingSelectTitle.
  ///
  /// In en, this message translates to:
  /// **'Select your NieR:Automata installation'**
  String get onboardingSelectTitle;

  /// No description provided for @onboardingSearchingDrives.
  ///
  /// In en, this message translates to:
  /// **'Scanning all drives...'**
  String get onboardingSearchingDrives;

  /// No description provided for @onboardingSearchingNier.
  ///
  /// In en, this message translates to:
  /// **'Searching for NieR:Automata...'**
  String get onboardingSearchingNier;

  /// No description provided for @onboardingSelectInstallation.
  ///
  /// In en, this message translates to:
  /// **'Select Installation'**
  String get onboardingSelectInstallation;

  /// No description provided for @onboardingFirstPlaythroughTitle.
  ///
  /// In en, this message translates to:
  /// **'Is this your first playthrough?'**
  String get onboardingFirstPlaythroughTitle;

  /// No description provided for @onboardingFirstPlaythroughHint.
  ///
  /// In en, this message translates to:
  /// **'We\'ll hide spoilers if so.'**
  String get onboardingFirstPlaythroughHint;

  /// No description provided for @onboardingFirstYes.
  ///
  /// In en, this message translates to:
  /// **'Yes — hide spoiler features'**
  String get onboardingFirstYes;

  /// No description provided for @onboardingFirstNo.
  ///
  /// In en, this message translates to:
  /// **'No — show everything'**
  String get onboardingFirstNo;

  /// No description provided for @onboardingMigrationAskTitle.
  ///
  /// In en, this message translates to:
  /// **'Modded NieR before?'**
  String get onboardingMigrationAskTitle;

  /// No description provided for @onboardingMigrationAskBody.
  ///
  /// In en, this message translates to:
  /// **'We\'ll pick up your old setup automatically.'**
  String get onboardingMigrationAskBody;

  /// No description provided for @onboardingMigrationYes.
  ///
  /// In en, this message translates to:
  /// **'Yes'**
  String get onboardingMigrationYes;

  /// No description provided for @onboardingMigrationNo.
  ///
  /// In en, this message translates to:
  /// **'No'**
  String get onboardingMigrationNo;

  /// No description provided for @onboardingMigrationResultsTitle.
  ///
  /// In en, this message translates to:
  /// **'What we found'**
  String get onboardingMigrationResultsTitle;

  /// No description provided for @onboardingMigrationResultsBody.
  ///
  /// In en, this message translates to:
  /// **''**
  String get onboardingMigrationResultsBody;

  /// No description provided for @onboardingMigrationNothingFound.
  ///
  /// In en, this message translates to:
  /// **'Nothing detected. You\'re clean.'**
  String get onboardingMigrationNothingFound;

  /// No description provided for @onboardingMigrationActionReshadeKept.
  ///
  /// In en, this message translates to:
  /// **'Disabled — NAMS has native DoF. Re-enable in NAMS config if you want it.'**
  String get onboardingMigrationActionReshadeKept;

  /// No description provided for @onboardingMigrationActionReshadeIncompatible.
  ///
  /// In en, this message translates to:
  /// **'Addon/ImGui version — replace or remove.'**
  String get onboardingMigrationActionReshadeIncompatible;

  /// No description provided for @onboardingMigrationActionTextures.
  ///
  /// In en, this message translates to:
  /// **'Will load automatically.'**
  String get onboardingMigrationActionTextures;

  /// No description provided for @onboardingMigrationActionLodMod.
  ///
  /// In en, this message translates to:
  /// **'LodMod.ini imported.'**
  String get onboardingMigrationActionLodMod;

  /// No description provided for @onboardingMigrationActionSkRes.
  ///
  /// In en, this message translates to:
  /// **'Picked up automatically.'**
  String get onboardingMigrationActionSkRes;

  /// No description provided for @onboardingMigrationActionNaiom.
  ///
  /// In en, this message translates to:
  /// **'Migration in progress — leave for now.'**
  String get onboardingMigrationActionNaiom;

  /// No description provided for @onboardingMigrationActionCutscenes.
  ///
  /// In en, this message translates to:
  /// **'Already integrated.'**
  String get onboardingMigrationActionCutscenes;

  /// No description provided for @onboardingMigrationActionExistingMods.
  ///
  /// In en, this message translates to:
  /// **'Already installed — kept as-is.'**
  String get onboardingMigrationActionExistingMods;

  /// No description provided for @onboardingMigrationLabelExistingMods.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, one{1 mod in nams/mods/} other{{count} mods in nams/mods/}}'**
  String onboardingMigrationLabelExistingMods(int count);

  /// No description provided for @onboardingModInstallAskTitle.
  ///
  /// In en, this message translates to:
  /// **'Install a mod before launching?'**
  String get onboardingModInstallAskTitle;

  /// No description provided for @onboardingModInstallAskBody.
  ///
  /// In en, this message translates to:
  /// **'Got a .zip or folder? We\'ll handle it.\n(Textures and cutscenes have their own tabs — install those later.)'**
  String get onboardingModInstallAskBody;

  /// No description provided for @onboardingModInstallYes.
  ///
  /// In en, this message translates to:
  /// **'Yes, install a mod now'**
  String get onboardingModInstallYes;

  /// No description provided for @onboardingModInstallSkip.
  ///
  /// In en, this message translates to:
  /// **'No, skip for now'**
  String get onboardingModInstallSkip;

  /// No description provided for @onboardingModInstallTitle.
  ///
  /// In en, this message translates to:
  /// **'Add your first mod'**
  String get onboardingModInstallTitle;

  /// No description provided for @onboardingModInstallBody.
  ///
  /// In en, this message translates to:
  /// **'Supports .zip, .7z, and folders.'**
  String get onboardingModInstallBody;

  /// No description provided for @onboardingModInstallSubBody.
  ///
  /// In en, this message translates to:
  /// **'Game files stay safe.'**
  String get onboardingModInstallSubBody;

  /// No description provided for @onboardingModInstallBusy.
  ///
  /// In en, this message translates to:
  /// **'Installing…'**
  String get onboardingModInstallBusy;

  /// No description provided for @onboardingModInstallInspecting.
  ///
  /// In en, this message translates to:
  /// **'Inspecting mod…'**
  String get onboardingModInstallInspecting;

  /// No description provided for @onboardingModInstallExtractingPercent.
  ///
  /// In en, this message translates to:
  /// **'Extracting… {percent}%'**
  String onboardingModInstallExtractingPercent(int percent);

  /// No description provided for @onboardingModInstallExtractingFile.
  ///
  /// In en, this message translates to:
  /// **'Extracting {percent}% — {file}'**
  String onboardingModInstallExtractingFile(int percent, String file);

  /// No description provided for @onboardingModInstallNotAMod.
  ///
  /// In en, this message translates to:
  /// **'That does not look like a valid NAMS mod. Make sure it is a .zip / .7z file with a mod.toml or a recognised data layout.'**
  String get onboardingModInstallNotAMod;

  /// No description provided for @onboardingOutfitHintHeader.
  ///
  /// In en, this message translates to:
  /// **'To wear it'**
  String get onboardingOutfitHintHeader;

  /// No description provided for @onboardingOutfitHintCompat.
  ///
  /// In en, this message translates to:
  /// **'Buy or equip from your inventory.'**
  String get onboardingOutfitHintCompat;

  /// No description provided for @onboardingOutfitHintData.
  ///
  /// In en, this message translates to:
  /// **'F1 in-game → wardrobe icon (top-left).'**
  String get onboardingOutfitHintData;

  /// No description provided for @onboardingOutfitHintMultiOutfit.
  ///
  /// In en, this message translates to:
  /// **'Or use Multi-Outfit NPC at Resistance Camp.'**
  String get onboardingOutfitHintMultiOutfit;

  /// No description provided for @onboardingOutfitHintMultiOutfitLink.
  ///
  /// In en, this message translates to:
  /// **'Get Multi-Outfit NPC'**
  String get onboardingOutfitHintMultiOutfitLink;

  /// No description provided for @onboardingOutfitHintMultiOutfitUrl.
  ///
  /// In en, this message translates to:
  /// **'https://www.nexusmods.com/nierautomata/mods/789'**
  String get onboardingOutfitHintMultiOutfitUrl;

  /// No description provided for @onboardingModInstallFailed.
  ///
  /// In en, this message translates to:
  /// **'Install failed: {reason}'**
  String onboardingModInstallFailed(String reason);

  /// No description provided for @onboardingModInstallNoGameDir.
  ///
  /// In en, this message translates to:
  /// **'Game directory not selected yet. Go back and pick your install first.'**
  String get onboardingModInstallNoGameDir;

  /// No description provided for @onboardingModInstallInstalledHeader.
  ///
  /// In en, this message translates to:
  /// **'Installed:'**
  String get onboardingModInstallInstalledHeader;

  /// No description provided for @onboardingModInstallSkipButton.
  ///
  /// In en, this message translates to:
  /// **'Skip'**
  String get onboardingModInstallSkipButton;

  /// No description provided for @onboardingModInstallDoneButton.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get onboardingModInstallDoneButton;

  /// No description provided for @onboardingModInstallPickerTitle.
  ///
  /// In en, this message translates to:
  /// **'Select a mod (.zip, .7z) or open a folder'**
  String get onboardingModInstallPickerTitle;

  /// No description provided for @onboardingModInstallFolderPickerTitle.
  ///
  /// In en, this message translates to:
  /// **'Select a mod folder'**
  String get onboardingModInstallFolderPickerTitle;

  /// No description provided for @onboardingReadyTitle.
  ///
  /// In en, this message translates to:
  /// **'You\'re all set!'**
  String get onboardingReadyTitle;

  /// No description provided for @onboardingCreateShortcut.
  ///
  /// In en, this message translates to:
  /// **'Create desktop shortcut'**
  String get onboardingCreateShortcut;

  /// No description provided for @onboardingFirstPlaythroughSpoilerFree.
  ///
  /// In en, this message translates to:
  /// **'First playthrough (spoiler-free)'**
  String get onboardingFirstPlaythroughSpoilerFree;

  /// No description provided for @onboardingFullAccess.
  ///
  /// In en, this message translates to:
  /// **'Full access'**
  String get onboardingFullAccess;

  /// No description provided for @detectionReShade.
  ///
  /// In en, this message translates to:
  /// **'ReShade'**
  String get detectionReShade;

  /// No description provided for @detectionHDTextures.
  ///
  /// In en, this message translates to:
  /// **'HD Textures'**
  String get detectionHDTextures;

  /// No description provided for @detectionLodMod.
  ///
  /// In en, this message translates to:
  /// **'LOD Mod (Automata-LodMod)'**
  String get detectionLodMod;

  /// No description provided for @detectionSkRes.
  ///
  /// In en, this message translates to:
  /// **'Special K Textures (SK_Res)'**
  String get detectionSkRes;

  /// No description provided for @detectionNaiom.
  ///
  /// In en, this message translates to:
  /// **'NAIOM'**
  String get detectionNaiom;

  /// No description provided for @detectionCutscenes.
  ///
  /// In en, this message translates to:
  /// **'Cutscene Mods (nams/cutscenes)'**
  String get detectionCutscenes;

  /// No description provided for @detectionCustomMovie.
  ///
  /// In en, this message translates to:
  /// **'Custom cutscenes in data/movie/'**
  String get detectionCustomMovie;

  /// No description provided for @detectionDetected.
  ///
  /// In en, this message translates to:
  /// **'Detected'**
  String get detectionDetected;

  /// No description provided for @detectionReShadeIncompatible.
  ///
  /// In en, this message translates to:
  /// **'Incompatible (addon/ImGui)'**
  String get detectionReShadeIncompatible;

  /// No description provided for @detectionNotFound.
  ///
  /// In en, this message translates to:
  /// **'Not found'**
  String get detectionNotFound;

  /// No description provided for @detectionNoneFound.
  ///
  /// In en, this message translates to:
  /// **'None found'**
  String get detectionNoneFound;

  /// No description provided for @detectionLodModMigrated.
  ///
  /// In en, this message translates to:
  /// **'Found — migrated into NAMS'**
  String get detectionLodModMigrated;

  /// No description provided for @detectionSkResAuto.
  ///
  /// In en, this message translates to:
  /// **'Found — loaded automatically'**
  String get detectionSkResAuto;

  /// No description provided for @detectionNaiomPending.
  ///
  /// In en, this message translates to:
  /// **'Found — not yet migrated, coming in a future update'**
  String get detectionNaiomPending;

  /// No description provided for @detectionNoneInstalled.
  ///
  /// In en, this message translates to:
  /// **'None installed'**
  String get detectionNoneInstalled;

  /// No description provided for @detectionCustomMovieHint.
  ///
  /// In en, this message translates to:
  /// **'Found — consider using nams/cutscenes/ instead for safe fallback'**
  String get detectionCustomMovieHint;

  /// No description provided for @detectionInstalled.
  ///
  /// In en, this message translates to:
  /// **'Installed'**
  String get detectionInstalled;

  /// No description provided for @detectionCustomFilesDetected.
  ///
  /// In en, this message translates to:
  /// **'Custom files detected'**
  String get detectionCustomFilesDetected;

  /// No description provided for @detectionMigratedIntoNams.
  ///
  /// In en, this message translates to:
  /// **'Migrated into NAMS'**
  String get detectionMigratedIntoNams;

  /// No description provided for @detectionLoadedAutomatically.
  ///
  /// In en, this message translates to:
  /// **'Loaded automatically'**
  String get detectionLoadedAutomatically;

  /// No description provided for @detectionMigrationComingSoon.
  ///
  /// In en, this message translates to:
  /// **'Found — migration coming soon'**
  String get detectionMigrationComingSoon;

  /// No description provided for @detectionNotSet.
  ///
  /// In en, this message translates to:
  /// **'Not set'**
  String get detectionNotSet;

  /// No description provided for @labelGame.
  ///
  /// In en, this message translates to:
  /// **'Game'**
  String get labelGame;

  /// No description provided for @labelMode.
  ///
  /// In en, this message translates to:
  /// **'Mode'**
  String get labelMode;

  /// No description provided for @labelDlc.
  ///
  /// In en, this message translates to:
  /// **'DLC'**
  String get labelDlc;

  /// No description provided for @labelNoDlc.
  ///
  /// In en, this message translates to:
  /// **'No DLC'**
  String get labelNoDlc;

  /// No description provided for @searchingForNier.
  ///
  /// In en, this message translates to:
  /// **'Searching for NieR:Automata...'**
  String get searchingForNier;

  /// No description provided for @autoButton.
  ///
  /// In en, this message translates to:
  /// **'AUTO'**
  String get autoButton;

  /// No description provided for @nierFound.
  ///
  /// In en, this message translates to:
  /// **'NieR:Automata Found'**
  String get nierFound;

  /// No description provided for @selectInstallation.
  ///
  /// In en, this message translates to:
  /// **'Select your installation:'**
  String get selectInstallation;

  /// No description provided for @notYourGame.
  ///
  /// In en, this message translates to:
  /// **'Not your game?'**
  String get notYourGame;

  /// No description provided for @searchAllDrives.
  ///
  /// In en, this message translates to:
  /// **'Search all drives'**
  String get searchAllDrives;

  /// No description provided for @selectManually.
  ///
  /// In en, this message translates to:
  /// **'Select manually'**
  String get selectManually;

  /// No description provided for @notFoundTitle.
  ///
  /// In en, this message translates to:
  /// **'Not Found'**
  String get notFoundTitle;

  /// No description provided for @notFoundMessage.
  ///
  /// In en, this message translates to:
  /// **'Could not find NieR:Automata via Steam. Want to scan all drives? This may take up to 30 seconds.'**
  String get notFoundMessage;

  /// No description provided for @scanDrives.
  ///
  /// In en, this message translates to:
  /// **'Scan drives'**
  String get scanDrives;

  /// No description provided for @confirmInstallation.
  ///
  /// In en, this message translates to:
  /// **'Is this the correct installation?'**
  String get confirmInstallation;

  /// No description provided for @cancelButton.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancelButton;

  /// No description provided for @noSelectManually.
  ///
  /// In en, this message translates to:
  /// **'No, select manually'**
  String get noSelectManually;

  /// No description provided for @yesButton.
  ///
  /// In en, this message translates to:
  /// **'Yes'**
  String get yesButton;

  /// No description provided for @installationsFoundTitle.
  ///
  /// In en, this message translates to:
  /// **'NieR:Automata Installations Found'**
  String get installationsFoundTitle;

  /// No description provided for @validInstallations.
  ///
  /// In en, this message translates to:
  /// **'Valid installations (with data folder):'**
  String get validInstallations;

  /// No description provided for @withoutDataFolder.
  ///
  /// In en, this message translates to:
  /// **'Without data folder:'**
  String get withoutDataFolder;

  /// No description provided for @noLogEntries.
  ///
  /// In en, this message translates to:
  /// **'No log entries found'**
  String get noLogEntries;

  /// No description provided for @noLogMatches.
  ///
  /// In en, this message translates to:
  /// **'No log entries match your search'**
  String get noLogMatches;

  /// No description provided for @logViewerTitle.
  ///
  /// In en, this message translates to:
  /// **'LOG VIEWER'**
  String get logViewerTitle;

  /// No description provided for @logSearchPlaceholder.
  ///
  /// In en, this message translates to:
  /// **'Search level / module / message...'**
  String get logSearchPlaceholder;

  /// No description provided for @logLiveBadge.
  ///
  /// In en, this message translates to:
  /// **'LIVE'**
  String get logLiveBadge;

  /// No description provided for @logAutoScroll.
  ///
  /// In en, this message translates to:
  /// **'Auto-scroll to newest'**
  String get logAutoScroll;

  /// No description provided for @entriesSuffix.
  ///
  /// In en, this message translates to:
  /// **'entries'**
  String get entriesSuffix;

  /// No description provided for @scrollToBottom.
  ///
  /// In en, this message translates to:
  /// **'Scroll to bottom'**
  String get scrollToBottom;

  /// No description provided for @openLogsFolder.
  ///
  /// In en, this message translates to:
  /// **'Open logs folder'**
  String get openLogsFolder;

  /// No description provided for @diagnosticsButton.
  ///
  /// In en, this message translates to:
  /// **'Generate diagnostics'**
  String get diagnosticsButton;

  /// No description provided for @diagnosticsTitle.
  ///
  /// In en, this message translates to:
  /// **'Diagnostics summary'**
  String get diagnosticsTitle;

  /// No description provided for @diagnosticsCollecting.
  ///
  /// In en, this message translates to:
  /// **'Collecting diagnostics...'**
  String get diagnosticsCollecting;

  /// No description provided for @diagnosticsCopy.
  ///
  /// In en, this message translates to:
  /// **'Copy summary'**
  String get diagnosticsCopy;

  /// No description provided for @diagnosticsCopied.
  ///
  /// In en, this message translates to:
  /// **'Summary copied to clipboard'**
  String get diagnosticsCopied;

  /// No description provided for @diagnosticsSaveFull.
  ///
  /// In en, this message translates to:
  /// **'Save full report'**
  String get diagnosticsSaveFull;

  /// No description provided for @diagnosticsSavedAt.
  ///
  /// In en, this message translates to:
  /// **'Full report saved to {path}'**
  String diagnosticsSavedAt(String path);

  /// No description provided for @modsTutorialBack.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get modsTutorialBack;

  /// No description provided for @modsTutorialNext.
  ///
  /// In en, this message translates to:
  /// **'Next'**
  String get modsTutorialNext;

  /// No description provided for @modsTutorialFinish.
  ///
  /// In en, this message translates to:
  /// **'Got it'**
  String get modsTutorialFinish;

  /// No description provided for @modsTutorialSkip.
  ///
  /// In en, this message translates to:
  /// **'Skip'**
  String get modsTutorialSkip;

  /// No description provided for @modsTutorialMenuEcosystem.
  ///
  /// In en, this message translates to:
  /// **'NAMS & the launcher'**
  String get modsTutorialMenuEcosystem;

  /// No description provided for @modsTutorialMenuInstall.
  ///
  /// In en, this message translates to:
  /// **'How to install a mod'**
  String get modsTutorialMenuInstall;

  /// No description provided for @modsTutorialMenuProfiles.
  ///
  /// In en, this message translates to:
  /// **'How profiles work'**
  String get modsTutorialMenuProfiles;

  /// No description provided for @modsTutorialMenuSupporting.
  ///
  /// In en, this message translates to:
  /// **'Supporting the ecosystem'**
  String get modsTutorialMenuSupporting;

  /// No description provided for @modsTutorialSupportingStep1Title.
  ///
  /// In en, this message translates to:
  /// **'An ecosystem, not a mod manager'**
  String get modsTutorialSupportingStep1Title;

  /// No description provided for @modsTutorialSupportingStep1Body.
  ///
  /// In en, this message translates to:
  /// **'**NAMS has been a 3+ year project.** What started as a single modloader grew into a whole platform that **mods**, **plugins**, and tools are now built on top of:\n\n- **Mods** are content additions — outfits, custom quests, new weapons, textures. They use NAMS\'s content system but don\'t run their own code.\n- **Plugins** are programs that run alongside the game and can extend the modloader itself — things like the multiplayer mod, debug overlays, or new game systems. They\'re written in code, loaded by NAMS at startup.\n- The launcher you\'re using right now is **not a plugin** — it\'s a separate app that prepares your mods and configs before launch.\n\nThe mods you see today aren\'t running in spite of an obstacle course — they\'re running on top of years of foundational work that exists specifically so you don\'t have to redo it.\n\nThis section is about **how that platform stays alive** and what it means for you — whether you\'re just playing, dipping into modding, or thinking about contributing something back.\n\nThe next pages start with the parts that apply to **everyone**, then go into more technical territory at the end. Skip ahead whenever you\'ve heard enough — none of it is required reading.'**
  String get modsTutorialSupportingStep1Body;

  /// No description provided for @modsTutorialSupportingStep2Title.
  ///
  /// In en, this message translates to:
  /// **'Content without code'**
  String get modsTutorialSupportingStep2Title;

  /// No description provided for @modsTutorialSupportingStep2Body.
  ///
  /// In en, this message translates to:
  /// **'**You don\'t need to be a programmer to add to this ecosystem.**\n\nNAMS has a content system that already supports — and is steadily growing to support more — declarative content additions:\n\n- **Custom quests** added on top of the vanilla story.\n- **New weapons and items** with their own behavior, no overrides.\n- **Accessories** in **new slots**, not replacing existing ones.\n- **Custom cutscenes** that live in `nams/cutscenes/` or are bundled inside a mod (e.g. a custom quest shipping its own cinematics) — the original cutscenes are **never replaced**, new ones simply load alongside.\n\nThe through-line: **additive, not replacive.** Vanilla content is preserved; modded content is layered on top. That means a modeler with no programming experience can build accessories, weapons, characters, and ship them as additions — without overwriting anything, without breaking saves, without conflicts with other mods doing the same.\n\nThis surface is going to **expand** over time. Every release adds more declarative hooks so non-programmers can do more.'**
  String get modsTutorialSupportingStep2Body;

  /// No description provided for @modsTutorialSupportingStep3Title.
  ///
  /// In en, this message translates to:
  /// **'Ways to contribute — not just code'**
  String get modsTutorialSupportingStep3Title;

  /// No description provided for @modsTutorialSupportingStep3Body.
  ///
  /// In en, this message translates to:
  /// **'Making content (the previous page) is one way to give back. It\'s far from the only one — and a lot of the things that keep an ecosystem healthy don\'t involve making mods at all. Here\'s what actually moves the needle:\n\n- **Write a guide.** \"How I built X with NAMS\", \"how I debugged Y\", \"the five things I wish I\'d known\". Most of the gaps in onboarding right now are documentation gaps.\n- **Report a real bug, well.** A reproducible repro, a log, the diagnostics report from the Logs panel. That\'s worth more than ten \"it doesn\'t work\" tickets.\n- **Model.** Accessories, weapons, characters, props. NAMS\'s content system loads those as **additions** — new slots, no overrides — so a modeler with no programming experience can ship work that just slots into everyone else\'s loadouts without conflicts.\n- **Translate.** The launcher is localized. If your language isn\'t there yet and you\'d use it, the strings are in `lib/l10n/` and a PR is welcome.\n- **Test on weird hardware.** Steam Deck, AMD GPUs, ultrawide setups, multi-monitor, controllers nobody owns. Bugs that only show up on rare configurations stay hidden until someone reports them.\n- **Just answer questions in the Discord.** Helping the next new person is contribution. Most of what survives in any ecosystem long-term is the culture of the people who showed up early.\n- **Reverse-engineer one game function and give the API back.** *(more on this on the next page if you\'re curious)* — for the technically-inclined, this is the highest-leverage contribution there is.\n\n### Closing thought (so far)\n\nA lot of free time, personal investment, and stubborn fleiß went into making this platform exist — always with the mindset of **enabling other people to start doing something**. If one of those bullet points feels doable, you\'re already most of the way there. The Discord (**YoRHa Continuum**) is the place to ask.\n\nThe next two pages get more technical — read on if you\'re curious about how plugins coexist and how new game APIs end up in NAMS, or stop here if you\'ve heard enough.'**
  String get modsTutorialSupportingStep3Body;

  /// No description provided for @modsTutorialSupportingStep4Title.
  ///
  /// In en, this message translates to:
  /// **'Plugins coexist — by design'**
  String get modsTutorialSupportingStep4Title;

  /// No description provided for @modsTutorialSupportingStep4Body.
  ///
  /// In en, this message translates to:
  /// **'*The next two pages are more technical and aimed at people thinking about building plugins. Skip if not your thing.*\n\nA defining feature of the NAMS platform is that **plugins can run together at the same time**, in the same game session, without fighting each other.\n\n**Multiplayer Mod by Jasper** is one of the biggest things to happen in the NAMS ecosystem and is still actively maintained — huge respect to that work. The YP devkit and the MP plugin **both load at once**, both work, both render their own UI on top of the game. That\'s not a happy accident — that\'s what NAMS\'s plugin host was built for.\n\nSo if you ship a plugin that follows the platform\'s expectations, **it can coexist with everything else already running** — your plugin, the MP plugin, the YP devkit, future plugins from people you\'ll never meet. You don\'t have to compete for hooks or fight load order; the platform mediates.\n\nThere are still refactorings happening every month to reduce edges where one plugin can accidentally break another. It\'s a moving target — but the direction is clear and the work is ongoing.'**
  String get modsTutorialSupportingStep4Body;

  /// No description provided for @modsTutorialSupportingStep5Title.
  ///
  /// In en, this message translates to:
  /// **'You stand on free reverse-engineering'**
  String get modsTutorialSupportingStep5Title;

  /// No description provided for @modsTutorialSupportingStep5Body.
  ///
  /// In en, this message translates to:
  /// **'Most of the engine APIs you\'d need to build a serious plugin already exist in NAMS — and **they exist because someone reverse-engineered the game** to solve their own problem and contributed the result back.\n\nThe game is closed-source. Every API in NAMS that lets you read or write some game state is the result of someone tracing functions, finding offsets, validating behavior. That\'s a lot of free work, and it lives in NAMS specifically so the **next** plugin author doesn\'t have to redo it.\n\nWhen you build on NAMS, you inherit all of that. You don\'t start from `LoadLibrary`; you start from APIs that someone already wrestled into existence — and the next person who needs the API you reverse-engineered gets the same gift.\n\n### Why this is the highest-leverage contribution\n\nIf you ever do this — even once — you\'ve permanently saved every future plugin author the same work. That\'s the asymmetry. A guide helps a few people read it; an API in NAMS helps everyone who\'ll ever need that capability, forever. The ecosystem grows on the back of people who reverse-engineer one thing for themselves and leave the result behind for everyone.'**
  String get modsTutorialSupportingStep5Body;

  /// No description provided for @modsTutorialEcosystemStep1Title.
  ///
  /// In en, this message translates to:
  /// **'Why this all exists'**
  String get modsTutorialEcosystemStep1Title;

  /// No description provided for @modsTutorialEcosystemStep1Body.
  ///
  /// In en, this message translates to:
  /// **'Modding NieR has historically been painful. Mods that work fine on their own start fighting the moment you stack a few of them — different DLL hooks (`dxgi`, `d3d11`, `dinput8`) compete for the same slot, the wrong wrapper wins the load order, and the game silently crashes on boot. People with 5–10 mods spend more time bisecting than playing.\n\nFor a long time the only answer was *manual installs only*: drop loose `.dat`/`.dtt` files into `data/`, hand-edit configs, never use a mod manager. That works for one or two mods, but it overwrites real game files and leaves no record of what changed. Tools like Vortex didn\'t help — they don\'t understand NieR\'s quirks.\n\n**NAMS exists to fix this at the modloader level**, and **this launcher exists to give NAMS a friendly face on top.**'**
  String get modsTutorialEcosystemStep1Body;

  /// No description provided for @modsTutorialEcosystemStep2Title.
  ///
  /// In en, this message translates to:
  /// **'What NAMS does'**
  String get modsTutorialEcosystemStep2Title;

  /// No description provided for @modsTutorialEcosystemStep2Body.
  ///
  /// In en, this message translates to:
  /// **'**NAMS is the modloader.** It\'s not a proxy DLL hijacking `dxgi.dll` or `d3d11.dll` like older tools did — that\'s the very mechanism that caused the conflicts in the first place.\n\nInstead, NAMS runs as its own host process: it loads NieR:Automata as a library inside that process (the game\'s exe transformed into a loadable `game.bin`) and initializes the modloader before the game starts. Nothing is injected into another process — NAMS *is* the process the game runs in, with full control over what\'s about to load.\n\nFrom there, NAMS does two big things:\n\n**1. Reimplements the features other tools used to provide** — LodMod, Limit Break, texture injection, fast loading — natively, in one coordinated layer. Mods plug into NAMS APIs instead of fighting over which DLL hook gets called first.\n\n**2. Provides a virtual file system (VFS):**\n\n- Every mod lives in its own folder under `nams/mods/<modId>/` — never overwriting real game files.\n- At runtime NAMS overlays active mods into the engine\'s view of `data/`.\n- Your vanilla `data/*.cpk` and `NieRAutomata.exe` are **never modified**, so launching unmodded through Steam still works exactly as before.\n\nMods declare what they change in a manifest. NAMS validates and loads them in a defined order, so you finally get **clean enable/disable per mod** and **knowable conflict detection** — neither was possible with the old drop-files-into-`data` approach.\n\n### How this fits together\n\nThis launcher is **not** built directly on top of NieR:Automata. It doesn\'t reverse-engineer the game, hook engine functions, or know anything about `.dat`/`.dtt` formats on its own. The order is:\n\n1. **NieR:Automata** — the game, untouched.\n2. **NAMS** — the modloader, designed first to make scalable modding possible at all (engine reimplementation, VFS, plugin host, content framework).\n3. **This launcher** — built on top of NAMS as the helper. It reads NAMS\'s TOML configs, writes into NAMS\'s folder layout, and talks to NAMS\'s APIs. That\'s it.\n\nThe practical consequence: NAMS is the load-bearing layer. The launcher is just a friendly UI in front of it, and could be replaced by a different UI (or the command line) without your mods caring.\n\n### And it has been\n\nThis isn\'t theoretical. **The NAO Launcher by Rustukun** is a separate launcher built on the same foundation — a different UI, different design choices, talks to the same NAMS underneath. Your mods, your `nams/mods/<modId>/` folders, your `disabled_mods.toml` — all of it works the same regardless of which launcher you\'re using.\n\nThat\'s the proof that NAMS is the platform and any launcher (this one, NAO, a future one nobody\'s written yet) is just a frontend choice. Pick whichever fits your workflow; your mod library doesn\'t have to move.'**
  String get modsTutorialEcosystemStep2Body;

  /// No description provided for @modsTutorialEcosystemStep3Title.
  ///
  /// In en, this message translates to:
  /// **'What this launcher adds — and what\'s different'**
  String get modsTutorialEcosystemStep3Title;

  /// No description provided for @modsTutorialEcosystemStep3Body.
  ///
  /// In en, this message translates to:
  /// **'NAMS handles loading. The launcher handles **everything around it** — install, organisation, troubleshooting:\n\n- **Mod Manager** — drag-and-drop install of NAMS-format mods, automatic layout normalization (wax/SK_Res wrappers, bundled assets), manifest inspection, conflict warnings.\n- **Textures** — manage standalone texture packs and `load_order` priority without hand-editing TOML.\n- **Cutscenes** — install HD cutscene mods, auto-detect codec (H264 vs MPEG-2), wire up the right NAMS settings.\n- **Profiles** — keep multiple mod loadouts side-by-side, switch with one click, without copying or losing state.\n- **Diagnostics** — full report of what\'s installed where, what\'s leftover from old installs, what NAMS sees vs what\'s actually on disk.\n\n### Why we built this\n\n**Nothing against manual installs.** Dropping one outfit\'s `.dat`/`.dtt` files into the right `data/` subfolder works fine for one or two mods. The launcher is built for the scale beyond that.\n\nThe NAMS ecosystem now supports things like:\n\n- **30+ outfit mods** with multi-outfit swapping per character.\n- **20+ custom quests** added on top of the vanilla story.\n- **10+ new weapons** with their own behaviour.\n- Plus textures, cutscenes, plugins, balance changes…\n\nManaging that by hand isn\'t a philosophical preference — it\'s **just not possible**. You can\'t track which file came from which mod, you can\'t enable or disable a single mod cleanly, you can\'t tell why something broke. At scale, manual modding hits a hard wall — and the NAMS ecosystem has been past that wall for a while.\n\n### How this differs from NAMH and Vortex\n\nIf you\'ve been around the NieR scene a while, you remember how previous mod managers ended:\n\n- **NAMH** (NieR Automata Mod Helper) became unmaintained, broke games in unrecoverable ways, hit \"program in use\" lock issues, and the standard recovery was *uninstall the game, reinstall, do it manually.* It worked by **directly replacing files in `data/`** — once a NAMH install went sideways, there was no clean way back.\n- **Vortex** never properly understood NieR\'s file structure. Its virtual file system makes assumptions that don\'t match how the game actually loads content, so installs would silently misplace files.\n\nThis launcher is built differently. The defining choices:\n\n1. **No file replacement, ever.** Mods live in `nams/mods/<modId>/` and get overlaid into the engine\'s view at runtime via NAMS\'s VFS. Vanilla `data/` is never touched. There\'s no \"unrecoverable state\" because nothing in the real game ever changed.\n2. **Every action is reversible.** Uninstall a mod → its folder and bundled assets are removed cleanly. Disable a mod → an entry in `disabled_mods.toml` and NAMS skips it. No hidden state, no irreversible writes.\n3. **Profiles instead of one global state.** Past managers committed everything to a single configuration. Profiles let you keep multiple loadouts side-by-side and switch atomically — nothing to corrupt, nothing to lose.\n4. **Built on a maintained modloader.** NAMH\'s death came from the modloader story being uncertain. NAMS is the foundation everything here is built on, and the launcher follows its updates.\n\nIf this launcher ever stops being maintained, your mods are still just files on disk that NAMS reads — you don\'t end up locked out of your own install.'**
  String get modsTutorialEcosystemStep3Body;

  /// No description provided for @modsTutorialEcosystemStep4Title.
  ///
  /// In en, this message translates to:
  /// **'Where to go next'**
  String get modsTutorialEcosystemStep4Title;

  /// No description provided for @modsTutorialEcosystemStep4Body.
  ///
  /// In en, this message translates to:
  /// **'If you\'ve never installed a mod here before:\n\n- **How to install a mod** — walks through the install flow tab-by-tab.\n- **How profiles work** — explains separate loadouts and when to use them.\n\nFind both in this same help menu (the **?** icon you used to open this).\n\n**The short version:** drop archives onto the right tab (Mod Manager for character/data mods, Textures for standalone texture packs, Cutscenes for HD cutscenes), hit Play, look at Logs if something breaks. The launcher tries to do the right thing automatically — if you disagree with a choice it made, every action is reversible from the UI.'**
  String get modsTutorialEcosystemStep4Body;

  /// No description provided for @modsTutorialHelpTooltip.
  ///
  /// In en, this message translates to:
  /// **'Tutorials & help'**
  String get modsTutorialHelpTooltip;

  /// No description provided for @modsTutorialInstallStep1Title.
  ///
  /// In en, this message translates to:
  /// **'Drop your mod here'**
  String get modsTutorialInstallStep1Title;

  /// No description provided for @modsTutorialInstallStep1Body.
  ///
  /// In en, this message translates to:
  /// **'This is the **Mod Manager** tab — it\'s where character, outfit, and other gameplay mods get installed.\n\nDrag a `.zip`, `.7z`, or `.rar` from Nexus onto this drop zone (or click to browse). The launcher unpacks it, checks the layout, and puts it in the right place. You don\'t need to extract anything yourself.\n\n**Good to know:** your vanilla game files stay untouched. Mods live in a separate `nams/` folder, so you can always launch the unmodded game through Steam if you want.'**
  String get modsTutorialInstallStep1Body;

  /// No description provided for @modsTutorialInstallStep2Title.
  ///
  /// In en, this message translates to:
  /// **'Trying to install a WAX mod? Read this first'**
  String get modsTutorialInstallStep2Title;

  /// No description provided for @modsTutorialInstallStep2Body.
  ///
  /// In en, this message translates to:
  /// **'**WAX mods do work here** — NAMS reimplements WAX up to a tested compatibility version. Most mods on Nexus that target that version or older will install and run normally.\n\n### How compatibility works\n\nNAMS is validated against a specific WAX version. Anything WAX shipped up to and including that version: supported. Anything WAX adds in a **newer** version after that: not automatically — that\'s a new feature on the WAX side that has to be reimplemented from scratch on the NAMS side.\n\n### What happens when WAX adds something new\n\nWhen WAX ships a new feature in a future version, it gets evaluated against NAMS\'s roadmap:\n\n- **In scope** — if the feature fits where NAMS is already heading, it gets implemented and a future NAMS update will support mods using it.\n- **Out of scope** — NAMS has its own content extensions to focus on (custom quests, custom world maps, custom plugin chips, expanded modding APIs, and more). Reimplementing every WAX feature isn\'t the priority. Some niche WAX features may simply not get NAMS equivalents.\n\n**This isn\'t a slight against WAX.** They\'re separate projects with separate goals. NAMS isn\'t trying to be a drop-in WAX replacement — it\'s its own platform that happens to be **broadly compatible** with WAX because most users want their existing mod libraries to keep working.\n\n### This pattern is normal\n\nThis kind of split is **how every modded-game ecosystem evolves**, not a NieR-specific weirdness. Concrete example: **Skyrim Legendary Edition (LE)** and **Skyrim Special Edition (SE)** are forks of the same engine. SE is broadly compatible with LE mods, but not 100% — some LE mods need conversion, and a handful never got ported because they relied on engine quirks SE changed. The Skyrim community didn\'t treat that as a flaw; it just became part of how the ecosystem worked. Same story with **OpenMW vs vanilla Morrowind**, **Java vs Bedrock Minecraft**, **KSP1 vs KSP2 mods**, etc.\n\nWhenever a platform reimplements another platform\'s behavior, you get a compatibility envelope — most of it works, edges don\'t. That\'s the deal across every modded-game scene that\'s been around long enough to fork.\n\n### Best practice if you\'re not sure\n\n1. **Create a fresh empty profile** (see *How profiles work* in the help menu) and switch to it.\n2. **Drop the WAX mod alone** into that profile. Nothing else.\n3. **Press Play.** Works? Install it into your real profile.\n4. **Doesn\'t work?** The mod is probably using a feature past NAMS\'s tested WAX version, or one that NAMS chose not to reimplement.\n\n### What to expect\n\n- If you have features X, Y, and Z working in NAMS and the WAX mod you want needs feature W that isn\'t supported — and you can live without W — you\'ve still got X, Y, Z working fine alongside it.\n- If feature W is essential to the mod and NAMS doesn\'t have it, you\'re picking between WAX (gets you W but loses NAMS\'s other benefits) or NAMS (everything else, but not W).\n\n**And don\'t forget the other side of the trade-off:** sticking with WAX means missing out on the **NAMS-exclusive mods** that don\'t run on WAX at all — multi-outfit swapping per character, custom quests, and the broader plugin ecosystem (Multiplayer Mod, the YP devkit, future plugins). Those depend on NAMS APIs that WAX doesn\'t have. So the choice isn\'t \"NAMS minus W\" vs \"WAX with W\" — it\'s \"NAMS-ecosystem minus W\" vs \"WAX with W minus everything NAMS-exclusive.\"\n\nThat\'s a real trade-off, and it\'s yours to make. We\'re not the right people to ask whether a specific WAX-exclusive feature will get NAMS support — that\'s an ecosystem-roadmap question, best directed at the YoRHa Continuum Discord.'**
  String get modsTutorialInstallStep2Body;

  /// No description provided for @modsTutorialInstallStep3Title.
  ///
  /// In en, this message translates to:
  /// **'Your installed mods'**
  String get modsTutorialInstallStep3Title;

  /// No description provided for @modsTutorialInstallStep3Body.
  ///
  /// In en, this message translates to:
  /// **'Every mod you installed shows up here.\n\n**Toggle on the right** — enable or disable the mod. Disabling keeps it installed but tells the modloader to skip it on next launch.\n\n**Game crashing on boot?** Disable half your mods, launch, repeat. The toggles let you bisect quickly.\n\n**Warning badges** mark mods that conflict with each other (two mods replacing the same outfit, etc.) — that\'s the usual reason a game won\'t reach the title screen.'**
  String get modsTutorialInstallStep3Body;

  /// No description provided for @modsTutorialInstallStep4Title.
  ///
  /// In en, this message translates to:
  /// **'Mod details'**
  String get modsTutorialInstallStep4Title;

  /// No description provided for @modsTutorialInstallStep4Body.
  ///
  /// In en, this message translates to:
  /// **'Click any mod in the list to see its details here: author, version, what it changes, conflicts with your other mods, and any **bundled texture packs or cutscenes** that came with it.\n\nIf a mod won\'t work, the most common reasons show up here — *requires a newer NAMS version* or *conflicts with another mod*. Both are visible **before** you press Play.\n\nThe **Uninstall** button cleans the mod up properly, including its bundled extras.'**
  String get modsTutorialInstallStep4Body;

  /// No description provided for @modsTutorialInstallStep5Title.
  ///
  /// In en, this message translates to:
  /// **'Standalone texture mods → Textures tab'**
  String get modsTutorialInstallStep5Title;

  /// No description provided for @modsTutorialInstallStep5Body.
  ///
  /// In en, this message translates to:
  /// **'**Texture-only mods** (HD upscale packs, color reskins) don\'t get installed here. They live in their own tab.\n\nClick **Textures** in the sidebar to install those. Drag `.zip` archives in the same way — the launcher knows what it\'s looking at.\n\n**Note:** if a character mod *bundles* its own textures, those install automatically with the mod. You only use the Textures tab for **standalone** texture packs.'**
  String get modsTutorialInstallStep5Body;

  /// No description provided for @modsTutorialInstallStep6Title.
  ///
  /// In en, this message translates to:
  /// **'Cutscene mods → Cutscenes tab'**
  String get modsTutorialInstallStep6Title;

  /// No description provided for @modsTutorialInstallStep6Body.
  ///
  /// In en, this message translates to:
  /// **'**Cutscene mods** (HD cutscenes, replacement videos) go in their own tab too.\n\nClick **Cutscenes** in the sidebar to install those.\n\n**Same rule as textures:** if a character mod bundles cutscenes, they install automatically — you only use the Cutscenes tab for **standalone** cutscene packs.'**
  String get modsTutorialInstallStep6Body;

  /// No description provided for @modsTutorialInstallStep7Title.
  ///
  /// In en, this message translates to:
  /// **'Press Play'**
  String get modsTutorialInstallStep7Title;

  /// No description provided for @modsTutorialInstallStep7Body.
  ///
  /// In en, this message translates to:
  /// **'Head back to the **Launcher** tab and press **PLAY**. The modloader reads your mods fresh on every game start, so you don\'t need to restart this launcher.\n\n### If the game crashes\n\nOpen **Logs** (bottom-right) — the modloader\'s output usually names the broken mod. Come back here and disable it.\n\n### Still broken with everything disabled?\n\nIf you (or a previous mod manager) ever dropped loose `.dat` / `.dtt` files directly into `<gameDir>/data/`, the engine still picks those up — and the modloader can\'t see or disable them. That\'s the kind of mess this launcher specifically avoids: every mod stays isolated in `nams/mods/<modId>/` instead of overwriting real game files.\n\nOpen **Logs → Diagnostics** and check the *Vanilla data/ overlay* section. Anything listed there is leftover from an old install — move those folders out of `data/` and your game is back to a clean state.'**
  String get modsTutorialInstallStep7Body;

  /// No description provided for @modsTutorialProfilesStep1Title.
  ///
  /// In en, this message translates to:
  /// **'What profiles are for'**
  String get modsTutorialProfilesStep1Title;

  /// No description provided for @modsTutorialProfilesStep1Body.
  ///
  /// In en, this message translates to:
  /// **'Profiles let you keep multiple separate mod loadouts side by side.\n\nFor example:\n\n- A **main** profile with the mods you actually play with.\n- A **test** profile for trying out anything new.\n\nIf a sketchy new mod breaks your game, just switch back to **main** and you\'re playing again. Your loadouts never mix.\n\n**Important:** mods you\'re not actively using aren\'t deleted — they\'re just parked, ready to come back when you switch.'**
  String get modsTutorialProfilesStep1Body;

  /// No description provided for @modsTutorialProfilesStep2Title.
  ///
  /// In en, this message translates to:
  /// **'Create a profile'**
  String get modsTutorialProfilesStep2Title;

  /// No description provided for @modsTutorialProfilesStep2Body.
  ///
  /// In en, this message translates to:
  /// **'Click **NEW** in the profile bar, type a name, confirm.\n\nThe launcher creates a fresh empty profile and switches to it. Your previous profile\'s mods stay safe on disk — they\'re not gone, they\'re just parked.\n\nNow you can install whatever you want into this new profile without touching your other loadouts.'**
  String get modsTutorialProfilesStep2Body;

  /// No description provided for @modsTutorialProfilesStep3Title.
  ///
  /// In en, this message translates to:
  /// **'Switch, rename, delete'**
  String get modsTutorialProfilesStep3Title;

  /// No description provided for @modsTutorialProfilesStep3Body.
  ///
  /// In en, this message translates to:
  /// **'**Switch** — pick any profile from the dropdown. Your mod list flips over instantly.\n\n**Rename** — change a profile\'s name without losing anything.\n\n**Delete** — permanently remove an inactive profile (you can\'t delete the active one or the last remaining one).\n\nThe whole switch happens in one step — if anything goes wrong it rolls back automatically, so you can\'t end up in a broken state.'**
  String get modsTutorialProfilesStep3Body;

  /// No description provided for @modsTutorialProfilesStep4Title.
  ///
  /// In en, this message translates to:
  /// **'What follows the profile, what doesn\'t'**
  String get modsTutorialProfilesStep4Title;

  /// No description provided for @modsTutorialProfilesStep4Body.
  ///
  /// In en, this message translates to:
  /// **'**Per-profile** (changes when you switch):\n\n- Your installed mods\n- Which mods are enabled or disabled\n- Textures that came bundled with a mod\n\n**Shared between all profiles** (never changes):\n\n- Standalone texture packs you installed via the Textures tab\n- Cutscene mods\n- Plugins\n- All your launcher settings\n\nSo profiles only flip what\'s actually mod-specific. Your global setup follows you everywhere.'**
  String get modsTutorialProfilesStep4Body;

  /// No description provided for @platformUnsupportedTitle.
  ///
  /// In en, this message translates to:
  /// **'Cannot launch on this platform'**
  String get platformUnsupportedTitle;

  /// No description provided for @platformUnsupportedLinux.
  ///
  /// In en, this message translates to:
  /// **'NieR:Automata is a Windows game. Run the launcher through Wine — that has worked in the past, though it has not been re-tested recently. Proton is currently broken because it interferes with the mod\'s Steam connection. Native Linux without a translation layer cannot run the game.\n\nIf you did get it working somehow, please use the command line directly instead of this launcher.'**
  String get platformUnsupportedLinux;

  /// No description provided for @platformUnsupportedMacos.
  ///
  /// In en, this message translates to:
  /// **'NieR:Automata is a Windows game. Run the launcher through CrossOver/Wine — that has worked in the past, though it has not been re-tested recently. Native macOS without a translation layer cannot run the game.\n\nIf you did get it working somehow, please use the command line directly instead of this launcher.'**
  String get platformUnsupportedMacos;

  /// No description provided for @playDisabledTooltip.
  ///
  /// In en, this message translates to:
  /// **'Launch unavailable on this platform'**
  String get playDisabledTooltip;

  /// No description provided for @diagnosticsClose.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get diagnosticsClose;

  /// No description provided for @diagnosticsSectionDataOverlay.
  ///
  /// In en, this message translates to:
  /// **'Vanilla data/ overlay (manual drops)'**
  String get diagnosticsSectionDataOverlay;

  /// No description provided for @diagnosticsSectionExternalLegacy.
  ///
  /// In en, this message translates to:
  /// **'External / legacy'**
  String get diagnosticsSectionExternalLegacy;

  /// No description provided for @diagnosticsSectionGameRootExtras.
  ///
  /// In en, this message translates to:
  /// **'Game root extras (non-vanilla)'**
  String get diagnosticsSectionGameRootExtras;

  /// No description provided for @diagnosticsFileCount.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 file} other{{count} files}}'**
  String diagnosticsFileCount(int count);

  /// No description provided for @diagnosticsMoreItems.
  ///
  /// In en, this message translates to:
  /// **'... {count} more'**
  String diagnosticsMoreItems(int count);

  /// No description provided for @refreshButton.
  ///
  /// In en, this message translates to:
  /// **'Refresh'**
  String get refreshButton;

  /// No description provided for @tabModloaderLabel.
  ///
  /// In en, this message translates to:
  /// **'Modloader'**
  String get tabModloaderLabel;

  /// No description provided for @tabYorhaLabel.
  ///
  /// In en, this message translates to:
  /// **'YoRHa Protocol'**
  String get tabYorhaLabel;

  /// No description provided for @configEditorTitle.
  ///
  /// In en, this message translates to:
  /// **'CONFIG EDITOR'**
  String get configEditorTitle;

  /// No description provided for @changelogTitle.
  ///
  /// In en, this message translates to:
  /// **'WHAT\'S NEW'**
  String get changelogTitle;

  /// No description provided for @tipDragTextures.
  ///
  /// In en, this message translates to:
  /// **'Drag texture mods directly into the Textures tab'**
  String get tipDragTextures;

  /// No description provided for @tipHdCutscenes.
  ///
  /// In en, this message translates to:
  /// **'HD cutscene mods are auto-detected and configured'**
  String get tipHdCutscenes;

  /// No description provided for @tipLodModPreviews.
  ///
  /// In en, this message translates to:
  /// **'LOD Mod settings come with before/after preview images'**
  String get tipLodModPreviews;

  /// No description provided for @tipFaqButton.
  ///
  /// In en, this message translates to:
  /// **'Use the FAQ button to see which mods YoRHa Protocol replaces'**
  String get tipFaqButton;

  /// No description provided for @tipReShadeAuto.
  ///
  /// In en, this message translates to:
  /// **'ReShade is auto-detected — no manual config needed'**
  String get tipReShadeAuto;

  /// No description provided for @tipFreecam.
  ///
  /// In en, this message translates to:
  /// **'YoRHa Protocol includes a built-in freecam with save slots'**
  String get tipFreecam;

  /// No description provided for @tipCustomQuests.
  ///
  /// In en, this message translates to:
  /// **'Custom quests are coming soon — stay tuned'**
  String get tipCustomQuests;

  /// No description provided for @sectionNams.
  ///
  /// In en, this message translates to:
  /// **'NAMS'**
  String get sectionNams;

  /// No description provided for @sectionTextureInjection.
  ///
  /// In en, this message translates to:
  /// **'TEXTURE INJECTION'**
  String get sectionTextureInjection;

  /// No description provided for @sectionLodMod.
  ///
  /// In en, this message translates to:
  /// **'LOD MOD'**
  String get sectionLodMod;

  /// No description provided for @sectionLevelOfDetail.
  ///
  /// In en, this message translates to:
  /// **'LEVEL OF DETAIL'**
  String get sectionLevelOfDetail;

  /// No description provided for @sectionAmbientOcclusion.
  ///
  /// In en, this message translates to:
  /// **'AMBIENT OCCLUSION'**
  String get sectionAmbientOcclusion;

  /// No description provided for @sectionShadows.
  ///
  /// In en, this message translates to:
  /// **'SHADOWS'**
  String get sectionShadows;

  /// No description provided for @sectionPostProcessing.
  ///
  /// In en, this message translates to:
  /// **'POST-PROCESSING'**
  String get sectionPostProcessing;

  /// No description provided for @labelValidateModelData.
  ///
  /// In en, this message translates to:
  /// **'Validate Model Data'**
  String get labelValidateModelData;

  /// No description provided for @tooltipValidateModelData.
  ///
  /// In en, this message translates to:
  /// **'The game validates model data when loading. Normally it silently fails and continues with broken data, which can cause invisible models or glitches. When enabled, NAMS surfaces the validation result as a dialog so you can see exactly which model failed and why.'**
  String get tooltipValidateModelData;

  /// No description provided for @labelPreloadMaxDimension.
  ///
  /// In en, this message translates to:
  /// **'Preload Max Dimension'**
  String get labelPreloadMaxDimension;

  /// No description provided for @tooltipPreloadMaxDimension.
  ///
  /// In en, this message translates to:
  /// **'Max texture size to preload into RAM at startup. 2048 = default, 4096 = preload up to 4K textures, 16384 = preload everything. Higher = longer loading but less stutter in-game.'**
  String get tooltipPreloadMaxDimension;

  /// No description provided for @labelPreloadAllTextures.
  ///
  /// In en, this message translates to:
  /// **'Preload All Textures'**
  String get labelPreloadAllTextures;

  /// No description provided for @tooltipPreloadAllTextures.
  ///
  /// In en, this message translates to:
  /// **'Preload ALL textures into RAM regardless of size. Eliminates all texture pop-in stutter but needs 32GB+ RAM and makes startup significantly slower.'**
  String get tooltipPreloadAllTextures;

  /// No description provided for @labelEnableLodMod.
  ///
  /// In en, this message translates to:
  /// **'Enable LodMod'**
  String get labelEnableLodMod;

  /// No description provided for @tooltipEnableLodMod.
  ///
  /// In en, this message translates to:
  /// **'Master toggle for all LodMod visual patches/rewrites.'**
  String get tooltipEnableLodMod;

  /// No description provided for @labelLodMultiplier.
  ///
  /// In en, this message translates to:
  /// **'LOD Multiplier'**
  String get labelLodMultiplier;

  /// No description provided for @tooltipLodMultiplier.
  ///
  /// In en, this message translates to:
  /// **'Controls LOD (Level of Detail) draw distances. 0 = LODs disabled (best quality, no pop-in). 1 = vanilla. 10+ helps reduce AO bleed without fully disabling LODs. Lower values = better visuals but may cost performance.'**
  String get tooltipLodMultiplier;

  /// No description provided for @labelDisableManualCulling.
  ///
  /// In en, this message translates to:
  /// **'Disable Manual Culling'**
  String get labelDisableManualCulling;

  /// No description provided for @tooltipDisableManualCulling.
  ///
  /// In en, this message translates to:
  /// **'Prevents models/geometry from randomly disappearing at certain distances or camera angles. Fixes things like the mall interior vanishing after crossing the bridge, buildings outside camp disappearing, etc. Rare ugly LOD models that would show up are filtered out.'**
  String get tooltipDisableManualCulling;

  /// No description provided for @labelAoWidth.
  ///
  /// In en, this message translates to:
  /// **'AO Width'**
  String get labelAoWidth;

  /// No description provided for @tooltipAoWidth.
  ///
  /// In en, this message translates to:
  /// **'Multiplier for AO horizontal resolution. Vanilla AO runs at 1/4 screen res. 2.0 = half screen res (crisper AO but heavy). 1.5 is a good balance. Range: 0.1 - 2.0. Setting only one axis to 2 can be a lighter alternative.'**
  String get tooltipAoWidth;

  /// No description provided for @labelAoHeight.
  ///
  /// In en, this message translates to:
  /// **'AO Height'**
  String get labelAoHeight;

  /// No description provided for @tooltipAoHeight.
  ///
  /// In en, this message translates to:
  /// **'Multiplier for AO vertical resolution. Vanilla AO runs at 1/4 screen res. 2.0 = half screen res (crisper AO but heavy). 1.5 is a good balance. Range: 0.1 - 2.0. Both at 2.0 can cost ~10 FPS in worst case.'**
  String get tooltipAoHeight;

  /// No description provided for @labelShadowResolution.
  ///
  /// In en, this message translates to:
  /// **'Shadow Resolution'**
  String get labelShadowResolution;

  /// No description provided for @tooltipShadowResolution.
  ///
  /// In en, this message translates to:
  /// **'Shadow map texture size. Higher = sharper shadows but heavier on GPU. 2048 = vanilla, 4096 = good upgrade, 8192 = very sharp. Must be power of 2. Sharpness depends on both resolution and distance (larger distance = more area to fit, so quality decreases).'**
  String get tooltipShadowResolution;

  /// No description provided for @labelDistanceMultiplier.
  ///
  /// In en, this message translates to:
  /// **'Distance Multiplier'**
  String get labelDistanceMultiplier;

  /// No description provided for @tooltipDistanceMultiplier.
  ///
  /// In en, this message translates to:
  /// **'Multiplies the per-scene shadow draw distance. 2.0 = shadows visible twice as far. Vanilla: 1.0. Disable Min/Max below for this to work properly, or use them to restrict the range this multiplier sets.'**
  String get tooltipDistanceMultiplier;

  /// No description provided for @labelDistanceMinimum.
  ///
  /// In en, this message translates to:
  /// **'Distance Minimum'**
  String get labelDistanceMinimum;

  /// No description provided for @tooltipDistanceMinimum.
  ///
  /// In en, this message translates to:
  /// **'Minimum shadow draw distance clamp. 0 = off (no minimum). Setting to ~70 with 8192 resolution matches vanilla quality while greatly increasing shadow distance.'**
  String get tooltipDistanceMinimum;

  /// No description provided for @labelDistanceMaximum.
  ///
  /// In en, this message translates to:
  /// **'Distance Maximum'**
  String get labelDistanceMaximum;

  /// No description provided for @tooltipDistanceMaximum.
  ///
  /// In en, this message translates to:
  /// **'Maximum shadow draw distance clamp. 0 = off (no maximum). Only worth setting if the default game distances cause performance issues.'**
  String get tooltipDistanceMaximum;

  /// No description provided for @labelDistancePss.
  ///
  /// In en, this message translates to:
  /// **'Distance PSS'**
  String get labelDistancePss;

  /// No description provided for @tooltipDistancePss.
  ///
  /// In en, this message translates to:
  /// **'Enables PSS shadow distribution for more even shadow quality. 0 = off. Good values: 0.5 - 0.9. Looks great in some areas but may appear blurry in others. Should be set much larger than other distance values (~1500 for large open areas).'**
  String get tooltipDistancePss;

  /// No description provided for @labelFilterStrengthBias.
  ///
  /// In en, this message translates to:
  /// **'Filter Strength Bias'**
  String get labelFilterStrengthBias;

  /// No description provided for @tooltipFilterStrengthBias.
  ///
  /// In en, this message translates to:
  /// **'Adjusts shadow blur filter strength per-scene. 0 = off. -1 = sharper shadows. Positive = softer. Different areas use different strengths (forest = softer). Can be combined with Min/Max to restrict the range.'**
  String get tooltipFilterStrengthBias;

  /// No description provided for @labelFilterStrengthMin.
  ///
  /// In en, this message translates to:
  /// **'Filter Strength Min'**
  String get labelFilterStrengthMin;

  /// No description provided for @tooltipFilterStrengthMin.
  ///
  /// In en, this message translates to:
  /// **'Forces a minimum shadow filter strength across all areas. 0 = off. Game default varies per scene (usually ~4). Use to prevent shadows from being too sharp in any area.'**
  String get tooltipFilterStrengthMin;

  /// No description provided for @labelFilterStrengthMax.
  ///
  /// In en, this message translates to:
  /// **'Filter Strength Max'**
  String get labelFilterStrengthMax;

  /// No description provided for @tooltipFilterStrengthMax.
  ///
  /// In en, this message translates to:
  /// **'Forces a maximum shadow filter strength across all areas. 0 = off. Game default varies per scene (usually ~4). Use to prevent shadows from being too blurry in any area.'**
  String get tooltipFilterStrengthMax;

  /// No description provided for @labelHqShadowModels.
  ///
  /// In en, this message translates to:
  /// **'HQ Shadow Models'**
  String get labelHqShadowModels;

  /// No description provided for @tooltipHqShadowModels.
  ///
  /// In en, this message translates to:
  /// **'Uses real-time HQ models for shadows instead of static LQ models. Tree shadows will sway with the wind instead of being frozen. Experimental — works well in city ruins, could cause issues in rare areas.'**
  String get tooltipHqShadowModels;

  /// No description provided for @labelForceAllShadowModels.
  ///
  /// In en, this message translates to:
  /// **'Force All Shadow Models'**
  String get labelForceAllShadowModels;

  /// No description provided for @tooltipForceAllShadowModels.
  ///
  /// In en, this message translates to:
  /// **'Forces all models to cast shadows, including small objects like rocks and grass. Experimental — may rarely cause invisible models to cast shadows. No issues noticed so far.'**
  String get tooltipForceAllShadowModels;

  /// No description provided for @labelDisableVignette.
  ///
  /// In en, this message translates to:
  /// **'Disable Vignette'**
  String get labelDisableVignette;

  /// No description provided for @tooltipDisableVignette.
  ///
  /// In en, this message translates to:
  /// **'Removes the dark vignette effect on screen edges. Some loading screens may still have it baked into textures.'**
  String get tooltipDisableVignette;

  /// No description provided for @configAppliesOnRestart.
  ///
  /// In en, this message translates to:
  /// **'Applies on restart'**
  String get configAppliesOnRestart;

  /// No description provided for @configAppliesLive.
  ///
  /// In en, this message translates to:
  /// **'Applies instantly (live)'**
  String get configAppliesLive;

  /// No description provided for @dropZoneBrowseFolder.
  ///
  /// In en, this message translates to:
  /// **'Or pick a folder'**
  String get dropZoneBrowseFolder;

  /// No description provided for @labelGiEnabled.
  ///
  /// In en, this message translates to:
  /// **'Enable Global Illumination'**
  String get labelGiEnabled;

  /// No description provided for @tooltipGiEnabled.
  ///
  /// In en, this message translates to:
  /// **'FAR-style global illumination. Big FPS gain at the cost of some lighting accuracy.'**
  String get tooltipGiEnabled;

  /// No description provided for @labelGiWorkgroupSize.
  ///
  /// In en, this message translates to:
  /// **'GI Workgroup Size'**
  String get labelGiWorkgroupSize;

  /// No description provided for @tooltipGiWorkgroupSize.
  ///
  /// In en, this message translates to:
  /// **'Number of light volumes processed per GI dispatch. 128 = vanilla quality, 64/32/16 = progressively faster but coarser. Lower values trade lighting fidelity for FPS.'**
  String get tooltipGiWorkgroupSize;

  /// No description provided for @labelGiMinLightExtent.
  ///
  /// In en, this message translates to:
  /// **'GI Min Light Extent'**
  String get labelGiMinLightExtent;

  /// No description provided for @tooltipGiMinLightExtent.
  ///
  /// In en, this message translates to:
  /// **'Cull small distant lights from GI. 0.0 = no culling (all lights contribute), 0.5 = aggressive culling. Range 0.0-1.0.'**
  String get tooltipGiMinLightExtent;

  /// No description provided for @cardExperimental.
  ///
  /// In en, this message translates to:
  /// **'EXPERIMENTAL'**
  String get cardExperimental;

  /// No description provided for @lodModResetButton.
  ///
  /// In en, this message translates to:
  /// **'Reset to defaults'**
  String get lodModResetButton;

  /// No description provided for @lodModResetConfirmTitle.
  ///
  /// In en, this message translates to:
  /// **'Reset LodMod settings?'**
  String get lodModResetConfirmTitle;

  /// No description provided for @lodModResetConfirmBody.
  ///
  /// In en, this message translates to:
  /// **'This will reset every LodMod field on this tab to its default value. Your current values will be overwritten. Continue?'**
  String get lodModResetConfirmBody;

  /// No description provided for @lodModResetConfirmAction.
  ///
  /// In en, this message translates to:
  /// **'Reset'**
  String get lodModResetConfirmAction;

  /// No description provided for @lodModResetToast.
  ///
  /// In en, this message translates to:
  /// **'LodMod settings reset to defaults'**
  String get lodModResetToast;

  /// No description provided for @experimentalWarningTitle.
  ///
  /// In en, this message translates to:
  /// **'Experimental — will break things'**
  String get experimentalWarningTitle;

  /// No description provided for @experimentalWarningBody.
  ///
  /// In en, this message translates to:
  /// **'These settings bypass game limits the engine relies on. They are NOT supported and are known to cause issues. Only enable if you understand what you\'re doing. NAMS and the launcher will not be debugged around problems caused by these.'**
  String get experimentalWarningBody;

  /// No description provided for @labelFpsUncapInMenus.
  ///
  /// In en, this message translates to:
  /// **'Uncap FPS in menus / loading'**
  String get labelFpsUncapInMenus;

  /// No description provided for @tooltipFpsUncapInMenus.
  ///
  /// In en, this message translates to:
  /// **'Removes the 60 FPS lock during menus and loading screens. Loading feels faster and menu animations get smoother. Safe: gameplay is unaffected.\n\nLive-toggleable if it was enabled at game launch. If it was disabled at launch, toggling it on later requires a restart.'**
  String get tooltipFpsUncapInMenus;

  /// No description provided for @labelFpsUncapInGameplay.
  ///
  /// In en, this message translates to:
  /// **'Uncap FPS in gameplay'**
  String get labelFpsUncapInGameplay;

  /// No description provided for @tooltipFpsUncapInGameplay.
  ///
  /// In en, this message translates to:
  /// **'Removes the 60 FPS lock during gameplay. WARNING: NieR:Automata\'s physics, animations, and cutscene timing are tied to the 60 FPS lock. Uncapping causes broken physics (jumping height, dodge i-frames), animation speed changes, audio desync in cutscenes, and softlocks in scripted sequences. Use only if you know exactly what trade-offs you\'re accepting.\n\nLive-toggleable if it was enabled at game launch. If it was disabled at launch, toggling it on later requires a restart.'**
  String get tooltipFpsUncapInGameplay;

  /// No description provided for @labelFpsLimit.
  ///
  /// In en, this message translates to:
  /// **'FPS Limit'**
  String get labelFpsLimit;

  /// No description provided for @tooltipFpsLimit.
  ///
  /// In en, this message translates to:
  /// **'FPS cap applied when uncap is active. 0 = unlimited. Otherwise 60-1000 (NAMS clamps values out of range). Values below 60 are clamped because the game\'s internal spin-wait loop ignores frametimes longer than the vanilla 60fps target. Tip: capping at half your monitor\'s refresh rate gives smoother motion than vanilla 60 (e.g. 72 on 144Hz, 82 on 165Hz, 120 on 240Hz).'**
  String get tooltipFpsLimit;

  /// No description provided for @tutorialValidateModel.
  ///
  /// In en, this message translates to:
  /// **'Shows you when a mod\'s model is broken instead of silently failing.'**
  String get tutorialValidateModel;

  /// No description provided for @labelValidateScripts.
  ///
  /// In en, this message translates to:
  /// **'Validate Scripts'**
  String get labelValidateScripts;

  /// No description provided for @tooltipValidateScripts.
  ///
  /// In en, this message translates to:
  /// **'Show script errors as dialog instead of silently crashing.'**
  String get tooltipValidateScripts;

  /// No description provided for @previewValidationDialog.
  ///
  /// In en, this message translates to:
  /// **'VALIDATION DIALOG'**
  String get previewValidationDialog;

  /// No description provided for @previewScriptErrorDialog.
  ///
  /// In en, this message translates to:
  /// **'SCRIPT ERROR DIALOG'**
  String get previewScriptErrorDialog;

  /// No description provided for @labelLoadingStallHints.
  ///
  /// In en, this message translates to:
  /// **'Loading Stall Hints'**
  String get labelLoadingStallHints;

  /// No description provided for @tooltipLoadingStallHints.
  ///
  /// In en, this message translates to:
  /// **'Show escalating hints when the loading screen takes too long. Helps identify missing or deleted mod files.'**
  String get tooltipLoadingStallHints;

  /// No description provided for @labelFixWindTimerBug.
  ///
  /// In en, this message translates to:
  /// **'Fix Wind Timer Bug'**
  String get labelFixWindTimerBug;

  /// No description provided for @tooltipFixWindTimerBug.
  ///
  /// In en, this message translates to:
  /// **'Fixes a vanilla bug where wind animation stops after max playtime. Uses the game\'s speed factor instead.'**
  String get tooltipFixWindTimerBug;

  /// No description provided for @labelDisablePluginLoading.
  ///
  /// In en, this message translates to:
  /// **'Disable Plugin Loading'**
  String get labelDisablePluginLoading;

  /// No description provided for @tooltipDisablePluginLoading.
  ///
  /// In en, this message translates to:
  /// **'Skip loading all plugin DLLs (e.g. YoRHa Protocol workspace). All NAMS engine features still work.'**
  String get tooltipDisablePluginLoading;

  /// No description provided for @labelDisableContentFeatures.
  ///
  /// In en, this message translates to:
  /// **'Disable Content Features'**
  String get labelDisableContentFeatures;

  /// No description provided for @tooltipDisableContentFeatures.
  ///
  /// In en, this message translates to:
  /// **'Master switch for all content-layer features. When on, NAMS runs as a pure engine layer (mouse fixes, validation, heap tuning, crash fixes) without any item / weapon / outfit / quest / accessory mod support. Useful for benchmarking or isolating engine vs content issues.'**
  String get tooltipDisableContentFeatures;

  /// No description provided for @labelContentItems.
  ///
  /// In en, this message translates to:
  /// **'Items / Weapons / Shops'**
  String get labelContentItems;

  /// No description provided for @tooltipContentItems.
  ///
  /// In en, this message translates to:
  /// **'Custom items, weapons, outfits and shop entries. Disable to play without any item-related mods. Requires restart.'**
  String get tooltipContentItems;

  /// No description provided for @labelContentAccessories.
  ///
  /// In en, this message translates to:
  /// **'Accessories'**
  String get labelContentAccessories;

  /// No description provided for @tooltipContentAccessories.
  ///
  /// In en, this message translates to:
  /// **'Custom accessories (face masks, lunar tear, masamune mask, etc.) and the accessory equip / unequip flow. Disable to play without accessory mods. Requires restart.'**
  String get tooltipContentAccessories;

  /// No description provided for @labelContentAssembleMeshes.
  ///
  /// In en, this message translates to:
  /// **'Player Meshes'**
  String get labelContentAssembleMeshes;

  /// No description provided for @tooltipContentAssembleMeshes.
  ///
  /// In en, this message translates to:
  /// **'Custom player meshes (mesh swaps, hair / outfit / mask overrides). Disable to render the vanilla player models unchanged. Requires restart.'**
  String get tooltipContentAssembleMeshes;

  /// No description provided for @labelContentQuestIntegration.
  ///
  /// In en, this message translates to:
  /// **'Quests / Mails / Voice'**
  String get labelContentQuestIntegration;

  /// No description provided for @tooltipContentQuestIntegration.
  ///
  /// In en, this message translates to:
  /// **'Custom quests, custom mails, custom voice lines, and the quest UI integration that activates them. Disable to play without quest mods. Requires restart.'**
  String get tooltipContentQuestIntegration;

  /// No description provided for @labelContentEffectsApplier.
  ///
  /// In en, this message translates to:
  /// **'Effect Rules'**
  String get labelContentEffectsApplier;

  /// No description provided for @tooltipContentEffectsApplier.
  ///
  /// In en, this message translates to:
  /// **'Per-frame application of weapon / outfit effect rules to player stats (damage multipliers, dodge tweaks, immunities, etc.).'**
  String get tooltipContentEffectsApplier;

  /// No description provided for @labelContentEquipTracker.
  ///
  /// In en, this message translates to:
  /// **'Equip Tracker'**
  String get labelContentEquipTracker;

  /// No description provided for @tooltipContentEquipTracker.
  ///
  /// In en, this message translates to:
  /// **'Detection of weapon equip / unequip changes. Drives effect rules and equip-time SDK callbacks.'**
  String get tooltipContentEquipTracker;

  /// No description provided for @labelContentMcd.
  ///
  /// In en, this message translates to:
  /// **'Custom Text'**
  String get labelContentMcd;

  /// No description provided for @tooltipContentMcd.
  ///
  /// In en, this message translates to:
  /// **'In-game text customization (custom item names, descriptions, dialogue strings supplied by mods).'**
  String get tooltipContentMcd;

  /// No description provided for @labelContentBuddyRubySelector.
  ///
  /// In en, this message translates to:
  /// **'Buddy Outfit Selector (experimental)'**
  String get labelContentBuddyRubySelector;

  /// No description provided for @tooltipContentBuddyRubySelector.
  ///
  /// In en, this message translates to:
  /// **'Patches the global buddy conversation script to add a \"Change outfit\" entry that lists modded buddy outfits. Disable if the patched conversation script causes instability or interferes with other script mods.'**
  String get tooltipContentBuddyRubySelector;

  /// No description provided for @cardContentFeatures.
  ///
  /// In en, this message translates to:
  /// **'CONTENT FEATURES'**
  String get cardContentFeatures;

  /// No description provided for @contentFeaturesDescription.
  ///
  /// In en, this message translates to:
  /// **'Per-feature toggles for the content layer. All default to ON. Useful for narrowing a problem down to a specific subsystem. Requires a game restart.'**
  String get contentFeaturesDescription;

  /// No description provided for @labelDisableReShadeLoading.
  ///
  /// In en, this message translates to:
  /// **'Disable ReShade Loading'**
  String get labelDisableReShadeLoading;

  /// No description provided for @tooltipDisableReShadeLoading.
  ///
  /// In en, this message translates to:
  /// **'Skip automatic ReShade DLL detection from the reshade/ folder and does not load it anymore.'**
  String get tooltipDisableReShadeLoading;

  /// No description provided for @labelDisableTextureInjection.
  ///
  /// In en, this message translates to:
  /// **'Disable Texture Injection'**
  String get labelDisableTextureInjection;

  /// No description provided for @tooltipDisableTextureInjection.
  ///
  /// In en, this message translates to:
  /// **'Skip texture injection from the mods folder. Useful for isolating issues or if you don\'t want to use texture mods even though they are installed.'**
  String get tooltipDisableTextureInjection;

  /// No description provided for @tooltipValidateModelDataSettings.
  ///
  /// In en, this message translates to:
  /// **'Surfaces model validation errors as a dialog instead of silent failure.'**
  String get tooltipValidateModelDataSettings;

  /// No description provided for @heapDefault.
  ///
  /// In en, this message translates to:
  /// **'Default'**
  String get heapDefault;

  /// No description provided for @heapPlus16MB.
  ///
  /// In en, this message translates to:
  /// **'+16 MB'**
  String get heapPlus16MB;

  /// No description provided for @heapPlus32MB.
  ///
  /// In en, this message translates to:
  /// **'+32 MB'**
  String get heapPlus32MB;

  /// No description provided for @heapPlus64MB.
  ///
  /// In en, this message translates to:
  /// **'+64 MB'**
  String get heapPlus64MB;

  /// No description provided for @heapPlus128MB.
  ///
  /// In en, this message translates to:
  /// **'+128 MB'**
  String get heapPlus128MB;

  /// No description provided for @heapPlus256MB.
  ///
  /// In en, this message translates to:
  /// **'+256 MB'**
  String get heapPlus256MB;

  /// No description provided for @heapCustomMB.
  ///
  /// In en, this message translates to:
  /// **'+{mb} MB'**
  String heapCustomMB(String mb);

  /// No description provided for @heapScriptEngine.
  ///
  /// In en, this message translates to:
  /// **'Script Engine'**
  String get heapScriptEngine;

  /// No description provided for @heapScriptEngineDesc.
  ///
  /// In en, this message translates to:
  /// **'For complex script mods (mRuby / HAP).'**
  String get heapScriptEngineDesc;

  /// No description provided for @heapPlayerModels.
  ///
  /// In en, this message translates to:
  /// **'Player Models'**
  String get heapPlayerModels;

  /// No description provided for @heapPlayerModelsDesc.
  ///
  /// In en, this message translates to:
  /// **'For large player model replacement mods.'**
  String get heapPlayerModelsDesc;

  /// No description provided for @heapPlayerTextures.
  ///
  /// In en, this message translates to:
  /// **'Player Textures'**
  String get heapPlayerTextures;

  /// No description provided for @heapPlayerTexturesDesc.
  ///
  /// In en, this message translates to:
  /// **'For high-res player texture mods.'**
  String get heapPlayerTexturesDesc;

  /// No description provided for @heapEnemyBgModels.
  ///
  /// In en, this message translates to:
  /// **'Enemy/BG Models'**
  String get heapEnemyBgModels;

  /// No description provided for @heapEnemyBgModelsDesc.
  ///
  /// In en, this message translates to:
  /// **'For enemy or environment model mods.'**
  String get heapEnemyBgModelsDesc;

  /// No description provided for @heapEnemyBgTextures.
  ///
  /// In en, this message translates to:
  /// **'Enemy/BG Textures'**
  String get heapEnemyBgTextures;

  /// No description provided for @heapEnemyBgTexturesDesc.
  ///
  /// In en, this message translates to:
  /// **'For high-res enemy/environment textures.'**
  String get heapEnemyBgTexturesDesc;

  /// No description provided for @tutorialLodModEnable.
  ///
  /// In en, this message translates to:
  /// **'Turn this on for better visuals. This is the most impactful setting.'**
  String get tutorialLodModEnable;

  /// No description provided for @tutorialLodModShadowRes.
  ///
  /// In en, this message translates to:
  /// **'Higher values mean sharper shadows. 8192 is recommended.'**
  String get tutorialLodModShadowRes;

  /// No description provided for @tutorialLodModComparison.
  ///
  /// In en, this message translates to:
  /// **'Click any comparison to see the difference fullscreen.'**
  String get tutorialLodModComparison;

  /// No description provided for @comparisonVanilla.
  ///
  /// In en, this message translates to:
  /// **'VANILLA'**
  String get comparisonVanilla;

  /// No description provided for @comparisonDefaultEnabled.
  ///
  /// In en, this message translates to:
  /// **'DEFAULT (ENABLED)'**
  String get comparisonDefaultEnabled;

  /// No description provided for @comparisonAo05x.
  ///
  /// In en, this message translates to:
  /// **'AO 0.5x'**
  String get comparisonAo05x;

  /// No description provided for @comparisonAo20x.
  ///
  /// In en, this message translates to:
  /// **'AO 2.0x'**
  String get comparisonAo20x;

  /// No description provided for @comparisonVignetteOn.
  ///
  /// In en, this message translates to:
  /// **'VIGNETTE ON'**
  String get comparisonVignetteOn;

  /// No description provided for @comparisonVignetteOff.
  ///
  /// In en, this message translates to:
  /// **'VIGNETTE OFF'**
  String get comparisonVignetteOff;

  /// No description provided for @comparison2048.
  ///
  /// In en, this message translates to:
  /// **'2048'**
  String get comparison2048;

  /// No description provided for @comparison8192.
  ///
  /// In en, this message translates to:
  /// **'8192'**
  String get comparison8192;

  /// No description provided for @comparisonDefault.
  ///
  /// In en, this message translates to:
  /// **'DEFAULT'**
  String get comparisonDefault;

  /// No description provided for @comparison20x.
  ///
  /// In en, this message translates to:
  /// **'2.0x'**
  String get comparison20x;

  /// No description provided for @comparisonPssMinus5.
  ///
  /// In en, this message translates to:
  /// **'PSS -5.0'**
  String get comparisonPssMinus5;

  /// No description provided for @comparisonBiasMinus5.
  ///
  /// In en, this message translates to:
  /// **'BIAS -5.0'**
  String get comparisonBiasMinus5;

  /// No description provided for @comparisonOff.
  ///
  /// In en, this message translates to:
  /// **'OFF'**
  String get comparisonOff;

  /// No description provided for @comparison30.
  ///
  /// In en, this message translates to:
  /// **'3.0'**
  String get comparison30;

  /// No description provided for @comparisonHqForceAll.
  ///
  /// In en, this message translates to:
  /// **'HQ + FORCE ALL'**
  String get comparisonHqForceAll;

  /// No description provided for @tutorialKeybind.
  ///
  /// In en, this message translates to:
  /// **'Click to change the keybind. Press any key to assign it.'**
  String get tutorialKeybind;

  /// No description provided for @tutorialDamageMultiplier.
  ///
  /// In en, this message translates to:
  /// **'Tweak gameplay — stack damage, enable infinite HP, and more.'**
  String get tutorialDamageMultiplier;

  /// No description provided for @labelOpenWorkspace.
  ///
  /// In en, this message translates to:
  /// **'Open Workspace'**
  String get labelOpenWorkspace;

  /// No description provided for @tooltipOpenWorkspace.
  ///
  /// In en, this message translates to:
  /// **'Open the YoRHa Protocol workspace in-game.'**
  String get tooltipOpenWorkspace;

  /// No description provided for @labelFreezeGame.
  ///
  /// In en, this message translates to:
  /// **'Freeze Game'**
  String get labelFreezeGame;

  /// No description provided for @tooltipFreezeGame.
  ///
  /// In en, this message translates to:
  /// **'Freeze/unfreeze the game. Useful for screenshots and frame stepping.'**
  String get tooltipFreezeGame;

  /// No description provided for @labelMaxSpeed.
  ///
  /// In en, this message translates to:
  /// **'Max Speed'**
  String get labelMaxSpeed;

  /// No description provided for @tooltipMaxSpeed.
  ///
  /// In en, this message translates to:
  /// **'Toggle maximum game speed for fast travel or testing.'**
  String get tooltipMaxSpeed;

  /// No description provided for @labelFreeCam.
  ///
  /// In en, this message translates to:
  /// **'Free Cam'**
  String get labelFreeCam;

  /// No description provided for @tooltipFreeCam.
  ///
  /// In en, this message translates to:
  /// **'Toggle free camera. Full keyboard/mouse and controller support.'**
  String get tooltipFreeCam;

  /// No description provided for @labelPhaseJump.
  ///
  /// In en, this message translates to:
  /// **'Phase Jump'**
  String get labelPhaseJump;

  /// No description provided for @tooltipPhaseJump.
  ///
  /// In en, this message translates to:
  /// **'Jump to a preselected game phase/scene. Set the target in-game.'**
  String get tooltipPhaseJump;

  /// No description provided for @labelToggleInput.
  ///
  /// In en, this message translates to:
  /// **'Toggle Input'**
  String get labelToggleInput;

  /// No description provided for @tooltipToggleInput.
  ///
  /// In en, this message translates to:
  /// **'Toggle game input on/off while the workspace is open.'**
  String get tooltipToggleInput;

  /// No description provided for @labelAdvanceFrame.
  ///
  /// In en, this message translates to:
  /// **'Advance Frame'**
  String get labelAdvanceFrame;

  /// No description provided for @tooltipAdvanceFrame.
  ///
  /// In en, this message translates to:
  /// **'Step one frame forward while the game is frozen. Hold to advance faster.'**
  String get tooltipAdvanceFrame;

  /// No description provided for @labelDevMode.
  ///
  /// In en, this message translates to:
  /// **'Dev Mode'**
  String get labelDevMode;

  /// No description provided for @tooltipDevMode.
  ///
  /// In en, this message translates to:
  /// **'Toggle developer mode. Enables penetration/stress test buttons and debug tools.'**
  String get tooltipDevMode;

  /// No description provided for @labelGlobalKeybinds.
  ///
  /// In en, this message translates to:
  /// **'Global Keybinds'**
  String get labelGlobalKeybinds;

  /// No description provided for @tooltipGlobalKeybinds.
  ///
  /// In en, this message translates to:
  /// **'Keybinds work while the workspace is closed.'**
  String get tooltipGlobalKeybinds;

  /// No description provided for @labelLoadingSpeedup.
  ///
  /// In en, this message translates to:
  /// **'Loading Speedup'**
  String get labelLoadingSpeedup;

  /// No description provided for @tooltipLoadingSpeedup.
  ///
  /// In en, this message translates to:
  /// **'Faster loading screens.'**
  String get tooltipLoadingSpeedup;

  /// No description provided for @labelShaders.
  ///
  /// In en, this message translates to:
  /// **'Shaders'**
  String get labelShaders;

  /// No description provided for @tooltipShaders.
  ///
  /// In en, this message translates to:
  /// **'Workspace shaders. Toggle off for performance.'**
  String get tooltipShaders;

  /// No description provided for @labelSound.
  ///
  /// In en, this message translates to:
  /// **'Sound'**
  String get labelSound;

  /// No description provided for @tooltipSound.
  ///
  /// In en, this message translates to:
  /// **'Workspace UI interaction sound.'**
  String get tooltipSound;

  /// No description provided for @labelDamageMultiplier.
  ///
  /// In en, this message translates to:
  /// **'Damage Multiplier'**
  String get labelDamageMultiplier;

  /// No description provided for @tooltipDamageMultiplier.
  ///
  /// In en, this message translates to:
  /// **'2.0 = double damage.'**
  String get tooltipDamageMultiplier;

  /// No description provided for @labelSyncEnemyLevels.
  ///
  /// In en, this message translates to:
  /// **'Sync Enemy Levels'**
  String get labelSyncEnemyLevels;

  /// No description provided for @tooltipSyncEnemyLevels.
  ///
  /// In en, this message translates to:
  /// **'Match enemy levels to yours.'**
  String get tooltipSyncEnemyLevels;

  /// No description provided for @labelInfiniteHealth.
  ///
  /// In en, this message translates to:
  /// **'Infinite Health'**
  String get labelInfiniteHealth;

  /// No description provided for @tooltipInfiniteHealth.
  ///
  /// In en, this message translates to:
  /// **'Take no damage.'**
  String get tooltipInfiniteHealth;

  /// No description provided for @labelInfiniteJump.
  ///
  /// In en, this message translates to:
  /// **'Infinite Jump'**
  String get labelInfiniteJump;

  /// No description provided for @tooltipInfiniteJump.
  ///
  /// In en, this message translates to:
  /// **'Unlimited mid-air jumps.'**
  String get tooltipInfiniteJump;

  /// No description provided for @labelNoPodCooldown.
  ///
  /// In en, this message translates to:
  /// **'No Pod Cooldown'**
  String get labelNoPodCooldown;

  /// No description provided for @tooltipNoPodCooldown.
  ///
  /// In en, this message translates to:
  /// **'Pod programs have no cooldown.'**
  String get tooltipNoPodCooldown;

  /// No description provided for @labelInfiniteAirDash.
  ///
  /// In en, this message translates to:
  /// **'Infinite Air Dash'**
  String get labelInfiniteAirDash;

  /// No description provided for @tooltipInfiniteAirDash.
  ///
  /// In en, this message translates to:
  /// **'Unlimited mid-air dashes.'**
  String get tooltipInfiniteAirDash;

  /// No description provided for @labelAutoStart.
  ///
  /// In en, this message translates to:
  /// **'Auto Start'**
  String get labelAutoStart;

  /// No description provided for @tooltipAutoStart.
  ///
  /// In en, this message translates to:
  /// **'Auto-start randomizer on game launch.'**
  String get tooltipAutoStart;

  /// No description provided for @labelGroundEnemies.
  ///
  /// In en, this message translates to:
  /// **'Ground Enemies'**
  String get labelGroundEnemies;

  /// No description provided for @tooltipGroundEnemies.
  ///
  /// In en, this message translates to:
  /// **'Randomize ground-based spawns.'**
  String get tooltipGroundEnemies;

  /// No description provided for @labelFlyingEnemies.
  ///
  /// In en, this message translates to:
  /// **'Flying Enemies'**
  String get labelFlyingEnemies;

  /// No description provided for @tooltipFlyingEnemies.
  ///
  /// In en, this message translates to:
  /// **'Randomize flying spawns.'**
  String get tooltipFlyingEnemies;

  /// No description provided for @labelAllowBigEnemies.
  ///
  /// In en, this message translates to:
  /// **'Allow Big Enemies'**
  String get labelAllowBigEnemies;

  /// No description provided for @tooltipAllowBigEnemies.
  ///
  /// In en, this message translates to:
  /// **'Allow large enemies.'**
  String get tooltipAllowBigEnemies;

  /// No description provided for @labelIncludeDlcEnemies.
  ///
  /// In en, this message translates to:
  /// **'Include DLC Enemies'**
  String get labelIncludeDlcEnemies;

  /// No description provided for @tooltipIncludeDlcEnemies.
  ///
  /// In en, this message translates to:
  /// **'Include DLC enemies.'**
  String get tooltipIncludeDlcEnemies;

  /// No description provided for @tutorialCameraAccel.
  ///
  /// In en, this message translates to:
  /// **'Removes mouse acceleration for 1:1 input.'**
  String get tutorialCameraAccel;

  /// No description provided for @tutorialWipBanner.
  ///
  /// In en, this message translates to:
  /// **'These features are coming in future NAMS updates.'**
  String get tutorialWipBanner;

  /// No description provided for @labelFixCameraAcceleration.
  ///
  /// In en, this message translates to:
  /// **'Fix Camera Acceleration'**
  String get labelFixCameraAcceleration;

  /// No description provided for @tooltipFixCameraAcceleration.
  ///
  /// In en, this message translates to:
  /// **'Linear 1:1 mouse movement. Removes deadzone and acceleration curve from camera rotation.'**
  String get tooltipFixCameraAcceleration;

  /// No description provided for @labelSensitivity.
  ///
  /// In en, this message translates to:
  /// **'Sensitivity'**
  String get labelSensitivity;

  /// No description provided for @tooltipSensitivity.
  ///
  /// In en, this message translates to:
  /// **'Camera sensitivity multiplier. Higher = faster rotation. 2.0 is a good default.'**
  String get tooltipSensitivity;

  /// No description provided for @labelFixAimAcceleration.
  ///
  /// In en, this message translates to:
  /// **'Fix Aim Acceleration'**
  String get labelFixAimAcceleration;

  /// No description provided for @tooltipFixAimAcceleration.
  ///
  /// In en, this message translates to:
  /// **'Remove clamp and deadzone from pod/mech aiming in top-down and side-scroll views.'**
  String get tooltipFixAimAcceleration;

  /// No description provided for @labelAimSensitivity.
  ///
  /// In en, this message translates to:
  /// **'Aim Sensitivity'**
  String get labelAimSensitivity;

  /// No description provided for @tooltipAimSensitivity.
  ///
  /// In en, this message translates to:
  /// **'Aim sensitivity for top-down/side-scroll. 0.001 for ~3500 DPI, 0.003 for ~800 DPI.'**
  String get tooltipAimSensitivity;

  /// No description provided for @labelAimOutputMultiplier.
  ///
  /// In en, this message translates to:
  /// **'Aim Output Multiplier'**
  String get labelAimOutputMultiplier;

  /// No description provided for @tooltipAimOutputMultiplier.
  ///
  /// In en, this message translates to:
  /// **'Raw multiplier for crosshair movement speed after normalization. Higher = faster crosshair. Most users won\'t need to change this.'**
  String get tooltipAimOutputMultiplier;

  /// No description provided for @labelDisablePodPet.
  ///
  /// In en, this message translates to:
  /// **'Disable Pod Pet'**
  String get labelDisablePodPet;

  /// No description provided for @tooltipDisablePodPet.
  ///
  /// In en, this message translates to:
  /// **'Disable the pod petting animation triggered by mouse movement. Recommended.'**
  String get tooltipDisablePodPet;

  /// No description provided for @labelDebugMenuKey.
  ///
  /// In en, this message translates to:
  /// **'Debug Menu Key'**
  String get labelDebugMenuKey;

  /// No description provided for @tooltipDebugMenuKey.
  ///
  /// In en, this message translates to:
  /// **'Opens the debug menu accessible after clearing the game. Usually requires a controller — this binding makes it possible with keyboard.'**
  String get tooltipDebugMenuKey;

  /// No description provided for @labelEvadeKey.
  ///
  /// In en, this message translates to:
  /// **'Evade Key'**
  String get labelEvadeKey;

  /// No description provided for @tooltipEvadeKey.
  ///
  /// In en, this message translates to:
  /// **'Dedicated evade/dodge in current movement direction.'**
  String get tooltipEvadeKey;

  /// No description provided for @todoThirdPersonMode.
  ///
  /// In en, this message translates to:
  /// **'Third-Person Mode'**
  String get todoThirdPersonMode;

  /// No description provided for @todoThirdPersonModeDesc.
  ///
  /// In en, this message translates to:
  /// **'Raw Input / Raw Input Char Follow / Disabled'**
  String get todoThirdPersonModeDesc;

  /// No description provided for @todoThirdPersonSensX.
  ///
  /// In en, this message translates to:
  /// **'Third-Person Sensitivity X'**
  String get todoThirdPersonSensX;

  /// No description provided for @todoThirdPersonSensXDesc.
  ///
  /// In en, this message translates to:
  /// **'Horizontal sensitivity (calibrated to Quake preset, yaw 0.022)'**
  String get todoThirdPersonSensXDesc;

  /// No description provided for @todoThirdPersonSensY.
  ///
  /// In en, this message translates to:
  /// **'Third-Person Sensitivity Y'**
  String get todoThirdPersonSensY;

  /// No description provided for @todoThirdPersonSensYDesc.
  ///
  /// In en, this message translates to:
  /// **'Vertical sensitivity'**
  String get todoThirdPersonSensYDesc;

  /// No description provided for @todoThirdPersonFixToggle.
  ///
  /// In en, this message translates to:
  /// **'Third-Person Fix Toggle'**
  String get todoThirdPersonFixToggle;

  /// No description provided for @todoThirdPersonFixToggleDesc.
  ///
  /// In en, this message translates to:
  /// **'Binding to toggle fix on/off'**
  String get todoThirdPersonFixToggleDesc;

  /// No description provided for @todoAimMode.
  ///
  /// In en, this message translates to:
  /// **'Aim Mode'**
  String get todoAimMode;

  /// No description provided for @todoAimModeDesc.
  ///
  /// In en, this message translates to:
  /// **'Disabled / Raw Input Only / Raw Input Crosshair'**
  String get todoAimModeDesc;

  /// No description provided for @todoAimSensXY.
  ///
  /// In en, this message translates to:
  /// **'Aim Sensitivity X / Y'**
  String get todoAimSensXY;

  /// No description provided for @todoAimSensXYDesc.
  ///
  /// In en, this message translates to:
  /// **'Separate horizontal and vertical aim sensitivity'**
  String get todoAimSensXYDesc;

  /// No description provided for @todoAimFixToggle.
  ///
  /// In en, this message translates to:
  /// **'Aim Fix Toggle'**
  String get todoAimFixToggle;

  /// No description provided for @todoAimFixToggleDesc.
  ///
  /// In en, this message translates to:
  /// **'Binding to toggle aim fix on/off'**
  String get todoAimFixToggleDesc;

  /// No description provided for @todoAutoFireToggle.
  ///
  /// In en, this message translates to:
  /// **'Auto Fire Toggle'**
  String get todoAutoFireToggle;

  /// No description provided for @todoAutoFireToggleDesc.
  ///
  /// In en, this message translates to:
  /// **'Toggles auto firing so button doesn\'t have to be held'**
  String get todoAutoFireToggleDesc;

  /// No description provided for @todoCustomCursors.
  ///
  /// In en, this message translates to:
  /// **'Custom Cursors'**
  String get todoCustomCursors;

  /// No description provided for @todoCustomCursorsDesc.
  ///
  /// In en, this message translates to:
  /// **'Custom mouse cursors in menus and hacking minigame'**
  String get todoCustomCursorsDesc;

  /// No description provided for @todoStatusSounds.
  ///
  /// In en, this message translates to:
  /// **'Status Sounds'**
  String get todoStatusSounds;

  /// No description provided for @todoStatusSoundsDesc.
  ///
  /// In en, this message translates to:
  /// **'Text-to-speech status sounds on config reload/profile switch'**
  String get todoStatusSoundsDesc;

  /// No description provided for @todoReloadConfigBinding.
  ///
  /// In en, this message translates to:
  /// **'Reload Config Binding'**
  String get todoReloadConfigBinding;

  /// No description provided for @todoReloadConfigBindingDesc.
  ///
  /// In en, this message translates to:
  /// **'Hotkey to reload config without restarting'**
  String get todoReloadConfigBindingDesc;

  /// No description provided for @todoMoveForward.
  ///
  /// In en, this message translates to:
  /// **'Move Forward / Left / Back / Right'**
  String get todoMoveForward;

  /// No description provided for @todoMoveForwardDesc.
  ///
  /// In en, this message translates to:
  /// **'Custom movement bindings'**
  String get todoMoveForwardDesc;

  /// No description provided for @todoJump.
  ///
  /// In en, this message translates to:
  /// **'Jump'**
  String get todoJump;

  /// No description provided for @todoJumpDesc.
  ///
  /// In en, this message translates to:
  /// **'Custom jump binding'**
  String get todoJumpDesc;

  /// No description provided for @todoWalk.
  ///
  /// In en, this message translates to:
  /// **'Walk'**
  String get todoWalk;

  /// No description provided for @todoWalkDesc.
  ///
  /// In en, this message translates to:
  /// **'Toggle walk mode'**
  String get todoWalkDesc;

  /// No description provided for @todoAutoRun.
  ///
  /// In en, this message translates to:
  /// **'Auto-Run'**
  String get todoAutoRun;

  /// No description provided for @todoAutoRunDesc.
  ///
  /// In en, this message translates to:
  /// **'Toggle auto-run'**
  String get todoAutoRunDesc;

  /// No description provided for @todoDisableTapEvade.
  ///
  /// In en, this message translates to:
  /// **'Disable Tap Evade'**
  String get todoDisableTapEvade;

  /// No description provided for @todoDisableTapEvadeDesc.
  ///
  /// In en, this message translates to:
  /// **'Prevent double-tap movement keys from evading'**
  String get todoDisableTapEvadeDesc;

  /// No description provided for @todoLightAttack.
  ///
  /// In en, this message translates to:
  /// **'Light Attack'**
  String get todoLightAttack;

  /// No description provided for @todoLightAttackDesc.
  ///
  /// In en, this message translates to:
  /// **'Custom light attack binding'**
  String get todoLightAttackDesc;

  /// No description provided for @todoHeavyAttack.
  ///
  /// In en, this message translates to:
  /// **'Heavy Attack'**
  String get todoHeavyAttack;

  /// No description provided for @todoHeavyAttackDesc.
  ///
  /// In en, this message translates to:
  /// **'Custom heavy attack binding'**
  String get todoHeavyAttackDesc;

  /// No description provided for @todoFirePodDash.
  ///
  /// In en, this message translates to:
  /// **'Fire / Pod Dash'**
  String get todoFirePodDash;

  /// No description provided for @todoFirePodDashDesc.
  ///
  /// In en, this message translates to:
  /// **'Fire pod or pod dash with jump'**
  String get todoFirePodDashDesc;

  /// No description provided for @todoUseProgram.
  ///
  /// In en, this message translates to:
  /// **'Use Program'**
  String get todoUseProgram;

  /// No description provided for @todoUseProgramDesc.
  ///
  /// In en, this message translates to:
  /// **'Use pod/flying unit program'**
  String get todoUseProgramDesc;

  /// No description provided for @todoLockOn.
  ///
  /// In en, this message translates to:
  /// **'Lock-On'**
  String get todoLockOn;

  /// No description provided for @todoLockOnDesc.
  ///
  /// In en, this message translates to:
  /// **'Lock-on to target'**
  String get todoLockOnDesc;

  /// No description provided for @todoSwitchWeapon.
  ///
  /// In en, this message translates to:
  /// **'Switch Weapon'**
  String get todoSwitchWeapon;

  /// No description provided for @todoSwitchWeaponDesc.
  ///
  /// In en, this message translates to:
  /// **'Cycle equipped weapons'**
  String get todoSwitchWeaponDesc;

  /// No description provided for @todoNextPrevProgram.
  ///
  /// In en, this message translates to:
  /// **'Next / Previous Program'**
  String get todoNextPrevProgram;

  /// No description provided for @todoNextPrevProgramDesc.
  ///
  /// In en, this message translates to:
  /// **'Cycle pod programs'**
  String get todoNextPrevProgramDesc;

  /// No description provided for @todoSelfDestruct.
  ///
  /// In en, this message translates to:
  /// **'Self Destruct'**
  String get todoSelfDestruct;

  /// No description provided for @todoSelfDestructDesc.
  ///
  /// In en, this message translates to:
  /// **'Trigger self destruct'**
  String get todoSelfDestructDesc;

  /// No description provided for @todoNextPrevItem.
  ///
  /// In en, this message translates to:
  /// **'Next / Previous Item'**
  String get todoNextPrevItem;

  /// No description provided for @todoNextPrevItemDesc.
  ///
  /// In en, this message translates to:
  /// **'Quick-switch items without opening menu twice'**
  String get todoNextPrevItemDesc;

  /// No description provided for @todoUseItem.
  ///
  /// In en, this message translates to:
  /// **'Use Item'**
  String get todoUseItem;

  /// No description provided for @todoUseItemDesc.
  ///
  /// In en, this message translates to:
  /// **'Immediately use selected quick menu item'**
  String get todoUseItemDesc;

  /// No description provided for @todoMenuUpDown.
  ///
  /// In en, this message translates to:
  /// **'Menu Up / Down / Left / Right'**
  String get todoMenuUpDown;

  /// No description provided for @todoMenuUpDownDesc.
  ///
  /// In en, this message translates to:
  /// **'Navigate menus with custom keys'**
  String get todoMenuUpDownDesc;

  /// No description provided for @todoMenuOpen.
  ///
  /// In en, this message translates to:
  /// **'Menu Open'**
  String get todoMenuOpen;

  /// No description provided for @todoMenuOpenDesc.
  ///
  /// In en, this message translates to:
  /// **'Open system menu'**
  String get todoMenuOpenDesc;

  /// No description provided for @todoMenuBackClose.
  ///
  /// In en, this message translates to:
  /// **'Menu Back / Close'**
  String get todoMenuBackClose;

  /// No description provided for @todoMenuBackCloseDesc.
  ///
  /// In en, this message translates to:
  /// **'Navigate back or close menu'**
  String get todoMenuBackCloseDesc;

  /// No description provided for @todoMenuEnterSkip.
  ///
  /// In en, this message translates to:
  /// **'Menu Enter / Skip Dialog'**
  String get todoMenuEnterSkip;

  /// No description provided for @todoMenuEnterSkipDesc.
  ///
  /// In en, this message translates to:
  /// **'Enter sub-menu or skip dialog'**
  String get todoMenuEnterSkipDesc;

  /// No description provided for @todoShortcutMenu.
  ///
  /// In en, this message translates to:
  /// **'Shortcut Menu'**
  String get todoShortcutMenu;

  /// No description provided for @todoShortcutMenuDesc.
  ///
  /// In en, this message translates to:
  /// **'Open shortcut menu'**
  String get todoShortcutMenuDesc;

  /// No description provided for @labelPreloadMaxDimensionShort.
  ///
  /// In en, this message translates to:
  /// **'Preload Max Dimension'**
  String get labelPreloadMaxDimensionShort;

  /// No description provided for @tooltipPreloadMaxDimensionShort.
  ///
  /// In en, this message translates to:
  /// **'0 = disabled (pure streaming), 2048 = default, 4096 = 4K textures, 16384 = everything.'**
  String get tooltipPreloadMaxDimensionShort;

  /// No description provided for @labelPreloadAllTexturesShort.
  ///
  /// In en, this message translates to:
  /// **'Preload All Textures'**
  String get labelPreloadAllTexturesShort;

  /// No description provided for @tooltipPreloadAllTexturesShort.
  ///
  /// In en, this message translates to:
  /// **'Preload ALL textures. No stutter but needs 32GB+ RAM.'**
  String get tooltipPreloadAllTexturesShort;

  /// No description provided for @labelVramBudget.
  ///
  /// In en, this message translates to:
  /// **'VRAM Budget (MB)'**
  String get labelVramBudget;

  /// No description provided for @tooltipVramBudget.
  ///
  /// In en, this message translates to:
  /// **'How much GPU memory the texture mod system may use. Pick a value to set a hard cap — e.g. 8192 means \"never use more than 8 GB for modded textures\", 16384 means \"never use more than 16 GB\". Auto (recommended) skips the cap and uses what your GPU actually has available.'**
  String get tooltipVramBudget;

  /// No description provided for @labelStreamingEnabled.
  ///
  /// In en, this message translates to:
  /// **'Background Loading'**
  String get labelStreamingEnabled;

  /// No description provided for @tooltipStreamingEnabled.
  ///
  /// In en, this message translates to:
  /// **'Loads textures in the background while you play. Prevents freezes and stuttering when new areas load in. Turn off only if you have issues — without it, the game may briefly freeze when loading new textures.'**
  String get tooltipStreamingEnabled;

  /// No description provided for @labelLoadOnlyRelevant.
  ///
  /// In en, this message translates to:
  /// **'Load Only Relevant'**
  String get labelLoadOnlyRelevant;

  /// No description provided for @tooltipLoadOnlyRelevant.
  ///
  /// In en, this message translates to:
  /// **'For huge packs (400+ files), only load textures matching a curated priority list — saves VRAM and loading time. Small packs (clothing, weapons) are always loaded in full. Turn on if you use a massive pack and want to save memory.'**
  String get tooltipLoadOnlyRelevant;

  /// No description provided for @tutorialDropTextures.
  ///
  /// In en, this message translates to:
  /// **'Drag texture mods here to install them. Zip files are extracted automatically.'**
  String get tutorialDropTextures;

  /// No description provided for @tutorialLoadOrder.
  ///
  /// In en, this message translates to:
  /// **'If mods overlap, drag to reorder. Top = highest priority.'**
  String get tutorialLoadOrder;

  /// No description provided for @textureOverlapLabel.
  ///
  /// In en, this message translates to:
  /// **'OVERLAP'**
  String get textureOverlapLabel;

  /// No description provided for @tooltipTextureOverlap.
  ///
  /// In en, this message translates to:
  /// **'Changes same textures as: {mods}. The one higher in the list (closer to HIGHEST) is what you see in-game.'**
  String tooltipTextureOverlap(String mods);

  /// No description provided for @tooltipFolderNotFound.
  ///
  /// In en, this message translates to:
  /// **'Folder not found in nams/inject/textures/'**
  String get tooltipFolderNotFound;

  /// No description provided for @priorityHighest.
  ///
  /// In en, this message translates to:
  /// **'HIGHEST'**
  String get priorityHighest;

  /// No description provided for @priorityMedium.
  ///
  /// In en, this message translates to:
  /// **'MEDIUM'**
  String get priorityMedium;

  /// No description provided for @priorityLowest.
  ///
  /// In en, this message translates to:
  /// **'LOWEST'**
  String get priorityLowest;

  /// No description provided for @nameOutfitTitle.
  ///
  /// In en, this message translates to:
  /// **'Name this outfit ({character})'**
  String nameOutfitTitle(String character);

  /// No description provided for @outfitNameHint.
  ///
  /// In en, this message translates to:
  /// **'Outfit name'**
  String get outfitNameHint;

  /// No description provided for @installedTextureCount.
  ///
  /// In en, this message translates to:
  /// **'Installed {count} texture {count, plural, =1{file} other{files}}'**
  String installedTextureCount(int count);

  /// No description provided for @installationFailed.
  ///
  /// In en, this message translates to:
  /// **'Installation failed: {error}'**
  String installationFailed(String error);

  /// No description provided for @removedItem.
  ///
  /// In en, this message translates to:
  /// **'Removed \"{name}\"'**
  String removedItem(String name);

  /// No description provided for @tutorialStarIcon.
  ///
  /// In en, this message translates to:
  /// **'Click the star to set a default outfit that loads on game start.'**
  String get tutorialStarIcon;

  /// No description provided for @installedOutfitsCount.
  ///
  /// In en, this message translates to:
  /// **'INSTALLED OUTFITS ({count})'**
  String installedOutfitsCount(int count);

  /// No description provided for @tooltipDlcDetected.
  ///
  /// In en, this message translates to:
  /// **'DLC detected (data100.cpk). Model files use DLC naming (pl000d).'**
  String get tooltipDlcDetected;

  /// No description provided for @tooltipNoDlcDetected.
  ///
  /// In en, this message translates to:
  /// **'No DLC detected. Model files will be renamed to non-DLC naming (pl0000).'**
  String get tooltipNoDlcDetected;

  /// No description provided for @installConfirmMod.
  ///
  /// In en, this message translates to:
  /// **'Install \"{name}\" ({character})?'**
  String installConfirmMod(String name, String character);

  /// No description provided for @installedOutfit.
  ///
  /// In en, this message translates to:
  /// **'Installed \"{name}\" ({character})'**
  String installedOutfit(String name, String character);

  /// No description provided for @crossInstallTextures.
  ///
  /// In en, this message translates to:
  /// **'This mod also contains texture files. Install them to nams/inject/textures/?'**
  String get crossInstallTextures;

  /// No description provided for @alsoInstalledTextures.
  ///
  /// In en, this message translates to:
  /// **'Also installed {count} texture {count, plural, =1{file} other{files}}'**
  String alsoInstalledTextures(int count);

  /// No description provided for @clearedAllStartupOutfits.
  ///
  /// In en, this message translates to:
  /// **'Cleared all startup outfits'**
  String get clearedAllStartupOutfits;

  /// No description provided for @clearedStartupOutfit.
  ///
  /// In en, this message translates to:
  /// **'Cleared startup outfit'**
  String get clearedStartupOutfit;

  /// No description provided for @setStartupOutfit.
  ///
  /// In en, this message translates to:
  /// **'Set \"{name}\" as startup outfit'**
  String setStartupOutfit(String name);

  /// No description provided for @tutorialDropCutscenes.
  ///
  /// In en, this message translates to:
  /// **'Drop cutscene mod archives here. Supports .zip, .7z, and .rar.'**
  String get tutorialDropCutscenes;

  /// No description provided for @tutorialInstalledCutscenes.
  ///
  /// In en, this message translates to:
  /// **'Your installed cutscene mods. Custom cutscenes load from here instead of data/movie/.'**
  String get tutorialInstalledCutscenes;

  /// No description provided for @selectCutsceneModFolder.
  ///
  /// In en, this message translates to:
  /// **'Select Cutscene Mod Folder'**
  String get selectCutsceneModFolder;

  /// No description provided for @cutsceneNamingHint.
  ///
  /// In en, this message translates to:
  /// **'Max {max} characters. This becomes the folder name in nams/cutscenes/.'**
  String cutsceneNamingHint(int max);

  /// No description provided for @cutsceneNameTooLong.
  ///
  /// In en, this message translates to:
  /// **'Name must be {max} characters or fewer.'**
  String cutsceneNameTooLong(int max);

  /// No description provided for @preparingInstall.
  ///
  /// In en, this message translates to:
  /// **'Preparing...'**
  String get preparingInstall;

  /// No description provided for @installedCutsceneMod.
  ///
  /// In en, this message translates to:
  /// **'Installed \"{name}\" ({count} USM {count, plural, =1{file} other{files}})'**
  String installedCutsceneMod(String name, int count);

  /// No description provided for @deleteCutsceneConfirm.
  ///
  /// In en, this message translates to:
  /// **'Delete \"{name}\" and all its files?'**
  String deleteCutsceneConfirm(String name);

  /// No description provided for @installedCutsceneModsCount.
  ///
  /// In en, this message translates to:
  /// **'INSTALLED CUTSCENE MODS ({count})'**
  String installedCutsceneModsCount(int count);

  /// No description provided for @cutsceneUsmCount.
  ///
  /// In en, this message translates to:
  /// **'{count} USM {count, plural, =1{file} other{files}}'**
  String cutsceneUsmCount(int count);

  /// No description provided for @cutsceneMatchCount.
  ///
  /// In en, this message translates to:
  /// **'{matching}/{total} match originals'**
  String cutsceneMatchCount(int matching, int total);

  /// No description provided for @tooltipMissingOriginals.
  ///
  /// In en, this message translates to:
  /// **'Files not matching originals: {files}'**
  String tooltipMissingOriginals(String files);

  /// No description provided for @cutsceneMismatchHint.
  ///
  /// In en, this message translates to:
  /// **'Some files don\'t match original cutscene names. Missing files will fall back to the original cutscenes.'**
  String get cutsceneMismatchHint;

  /// No description provided for @cutsceneMigrationBannerBody.
  ///
  /// In en, this message translates to:
  /// **'Found {count} custom cutscene {count, plural, =1{file} other{files}} directly in data/movie/. These overwrite the originals permanently. Next time, install cutscene mods here instead — if a custom file fails to load, the original plays as fallback.'**
  String cutsceneMigrationBannerBody(int count);

  /// No description provided for @hardwareInfoLabel.
  ///
  /// In en, this message translates to:
  /// **'{ram}GB RAM | {gpu}'**
  String hardwareInfoLabel(int ram, String gpu);

  /// No description provided for @hardwareInfoRamOnly.
  ///
  /// In en, this message translates to:
  /// **'{ram}GB RAM'**
  String hardwareInfoRamOnly(int ram);

  /// No description provided for @texturesScanResult.
  ///
  /// In en, this message translates to:
  /// **'{count} texture files, {sizeMB}MB total, max {maxDim}px'**
  String texturesScanResult(int count, int sizeMB, int maxDim);

  /// No description provided for @recommendedSettings.
  ///
  /// In en, this message translates to:
  /// **'Recommended: Preload {dim}, Preload All {allLabel}'**
  String recommendedSettings(int dim, String allLabel);

  /// No description provided for @applyRecommended.
  ///
  /// In en, this message translates to:
  /// **'APPLY'**
  String get applyRecommended;

  /// No description provided for @settingsMatchRecommended.
  ///
  /// In en, this message translates to:
  /// **'Your settings match the recommendation'**
  String get settingsMatchRecommended;

  /// No description provided for @reasonNoTextures.
  ///
  /// In en, this message translates to:
  /// **'No textures installed'**
  String get reasonNoTextures;

  /// No description provided for @reasonFitsInMemory.
  ///
  /// In en, this message translates to:
  /// **'{ramGB}GB RAM, {textureSizeMB}MB textures — fits in memory, preload everything for zero stutter'**
  String reasonFitsInMemory(int ramGB, int textureSizeMB);

  /// No description provided for @reasonExceedsRam.
  ///
  /// In en, this message translates to:
  /// **'{ramGB}GB RAM, ~{estimatedGB}GB estimated texture memory — preloading all will freeze or crash your system. Use a low preload dimension or remove some texture packs.'**
  String reasonExceedsRam(int ramGB, int estimatedGB);

  /// No description provided for @reasonTooLargeForAll.
  ///
  /// In en, this message translates to:
  /// **'{ramGB}GB RAM, {textureSizeMB}MB textures — too large to preload all, preload up to 4K on demand'**
  String reasonTooLargeForAll(int ramGB, int textureSizeMB);

  /// No description provided for @reasonMediumRam.
  ///
  /// In en, this message translates to:
  /// **'{ramGB}GB RAM — preload up to 4K, larger textures load on demand'**
  String reasonMediumRam(int ramGB);

  /// No description provided for @reasonLowRam.
  ///
  /// In en, this message translates to:
  /// **'{ramGB}GB RAM — preload small textures only to save memory'**
  String reasonLowRam(int ramGB);

  /// No description provided for @analyzingHardware.
  ///
  /// In en, this message translates to:
  /// **'Analyzing hardware and textures...'**
  String get analyzingHardware;

  /// No description provided for @texturesBloatWarning.
  ///
  /// In en, this message translates to:
  /// **'This mod has {total} textures but only {relevant} are visually relevant (based on GPUnity\'s curated reference set). The remaining {excess} textures add load time and RAM usage with no visible benefit.'**
  String texturesBloatWarning(int total, int relevant, int excess);

  /// No description provided for @cleanUnneededTextures.
  ///
  /// In en, this message translates to:
  /// **'REMOVE {count} UNNEEDED'**
  String cleanUnneededTextures(int count);

  /// No description provided for @cleanedTextures.
  ///
  /// In en, this message translates to:
  /// **'Removed {deleted} unneeded textures, kept {kept}'**
  String cleanedTextures(int deleted, int kept);

  /// No description provided for @confirmCleanTextures.
  ///
  /// In en, this message translates to:
  /// **'Remove unneeded textures?'**
  String get confirmCleanTextures;

  /// No description provided for @confirmCleanTexturesBody.
  ///
  /// In en, this message translates to:
  /// **'This will permanently delete {count} texture files ({sizeMB} MB) from this mod folder.'**
  String confirmCleanTexturesBody(int count, String sizeMB);

  /// No description provided for @confirmCleanTexturesDetail1.
  ///
  /// In en, this message translates to:
  /// **'Only textures matching the GPUnity curated reference set will be kept'**
  String get confirmCleanTexturesDetail1;

  /// No description provided for @confirmCleanTexturesDetail2.
  ///
  /// In en, this message translates to:
  /// **'This only affects the selected mod folder, not other installed mods'**
  String get confirmCleanTexturesDetail2;

  /// No description provided for @confirmCleanTexturesDetail3.
  ///
  /// In en, this message translates to:
  /// **'This cannot be undone — drop the mod again to restore removed files'**
  String get confirmCleanTexturesDetail3;

  /// No description provided for @texturesBloatDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Unnecessary textures detected'**
  String get texturesBloatDialogTitle;

  /// No description provided for @texturesBloatDialogBody.
  ///
  /// In en, this message translates to:
  /// **'This texture pack has {total} files but only {relevant} match the GPUnity curated reference set. The remaining {excess} textures are likely unnecessary.'**
  String texturesBloatDialogBody(int total, int relevant, int excess);

  /// No description provided for @texturesBloatPoint1.
  ///
  /// In en, this message translates to:
  /// **'Much longer game startup — the engine loads every texture at launch'**
  String get texturesBloatPoint1;

  /// No description provided for @texturesBloatPoint2.
  ///
  /// In en, this message translates to:
  /// **'Random stutter and frame drops — the game streams textures that add no visual benefit'**
  String get texturesBloatPoint2;

  /// No description provided for @texturesBloatPoint3.
  ///
  /// In en, this message translates to:
  /// **'High RAM usage — up to several GB wasted on textures you cannot see'**
  String get texturesBloatPoint3;

  /// No description provided for @texturesBloatPoint4.
  ///
  /// In en, this message translates to:
  /// **'Some AI-upscaled textures may contain artifacts or corruption'**
  String get texturesBloatPoint4;

  /// No description provided for @texturesBloatPoint5.
  ///
  /// In en, this message translates to:
  /// **'Almost no visual difference — most are tiny UI elements, particle effects, etc.'**
  String get texturesBloatPoint5;

  /// No description provided for @texturesBloatRecommendation.
  ///
  /// In en, this message translates to:
  /// **'Removing them is safe and recommended for better performance.'**
  String get texturesBloatRecommendation;

  /// No description provided for @texturesBloatKeepAll.
  ///
  /// In en, this message translates to:
  /// **'Keep all'**
  String get texturesBloatKeepAll;

  /// No description provided for @texturesBloatRemoveUnneeded.
  ///
  /// In en, this message translates to:
  /// **'Remove unneeded ({count})'**
  String texturesBloatRemoveUnneeded(int count);

  /// No description provided for @texturesProgressExtracting.
  ///
  /// In en, this message translates to:
  /// **'Extracting archive...'**
  String get texturesProgressExtracting;

  /// No description provided for @texturesProgressCopying.
  ///
  /// In en, this message translates to:
  /// **'Copying files...'**
  String get texturesProgressCopying;

  /// No description provided for @texturesProgressAnalyzing.
  ///
  /// In en, this message translates to:
  /// **'Analyzing textures...'**
  String get texturesProgressAnalyzing;

  /// No description provided for @texturesAnalyzingSetup.
  ///
  /// In en, this message translates to:
  /// **'Analyzing your texture setup...'**
  String get texturesAnalyzingSetup;

  /// No description provided for @texturesBusyMessage.
  ///
  /// In en, this message translates to:
  /// **'Please wait — texture installation in progress'**
  String get texturesBusyMessage;

  /// No description provided for @texturesInstallProgress.
  ///
  /// In en, this message translates to:
  /// **'Installing: {files}/{totalFiles} files — {mb}/{totalMb} MB'**
  String texturesInstallProgress(
    int files,
    int totalFiles,
    int mb,
    int totalMb,
  );

  /// No description provided for @texturesAnalyzeProgress.
  ///
  /// In en, this message translates to:
  /// **'Analyzing: {scanned}/{total} textures'**
  String texturesAnalyzeProgress(int scanned, int total);

  /// No description provided for @cleaningTextures.
  ///
  /// In en, this message translates to:
  /// **'Removing unneeded textures...'**
  String get cleaningTextures;

  /// No description provided for @textureMergeTitle.
  ///
  /// In en, this message translates to:
  /// **'Add to existing or install new?'**
  String get textureMergeTitle;

  /// No description provided for @textureMergeDescription.
  ///
  /// In en, this message translates to:
  /// **'You already have texture mods installed. Do you want to add these files to an existing mod or install as a new one?'**
  String get textureMergeDescription;

  /// No description provided for @textureMergeNewMod.
  ///
  /// In en, this message translates to:
  /// **'Install as new mod'**
  String get textureMergeNewMod;

  /// No description provided for @textureMergeAddTo.
  ///
  /// In en, this message translates to:
  /// **'Add to: {name}'**
  String textureMergeAddTo(String name);

  /// No description provided for @cutsceneMergeTitle.
  ///
  /// In en, this message translates to:
  /// **'Add to existing or install new?'**
  String get cutsceneMergeTitle;

  /// No description provided for @cutsceneMergeDescription.
  ///
  /// In en, this message translates to:
  /// **'You already have cutscene mods installed. Multi-part cutscene packs should be merged into the same mod.'**
  String get cutsceneMergeDescription;

  /// No description provided for @cutsceneMergeNewMod.
  ///
  /// In en, this message translates to:
  /// **'Install as new mod'**
  String get cutsceneMergeNewMod;

  /// No description provided for @cutsceneMergeAddTo.
  ///
  /// In en, this message translates to:
  /// **'Add to: {name}'**
  String cutsceneMergeAddTo(String name);

  /// No description provided for @headerMods.
  ///
  /// In en, this message translates to:
  /// **'MODS'**
  String get headerMods;

  /// No description provided for @cutsceneBundledWith.
  ///
  /// In en, this message translates to:
  /// **'Bundled with {modId}'**
  String cutsceneBundledWith(String modId);

  /// No description provided for @cutsceneStatusHd.
  ///
  /// In en, this message translates to:
  /// **'HD'**
  String get cutsceneStatusHd;

  /// No description provided for @cutsceneStatusHdTooltip.
  ///
  /// In en, this message translates to:
  /// **'[cutscene] hd_cutscenes in nams.toml — must be true for HD cutscene mods to load.'**
  String get cutsceneStatusHdTooltip;

  /// No description provided for @cutsceneStatusH264.
  ///
  /// In en, this message translates to:
  /// **'H264'**
  String get cutsceneStatusH264;

  /// No description provided for @cutsceneStatusH264Tooltip.
  ///
  /// In en, this message translates to:
  /// **'[cutscene] enable_h264 in nams.toml — must be true to play H264-encoded cutscenes.'**
  String get cutsceneStatusH264Tooltip;

  /// No description provided for @tabPlugins.
  ///
  /// In en, this message translates to:
  /// **'Plugins'**
  String get tabPlugins;

  /// No description provided for @headerPlugins.
  ///
  /// In en, this message translates to:
  /// **'PLUGINS'**
  String get headerPlugins;

  /// No description provided for @pluginIntroTitle.
  ///
  /// In en, this message translates to:
  /// **'NAMS plugins — native Rust extensions on top of the modding SDK'**
  String get pluginIntroTitle;

  /// No description provided for @pluginIntroBody.
  ///
  /// In en, this message translates to:
  /// **'NAMS is the modding SDK; plugins are Rust-built .dll modules that hook into it for custom native logic. YoRHa Protocol itself is one such plugin. Drop a plugin DLL here and the launcher copies it into the plugins/ folder next to NAMS; NAMS auto-loads every plugin there on the next launch. Toggling a plugin off moves it aside so it isn\'t loaded, without deleting it.'**
  String get pluginIntroBody;

  /// No description provided for @pluginDropHere.
  ///
  /// In en, this message translates to:
  /// **'Drop plugin DLL here'**
  String get pluginDropHere;

  /// No description provided for @pluginDropHereHint.
  ///
  /// In en, this message translates to:
  /// **'or click to browse'**
  String get pluginDropHereHint;

  /// No description provided for @pluginListEmpty.
  ///
  /// In en, this message translates to:
  /// **'No plugins installed'**
  String get pluginListEmpty;

  /// No description provided for @pluginInstalled.
  ///
  /// In en, this message translates to:
  /// **'Installed plugin: {name}'**
  String pluginInstalled(String name);

  /// No description provided for @pluginInstallFailed.
  ///
  /// In en, this message translates to:
  /// **'Plugin install failed'**
  String get pluginInstallFailed;

  /// No description provided for @pluginDelete.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get pluginDelete;

  /// No description provided for @pluginDeleteConfirmTitle.
  ///
  /// In en, this message translates to:
  /// **'Delete plugin?'**
  String get pluginDeleteConfirmTitle;

  /// No description provided for @pluginDeleteConfirmBody.
  ///
  /// In en, this message translates to:
  /// **'Permanently remove {name} from the plugins folder? This cannot be undone.'**
  String pluginDeleteConfirmBody(String name);

  /// No description provided for @pluginReasonMissingExport.
  ///
  /// In en, this message translates to:
  /// **'not a NAMS plugin (missing required get_plugin_register export)'**
  String get pluginReasonMissingExport;

  /// No description provided for @pluginReasonNotADll.
  ///
  /// In en, this message translates to:
  /// **'not a valid Windows DLL'**
  String get pluginReasonNotADll;

  /// No description provided for @pluginReasonNot64bit.
  ///
  /// In en, this message translates to:
  /// **'DLL is not 64-bit (NAMS plugins must be x64)'**
  String get pluginReasonNot64bit;

  /// No description provided for @pluginReasonCorrupt.
  ///
  /// In en, this message translates to:
  /// **'DLL is corrupt or has an invalid PE structure'**
  String get pluginReasonCorrupt;

  /// No description provided for @pluginReasonNoExports.
  ///
  /// In en, this message translates to:
  /// **'DLL has no export table'**
  String get pluginReasonNoExports;

  /// No description provided for @pluginReasonReservedName.
  ///
  /// In en, this message translates to:
  /// **'reserved filename — built into the launcher'**
  String get pluginReasonReservedName;

  /// No description provided for @pluginReasonFileNotFound.
  ///
  /// In en, this message translates to:
  /// **'file not found'**
  String get pluginReasonFileNotFound;

  /// No description provided for @pluginReasonReadFailed.
  ///
  /// In en, this message translates to:
  /// **'could not read file'**
  String get pluginReasonReadFailed;

  /// No description provided for @pluginReasonCopyFailed.
  ///
  /// In en, this message translates to:
  /// **'could not copy file into the plugins folder'**
  String get pluginReasonCopyFailed;

  /// No description provided for @pluginReasonIncompatible.
  ///
  /// In en, this message translates to:
  /// **'looks like {tool}, which is not compatible with the launcher'**
  String pluginReasonIncompatible(String tool);

  /// No description provided for @modIntroTitle.
  ///
  /// In en, this message translates to:
  /// **'Powered by NAMS — your data/ folder stays untouched'**
  String get modIntroTitle;

  /// No description provided for @modIntroBody.
  ///
  /// In en, this message translates to:
  /// **'NAMS loads mods from nams/mods/ through a virtual file system on top of the original game data, so nothing is ever copied or overwritten in data/. Mods can be turned on or off at any time without reinstalling, multiple outfits can coexist for the same character, and uninstalling a mod just removes its folder — the vanilla game is always intact underneath.'**
  String get modIntroBody;

  /// No description provided for @modListEmpty.
  ///
  /// In en, this message translates to:
  /// **'No mods installed'**
  String get modListEmpty;

  /// No description provided for @modListEmptyHint.
  ///
  /// In en, this message translates to:
  /// **'Drop a mod folder or archive into the box above to install.'**
  String get modListEmptyHint;

  /// No description provided for @modSearchPlaceholder.
  ///
  /// In en, this message translates to:
  /// **'Search mods…'**
  String get modSearchPlaceholder;

  /// No description provided for @modFilterAll.
  ///
  /// In en, this message translates to:
  /// **'All'**
  String get modFilterAll;

  /// No description provided for @dropModHere.
  ///
  /// In en, this message translates to:
  /// **'Drop mod here'**
  String get dropModHere;

  /// No description provided for @dropModHereHint.
  ///
  /// In en, this message translates to:
  /// **'or click to browse'**
  String get dropModHereHint;

  /// No description provided for @modKindNative.
  ///
  /// In en, this message translates to:
  /// **'NATIVE'**
  String get modKindNative;

  /// No description provided for @modKindNativeTooltip.
  ///
  /// In en, this message translates to:
  /// **'NAMS mod with an entities/ folder. Defines new items, weapons, outfits, accessories, quests etc. via TOML bundles.'**
  String get modKindNativeTooltip;

  /// No description provided for @modKindData.
  ///
  /// In en, this message translates to:
  /// **'DATA'**
  String get modKindData;

  /// No description provided for @modKindDataTooltip.
  ///
  /// In en, this message translates to:
  /// **'The classic mod format - same files that would normally go into NieRAutomata/data/, just managed under nams/mods/ instead keeping original data dir clean'**
  String get modKindDataTooltip;

  /// No description provided for @textureOutfitLinkedTitle.
  ///
  /// In en, this message translates to:
  /// **'Outfit-linked textures'**
  String get textureOutfitLinkedTitle;

  /// No description provided for @textureOutfitLinkedSubtitle.
  ///
  /// In en, this message translates to:
  /// **'These textures live inside their mod folder and load only while that outfit is equipped. NAMS hot-swaps them when you change outfits in-game.'**
  String get textureOutfitLinkedSubtitle;

  /// No description provided for @textureOutfitLinkedEntry.
  ///
  /// In en, this message translates to:
  /// **'{count} textures — active only with this outfit'**
  String textureOutfitLinkedEntry(int count);

  /// No description provided for @modKindTexture.
  ///
  /// In en, this message translates to:
  /// **'TEXTURES'**
  String get modKindTexture;

  /// No description provided for @modKindTextureTooltip.
  ///
  /// In en, this message translates to:
  /// **'A texture pack. Its .dds files were installed to nams/inject/textures/ and are managed from the Textures tab.'**
  String get modKindTextureTooltip;

  /// No description provided for @modKindUnknown.
  ///
  /// In en, this message translates to:
  /// **'UNKNOWN'**
  String get modKindUnknown;

  /// No description provided for @modKindUnknownTooltip.
  ///
  /// In en, this message translates to:
  /// **'The launcher couldn\'t recognise this folder as a valid mod.'**
  String get modKindUnknownTooltip;

  /// No description provided for @modCompatChip.
  ///
  /// In en, this message translates to:
  /// **'wax compat'**
  String get modCompatChip;

  /// No description provided for @modCompatChipTooltip.
  ///
  /// In en, this message translates to:
  /// **' NAMS reads these too for compatibility with existing wax mods.'**
  String get modCompatChipTooltip;

  /// No description provided for @modDataChip.
  ///
  /// In en, this message translates to:
  /// **'+data'**
  String get modDataChip;

  /// No description provided for @modDataChipTooltip.
  ///
  /// In en, this message translates to:
  /// **'Ships a data/ overlay alongside its metadata. Models, textures, sounds etc. live here.'**
  String get modDataChipTooltip;

  /// No description provided for @modDetailNoSelection.
  ///
  /// In en, this message translates to:
  /// **'Select a mod to see details'**
  String get modDetailNoSelection;

  /// No description provided for @modAuthor.
  ///
  /// In en, this message translates to:
  /// **'Author'**
  String get modAuthor;

  /// No description provided for @modVersion.
  ///
  /// In en, this message translates to:
  /// **'Version'**
  String get modVersion;

  /// No description provided for @modRootPath.
  ///
  /// In en, this message translates to:
  /// **'Path'**
  String get modRootPath;

  /// No description provided for @modNativeBundles.
  ///
  /// In en, this message translates to:
  /// **'Native bundles'**
  String get modNativeBundles;

  /// No description provided for @modDataContent.
  ///
  /// In en, this message translates to:
  /// **'Data content'**
  String get modDataContent;

  /// No description provided for @modDataPlayerModels.
  ///
  /// In en, this message translates to:
  /// **'Player models'**
  String get modDataPlayerModels;

  /// No description provided for @modRequiresLabel.
  ///
  /// In en, this message translates to:
  /// **'Requires'**
  String get modRequiresLabel;

  /// No description provided for @modRequiresPluginsLabel.
  ///
  /// In en, this message translates to:
  /// **'Requires plugins'**
  String get modRequiresPluginsLabel;

  /// No description provided for @modRequiresMissing.
  ///
  /// In en, this message translates to:
  /// **'missing'**
  String get modRequiresMissing;

  /// No description provided for @modConflictsLabel.
  ///
  /// In en, this message translates to:
  /// **'Conflicts'**
  String get modConflictsLabel;

  /// No description provided for @modConflictOverlapFile.
  ///
  /// In en, this message translates to:
  /// **'{otherId} also ships {file}'**
  String modConflictOverlapFile(String otherId, String file);

  /// No description provided for @modOpenFolder.
  ///
  /// In en, this message translates to:
  /// **'Open folder'**
  String get modOpenFolder;

  /// No description provided for @modEnable.
  ///
  /// In en, this message translates to:
  /// **'Enable'**
  String get modEnable;

  /// No description provided for @modDisable.
  ///
  /// In en, this message translates to:
  /// **'Disable'**
  String get modDisable;

  /// No description provided for @modDisabled.
  ///
  /// In en, this message translates to:
  /// **'Disabled'**
  String get modDisabled;

  /// No description provided for @modDisabledTooltip.
  ///
  /// In en, this message translates to:
  /// **'This mod is disabled. NAMS will not load it on next game start. Enable it to load again — no need to delete and reinstall.'**
  String get modDisabledTooltip;

  /// No description provided for @modEnableTooltip.
  ///
  /// In en, this message translates to:
  /// **'Mod is loaded by NAMS. Click to disable without removing the files.'**
  String get modEnableTooltip;

  /// No description provided for @modDisableNotice.
  ///
  /// In en, this message translates to:
  /// **'Disabled — takes effect on next game start.'**
  String get modDisableNotice;

  /// No description provided for @modEnableNotice.
  ///
  /// In en, this message translates to:
  /// **'Enabled — takes effect on next game start.'**
  String get modEnableNotice;

  /// No description provided for @modGroupAddButton.
  ///
  /// In en, this message translates to:
  /// **'Add group'**
  String get modGroupAddButton;

  /// No description provided for @modGroupNewDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'New group'**
  String get modGroupNewDialogTitle;

  /// No description provided for @modGroupRenameDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Rename group'**
  String get modGroupRenameDialogTitle;

  /// No description provided for @modGroupUngrouped.
  ///
  /// In en, this message translates to:
  /// **'Ungrouped'**
  String get modGroupUngrouped;

  /// No description provided for @modGroupRename.
  ///
  /// In en, this message translates to:
  /// **'Rename group'**
  String get modGroupRename;

  /// No description provided for @modGroupDelete.
  ///
  /// In en, this message translates to:
  /// **'Delete group'**
  String get modGroupDelete;

  /// No description provided for @modGroupDeleteConfirmTitle.
  ///
  /// In en, this message translates to:
  /// **'Delete group?'**
  String get modGroupDeleteConfirmTitle;

  /// No description provided for @modGroupDeleteConfirmBody.
  ///
  /// In en, this message translates to:
  /// **'Mods in \"{name}\" move back to Ungrouped. Nothing is deleted from disk.'**
  String modGroupDeleteConfirmBody(String name);

  /// No description provided for @modGroupCountLabel.
  ///
  /// In en, this message translates to:
  /// **'{count} mods'**
  String modGroupCountLabel(int count);

  /// No description provided for @modGroupDropHint.
  ///
  /// In en, this message translates to:
  /// **'Drop to move here'**
  String get modGroupDropHint;

  /// No description provided for @modRename.
  ///
  /// In en, this message translates to:
  /// **'Rename'**
  String get modRename;

  /// No description provided for @modRenameDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Rename mod'**
  String get modRenameDialogTitle;

  /// No description provided for @modMoveToGroup.
  ///
  /// In en, this message translates to:
  /// **'Move to group'**
  String get modMoveToGroup;

  /// No description provided for @modUninstall.
  ///
  /// In en, this message translates to:
  /// **'Uninstall'**
  String get modUninstall;

  /// No description provided for @modUninstallConfirmTitle.
  ///
  /// In en, this message translates to:
  /// **'Uninstall mod?'**
  String get modUninstallConfirmTitle;

  /// No description provided for @modUninstallConfirmBody.
  ///
  /// In en, this message translates to:
  /// **'This will permanently delete the mod folder \"{id}\".'**
  String modUninstallConfirmBody(String id);

  /// No description provided for @modProfileLabel.
  ///
  /// In en, this message translates to:
  /// **'Profile'**
  String get modProfileLabel;

  /// No description provided for @modProfileNewButton.
  ///
  /// In en, this message translates to:
  /// **'New'**
  String get modProfileNewButton;

  /// No description provided for @modProfileRenameButton.
  ///
  /// In en, this message translates to:
  /// **'Rename'**
  String get modProfileRenameButton;

  /// No description provided for @modProfileDeleteButton.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get modProfileDeleteButton;

  /// No description provided for @modProfileNewDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'New profile'**
  String get modProfileNewDialogTitle;

  /// No description provided for @modProfileNewDialogHint.
  ///
  /// In en, this message translates to:
  /// **'Profile name (letters, numbers, _ -)'**
  String get modProfileNewDialogHint;

  /// No description provided for @modProfileRenameDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Rename profile'**
  String get modProfileRenameDialogTitle;

  /// No description provided for @modProfileDeleteDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Delete profile?'**
  String get modProfileDeleteDialogTitle;

  /// No description provided for @modProfileDeleteDialogBody.
  ///
  /// In en, this message translates to:
  /// **'Permanently remove the folder mods_profile_{name}/ and any bundled texture packs from this profile. This cannot be undone.'**
  String modProfileDeleteDialogBody(String name);

  /// No description provided for @modProfileDeleteConfirm.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get modProfileDeleteConfirm;

  /// No description provided for @modProfileErrorNameEmpty.
  ///
  /// In en, this message translates to:
  /// **'Name required'**
  String get modProfileErrorNameEmpty;

  /// No description provided for @modProfileErrorNameInvalid.
  ///
  /// In en, this message translates to:
  /// **'Use letters, numbers, _ or - only'**
  String get modProfileErrorNameInvalid;

  /// No description provided for @modProfileErrorNameCollision.
  ///
  /// In en, this message translates to:
  /// **'A profile with this name already exists'**
  String get modProfileErrorNameCollision;

  /// No description provided for @modProfileErrorDeleteActive.
  ///
  /// In en, this message translates to:
  /// **'Switch to another profile before deleting this one'**
  String get modProfileErrorDeleteActive;

  /// No description provided for @modProfileErrorDeleteLast.
  ///
  /// In en, this message translates to:
  /// **'Cannot delete the only remaining profile'**
  String get modProfileErrorDeleteLast;

  /// No description provided for @modProfileErrorTargetMissing.
  ///
  /// In en, this message translates to:
  /// **'Profile folder is missing on disk'**
  String get modProfileErrorTargetMissing;

  /// No description provided for @modProfileErrorFsBusy.
  ///
  /// In en, this message translates to:
  /// **'Filesystem is busy. Close the game and retry.'**
  String get modProfileErrorFsBusy;

  /// No description provided for @modProfileLockedRunning.
  ///
  /// In en, this message translates to:
  /// **'Stop the game before changing profiles.'**
  String get modProfileLockedRunning;

  /// No description provided for @modProfileEmptyHint.
  ///
  /// In en, this message translates to:
  /// **'Empty profile — drop a mod to get started'**
  String get modProfileEmptyHint;

  /// No description provided for @modProfileSwitchedToast.
  ///
  /// In en, this message translates to:
  /// **'Switched to profile {name}'**
  String modProfileSwitchedToast(String name);

  /// No description provided for @modProfileCreatedToast.
  ///
  /// In en, this message translates to:
  /// **'Created and switched to profile {name}'**
  String modProfileCreatedToast(String name);

  /// No description provided for @modProfileDeletedToast.
  ///
  /// In en, this message translates to:
  /// **'Deleted profile {name}'**
  String modProfileDeletedToast(String name);

  /// No description provided for @modProfileRenamedToast.
  ///
  /// In en, this message translates to:
  /// **'Renamed profile to {name}'**
  String modProfileRenamedToast(String name);

  /// No description provided for @modInstallNeedsName.
  ///
  /// In en, this message translates to:
  /// **'Name this mod'**
  String get modInstallNeedsName;

  /// No description provided for @modInstallExistsPickAnother.
  ///
  /// In en, this message translates to:
  /// **'A mod named \"{id}\" already exists. Pick a different name.'**
  String modInstallExistsPickAnother(String id);

  /// No description provided for @modInspectBusy.
  ///
  /// In en, this message translates to:
  /// **'Inspecting mod…'**
  String get modInspectBusy;

  /// No description provided for @modInstallBusy.
  ///
  /// In en, this message translates to:
  /// **'Installing mod…'**
  String get modInstallBusy;

  /// No description provided for @modVariantDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Choose what to install'**
  String get modVariantDialogTitle;

  /// No description provided for @modVariantDialogSubtitle.
  ///
  /// In en, this message translates to:
  /// **'This archive contains multiple options. Pick the ones you want.'**
  String get modVariantDialogSubtitle;

  /// No description provided for @modOutfitChoiceDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Choose what to install'**
  String get modOutfitChoiceDialogTitle;

  /// No description provided for @modOutfitChoiceDialogSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Tick everything you want. Each item installs as its own mod. If an outfit ships textures, they come along and you can fine-tune which sets it uses later in the Textures tab.'**
  String get modOutfitChoiceDialogSubtitle;

  /// No description provided for @variantCatPlayer.
  ///
  /// In en, this message translates to:
  /// **'Outfits'**
  String get variantCatPlayer;

  /// No description provided for @variantCatWeapon.
  ///
  /// In en, this message translates to:
  /// **'Weapons'**
  String get variantCatWeapon;

  /// No description provided for @variantCatAccessory.
  ///
  /// In en, this message translates to:
  /// **'Accessories'**
  String get variantCatAccessory;

  /// No description provided for @variantCatEnemy.
  ///
  /// In en, this message translates to:
  /// **'Enemies'**
  String get variantCatEnemy;

  /// No description provided for @variantCatModelVariant.
  ///
  /// In en, this message translates to:
  /// **'Model variants'**
  String get variantCatModelVariant;

  /// No description provided for @variantCatItem.
  ///
  /// In en, this message translates to:
  /// **'Items'**
  String get variantCatItem;

  /// No description provided for @variantCatWorldProp.
  ///
  /// In en, this message translates to:
  /// **'World props'**
  String get variantCatWorldProp;

  /// No description provided for @variantCatMap.
  ///
  /// In en, this message translates to:
  /// **'Maps'**
  String get variantCatMap;

  /// No description provided for @variantCatEffects.
  ///
  /// In en, this message translates to:
  /// **'Effects'**
  String get variantCatEffects;

  /// No description provided for @variantCatScripting.
  ///
  /// In en, this message translates to:
  /// **'Scripting'**
  String get variantCatScripting;

  /// No description provided for @variantCatLocalization.
  ///
  /// In en, this message translates to:
  /// **'Localization'**
  String get variantCatLocalization;

  /// No description provided for @variantCatUi.
  ///
  /// In en, this message translates to:
  /// **'UI'**
  String get variantCatUi;

  /// No description provided for @variantCatCutscenes.
  ///
  /// In en, this message translates to:
  /// **'Cutscenes'**
  String get variantCatCutscenes;

  /// No description provided for @variantCatAudio.
  ///
  /// In en, this message translates to:
  /// **'Audio'**
  String get variantCatAudio;

  /// No description provided for @variantCatMisc.
  ///
  /// In en, this message translates to:
  /// **'Misc'**
  String get variantCatMisc;

  /// No description provided for @variantCatOther.
  ///
  /// In en, this message translates to:
  /// **'Other'**
  String get variantCatOther;

  /// No description provided for @variantPickOneSuffix.
  ///
  /// In en, this message translates to:
  /// **'pick one'**
  String get variantPickOneSuffix;

  /// No description provided for @modVariantSelectAll.
  ///
  /// In en, this message translates to:
  /// **'Select all'**
  String get modVariantSelectAll;

  /// No description provided for @modVariantSelectNone.
  ///
  /// In en, this message translates to:
  /// **'None'**
  String get modVariantSelectNone;

  /// No description provided for @modVariantInstall.
  ///
  /// In en, this message translates to:
  /// **'Install'**
  String get modVariantInstall;

  /// No description provided for @modVariantInstallSelected.
  ///
  /// In en, this message translates to:
  /// **'Install {count}'**
  String modVariantInstallSelected(int count);

  /// No description provided for @modVariantTexture.
  ///
  /// In en, this message translates to:
  /// **'textures'**
  String get modVariantTexture;

  /// No description provided for @modVariantInstalledToast.
  ///
  /// In en, this message translates to:
  /// **'Installed {count} option(s)'**
  String modVariantInstalledToast(int count);

  /// No description provided for @modUninstallBusy.
  ///
  /// In en, this message translates to:
  /// **'Uninstalling mod…'**
  String get modUninstallBusy;

  /// No description provided for @modInstalled.
  ///
  /// In en, this message translates to:
  /// **'Installed: {id}'**
  String modInstalled(String id);

  /// No description provided for @modInstallFailed.
  ///
  /// In en, this message translates to:
  /// **'Install failed: {reason}'**
  String modInstallFailed(String reason);

  /// No description provided for @modInstallReasonUnknownDrop.
  ///
  /// In en, this message translates to:
  /// **'Unknown drop — the folder doesn\'t match any supported mod layout.'**
  String get modInstallReasonUnknownDrop;

  /// No description provided for @modInstallReasonInvalidMixed.
  ///
  /// In en, this message translates to:
  /// **'Invalid layout — a mod can\'t mix entities and wax-style configs.'**
  String get modInstallReasonInvalidMixed;

  /// No description provided for @modInstallReasonNativeEmpty.
  ///
  /// In en, this message translates to:
  /// **'No entity files found in entities/.'**
  String get modInstallReasonNativeEmpty;

  /// No description provided for @modInstallReasonDataEmpty.
  ///
  /// In en, this message translates to:
  /// **'No recognisable content found.'**
  String get modInstallReasonDataEmpty;

  /// No description provided for @modInstallReasonArchiveExtractFailed.
  ///
  /// In en, this message translates to:
  /// **'Couldn\'t extract the archive.'**
  String get modInstallReasonArchiveExtractFailed;

  /// No description provided for @modInstallReasonMoveFailed.
  ///
  /// In en, this message translates to:
  /// **'Couldn\'t move the files into nams/mods/.'**
  String get modInstallReasonMoveFailed;

  /// No description provided for @modInstallReasonTextureOnly.
  ///
  /// In en, this message translates to:
  /// **'This is a texture pack (only .dds files). Install it from the Textures tab instead.'**
  String get modInstallReasonTextureOnly;

  /// No description provided for @modUninstalled.
  ///
  /// In en, this message translates to:
  /// **'Removed: {id}'**
  String modUninstalled(String id);

  /// No description provided for @modCountFiles.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 file} other{{count} files}}'**
  String modCountFiles(int count);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
