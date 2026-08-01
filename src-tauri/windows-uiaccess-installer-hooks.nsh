!include LogicLib.nsh
!include x64.nsh

!define CYRENE_UIACCESS_CERTIFICATE "$INSTDIR\certificates\CyreneCompassUIAccess.cer"
!define CYRENE_UIACCESS_THUMBPRINT "C4BF43FCE7DD11CE4506EB6C521ADC02A4F93F2A"

!macro NSIS_HOOK_PREINSTALL
  ${If} ${RunningX64}
    StrCpy $INSTDIR "$PROGRAMFILES64\CyreneCompass"
  ${Else}
    StrCpy $INSTDIR "$PROGRAMFILES\CyreneCompass"
  ${EndIf}
  SetOutPath $INSTDIR
!macroend

!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToStack '"$SYSDIR\certutil.exe" -addstore -f Root "${CYRENE_UIACCESS_CERTIFICATE}"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "无法安装 CyreneCompass UIAccess 根证书，错误代码：$0。程序将无法使用 UIAccess。"
    Abort
  ${EndIf}

  nsExec::ExecToStack '"$SYSDIR\certutil.exe" -addstore -f TrustedPublisher "${CYRENE_UIACCESS_CERTIFICATE}"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "无法信任 CyreneCompass UIAccess 发布者证书，错误代码：$0。程序将无法使用 UIAccess。"
    Abort
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToStack '"$SYSDIR\certutil.exe" -delstore TrustedPublisher "${CYRENE_UIACCESS_THUMBPRINT}"'
  Pop $0
  Pop $1
  nsExec::ExecToStack '"$SYSDIR\certutil.exe" -delstore Root "${CYRENE_UIACCESS_THUMBPRINT}"'
  Pop $0
  Pop $1
!macroend
