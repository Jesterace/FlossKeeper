!include "MUI2.nsh"

!define APP_NAME "FlossKeeper"
!define APP_VERSION "1.3.0"
!define APP_PUBLISHER "Jesterace"
!define APP_EXE "FlossKeeper.exe"
!define INSTALL_DIR "$LOCALAPPDATA\Programs\FlossKeeper"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "../dist/windows/FlossKeeper-v${APP_VERSION}-Setup-icons.exe"
InstallDir "${INSTALL_DIR}"
RequestExecutionLevel user

!define MUI_ICON "flosskeeper.ico"
!define MUI_UNICON "flosskeeper.ico"

!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Launch FlossKeeper"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "FlossKeeper" SEC_APP
    SectionIn RO
    SetShellVarContext current
    SetOutPath "$INSTDIR"

    File /oname=FlossKeeper.exe "../dist/windows/FlossKeeper-v1.3.0-x86_64.exe"
    File /oname=flosskeeper.ico "flosskeeper.ico"

    WriteUninstaller "$INSTDIR\Uninstall.exe"

    CreateDirectory "$SMPROGRAMS\FlossKeeper"
    CreateShortCut "$SMPROGRAMS\FlossKeeper\FlossKeeper.lnk" "$INSTDIR\FlossKeeper.exe" "" "$INSTDIR\flosskeeper.ico" 0
    CreateShortCut "$SMPROGRAMS\FlossKeeper\Uninstall FlossKeeper.lnk" "$INSTDIR\Uninstall.exe" "" "$INSTDIR\flosskeeper.ico" 0

    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "DisplayName" "FlossKeeper 1.3.0"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "Publisher" "${APP_PUBLISHER}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "DisplayVersion" "1.3.0"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "DisplayIcon" "$INSTDIR\flosskeeper.ico"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper" "NoRepair" 1
SectionEnd

Section "Desktop shortcut" SEC_DESKTOP
    SetShellVarContext current
    CreateShortCut "$DESKTOP\FlossKeeper.lnk" "$INSTDIR\FlossKeeper.exe" "" "$INSTDIR\flosskeeper.ico" 0
SectionEnd

Section "Uninstall"
    SetShellVarContext current

    Delete "$DESKTOP\FlossKeeper.lnk"

    Delete "$SMPROGRAMS\FlossKeeper\FlossKeeper.lnk"
    Delete "$SMPROGRAMS\FlossKeeper\Uninstall FlossKeeper.lnk"
    RMDir "$SMPROGRAMS\FlossKeeper"

    Delete "$INSTDIR\FlossKeeper.exe"
    Delete "$INSTDIR\flosskeeper.ico"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"

    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\FlossKeeper"
SectionEnd

LangString DESC_SEC_APP ${LANG_ENGLISH} "Install FlossKeeper."
LangString DESC_SEC_DESKTOP ${LANG_ENGLISH} "Create a desktop shortcut."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_APP} $(DESC_SEC_APP)
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_DESKTOP} $(DESC_SEC_DESKTOP)
!insertmacro MUI_FUNCTION_DESCRIPTION_END
