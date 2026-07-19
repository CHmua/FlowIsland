!macro CLEAN_LEGACY_SHORTCUTS
  Delete "$SMPROGRAMS\FlowIsland.lnk"
  RMDir /r "$SMPROGRAMS\FlowIsland"
  Delete "$SMPROGRAMS\NetSpeed Dynamic Pro.lnk"
  Delete "$SMPROGRAMS\NetSpeed Dynamic.lnk"
  Delete "$SMPROGRAMS\NSD.lnk"
  RMDir /r "$SMPROGRAMS\NetSpeed Dynamic Pro"
  RMDir /r "$SMPROGRAMS\NetSpeed Dynamic"
  RMDir /r "$SMPROGRAMS\NSD"
  Delete "$DESKTOP\FlowIsland.lnk"
  Delete "$DESKTOP\NetSpeed Dynamic Pro.lnk"
  Delete "$DESKTOP\NetSpeed Dynamic.lnk"
  Delete "$DESKTOP\NSD.lnk"
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro CLEAN_LEGACY_SHORTCUTS
  SetShellVarContext all
  !insertmacro CLEAN_LEGACY_SHORTCUTS
  SetShellVarContext current
!macroend

!macro NSIS_HOOK_POSTINSTALL
  SetShellVarContext current
  CreateShortCut "$SMPROGRAMS\FlowIsland.lnk" "$INSTDIR\flow-island.exe" "" "$INSTDIR\flow-island.exe" 0
  CreateShortCut "$DESKTOP\FlowIsland.lnk" "$INSTDIR\flow-island.exe" "" "$INSTDIR\flow-island.exe" 0
!macroend
