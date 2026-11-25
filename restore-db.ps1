[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$BackupFile
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path -LiteralPath $BackupFile)) {
    throw "Backup file not found: $BackupFile"
}

$dataDir = Join-Path $env:APPDATA "xd4t-ielts65"
$dbPath = Join-Path $dataDir "ielts65.db"
New-Item -ItemType Directory -Path $dataDir -Force | Out-Null

Get-Process -Name "xd4t-ielts65" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 400

foreach ($suf in @("-wal", "-shm")) {
    $sidecar = "$dbPath$suf"
    if (Test-Path -LiteralPath $sidecar) {
        Remove-Item -LiteralPath $sidecar -Force
    }
}

Copy-Item -LiteralPath $BackupFile -Destination $dbPath -Force
foreach ($suf in @("-wal", "-shm")) {
    $srcSidecar = "$BackupFile$suf"
    if (Test-Path -LiteralPath $srcSidecar) {
        Copy-Item -LiteralPath $srcSidecar -Destination "$dbPath$suf" -Force
    }
}

Write-Host "[restore] Restored database -> $dbPath" -ForegroundColor Green

$appCandidates = @(
    (Join-Path $env:LOCALAPPDATA "XD4T-1ElLTS\xd4t-ielts65.exe"),
    (Join-Path $PSScriptRoot "xd4t-ielts65.exe"),
    (Join-Path $PSScriptRoot "src-tauri\target\release\xd4t-ielts65.exe")
)
$appExe = $appCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if ($appExe) {
    Start-Process -FilePath $appExe -WorkingDirectory (Split-Path -Parent $appExe)
    Write-Host "[restore] Relaunched $appExe"
}
