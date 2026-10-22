; Bornopata for Windows: the installer (Inno Setup).
;
; It installs into Program Files\Bornopata (64 bit): the text service DLL (a 64 bit and a 32 bit one), the Server, the
; settings program and the data. Then it (1) gives the folder read and execute rights to ALL APPLICATION PACKAGES, so
; sandboxed applications (the taskbar search, Settings) can load the DLL, (2) registers the text service with regsvr32,
; once for the 64 bit and once for the 32 bit DLL (the icon goes to %ProgramData%\Bornopata), (3) puts a shortcut to the
; Server in the machine-wide Startup folder, so it runs at every login, (4) starts the Server at once when the wizard
; closes, so nobody has to sign out first, and (5) adds the keyboard to the user's language list.
; Uninstalling reverses it: stop the Server, unregister the DLLs, delete the files. The user's data in %APPDATA%\Bornopata
; stays.
;
; Upgrades: the DLL is loaded into every application, so a locked file cannot be overwritten. The DLLs are named by
; version (bornopata_tsf-<version>.dll) and installed side by side, only the new one is registered, and the old ones are
; deleted afterwards (those still in use at restart). Applications that are already open keep the old DLL until they are
; restarted, and the Server serves both. Inno's CloseApplications would ask the Restart Manager to close everything that
; has a DLL loaded, which for an input method is every application with a text box, so it is off; the few executables
; that have to be replaced are stopped with taskkill.
; When the Server is killed an application that is being typed in starts it again (see the DLL's launch.rs), and that
; locks the file and the data it maps. The installer therefore holds the mutex Global\BornopataInstaller while it runs,
; which the new DLL sees and keeps its hands off. The old DLLs do not know it, so the Server executable is renamed out of
; the way before it is stopped, and is installed last: until the new one has landed no old DLL can start a Server.
;
; The version comes from the build script with /DAppVersion=..., 0.1.0 by default. How to build it: README.md here.

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
; VersionInfoVersion takes a.b.c.d numbers only; the build script passes the version without a pre-release suffix.
#ifndef AppVersionNumeric
  #define AppVersionNumeric AppVersion
#endif
#define AppName "Bornopata"
#define Publisher "Bornopata"
#define WebsiteUrl "https://andrew-velox.github.io/Bornopata-Web/"
; The script is in apps\windows\installer, three levels below the repository root.
#define Repo "..\..\.."
; The versioned names of the DLLs, see "Upgrades" above.
#define TsfDll "bornopata_tsf-" + AppVersion + ".dll"
#define TsfDll32 "bornopata_tsf-" + AppVersion + "-x86.dll"

[Setup]
AppId={{5E0C6B2A-3D84-4F7B-A1C9-7B2E94D8F013}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#Publisher}
AppSupportURL={#WebsiteUrl}
VersionInfoVersion={#AppVersionNumeric}
DefaultDirName={autopf}\Bornopata
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
PrivilegesRequired=admin
; Do not let the Restart Manager close every application that has the DLL loaded (that is every application with a text box).
CloseApplications=no
OutputDir={#Repo}\target\installer
OutputBaseFilename=bornopata-{#AppVersion}-windows-x86_64-setup
SetupIconFile={#Repo}\apps\windows\tsf\resources\bornopata.ico
UninstallDisplayIcon={app}\bornopata.ico
LicenseFile={#Repo}\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Messages]
; The finish page: the DLL is loaded into every application, so one that was already open before the installation has to be
; restarted to use the new version (see "Upgrades"). That is the thing most likely to be taken for "it did not work".
FinishedLabel=Bornopata is installed and added to your keyboards. Press Win+Space in a text box to choose it.%n%nApplications that were already open need to be closed and opened again before they offer it (or sign out and sign in again). If Bornopata is not in the list, add it in Settings > Time & language > Language & region, under Bangla (Bangladesh).

[Files]
; The binaries. The DLLs are installed side by side by version; uninstalling registers one still in use for deletion at restart.
Source: "{#Repo}\target\release\bornopata_tsf.dll"; DestDir: "{app}"; DestName: "{#TsfDll}"; Flags: ignoreversion uninsrestartdelete
Source: "{#Repo}\target\i686-pc-windows-msvc\release\bornopata_tsf.dll"; DestDir: "{app}"; DestName: "{#TsfDll32}"; Flags: ignoreversion uninsrestartdelete
Source: "{#Repo}\target\release\bornopata-settings.exe"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#Repo}\apps\windows\tsf\resources\bornopata.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "add-keyboard.ps1"; DestDir: "{app}"; Flags: ignoreversion
; The data: the dictionary, the word pair model and the glosses, where the Server looks (data next to the program).
Source: "{#Repo}\target\release-inputs\words.bnd"; DestDir: "{app}\data"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#Repo}\target\release-inputs\lm.bnd"; DestDir: "{app}\data"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#Repo}\target\release-inputs\gloss-en.bnd"; DestDir: "{app}\data"; Flags: ignoreversion skipifsourcedoesntexist
; Licenses and attributions.
Source: "{#Repo}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Repo}\NOTICE.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Repo}\assets\dictionary\ATTRIBUTION.txt"; DestDir: "{app}\licenses"; DestName: "DICTIONARY-ATTRIBUTION.txt"; Flags: ignoreversion
Source: "{#Repo}\assets\gloss\ATTRIBUTION.txt"; DestDir: "{app}\licenses"; DestName: "GLOSS-ATTRIBUTION.txt"; Flags: ignoreversion
Source: "{#Repo}\assets\emoji\LICENSE-unicode.txt"; DestDir: "{app}\licenses"; DestName: "EMOJI-LICENSE-unicode.txt"; Flags: ignoreversion
; The Server goes last: the moment it lands an old DLL can start it and take hold of the data files (see the top).
Source: "{#Repo}\target\release\bornopata-server.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Bornopata Settings"; Filename: "{app}\bornopata-settings.exe"; IconFilename: "{app}\bornopata.ico"
Name: "{group}\Uninstall Bornopata"; Filename: "{uninstallexe}"
; Starts at login: Explorer runs the shortcut with ShellExecute, which also lets a Server that asks for uiAccess start.
; {commonstartup} (all users) and not {userstartup}: this is an administrator installation, and the per-user folder would
; be that of whoever is elevated, which is not necessarily the user it is for. The machine-wide Startup entry still runs
; for each user, in the session of that user's Explorer.
Name: "{commonstartup}\Bornopata Server"; Filename: "{app}\bornopata-server.exe"; WorkingDir: "{app}"; IconFilename: "{app}\bornopata.ico"

[Run]
; (1) Sandboxed applications must be able to read the folder to load the DLL (*S-1-15-2-1 is ALL APPLICATION PACKAGES, a SID,
;     so it does not depend on the language).
Filename: "{sys}\icacls.exe"; Parameters: """{app}"" /grant *S-1-15-2-1:(OI)(CI)RX /T /C /Q"; \
  Flags: runhidden waituntilterminated; StatusMsg: "Setting access for sandboxed applications..."
; (2) Registers the text service (HKCR, and the icon in %ProgramData%\Bornopata). This version's DLL is registered, so
;     InprocServer32 points at the new file; an old DLL must NOT be unregistered (that would remove the whole CLSID).
Filename: "{sys}\regsvr32.exe"; Parameters: "/s ""{app}\{#TsfDll}"""; \
  Flags: runhidden waituntilterminated; StatusMsg: "Registering the input method..."
; The 32 bit DLL is registered with the 32 bit regsvr32 in SysWOW64, so InprocServer32 lands under WOW6432Node for 32 bit
; applications.
Filename: "{syswow64}\regsvr32.exe"; Parameters: "/s ""{app}\{#TsfDll32}"""; \
  Flags: runhidden waituntilterminated; StatusMsg: "Registering the input method (32 bit)..."
; (4) Starting the Server when the wizard closes is in [Code], NextButtonClick.
; (5) Adds the keyboard to the language list of the user who ran the installer (not the administrator), see
;     add-keyboard.ps1. The text service has to be registered first.
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""{app}\add-keyboard.ps1"""; \
  Flags: runhidden runasoriginaluser waituntilterminated; StatusMsg: "Adding the Bornopata keyboard to your languages..."

[UninstallRun]
; The reverse: stop the Server and the settings program, unregister the DLLs; Inno deletes the files afterwards.
Filename: "{sys}\taskkill.exe"; Parameters: "/im bornopata-server.exe /f"; \
  Flags: runhidden; RunOnceId: "KillServer"
Filename: "{sys}\taskkill.exe"; Parameters: "/im bornopata-settings.exe /f"; \
  Flags: runhidden; RunOnceId: "KillSettings"
Filename: "{sys}\regsvr32.exe"; Parameters: "/u /s ""{app}\{#TsfDll}"""; \
  Flags: runhidden; RunOnceId: "UnregDll"
Filename: "{syswow64}\regsvr32.exe"; Parameters: "/u /s ""{app}\{#TsfDll32}"""; \
  Flags: runhidden; RunOnceId: "UnregDll32"

[UninstallDelete]
; DLLs of earlier versions (they are normally deleted at an upgrade; the ones still in use are left to this).
Type: files; Name: "{app}\bornopata_tsf-*.dll"
Type: files; Name: "{app}\bornopata-server.old-*.exe"

[Code]
function CreateMutex(Attributes: Longint; InitialOwner: BOOL; Name: String): THandle;
  external 'CreateMutexW@kernel32.dll stdcall';

{ Held while installing and uninstalling; the name is the one in the DLL's launch.rs. The handle is not closed: the system
  takes it back when the process ends. }
procedure HoldInstallerMutex;
begin
  if CreateMutex(0, False, 'Global\BornopataInstaller') = 0 then
    Log('could not make the installer mutex');
end;

function InitializeSetup: Boolean;
begin
  HoldInstallerMutex;
  Result := True;
end;

function InitializeUninstall: Boolean;
begin
  HoldInstallerMutex;
  Result := True;
end;

{ A running executable cannot be overwritten but can be renamed: renamed to bornopata-server.old-<random>.exe it frees the
  name, so an old DLL cannot start it (see the top). It is deleted when the installation is done. }
procedure RetireServerExe;
var
  Path: String;
begin
  Path := ExpandConstant('{app}\bornopata-server.exe');
  if FileExists(Path) then
    if not RenameFile(Path, ExpandConstant('{app}\bornopata-server.old-') + IntToStr(Random(1000000)) + '.exe') then
      Log('could not rename the old Server: ' + Path);
end;

procedure KillProcess(const Image: String);
var
  ResultCode: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/im ' + Image + ' /f', '',
    SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

{ Reinstalling the same version (what happens while developing): the target name is that of a DLL already loaded, which
  cannot be overwritten but can be renamed. It is renamed first, and DeleteStaleDlls deletes it afterwards or registers it
  for deletion at restart. }
procedure RetireLoadedDll(const Name: String);
var
  Path, Retired: String;
begin
  Path := ExpandConstant('{app}\') + Name;
  if FileExists(Path) then
  begin
    Retired := ExpandConstant('{app}\bornopata_tsf-{#AppVersion}.old-') + IntToStr(Random(1000000)) + '.dll';
    if not RenameFile(Path, Retired) then
      Log('could not rename the old DLL: ' + Path);
  end;
end;

{ Before anything is overwritten the Server and the settings program are stopped (only these executables are overwritten;
  the DLLs go side by side, no application has to be closed). taskkill returns non-zero when nothing runs, which is fine. }
function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  RetireServerExe;
  KillProcess('bornopata-server.exe');
  KillProcess('bornopata-settings.exe');
  RetireLoadedDll('{#TsfDll}');
  RetireLoadedDll('{#TsfDll32}');
  Result := '';
end;

{ Deletes the Servers renamed out of the way; the ones that cannot be deleted go to the restart list. }
procedure DeleteRetiredServers;
var
  Dir, Path: String;
  Found: TFindRec;
begin
  Dir := ExpandConstant('{app}');
  if FindFirst(Dir + '\bornopata-server.old-*.exe', Found) then
  begin
    try
      repeat
        Path := Dir + '\' + Found.Name;
        if not DeleteFile(Path) then
          RestartReplace(Path, '');
      until not FindNext(Found);
    finally
      FindClose(Found);
    end;
  end;
end;

{ Deletes the DLLs of earlier versions. One that an application still has loaded cannot be deleted and goes to the restart
  list: those applications keep the old DLL until they are restarted, and the Server serves both versions. }
procedure DeleteStaleDlls;
var
  Dir, Path: String;
  Found: TFindRec;
begin
  Dir := ExpandConstant('{app}');
  if FindFirst(Dir + '\bornopata_tsf*.dll', Found) then
  begin
    try
      repeat
        if (CompareText(Found.Name, '{#TsfDll}') <> 0) and (CompareText(Found.Name, '{#TsfDll32}') <> 0) then
        begin
          Path := Dir + '\' + Found.Name;
          if not DeleteFile(Path) then
            RestartReplace(Path, '');
        end;
      until not FindNext(Found);
    finally
      FindClose(Found);
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    DeleteStaleDlls;
    DeleteRetiredServers;
  end;
end;

{ Uninstalling too: the Server is renamed before [UninstallRun] kills it, so an old DLL cannot start it again and the
  file can be deleted. }
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then
    RetireServerExe;
end;

{ After the finish page the Server is started at once, as the user who ran the installer and not as the administrator:
  with ShellExecute, which is the same as double-clicking it. }
function NextButtonClick(CurPageID: Integer): Boolean;
var
  ErrorCode: Integer;
begin
  Result := True;
  if (CurPageID = wpFinished) and (not WizardSilent) then
    ShellExecAsOriginalUser(
      '', ExpandConstant('{app}\bornopata-server.exe'), '', '',
      SW_SHOWNORMAL, ewNoWait, ErrorCode);
end;
