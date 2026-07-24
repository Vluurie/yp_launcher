// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for German (`de`).
class AppLocalizationsDe extends AppLocalizations {
  AppLocalizationsDe([String locale = 'de']) : super(locale);

  @override
  String get appTitle => 'YoRHa Protocol Launcher';

  @override
  String get tooltipLauncherSource => 'Launcher-Quellcode';

  @override
  String get tooltipNamsSource => 'NAMS-Projektquellcode';

  @override
  String get tooltipGuide => 'Anleitung';

  @override
  String get tooltipDiscord => 'Discord';

  @override
  String get tooltipLanguage => 'Sprache';

  @override
  String get languageSupportNotice =>
      'Übersetzungen stammen aus der Community oder wurden automatisch erstellt und können ungenau sein. Die Maintainer sprechen nur Englisch – bitte stelle Fragen auf Englisch.';

  @override
  String get tooltipCopyCommand =>
      'Kopiert den NAMS-Befehl in die Zwischenablage, damit du ihn in ein Terminal einfügen und das Spiel manuell starten kannst.';

  @override
  String get notificationCommandCopied =>
      'Startbefehl kopiert – füge ihn in ein Terminal ein, um das Spiel manuell zu starten.';

  @override
  String get notificationCommandNotReady =>
      'Startbefehl konnte nicht erstellt werden – die Launcher-Dateien sind noch nicht bereit.';

  @override
  String get textureAutoRecommended => 'Automatisch (empfohlen)';

  @override
  String get detectionDlcPresent => 'DLC: vorhanden';

  @override
  String get detectionDlcNotDetected => 'DLC: nicht erkannt';

  @override
  String get detectionDlcPresentTooltip =>
      'DLC data100.cpk gefunden. Mods, die reine DLC-Outfit-Dateien mitliefern (pl000d, pl010d, pl020d), werden unverändert installiert.';

  @override
  String get detectionDlcNotDetectedTooltip =>
      'Kein DLC erkannt. Mods, die reine DLC-Outfit-Dateien mitliefern (pl000d, pl010d, pl020d), werden unter den Nicht-DLC-Namen (pl0000, pl0100, pl0200) installiert, damit sie im Spiel erscheinen.';

  @override
  String get detectionExeWolfLimitBreak => 'EXE: Wolf Limit Break';

  @override
  String get detectionExeOriginal => 'EXE: Original';

  @override
  String get detectionExeMissing => 'EXE: fehlt';

  @override
  String get detectionExeUnrecognised => 'EXE: nicht erkannt';

  @override
  String get detectionExeUnrecognisedTooltip =>
      'NieRAutomata.exe ist vorhanden, aber ihr Hash steht nicht auf unserer bekannten Liste. NAMS läuft trotzdem; dies ist nur ein Hinweis darauf, dass wir genau diesen Build noch nicht gesehen haben.';

  @override
  String get detectionExeWolfLimitBreakTooltip =>
      'Wolf-Limit-Break-Version von NieRAutomata.exe erkannt. NAMS benötigt diesen Patch nicht und wurde nie damit getestet. Das Spiel startet möglicherweise trotzdem, aber Leistungsprobleme, Speicherabstürze oder Mod-Inkompatibilitäten sind möglich. Für vollständige Unterstützung stelle die originale Steam-Programmdatei wieder her (Spieldateien in Steam überprüfen).';

  @override
  String get detectionExeLegacyWin7 => 'EXE: Windows-7-Version';

  @override
  String get detectionExeLegacyWin7Tooltip =>
      'Dies ist die alte Windows-7/8-Version von NieRAutomata.exe. NAMS benötigt die Windows-10/11-Steam-Version und kann diese hier nicht starten. Das kommt unter Proton/Linux häufig vor, wo Steam manchmal die Windows-7-Programmdatei herunterlädt.\n\nSo behebst du es:\n1. Lösche jede .exe in deinem NieRAutomata-Spielordner.\n2. Stelle Proton in Steam auf \'Proton Experimental\' (Rechtsklick auf das Spiel > Eigenschaften > Kompatibilität).\n3. Rechtsklick auf das Spiel > Eigenschaften > Installierte Dateien > Integrität der Spieldateien überprüfen.\n4. Starte das Spiel einmal über Steam, damit die korrekte Programmdatei behalten wird, und nutze dann wieder den Launcher.';

  @override
  String get launchOptionsTitle => 'STARTOPTIONEN';

  @override
  String get launchWrapperTitle => 'START-WRAPPER (LINUX)';

  @override
  String get launchWrapperDesc =>
      'Stellt dem Spielstart einen Befehl voran, z. B. gamescope oder mangohud. Der Launcher startet das Spiel über Proton, daher greifen Steam-Startoptionen hier nicht. Leer lassen, um normal zu starten. Wird beim nächsten Start angewendet.';

  @override
  String get launchWrapperHint => 'gamescope -w 2560 -h 1440 -f --';

  @override
  String get launchWrapperExample =>
      'Beispiele:\ngamescope -w 2560 -h 1440 -f --\nmangohud\ngamemoderun';

  @override
  String get tabLauncherSettings => 'Fehlerbehebung';

  @override
  String get troubleWrongExeTitle => 'Falsche Spielversion';

  @override
  String get troubleWrongExeSummary =>
      'Dies ist die alte Windows 7/8 NieRAutomata.exe. NAMS kann sie nicht starten.';

  @override
  String get troubleMissingFilesTitle => 'Fehlende Launcher-Dateien';

  @override
  String troubleMissingFilesSummary(String files) {
    return 'Fehlt: $files';
  }

  @override
  String get troubleMissingFilesDesc =>
      'Diese Dateien gehören zum portablen Launcher, sind aber verschwunden. Wahrscheinlich hat ein Antivirus (meist Windows Defender) sie in Quarantäne verschoben. Schließe den Launcher-Ordner im Antivirus aus und stelle die Dateien wieder her, oder lade den Launcher neu herunter.';

  @override
  String get troubleRecentErrorsTitle => 'Letzte NAMS-Fehler';

  @override
  String get troubleFoldersTitle => 'Ordner öffnen';

  @override
  String get troubleFoldersDesc =>
      'Schnellzugriff auf Ordner, nach denen ein Betreuer fragen könnte.';

  @override
  String get troubleOpenGame => 'Spielordner';

  @override
  String get troubleOpenNamsConfig => 'NAMS-Config';

  @override
  String get troubleOpenLogs => 'Logs';

  @override
  String get troubleOpenBins => 'Launcher-Ordner';

  @override
  String get troubleClearCacheTitle => 'Cache leeren';

  @override
  String get troubleClearCacheDesc =>
      'Löscht zwischengespeicherten Zustand, der veralten kann: Log-Dateien, den Erkennungs-Cache, die settings.json des Mods und den Runtime-Extract-Stempel. Deine Mods und Spieldateien bleiben unangetastet.';

  @override
  String get troubleClearCacheButton => 'Cache leeren';

  @override
  String get troubleClearCacheConfirm =>
      'Dies löscht Logs, den Erkennungs-Cache, die In-Game-settings.json des Mods und erzwingt beim nächsten Start eine neue Runtime-Extraktion. Mods und Spieldateien bleiben unangetastet. Fortfahren?';

  @override
  String troubleClearCacheDone(int count) {
    return 'Cache geleert ($count Log-Dateien entfernt).';
  }

  @override
  String get verifyInstallTitle => 'INSTALLATIONSDIAGNOSE';

  @override
  String get verifyInstallDesc =>
      'Führt die integrierten NAMS-Prüfungen aus, um zu diagnostizieren, warum das Spiel evtl. nicht startet (falscher Windows-Build, fehlende Steam-Dateien, Berechtigungen).';

  @override
  String get verifyInstallButton => 'Installation prüfen';

  @override
  String get verifyInstallRunning => 'Wird geprüft...';

  @override
  String get verifyInstallOk => 'Alle Prüfungen bestanden.';

  @override
  String get verifyInstallFailed =>
      'Einige Prüfungen fehlgeschlagen. Details unten.';

  @override
  String get verifyNoRuntime =>
      'Prüfung nicht möglich: keine Proton-/Wine-Laufzeit für diese Installation gefunden.';

  @override
  String get verifySteamNotRunning =>
      'Prüfung nicht möglich: Steam muss laufen und das Spiel besitzen.';

  @override
  String get verifyInstallError =>
      'Prüfung konnte nicht ausgeführt werden. Stelle sicher, dass ein Spielordner ausgewählt ist.';

  @override
  String get verifyInstallNoGameDir => 'Wähle zuerst deinen Spielordner aus.';

  @override
  String get verifyCheckSteamInstall => 'Steam-Installation';

  @override
  String get verifyCheckNierExe => 'Spiel-Programmdatei';

  @override
  String get verifyCheckSteamApi64 => 'Steam-API-Bibliothek';

  @override
  String get verifyCheckRuntimeWritable => 'Laufzeit beschreibbar';

  @override
  String get verifyCheckRuntimeCached =>
      'Laufzeitbibliothek zwischengespeichert';

  @override
  String get launchOptionMinimizeOnLaunch =>
      'Launcher während des Spielens minimieren';

  @override
  String get launchOptionPreferDedicatedGpu => 'Dedizierte GPU bevorzugen';

  @override
  String get launchOptionPreferDedicatedGpuTooltip =>
      'Weist das System an, das Spiel auf der dedizierten Grafikkarte statt auf der stromsparenden auszuführen. Nur relevant bei PCs mit zwei GPUs (z. B. Gaming-Laptops).';

  @override
  String get failTitlePanic => 'NAMS ist abgestürzt';

  @override
  String get failTitleUnknown => 'Spielstart fehlgeschlagen';

  @override
  String get failExplanationPanic =>
      'NAMS ist auf einen nicht behebbaren Fehler gestoßen, bevor das Spiel starten konnte. Das ist fast immer ein Bug – bitte teile den Bericht unten mit dem Maintainer.';

  @override
  String get failExplanationUnknown =>
      'Das Spiel wurde innerhalb von 60 Sekunden nicht gestartet und es wurde kein Fehler gemeldet.';

  @override
  String get failHintPanicShare =>
      'Kopiere den vollständigen Bericht unten und sende ihn an den Maintainer.';

  @override
  String get failHintPanicReboot =>
      'Versuche es nach einem Neustart erneut – manchmal löst sich ein hängengebliebener Handle von selbst.';

  @override
  String get failHintUnknownSpawned =>
      'NAMS scheint gestartet zu sein, aber das Spielfenster ist nie erschienen.';

  @override
  String get failHintUnknownTaskManager =>
      'Prüfe den Task-Manager – läuft NieRAutomata.exe, ist aber unsichtbar? Beende den Prozess und versuche es erneut.';

  @override
  String get failHintUnknownOtherLauncher =>
      'Stelle sicher, dass kein anderer Launcher bzw. kein DRM-Tool die EXE blockiert (FAR, Special K usw.).';

  @override
  String get failTitleNamsFailure => 'NAMS hat einen Fehler gemeldet';

  @override
  String get failExplanationNamsFailure =>
      'Eine NAMS-Prüfung ist fehlgeschlagen, bevor das Spiel starten konnte. Details findest du im Bericht unten.';

  @override
  String get failHintShareReport =>
      'Kopiere den vollständigen Bericht unten und teile ihn zur Diagnose.';

  @override
  String get failTitleInstallNotFound =>
      'NieR:Automata-Installation nicht gefunden';

  @override
  String get failExplanationInstallNotFound =>
      'NAMS konnte deine NieR:Automata-Installation nicht ermitteln. Der gespeicherte Pfad ist möglicherweise falsch oder die Steam-Erkennung ist fehlgeschlagen.';

  @override
  String get failHintRepickDirectory =>
      'Wähle dein Spielverzeichnis im Launcher erneut aus, um den gespeicherten Pfad zu aktualisieren.';

  @override
  String get failHintVerifyFiles =>
      'Überprüfe die Spieldateien in Steam (Bibliothek → NieR:Automata → Eigenschaften → Installierte Dateien → Überprüfen).';

  @override
  String get failTitleFolderCreate =>
      'Benötigter Ordner konnte nicht erstellt werden';

  @override
  String get failExplanationFolderCreate =>
      'NAMS konnte kein Verzeichnis neben NAMS.exe anlegen. Der Installationsordner ist möglicherweise schreibgeschützt.';

  @override
  String get failHintWritableFolder =>
      'Stelle sicher, dass der Launcher-Installationsordner (in dem NAMS.exe liegt) beschreibbar ist.';

  @override
  String get failHintProgramFiles =>
      'Liegt er unter Programme oder wird er von OneDrive synchronisiert, verschiebe den Launcher in einen normalen Ordner oder wähle per Rechtsklick „Immer auf diesem Gerät behalten“.';

  @override
  String get failTitleRuntimePrep =>
      'Vorbereitung der Laufzeitumgebung fehlgeschlagen';

  @override
  String get failExplanationRuntimePrep =>
      'NAMS konnte seine Laufzeitumgebung (game.bin / steam_api64.dll) nicht vorbereiten. Das liegt meist an fehlenden Schreibrechten oder an einem Virenscanner.';

  @override
  String get failHintAntivirusExclusions =>
      'Füge sowohl den Launcher-Installationsordner als auch deinen Spielordner zu den Ausnahmen deines Virenscanners hinzu und versuche es erneut.';

  @override
  String get failHintWritableCache =>
      'Stelle sicher, dass der Installationsordner beschreibbar ist, damit der Laufzeit-Cache erstellt werden kann.';

  @override
  String get failTitleHostFailure => 'NAMS-Hostfehler';

  @override
  String get failExplanationHostFailure =>
      'NAMS konnte den Spiel-Host (game.bin) nicht laden und starten. Das liegt meist an der Umgebung oder an beschädigten Dateien.';

  @override
  String get failHintReboot =>
      'Starte neu und versuche es erneut – manchmal löst sich ein hängengebliebener Handle von selbst.';

  @override
  String get failHintPersistShare =>
      'Falls das Problem bestehen bleibt, kopiere den vollständigen Bericht und sende ihn an den Maintainer.';

  @override
  String get failTitleSteamNotRunning => 'Steam läuft nicht / nicht angemeldet';

  @override
  String get failExplanationSteamNotRunning =>
      'NAMS konnte keine angemeldete Steam-Sitzung erreichen. Steam muss laufen und angemeldet sein.';

  @override
  String get failHintStartSteam =>
      'Starte Steam, melde dich an und starte das Spiel erneut.';

  @override
  String get failTitleSteamNotOwned =>
      'Steam-Konto besitzt NieR:Automata nicht';

  @override
  String get failExplanationSteamNotOwned =>
      'Das angemeldete Steam-Konto besitzt NieR:Automata nicht.';

  @override
  String get failHintSignInOwner =>
      'Melde dich mit dem Steam-Konto an, das NieR:Automata besitzt.';

  @override
  String get failTitleSteamCheckFailed => 'Steam-Prüfung fehlgeschlagen';

  @override
  String get failExplanationSteamCheckFailed =>
      'Bei der Überprüfung des Steam-Besitzes ist in NAMS ein interner Fehler aufgetreten.';

  @override
  String get failHintRestartSteam =>
      'Starte Steam und den Launcher neu und versuche es erneut.';

  @override
  String get failTitleInvalidArgs => 'Ungültige Startparameter';

  @override
  String get failExplanationInvalidArgs =>
      'Der Launcher hat Parameter übergeben, die NAMS nicht verarbeiten konnte. Das ist ein Fehler im Launcher.';

  @override
  String get failTitleExitedUnexpectedly => 'Spiel wurde unerwartet beendet';

  @override
  String get failExplanationExitedUnexpectedly =>
      'NAMS hat das Spiel gestartet, es wurde jedoch mit einem Fehlercode beendet. Das Spiel ist möglicherweise abgestürzt.';

  @override
  String get failHintCheckLogViewer =>
      'Prüfe den integrierten Log-Viewer (nams.log) auf Absturzdetails.';

  @override
  String get failHeadlinePanicked =>
      'NAMS ist mit einem schweren Fehler abgebrochen';

  @override
  String get failSectionWhatHappened => 'Was passiert ist';

  @override
  String get failSectionReportedByNams => 'Von NAMS gemeldet';

  @override
  String get failSectionTryThis => 'Versuche Folgendes';

  @override
  String get failSectionDiagnosticDetail => 'Diagnosedetails';

  @override
  String get failSectionLaunchManually => 'Manuell über ein Terminal starten';

  @override
  String get failSectionRawOutput => 'Rohausgabe';

  @override
  String get failManualCommandHint =>
      'Falls die Launcher-Oberfläche bei dir immer wieder fehlschlägt, füge dies in ein Terminal ein, um das Spiel manuell zu starten. Es ist exakt derselbe Befehl, den die Start-Schaltfläche ausführt.';

  @override
  String get failDetailOs => 'Betriebssystem';

  @override
  String get failDetailCause => 'Ursache';

  @override
  String get failDetailSuggested => 'Empfehlung';

  @override
  String get failActionCopyReport => 'Bericht kopieren';

  @override
  String get failActionOpenLogFile => 'Logdatei öffnen';

  @override
  String get logDetailOs => 'Betriebssystem';

  @override
  String get logDetailLocale => 'Gebietsschema';

  @override
  String get logNoModsInstalled => 'Keine Mods installiert.';

  @override
  String get logSectionSystem => 'System';

  @override
  String get logSectionModsNams => 'Mods (NAMS)';

  @override
  String get logSectionCutscenes => 'Cutscenes';

  @override
  String get logSectionTextures => 'Texturen';

  @override
  String get tooltipOpenInModManager => 'Im Mod-Manager öffnen';

  @override
  String get tooltipOpenInCutscenesTab => 'Im Tab „Cutscenes“ öffnen';

  @override
  String tooltipOpenInTexturesTab(String name) {
    return '$name\n\nIm Tab „Texturen“ öffnen';
  }

  @override
  String get actionCancel => 'Abbrechen';

  @override
  String get tooltipMinimize => 'Minimieren';

  @override
  String get tooltipMaximize => 'Maximieren';

  @override
  String get tooltipRestore => 'Wiederherstellen';

  @override
  String get tooltipClose => 'Schließen';

  @override
  String get infoText => 'Wähle dein Spiel aus, drücke auf Spielen – fertig.';

  @override
  String get helpPrefix => 'Launcher funktioniert nicht? Probiere den ';

  @override
  String get helpNaoLauncher => 'NAO Launcher';

  @override
  String get helpOr => ' oder die ';

  @override
  String get helpCommandLine => 'Kommandozeile';

  @override
  String get helpJoinDiscord => '. Tritt unserem ';

  @override
  String get helpDiscord => 'Discord';

  @override
  String get helpSuffix => ' bei, um Hilfe zu erhalten.';

  @override
  String get noFileSelected => 'Keine Datei ausgewählt';

  @override
  String get selectButton => 'AUSWÄHLEN';

  @override
  String get filePickerTitle => 'Spielprogramm auswählen';

  @override
  String get playButton => 'SPIELEN';

  @override
  String get stopButton => 'STOPPEN';

  @override
  String get statusReady => 'Bereit zum Starten!';

  @override
  String get statusSelectGame =>
      'Wähle zunächst die ausführbare Datei des Spiels aus.';

  @override
  String get statusPreparing => 'Launcher-Dateien werden vorbereitet ...';

  @override
  String get statusLaunching => 'Spiel wird gestartet ...';

  @override
  String get statusRunning => 'Das Spiel läuft.';

  @override
  String get statusStopped => 'Das Spiel wurde beendet.';

  @override
  String get statusStopping => 'Spiel wird beendet ...';

  @override
  String get errorInvalidExe =>
      'Das scheint nicht NieR:Automata zu sein. Stelle sicher, dass es die neueste Steam-Version ist.';

  @override
  String get errorFileNotExist => 'Die ausgewählte Datei ist nicht vorhanden';

  @override
  String get errorSelectFailed => 'Datei konnte nicht ausgewählt werden';

  @override
  String get errorStartFailed => 'Das Spiel konnte nicht gestartet werden.';

  @override
  String get errorStopFailed => 'Das Spiel konnte nicht beendet werden.';

  @override
  String errorFilesQuarantined(String files) {
    return 'Fehlende Launcher-Dateien: $files. Dies wird häufig durch Antivirensoftware verursacht. Wir laden Mods zur Laufzeit – das ist beim Modding von Spielen üblich, kann aber Fehlalarme auslösen. Stelle die Dateien aus der Quarantäne wieder her oder lade den Launcher erneut herunter und füge anschließend eine Ausnahme für den Installationsordner des Launchers hinzu (den Ordner, der NAMS.exe enthält).';
  }

  @override
  String get notifyFilesQuarantined =>
      'Fehlende Launcher-Dateien erkannt. Dies wird häufig durch Antivirensoftware verursacht. Wir laden Mods zur Laufzeit, was beim Modding von Spielen üblich ist, aber Fehlalarme auslösen kann. Stelle die Dateien aus der Quarantäne wieder her oder lade den Launcher erneut herunter und füge anschließend eine Ausnahme für den Installationsordner des Launchers hinzu (den Ordner, der NAMS.exe enthält).';

  @override
  String get featureReshade =>
      'ReShade – bereits installiert? YP erkennt es automatisch.';

  @override
  String get featureTextures =>
      'HD-Texturen – lege Texturen unter nams/inject/textures/ ab oder sie werden aus SK_Res/ übernommen.';

  @override
  String get featureLodMod =>
      'LOD Mod – integrierte Grafikoptimierungen für Schatten, Details und Pop-ins. Standardmäßig deaktiviert.';

  @override
  String get tooltipEditConfigs =>
      'Grafikeinstellungen ändern, ohne Dateien zu bearbeiten';

  @override
  String get tooltipOpenLogs => 'Protokollordner im Explorer öffnen';

  @override
  String get tooltipCreateShortcut =>
      'Desktopverknüpfung zum Starten mit YoRHa Protocol erstellen';

  @override
  String get shortcutName => 'NieR Automata (YoRHa Protocol)';

  @override
  String get shortcutDescription => 'NieR:Automata mit YoRHa Protocol starten';

  @override
  String get notifyShortcutCreated => 'Desktopverknüpfung erstellt!';

  @override
  String get notifyShortcutFailed =>
      'Desktopverknüpfung konnte nicht erstellt werden.';

  @override
  String get notifyLodModMigrated =>
      'Deine alten LodMod.ini-Einstellungen wurden gefunden, in lodmod.toml importiert und LodMod wurde aktiviert.';

  @override
  String get notifyReShadeDetected =>
      'ReShade erkannt – standardmäßig deaktiviert. NAMS enthält bereits eine gepatchte native Tiefenschärfe, daher ist ReShade optional. Du kannst es jederzeit im NAMS-Konfigurationsreiter wieder aktivieren (ReShade-Laden deaktivieren → aus).';

  @override
  String get notifyNaiomMigrated =>
      'Deine alten NAIOM-Einstellungen wurden gefunden und in nams.toml importiert. Sieh im NAIOM-Reiter nach. Du kannst die alten NAIOM-Dateien (dinput8.dll, NAIOM.ini) aus dem Spielordner entfernen.';

  @override
  String notifyNaiomSkipped(String entries) {
    return 'Einige NAIOM-Belegungen verwenden Tasten, die NAMS nicht unterstützt, und wurden nicht importiert: $entries. Belege sie im NAIOM-Reiter neu.';
  }

  @override
  String notifyPlatformUnsupported(String platform) {
    return 'Auf $platform wurde keine Windows-Kompatibilitätsschicht gefunden, daher kann das Spiel von hier aus nicht gestartet werden. Mods, Texturen und Konfigurationen funktionieren weiterhin. Installiere CrossOver und richte NieR:Automata in einer Bottle ein, um das Starten zu ermöglichen.';
  }

  @override
  String get notifyReShadeIncompatible =>
      'ReShade mit Add-on-/ImGui-Unterstützung erkannt – nicht kompatibel. Verwende die Standardversion von ReShade ohne Add-on-Unterstützung.';

  @override
  String notifyTexturesDetected(String folder) {
    return 'HD-Texturen in $folder gefunden – sie werden beim Start geladen.';
  }

  @override
  String get errorAppDataNotFound =>
      'Umgebungsvariable APPDATA wurde nicht gefunden';

  @override
  String get errorWinePrefixNotFound =>
      'Wine-Präfix wurde nicht gefunden. Bitte setze die Umgebungsvariable WINEPREFIX.';

  @override
  String get errorHomeNotFound => 'Umgebungsvariable HOME wurde nicht gefunden';

  @override
  String get errorNoWineUser =>
      'Im Wine-Präfix wurde kein Benutzerverzeichnis gefunden';

  @override
  String errorWineUsersNotFound(String prefix) {
    return 'Verzeichnis drive_c/users wurde im Wine-Präfix $prefix nicht gefunden';
  }

  @override
  String errorPlatformNotSupported(String os) {
    return 'Die Plattform $os wird nicht unterstützt';
  }

  @override
  String errorExeNotFound(String dir) {
    return 'NieRAutomata.exe wurde in $dir nicht gefunden';
  }

  @override
  String get errorDirNotWritable => 'Launcher-Ordner ist schreibgeschützt';

  @override
  String errorDirNotWritableBody(String dir) {
    return 'In den YP-Launcher-Ordner kann nicht geschrieben werden:\n$dir\n\nNAMS speichert seinen Laufzeitcache, Plugins und Protokolle neben NAMS.exe. Daher muss der Launcher-Ordner beschreibbar sein.\n\nSo behebst du das Problem:\n1. Schließe den Launcher.\n2. Verschiebe den gesamten entpackten YP-Launcher-Ordner aus Program Files (oder einem anderen geschützten Speicherort) in einen normalen Ordner, der dir gehört, zum Beispiel Desktop, Dokumente oder D:\\Games\\YP Launcher.\n3. Starte den Launcher am neuen Speicherort erneut.\n\nAlternative: Klicke mit der rechten Maustaste auf die Launcher-.exe und wähle „Als Administrator ausführen“, um Schreibzugriff am aktuellen Speicherort zu erlauben.';
  }

  @override
  String get errorGameDirNotWritable => 'Spielordner ist schreibgeschützt';

  @override
  String errorGameDirNotWritableBody(String gameDir, String namsDir) {
    return 'In den Spielordner kann nicht geschrieben werden:\n$gameDir\n\nNAMS schreibt Mods und Konfigurationen nach:\n$namsDir\nkann dort aber keine Dateien erstellen. NieR ist wahrscheinlich unter C:\\Program Files (x86)\\Steam installiert, das von Windows geschützt wird.\n\nSo behebst du das Problem (empfohlen – verschiebe die Steam-Bibliothek aus Program Files):\n1. Öffne Steam > Einstellungen > Speicher.\n2. Klicke auf das Laufwerks-Dropdown > „Laufwerk hinzufügen“ und wähle einen Ordner auf einem anderen Laufwerk, zum Beispiel D:\\SteamLibrary.\n3. Öffne deine Bibliothek, klicke mit der rechten Maustaste auf NieR:Automata > Eigenschaften > Installierte Dateien > „Installationsordner verschieben“ und verschiebe das Spiel in die neue Bibliothek.\n4. Wähle das Spiel in diesem Launcher erneut aus und drücke wieder auf Spielen.\n\nSchnelle Alternative: Klicke mit der rechten Maustaste auf die Launcher-.exe und wähle „Als Administrator ausführen“, damit sie in Program Files schreiben kann. Das Verschieben der Bibliothek ist langfristig die sauberere Lösung.';
  }

  @override
  String get errorNoCompatLayer => 'CrossOver wurde nicht gefunden';

  @override
  String get errorNoCompatLayerBody =>
      'Zum Ausführen von NieR:Automata auf diesem System wird CrossOver benötigt, das Windows-Programme unter macOS ausführt. Es wurde unter /Applications nicht gefunden.\n\nOhne CrossOver kann der Launcher weiterhin Mods, Texturen und Konfigurationen verwalten – nur das Starten des Spiels ist nicht verfügbar.\n\nSo behebst du das Problem:\n1. Installiere CrossOver von codeweavers.com.\n2. Installiere Steam und NieR:Automata in einer CrossOver-Bottle.\n3. Wähle NieRAutomata.exe aus dieser Bottle in diesem Launcher aus.';

  @override
  String get errorNoCompatLayerLinux => 'Kein Proton oder Wine gefunden';

  @override
  String get errorNoCompatLayerLinuxBody =>
      'Um NieR:Automata unter Linux auszuführen, wird Proton (empfohlen) oder Wine benötigt, aber es wurde nichts gefunden.\n\nOhne eine solche Laufzeitumgebung kann der Launcher weiterhin Mods, Texturen und Konfigurationen verwalten - nur das Starten des Spiels ist nicht möglich.\n\nSo behebst du es:\n1. Installiere in Steam ein Proton-Build (Proton Experimental funktioniert gut). Liegt es auf einem anderen Laufwerk, durchsucht der Launcher jetzt jede Steam-Bibliothek.\n2. Stelle sicher, dass du NieRAutomata.exe aus deiner Steam-Bibliothek ausgewählt hast (ein Pfad, der steamapps/common enthält).\n3. Oder setze YP_PROTON_PATH vor dem Start des Launchers auf deine Proton-Datei, z. B. YP_PROTON_PATH=\"\$HOME/.steam/steam/steamapps/common/Proton - Experimental/proton\".';

  @override
  String get errorProtonMissing => 'Proton wurde nicht gefunden';

  @override
  String errorProtonMissingBody(String path) {
    return 'Die konfigurierte Proton-Laufzeitumgebung fehlt unter:\n$path\n\nInstalliere Proton über Steam neu oder setze YP_PROTON_PATH vor dem Start des Launchers auf eine gültige Proton-Datei.';
  }

  @override
  String get errorNoZDrive => 'Das Wine-Präfix besitzt kein Laufwerk Z:';

  @override
  String errorNoZDriveBody(String prefix) {
    return 'Wine ordnet Z: dem Dateisystem des Hosts zu. So übergibt der Launcher NAMS.exe an das Spiel. In diesem Präfix fehlt dosdevices/z::\n$prefix\n\nDies ist eine CrossOver-Standardeinstellung, daher wurde die Bottle wahrscheinlich verändert. Eine neue Bottle zu erstellen und das Spiel dort neu zu installieren, ist die schnellste Lösung.';
  }

  @override
  String get errorExeOutsidePrefix =>
      'Diese ausführbare Datei befindet sich nicht in einer Bottle';

  @override
  String errorExeOutsidePrefixBody(String exeName, String path) {
    return 'Der Launcher startet das Spiel über CrossOver, daher muss sich $exeName in einer CrossOver-Bottle befinden:\n$path\n\nInstalliere das Spiel in einer Bottle und wähle anschließend die ausführbare Datei dort aus.';
  }

  @override
  String get headerNams => 'NAMS';

  @override
  String get headerLodMod => 'LOD MOD';

  @override
  String get headerTextures => 'TEXTUREN';

  @override
  String get headerYorhaProtocol => 'YORHA PROTOCOL';

  @override
  String get headerNaiom => 'NAIOM';

  @override
  String get headerCutscenes => 'ZWISCHENSEQUENZEN';

  @override
  String get tooltipEditsNamsToml => 'Bearbeitet nams/nams.toml';

  @override
  String get tooltipEditsLodmodToml => 'Bearbeitet nams/lodmod.toml';

  @override
  String get tooltipEditsTextureInjectionToml =>
      'Bearbeitet nams/texture_injection.toml';

  @override
  String get tooltipEditsSettingsJson =>
      'Bearbeitet %APPDATA%\\NAMS\\settings.json';

  @override
  String get tooltipEditsNaiom =>
      'Bearbeitet die [mouse]-Einstellungen in nams/nams.toml. Die meisten Einstellungen gelten nach dem Speichern; einige erfordern einen Neustart des Spiels.';

  @override
  String get tooltipCutscenesLocation =>
      'Mods: nams/cutscenes/<mod_name>/movie/*.usm';

  @override
  String get cardGeneral => 'ALLGEMEIN';

  @override
  String get cardLoading => 'LADEN';

  @override
  String get cardHeapOverrides => 'HEAP-ÜBERSCHREIBUNGEN';

  @override
  String get cardLevelOfDetail => 'DETAILSTUFE';

  @override
  String get cardAmbientOcclusion => 'UMGEBUNGSVERDECKUNG';

  @override
  String get cardPostProcessing => 'NACHBEARBEITUNG';

  @override
  String get cardShadows => 'SCHATTEN';

  @override
  String get cardGlobalIllumination => 'GLOBALE BELEUCHTUNG';

  @override
  String get cardPreloading => 'VORLADEN';

  @override
  String get cardTextureConfig => 'KONFIGURATION';

  @override
  String get cardInstallTextures => 'TEXTUREN INSTALLIEREN';

  @override
  String get cardHowItWorks => 'SO FUNKTIONIERT ES';

  @override
  String get cardKeybinds => 'TASTENBELEGUNGEN';

  @override
  String get cardWorkspace => 'ARBEITSBEREICH';

  @override
  String get cardCheats => 'CHEATS';

  @override
  String get cardRandomizer => 'ZUFALLSGENERATOR';

  @override
  String get cardThirdPersonCamera => 'DRITTPERSON-KAMERA';

  @override
  String get cardPodAiming => 'POD / ZIELEN';

  @override
  String get cardMisc => 'SONSTIGES';

  @override
  String get cardMovementBindings => 'BEWEGUNGSBELEGUNGEN';

  @override
  String get cardCombatBindings => 'KAMPFBELEGUNGEN';

  @override
  String get cardNonStandardBindings => 'NICHT STANDARDMÄSSIGE BELEGUNGEN';

  @override
  String get cardMenuNavigation => 'MENÜNAVIGATION';

  @override
  String get cardStructure => 'STRUKTUR';

  @override
  String get buttonSave => 'SPEICHERN';

  @override
  String get buttonDiscard => 'VERWERFEN';

  @override
  String get buttonCancel => 'ABBRECHEN';

  @override
  String get buttonInstall => 'INSTALLIEREN';

  @override
  String get buttonDelete => 'LÖSCHEN';

  @override
  String get buttonYes => 'Ja';

  @override
  String get buttonNo => 'Nein';

  @override
  String get buttonContinue => 'Weiter';

  @override
  String get buttonBack => 'Zurück';

  @override
  String get buttonGetStarted => 'Loslegen';

  @override
  String get buttonAutoDetect => 'Automatisch erkennen';

  @override
  String get buttonSelectManually => 'Manuell auswählen';

  @override
  String get buttonGoToLauncher => 'Zum Launcher';

  @override
  String get unsavedChangesTitle => 'Ungespeicherte Änderungen';

  @override
  String get unsavedChangesMessage =>
      'Du hast ungespeicherte Änderungen. Verwerfen?';

  @override
  String get stay => 'BLEIBEN';

  @override
  String get discard => 'VERWERFEN';

  @override
  String get enterValidNumber => 'Gib eine gültige Zahl ein';

  @override
  String get pressKey => 'Drücken ...';

  @override
  String get tabLauncher => 'Launcher';

  @override
  String get tabYorhaProtocol => 'YP-Devkit';

  @override
  String get tabNams => 'NAMS';

  @override
  String get tabLodMod => 'LOD Mod';

  @override
  String get tabNaiom => 'NAIOM';

  @override
  String get tabTextures => 'Texturen';

  @override
  String get tabMods => 'Mod-Manager';

  @override
  String get tabCutscenes => 'Cutscenes';

  @override
  String get tabThirdParty => 'Drittanbieter';

  @override
  String get thirdPartyTitle => 'Drittanbieter-Runtimes';

  @override
  String get thirdPartySubtitle =>
      'ReShade und 3DMigoto, von NAMS aus einem verwalteten Ordner geladen. Kein Umbenennen, keine DLL-Konflikte.';

  @override
  String get thirdPartyReShadeHeader => 'RESHADE';

  @override
  String get thirdPartyMigotoHeader => '3DMIGOTO';

  @override
  String get thirdPartyReShadeHowto =>
      'Installiere ReShade ganz normal in deinen NieRAutomata-Ordner (wähle dxgi als API). Der Launcher verschiebt es nach NAMS und richtet die Pfade ein.';

  @override
  String get thirdPartyMigotoHowto =>
      'Zieh ein 3DMigoto-Shader-Mod-Archiv hierher. Der Launcher installiert es und setzt das Loader-Target, damit NAMS es einhakt.';

  @override
  String get thirdPartyStatusInstalled => 'Installiert';

  @override
  String get thirdPartyStatusNotInstalled => 'Nicht installiert';

  @override
  String get thirdPartyStatusFoundInGame =>
      'Im Spielordner gefunden — klicke auf Importieren, um es für NAMS einzurichten.';

  @override
  String get thirdPartyEnable => 'Aktiv';

  @override
  String get thirdPartyImport => 'In NAMS importieren';

  @override
  String get thirdPartyRepair => 'Reparieren';

  @override
  String get thirdPartyRemove => 'Entfernen';

  @override
  String get thirdPartyGetReShade => 'ReShade holen';

  @override
  String get thirdPartyShadersMissing =>
      'Noch keine Effekte installiert — füge ein Preset oder Shader-Paket hinzu, sonst passiert nichts.';

  @override
  String get thirdPartyOpenFolder => 'Ordner öffnen';

  @override
  String thirdPartyPresetsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Presets',
      one: '1 Preset',
      zero: 'Keine Presets',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyInstallCard => 'INSTALLIEREN';

  @override
  String get thirdPartyDropHere =>
      'ReShade-Preset / 3DMigoto-Mod hierher ziehen';

  @override
  String get thirdPartyImported => 'In NAMS importiert';

  @override
  String get thirdPartyInstalled => 'Installiert';

  @override
  String get thirdPartyInstallFailed =>
      'Diese Datei konnte nicht installiert werden';

  @override
  String get thirdPartyRedirectMods =>
      'Das ist ein Spieldaten-Mod — nutze den Mod-Manager-Tab.';

  @override
  String get thirdPartyRedirectTextures =>
      'Das ist ein Textur-Paket — nutze den Texturen-Tab.';

  @override
  String get thirdPartyUnsupported =>
      'Dieser Dateityp wird hier nicht unterstützt.';

  @override
  String get thirdPartyLodModPrompt =>
      'Das ist ein LodMod. NAMS hat LodMod eingebaut — die Einstellungen in den LodMod-Tab importieren?';

  @override
  String get thirdPartyStatusActive => 'Aktiv';

  @override
  String get thirdPartyStatusInactive => 'Installiert, im NAMS-Tab deaktiviert';

  @override
  String get thirdPartyVersion => 'Version';

  @override
  String get thirdPartyAddonBuild => 'Addon-Build';

  @override
  String get thirdPartyPresets => 'PRESETS';

  @override
  String get thirdPartyNonePresets => 'Keine Presets installiert';

  @override
  String get thirdPartyShaderRepos => 'SHADER';

  @override
  String get thirdPartyNoneShaders =>
      'Keine Shader — Effekte kompilieren nicht';

  @override
  String thirdPartyShaderCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Effekte',
      one: '1 Effekt',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyAddons => 'ADDONS';

  @override
  String get thirdPartyFiles => 'DATEIEN';

  @override
  String get thirdPartyShaderFixes => 'ShaderFixes';

  @override
  String thirdPartyShaderFixCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Fixes',
      one: '1 Fix',
      zero: 'keine',
    );
    return '$_temp0';
  }

  @override
  String get thirdPartyLoaderTarget => 'Loader-Target';

  @override
  String get thirdPartyHunting => 'Hunting';

  @override
  String get thirdPartyHuntingOn => 'an';

  @override
  String thirdPartyUpdateTitle(String name) {
    return 'Installiertes $name ersetzen?';
  }

  @override
  String thirdPartyUpdateBody(String name) {
    return '$name ist bereits installiert. Die abgelegte Datei ist ein anderer Build. Das installierte damit ersetzen?';
  }

  @override
  String get thirdPartyUpdateInstalledLabel => 'Jetzt installiert';

  @override
  String get thirdPartyUpdateIncomingLabel => 'Abgelegt';

  @override
  String get thirdPartyUpdateReplace => 'Ersetzen';

  @override
  String get thirdPartyUpdateKeep => 'Aktuelle behalten';

  @override
  String get thirdPartyUpdateSkipped => 'Installierte Version behalten.';

  @override
  String get thirdPartyD3dCompilerMissing =>
      'Keine d3dcompiler_47.dll hier. Unter Wine greift ReShade auf Wines Stub zurück und Effekte kompilieren nicht. Installiere eine ReShade-Version, die sie mitliefert, oder lege eine native d3dcompiler_47.dll in diesen Ordner.';

  @override
  String get thirdPartyBanner =>
      'Die Tools auf diesem Tab sind Drittanbieter-Software. NAMS lädt sie nur. NAMS implementiert sie nicht und kann ihr Verhalten nicht ändern. Alles, was du darüber installierst, läuft eigenständig, daher liegen Korrektheit, Kompatibilität und Sicherheit in deiner Verantwortung. Wenn etwas nicht richtig funktioniert, liegt das am Tool oder am Mod, nicht an NAMS.';

  @override
  String get thirdPartyConfigSection => 'EINSTELLUNGEN';

  @override
  String get thirdPartyRestartRequired =>
      'wird beim nächsten Spielstart übernommen';

  @override
  String get thirdPartyReShadePerformanceMode => 'Performance-Modus';

  @override
  String get thirdPartyReShadePerformanceModeHint =>
      'Effekte beim Start nicht neu kompilieren. Beim Anpassen von Effekten ausschalten.';

  @override
  String get thirdPartyReShadeShowFps => 'FPS anzeigen';

  @override
  String get thirdPartyReShadeShowFpsHint =>
      'Zeigt die Bildrate in der Ecke des ReShade-Overlays.';

  @override
  String get thirdPartyReShadeShowClock => 'Uhr anzeigen';

  @override
  String get thirdPartyReShadeShowClockHint =>
      'Zeigt die aktuelle Uhrzeit in der Ecke des ReShade-Overlays.';

  @override
  String get thirdPartyReShadeActivePreset => 'Aktives Preset';

  @override
  String get thirdPartyReShadeOverlayKey => 'Overlay-Taste';

  @override
  String get thirdPartyReShadeNoKey => 'nicht belegt';

  @override
  String get thirdPartyMigotoHunting => 'Hunting';

  @override
  String get thirdPartyMigotoHuntingOff => 'Aus';

  @override
  String get thirdPartyMigotoHuntingOn => 'An';

  @override
  String get thirdPartyMigotoHuntingNoMarking => 'An, ohne Markierung';

  @override
  String get thirdPartyMigotoHuntingHint =>
      'Entwicklermodus: Shader durchschalten, um sie zu finden und zu dumpen. Für normales Spielen aus lassen.';

  @override
  String get thirdPartyMigotoMarkingMode => 'Markierungsmodus';

  @override
  String get thirdPartyMigotoMarkingModeHint =>
      'Wie der gesuchte Shader im Spiel hervorgehoben wird: überspringen (keine Änderung), Original, Pink oder Mono.';

  @override
  String get thirdPartyMigotoMarkingSkip => 'Überspringen';

  @override
  String get thirdPartyMigotoMarkingOriginal => 'Original';

  @override
  String get thirdPartyMigotoMarkingPink => 'Pink';

  @override
  String get thirdPartyMigotoMarkingMono => 'Mono';

  @override
  String get thirdPartyMigotoVerboseOverlay => 'Ausführliches Overlay';

  @override
  String get thirdPartyMigotoVerboseOverlayHint =>
      'Zeigt detaillierten 3DMigoto-Status auf dem Bildschirm. Nützlich beim Debuggen, sonst störend.';

  @override
  String get thirdPartyMigotoCacheShaders => 'Shader zwischenspeichern';

  @override
  String get thirdPartyMigotoCacheShadersHint =>
      'Kompilierte Shader über Starts hinweg wiederverwenden. Nur zum Debuggen ausschalten.';

  @override
  String get thirdPartyMigotoCheckForegroundWindow =>
      'Nur im Vordergrund hooken';

  @override
  String get thirdPartyMigotoCheckForegroundWindowHint =>
      'Overlay und Tastenkürzel nur anwenden, wenn das Spielfenster im Fokus ist.';

  @override
  String get thirdPartyConfigApplied => 'Einstellungen gespeichert.';

  @override
  String get thirdPartyConfigNoIni =>
      'Noch keine Konfigurationsdatei — wird bei der Installation erstellt.';

  @override
  String get tabSectionGeneral => 'Allgemein';

  @override
  String get tabSectionConfig => 'Konfiguration';

  @override
  String get tabSectionMods => 'Mods';

  @override
  String get tabDividerConfig => 'KONFIGURATION';

  @override
  String get tabDividerMods => 'MODS';

  @override
  String get infoBarVersionPrefix => 'YP Launcher ';

  @override
  String get infoBarLogs => 'PROTOKOLLE';

  @override
  String get infoBarFaq => 'FAQ';

  @override
  String get infoBarWhatsNew => 'NEUIGKEITEN';

  @override
  String get infoBarShortcut => 'VERKNÜPFUNG';

  @override
  String get tooltipFaq => 'Brauche ich weiterhin andere Mods?';

  @override
  String get chipLodModOn => 'LOD MOD AN';

  @override
  String get chipLodModOff => 'LOD MOD AUS';

  @override
  String get chipReShade => 'ReShade';

  @override
  String get chipNoTextures => 'Keine Texturen';

  @override
  String get chipNoMods => 'Keine Mods';

  @override
  String get chipNoCutsceneMod => 'Keine Cutscene-Mod';

  @override
  String chipTexturesCount(String details) {
    return 'Texturen ($details)';
  }

  @override
  String chipModsCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Mods',
      one: 'Mod',
    );
    return '$count $_temp0';
  }

  @override
  String chipInjectedCount(int count) {
    return '$count injiziert';
  }

  @override
  String get chipSkRes => 'SK_Res';

  @override
  String chipCutsceneMod(int width, int height, String codec) {
    return 'Cutscene-Mod (${width}x$height $codec)';
  }

  @override
  String get warningPluginLoadingDisabled =>
      'Das Laden von Plugins ist deaktiviert – der YoRHa-Protocol-Arbeitsbereich wird nicht geladen';

  @override
  String get warningReShadeDisabled =>
      'Das automatische Laden von ReShade ist deaktiviert';

  @override
  String get warningTextureInjectionDisabled =>
      'Die Texturinjektion ist deaktiviert';

  @override
  String get heapOverridesDescription =>
      'Zusätzlicher Speicher oberhalb der NAMS-Standardwerte. Übergeordnete Heaps wachsen automatisch. Erhöhe die Werte nur, wenn Mods mehr Speicher benötigen. Neustart erforderlich.';

  @override
  String get lodModDescription =>
      'In NAMS integrierte Patches für bessere Grafikqualität, inspiriert von Automata-LodMod von emoose. Entfernt LOD-Pop-ins, schärft Schatten und Umgebungsverdeckung, erzwingt Schattenwurf für alle Objekte einschließlich Vegetation, deaktiviert manuelles Culling, damit Objekte nicht plötzlich ein- oder ausgeblendet werden, und entfernt die Vignette.';

  @override
  String get dropModelModHere => 'Modell-Mod-Ordner oder Archiv hier ablegen';

  @override
  String get dropToInstall => 'Zum Installieren ablegen';

  @override
  String get orClickToBrowse => 'oder zum Durchsuchen klicken';

  @override
  String get mixedModDetected => 'Gemischte Mod erkannt';

  @override
  String get noOutfitsInstalled => 'Noch keine Outfits installiert';

  @override
  String get defaultNoMod => 'Standard (keine Mod)';

  @override
  String get clearAllStartupOutfits => 'Alle Start-Outfits entfernen';

  @override
  String get removeOutfitTitle => 'Outfit entfernen?';

  @override
  String get noModelFoundError =>
      'Kein Modell gefunden. Benötigt pl000d.dat/.dtt (2B), pl010d.dat/.dtt (A2) oder pl020d.dat/.dtt (9S).';

  @override
  String get unsupportedArchiveFormat => 'Nicht unterstütztes Archivformat.';

  @override
  String get extractingArchive => 'Archiv wird entpackt ...';

  @override
  String extractingArchivePercent(int percent) {
    return 'Wird entpackt – $percent%';
  }

  @override
  String extractingArchivePercentFile(int percent, String file) {
    return 'Wird entpackt: $percent% – $file';
  }

  @override
  String get cutsceneScanningArchive =>
      'Archiv wird nach einem movie-Ordner durchsucht ...';

  @override
  String cutsceneCopyingFile(int current, int total, String name) {
    return 'Kopieren $current/$total – $name';
  }

  @override
  String cutsceneCopyingFileBytes(
    int current,
    int total,
    String name,
    String mbDone,
    String mbTotal,
  ) {
    return 'Kopieren $current/$total – $name ($mbDone / $mbTotal MB)';
  }

  @override
  String get failedToExtractArchive => 'Archiv konnte nicht entpackt werden.';

  @override
  String get extractFailedDiskFull =>
      'Entpacken fehlgeschlagen: Auf dem temporären Laufwerk ist nicht genügend freier Speicherplatz vorhanden. Gib Speicherplatz frei und versuche es erneut.';

  @override
  String get textureDeleteConfirmTitle => 'Texturpaket löschen?';

  @override
  String textureDeleteConfirmBody(String name) {
    return '$name dauerhaft aus nams/inject/textures/ entfernen? Dies kann nicht rückgängig gemacht werden.';
  }

  @override
  String get textureDeleteLabel => 'Löschen';

  @override
  String get busyDeletingTexture => 'Texturpaket wird gelöscht ...';

  @override
  String get busyDeletingCutscene => 'Cutscene-Mod wird gelöscht ...';

  @override
  String get busyCloseTitle => 'Vorgang läuft';

  @override
  String get busyCloseBody =>
      'Der Launcher installiert, löscht oder entpackt gerade Dateien. Wenn du ihn jetzt schließt, können beschädigte oder unvollständige Dateien auf dem Datenträger zurückbleiben. Warte, bis der aktuelle Vorgang abgeschlossen ist, oder erzwinge das Schließen.';

  @override
  String get busyCloseForce => 'Schließen erzwingen';

  @override
  String extractFailedDetail(String detail) {
    return 'Entpacken fehlgeschlagen: $detail';
  }

  @override
  String get noOutfitsInstalledNotif => 'Keine Outfits installiert.';

  @override
  String get dialogSelectModFolder => 'Modell-Mod-Ordner auswählen';

  @override
  String get dialogNameOutfitShown =>
      'Dieser Name wird im Spiel beim Wechseln von Outfits angezeigt.';

  @override
  String get dropTextureHere => 'Texturordner oder Archiv hier ablegen';

  @override
  String get installedToTextures => 'Installiert unter: nams/inject/textures/';

  @override
  String get installingTextures => 'Texturen werden installiert ...';

  @override
  String get textureDropAnalyzing => 'Abgelegte Dateien werden analysiert ...';

  @override
  String get textureDropNoDds =>
      'In den abgelegten Dateien wurden keine .dds-Texturen gefunden. Es werden nur .dds-Dateien unterstützt.';

  @override
  String get cutsceneDropAnalyzing => 'Abgelegte Dateien werden analysiert ...';

  @override
  String get cutsceneDropNoUsm =>
      'In den abgelegten Dateien wurden keine .usm-Cutscenes gefunden.';

  @override
  String get modDropAnalyzing => 'Abgelegte Dateien werden analysiert ...';

  @override
  String get modDropNotAMod =>
      'Das sieht nicht wie eine NAMS-Mod aus. Lege einen Ordner oder ein Archiv ab, das entities/, wax/, data/ oder eine mod.toml enthält.';

  @override
  String get modDropMisroutedCutscenes =>
      'Das sieht wie eine eigenständige Cutscene-Mod aus – lege sie stattdessen im Reiter Cutscenes ab. Gebündelte Cutscenes gehören in eine Mod, die bereits entities/ oder mod.toml enthält.';

  @override
  String modSideInstalledTextures(String names) {
    return 'Gebündelte Texturpakete wurden außerdem nach nams/inject/textures/ installiert: $names';
  }

  @override
  String modLooseUnpairedWarning(String names) {
    return 'Installiert, aber bei einigen Dateien fehlt das zugehörige Vanilla-Gegenstück (.dat/.dtt): $names. Der Mod funktioniert evtl. nicht vollständig.';
  }

  @override
  String get modBundledTexturesLabel => 'Gebündelte Texturen';

  @override
  String get modBundledCutscenesLabel => 'Gebündelte Cutscenes';

  @override
  String textureBundledWithMod(String modId) {
    return 'Gebündelt mit Mod: $modId';
  }

  @override
  String modUninstallAlsoTexturesLabel(String names) {
    return 'Gebündelte Texturpakete ebenfalls löschen: $names';
  }

  @override
  String get noTextures => 'Keine Texturen';

  @override
  String get noTexturesInstalled => 'Keine Texturen installiert';

  @override
  String get textureConflictHint =>
      'Beide Mods werden geladen. Sie verändern teilweise dieselben Inhalte. Setze die wichtigere Mod nach oben.';

  @override
  String get noConflictsFound =>
      'Keine Konflikte gefunden. Alle Mods werden unabhängig voneinander geladen.';

  @override
  String get selectTextureFiles => 'Texturdateien oder Archive auswählen';

  @override
  String get noTextureFilesFound =>
      'Keine Texturdateien gefunden (.dds, .png, .tga)';

  @override
  String get cheatsAppliedNote => 'Wird einmal beim Spielstart angewendet.';

  @override
  String get wipLabel => 'IN ARBEIT';

  @override
  String get dropCutsceneHere => 'Cutscene-Mod-Ordner oder Archiv hier ablegen';

  @override
  String get cutsceneMovieHint =>
      'Die Mod muss einen movie/-Ordner mit .usm-Dateien enthalten';

  @override
  String get cutsceneNamingTitle => 'Diese Cutscene-Mod benennen';

  @override
  String get removeCutsceneModTitle => 'Cutscene-Mod entfernen?';

  @override
  String get noCutsceneModsInstalled => 'Noch keine Cutscene-Mods installiert';

  @override
  String get cutsceneHowItWorks1 =>
      'Benutzerdefinierte Cutscenes werden aus nams/cutscenes/ statt aus data/movie/ geladen.';

  @override
  String get cutsceneHowItWorks2 =>
      'Fehlt eine benutzerdefinierte Datei oder ist sie beschädigt, wird ersatzweise das Original abgespielt.';

  @override
  String get cutsceneHowItWorks3 =>
      'Deine ursprünglichen Spieldateien werden nie verändert – Mods werden von einem separaten Speicherort geladen.';

  @override
  String get cutsceneStructurePath =>
      'nams/cutscenes/<mod_name>/movie/<filename>.usm';

  @override
  String get cutsceneFolderNameLimit =>
      'Ordnernamen dürfen höchstens 27 Zeichen lang sein.';

  @override
  String get cutsceneMigrationTitle =>
      'Benutzerdefinierte Cutscene-Dateien in data/movie/ erkannt';

  @override
  String cutsceneMigrationBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Cutscene-Dateien',
      one: 'Cutscene-Datei',
    );
    return '$count benutzerdefinierte $_temp0 wurden direkt in data/movie/ gefunden. Diese überschreiben die Originale dauerhaft. Installiere Cutscene-Mods künftig stattdessen hier – falls eine benutzerdefinierte Datei nicht geladen werden kann, wird ersatzweise das Original abgespielt.';
  }

  @override
  String get noMovieFolderFound =>
      'Kein movie/-Ordner mit .usm-Dateien gefunden.';

  @override
  String get noUsmFilesFound =>
      'In der Mod wurden keine .usm-Dateien gefunden.';

  @override
  String get onboardingWelcomeTitle => 'Willkommen bei YoRHa Protocol';

  @override
  String get onboardingWelcomeSubtitle =>
      'Ein Launcher. Alle deine Mods. Neue NieR-Inhalte.\n\nMods per Drag-and-drop, Outfit-Wechsel mitten im Spiel, neue Quests und ein integriertes Devkit – ohne Konfigurationsalbtraum.';

  @override
  String get onboardingSelectTitle =>
      'Wähle deine NieR:Automata-Installation aus';

  @override
  String get onboardingSearchingDrives =>
      'Alle Laufwerke werden durchsucht ...';

  @override
  String get onboardingSearchingNier => 'NieR:Automata wird gesucht ...';

  @override
  String get onboardingSelectInstallation => 'Installation auswählen';

  @override
  String get onboardingFirstPlaythroughTitle =>
      'Ist dies dein erster Spieldurchlauf?';

  @override
  String get onboardingFirstPlaythroughHint =>
      'Falls ja, blenden wir Spoiler aus.';

  @override
  String get onboardingFirstYes => 'Ja – Spoilerfunktionen ausblenden';

  @override
  String get onboardingFirstNo => 'Nein – alles anzeigen';

  @override
  String get onboardingMigrationAskTitle => 'NieR schon einmal gemoddet?';

  @override
  String get onboardingMigrationAskBody =>
      'Wir übernehmen deine alte Einrichtung automatisch.';

  @override
  String get onboardingMigrationYes => 'Ja';

  @override
  String get onboardingMigrationNo => 'Nein';

  @override
  String get onboardingMigrationResultsTitle => 'Was wir gefunden haben';

  @override
  String get onboardingMigrationResultsBody => '';

  @override
  String get onboardingMigrationNothingFound =>
      'Nichts erkannt. Deine Installation ist sauber.';

  @override
  String get onboardingMigrationActionReshadeKept =>
      'Deaktiviert – NAMS besitzt native Tiefenschärfe. Bei Bedarf kannst du es in der NAMS-Konfiguration wieder aktivieren.';

  @override
  String get onboardingMigrationActionReshadeIncompatible =>
      'Add-on-/ImGui-Version – ersetzen oder entfernen.';

  @override
  String get onboardingMigrationActionTextures => 'Wird automatisch geladen.';

  @override
  String get onboardingMigrationActionLodMod => 'LodMod.ini wurde importiert.';

  @override
  String get onboardingMigrationActionSkRes => 'Automatisch übernommen.';

  @override
  String get onboardingMigrationActionNaiom =>
      'Deine NAIOM-Einstellungen werden automatisch in NAMS importiert. Anschließend kannst du die alten NAIOM-Dateien löschen (dinput8.dll, NAIOM.ini).';

  @override
  String get onboardingMigrationActionCutscenes => 'Bereits integriert.';

  @override
  String get onboardingMigrationActionExistingMods =>
      'Bereits installiert – unverändert beibehalten.';

  @override
  String onboardingMigrationLabelExistingMods(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Mods in nams/mods/',
      one: '1 Mod in nams/mods/',
    );
    return '$_temp0';
  }

  @override
  String get onboardingModInstallAskTitle =>
      'Vor dem Start eine Mod installieren?';

  @override
  String get onboardingModInstallAskBody =>
      'Du hast eine .zip-Datei oder einen Ordner? Wir kümmern uns darum.\n(Texturen und Cutscenes besitzen eigene Reiter – installiere sie später dort.)';

  @override
  String get onboardingModInstallYes => 'Ja, jetzt eine Mod installieren';

  @override
  String get onboardingModInstallSkip => 'Nein, vorerst überspringen';

  @override
  String get onboardingModInstallTitle => 'Füge deine erste Mod hinzu';

  @override
  String get onboardingModInstallBody => 'Unterstützt .zip, .7z und Ordner.';

  @override
  String get onboardingModInstallSubBody => 'Die Spieldateien bleiben sicher.';

  @override
  String get onboardingModInstallBusy => 'Wird installiert …';

  @override
  String get onboardingModInstallInspecting => 'Mod wird geprüft …';

  @override
  String onboardingModInstallExtractingPercent(int percent) {
    return 'Wird entpackt … $percent%';
  }

  @override
  String onboardingModInstallExtractingFile(int percent, String file) {
    return 'Wird entpackt: $percent% – $file';
  }

  @override
  String get onboardingModInstallNotAMod =>
      'Das sieht nicht wie eine gültige NAMS-Mod aus. Stelle sicher, dass es sich um eine .zip-/.7z-Datei mit einer mod.toml oder einer erkannten Datenstruktur handelt.';

  @override
  String get onboardingOutfitHintHeader => 'So trägst du es';

  @override
  String get onboardingOutfitHintCompat =>
      'Kaufe es oder rüste es aus deinem Inventar aus.';

  @override
  String get onboardingOutfitHintData =>
      'Im Spiel F1 → Garderobensymbol (oben links).';

  @override
  String get onboardingOutfitHintMultiOutfit =>
      'Oder verwende den Multi-Outfit-NPC im Widerstandslager.';

  @override
  String get onboardingOutfitHintMultiOutfitLink =>
      'Multi-Outfit-NPC herunterladen';

  @override
  String get onboardingOutfitHintMultiOutfitUrl =>
      'https://www.nexusmods.com/nierautomata/mods/789';

  @override
  String onboardingModInstallFailed(String reason) {
    return 'Installation fehlgeschlagen: $reason';
  }

  @override
  String get onboardingModInstallNoGameDir =>
      'Es wurde noch kein Spielverzeichnis ausgewählt. Gehe zurück und wähle zuerst deine Installation aus.';

  @override
  String get onboardingModInstallInstalledHeader => 'Installiert:';

  @override
  String get onboardingModInstallSkipButton => 'Überspringen';

  @override
  String get onboardingModInstallDoneButton => 'Fertig';

  @override
  String get onboardingModInstallPickerTitle =>
      'Eine Mod (.zip, .7z) auswählen oder einen Ordner öffnen';

  @override
  String get onboardingModInstallFolderPickerTitle => 'Mod-Ordner auswählen';

  @override
  String get onboardingReadyTitle => 'Alles ist eingerichtet!';

  @override
  String get onboardingCreateShortcut => 'Desktopverknüpfung erstellen';

  @override
  String get onboardingFirstPlaythroughSpoilerFree =>
      'Erster Spieldurchlauf (spoilerfrei)';

  @override
  String get onboardingFullAccess => 'Vollständiger Zugriff';

  @override
  String get detectionReShade => 'ReShade';

  @override
  String get detectionHDTextures => 'HD-Texturen';

  @override
  String get detectionLodMod => 'LOD Mod (Automata-LodMod)';

  @override
  String get detectionSkRes => 'Special-K-Texturen (SK_Res)';

  @override
  String get detectionNaiom => 'NAIOM';

  @override
  String get detectionCutscenes => 'Cutscene-Mods (nams/cutscenes)';

  @override
  String get detectionCustomMovie =>
      'Benutzerdefinierte Cutscenes in data/movie/';

  @override
  String get detectionDetected => 'Erkannt';

  @override
  String get detectionReShadeIncompatible => 'Nicht kompatibel (Add-on/ImGui)';

  @override
  String get detectionNotFound => 'Nicht gefunden';

  @override
  String get detectionNoneFound => 'Nichts gefunden';

  @override
  String get detectionLodModMigrated => 'Gefunden – nach NAMS migriert';

  @override
  String get detectionSkResAuto => 'Gefunden – wird automatisch geladen';

  @override
  String get detectionNaiomPending =>
      'Gefunden – Einstellungen werden automatisch importiert';

  @override
  String get detectionNoneInstalled => 'Nichts installiert';

  @override
  String get detectionCustomMovieHint =>
      'Gefunden – erwäge für einen sicheren Rückgriff stattdessen nams/cutscenes/ zu verwenden';

  @override
  String get detectionInstalled => 'Installiert';

  @override
  String get detectionCustomFilesDetected =>
      'Benutzerdefinierte Dateien erkannt';

  @override
  String get detectionMigratedIntoNams => 'Nach NAMS migriert';

  @override
  String get detectionLoadedAutomatically => 'Automatisch geladen';

  @override
  String get detectionMigrationComingSoon =>
      'Einstellungen automatisch importiert';

  @override
  String get detectionNotSet => 'Nicht festgelegt';

  @override
  String get labelGame => 'Spiel';

  @override
  String get labelMode => 'Modus';

  @override
  String get labelDlc => 'DLC';

  @override
  String get labelNoDlc => 'Kein DLC';

  @override
  String get searchingForNier => 'NieR:Automata wird gesucht ...';

  @override
  String get autoButton => 'AUTO';

  @override
  String get nierFound => 'NieR:Automata gefunden';

  @override
  String get selectInstallation => 'Wähle deine Installation aus:';

  @override
  String get notYourGame => 'Nicht dein Spiel?';

  @override
  String get searchAllDrives => 'Alle Laufwerke durchsuchen';

  @override
  String get selectManually => 'Manuell auswählen';

  @override
  String get notFoundTitle => 'Nicht gefunden';

  @override
  String get notFoundMessage =>
      'NieR:Automata konnte nicht über Steam gefunden werden. Möchtest du alle Laufwerke durchsuchen? Dies kann bis zu 30 Sekunden dauern.';

  @override
  String get scanDrives => 'Laufwerke durchsuchen';

  @override
  String get confirmInstallation => 'Ist dies die richtige Installation?';

  @override
  String get cancelButton => 'Abbrechen';

  @override
  String get noSelectManually => 'Nein, manuell auswählen';

  @override
  String get yesButton => 'Ja';

  @override
  String get installationsFoundTitle => 'NieR:Automata-Installationen gefunden';

  @override
  String get validInstallations => 'Gültige Installationen (mit data-Ordner):';

  @override
  String get withoutDataFolder => 'Ohne data-Ordner:';

  @override
  String get noLogEntries => 'Keine Protokolleinträge gefunden';

  @override
  String get noLogMatches => 'Keine Protokolleinträge entsprechen deiner Suche';

  @override
  String get logViewerTitle => 'PROTOKOLLANZEIGE';

  @override
  String get logSearchPlaceholder =>
      'Ebene / Modul / Nachricht durchsuchen ...';

  @override
  String get logLiveBadge => 'LIVE';

  @override
  String get logAutoScroll => 'Automatisch zum neuesten Eintrag scrollen';

  @override
  String get entriesSuffix => 'Einträge';

  @override
  String get scrollToBottom => 'Nach unten scrollen';

  @override
  String get openLogsFolder => 'Protokollordner öffnen';

  @override
  String get diagnosticsButton => 'Diagnose erstellen';

  @override
  String get diagnosticsSubtitle =>
      'Erstellt einen vollständigen Bericht deiner Installation (Spieldateien, Mods, ReShade/3DMigoto, Einstellungen), den du bei Hilfeanfragen teilen kannst.';

  @override
  String get copyCommandTitle => 'Manueller Startbefehl';

  @override
  String get copyCommandDesc =>
      'Wenn das Spiel nicht über den Launcher startet, kopiere den NAMS-Befehl und führe ihn im Terminal aus. NAMS gibt dort den Grund für den Fehler aus, das ist der schnellste Weg zur Ursache.';

  @override
  String get copyCommandButton => 'Befehl kopieren';

  @override
  String get diagnosticsTitle => 'Diagnosezusammenfassung';

  @override
  String get diagnosticsCollecting => 'Diagnosedaten werden gesammelt ...';

  @override
  String get diagnosticsCopy => 'Zusammenfassung kopieren';

  @override
  String get diagnosticsCopied =>
      'Zusammenfassung in die Zwischenablage kopiert';

  @override
  String get diagnosticsSaveFull => 'Vollständigen Bericht speichern';

  @override
  String diagnosticsSavedAt(String path) {
    return 'Vollständiger Bericht unter $path gespeichert';
  }

  @override
  String get modsTutorialBack => 'Zurück';

  @override
  String get modsTutorialNext => 'Weiter';

  @override
  String get modsTutorialFinish => 'Verstanden';

  @override
  String get modsTutorialSkip => 'Überspringen';

  @override
  String get modsTutorialMenuEcosystem => 'NAMS und der Launcher';

  @override
  String get modsTutorialMenuInstall => 'Eine Mod installieren';

  @override
  String get modsTutorialMenuProfiles => 'So funktionieren Profile';

  @override
  String get modsTutorialMenuSupporting => 'Das Ökosystem unterstützen';

  @override
  String get modsTutorialSupportingStep1Title =>
      'Ein Ökosystem, kein Mod-Manager';

  @override
  String get modsTutorialSupportingStep1Body =>
      '**NAMS ist seit über drei Jahren in Entwicklung.** Was als einzelner Modloader begann, ist zu einer ganzen Plattform gewachsen, auf der heute **Mods**, **Plugins** und Werkzeuge aufbauen:\n\n- **Mods** sind Inhaltserweiterungen – Outfits, benutzerdefinierte Quests, neue Waffen und Texturen. Sie nutzen das Inhaltssystem von NAMS, führen aber keinen eigenen Code aus.\n- **Plugins** sind Programme, die parallel zum Spiel laufen und den Modloader selbst erweitern können – etwa die Multiplayer-Mod, Debug-Overlays oder neue Spielsysteme. Sie werden als Code geschrieben und beim Start von NAMS geladen.\n- Der Launcher, den du gerade verwendest, ist **kein Plugin** – er ist eine eigenständige Anwendung, die deine Mods und Konfigurationen vor dem Spielstart vorbereitet.\n\nDie Mods, die du heute siehst, laufen nicht trotz eines Hindernisparcours – sie bauen auf jahrelanger Grundlagenarbeit auf, die gerade deshalb existiert, damit du sie nicht erneut leisten musst.\n\nIn diesem Abschnitt geht es darum, **wie diese Plattform am Leben bleibt** und was das für dich bedeutet – egal, ob du nur spielst, erste Schritte beim Modding machst oder etwas beitragen möchtest.\n\nDie nächsten Seiten beginnen mit den Punkten, die **alle** betreffen, und werden zum Ende hin technischer. Du kannst jederzeit vorspringen, wenn du genug gelesen hast – nichts davon ist Pflichtlektüre.';

  @override
  String get modsTutorialSupportingStep2Title => 'Inhalte ohne Programmierung';

  @override
  String get modsTutorialSupportingStep2Body =>
      '**Du musst nicht programmieren können, um zu diesem Ökosystem beizutragen.**\n\nNAMS besitzt ein Inhaltssystem, das bereits deklarative Inhaltserweiterungen unterstützt und laufend um weitere Möglichkeiten ergänzt wird:\n\n- **Benutzerdefinierte Quests**, die zusätzlich zur ursprünglichen Geschichte eingebunden werden.\n- **Neue Waffen und Gegenstände** mit eigenem Verhalten, ohne vorhandene Inhalte zu überschreiben.\n- **Accessoires** in **neuen Plätzen**, statt bestehende zu ersetzen.\n- **Benutzerdefinierte Cutscenes**, die in `nams/cutscenes/` liegen oder in einer Mod gebündelt sind (zum Beispiel eine benutzerdefinierte Quest mit eigenen Filmsequenzen) – die ursprünglichen Cutscenes werden **niemals ersetzt**, neue werden einfach zusätzlich geladen.\n\nDas Grundprinzip lautet: **ergänzen statt ersetzen.** Die ursprünglichen Inhalte bleiben erhalten; Mod-Inhalte werden darübergelegt. Dadurch können Modellierer ohne Programmiererfahrung Accessoires, Waffen und Charaktere erstellen und als Ergänzungen veröffentlichen – ohne etwas zu überschreiben, Spielstände zu beschädigen oder Konflikte mit anderen Mods zu verursachen, die dasselbe Prinzip nutzen.\n\nDiese Möglichkeiten werden sich mit der Zeit **erweitern**. Jede Veröffentlichung fügt weitere deklarative Schnittstellen hinzu, damit Nichtprogrammierer noch mehr umsetzen können.';

  @override
  String get modsTutorialSupportingStep3Title =>
      'Möglichkeiten beizutragen – nicht nur mit Code';

  @override
  String get modsTutorialSupportingStep3Body =>
      'Inhalte zu erstellen (wie auf der vorherigen Seite beschrieben) ist eine Möglichkeit, etwas zurückzugeben. Es ist bei Weitem nicht die einzige – und viele Dinge, die ein Ökosystem gesund halten, haben überhaupt nichts mit dem Erstellen von Mods zu tun. Folgendes macht tatsächlich einen Unterschied:\n\n- **Schreibe eine Anleitung.** „Wie ich X mit NAMS gebaut habe“, „wie ich Y debuggt habe“, „die fünf Dinge, die ich gern vorher gewusst hätte“. Die meisten Lücken beim Einstieg sind derzeit Dokumentationslücken.\n- **Melde einen echten Fehler – und zwar gut.** Eine reproduzierbare Fehlerbeschreibung, ein Protokoll und der Diagnosebericht aus dem Protokollbereich sind mehr wert als zehn Tickets mit „es funktioniert nicht“.\n- **Modelliere.** Accessoires, Waffen, Charaktere und Requisiten. Das Inhaltssystem von NAMS lädt sie als **Ergänzungen** – neue Plätze, keine Überschreibungen. Dadurch können Modellierer ohne Programmiererfahrung Werke veröffentlichen, die sich konfliktfrei in die Zusammenstellungen anderer Spieler einfügen.\n- **Übersetze.** Der Launcher ist lokalisiert. Falls deine Sprache noch fehlt und du sie verwenden würdest, findest du die Texte unter `lib/l10n/`; Pull Requests sind willkommen.\n- **Teste auf ungewöhnlicher Hardware.** Steam Deck, AMD-GPUs, Ultrawide-Konfigurationen, mehrere Monitore oder Controller, die kaum jemand besitzt. Fehler, die nur bei seltenen Konfigurationen auftreten, bleiben verborgen, bis sie jemand meldet.\n- **Beantworte einfach Fragen auf Discord.** Der nächsten neuen Person zu helfen, ist ebenfalls ein Beitrag. Was in einem Ökosystem langfristig überlebt, ist zu einem großen Teil die Kultur der Menschen, die früh dabei waren.\n- **Analysiere eine Spielfunktion per Reverse Engineering und stelle die API bereit.** *(Mehr dazu auf der nächsten Seite, falls du neugierig bist.)* Für technisch Interessierte ist dies der Beitrag mit der größten Hebelwirkung.\n\n### Abschließender Gedanke (vorerst)\n\nViel Freizeit, persönlicher Einsatz und hartnäckiger Fleiß sind in die Entstehung dieser Plattform geflossen – immer mit dem Ziel, **anderen Menschen den Einstieg in etwas Eigenes zu ermöglichen**. Wenn einer dieser Punkte für dich machbar klingt, hast du den größten Teil bereits geschafft. Auf dem Discord (**YoRHa Continuum**) kannst du Fragen stellen.\n\nDie nächsten beiden Seiten werden technischer. Lies weiter, wenn du wissen möchtest, wie Plugins nebeneinander bestehen und wie neue Spiel-APIs in NAMS gelangen, oder höre hier auf, wenn du genug erfahren hast.';

  @override
  String get modsTutorialSupportingStep4Title =>
      'Plugins bestehen bewusst nebeneinander';

  @override
  String get modsTutorialSupportingStep4Body =>
      '*Die nächsten beiden Seiten sind technischer und richten sich an Personen, die über die Entwicklung von Plugins nachdenken. Überspringe sie, wenn das nicht dein Thema ist.*\n\nEin entscheidendes Merkmal der NAMS-Plattform ist, dass **mehrere Plugins gleichzeitig** in derselben Spielsitzung laufen können, ohne sich gegenseitig zu behindern.\n\nDie **Multiplayer Mod von Jasper** ist eine der größten Entwicklungen im NAMS-Ökosystem und wird weiterhin aktiv gepflegt – großen Respekt für diese Arbeit. Das YP-Devkit und das MP-Plugin werden **gleichzeitig geladen**, beide funktionieren und beide zeichnen ihre eigene Benutzeroberfläche über das Spiel. Das ist kein glücklicher Zufall – genau dafür wurde der Plugin-Host von NAMS entwickelt.\n\nWenn du also ein Plugin veröffentlichst, das die Vorgaben der Plattform einhält, **kann es mit allem anderen bereits Laufenden koexistieren** – deinem Plugin, dem MP-Plugin, dem YP-Devkit und zukünftigen Plugins von Menschen, die du nie kennenlernen wirst. Du musst nicht um Hooks konkurrieren oder gegen die Ladereihenfolge kämpfen; die Plattform vermittelt.\n\nJeden Monat finden weiterhin Umstrukturierungen statt, um Grenzfälle zu reduzieren, in denen ein Plugin versehentlich ein anderes beeinträchtigen kann. Das Ziel bewegt sich ständig – aber die Richtung ist klar und die Arbeit geht weiter.';

  @override
  String get modsTutorialSupportingStep5Title =>
      'Du baust auf frei verfügbarem Reverse Engineering auf';

  @override
  String get modsTutorialSupportingStep5Body =>
      'Die meisten Engine-APIs, die du für ein ernsthaftes Plugin benötigen würdest, existieren bereits in NAMS – und **sie existieren, weil jemand das Spiel per Reverse Engineering analysiert hat**, um ein eigenes Problem zu lösen, und das Ergebnis anschließend beigetragen hat.\n\nDas Spiel ist Closed Source. Jede API in NAMS, mit der sich ein Spielzustand lesen oder verändern lässt, ist das Ergebnis davon, dass jemand Funktionen nachverfolgt, Offsets gefunden und das Verhalten überprüft hat. Darin steckt viel unbezahlte Arbeit, und sie befindet sich gezielt in NAMS, damit der **nächste** Plugin-Entwickler sie nicht wiederholen muss.\n\nWenn du auf NAMS aufbaust, erbst du all das. Du beginnst nicht bei `LoadLibrary`, sondern bei APIs, die jemand bereits mühsam geschaffen hat – und die nächste Person, die eine von dir analysierte API benötigt, erhält dasselbe Geschenk.\n\n### Warum dies der Beitrag mit der größten Hebelwirkung ist\n\nWenn du das auch nur ein einziges Mal tust, ersparst du jedem zukünftigen Plugin-Entwickler dauerhaft dieselbe Arbeit. Darin liegt die Asymmetrie. Eine Anleitung hilft einigen Menschen, die sie lesen; eine API in NAMS hilft für immer allen, die diese Funktion jemals benötigen werden. Das Ökosystem wächst durch Menschen, die eine Sache für sich selbst per Reverse Engineering untersuchen und das Ergebnis anschließend allen hinterlassen.';

  @override
  String get modsTutorialEcosystemStep1Title => 'Warum all das existiert';

  @override
  String get modsTutorialEcosystemStep1Body =>
      'Das Modding von NieR war historisch mühsam. Mods, die einzeln problemlos funktionieren, beginnen sich gegenseitig zu stören, sobald mehrere davon kombiniert werden – verschiedene DLL-Hooks (`dxgi`, `d3d11`, `dinput8`) konkurrieren um denselben Platz, der falsche Wrapper gewinnt die Ladereihenfolge und das Spiel stürzt beim Start kommentarlos ab. Menschen mit 5–10 Mods verbringen mehr Zeit mit Fehlereingrenzung als mit Spielen.\n\nLange Zeit lautete die einzige Antwort: *nur manuell installieren*. Lose `.dat`-/`.dtt`-Dateien in `data/` ablegen, Konfigurationen von Hand bearbeiten und niemals einen Mod-Manager verwenden. Für eine oder zwei Mods funktioniert das, doch dabei werden echte Spieldateien überschrieben und es bleibt kein nachvollziehbarer Verlauf der Änderungen. Werkzeuge wie Vortex halfen nicht – sie verstehen die Besonderheiten von NieR nicht.\n\n**NAMS wurde entwickelt, um dieses Problem auf Ebene des Modloaders zu lösen**, und **dieser Launcher gibt NAMS eine benutzerfreundliche Oberfläche.**';

  @override
  String get modsTutorialEcosystemStep2Title => 'Was NAMS macht';

  @override
  String get modsTutorialEcosystemStep2Body =>
      '**NAMS ist der Modloader.** Es ist keine Proxy-DLL, die wie ältere Werkzeuge `dxgi.dll` oder `d3d11.dll` übernimmt – genau dieser Mechanismus hat die Konflikte überhaupt erst verursacht.\n\nStattdessen läuft NAMS als eigener Hostprozess: Es lädt NieR:Automata innerhalb dieses Prozesses als Bibliothek (die EXE des Spiels wird in eine ladbare `game.bin` umgewandelt) und initialisiert den Modloader, bevor das Spiel startet. In keinen anderen Prozess wird etwas injiziert – NAMS *ist* der Prozess, in dem das Spiel läuft, und besitzt vollständige Kontrolle darüber, was gleich geladen wird.\n\nVon dort aus übernimmt NAMS zwei große Aufgaben:\n\n**1. Es implementiert Funktionen, die früher andere Werkzeuge bereitstellten, neu** – LodMod, Limit Break, Texturinjektion und schnelles Laden – nativ und in einer koordinierten Schicht. Mods verwenden NAMS-APIs, statt darum zu kämpfen, welcher DLL-Hook zuerst aufgerufen wird.\n\n**2. Es stellt ein virtuelles Dateisystem (VFS) bereit:**\n\n- Jede Mod befindet sich in einem eigenen Ordner unter `nams/mods/<modId>/` – echte Spieldateien werden niemals überschrieben.\n- Zur Laufzeit legt NAMS aktive Mods über die Sicht der Engine auf `data/`.\n- Deine ursprünglichen Dateien `data/*.cpk` und `NieRAutomata.exe` werden **niemals verändert**, sodass der ungemoddete Start über Steam weiterhin genau wie zuvor funktioniert.\n\nMods erklären in einem Manifest, was sie verändern. NAMS prüft und lädt sie in einer festgelegten Reihenfolge. Dadurch erhältst du endlich ein **sauberes Aktivieren und Deaktivieren einzelner Mods** sowie eine **nachvollziehbare Konflikterkennung** – beides war mit dem alten Ansatz, Dateien direkt in `data` abzulegen, nicht möglich.\n\n### So greift alles ineinander\n\nDieser Launcher baut **nicht** direkt auf NieR:Automata auf. Er analysiert das Spiel nicht per Reverse Engineering, hängt sich nicht in Engine-Funktionen ein und kennt die Formate `.dat`/`.dtt` nicht von selbst. Die Reihenfolge lautet:\n\n1. **NieR:Automata** – das unveränderte Spiel.\n2. **NAMS** – der Modloader, der zuerst entwickelt wurde, um skalierbares Modding überhaupt zu ermöglichen (Neuimplementierung von Engine-Funktionen, VFS, Plugin-Host und Inhaltsframework).\n3. **Dieser Launcher** – als Hilfsanwendung auf NAMS aufgebaut. Er liest die TOML-Konfigurationen von NAMS, schreibt in dessen Ordnerstruktur und kommuniziert mit dessen APIs. Mehr nicht.\n\nDie praktische Folge: NAMS ist die tragende Schicht. Der Launcher ist lediglich eine benutzerfreundliche Oberfläche davor und könnte durch eine andere Oberfläche oder die Kommandozeile ersetzt werden, ohne dass deine Mods davon betroffen wären.\n\n### Und genau das ist bereits geschehen\n\nDas ist keine Theorie. **Der NAO Launcher von Rustukun** ist ein separater Launcher auf derselben Grundlage – andere Oberfläche, andere Designentscheidungen, aber dasselbe NAMS darunter. Deine Mods, deine Ordner `nams/mods/<modId>/` und deine `disabled_mods.toml` funktionieren unabhängig vom verwendeten Launcher identisch.\n\nDas beweist, dass NAMS die Plattform ist und jeder Launcher – dieser, NAO oder ein zukünftiger, den noch niemand geschrieben hat – nur eine Frontend-Entscheidung darstellt. Wähle den Launcher, der zu deinem Arbeitsablauf passt; deine Mod-Bibliothek muss nicht umziehen.';

  @override
  String get modsTutorialEcosystemStep3Title =>
      'Was dieser Launcher ergänzt – und was anders ist';

  @override
  String get modsTutorialEcosystemStep3Body =>
      'NAMS übernimmt das Laden. Der Launcher kümmert sich um **alles drumherum** – Installation, Organisation und Fehlerbehebung:\n\n- **Mod-Manager** – Drag-and-drop-Installation von Mods im NAMS-Format, automatische Normalisierung der Ordnerstruktur (wax-/SK_Res-Wrapper, gebündelte Inhalte), Manifestprüfung und Konfliktwarnungen.\n- **Texturen** – eigenständige Texturpakete und die Priorität von `load_order` verwalten, ohne TOML-Dateien von Hand zu bearbeiten.\n- **Cutscenes** – HD-Cutscene-Mods installieren, den Codec automatisch erkennen (H264 oder MPEG-2) und die richtigen NAMS-Einstellungen setzen.\n- **Profile** – mehrere Mod-Zusammenstellungen nebeneinander aufbewahren und mit einem Klick wechseln, ohne Zustände zu kopieren oder zu verlieren.\n- **Diagnose** – vollständiger Bericht darüber, was wo installiert ist, welche Reste alter Installationen vorhanden sind und was NAMS erkennt im Vergleich zu dem, was tatsächlich auf dem Datenträger liegt.\n\n### Warum wir das entwickelt haben\n\n**Nichts gegen manuelle Installationen.** Die `.dat`-/`.dtt`-Dateien eines Outfits in den richtigen Unterordner von `data/` zu legen, funktioniert für eine oder zwei Mods problemlos. Der Launcher ist für den darüber hinausgehenden Umfang gebaut.\n\nDas NAMS-Ökosystem unterstützt inzwischen unter anderem:\n\n- **Mehr als 30 Outfit-Mods** mit mehreren wechselbaren Outfits pro Charakter.\n- **Mehr als 20 benutzerdefinierte Quests**, die zur ursprünglichen Geschichte hinzugefügt werden.\n- **Mehr als 10 neue Waffen** mit eigenem Verhalten.\n- Außerdem Texturen, Cutscenes, Plugins, Balanceänderungen und mehr.\n\nAll das von Hand zu verwalten, ist keine Frage philosophischer Vorlieben – es ist **schlicht nicht möglich**. Du kannst nicht mehr nachvollziehen, welche Datei aus welcher Mod stammt, eine einzelne Mod nicht sauber aktivieren oder deaktivieren und nicht erkennen, warum etwas kaputtging. Bei diesem Umfang stößt manuelles Modding an eine harte Grenze – und das NAMS-Ökosystem hat diese Grenze längst überschritten.\n\n### Unterschiede zu NAMH und Vortex\n\nWenn du schon länger in der NieR-Szene unterwegs bist, erinnerst du dich vielleicht daran, wie frühere Mod-Manager endeten:\n\n- **NAMH** (NieR Automata Mod Helper) wurde nicht mehr gepflegt, konnte Spiele irreparabel beschädigen, litt unter Sperrproblemen wie „Programm wird verwendet“, und die Standardlösung lautete: *Spiel deinstallieren, neu installieren und alles manuell machen.* NAMH funktionierte durch das **direkte Ersetzen von Dateien in `data/`**. Wenn eine NAMH-Installation schiefging, gab es keinen sauberen Weg zurück.\n- **Vortex** verstand die Dateistruktur von NieR nie richtig. Sein virtuelles Dateisystem trifft Annahmen, die nicht zur tatsächlichen Ladeweise des Spiels passen, wodurch Installationen Dateien unbemerkt an falschen Orten ablegten.\n\nDieser Launcher ist anders aufgebaut. Die entscheidenden Prinzipien:\n\n1. **Niemals Dateien ersetzen.** Mods liegen in `nams/mods/<modId>/` und werden zur Laufzeit über das VFS von NAMS in die Sicht der Engine eingeblendet. Das ursprüngliche `data/` wird niemals verändert. Einen „nicht wiederherstellbaren Zustand“ gibt es nicht, weil sich am echten Spiel nichts geändert hat.\n2. **Jede Aktion ist umkehrbar.** Eine Mod deinstallieren → ihr Ordner und ihre gebündelten Inhalte werden sauber entfernt. Eine Mod deaktivieren → ein Eintrag in `disabled_mods.toml`, und NAMS überspringt sie. Kein versteckter Zustand, keine unumkehrbaren Schreibvorgänge.\n3. **Profile statt eines einzigen globalen Zustands.** Frühere Manager legten alles in einer einzigen Konfiguration fest. Mit Profilen kannst du mehrere Zusammenstellungen nebeneinander aufbewahren und atomar wechseln – nichts kann vermischt werden oder verloren gehen.\n4. **Auf einem gepflegten Modloader aufgebaut.** Das Ende von NAMH hing auch mit der unsicheren Zukunft des Modloaders zusammen. NAMS ist die Grundlage für alles hier, und der Launcher folgt seinen Aktualisierungen.\n\nSollte dieser Launcher irgendwann nicht mehr gepflegt werden, bleiben deine Mods dennoch einfache Dateien auf dem Datenträger, die NAMS lesen kann – du wirst nicht aus deiner eigenen Installation ausgesperrt.';

  @override
  String get modsTutorialEcosystemStep4Title => 'Wie es weitergeht';

  @override
  String get modsTutorialEcosystemStep4Body =>
      'Falls du hier noch nie eine Mod installiert hast:\n\n- **Eine Mod installieren** – führt dich Reiter für Reiter durch den Installationsablauf.\n- **So funktionieren Profile** – erklärt getrennte Zusammenstellungen und wann sie sinnvoll sind.\n\nBeide Anleitungen findest du in diesem Hilfemenü (über das **?**-Symbol, mit dem du es geöffnet hast).\n\n**Kurz zusammengefasst:** Lege Archive im passenden Reiter ab (Mod-Manager für Charakter-/Daten-Mods, Texturen für eigenständige Texturpakete, Cutscenes für HD-Cutscenes), drücke auf Spielen und sieh bei Problemen in die Protokolle. Der Launcher versucht automatisch die richtige Entscheidung zu treffen – falls du mit einer Entscheidung nicht einverstanden bist, lässt sich jede Aktion über die Benutzeroberfläche rückgängig machen.';

  @override
  String get modsTutorialHelpTooltip => 'Anleitungen und Hilfe';

  @override
  String get modsTutorialInstallStep1Title => 'Lege deine Mod hier ab';

  @override
  String get modsTutorialInstallStep1Body =>
      'Dies ist der Reiter **Mod-Manager** – hier werden Charakter-, Outfit- und andere Gameplay-Mods installiert.\n\nZiehe eine `.zip`-, `.7z`- oder `.rar`-Datei von Nexus auf diesen Ablagebereich oder klicke zum Durchsuchen. Der Launcher entpackt sie, prüft die Struktur und legt sie am richtigen Ort ab. Du musst selbst nichts entpacken.\n\n**Gut zu wissen:** Deine ursprünglichen Spieldateien bleiben unverändert. Mods liegen in einem separaten Ordner `nams/`, sodass du das ungemoddete Spiel jederzeit über Steam starten kannst.';

  @override
  String get modsTutorialInstallStep2Title =>
      'Du möchtest eine WAX-Mod installieren? Lies zuerst dies';

  @override
  String get modsTutorialInstallStep2Body =>
      '**WAX-Mods funktionieren hier.** NAMS implementiert WAX bis zu einer getesteten Kompatibilitätsversion neu. Die meisten Mods auf Nexus, die auf diese oder eine ältere Version abzielen, lassen sich normal installieren und ausführen.\n\n### So funktioniert die Kompatibilität\n\nNAMS wird gegen eine bestimmte WAX-Version geprüft. Alles, was WAX bis einschließlich dieser Version bereitgestellt hat, wird unterstützt. Alles, was WAX danach in einer **neueren** Version ergänzt, wird nicht automatisch unterstützt – es handelt sich um eine neue Funktion auf WAX-Seite, die auf NAMS-Seite von Grund auf neu implementiert werden muss.\n\n### Was geschieht, wenn WAX etwas Neues ergänzt?\n\nVeröffentlicht WAX in einer zukünftigen Version eine neue Funktion, wird sie anhand der NAMS-Roadmap bewertet:\n\n- **Im Umfang enthalten** – passt die Funktion zu der Richtung, in die sich NAMS ohnehin entwickelt, wird sie umgesetzt und eine zukünftige NAMS-Version unterstützt Mods, die sie verwenden.\n- **Nicht im Umfang enthalten** – NAMS konzentriert sich auf eigene Inhaltserweiterungen (benutzerdefinierte Quests, eigene Weltkarten, benutzerdefinierte Plugin-Chips, erweiterte Modding-APIs und mehr). Jede WAX-Funktion neu zu implementieren, hat keine Priorität. Einige spezielle WAX-Funktionen erhalten möglicherweise nie ein NAMS-Gegenstück.\n\n**Das ist keine Abwertung von WAX.** Es handelt sich um getrennte Projekte mit unterschiedlichen Zielen. NAMS versucht nicht, WAX vollständig zu ersetzen – es ist eine eigene Plattform, die zufällig **weitgehend kompatibel** mit WAX ist, weil die meisten Benutzer ihre vorhandenen Mod-Bibliotheken weiterverwenden möchten.\n\n### Dieses Muster ist normal\n\nEine solche Aufspaltung ist **die normale Entwicklung jedes Modding-Ökosystems** und keine Besonderheit von NieR. Ein konkretes Beispiel: **Skyrim Legendary Edition (LE)** und **Skyrim Special Edition (SE)** sind Abzweigungen derselben Engine. SE ist weitgehend, aber nicht vollständig mit LE-Mods kompatibel. Einige LE-Mods müssen konvertiert werden, andere wurden nie portiert, weil sie sich auf Engine-Eigenheiten stützten, die SE verändert hat. Die Skyrim-Community betrachtete das nicht als Fehler; es wurde einfach Teil der Funktionsweise des Ökosystems. Dasselbe gilt für **OpenMW gegenüber dem ursprünglichen Morrowind**, **Java gegenüber Bedrock Minecraft**, **KSP1- gegenüber KSP2-Mods** und andere Beispiele.\n\nWann immer eine Plattform das Verhalten einer anderen neu implementiert, entsteht ein Kompatibilitätsbereich – der Großteil funktioniert, Randfälle nicht. Das ist in jeder Modding-Szene der Fall, die lange genug existiert, um sich aufzuspalten.\n\n### Empfohlene Vorgehensweise bei Unsicherheit\n\n1. **Erstelle ein neues, leeres Profil** (siehe *So funktionieren Profile* im Hilfemenü) und wechsle dorthin.\n2. **Lege nur die WAX-Mod** in diesem Profil ab. Sonst nichts.\n3. **Drücke auf Spielen.** Funktioniert sie? Dann installiere sie in deinem eigentlichen Profil.\n4. **Funktioniert sie nicht?** Wahrscheinlich verwendet die Mod eine Funktion nach der von NAMS getesteten WAX-Version oder eine Funktion, die NAMS bewusst nicht neu implementiert hat.\n\n### Was du erwarten kannst\n\n- Wenn die Funktionen X, Y und Z in NAMS laufen und die gewünschte WAX-Mod zusätzlich die nicht unterstützte Funktion W benötigt – du aber auf W verzichten kannst –, funktionieren X, Y und Z weiterhin problemlos daneben.\n- Ist W für die Mod unverzichtbar und NAMS besitzt diese Funktion nicht, entscheidest du dich zwischen WAX (W ist verfügbar, andere NAMS-Vorteile entfallen) und NAMS (alles andere ist verfügbar, aber nicht W).\n\n**Vergiss auch die andere Seite dieses Kompromisses nicht:** Bei WAX zu bleiben bedeutet, auf die **NAMS-exklusiven Mods** zu verzichten, die unter WAX überhaupt nicht laufen – mehrere wechselbare Outfits pro Charakter, benutzerdefinierte Quests und das breitere Plugin-Ökosystem (Multiplayer Mod, YP-Devkit und zukünftige Plugins). Diese hängen von NAMS-APIs ab, die WAX nicht besitzt. Die Wahl lautet daher nicht „NAMS ohne W“ gegenüber „WAX mit W“, sondern „NAMS-Ökosystem ohne W“ gegenüber „WAX mit W, aber ohne alles NAMS-Exklusive“.\n\nDas ist ein echter Kompromiss, und die Entscheidung liegt bei dir. Wir sind nicht die richtigen Ansprechpartner dafür, ob eine bestimmte WAX-exklusive Funktion NAMS-Unterstützung erhalten wird – das ist eine Frage der Ökosystem-Roadmap und sollte am besten im Discord von YoRHa Continuum gestellt werden.';

  @override
  String get modsTutorialInstallStep3Title => 'Deine installierten Mods';

  @override
  String get modsTutorialInstallStep3Body =>
      'Jede installierte Mod wird hier angezeigt.\n\n**Schalter rechts** – aktiviert oder deaktiviert die Mod. Beim Deaktivieren bleibt sie installiert, der Modloader überspringt sie jedoch beim nächsten Start.\n\n**Das Spiel stürzt beim Start ab?** Deaktiviere die Hälfte deiner Mods, starte das Spiel und wiederhole den Vorgang. Mit den Schaltern kannst du den Fehler schnell eingrenzen.\n\n**Warnsymbole** kennzeichnen Mods, die miteinander in Konflikt stehen (zum Beispiel zwei Mods, die dasselbe Outfit ersetzen). Das ist der häufigste Grund dafür, dass das Spiel den Titelbildschirm nicht erreicht.';

  @override
  String get modsTutorialInstallStep4Title => 'Mod-Details';

  @override
  String get modsTutorialInstallStep4Body =>
      'Klicke auf eine Mod in der Liste, um hier ihre Details anzuzeigen: Autor, Version, veränderte Inhalte, Konflikte mit anderen Mods und alle **gebündelten Texturpakete oder Cutscenes**, die mitgeliefert wurden.\n\nFunktioniert eine Mod nicht, werden die häufigsten Ursachen hier angezeigt – etwa *erfordert eine neuere NAMS-Version* oder *steht in Konflikt mit einer anderen Mod*. Beides ist sichtbar, **bevor** du auf Spielen drückst.\n\nDie Schaltfläche **Deinstallieren** entfernt die Mod einschließlich ihrer gebündelten Zusatzinhalte sauber.';

  @override
  String get modsTutorialInstallStep5Title =>
      'Eigenständige Textur-Mods → Reiter Texturen';

  @override
  String get modsTutorialInstallStep5Body =>
      '**Reine Textur-Mods** (HD-Upscaling-Pakete oder Farbänderungen) werden nicht hier installiert. Sie besitzen einen eigenen Reiter.\n\nKlicke in der Seitenleiste auf **Texturen**, um sie zu installieren. Ziehe `.zip`-Archive auf dieselbe Weise hinein – der Launcher erkennt, womit er es zu tun hat.\n\n**Hinweis:** Wenn eine Charakter-Mod eigene Texturen *bündelt*, werden diese automatisch zusammen mit der Mod installiert. Den Texturen-Reiter verwendest du nur für **eigenständige** Texturpakete.';

  @override
  String get modsTutorialInstallStep6Title =>
      'Cutscene-Mods → Reiter Cutscenes';

  @override
  String get modsTutorialInstallStep6Body =>
      'Auch **Cutscene-Mods** (HD-Cutscenes oder Ersatzvideos) besitzen einen eigenen Reiter.\n\nKlicke in der Seitenleiste auf **Cutscenes**, um sie zu installieren.\n\n**Es gilt dieselbe Regel wie bei Texturen:** Bündelt eine Charakter-Mod Cutscenes, werden diese automatisch installiert. Den Reiter Cutscenes verwendest du nur für **eigenständige** Cutscene-Pakete.';

  @override
  String get modsTutorialInstallStep7Title => 'Drücke auf Spielen';

  @override
  String get modsTutorialInstallStep7Body =>
      'Wechsle zurück zum Reiter **Launcher** und drücke auf **SPIELEN**. Der Modloader liest deine Mods bei jedem Spielstart neu ein, daher musst du diesen Launcher nicht neu starten.\n\n### Wenn das Spiel abstürzt\n\nÖffne **Protokolle** (unten rechts) – die Ausgabe des Modloaders nennt normalerweise die fehlerhafte Mod. Kehre anschließend hierher zurück und deaktiviere sie.\n\n### Auch bei deaktivierten Mods weiterhin fehlerhaft?\n\nFalls du oder ein früherer Mod-Manager irgendwann lose `.dat`-/`.dtt`-Dateien direkt in `<gameDir>/data/` abgelegt habt, werden diese weiterhin von der Engine erkannt – der Modloader kann sie jedoch weder sehen noch deaktivieren. Genau dieses Durcheinander vermeidet der Launcher: Jede Mod bleibt in `nams/mods/<modId>/` isoliert, statt echte Spieldateien zu überschreiben.\n\nÖffne **Protokolle → Diagnose** und prüfe den Abschnitt *Überlagerung des ursprünglichen data/-Ordners*. Alles, was dort aufgeführt wird, stammt aus einer alten Installation. Verschiebe diese Ordner aus `data/`, und dein Spiel befindet sich wieder in einem sauberen Zustand.';

  @override
  String get modsTutorialProfilesStep1Title => 'Wofür Profile gedacht sind';

  @override
  String get modsTutorialProfilesStep1Body =>
      'Mit Profilen kannst du mehrere getrennte Mod-Zusammenstellungen nebeneinander aufbewahren.\n\nZum Beispiel:\n\n- Ein **Hauptprofil** mit den Mods, mit denen du tatsächlich spielst.\n- Ein **Testprofil**, um neue Dinge auszuprobieren.\n\nBeschädigt eine fragwürdige neue Mod dein Spiel, wechselst du einfach zurück zu **Hauptprofil** und kannst sofort wieder spielen. Deine Zusammenstellungen werden nie vermischt.\n\n**Wichtig:** Mods, die du derzeit nicht verwendest, werden nicht gelöscht – sie werden nur beiseitegelegt und stehen beim nächsten Wechsel wieder bereit.';

  @override
  String get modsTutorialProfilesStep2Title => 'Ein Profil erstellen';

  @override
  String get modsTutorialProfilesStep2Body =>
      'Klicke in der Profilleiste auf **NEU**, gib einen Namen ein und bestätige.\n\nDer Launcher erstellt ein neues, leeres Profil und wechselt dorthin. Die Mods deines vorherigen Profils bleiben sicher auf dem Datenträger – sie sind nicht verschwunden, sondern lediglich beiseitegelegt.\n\nNun kannst du in diesem neuen Profil beliebige Inhalte installieren, ohne deine anderen Zusammenstellungen zu verändern.';

  @override
  String get modsTutorialProfilesStep3Title =>
      'Wechseln, umbenennen und löschen';

  @override
  String get modsTutorialProfilesStep3Body =>
      '**Wechseln** – wähle ein Profil aus dem Dropdown-Menü. Deine Mod-Liste wird sofort umgestellt.\n\n**Umbenennen** – ändere den Namen eines Profils, ohne Inhalte zu verlieren.\n\n**Löschen** – entferne ein inaktives Profil dauerhaft. Das aktive oder letzte verbleibende Profil kann nicht gelöscht werden.\n\nDer gesamte Wechsel erfolgt in einem einzigen Schritt. Falls etwas schiefgeht, wird automatisch zurückgerollt, sodass kein beschädigter Zustand entstehen kann.';

  @override
  String get modsTutorialProfilesStep4Title =>
      'Was zum Profil gehört – und was nicht';

  @override
  String get modsTutorialProfilesStep4Body =>
      '**Profilspezifisch** (ändert sich beim Wechsel):\n\n- Deine installierten Mods\n- Welche Mods aktiviert oder deaktiviert sind\n- Texturen, die mit einer Mod gebündelt wurden\n\n**Zwischen allen Profilen gemeinsam** (ändert sich nie):\n\n- Eigenständige Texturpakete, die du über den Texturen-Reiter installiert hast\n- Cutscene-Mods\n- Plugins\n- Alle Launcher-Einstellungen\n\nProfile wechseln also nur tatsächlich mod-spezifische Inhalte. Deine globale Einrichtung bleibt überall erhalten.';

  @override
  String get platformUnsupportedTitle =>
      'Starten auf dieser Plattform nicht möglich';

  @override
  String get platformUnsupportedLinux =>
      'NieR:Automata ist ein Windows-Spiel und benötigt daher eine Kompatibilitätsschicht, um unter Linux ausgeführt zu werden.\n\nInstalliere Steam mit Proton (das Spiel läuft unter Proton problemlos) oder installiere CrossOver/Wine. Sobald eine Laufzeitumgebung vorhanden ist, kann der Launcher das Spiel starten.\n\nUnter nativem Linux ohne Übersetzungsschicht kann das Spiel nicht ausgeführt werden.';

  @override
  String get platformUnsupportedMacos =>
      'NieR:Automata ist ein Windows-Spiel. Führe den Launcher über CrossOver/Wine aus – das hat in der Vergangenheit funktioniert, wurde jedoch in letzter Zeit nicht erneut getestet. Unter nativem macOS ohne Übersetzungsschicht kann das Spiel nicht ausgeführt werden.\n\nFalls du es dennoch zum Laufen gebracht hast, verwende bitte direkt die Kommandozeile statt dieses Launchers.';

  @override
  String get playDisabledTooltip =>
      'Starten auf dieser Plattform nicht verfügbar';

  @override
  String get diagnosticsClose => 'Schließen';

  @override
  String get diagnosticsSectionDataOverlay =>
      'Überlagerung des ursprünglichen data/-Ordners (manuell abgelegte Dateien)';

  @override
  String get diagnosticsSectionExternalLegacy => 'Extern / veraltet';

  @override
  String get diagnosticsSectionGameRootExtras =>
      'Zusätzliche Dateien im Spielstammverzeichnis (nicht original)';

  @override
  String get diagnosticsSectionGameIdentity => 'Spiel-Identität';

  @override
  String get diagnosticsSectionNamsHealth => 'NAMS-Status';

  @override
  String get diagnosticsSectionReshade => 'ReShade';

  @override
  String get diagnosticsSectionMigoto => '3DMigoto';

  @override
  String get diagnosticsSectionTexturePacks => 'Textur-Pakete';

  @override
  String get diagnosticsSectionVanillaDrops => 'Dateien im originalen data/';

  @override
  String get diagnosticsSectionNonDefault => 'Geänderte Einstellungen';

  @override
  String get diagnosticsSectionRecentIssues => 'Letzte NAMS-Probleme';

  @override
  String diagnosticsExeVariant(String variant) {
    return 'exe: $variant';
  }

  @override
  String get diagnosticsExeUnsupported => 'nicht unterstützt';

  @override
  String get diagnosticsDlcPresent => 'DLC';

  @override
  String get diagnosticsGameRunning => 'läuft';

  @override
  String get diagnosticsNamsPresent => 'NAMS.exe';

  @override
  String diagnosticsMissingFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count fehlende Dateien',
      one: '1 fehlende Datei',
    );
    return '$_temp0';
  }

  @override
  String get diagnosticsInstalled => 'installiert';

  @override
  String get diagnosticsEnabled => 'aktiv';

  @override
  String get diagnosticsShadersMissing => 'Shader fehlen';

  @override
  String get diagnosticsTexturePacksUnavailable =>
      'NAMS-Texturabfrage nicht verfügbar';

  @override
  String get diagnosticsExtraFile => 'zusätzlich';

  @override
  String diagnosticsFileCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Dateien',
      one: '1 Datei',
    );
    return '$_temp0';
  }

  @override
  String diagnosticsMoreItems(int count) {
    return '... $count weitere';
  }

  @override
  String get refreshButton => 'Aktualisieren';

  @override
  String get tabModloaderLabel => 'Modloader';

  @override
  String get tabYorhaLabel => 'YoRHa Protocol';

  @override
  String get configEditorTitle => 'KONFIGURATIONSEDITOR';

  @override
  String get changelogTitle => 'NEUIGKEITEN';

  @override
  String get tipDragTextures =>
      'Ziehe Textur-Mods direkt in den Texturen-Reiter';

  @override
  String get tipHdCutscenes =>
      'HD-Cutscene-Mods werden automatisch erkannt und konfiguriert';

  @override
  String get tipLodModPreviews =>
      'LOD-Mod-Einstellungen enthalten Vorher-/Nachher-Vorschaubilder';

  @override
  String get tipFaqButton =>
      'Über die FAQ-Schaltfläche erfährst du, welche Mods YoRHa Protocol ersetzt';

  @override
  String get tipReShadeAuto =>
      'ReShade wird automatisch erkannt – keine manuelle Konfiguration erforderlich';

  @override
  String get tipFreecam =>
      'YoRHa Protocol enthält eine integrierte freie Kamera mit Speicherplätzen';

  @override
  String get tipCustomQuests =>
      'Benutzerdefinierte Quests folgen bald – bleib gespannt';

  @override
  String get sectionNams => 'NAMS';

  @override
  String get sectionTextureInjection => 'TEXTURINJEKTION';

  @override
  String get sectionLodMod => 'LOD MOD';

  @override
  String get sectionLevelOfDetail => 'DETAILSTUFE';

  @override
  String get sectionAmbientOcclusion => 'UMGEBUNGSVERDECKUNG';

  @override
  String get sectionShadows => 'SCHATTEN';

  @override
  String get sectionPostProcessing => 'NACHBEARBEITUNG';

  @override
  String get labelValidateModelData => 'Modelldaten überprüfen';

  @override
  String get tooltipValidateModelData =>
      'Das Spiel überprüft Modelldaten beim Laden. Normalerweise schlägt die Prüfung unbemerkt fehl und das Spiel fährt mit beschädigten Daten fort, was unsichtbare Modelle oder Grafikfehler verursachen kann. Ist diese Option aktiviert, zeigt NAMS das Prüfergebnis in einem Dialog an, sodass du genau siehst, welches Modell warum fehlgeschlagen ist.';

  @override
  String get labelPreloadMaxDimension => 'Maximale Vorladedimension';

  @override
  String get tooltipPreloadMaxDimension =>
      'Maximale Texturgröße, die beim Start in den Arbeitsspeicher vorgeladen wird. 2048 = Standard, 4096 = Texturen bis 4K vorladen, 16384 = alles vorladen. Höhere Werte bedeuten längere Ladezeiten, aber weniger Ruckler im Spiel.';

  @override
  String get labelPreloadAllTextures => 'Alle Texturen vorladen';

  @override
  String get tooltipPreloadAllTextures =>
      'Lädt unabhängig von ihrer Größe ALLE Texturen in den Arbeitsspeicher vor. Beseitigt sämtliche Ruckler durch plötzlich eingeblendete Texturen, benötigt aber mindestens 32 GB RAM und verlangsamt den Start deutlich.';

  @override
  String get labelEnableLodMod => 'LodMod aktivieren';

  @override
  String get tooltipEnableLodMod =>
      'Hauptschalter für alle visuellen LodMod-Patches und -Neuschreibungen.';

  @override
  String get labelLodMultiplier => 'LOD-Multiplikator';

  @override
  String get tooltipLodMultiplier =>
      'Steuert die Darstellungsentfernungen der LOD-Stufen (Detailstufen). 0 = LODs deaktiviert (beste Qualität, keine Pop-ins). 1 = ursprünglicher Wert. 10+ hilft, AO-Ausbluten zu reduzieren, ohne LODs vollständig zu deaktivieren. Niedrigere Werte verbessern die Darstellung, können aber Leistung kosten.';

  @override
  String get labelDisableManualCulling => 'Manuelles Culling deaktivieren';

  @override
  String get tooltipDisableManualCulling =>
      'Verhindert, dass Modelle oder Geometrie bei bestimmten Entfernungen oder Kamerawinkeln zufällig verschwinden. Behebt beispielsweise das Verschwinden des Einkaufszentrums nach dem Überqueren der Brücke oder von Gebäuden außerhalb des Lagers. Seltene, unschöne LOD-Modelle, die dadurch sichtbar würden, werden herausgefiltert.';

  @override
  String get labelAoWidth => 'AO-Breite';

  @override
  String get tooltipAoWidth =>
      'Multiplikator für die horizontale AO-Auflösung. Die ursprüngliche AO läuft mit einem Viertel der Bildschirmauflösung. 2.0 = halbe Bildschirmauflösung (schärfere AO, aber rechenintensiv). 1.5 ist ein guter Kompromiss. Bereich: 0.1–2.0. Nur eine Achse auf 2 zu setzen, kann eine leichtere Alternative sein.';

  @override
  String get labelAoHeight => 'AO-Höhe';

  @override
  String get tooltipAoHeight =>
      'Multiplikator für die vertikale AO-Auflösung. Die ursprüngliche AO läuft mit einem Viertel der Bildschirmauflösung. 2.0 = halbe Bildschirmauflösung (schärfere AO, aber rechenintensiv). 1.5 ist ein guter Kompromiss. Bereich: 0.1–2.0. Beide Achsen auf 2.0 können im schlimmsten Fall etwa 10 FPS kosten.';

  @override
  String get labelShadowResolution => 'Schattenauflösung';

  @override
  String get tooltipShadowResolution =>
      'Texturgröße der Schattenkarte. Höher = schärfere Schatten, aber größere GPU-Belastung. 2048 = ursprünglich, 4096 = gute Verbesserung, 8192 = sehr scharf. Muss eine Zweierpotenz sein. Die Schärfe hängt sowohl von der Auflösung als auch von der Entfernung ab (größere Entfernung = mehr abzudeckende Fläche, daher sinkt die Qualität).';

  @override
  String get labelDistanceMultiplier => 'Entfernungsmultiplikator';

  @override
  String get tooltipDistanceMultiplier =>
      'Multipliziert die Schatten-Darstellungsentfernung pro Szene. 2.0 = Schatten sind doppelt so weit sichtbar. Ursprünglicher Wert: 1.0. Deaktiviere die unten stehenden Mindest-/Höchstwerte, damit dies korrekt funktioniert, oder nutze sie, um den vom Multiplikator gesetzten Bereich einzuschränken.';

  @override
  String get labelDistanceMinimum => 'Mindestentfernung';

  @override
  String get tooltipDistanceMinimum =>
      'Untergrenze für die Schatten-Darstellungsentfernung. 0 = aus (kein Mindestwert). Etwa 70 bei einer Auflösung von 8192 entspricht der ursprünglichen Qualität und erhöht die Schattenentfernung deutlich.';

  @override
  String get labelDistanceMaximum => 'Höchstentfernung';

  @override
  String get tooltipDistanceMaximum =>
      'Obergrenze für die Schatten-Darstellungsentfernung. 0 = aus (kein Höchstwert). Nur sinnvoll, wenn die ursprünglichen Entfernungen des Spiels Leistungsprobleme verursachen.';

  @override
  String get labelDistancePss => 'PSS-Entfernung';

  @override
  String get tooltipDistancePss =>
      'Aktiviert die PSS-Schattenverteilung für eine gleichmäßigere Schattenqualität. 0 = aus. Gute Werte: 0.5–0.9. Sieht in einigen Gebieten hervorragend aus, kann in anderen jedoch unscharf wirken. Sollte deutlich höher als andere Entfernungswerte gesetzt werden (etwa 1500 für große offene Gebiete).';

  @override
  String get labelFilterStrengthBias => 'Filterstärke-Versatz';

  @override
  String get tooltipFilterStrengthBias =>
      'Passt die Stärke des Schatten-Weichzeichnungsfilters pro Szene an. 0 = aus. -1 = schärfere Schatten. Positive Werte = weichere Schatten. Verschiedene Gebiete verwenden unterschiedliche Stärken (Wald = weicher). Kann mit Mindest-/Höchstwerten kombiniert werden, um den Bereich einzuschränken.';

  @override
  String get labelFilterStrengthMin => 'Minimale Filterstärke';

  @override
  String get tooltipFilterStrengthMin =>
      'Erzwingt in allen Gebieten eine minimale Schattenfilterstärke. 0 = aus. Der Spielstandard variiert je nach Szene (meist etwa 4). Verhindert, dass Schatten in einem Gebiet zu scharf werden.';

  @override
  String get labelFilterStrengthMax => 'Maximale Filterstärke';

  @override
  String get tooltipFilterStrengthMax =>
      'Erzwingt in allen Gebieten eine maximale Schattenfilterstärke. 0 = aus. Der Spielstandard variiert je nach Szene (meist etwa 4). Verhindert, dass Schatten in einem Gebiet zu unscharf werden.';

  @override
  String get labelHqShadowModels => 'HQ-Schattenmodelle';

  @override
  String get tooltipHqShadowModels =>
      'Verwendet für Schatten HQ-Echtzeitmodelle statt statischer LQ-Modelle. Baumschatten bewegen sich dann mit dem Wind, statt eingefroren zu sein. Experimentell – funktioniert in den Stadtruinen gut, kann in seltenen Gebieten jedoch Probleme verursachen.';

  @override
  String get labelForceAllShadowModels => 'Schatten für alle Modelle erzwingen';

  @override
  String get tooltipForceAllShadowModels =>
      'Erzwingt Schattenwurf für alle Modelle, einschließlich kleiner Objekte wie Steine und Gras. Experimentell – in seltenen Fällen können unsichtbare Modelle Schatten werfen. Bisher wurden keine Probleme festgestellt.';

  @override
  String get labelDisableVignette => 'Vignette deaktivieren';

  @override
  String get tooltipDisableVignette =>
      'Entfernt den dunklen Vignetteneffekt an den Bildschirmrändern. Bei einigen Ladebildschirmen kann er weiterhin fest in die Texturen eingebettet sein.';

  @override
  String get configAppliesOnRestart => 'Gilt nach einem Neustart';

  @override
  String get configAppliesLive => 'Gilt sofort (live)';

  @override
  String get dropZoneBrowseFolder => 'Oder einen Ordner auswählen';

  @override
  String get labelGiEnabled => 'Globale Beleuchtung aktivieren';

  @override
  String get tooltipGiEnabled =>
      'Globale Beleuchtung im FAR-Stil. Deutlicher FPS-Gewinn auf Kosten eines Teils der Beleuchtungsgenauigkeit.';

  @override
  String get labelGiWorkgroupSize => 'GI-Arbeitsgruppengröße';

  @override
  String get tooltipGiWorkgroupSize =>
      'Anzahl der Lichtvolumen, die pro GI-Aufruf verarbeitet werden. 128 = ursprüngliche Qualität, 64/32/16 = zunehmend schneller, aber gröber. Niedrigere Werte tauschen Beleuchtungsgenauigkeit gegen FPS.';

  @override
  String get labelGiMinLightExtent => 'Minimale GI-Lichtausdehnung';

  @override
  String get tooltipGiMinLightExtent =>
      'Entfernt kleine, weit entfernte Lichter aus der GI. 0.0 = kein Culling (alle Lichter tragen bei), 0.5 = aggressives Culling. Bereich: 0.0–1.0.';

  @override
  String get cardExperimental => 'EXPERIMENTELL';

  @override
  String get lodModResetButton => 'Auf Standardwerte zurücksetzen';

  @override
  String get lodModResetConfirmTitle => 'LodMod-Einstellungen zurücksetzen?';

  @override
  String get lodModResetConfirmBody =>
      'Dadurch wird jedes LodMod-Feld in diesem Reiter auf seinen Standardwert zurückgesetzt. Deine aktuellen Werte werden überschrieben. Fortfahren?';

  @override
  String get lodModResetConfirmAction => 'Zurücksetzen';

  @override
  String get lodModResetToast =>
      'LodMod-Einstellungen auf Standardwerte zurückgesetzt';

  @override
  String get experimentalWarningTitle =>
      'Experimentell – kann Fehler verursachen';

  @override
  String get experimentalWarningBody =>
      'Diese Einstellungen umgehen Spielgrenzen, auf die sich die Engine verlässt. Sie werden NICHT unterstützt und verursachen bekanntermaßen Probleme. Aktiviere sie nur, wenn du genau weißt, was du tust. Fehler, die dadurch entstehen, werden weder in NAMS noch im Launcher untersucht.';

  @override
  String get labelFpsUncapInMenus => 'FPS in Menüs / beim Laden freigeben';

  @override
  String get tooltipFpsUncapInMenus =>
      'Entfernt die 60-FPS-Begrenzung in Menüs und Ladebildschirmen. Das Laden wirkt schneller und Menüanimationen werden flüssiger. Sicher: Das Gameplay bleibt unbeeinflusst.\n\nKann live umgeschaltet werden, wenn die Option beim Spielstart aktiviert war. War sie beim Start deaktiviert, erfordert das spätere Aktivieren einen Neustart.';

  @override
  String get labelFpsUncapInGameplay => 'FPS im Gameplay freigeben';

  @override
  String get tooltipFpsUncapInGameplay =>
      'Entfernt die 60-FPS-Begrenzung während des Gameplays. WARNUNG: Physik, Animationen und das Timing von Cutscenes in NieR:Automata sind an die 60-FPS-Begrenzung gekoppelt. Das Aufheben verursacht fehlerhafte Physik (Sprunghöhe, Unverwundbarkeitsfenster beim Ausweichen), veränderte Animationsgeschwindigkeiten, asynchronen Ton in Cutscenes und Softlocks in geskripteten Abläufen. Verwende dies nur, wenn du die Kompromisse genau verstehst.\n\nKann live umgeschaltet werden, wenn die Option beim Spielstart aktiviert war. War sie beim Start deaktiviert, erfordert das spätere Aktivieren einen Neustart.';

  @override
  String get labelFpsLimit => 'FPS-Begrenzung';

  @override
  String get tooltipFpsLimit =>
      'FPS-Grenze, die bei freigegebener Bildrate gilt. 0 = unbegrenzt. Ansonsten 60–1000 (NAMS begrenzt Werte außerhalb dieses Bereichs). Werte unter 60 werden angehoben, weil die interne Spin-Wait-Schleife des Spiels Framezeiten ignoriert, die länger als das ursprüngliche 60-FPS-Ziel sind. Tipp: Eine Begrenzung auf die Hälfte der Bildwiederholrate deines Monitors liefert flüssigere Bewegungen als die ursprünglichen 60 FPS (z. B. 72 bei 144 Hz, 82 bei 165 Hz oder 120 bei 240 Hz).';

  @override
  String get tutorialValidateModel =>
      'Zeigt an, wenn das Modell einer Mod beschädigt ist, statt unbemerkt fehlzuschlagen.';

  @override
  String get labelValidateScripts => 'Skripte überprüfen';

  @override
  String get tooltipValidateScripts =>
      'Zeigt Skriptfehler in einem Dialog an, statt dass das Spiel kommentarlos abstürzt.';

  @override
  String get previewValidationDialog => 'ÜBERPRÜFUNGSDIALOG';

  @override
  String get previewScriptErrorDialog => 'SKRIPTFEHLERDIALOG';

  @override
  String get labelLoadingStallHints => 'Hinweise bei festhängendem Laden';

  @override
  String get tooltipLoadingStallHints =>
      'Zeigt zunehmend deutliche Hinweise an, wenn der Ladebildschirm zu lange dauert. Hilft dabei, fehlende oder gelöschte Mod-Dateien zu erkennen.';

  @override
  String get labelFixWindTimerBug => 'Wind-Timer-Fehler beheben';

  @override
  String get tooltipFixWindTimerBug =>
      'Behebt einen Fehler des ursprünglichen Spiels, durch den die Windanimation nach Erreichen der maximalen Spielzeit stoppt. Verwendet stattdessen den Geschwindigkeitsfaktor des Spiels.';

  @override
  String get labelDisablePluginLoading => 'Plugin-Laden deaktivieren';

  @override
  String get tooltipDisablePluginLoading =>
      'Überspringt das Laden aller Plugin-DLLs (z. B. des YoRHa-Protocol-Arbeitsbereichs). Alle Engine-Funktionen von NAMS funktionieren weiterhin.';

  @override
  String get labelDisableContentFeatures => 'Inhaltsfunktionen deaktivieren';

  @override
  String get tooltipDisableContentFeatures =>
      'Hauptschalter für alle Funktionen der Inhaltsschicht. Ist er aktiviert, läuft NAMS als reine Engine-Schicht (Mauskorrekturen, Überprüfung, Heap-Anpassung, Absturzbehebungen), ohne Unterstützung für Gegenstands-, Waffen-, Outfit-, Quest- oder Accessoire-Mods. Nützlich für Benchmarks oder zur Abgrenzung von Engine- gegenüber Inhaltsproblemen.';

  @override
  String get labelContentItems => 'Gegenstände / Waffen / Läden';

  @override
  String get tooltipContentItems =>
      'Benutzerdefinierte Gegenstände, Waffen, Outfits und Ladeneinträge. Deaktivieren, um ohne gegenstandsbezogene Mods zu spielen. Neustart erforderlich.';

  @override
  String get labelContentAccessories => 'Accessoires';

  @override
  String get tooltipContentAccessories =>
      'Benutzerdefinierte Accessoires (Gesichtsmasken, Lunar Tear, Masamune-Maske usw.) sowie der Ablauf zum An- und Ablegen. Deaktivieren, um ohne Accessoire-Mods zu spielen. Neustart erforderlich.';

  @override
  String get labelContentAssembleMeshes => 'Spielermodelle';

  @override
  String get tooltipContentAssembleMeshes =>
      'Benutzerdefinierte Spielermodelle (Mesh-Wechsel sowie Haar-, Outfit- und Maskenüberschreibungen). Deaktivieren, um die ursprünglichen Spielermodelle unverändert darzustellen. Neustart erforderlich.';

  @override
  String get labelContentQuestIntegration => 'Quests / Nachrichten / Sprache';

  @override
  String get tooltipContentQuestIntegration =>
      'Benutzerdefinierte Quests, Nachrichten, Sprachzeilen und die Quest-UI-Integration, die sie aktiviert. Deaktivieren, um ohne Quest-Mods zu spielen. Neustart erforderlich.';

  @override
  String get labelContentEffectsApplier => 'Effektregeln';

  @override
  String get tooltipContentEffectsApplier =>
      'Wendet in jedem Frame Waffen- und Outfit-Effektregeln auf Spielerwerte an (Schadensmultiplikatoren, Änderungen beim Ausweichen, Immunitäten usw.).';

  @override
  String get labelContentEquipTracker => 'Ausrüstungsüberwachung';

  @override
  String get tooltipContentEquipTracker =>
      'Erkennt Änderungen beim Ausrüsten und Ablegen von Waffen. Steuert Effektregeln und SDK-Callbacks beim Ausrüsten.';

  @override
  String get labelContentMcd => 'Benutzerdefinierter Text';

  @override
  String get tooltipContentMcd =>
      'Anpassung von Texten im Spiel (benutzerdefinierte Gegenstandsnamen, Beschreibungen und von Mods bereitgestellte Dialogtexte).';

  @override
  String get labelContentBuddyRubySelector =>
      'Begleiter-Outfit-Auswahl (experimentell)';

  @override
  String get tooltipContentBuddyRubySelector =>
      'Patcht das globale Gesprächsskript für Begleiter und fügt einen Eintrag „Outfit wechseln“ hinzu, der gemoddete Begleiter-Outfits auflistet. Deaktiviere dies, falls das gepatchte Gesprächsskript Instabilität verursacht oder andere Skript-Mods beeinträchtigt.';

  @override
  String get cardContentFeatures => 'INHALTSFUNKTIONEN';

  @override
  String get contentFeaturesDescription =>
      'Einzelschalter für Funktionen der Inhaltsschicht. Standardmäßig sind alle aktiviert. Nützlich, um ein Problem auf ein bestimmtes Subsystem einzugrenzen. Erfordert einen Neustart des Spiels.';

  @override
  String get labelDisableReShadeLoading => 'ReShade-Laden deaktivieren';

  @override
  String get tooltipDisableReShadeLoading =>
      'Überspringt die automatische Erkennung der ReShade-DLL im Ordner reshade/ und lädt sie nicht mehr.';

  @override
  String get labelDisable3dmigotoLoading => '3DMigoto-Laden deaktivieren';

  @override
  String get tooltipDisable3dmigotoLoading =>
      'Lädt die 3DMigoto-Runtime aus thirdparty/3dmigoto/ nicht mehr. Ausschalten, um Shader-Mods nicht zu laden.';

  @override
  String get labelDisableTextureInjection => 'Texturinjektion deaktivieren';

  @override
  String get tooltipDisableTextureInjection =>
      'Überspringt die Texturinjektion aus dem Mods-Ordner. Nützlich zur Fehlereingrenzung oder wenn du installierte Textur-Mods nicht verwenden möchtest.';

  @override
  String get labelOutfitSwapVisualEffects =>
      'Visuelle Effekte beim Outfit-Wechsel';

  @override
  String get tooltipOutfitSwapVisualEffects =>
      'Spielt beim direkten Outfit-Wechsel die visuellen Effekte ab: die Blendenanimation beim Erscheinen des Pods, den Vorhang und den Glitch-Filter des Hacking-Bildschirms. Deaktiviere dies für einen sofortigen Wechsel ohne Effekte – das Modell wird weiterhin neu geladen. Wirkt sofort, kein Neustart erforderlich.';

  @override
  String get labelExperimentalDefaultOutfits =>
      'Standard-Outfits (experimentell)';

  @override
  String get tooltipExperimentalDefaultOutfits =>
      'Ermöglicht es, installierte Outfit-Mods ab Spielstart zu aktivieren, so als lägen ihre Dateien im data-Ordner des Spiels. Wenn aktiviert, zeigt das Mod-Detailfenster pro Spielermodell einen Stern-Button, um es als Standard beim Start festzulegen. Standardmäßig aus, solange sich die Funktion stabilisiert. Erfordert einen Spielneustart.';

  @override
  String get labelDisableSplashScreen => 'Startbildschirm deaktivieren';

  @override
  String get tooltipDisableSplashScreen =>
      'Überspringt das Startfenster, das während des Ladens des Spiels angezeigt wird. Das ursprüngliche Spiel zeigte sein Fenster, bevor es bereit war, was Größenänderungs- und Flackerartefakte verursachte; NAMS vervollständigt den Startbildschirm, sodass das Fenster erst sichtbar wird, wenn es bereit ist. Durch Aktivieren dieser Option kehren die ursprünglichen Startartefakte zurück.';

  @override
  String get tooltipValidateModelDataSettings =>
      'Zeigt Fehler bei der Modellüberprüfung in einem Dialog an, statt unbemerkt fehlzuschlagen.';

  @override
  String get heapDefault => 'Standard';

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
  String get heapScriptEngine => 'Skript-Engine';

  @override
  String get heapScriptEngineDesc => 'Für komplexe Skript-Mods (mRuby / HAP).';

  @override
  String get heapPlayerModels => 'Spielermodelle';

  @override
  String get heapPlayerModelsDesc =>
      'Für große Mods, die Spielermodelle ersetzen.';

  @override
  String get heapPlayerTextures => 'Spielertexturen';

  @override
  String get heapPlayerTexturesDesc => 'Für hochauflösende Spielertextur-Mods.';

  @override
  String get heapEnemyBgModels => 'Gegner-/Hintergrundmodelle';

  @override
  String get heapEnemyBgModelsDesc =>
      'Für Mods an Gegner- oder Umgebungsmodellen.';

  @override
  String get heapEnemyBgTextures => 'Gegner-/Hintergrundtexturen';

  @override
  String get heapEnemyBgTexturesDesc =>
      'Für hochauflösende Gegner- oder Umgebungstexturen.';

  @override
  String get tutorialLodModEnable =>
      'Aktiviere dies für eine bessere Darstellung. Dies ist die wirkungsvollste Einstellung.';

  @override
  String get tutorialLodModShadowRes =>
      'Höhere Werte bedeuten schärfere Schatten. 8192 wird empfohlen.';

  @override
  String get tutorialLodModComparison =>
      'Klicke auf einen Vergleich, um den Unterschied im Vollbild anzuzeigen.';

  @override
  String get comparisonVanilla => 'ORIGINAL';

  @override
  String get comparisonDefaultEnabled => 'STANDARD (AKTIVIERT)';

  @override
  String get comparisonAo05x => 'AO 0.5x';

  @override
  String get comparisonAo20x => 'AO 2.0x';

  @override
  String get comparisonVignetteOn => 'VIGNETTE AN';

  @override
  String get comparisonVignetteOff => 'VIGNETTE AUS';

  @override
  String get comparison2048 => '2048';

  @override
  String get comparison8192 => '8192';

  @override
  String get comparisonDefault => 'STANDARD';

  @override
  String get comparison20x => '2.0x';

  @override
  String get comparisonPssMinus5 => 'PSS -5.0';

  @override
  String get comparisonBiasMinus5 => 'VERSATZ -5.0';

  @override
  String get comparisonOff => 'AUS';

  @override
  String get comparison30 => '3.0';

  @override
  String get comparisonHqForceAll => 'HQ + ALLE ERZWINGEN';

  @override
  String get tutorialKeybind =>
      'Klicke, um die Tastenbelegung zu ändern. Drücke eine beliebige Taste, um sie zuzuweisen.';

  @override
  String get tutorialDamageMultiplier =>
      'Passe das Gameplay an – erhöhe den Schaden, aktiviere unendliche Gesundheit und mehr.';

  @override
  String get labelOpenWorkspace => 'Arbeitsbereich öffnen';

  @override
  String get tooltipOpenWorkspace =>
      'Öffnet den YoRHa-Protocol-Arbeitsbereich im Spiel.';

  @override
  String get labelFreezeGame => 'Spiel einfrieren';

  @override
  String get tooltipFreezeGame =>
      'Friert das Spiel ein oder setzt es fort. Nützlich für Screenshots und Einzelbildschritte.';

  @override
  String get labelMaxSpeed => 'Maximale Geschwindigkeit';

  @override
  String get tooltipMaxSpeed =>
      'Schaltet die maximale Spielgeschwindigkeit für schnelles Reisen oder Tests ein und aus.';

  @override
  String get labelFreeCam => 'Freie Kamera';

  @override
  String get tooltipFreeCam =>
      'Schaltet die freie Kamera ein und aus. Volle Unterstützung für Tastatur/Maus und Controller.';

  @override
  String get labelPhaseJump => 'Phasensprung';

  @override
  String get tooltipPhaseJump =>
      'Springt zu einer zuvor ausgewählten Spielphase oder Szene. Lege das Ziel im Spiel fest.';

  @override
  String get labelToggleInput => 'Eingabe umschalten';

  @override
  String get tooltipToggleInput =>
      'Schaltet die Spieleingabe ein oder aus, während der Arbeitsbereich geöffnet ist.';

  @override
  String get labelAdvanceFrame => 'Einzelbild weiter';

  @override
  String get tooltipAdvanceFrame =>
      'Geht bei eingefrorenem Spiel ein Einzelbild vorwärts. Gedrückt halten, um schneller fortzufahren.';

  @override
  String get labelDevMode => 'Entwicklermodus';

  @override
  String get tooltipDevMode =>
      'Schaltet den Entwicklermodus ein und aus. Aktiviert Penetrations-/Stresstest-Schaltflächen und Debug-Werkzeuge.';

  @override
  String get labelWarpSave1 => 'Warp 1 speichern';

  @override
  String get tooltipWarpSave1 =>
      'Speichert die aktuelle Position und Ausrichtung im Warp-Speicherplatz 1.';

  @override
  String get labelWarpGoto1 => 'Zu 1 warpen';

  @override
  String get tooltipWarpGoto1 =>
      'Teleportiert zur im Warp-Speicherplatz 1 gespeicherten Position.';

  @override
  String get labelWarpSave2 => 'Warp 2 speichern';

  @override
  String get tooltipWarpSave2 =>
      'Speichert die aktuelle Position und Ausrichtung im Warp-Speicherplatz 2.';

  @override
  String get labelWarpGoto2 => 'Zu 2 warpen';

  @override
  String get tooltipWarpGoto2 =>
      'Teleportiert zur im Warp-Speicherplatz 2 gespeicherten Position.';

  @override
  String get labelGlobalKeybinds => 'Globale Tastenbelegungen';

  @override
  String get tooltipGlobalKeybinds =>
      'Tastenbelegungen funktionieren auch bei geschlossenem Arbeitsbereich.';

  @override
  String get labelLoadingSpeedup => 'Schnelleres Laden';

  @override
  String get tooltipLoadingSpeedup => 'Beschleunigt Ladebildschirme.';

  @override
  String get labelShaders => 'Shader';

  @override
  String get tooltipShaders =>
      'Shader des Arbeitsbereichs. Für bessere Leistung deaktivieren.';

  @override
  String get labelSound => 'Ton';

  @override
  String get tooltipSound =>
      'Ton bei Interaktionen mit der Benutzeroberfläche des Arbeitsbereichs.';

  @override
  String get labelDamageMultiplier => 'Schadensmultiplikator';

  @override
  String get tooltipDamageMultiplier => '2.0 = doppelter Schaden.';

  @override
  String get labelSyncEnemyLevels => 'Gegnerstufen synchronisieren';

  @override
  String get tooltipSyncEnemyLevels =>
      'Passt die Stufen der Gegner an deine Stufe an.';

  @override
  String get labelInfiniteHealth => 'Unendliche Gesundheit';

  @override
  String get tooltipInfiniteHealth => 'Du erleidest keinen Schaden.';

  @override
  String get labelInfiniteJump => 'Unendliche Sprünge';

  @override
  String get tooltipInfiniteJump => 'Unbegrenzte Sprünge in der Luft.';

  @override
  String get labelNoPodCooldown => 'Keine Pod-Abklingzeit';

  @override
  String get tooltipNoPodCooldown =>
      'Pod-Programme besitzen keine Abklingzeit.';

  @override
  String get labelInfiniteAirDash => 'Unendliche Luftsprints';

  @override
  String get tooltipInfiniteAirDash => 'Unbegrenzte Sprints in der Luft.';

  @override
  String get labelAutoStart => 'Automatisch starten';

  @override
  String get tooltipAutoStart =>
      'Startet den Zufallsgenerator beim Spielstart automatisch.';

  @override
  String get labelGroundEnemies => 'Bodengegner';

  @override
  String get tooltipGroundEnemies => 'Zufällige bodengebundene Gegner-Spawns.';

  @override
  String get labelFlyingEnemies => 'Fliegende Gegner';

  @override
  String get tooltipFlyingEnemies => 'Zufällige fliegende Gegner-Spawns.';

  @override
  String get labelAllowBigEnemies => 'Große Gegner erlauben';

  @override
  String get tooltipAllowBigEnemies => 'Erlaubt große Gegner.';

  @override
  String get labelIncludeDlcEnemies => 'DLC-Gegner einbeziehen';

  @override
  String get tooltipIncludeDlcEnemies => 'Bezieht DLC-Gegner ein.';

  @override
  String get tutorialCameraAccel =>
      'Entfernt die Mausbeschleunigung für eine direkte 1:1-Eingabe.';

  @override
  String get tutorialWipBanner =>
      'Diese Funktionen erscheinen in zukünftigen NAMS-Aktualisierungen.';

  @override
  String get labelFixCameraAcceleration => 'Kamerabeschleunigung korrigieren';

  @override
  String get tooltipFixCameraAcceleration =>
      'Lineare 1:1-Mausbewegung. Entfernt Totzone und Beschleunigungskurve aus der Kameradrehung.';

  @override
  String get labelSensitivity => 'Empfindlichkeit';

  @override
  String get tooltipSensitivity =>
      'Multiplikator für die Kameraempfindlichkeit. Höher = schnellere Drehung. 2.0 ist ein guter Standardwert.';

  @override
  String get labelAimSensitivity => 'Zielempfindlichkeit';

  @override
  String get tooltipAimSensitivity =>
      'Zielempfindlichkeit für Draufsicht und Seitenansicht. 0.001 für etwa 3500 DPI, 0.003 für etwa 800 DPI.';

  @override
  String get labelAimOutputMultiplier => 'Zielausgabe-Multiplikator';

  @override
  String get tooltipAimOutputMultiplier =>
      'Direkter Multiplikator für die Fadenkreuzgeschwindigkeit nach der Normalisierung. Höher = schnelleres Fadenkreuz. Die meisten Benutzer müssen diesen Wert nicht ändern.';

  @override
  String get labelDisablePodPet => 'Pod-Streicheln deaktivieren';

  @override
  String get tooltipDisablePodPet =>
      'Deaktiviert die durch Mausbewegung ausgelöste Pod-Streichelanimation. Empfohlen.';

  @override
  String get labelDebugMenuKey => 'Taste für Debug-Menü';

  @override
  String get tooltipDebugMenuKey =>
      'Öffnet das nach Abschluss des Spiels zugängliche Debug-Menü. Normalerweise ist dafür ein Controller erforderlich – diese Belegung ermöglicht es mit der Tastatur.';

  @override
  String get labelThirdPersonMode => 'Korrektur der Drittperson-Kamera';

  @override
  String get tooltipThirdPersonMode =>
      'Direkte Mauseingabe für die Drittperson-Kamera. Gleichmäßige, unmittelbare Kamerasteuerung, die die Mausoptionen im Spiel ignoriert.';

  @override
  String get labelThirdPersonCharFollow => 'Kamera folgt dem Charakter';

  @override
  String get tooltipThirdPersonCharFollow =>
      'Behält die automatische Kameraführung des Spiels beim Bewegen bei, wie mit einem Controller.';

  @override
  String get labelThirdPersonSensX => 'Horizontale Empfindlichkeit';

  @override
  String get tooltipThirdPersonSensX =>
      'Kamerageschwindigkeit nach links und rechts. Ein negativer Wert kehrt die Richtung um.';

  @override
  String get labelThirdPersonSensY => 'Vertikale Empfindlichkeit';

  @override
  String get tooltipThirdPersonSensY =>
      'Kamerageschwindigkeit nach oben und unten. Ein negativer Wert kehrt die Richtung um.';

  @override
  String get labelAimMode => 'Pod-Zielen korrigieren';

  @override
  String get tooltipAimMode =>
      'Entfernt Begrenzung und Totzone beim Zielen mit Pod oder Mech in Draufsicht und Seitenansicht.';

  @override
  String get labelAimCrosshair => 'Fadenkreuzmodus';

  @override
  String get tooltipAimCrosshair =>
      'Zielen durch Zeigen: Der Pod zielt auf ein Fadenkreuz, das deiner Maus folgt, ähnlich wie in einem Twin-Stick-Shooter. Das Fadenkreuz besteht aus den eigenen UI-Elementen des Spiels und wirkt daher, als hätte es schon immer zu NieR:Automata gehört. Empfohlen.';

  @override
  String get labelAimCrosshairAlways => 'Fadenkreuz immer anzeigen';

  @override
  String get tooltipAimCrosshairAlways =>
      'Hält das Fadenkreuz auch sichtbar, wenn nicht geschossen wird. Aus = nur beim Schießen sichtbar.';

  @override
  String get naiomNeedsCrosshair =>
      'Aktiviere den Fadenkreuzmodus, um dies zu verwenden';

  @override
  String get labelAimSensX => 'Horizontale Zielempfindlichkeit';

  @override
  String get tooltipAimSensX =>
      'Multiplikator für die Zielgeschwindigkeit nach links und rechts. Ein negativer Wert kehrt die Richtung um.';

  @override
  String get labelAimSensY => 'Vertikale Zielempfindlichkeit';

  @override
  String get tooltipAimSensY =>
      'Multiplikator für die Zielgeschwindigkeit nach oben und unten. Ein negativer Wert kehrt die Richtung um.';

  @override
  String get labelDisableTapEvade => 'Ausweichen per Doppeltippen deaktivieren';

  @override
  String get tooltipDisableTapEvade =>
      'Verhindert das Ausweichen durch doppeltes Antippen von Bewegungstasten. Nur zusammen mit einer eigenen Ausweichtaste sinnvoll.';

  @override
  String get labelCustomCursorMenu => 'Menüzeiger';

  @override
  String get tooltipCustomCursorMenu =>
      'Benutzerdefinierter Mauszeiger für die Menüs (.cur- oder .ani-Datei). Leer = mitgelieferter Standardzeiger.';

  @override
  String get labelCustomCursorHacking => 'Hacking-Zeiger';

  @override
  String get tooltipCustomCursorHacking =>
      'Benutzerdefinierter Zeiger für das Hacking-Minispiel. Leer = derselbe Zeiger wie im Menü.';

  @override
  String get labelDisableDefaultCursor => 'Systemzeiger beibehalten';

  @override
  String get tooltipDisableDefaultCursor =>
      'Verwendet nicht den mitgelieferten Zeiger, sondern behält den normalen Windows-Zeiger bei, sofern oben keine eigene Datei ausgewählt wurde.';

  @override
  String get labelBindMoveForward => 'Vorwärts bewegen';

  @override
  String get tooltipBindMoveForward => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindMoveBackward => 'Rückwärts bewegen';

  @override
  String get tooltipBindMoveBackward => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindMoveLeft => 'Nach links bewegen';

  @override
  String get tooltipBindMoveLeft => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindMoveRight => 'Nach rechts bewegen';

  @override
  String get tooltipBindMoveRight => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindJump => 'Springen';

  @override
  String get tooltipBindJump => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindWalk => 'Gehen';

  @override
  String get tooltipBindWalk => 'Gedrückt halten, um langsam zu gehen.';

  @override
  String get labelBindAutoRun => 'Automatisch laufen';

  @override
  String get tooltipBindAutoRun =>
      'Läuft weiter, ohne dass Bewegungstasten gedrückt gehalten werden müssen.';

  @override
  String get labelBindLightAttack => 'Leichter Angriff';

  @override
  String get tooltipBindLightAttack => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindHeavyAttack => 'Schwerer Angriff';

  @override
  String get tooltipBindHeavyAttack => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindFire => 'Feuern / Pod-Sprint';

  @override
  String get tooltipBindFire =>
      'Feuert den Pod ab. Zusammen mit Springen wird der Pod-Sprint ausgeführt – auch bei aktiviertem automatischem Feuern.';

  @override
  String get labelBindProgram => 'Programm verwenden';

  @override
  String get tooltipBindProgram =>
      'Verwendet das Pod- oder Flugeinheitenprogramm.';

  @override
  String get labelBindLockOn => 'Zielerfassung';

  @override
  String get tooltipBindLockOn => 'Erfasst das aktuelle Ziel.';

  @override
  String get labelBindUse => 'Benutzen / Interagieren';

  @override
  String get tooltipBindUse => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindSelfDestruct => 'Selbstzerstörung';

  @override
  String get tooltipBindSelfDestruct => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindLight => 'Licht umschalten';

  @override
  String get tooltipBindLight => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindResetCamera => 'Kamera zurücksetzen';

  @override
  String get tooltipBindResetCamera => 'Entspricht der Belegung im Spiel.';

  @override
  String get labelBindSwitchWeapon => 'Waffensatz wechseln';

  @override
  String get tooltipBindSwitchWeapon =>
      'Wechselt zyklisch zwischen den ausgerüsteten Waffensätzen.';

  @override
  String get labelBindNextProgram => 'Nächstes Programm';

  @override
  String get tooltipBindNextProgram => 'Wählt das nächste Pod-Programm aus.';

  @override
  String get labelBindPreviousProgram => 'Vorheriges Programm';

  @override
  String get tooltipBindPreviousProgram =>
      'Wählt das vorherige Pod-Programm aus.';

  @override
  String get labelBindMenuUp => 'Menü nach oben';

  @override
  String get tooltipBindMenuUp => 'Navigiert in Menüs nach oben.';

  @override
  String get labelBindMenuDown => 'Menü nach unten';

  @override
  String get tooltipBindMenuDown => 'Navigiert in Menüs nach unten.';

  @override
  String get labelBindMenuLeft => 'Menü nach links';

  @override
  String get tooltipBindMenuLeft => 'Navigiert in Menüs nach links.';

  @override
  String get labelBindMenuRight => 'Menü nach rechts';

  @override
  String get tooltipBindMenuRight => 'Navigiert in Menüs nach rechts.';

  @override
  String get labelBindMenuOpen => 'Menü öffnen';

  @override
  String get tooltipBindMenuOpen => 'Öffnet das Systemmenü.';

  @override
  String get labelBindMenuBack => 'Im Menü zurück / schließen';

  @override
  String get tooltipBindMenuBack =>
      'Geht in Menüs zurück oder schließt sie auf der obersten Ebene.';

  @override
  String get labelBindMenuEnter => 'Menü bestätigen / Dialog überspringen';

  @override
  String get tooltipBindMenuEnter =>
      'Öffnet das ausgewählte Untermenü oder überspringt einen Dialog.';

  @override
  String get labelBindShortcutMenu => 'Schnellmenü';

  @override
  String get tooltipBindShortcutMenu => 'Öffnet das Schnellmenü.';

  @override
  String get labelBindEvade => 'Ausweichen (eigene Taste)';

  @override
  String get tooltipBindEvade =>
      'Weicht mit einer einzelnen Taste in die aktuelle Bewegungsrichtung aus – kein Doppeltippen erforderlich.';

  @override
  String get labelBindAutoFire => 'Automatisches Feuern umschalten';

  @override
  String get tooltipBindAutoFire =>
      'Schaltet dauerhaftes Pod-Feuer ein oder aus, sodass die Feuertaste nicht gehalten werden muss.';

  @override
  String get labelBindNextItem => 'Nächster Gegenstand';

  @override
  String get tooltipBindNextItem =>
      'Wechselt sofort zum nächsten Schnellgegenstand. Funktioniert unbemerkt im Hintergrund – im Spiel erscheint absichtlich kein Gegenstandsmenü.';

  @override
  String get labelBindPreviousItem => 'Vorheriger Gegenstand';

  @override
  String get tooltipBindPreviousItem =>
      'Wechselt sofort zum vorherigen Schnellgegenstand. Funktioniert unbemerkt im Hintergrund – im Spiel erscheint absichtlich kein Gegenstandsmenü.';

  @override
  String get labelBindUseItem => 'Gegenstand verwenden';

  @override
  String get tooltipBindUseItem =>
      'Verwendet den ausgewählten Schnellgegenstand sofort. Funktioniert unbemerkt im Hintergrund – im Spiel erscheint absichtlich kein Gegenstandsmenü.';

  @override
  String get labelBindThirdPersonToggle => 'Kamerakorrektur umschalten';

  @override
  String get tooltipBindThirdPersonToggle =>
      'Schaltet die Korrektur der Drittperson-Kamera während des Spielens ein oder aus.';

  @override
  String get labelBindAimToggle => 'Zielkorrektur umschalten';

  @override
  String get tooltipBindAimToggle =>
      'Schaltet die Korrektur für das Pod-Zielen während des Spielens ein oder aus.';

  @override
  String get keybindUnbound => 'Nicht belegt';

  @override
  String keybindConflict(String other) {
    return 'Wird außerdem verwendet von: $other';
  }

  @override
  String get keybindMouseNotSupported =>
      'Maustasten funktionieren für diese Aktion nicht – sie benötigt eine Tastaturtaste.';

  @override
  String get naiomResetConfirmTitle => 'NAIOM-Einstellungen zurücksetzen?';

  @override
  String get naiomResetConfirmBody =>
      'Dadurch werden alle Kamera-, Ziel-, Zeiger- und Tastenbelegungseinstellungen in diesem Reiter auf ihre Standardwerte zurückgesetzt. Bis du auf Speichern drückst, wird nichts geschrieben, sodass du die Änderungen anschließend noch verwerfen kannst. Fortfahren?';

  @override
  String get naiomControllerNote =>
      'Spielst du mit einem Controller? Diese Einstellungen sind für Maus und Tastatur ausgelegt, einige davon – insbesondere die Kamera- und Zielkorrekturen – beeinflussen jedoch auch Controller-Eingaben. Wenn du wieder mit einem Controller spielst, deaktiviere diese Einstellungen zuerst, um das ursprüngliche Gamepad-Gefühl wiederherzustellen.';

  @override
  String get cardCheatEngine => 'CHEAT ENGINE';

  @override
  String get cheatTableConvertDesc =>
      'Du hast eine Cheat-Engine-Tabelle (.CT), die nicht mit NAMS funktioniert? Repariere sie hier. Die korrigierte Kopie wird neben der Originaldatei gespeichert.';

  @override
  String get cheatTableConvertButton => 'Cheat-Tabelle reparieren ...';

  @override
  String cheatTableConvertSuccess(String file) {
    return 'Repariert! Gespeichert als $file';
  }

  @override
  String get cheatTableConvertNone =>
      'Diese Tabelle funktioniert bereits mit NAMS – nichts zu reparieren.';

  @override
  String get cheatTableConvertError =>
      'Diese Tabelle konnte nicht repariert werden. Stelle sicher, dass die Datei eine gültige .CT-Datei ist.';

  @override
  String get naiomBetaBadge => 'BETA';

  @override
  String get naiomRestartBadge => 'NEUSTART';

  @override
  String get naiomRestartTooltip =>
      'Wird nach einem Neustart des Spiels wirksam.';

  @override
  String get naiomNeedsCameraFix =>
      'Aktiviere „Kamerabeschleunigung korrigieren“, um dies zu verwenden';

  @override
  String get naiomNeedsThirdPerson =>
      'Aktiviere „Korrektur der Drittperson-Kamera“, um dies zu verwenden';

  @override
  String get naiomNeedsAimMode =>
      'Aktiviere „Pod-Zielen korrigieren“, um dies zu verwenden';

  @override
  String get naiomCrosshairOverrides =>
      'Wird bei aktiviertem Fadenkreuzmodus nicht verwendet – das Fadenkreuz besitzt eine eigene Geschwindigkeit';

  @override
  String get naiomThirdPersonRestartNote =>
      'Das Aktivieren erfordert einen Neustart des Spiels. Das Deaktivieren funktioniert während des Spielens.';

  @override
  String get naiomTapEvadeWarning =>
      'Es ist keine Ausweichtaste belegt! Wenn Ausweichen per Doppeltippen deaktiviert und keine eigene Ausweichtaste festgelegt ist, kannst du überhaupt nicht ausweichen. Belege unter „Nicht standardmäßige Aktionen“ eine Ausweichtaste.';

  @override
  String get naiomCrosshairNote =>
      'Das Fadenkreuz wird nur während des normalen Gameplays in Draufsicht oder Seitenansicht bei Mauseingabe angezeigt. Falls es an einer Stelle nicht sichtbar ist, ist das normalerweise beabsichtigt und kein Fehler.';

  @override
  String get naiomBindingsIntro =>
      'Zusätzliche Tasten neben der ursprünglichen Steuerung des Spiels – die Originaltasten funktionieren weiterhin. Änderungen gelten nach dem Speichern; ein Neustart ist nicht erforderlich.';

  @override
  String get naiomCrosshairPreviewLabel => 'Fadenkreuzmodus im Spiel';

  @override
  String get naiomCursorPick => 'Datei auswählen ...';

  @override
  String get naiomCursorClear => 'Entfernen';

  @override
  String get naiomCursorInvalid =>
      'Keine gültige Zeigerdatei – benötigt eine echte .cur- oder .ani-Datei';

  @override
  String get naiomLiveBadge => 'LIVE';

  @override
  String get naiomLiveTooltip =>
      'Gilt nach dem Speichern – kein Neustart des Spiels erforderlich.';

  @override
  String get labelPreloadMaxDimensionShort => 'Maximale Vorladedimension';

  @override
  String get tooltipPreloadMaxDimensionShort =>
      '0 = deaktiviert (reines Streaming), 2048 = Standard, 4096 = 4K-Texturen, 16384 = alles.';

  @override
  String get labelPreloadAllTexturesShort => 'Alle Texturen vorladen';

  @override
  String get tooltipPreloadAllTexturesShort =>
      'Lädt ALLE Texturen vor. Keine Ruckler, benötigt aber mindestens 32 GB RAM.';

  @override
  String get labelVramBudget => 'VRAM-Budget (MB)';

  @override
  String get tooltipVramBudget =>
      'Wie viel GPU-Speicher das Textur-Mod-System verwenden darf. Wähle einen Wert für eine feste Obergrenze – 8192 bedeutet beispielsweise „niemals mehr als 8 GB für gemoddete Texturen verwenden“, 16384 bedeutet „niemals mehr als 16 GB“. Automatisch (empfohlen) überspringt die Obergrenze und verwendet den tatsächlich verfügbaren Speicher deiner GPU.';

  @override
  String get labelStreamingEnabled => 'Laden im Hintergrund';

  @override
  String get tooltipStreamingEnabled =>
      'Lädt Texturen während des Spielens im Hintergrund. Verhindert Einfrieren und Ruckler beim Laden neuer Gebiete. Nur bei Problemen deaktivieren – ohne diese Option kann das Spiel beim Laden neuer Texturen kurz einfrieren.';

  @override
  String get labelLoadOnlyRelevant => 'Nur relevante laden';

  @override
  String get tooltipLoadOnlyRelevant =>
      'Lädt bei sehr großen Paketen (über 400 Dateien) nur Texturen, die einer kuratierten Prioritätenliste entsprechen. Das spart VRAM und Ladezeit. Kleine Pakete (Kleidung, Waffen) werden immer vollständig geladen. Aktiviere dies, wenn du ein riesiges Paket verwendest und Speicher sparen möchtest.';

  @override
  String get tutorialDropTextures =>
      'Ziehe Textur-Mods hierher, um sie zu installieren. ZIP-Dateien werden automatisch entpackt.';

  @override
  String get tutorialLoadOrder =>
      'Wenn sich Mods überschneiden, kannst du sie durch Ziehen neu anordnen. Oben = höchste Priorität.';

  @override
  String get textureOverlapLabel => 'ÜBERSCHNEIDUNG';

  @override
  String tooltipTextureOverlap(String mods) {
    return 'Ändert dieselben Texturen wie: $mods. Die Mod, die in der Liste weiter oben und näher an HÖCHSTE steht, ist im Spiel sichtbar.';
  }

  @override
  String get tooltipFolderNotFound =>
      'Ordner in nams/inject/textures/ nicht gefunden';

  @override
  String get priorityHighest => 'HÖCHSTE';

  @override
  String get priorityMedium => 'MITTLERE';

  @override
  String get priorityLowest => 'NIEDRIGSTE';

  @override
  String nameOutfitTitle(String character) {
    return 'Dieses Outfit benennen ($character)';
  }

  @override
  String get outfitNameHint => 'Outfit-Name';

  @override
  String installedTextureCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Texturdateien',
      one: 'Texturdatei',
    );
    return '$count $_temp0 installiert';
  }

  @override
  String installationFailed(String error) {
    return 'Installation fehlgeschlagen: $error';
  }

  @override
  String removedItem(String name) {
    return '„$name“ entfernt';
  }

  @override
  String get tutorialStarIcon =>
      'Klicke auf den Stern, um ein Standard-Outfit festzulegen, das beim Spielstart geladen wird.';

  @override
  String installedOutfitsCount(int count) {
    return 'INSTALLIERTE OUTFITS ($count)';
  }

  @override
  String get tooltipDlcDetected =>
      'DLC erkannt (data100.cpk). Modelldateien verwenden die DLC-Namensgebung (pl000d).';

  @override
  String get tooltipNoDlcDetected =>
      'Kein DLC erkannt. Modelldateien werden auf die Namensgebung ohne DLC umbenannt (pl0000).';

  @override
  String installConfirmMod(String name, String character) {
    return '„$name“ ($character) installieren?';
  }

  @override
  String installedOutfit(String name, String character) {
    return '„$name“ ($character) installiert';
  }

  @override
  String get crossInstallTextures =>
      'Diese Mod enthält außerdem Texturdateien. Nach nams/inject/textures/ installieren?';

  @override
  String alsoInstalledTextures(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Texturdateien',
      one: 'Texturdatei',
    );
    return 'Außerdem $count $_temp0 installiert';
  }

  @override
  String get clearedAllStartupOutfits => 'Alle Start-Outfits entfernt';

  @override
  String get clearedStartupOutfit => 'Start-Outfit entfernt';

  @override
  String setStartupOutfit(String name) {
    return '„$name“ als Start-Outfit festgelegt';
  }

  @override
  String get tutorialDropCutscenes =>
      'Lege Archive mit Cutscene-Mods hier ab. Unterstützt .zip, .7z und .rar.';

  @override
  String get tutorialInstalledCutscenes =>
      'Deine installierten Cutscene-Mods. Benutzerdefinierte Cutscenes werden von hier statt aus data/movie/ geladen.';

  @override
  String get selectCutsceneModFolder => 'Cutscene-Mod-Ordner auswählen';

  @override
  String cutsceneNamingHint(int max) {
    return 'Höchstens $max Zeichen. Dieser Text wird zum Ordnernamen in nams/cutscenes/.';
  }

  @override
  String cutsceneNameTooLong(int max) {
    return 'Der Name darf höchstens $max Zeichen lang sein.';
  }

  @override
  String get preparingInstall => 'Wird vorbereitet ...';

  @override
  String installedCutsceneMod(String name, int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Dateien',
      one: 'Datei',
    );
    return '„$name“ installiert ($count USM-$_temp0)';
  }

  @override
  String deleteCutsceneConfirm(String name) {
    return '„$name“ und alle zugehörigen Dateien löschen?';
  }

  @override
  String installedCutsceneModsCount(int count) {
    return 'INSTALLIERTE ZWISCHENSEQUENZ-MODS ($count)';
  }

  @override
  String cutsceneUsmCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Dateien',
      one: 'Datei',
    );
    return '$count USM-$_temp0';
  }

  @override
  String cutsceneMatchCount(int matching, int total) {
    return '$matching/$total stimmen mit Originalen überein';
  }

  @override
  String tooltipMissingOriginals(String files) {
    return 'Dateien ohne Übereinstimmung mit Originalen: $files';
  }

  @override
  String get cutsceneMismatchHint =>
      'Einige Dateien entsprechen keinen ursprünglichen Namen von Cutscenes. Bei fehlenden Dateien wird auf die ursprünglichen Cutscenes zurückgegriffen.';

  @override
  String cutsceneMigrationBannerBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Cutscene-Dateien',
      one: 'Cutscene-Datei',
    );
    return '$count benutzerdefinierte $_temp0 wurden direkt in data/movie/ gefunden. Diese überschreiben die Originale dauerhaft. Installiere Cutscene-Mods künftig stattdessen hier – kann eine benutzerdefinierte Datei nicht geladen werden, wird ersatzweise das Original abgespielt.';
  }

  @override
  String hardwareInfoLabel(int ram, String gpu) {
    return '$ram GB RAM | $gpu';
  }

  @override
  String hardwareInfoRamOnly(int ram) {
    return '$ram GB RAM';
  }

  @override
  String texturesScanResult(int count, int sizeMB, int maxDim) {
    return '$count Texturdateien, insgesamt $sizeMB MB, maximal $maxDim px';
  }

  @override
  String recommendedSettings(int dim, String allLabel) {
    return 'Empfohlen: Vorladen $dim, alle vorladen $allLabel';
  }

  @override
  String get applyRecommended => 'ANWENDEN';

  @override
  String get settingsMatchRecommended =>
      'Deine Einstellungen entsprechen der Empfehlung';

  @override
  String get reasonNoTextures => 'Keine Texturen installiert';

  @override
  String reasonFitsInMemory(int ramGB, int textureSizeMB) {
    return '$ramGB GB RAM, $textureSizeMB MB Texturen – passen in den Speicher; lade alles vor, um Ruckler vollständig zu vermeiden';
  }

  @override
  String reasonExceedsRam(int ramGB, int estimatedGB) {
    return '$ramGB GB RAM, geschätzter Texturspeicherbedarf von etwa $estimatedGB GB – das Vorladen aller Texturen lässt dein System einfrieren oder abstürzen. Verwende eine niedrige Vorladedimension oder entferne einige Texturpakete.';
  }

  @override
  String reasonTooLargeForAll(int ramGB, int textureSizeMB) {
    return '$ramGB GB RAM, $textureSizeMB MB Texturen – zu groß, um alles vorzuladen; Texturen bis 4K bei Bedarf vorladen';
  }

  @override
  String reasonMediumRam(int ramGB) {
    return '$ramGB GB RAM – Texturen bis 4K vorladen, größere bei Bedarf laden';
  }

  @override
  String reasonLowRam(int ramGB) {
    return '$ramGB GB RAM – nur kleine Texturen vorladen, um Speicher zu sparen';
  }

  @override
  String get analyzingHardware => 'Hardware und Texturen werden analysiert ...';

  @override
  String texturesBloatWarning(int total, int relevant, int excess) {
    return 'Diese Mod enthält $total Texturen, aber nur $relevant davon sind visuell relevant (basierend auf der kuratierten Referenzmenge von GPUnity). Die verbleibenden $excess Texturen verlängern die Ladezeit und erhöhen den RAM-Verbrauch, ohne sichtbaren Nutzen.';
  }

  @override
  String cleanUnneededTextures(int count) {
    return '$count UNNÖTIGE ENTFERNEN';
  }

  @override
  String cleanedTextures(int deleted, int kept) {
    return '$deleted unnötige Texturen entfernt, $kept beibehalten';
  }

  @override
  String get confirmCleanTextures => 'Unnötige Texturen entfernen?';

  @override
  String confirmCleanTexturesBody(int count, String sizeMB) {
    return 'Dadurch werden $count Texturdateien ($sizeMB MB) dauerhaft aus diesem Mod-Ordner gelöscht.';
  }

  @override
  String get confirmCleanTexturesDetail1 =>
      'Nur Texturen, die der kuratierten Referenzmenge von GPUnity entsprechen, werden beibehalten';

  @override
  String get confirmCleanTexturesDetail2 =>
      'Dies betrifft nur den ausgewählten Mod-Ordner, nicht andere installierte Mods';

  @override
  String get confirmCleanTexturesDetail3 =>
      'Dies kann nicht rückgängig gemacht werden – lege die Mod erneut ab, um entfernte Dateien wiederherzustellen';

  @override
  String get texturesBloatDialogTitle => 'Unnötige Texturen erkannt';

  @override
  String texturesBloatDialogBody(int total, int relevant, int excess) {
    return 'Dieses Texturpaket enthält $total Dateien, aber nur $relevant entsprechen der kuratierten Referenzmenge von GPUnity. Die verbleibenden $excess Texturen sind wahrscheinlich unnötig.';
  }

  @override
  String get texturesBloatPoint1 =>
      'Deutlich längerer Spielstart – die Engine lädt beim Start jede Textur';

  @override
  String get texturesBloatPoint2 =>
      'Zufällige Ruckler und Bildrateneinbrüche – das Spiel streamt Texturen ohne sichtbaren Nutzen';

  @override
  String get texturesBloatPoint3 =>
      'Hoher RAM-Verbrauch – mehrere GB können für unsichtbare Texturen verschwendet werden';

  @override
  String get texturesBloatPoint4 =>
      'Einige KI-hochskalierten Texturen können Artefakte oder Beschädigungen enthalten';

  @override
  String get texturesBloatPoint5 =>
      'Kaum sichtbarer Unterschied – meist handelt es sich um winzige UI-Elemente, Partikeleffekte usw.';

  @override
  String get texturesBloatRecommendation =>
      'Das Entfernen ist sicher und wird für eine bessere Leistung empfohlen.';

  @override
  String get texturesBloatKeepAll => 'Alle behalten';

  @override
  String texturesBloatRemoveUnneeded(int count) {
    return 'Unnötige entfernen ($count)';
  }

  @override
  String get texturesProgressExtracting => 'Archiv wird entpackt ...';

  @override
  String get texturesProgressCopying => 'Dateien werden kopiert ...';

  @override
  String get texturesProgressAnalyzing => 'Texturen werden analysiert ...';

  @override
  String get texturesAnalyzingSetup =>
      'Deine Texturkonfiguration wird analysiert ...';

  @override
  String get texturesBusyMessage =>
      'Bitte warten – Texturen werden installiert';

  @override
  String texturesInstallProgress(
    int files,
    int totalFiles,
    int mb,
    int totalMb,
  ) {
    return 'Installation: $files/$totalFiles Dateien – $mb/$totalMb MB';
  }

  @override
  String texturesAnalyzeProgress(int scanned, int total) {
    return 'Analyse: $scanned/$total Texturen';
  }

  @override
  String get cleaningTextures => 'Unnötige Texturen werden entfernt ...';

  @override
  String get textureMergeTitle =>
      'Zu vorhandener Mod hinzufügen oder neu installieren?';

  @override
  String get textureMergeDescription =>
      'Du hast bereits Textur-Mods installiert. Möchtest du diese Dateien zu einer vorhandenen Mod hinzufügen oder als neue Mod installieren?';

  @override
  String get textureMergeNewMod => 'Als neue Mod installieren';

  @override
  String textureMergeAddTo(String name) {
    return 'Hinzufügen zu: $name';
  }

  @override
  String get cutsceneMergeTitle =>
      'Zu vorhandener Mod hinzufügen oder neu installieren?';

  @override
  String get cutsceneMergeDescription =>
      'Du hast bereits Cutscene-Mods installiert. Mehrteilige Cutscene-Pakete sollten in derselben Mod zusammengeführt werden.';

  @override
  String get cutsceneMergeNewMod => 'Als neue Mod installieren';

  @override
  String cutsceneMergeAddTo(String name) {
    return 'Hinzufügen zu: $name';
  }

  @override
  String get headerMods => 'MODS';

  @override
  String cutsceneBundledWith(String modId) {
    return 'Gebündelt mit $modId';
  }

  @override
  String get cutsceneStatusHd => 'HD';

  @override
  String get cutsceneStatusHdTooltip =>
      '[cutscene] hd_cutscenes in nams.toml – muss true sein, damit HD-Cutscene-Mods geladen werden.';

  @override
  String get cutsceneStatusH264 => 'H264';

  @override
  String get cutsceneStatusH264Tooltip =>
      '[cutscene] enable_h264 in nams.toml – muss true sein, damit H264-codierte Cutscenes abgespielt werden.';

  @override
  String get modIntroTitle =>
      'Unterstützt durch NAMS – dein data/-Ordner bleibt unverändert';

  @override
  String get modIntroBody =>
      'NAMS lädt Mods aus nams/mods/ über ein virtuelles Dateisystem zusätzlich zu den ursprünglichen Spieldaten. Dadurch wird niemals etwas nach data/ kopiert oder dort überschrieben. Mods lassen sich jederzeit ohne Neuinstallation ein- oder ausschalten, mehrere Outfits für denselben Charakter können nebeneinander bestehen, und beim Deinstallieren einer Mod wird lediglich ihr Ordner entfernt – das ursprüngliche Spiel bleibt darunter stets unverändert.';

  @override
  String get modListEmpty => 'Keine Mods installiert';

  @override
  String get modListEmptyHint =>
      'Lege einen Mod-Ordner oder ein Archiv im Feld oben ab, um ihn zu installieren.';

  @override
  String get modSearchPlaceholder => 'Mods durchsuchen …';

  @override
  String get modFilterAll => 'Alle';

  @override
  String get modCollapseAll => 'Alle Gruppen einklappen';

  @override
  String get modExpandAll => 'Alle Gruppen ausklappen';

  @override
  String get modBulkInstall => 'Mehrere Mods aus Ordner installieren';

  @override
  String modBulkInstallBusy(int done, int total, String name) {
    return 'Installation $done von $total: $name';
  }

  @override
  String get modBulkInstallScanning =>
      'Ordner wird nach Mod-Archiven durchsucht …';

  @override
  String get modBulkInstallNone =>
      'In diesem Ordner wurden keine Mod-Archive (.zip / .7z / .rar) gefunden.';

  @override
  String modBulkInstallDone(int installed, int total) {
    return '$installed von $total Mods installiert.';
  }

  @override
  String get modLooseInstall => 'Lose Dateien aus Ordner installieren';

  @override
  String get modLooseInstallScanning =>
      'Ordner wird nach losen Spieldateien durchsucht…';

  @override
  String get modLooseInstallNone =>
      'Keine losen Spieldateien (.dat / .dtt) in diesem Ordner gefunden.';

  @override
  String modLooseInstallBusy(int count) {
    return '$count lose Dateien werden installiert…';
  }

  @override
  String modLooseInstallProgress(int done, int total) {
    return '$done von $total Dateien werden kopiert…';
  }

  @override
  String get modLooseInstallFinalizing =>
      'Dateien werden in den Mod eingefügt…';

  @override
  String modLooseInstallDone(int count, String id) {
    return '$count lose Dateien in $id installiert.';
  }

  @override
  String get modGroup2b => '2B-OUTFITS';

  @override
  String get modGroup9s => '9S-OUTFITS';

  @override
  String get modGroupA2 => 'A2-OUTFITS';

  @override
  String get modGroupOtherOutfits => 'ANDERE OUTFITS';

  @override
  String get modGroupWeapons => 'WAFFEN';

  @override
  String get modGroupAccessories => 'ACCESSOIRES';

  @override
  String get modGroupItems => 'GEGENSTÄNDE';

  @override
  String get modGroupEnemies => 'GEGNER';

  @override
  String get modGroupWorldProps => 'WELTOBJEKTE';

  @override
  String get modGroupModelVariants => 'MODELLVARIANTEN';

  @override
  String get modGroupMaps => 'KARTEN / SCHAUPLÄTZE';

  @override
  String get modGroupUi => 'UI / SCHRIFTEN';

  @override
  String get modGroupMisc => 'SONSTIGE TEXTUREN';

  @override
  String get modGroupArchives => 'CPK-ARCHIVE';

  @override
  String get modGroupEffects => 'EFFEKTE';

  @override
  String get modGroupScripting => 'SKRIPTE';

  @override
  String get modGroupLocalization => 'TEXT UND LOKALISIERUNG';

  @override
  String get modGroupCutscenes => 'ZWISCHENSEQUENZEN';

  @override
  String get modGroupAudio => 'AUDIO';

  @override
  String get modGroupTextures => 'TEXTUREN';

  @override
  String get modGroupNative => 'NATIVE MODS';

  @override
  String get modGroupOther => 'SONSTIGES';

  @override
  String get modGroupMixed => 'GEMISCHTE INHALTE';

  @override
  String get modGroupWax => 'WAX COMPACT';

  @override
  String get modGroupMultiHint =>
      'Diese Mod ersetzt Modelle für mehrere Charaktere und wird daher unter jedem von ihnen aufgeführt.';

  @override
  String get modGroupMixedHint =>
      'Diese Mod ändert mehrere Arten von Inhalten gleichzeitig. Klicke sie an, um alles zu sehen, was sie enthält, und welche Kategorien sie betrifft.';

  @override
  String get modRename => 'Umbenennen';

  @override
  String get modRenameDialogTitle => 'Mod umbenennen';

  @override
  String get modRenameReset => 'Auf ursprünglichen Namen zurücksetzen';

  @override
  String get dropModHere => 'Mod hier ablegen';

  @override
  String get dropModHereHint => 'oder zum Durchsuchen klicken';

  @override
  String get modKindNative => 'NATIV';

  @override
  String get modKindNativeTooltip =>
      'NAMS-Mod mit einem entities/-Ordner. Definiert neue Gegenstände, Waffen, Outfits, Accessoires, Quests usw. über TOML-Pakete.';

  @override
  String get modKindData => 'DATEN';

  @override
  String get modKindDataTooltip =>
      'Das klassische Mod-Format – dieselben Dateien, die normalerweise nach NieRAutomata/data/ gehören würden, werden stattdessen unter nams/mods/ verwaltet, sodass der ursprüngliche data-Ordner sauber bleibt';

  @override
  String get textureOutfitLinkedTitle => 'Outfit-gebundene Texturen';

  @override
  String get textureOutfitLinkedSubtitle =>
      'Diese Texturen befinden sich im Ordner ihrer Mod und werden nur geladen, solange dieses Outfit ausgerüstet ist. NAMS wechselt sie sofort, wenn du im Spiel das Outfit änderst.';

  @override
  String textureOutfitLinkedEntry(int count) {
    return '$count Texturen – nur mit diesem Outfit aktiv';
  }

  @override
  String get modKindTexture => 'TEXTUREN';

  @override
  String get modKindTextureTooltip =>
      'Ein Texturpaket. Seine .dds-Dateien wurden nach nams/inject/textures/ installiert und werden im Reiter Texturen verwaltet.';

  @override
  String get modKindUnknown => 'UNBEKANNT';

  @override
  String get modKindUnknownTooltip =>
      'Der Launcher konnte diesen Ordner nicht als gültige Mod erkennen.';

  @override
  String get modCompatChip => 'wax-kompatibel';

  @override
  String get modCompatChipTooltip =>
      ' NAMS liest diese zur Kompatibilität mit vorhandenen wax-Mods ebenfalls ein.';

  @override
  String get modDataChip => '+Daten';

  @override
  String get modDataChipTooltip =>
      'Enthält zusätzlich zu den Metadaten eine data/-Überlagerung. Modelle, Texturen, Töne usw. befinden sich hier.';

  @override
  String get modDetailNoSelection =>
      'Wähle eine Mod aus, um Details anzuzeigen';

  @override
  String get modAuthor => 'Autor';

  @override
  String get modVersion => 'Version';

  @override
  String get modRootPath => 'Pfad';

  @override
  String get modNativeBundles => 'Native Pakete';

  @override
  String get modDataContent => 'Dateninhalt';

  @override
  String get modDataPlayerModels => 'Spielermodelle';

  @override
  String get modRequiresLabel => 'Erfordert';

  @override
  String get modRequiresPluginsLabel => 'Erfordert Plugins';

  @override
  String get modRequiresMissing => 'fehlt';

  @override
  String get modConflictsLabel => 'Konflikte';

  @override
  String get modLoadOrderHint =>
      'Diese Mods ersetzen dieselben Dateien. Ziehe sie zum Neuordnen – die oberste gewinnt.';

  @override
  String get modConflictKeep => 'DIESE BEHALTEN';

  @override
  String get modConflictResolve => 'LÖSEN';

  @override
  String get modConflictDialogTitle => 'Welche Mod soll Vorrang haben?';

  @override
  String modConflictKeepTooltip(String id) {
    return '$id beibehalten und die anderen deaktivieren';
  }

  @override
  String modConflictPickBody(int mods, int files) {
    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$files Dateien',
      one: 'Datei',
    );
    return '$mods aktivierte Mods ersetzen dieselben $_temp0. Wähle die Mod aus, die beibehalten werden soll – die anderen werden deaktiviert.';
  }

  @override
  String modConflictOverlapFile(String otherId, String file) {
    return '$otherId enthält ebenfalls $file';
  }

  @override
  String get modOpenFolder => 'Ordner öffnen';

  @override
  String get modEnable => 'Aktivieren';

  @override
  String get modDisable => 'Deaktivieren';

  @override
  String get modDisabled => 'Deaktiviert';

  @override
  String get modDisabledTooltip =>
      'Diese Mod ist deaktiviert. NAMS lädt sie beim nächsten Spielstart nicht. Aktiviere sie, um sie wieder zu laden – Löschen und Neuinstallieren ist nicht erforderlich.';

  @override
  String get modEnableTooltip =>
      'Die Mod wird von NAMS geladen. Klicke, um sie zu deaktivieren, ohne die Dateien zu entfernen.';

  @override
  String get modDefaultTooltip =>
      'Ab Spielstart aktiv, als lägen ihre Dateien in NieRAutomata/data. Klicke, um sie auszuschalten.';

  @override
  String get modSetDefaultTooltip =>
      'Macht diese Mod ab Spielstart aktiv, ohne etwas nach NieRAutomata/data zu kopieren.';

  @override
  String get modSetDefaultOutfitTooltip =>
      'Trage dies ab Spielstart, ohne etwas nach NieRAutomata/data zu kopieren. Ersetzt das derzeitige Standard-Outfit – es kann nur eines geben.';

  @override
  String get modDefaultChip => 'STANDARD';

  @override
  String get modDefaultKindOutfitBare => 'Outfit';

  @override
  String get modDefaultKindOutfitConfig => 'Outfit + Konfiguration';

  @override
  String get modDefaultKindOutfitAnimation => 'Animation';

  @override
  String get modDefaultKindOutfitBareTooltip =>
      'Ersetzt die Modelldateien direkt. Es kann jeweils nur ein Outfit das Standard-Outfit sein.';

  @override
  String get modDefaultKindOutfitConfigTooltip =>
      'Diese Mod enthält eine Outfit-Konfiguration, sodass ihre Mesh-Regeln und Effekte mitgeladen werden. Es kann jeweils nur ein Outfit das Standard-Outfit sein.';

  @override
  String get modDefaultKindOutfitAnimationTooltip =>
      'Animationsdaten, kein Outfit. Bleibt unter jedem getragenen Outfit aktiv.';

  @override
  String get modDefaultReplaceTitle => 'Standard ersetzen?';

  @override
  String modDefaultReplaceBody(String model, String current, String next) {
    return '$model wird derzeit ab Spielstart durch „$current“ getragen.\n\nWenn „$next“ zum Standard gemacht wird, wird dies entfernt, da jeweils nur eine Mod einen Charakter einkleiden kann.';
  }

  @override
  String get modDefaultReplaceConfirm => 'Ersetzen';

  @override
  String get modDefaultOutfitAuto => 'Standard-Outfit';

  @override
  String get modDefaultOutfitPickTooltip =>
      'Diese Mod enthält mehrere Outfits. Wähle das Outfit, das du ab Spielstart tragen möchtest. „Standard-Outfit“ ist das Outfit, das ohne Gegenstand getragen wird.';

  @override
  String modDefaultRowTooltip(String files) {
    return 'Ab Spielstart aktiv: $files';
  }

  @override
  String get modDisableNotice => 'Deaktiviert – gilt beim nächsten Spielstart.';

  @override
  String get modEnableNotice => 'Aktiviert – gilt beim nächsten Spielstart.';

  @override
  String get modUninstall => 'Deinstallieren';

  @override
  String get modUninstallConfirmTitle => 'Mod deinstallieren?';

  @override
  String modUninstallConfirmBody(String id) {
    return 'Dadurch wird der Mod-Ordner „$id“ dauerhaft gelöscht.';
  }

  @override
  String get modProfileLabel => 'Profil';

  @override
  String get modProfileNewButton => 'Neu';

  @override
  String get modProfileRenameButton => 'Umbenennen';

  @override
  String get modProfileDeleteButton => 'Löschen';

  @override
  String get modProfileNewDialogTitle => 'Neues Profil';

  @override
  String get modProfileNewDialogHint => 'Profilname (Buchstaben, Zahlen, _ -)';

  @override
  String get modProfileRenameDialogTitle => 'Profil umbenennen';

  @override
  String get modProfileDeleteDialogTitle => 'Profil löschen?';

  @override
  String modProfileDeleteDialogBody(String name) {
    return 'Entfernt den Ordner mods_profile_$name/ und alle gebündelten Texturpakete dieses Profils dauerhaft. Dies kann nicht rückgängig gemacht werden.';
  }

  @override
  String get modProfileDeleteConfirm => 'Löschen';

  @override
  String get modProfileErrorNameEmpty => 'Name erforderlich';

  @override
  String get modProfileErrorNameInvalid =>
      'Verwende nur Buchstaben, Zahlen, _ oder -';

  @override
  String get modProfileErrorNameCollision =>
      'Ein Profil mit diesem Namen ist bereits vorhanden';

  @override
  String get modProfileErrorDeleteActive =>
      'Wechsle zu einem anderen Profil, bevor du dieses löschst';

  @override
  String get modProfileErrorDeleteLast =>
      'Das einzige verbleibende Profil kann nicht gelöscht werden';

  @override
  String get modProfileErrorTargetMissing =>
      'Der Profilordner fehlt auf dem Datenträger';

  @override
  String get modProfileErrorFsBusy =>
      'Das Dateisystem ist beschäftigt. Schließe das Spiel und versuche es erneut.';

  @override
  String get modProfileLockedRunning =>
      'Beende das Spiel, bevor du das Profil wechselst.';

  @override
  String get modProfileEmptyHint =>
      'Leeres Profil – lege eine Mod ab, um zu beginnen';

  @override
  String modProfileSwitchedToast(String name) {
    return 'Zu Profil $name gewechselt';
  }

  @override
  String modProfileCreatedToast(String name) {
    return 'Profil $name erstellt und ausgewählt';
  }

  @override
  String modProfileDeletedToast(String name) {
    return 'Profil $name gelöscht';
  }

  @override
  String modProfileRenamedToast(String name) {
    return 'Profil in $name umbenannt';
  }

  @override
  String get modInstallNeedsName => 'Diese Mod benennen';

  @override
  String modInstallExistsPickAnother(String id) {
    return 'Eine Mod namens „$id“ ist bereits vorhanden. Wähle einen anderen Namen.';
  }

  @override
  String get modInspectBusy => 'Mod wird geprüft …';

  @override
  String get modInstallBusy => 'Mod wird installiert …';

  @override
  String get modVariantDialogTitle => 'Installationsinhalte auswählen';

  @override
  String get modVariantDialogSubtitle =>
      'Dieses Archiv enthält mehrere Optionen. Wähle die gewünschten aus.';

  @override
  String get modOutfitChoiceDialogTitle => 'Installationsinhalte auswählen';

  @override
  String get modOutfitChoiceDialogSubtitle =>
      'Markiere alles, was du möchtest. Jeder Eintrag wird als eigene Mod installiert. Enthält ein Outfit Texturen, werden diese mitinstalliert; im Texturen-Reiter kannst du später genauer festlegen, welche Sätze es verwendet.';

  @override
  String get variantCatPlayer => 'Outfits';

  @override
  String get variantCatWeapon => 'Waffen';

  @override
  String get variantCatAccessory => 'Accessoires';

  @override
  String get variantCatEnemy => 'Gegner';

  @override
  String get variantCatModelVariant => 'Modellvarianten';

  @override
  String get variantCatItem => 'Gegenstände';

  @override
  String get variantCatWorldProp => 'Weltobjekte';

  @override
  String get variantCatMap => 'Karten';

  @override
  String get variantCatEffects => 'Effekte';

  @override
  String get variantCatScripting => 'Skripte';

  @override
  String get variantCatLocalization => 'Lokalisierung';

  @override
  String get variantCatUi => 'Benutzeroberfläche';

  @override
  String get variantCatCutscenes => 'Cutscenes';

  @override
  String get variantCatAudio => 'Audio';

  @override
  String get variantCatMisc => 'Sonstiges';

  @override
  String get variantCatOther => 'Andere';

  @override
  String get variantPickOneSuffix => 'eine auswählen';

  @override
  String get modVariantSelectAll => 'Alle auswählen';

  @override
  String get modVariantSelectNone => 'Keine';

  @override
  String get modVariantInstall => 'Installieren';

  @override
  String modVariantInstallSelected(int count) {
    return '$count installieren';
  }

  @override
  String get modVariantTexture => 'Texturen';

  @override
  String modVariantInstalledToast(int count) {
    return '$count Option(en) installiert';
  }

  @override
  String get modUninstallBusy => 'Mod wird deinstalliert …';

  @override
  String modInstalled(String id) {
    return 'Installiert: $id';
  }

  @override
  String modInstallFailed(String reason) {
    return 'Installation fehlgeschlagen: $reason';
  }

  @override
  String get modInstallReasonUnknownDrop =>
      'Unbekannte Ablage – der Ordner entspricht keiner unterstützten Mod-Struktur.';

  @override
  String get modInstallReasonUnsupportedNasa =>
      'Dies ist eine NASA-Mod (enthält sadfutago.cpk), die von diesem Launcher nicht unterstützt wird.';

  @override
  String get modInstallReasonInvalidMixed =>
      'Ungültige Struktur – eine Mod darf entities- und wax-artige Konfigurationen nicht mischen.';

  @override
  String get modInstallReasonNativeEmpty =>
      'In entities/ wurden keine Entity-Dateien gefunden.';

  @override
  String get modInstallReasonDataEmpty => 'Keine erkennbaren Inhalte gefunden.';

  @override
  String get modInstallReasonArchiveExtractFailed =>
      'Das Archiv konnte nicht entpackt werden.';

  @override
  String get modInstallReasonMoveFailed =>
      'Die Dateien konnten nicht nach nams/mods/ verschoben werden.';

  @override
  String get modInstallReasonTextureOnly =>
      'Dies ist ein Texturpaket (nur .dds-Dateien). Installiere es stattdessen über den Texturen-Reiter.';

  @override
  String modUninstalled(String id) {
    return 'Entfernt: $id';
  }

  @override
  String modCountFiles(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count Dateien',
      one: '1 Datei',
    );
    return '$_temp0';
  }
}
