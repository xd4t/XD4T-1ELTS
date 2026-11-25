[CmdletBinding()]
param(
    [switch]$NoWidget,
    [switch]$NoAutostart,
    [switch]$BuildIfMissing
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
$installDir = Join-Path $env:LOCALAPPDATA "XD4T-1ELTS"

Write-Host "=== XD4T // IELTS 6.5 - 1-Click Installer ===" -ForegroundColor Cyan

# 1. Check WebView2 Runtime
$wv2Keys = @(
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    "HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
)
$hasWv2 = $false
foreach ($k in $wv2Keys) {
    if (Test-Path -LiteralPath $k) {
        $pv = (Get-ItemProperty -LiteralPath $k -ErrorAction SilentlyContinue).pv
        if ($pv) { $hasWv2 = $true; break }
    }
}
if (-not $hasWv2) {
    Write-Warning "WebView2 Runtime not detected. Download Evergreen Bootstrapper: https://developer.microsoft.com/en-us/microsoft-edge/webview2/"
}

# 2. Locate binaries (Release zip layout or dev build layout)
$appCandidates = @(
    (Join-Path $root "xd4t-ielts65.exe"),
    (Join-Path $root "src-tauri\target\release\xd4t-ielts65.exe")
)
$widgetCandidates = @(
    (Join-Path $root "XD4TIeltsBar.exe"),
    (Join-Path $root "taskbar-widget\XD4TIeltsBar.exe")
)

$appSrc = $appCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
$widgetSrc = $widgetCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1

if (-not $appSrc) {
    $hasNpm = [bool](Get-Command npm -ErrorAction SilentlyContinue)
    $hasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)
    if ($hasNpm -and $hasCargo) {
        $doBuild = $BuildIfMissing.IsPresent
        if (-not $doBuild) {
            $ans = Read-Host "xd4t-ielts65.exe not found. Build from source now? (Y/n)"
            $doBuild = ($ans -eq "" -or $ans -match "^[Yy]")
        }
        if ($doBuild) {
            Push-Location $root
            try {
                npm install
                npx tauri build
            } finally {
                Pop-Location
            }
            $appSrc = $appCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
        }
    }
    if (-not $appSrc) {
        throw "xd4t-ielts65.exe not found. Download XD4T-1ELTS-v*-win-x64.zip from GitHub Releases, extract it, and run install.ps1."
    }
}

if (-not $NoWidget -and -not $widgetSrc) {
    $widgetBuild = Join-Path $root "taskbar-widget\build.ps1"
    if (Test-Path -LiteralPath $widgetBuild) {
        & $widgetBuild
        $widgetSrc = $widgetCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    }
}

# 3. Stop existing processes before replacing binaries
Get-Process -Name "xd4t-ielts65" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
if (-not $NoWidget) {
    Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Milliseconds 350

# 4. Copy binaries to %LOCALAPPDATA%\XD4T-1ELTS\
New-Item -ItemType Directory -Path $installDir -Force | Out-Null
$appDst = Join-Path $installDir "xd4t-ielts65.exe"
Copy-Item -LiteralPath $appSrc -Destination $appDst -Force
Write-Host "[install] Installed app -> $appDst"

$widgetDst = Join-Path $installDir "XD4TIeltsBar.exe"
if (-not $NoWidget -and $widgetSrc) {
    Copy-Item -LiteralPath $widgetSrc -Destination $widgetDst -Force
    Write-Host "[install] Installed widget -> $widgetDst"
}

# 5. Create Desktop & Start Menu shortcuts
$desktopDir = [Environment]::GetFolderPath('Desktop')
$startMenuDir = [Environment]::GetFolderPath('Programs')
$wsh = New-Object -ComObject WScript.Shell

foreach ($lnkDir in @($desktopDir, $startMenuDir)) {
    if ($lnkDir -and (Test-Path -LiteralPath $lnkDir)) {
        $lnkPath = Join-Path $lnkDir "XD4T IELTS 6.5.lnk"
        $sc = $wsh.CreateShortcut($lnkPath)
        $sc.TargetPath = $appDst
        $sc.WorkingDirectory = $installDir
        $sc.Description = "XD4T // IELTS 6.5 Study Companion"
        $sc.Save()
        Write-Host "[install] Shortcut -> $lnkPath"
    }
}

# 6. Register autostart (Scheduled Task Interactive + HKCU\Run) & launch widget (if enabled)
if (-not $NoWidget -and (Test-Path -LiteralPath $widgetDst)) {
    if (-not $NoAutostart) {
        $taskName = "XD4T-IeltsBar"
        $trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
        $trigger.Delay = "PT5S"
        $action = New-ScheduledTaskAction -Execute $widgetDst -WorkingDirectory $installDir
        $settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Seconds 0) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew
        $principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Limited
        Register-ScheduledTask -TaskName $taskName -Trigger $trigger -Action $action -Settings $settings -Principal $principal -Force | Out-Null
        Write-Host "[install] Scheduled Task registered: $taskName (AtLogOn +5s, Interactive)"

        $runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
        if (Test-Path -LiteralPath $runKey) {
            Remove-ItemProperty -Path $runKey -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue
        }
        Start-ScheduledTask -TaskName $taskName
        Write-Host "[install] Launched XD4TIeltsBar via Scheduled Task"
    } else {
        Start-Process -FilePath $widgetDst -WorkingDirectory $installDir
        Write-Host "[install] Launched XD4TIeltsBar widget"
    }
}

Write-Host "[install] Done! Open 'XD4T IELTS 6.5' from Desktop or Start Menu." -ForegroundColor Green
