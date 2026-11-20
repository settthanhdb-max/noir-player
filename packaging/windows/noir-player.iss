; NOIR Player -- Windows installer (Inno Setup 6).
;
; Build on Windows:
;   1. cargo build --release            (in the repo root)
;   2. Download Inno Setup: https://jrsoftware.org/isdl.php
;   3. & "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" packaging\windows\noir-player.iss
;      (from PowerShell -- the & call operator is required)
;      -> output: packaging\windows\Output\noir-player-setup.exe
;
; The installer is per-user (no admin rights required): it installs to
; %LOCALAPPDATA%\NOIR Player and registers file associations under
; HKCU\Software\Classes.

#define MyAppName "NOIR Player"
#define MyAppVersion "0.1.5"
#define MyAppPublisher "NOIR Player"
#define MyAppExeName "noir-player.exe"

[Setup]
AppId={{7D4C1A2E-5B3F-4E8A-9C6D-1F0A2B3C4D5E}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\NOIR Player
DefaultGroupName={#MyAppName}
PrivilegesRequired=lowest
OutputBaseFilename=noir-player-setup
OutputDir=Output
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\{#MyAppExeName}
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "assoc"; Description: "Open media files (MP4, MKV, MP3, ...) with NOIR Player by default"; \
    GroupDescription: "File associations"; Flags: checkedonce

[Files]
Source: "..\..\target\release\noir-player.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Per-user class identifiers. Doubled quotes inside ValueData escape Inno strings.
Root: HKCU; Subkey: "Software\Classes\NoirPlayer.media"; ValueType: string; ValueName: ""; \
    ValueData: "NOIR Player Media"; Tasks: assoc; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\NoirPlayer.media\DefaultIcon"; ValueType: string; ValueName: ""; \
    ValueData: "{app}\{#MyAppExeName},0"; Tasks: assoc; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\NoirPlayer.media\shell\open\command"; ValueType: string; ValueName: ""; \
    ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: assoc; Flags: uninsdeletekey

Root: HKCU; Subkey: "Software\Classes\.mp4"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.m4v"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.mkv"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.avi"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.mov"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.webm"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.wmv"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.flv"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.mpg"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.mpeg"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.ts"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.m2ts"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.mp3"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.m4a"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.aac"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.flac"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.wav"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.ogg"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.opus"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.wma"; ValueType: string; ValueName: ""; ValueData: "NoirPlayer.media"; Tasks: assoc; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch NOIR Player"; \
    Flags: nowait postinstall skipifsilent

[UninstallRun]
; Nothing extra: the app stores settings under %APPDATA%\noir\noir-player,
; which is intentionally left in place on uninstall.

[UninstallDelete]
; Nothing: user data (history, playlists) is preserved.
