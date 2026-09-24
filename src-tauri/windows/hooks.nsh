!macro NSIS_HOOK_POSTUNINSTALL
  ; Byte's startup preference is stored outside the installer-owned registry
  ; keys. Remove only that executable launch entry on a real uninstall so the
  ; user is never left with a dead HKCU Run value.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Byte"
!macroend
