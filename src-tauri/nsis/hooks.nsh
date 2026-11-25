!macro NSIS_HOOK_PREINSTALL
  nsExec::ExecToLog 'taskkill /F /IM xd4t-ielts65.exe'
  nsExec::ExecToLog 'taskkill /F /IM XD4TIeltsBar.exe'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "XD4TIeltsBar" '"$INSTDIR\XD4TIeltsBar.exe"'
  nsExec::ExecToLog 'powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "$$a = New-ScheduledTaskAction -Execute \"$INSTDIR\XD4TIeltsBar.exe\"; $$t = New-ScheduledTaskTrigger -AtLogOn -User $$env:USERNAME; $$t.Delay = \"PT5S\"; $$s = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew; Register-ScheduledTask -TaskName \"XD4T-IeltsBar\" -Action $$a -Trigger $$t -Settings $$s -Force | Out-Null"'
  Exec '"$INSTDIR\XD4TIeltsBar.exe"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /F /IM xd4t-ielts65.exe'
  nsExec::ExecToLog 'taskkill /F /IM XD4TIeltsBar.exe'
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "XD4TIeltsBar"
  nsExec::ExecToLog 'schtasks /Delete /TN "XD4T-IeltsBar" /F'
!macroend
