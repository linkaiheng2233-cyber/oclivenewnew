; Keep Windows shortcuts bound to the current Chat Pro icon after an in-place update.
; The desktop shortcut is created from the finish page, after NSIS_HOOK_POSTINSTALL,
; so its icon is updated from .onGUIEnd without changing the user's checkbox choice.

Var OCLiveDesktopShortcutPath
Var OCLiveShortcutIconPath
Var OCLiveUninstallerHandle
Var OCLiveOwnedUninstallEntry

!macro OCLIVE_REFRESH_SHORTCUT ShortcutPath
  ${If} ${FileExists} "${ShortcutPath}"
    Delete "${ShortcutPath}"
    CreateShortcut "${ShortcutPath}" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\icons\icon.ico" 0 SW_SHOWNORMAL "" "${PRODUCTNAME}"
    !insertmacro SetLnkAppUserModelId "${ShortcutPath}"
  ${EndIf}
!macroend

Function OCLiveRefreshDesktopShortcutIcon
  !insertmacro ComHlpr_CreateInProcInstance ${CLSID_ShellLink} ${IID_IShellLink} r0 ""
  ${If} $0 P<> 0
    ${IUnknown::QueryInterface} $0 '("${IID_IPersistFile}",.r1)'
    ${If} $1 P<> 0
      ${IPersistFile::Load} $1 '("$OCLiveDesktopShortcutPath", ${STGM_READWRITE}).r2'
      ${If} $2 = 0
        ${IShellLink::SetIconLocation} $0 '("$OCLiveShortcutIconPath", 0).r2'
        ${If} $2 = 0
          ${IPersistFile::Save} $1 '("$OCLiveDesktopShortcutPath",1)'
        ${EndIf}
      ${EndIf}
      ${IUnknown::Release} $1 ""
    ${EndIf}
    ${IUnknown::Release} $0 ""
  ${EndIf}
FunctionEnd

Function .onGUIEnd
  IfFileExists "$OCLiveDesktopShortcutPath" 0 oclive_gui_end_done
  IfFileExists "$OCLiveShortcutIconPath" 0 oclive_gui_end_done
  Call OCLiveRefreshDesktopShortcutIcon
  oclive_gui_end_done:
FunctionEnd

!macro NSIS_HOOK_POSTINSTALL
  StrCpy $OCLiveDesktopShortcutPath "$DESKTOP\${PRODUCTNAME}.lnk"
  StrCpy $OCLiveShortcutIconPath "$INSTDIR\icons\icon.ico"
  ; Silent/passive installs create the desktop shortcut before this hook.
  ; Interactive installs may create it later from the finish page, which is
  ; still covered by .onGUIEnd above.
  !insertmacro OCLIVE_REFRESH_SHORTCUT "$DESKTOP\${PRODUCTNAME}.lnk"
  !if "${STARTMENUFOLDER}" != ""
    !insertmacro OCLIVE_REFRESH_SHORTCUT "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !else
    !insertmacro OCLIVE_REFRESH_SHORTCUT "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !endif
!macroend

; Match the registry view selected by Tauri's SetContext.
!macro OCLIVE_UNINSTALL_REGISTRY_VIEW
  StrCpy $1 0
  ${If} ${RunningX64}
    !if "${ARCH}" == "x64"
      StrCpy $1 0x0100
    !else if "${ARCH}" == "arm64"
      StrCpy $1 0x0100
    !else
      StrCpy $1 0x0200
    !endif
  ${EndIf}
!macroend

; Return the Win32 result in $0 and close any opened handle before continuing.
!macro OCLIVE_OPEN_UNINSTALL_REGISTRATION Access
  !insertmacro OCLIVE_UNINSTALL_REGISTRY_VIEW
  IntOp $1 $1 | ${Access}
  System::Call 'advapi32::RegOpenKeyExW(p 0x80000001, w "${UNINSTKEY}", i 0, i r1, *p .r2) i.r0'
  ${If} $0 = 0
    System::Call 'advapi32::RegCloseKey(p r2)'
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !if "${INSTALLMODE}" == "currentUser"
    Push $0
    Push $1
    Push $2
    StrCpy $OCLiveOwnedUninstallEntry 0
    ; DELETE. A missing key or parent leaves nothing for the uninstaller to remove.
    !insertmacro OCLIVE_OPEN_UNINSTALL_REGISTRATION 0x10000
    DetailPrint "Chat Pro uninstall registration preflight (HKCU result $0)."
    ${If} $0 <> 0
    ${AndIf} $0 <> 2
    ${AndIf} $0 <> 3
      DetailPrint "Uninstall stopped because Windows denied access to the registration (error $0)."
      MessageBox MB_OK|MB_ICONEXCLAMATION "Windows could not grant access to remove the Chat Pro entry from Installed apps (error $0). Chat Pro is still installed. Retry from the installing account or contact support." /SD IDOK
      SetErrorLevel 2
      Quit
    ${EndIf}
    ${If} $0 = 0
      ReadRegStr $1 HKCU "${UNINSTKEY}" "UninstallString"
      ${If} $1 != "$\"$INSTDIR\uninstall.exe$\""
      ${AndIf} $1 != "$INSTDIR\uninstall.exe"
        DetailPrint "Chat Pro uninstall registration points to a different installation."
        MessageBox MB_OK|MB_ICONEXCLAMATION "The Installed apps entry points to a different Chat Pro installation. Open that installation's uninstaller or contact support." /SD IDOK
        SetErrorLevel 2
        Quit
      ${EndIf}
      StrCpy $OCLiveOwnedUninstallEntry 1
    ${EndIf}
    ; Keep the original executable at its registered path through Tauri's
    ; early Delete. Windows releases this handle if the uninstall aborts.
    ; GENERIC_READ, FILE_SHARE_READ, OPEN_EXISTING.
    System::Call 'kernel32::CreateFileW(w "$INSTDIR\uninstall.exe", i 0x80000000, i 1, p 0, i 3, i 0x80, p 0) p.r0 ?e'
    Pop $2
    ${If} $0 P= -1
      DetailPrint "Could not keep the Chat Pro uninstaller available (file result $2)."
      MessageBox MB_OK|MB_ICONEXCLAMATION "Windows could not keep the Chat Pro uninstaller available (error $2). Close other installers and retry." /SD IDOK
      SetErrorLevel 2
      Quit
    ${EndIf}
    StrCpy $OCLiveUninstallerHandle $0
    Pop $2
    Pop $1
    Pop $0
  !endif
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  !if "${INSTALLMODE}" == "currentUser"
    Push $0
    Push $1
    Push $2
    ; KEY_QUERY_VALUE. Confirm the registration actually went away.
    !insertmacro OCLIVE_OPEN_UNINSTALL_REGISTRATION 0x0001
    DetailPrint "Chat Pro uninstall registration after cleanup (HKCU result $0)."
    ${If} $0 = 0
    ${AndIf} $OCLiveOwnedUninstallEntry = 1
      ReadRegStr $2 HKCU "${UNINSTKEY}" "UninstallString"
      ${If} $2 == "$\"$INSTDIR\uninstall.exe$\""
      ${OrIf} $2 == "$INSTDIR\uninstall.exe"
        !insertmacro OCLIVE_UNINSTALL_REGISTRY_VIEW
        ; Retry only this verified leaf in the same hive and view.
        System::Call 'advapi32::RegDeleteKeyExW(p 0x80000001, w "${UNINSTKEY}", i r1, i 0) i.r0'
        DetailPrint "Chat Pro uninstall registration retry (HKCU result $0)."
        !insertmacro OCLIVE_OPEN_UNINSTALL_REGISTRATION 0x0001
        DetailPrint "Chat Pro uninstall registration after retry (HKCU result $0)."
      ${EndIf}
    ${EndIf}
    ${If} $0 <> 2
    ${AndIf} $0 <> 3
      DetailPrint "Could not verify removal of the uninstall registration (registry result $0)."
      MessageBox MB_OK|MB_ICONEXCLAMATION "The uninstaller could not verify removal of the Chat Pro entry from Installed apps (registry result $0). The original uninstaller is still available for retry. Keep this message for support." /SD IDOK
      SetErrorLevel 2
      Quit
    ${EndIf}
    StrCpy $0 $OCLiveUninstallerHandle
    System::Call 'kernel32::CloseHandle(p r0)'
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    Pop $2
    Pop $1
    Pop $0
  !endif
!macroend
