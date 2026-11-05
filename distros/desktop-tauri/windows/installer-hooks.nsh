; Keep Windows shortcuts bound to the current Chat Pro icon after an in-place update.
; The desktop shortcut is created from the finish page, after NSIS_HOOK_POSTINSTALL,
; so its icon is updated from .onGUIEnd without changing the user's checkbox choice.

Var OCLiveDesktopShortcutPath
Var OCLiveShortcutIconPath

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

; Match the registry view selected by Tauri's SetContext. Return the Win32
; result in $0 and close any opened handle before continuing.
!macro OCLIVE_OPEN_UNINSTALL_REGISTRATION Access
  StrCpy $1 ${Access}
  ${If} ${RunningX64}
    !if "${ARCH}" == "x64"
      IntOp $1 $1 | 0x0100
    !else if "${ARCH}" == "arm64"
      IntOp $1 $1 | 0x0100
    !else
      IntOp $1 $1 | 0x0200
    !endif
  ${EndIf}
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
    ; DELETE. A missing key or parent leaves nothing for the uninstaller to remove.
    !insertmacro OCLIVE_OPEN_UNINSTALL_REGISTRATION 0x10000
    ${If} $0 <> 0
    ${AndIf} $0 <> 2
    ${AndIf} $0 <> 3
      DetailPrint "Uninstall stopped because Windows denied access to the registration (error $0)."
      MessageBox MB_OK|MB_ICONEXCLAMATION "Windows could not grant access to remove the Chat Pro entry from Installed apps (error $0). Chat Pro is still installed. Retry from the installing account or contact support." /SD IDOK
      SetErrorLevel 2
      Quit
    ${EndIf}
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
    ${If} $0 <> 2
    ${AndIf} $0 <> 3
      DetailPrint "Could not verify removal of the uninstall registration (registry result $0)."
      MessageBox MB_OK|MB_ICONEXCLAMATION "The uninstaller could not verify that the Chat Pro entry was removed from Installed apps (registry result $0). Keep this message for support." /SD IDOK
      SetErrorLevel 2
      Quit
    ${EndIf}
    Pop $2
    Pop $1
    Pop $0
  !endif
!macroend
