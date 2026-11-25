!macro NSIS_HOOK_PREINSTALL
  nsExec::ExecToLog 'taskkill /F /IM xd4t-ielts65.exe'
  nsExec::ExecToLog 'taskkill /F /IM XD4TIeltsBar.exe'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "XD4TIeltsBar" '"$INSTDIR\XD4TIeltsBar.exe"'
  System::Call 'Kernel32::SetEnvironmentVariable(t "XD4T_INSTDIR", t "$INSTDIR")'
  nsExec::ExecToLog 'powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "$$exe = Join-Path $$env:XD4T_INSTDIR '\''XD4TIeltsBar.exe'\''; $$a = New-ScheduledTaskAction -Execute $$exe; $$t = New-ScheduledTaskTrigger -AtLogOn -User $$env:USERNAME; $$t.Delay = '\''PT5S'\''; $$s = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew; Register-ScheduledTask -TaskName '\''XD4T-IeltsBar'\'' -Action $$a -Trigger $$t -Settings $$s -Force | Out-Null"'
  Exec '"$INSTDIR\XD4TIeltsBar.exe"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /F /IM xd4t-ielts65.exe'
  nsExec::ExecToLog 'taskkill /F /IM XD4TIeltsBar.exe'
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "XD4TIeltsBar"
  nsExec::ExecToLog 'schtasks /Delete /TN "XD4T-IeltsBar" /F'
!macroend
