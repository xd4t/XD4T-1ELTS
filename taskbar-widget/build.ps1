[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
$srcDir = Join-Path $root "src"
$outExe = Join-Path $root "XD4TIeltsBar.exe"
$manifest = Join-Path $srcDir "app.manifest"

$csc = "C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
if (-not (Test-Path $csc)) {
    throw "csc.exe not found at $csc"
}

$running = Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "[build] Stopping running XD4TIeltsBar instance..."
    if (-not ("XD4TIeltsBuildIpc" -as [type])) {
        Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class XD4TIeltsBuildIpc {
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int RegisterWindowMessageW(string lpString);
    [DllImport("user32.dll")]
    public static extern bool PostMessageW(IntPtr hWnd, int Msg, IntPtr wParam, IntPtr lParam);
}
"@
    }
    $exitMsg = [XD4TIeltsBuildIpc]::RegisterWindowMessageW("XD4TIeltsBar_Exit")
    [void][XD4TIeltsBuildIpc]::PostMessageW([IntPtr]0xFFFF, $exitMsg, [IntPtr]::Zero, [IntPtr]::Zero)
    $deadline = (Get-Date).AddSeconds(2)
    while ((Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $deadline)) {
        Start-Sleep -Milliseconds 150
    }
    Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 200
}

$fwDir = "C:\Windows\Microsoft.NET\Framework64\v4.0.30319"
$wpfDir = Join-Path $fwDir "WPF"
$csFiles = @(Get-ChildItem -Path $srcDir -Filter "*.cs" | Sort-Object Name | ForEach-Object { $_.FullName })
$refs = @(
    "$fwDir\System.dll",
    "$fwDir\System.Core.dll",
    "$fwDir\System.Drawing.dll",
    "$fwDir\System.Windows.Forms.dll",
    "$fwDir\System.Web.Extensions.dll",
    "$wpfDir\UIAutomationClient.dll",
    "$wpfDir\UIAutomationTypes.dll",
    "$wpfDir\WindowsBase.dll"
)

$cscArgs = New-Object System.Collections.Generic.List[string]
$cscArgs.Add("/nologo")
$cscArgs.Add("/optimize+")
$cscArgs.Add("/target:winexe")
$cscArgs.Add("/platform:anycpu")
$cscArgs.Add("/utf8output")
if (Test-Path $manifest) {
    $cscArgs.Add("/win32manifest:`"$manifest`"")
}
$cscArgs.Add("/out:`"$outExe`"")
foreach ($r in $refs) {
    $cscArgs.Add("/reference:`"$r`"")
}
foreach ($f in $csFiles) {
    $cscArgs.Add("`"$f`"")
}

& $csc @cscArgs
if ($LASTEXITCODE -ne 0) {
    throw "csc.exe failed with exit code $LASTEXITCODE"
}

$info = Get-Item $outExe
Write-Host "[build] OK: $($info.FullName) ($($info.Length) bytes)"
