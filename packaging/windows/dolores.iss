; Compiled only from a verified staged payload by scripts/installer.py.
#ifndef InstallerAppId
#define InstallerAppId "A7B728D7-805D-420A-A33B-C9077C8D410E"
#endif
#ifndef ShortcutName
#define ShortcutName "Dolores"
#endif
[Setup]
AppId={#InstallerAppId}
AppName=Dolores
AppVersion={#AppVersion}
VersionInfoVersion={#NumericVersion}
AppPublisher=Dolores contributors
DefaultDirName={autopf}\Dolores
DefaultGroupName=Dolores
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
UsePreviousPrivileges=yes
ArchitecturesAllowed=x64os
ArchitecturesInstallIn64BitMode=x64os
MinVersion=10.0
OutputDir={#OutputDirectory}
OutputBaseFilename=Dolores-{#AppVersion}-windows-x64-setup
SetupIconFile=..\..\apps\dolores_flutter\windows\runner\resources\app_icon.ico
UninstallDisplayIcon={app}\dolores_flutter.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
LicenseFile={#PayloadDirectory}\LICENSE

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "{#PayloadDirectory}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#ShortcutName}"; Filename: "{app}\Start-Dolores.cmd"; IconFilename: "{app}\dolores_flutter.exe"
Name: "{autodesktop}\{#ShortcutName}"; Filename: "{app}\Start-Dolores.cmd"; IconFilename: "{app}\dolores_flutter.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Start-Dolores.cmd"; Description: "Launch Dolores"; Flags: postinstall shellexec skipifsilent runasoriginaluser

[Code]
var
  RemoveUserData: Boolean;
  DataDirectory: String;
  DataLock: Integer;

function GetFileAttributesW(FileName: String): Cardinal;
  external 'GetFileAttributesW@kernel32.dll stdcall';
function CreateFileW(FileName: String; AccessMode, ShareMode: Cardinal;
  SecurityAttributes: Integer; CreationDisposition, FlagsAndAttributes: Cardinal;
  TemplateFile: Integer): Integer;
  external 'CreateFileW@kernel32.dll stdcall';
function CloseHandle(Handle: Integer): Boolean;
  external 'CloseHandle@kernel32.dll stdcall';

function SafeDataTree(Path: String): Boolean;
var
  FindRec: TFindRec;
  Child: String;
begin
  Result := False;
  if (GetFileAttributesW(Path) and $400) <> 0 then exit;
  Result := True;
  if FindFirst(AddBackslash(Path) + '*', FindRec) then
  begin
    try
      repeat
        if (FindRec.Name <> '.') and (FindRec.Name <> '..') then
        begin
          Child := AddBackslash(Path) + FindRec.Name;
          if (FindRec.Attributes and $400) <> 0 then Result := False;
          if ((FindRec.Attributes and FILE_ATTRIBUTE_DIRECTORY) <> 0) and Result then
            Result := SafeDataTree(Child);
        end;
      until (not Result) or (not FindNext(FindRec));
    finally
      FindClose(FindRec);
    end;
  end;
end;

function InitializeUninstall(): Boolean;
var
  Form: TSetupForm;
  Info: TNewStaticText;
  RemoveBox: TNewCheckBox;
  Proceed, Cancel: TNewButton;
begin
  Result := True;
  RemoveUserData := False;
  DataLock := -1;
  DataDirectory := ExpandConstant('{userappdata}\dev.dolores.desktop');
#ifdef UninstallFixtureDirectory
  { Explicit diagnostic compilation only. Production never reads a purge switch. }
  DataDirectory := '{#UninstallFixtureDirectory}';
  if UninstallSilent then
    RemoveUserData := ExpandConstant('{param:REMOVEFIXTUREDATA|0}') = '1';
  if not RemoveUserData then
#endif
  if UninstallSilent then exit; { Silent uninstall always preserves user data. }
  if not UninstallSilent then
  begin
  Form := CreateCustomForm(ScaleX(520), ScaleY(235), True, False);
  try
    Form.Caption := 'Uninstall Dolores';
    Form.ClientWidth := ScaleX(520);
    Form.ClientHeight := ScaleY(235);
    Info := TNewStaticText.Create(Form);
    Info.Parent := Form;
    Info.SetBounds(ScaleX(20), ScaleY(16), ScaleX(480), ScaleY(108));
    Info.AutoSize := False;
    Info.WordWrap := True;
    Info.Caption := 'Remove the application. Your local data is kept unless you select the option below.' + #13#10 + #13#10 +
      'Data for the account running this uninstaller: ' + DataDirectory + #13#10 +
      'Other accounts, custom data locations, projects, shared skills and OS-vault credentials are kept.';
    RemoveBox := TNewCheckBox.Create(Form);
    RemoveBox.Parent := Form;
    RemoveBox.SetBounds(ScaleX(20), ScaleY(135), ScaleX(480), ScaleY(32));
    RemoveBox.Caption := 'Remove my conversations, settings and attachments';
    RemoveBox.Checked := False;
    Proceed := TNewButton.Create(Form);
    Proceed.Parent := Form;
    Proceed.SetBounds(ScaleX(296), ScaleY(190), ScaleX(95), ScaleY(28));
    Proceed.Caption := 'Uninstall';
    Proceed.ModalResult := mrOK;
    Cancel := TNewButton.Create(Form);
    Cancel.Parent := Form;
    Cancel.SetBounds(ScaleX(403), ScaleY(190), ScaleX(95), ScaleY(28));
    Cancel.Caption := 'Cancel';
    Cancel.Cancel := True;
    Cancel.ModalResult := mrCancel;
    Result := Form.ShowModal = mrOK;
    RemoveUserData := Result and RemoveBox.Checked;
  finally
    Form.Free;
  end;
  end;
  if RemoveUserData and DirExists(DataDirectory) then
  begin
    if not SafeDataTree(DataDirectory) then
    begin
      SuppressibleMsgBox('The data folder contains a linked folder. Data was kept. Review the links or uninstall without removing data.', mbError, MB_OK, IDOK);
      Result := False;
      exit;
    end;
    if FileExists(DataDirectory + '\dolores.lock') then
    begin
      DataLock := CreateFileW(DataDirectory + '\dolores.lock', $C0000000, 0, 0, 3, $80, 0);
      if DataLock = -1 then
      begin
        SuppressibleMsgBox('Dolores is using this data folder, or it is unavailable. Close Dolores, then retry. Data was kept.', mbError, MB_OK, IDOK);
        Result := False;
      end;
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if (CurUninstallStep = usPostUninstall) and RemoveUserData then
  begin
    if DataLock <> -1 then
    begin
      CloseHandle(DataLock);
      DataLock := -1;
    end;
    if DirExists(DataDirectory) then
      if (not SafeDataTree(DataDirectory)) or (not DelTree(DataDirectory, True, True, True)) then
        SuppressibleMsgBox('Some local data could not be removed. Review ' + DataDirectory + ' after closing Dolores.', mbError, MB_OK, IDOK);
  end;
end;

function InitializeSetup(): Boolean;
begin
  Result := FileExists(ExpandConstant('{sys}\msvcp140.dll')) and
    FileExists(ExpandConstant('{sys}\vcruntime140.dll')) and
    FileExists(ExpandConstant('{sys}\vcruntime140_1.dll'));
  if not Result then
    MsgBox('Dolores needs the Microsoft Visual C++ Redistributable for x64. Install or repair it from https://aka.ms/vs/17/release/vc_redist.x64.exe, then run this installer again.', mbError, MB_OK);
end;
