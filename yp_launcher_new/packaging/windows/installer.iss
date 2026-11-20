#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\..\stage\YoRHa Protocol Launcher"
#endif
#define AppName "YoRHa Protocol Launcher"
#define AppExe "YoRHa-Protocol-Launcher.exe"

[Setup]
AppId={{8B3F2C6E-9D41-4A7B-A5D0-3E6F1C9B7A24}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=Native Application Modding System
AppPublisherURL=https://github.com/Vluurie/yp_launcher
AppSupportURL=https://github.com/Vluurie/yp_launcher/issues
AppUpdatesURL=https://github.com/Vluurie/yp_launcher/releases
AppCopyright=Copyright (c) 2026 Vluurie
AppMutex=YoRHaProtocolLauncher
MinVersion=10.0.17763
DefaultDirName={localappdata}\Programs\{#AppName}
LicenseFile=..\..\..\LICENSE
PrivilegesRequired=lowest
DisableProgramGroupPage=yes
OutputBaseFilename=YoRHa-Protocol-Launcher-{#AppVersion}-windows-x86_64-setup
SetupIconFile=..\..\assets\icon\app_icon.ico
UninstallDisplayIcon={app}\{#AppExe}
VersionInfoVersion={#AppVersion}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
SetupLogging=yes
WizardImageFile=..\..\..\windows\wizard.png
WizardSmallImageFile=..\..\..\windows\wizard_small.png
DisableWelcomePage=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "thai"; MessagesFile: "compiler:Languages\Thai.isl"
#ifdef ChineseIsl
Name: "chinesesimplified"; MessagesFile: "{#ChineseIsl}"
#endif

[Tasks]
Name: desktopicon; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked

[InstallDelete]
Type: files; Name: "{app}\assets\bins\7z.exe"
Type: files; Name: "{app}\assets\bins\7z.dll"
Type: files; Name: "{app}\assets\bins\7z-License.txt"
Type: files; Name: "{app}\assets\bins\mrbc.exe"
Type: files; Name: "{app}\YoRHa Protocol Launcher.exe"
Type: files; Name: "{app}\flutter_windows.dll"
Type: files; Name: "{app}\dartjni.dll"
Type: files; Name: "{app}\desktop_drop_plugin.dll"
Type: files; Name: "{app}\screen_retriever_windows_plugin.dll"
Type: files; Name: "{app}\url_launcher_windows_plugin.dll"
Type: files; Name: "{app}\window_manager_plugin.dll"
Type: files; Name: "{app}\yp_formats.dll"
Type: files; Name: "{app}\native_assets.json"

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
#ifdef VCRedistDir
Source: "{#VCRedistDir}\vc_redist.x64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: VCRedistNeeded
#endif

[Icons]
Name: "{userprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{userdesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
#ifdef VCRedistDir
Filename: "{tmp}\vc_redist.x64.exe"; Parameters: "/install /quiet /norestart"; StatusMsg: "Installing Microsoft Visual C++ Runtime..."; Flags: shellexec waituntilterminated; Check: VCRedistNeeded
#endif
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}"

[Code]
#ifdef VCRedistDir
function VCRedistNeeded: Boolean;
var
  Bld: Cardinal;
begin
  Result := True;
  if RegQueryDWordValue(HKLM, 'SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64', 'Bld', Bld) then
    Result := Bld < 29913;
end;
#endif

procedure MoveFlutterRuntime;
var
  App, OldBins, NewBins: String;
begin
  App := ExpandConstant('{app}');
  if not FileExists(App + '\flutter_windows.dll') then
    exit;
  OldBins := App + '\data\flutter_assets\assets\bins';
  NewBins := App + '\assets\bins';
  if DirExists(OldBins) then
    if not DirExists(NewBins) then
      if ForceDirectories(App + '\assets') then
        RenameFile(OldBins, NewBins);
  if DirExists(OldBins) then
    exit;
  DelTree(App + '\data\flutter_assets', True, True, True);
  DeleteFile(App + '\data\app.so');
  DeleteFile(App + '\data\icudtl.dat');
  RemoveDir(App + '\data');
end;

procedure SeedLanguage;
var
  Dir, PrefsFile, Lang: String;
begin
  Dir := ExpandConstant('{userappdata}\YoRHa Protocol\YoRHa Protocol Launcher');
  PrefsFile := Dir + '\shared_preferences.json';
  if FileExists(PrefsFile) then
    exit;
  if FileExists(Dir + '\settings.json') then
    exit;
  Lang := 'en';
  if ActiveLanguage = 'german' then
    Lang := 'de'
  else if ActiveLanguage = 'thai' then
    Lang := 'th'
  else if ActiveLanguage = 'chinesesimplified' then
    Lang := 'zh';
  if ForceDirectories(Dir) then
    SaveStringToFile(PrefsFile, '{"flutter.app_locale":"' + Lang + '"}', False);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
    MoveFlutterRuntime
  else if CurStep = ssPostInstall then
    SeedLanguage;
end;
