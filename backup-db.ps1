[CmdletBinding()]
param(
    [string]$OutDir = "."
)

$ErrorActionPreference = "Stop"
$dbPath = Join-Path $env:APPDATA "xd4t-ielts65\ielts65.db"
if (-not (Test-Path -LiteralPath $dbPath)) {
    throw "Database not found at $dbPath"
}

if (-not (Test-Path -LiteralPath $OutDir)) {
    New-Item -ItemType Directory -Path $OutDir -Force | Out-Null
}
$resolvedOut = (Resolve-Path -LiteralPath $OutDir).Path
$stamp = Get-Date -Format "yyyyMMdd-HHmm"
$backupFile = Join-Path $resolvedOut "ielts65-backup-$stamp.db"

$python = Get-Command python -ErrorAction SilentlyContinue
if ($python) {
    & $python.Source -c "import sqlite3, sys; src=sqlite3.connect(sys.argv[1]); dst=sqlite3.connect(sys.argv[2]); src.backup(dst); dst.close(); src.close()" $dbPath $backupFile
    if ($LASTEXITCODE -ne 0) {
        throw "SQLite online backup failed"
    }
} else {
    $wasRunning = [bool](Get-Process -Name "xd4t-ielts65" -ErrorAction SilentlyContinue)
    if ($wasRunning) {
        Write-Warning "Python not found; stopping xd4t-ielts65.exe briefly for consistent file copy..."
        Get-Process -Name "xd4t-ielts65" | Stop-Process -Force -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 400
    }
    Copy-Item -LiteralPath $dbPath -Destination $backupFile -Force
    foreach ($suf in @("-wal", "-shm")) {
        $wal = "$dbPath$suf"
        if (Test-Path -LiteralPath $wal) {
            Copy-Item -LiteralPath $wal -Destination "$backupFile$suf" -Force
        }
    }
}

$info = Get-Item -LiteralPath $backupFile
Write-Host "[backup] OK: $($info.FullName) ($($info.Length) bytes)" -ForegroundColor Green
