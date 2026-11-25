[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$running = Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue
if ($running) {
    Stop-Process -Name "XD4TIeltsBar" -Force -ErrorAction SilentlyContinue
    Write-Host "[remove] Stopped XD4TIeltsBar."
}

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
if (Test-Path $runKey) {
    $prop = Get-ItemProperty -Path $runKey -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue
    if ($null -ne $prop) {
        Remove-ItemProperty -Path $runKey -Name "XD4TIeltsBar" -ErrorAction Stop
        Write-Host "[remove] Removed HKCU\...\Run\XD4TIeltsBar"
    }
}
