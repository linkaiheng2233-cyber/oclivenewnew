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
