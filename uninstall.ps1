[CmdletBinding()]
param(
    [switch]$PurgeData
)

$ErrorActionPreference = "Stop"
$installDir = Join-Path $env:LOCALAPPDATA "XD4T-1ElLTS"
$dataDir = Join-Path $env:APPDATA "xd4t-ielts65"

Get-Process -Name "xd4t-ielts65" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 350

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
if (Test-Path -LiteralPath $runKey) {
    Remove-ItemProperty -Path $runKey -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue
    Write-Host "[uninstall] Removed autostart entry XD4TIeltsBar"
}

$desktopDir = [Environment]::GetFolderPath('Desktop')
$startMenuDir = [Environment]::GetFolderPath('Programs')
foreach ($lnkDir in @($desktopDir, $startMenuDir)) {
    $lnkPath = Join-Path $lnkDir "XD4T IELTS 6.5.lnk"
    if (Test-Path -LiteralPath $lnkPath) {
        Remove-Item -LiteralPath $lnkPath -Force
        Write-Host "[uninstall] Removed shortcut $lnkPath"
    }
}

if (Test-Path -LiteralPath $installDir) {
    Remove-Item -LiteralPath $installDir -Recurse -Force
    Write-Host "[uninstall] Removed $installDir"
}

if ($PurgeData -and (Test-Path -LiteralPath $dataDir)) {
    Remove-Item -LiteralPath $dataDir -Recurse -Force
    Write-Host "[uninstall] Purged user data at $dataDir"
} else {
    Write-Host "[uninstall] Kept study database at $dataDir (pass -PurgeData to remove)"
}

Write-Host "[uninstall] Completed." -ForegroundColor Green
