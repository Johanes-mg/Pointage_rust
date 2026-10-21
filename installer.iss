[Setup]
AppId={{8E7A8C4F-1234-4567-89AB-CDEF01234567}
AppName=Pointage
AppVersion=2.2.0
AppVerName=Pointage 2.2.0
AppPublisher=Johanes-mg
AppPublisherURL=https://johanes-mg.github.io
AppSupportURL=https://johanes-mg.github.io
AppUpdatesURL=https://johanes-mg.github.io
DefaultDirName={autopf}\Pointage
DefaultGroupName=Pointage
DisableProgramGroupPage=yes
OutputDir=Installeur
OutputBaseFilename=PointageSetup
SetupIconFile=Images\logo.ico
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
PrivilegesRequired=admin
UninstallDisplayIcon={app}\pointage.exe
DisableDirPage=no
DisableReadyPage=no
UsePreviousAppDir=yes
AllowUNCPath=no
ChangesEnvironment=no
VersionInfoVersion=2.2.0
VersionInfoCompany=Johanes-mg
VersionInfoDescription=Pointage 2.2.0 - Installation
VersionInfoProductName=Pointage
VersionInfoProductVersion=2.2.0

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\x86_64-pc-windows-gnullvm\release\pointage.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "Images\*.png"; DestDir: "{app}\Images"; Flags: ignoreversion
Source: "Images\*.ico"; DestDir: "{app}\Images"; Flags: ignoreversion

[Icons]
Name: "{group}\Pointage"; Filename: "{app}\pointage.exe"; IconFilename: "{app}\pointage.exe"; IconIndex: 0; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,Pointage}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Pointage"; Filename: "{app}\pointage.exe"; Tasks: desktopicon; IconFilename: "{app}\pointage.exe"; IconIndex: 0; WorkingDir: "{app}"

[Run]
Filename: "{app}\pointage.exe"; Description: "{cm:LaunchProgram,Pointage}"; Flags: nowait postinstall skipifsilent

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Reponse: Integer;
  DossierDonnees: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    DossierDonnees := ExpandConstant('{userappdata}\Johanes\Pointage');
    if DirExists(DossierDonnees) then
    begin
      Reponse := MsgBox(
        'Voulez-vous également supprimer vos données de pointage ?' + #13#10 + #13#10 +
        'Dossier : ' + DossierDonnees + #13#10 + #13#10 +
        'Si vous répondez Non, vos données seront conservées.',
        mbConfirmation, MB_YESNO);

      if Reponse = IDYES then
      begin
        DelTree(DossierDonnees, True, True, True);
      end;
    end;
  end;
end;
