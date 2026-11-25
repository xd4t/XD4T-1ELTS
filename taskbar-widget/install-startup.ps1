[CmdletBinding()]
param(
    [switch]$NoLaunch,
    [switch]$ForceRestart
)

$ErrorActionPreference = "Stop"
$exePath = Join-Path $PSScriptRoot "XD4TIeltsBar.exe"
if (-not (Test-Path -LiteralPath $exePath)) {
    & (Join-Path $PSScriptRoot "build.ps1")
}

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
if (-not (Test-Path -LiteralPath $runKey)) {
    New-Item -Path $runKey -Force | Out-Null
}
$value = "`"$exePath`""
Set-ItemProperty -Path $runKey -Name "XD4TIeltsBar" -Value $value -Type String
Write-Host "[install] Added Startup registry entry: HKCU\...\Run\XD4TIeltsBar = $value"

if ($NoLaunch) { return }

if (-not ("XD4TIeltsLauncher" -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public static class XD4TIeltsLauncher
{
    private const uint DESKTOP_ALL = 0x01FF;
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }
    public sealed class WidgetStatus
    {
        public bool OnDefaultDesktop;
        public bool IsVisible;
        public uint Pid;
        public IntPtr Hwnd;
        public int Left, Top, Right, Bottom;
        public string Error;
    }

    [DllImport("kernel32.dll")]
    private static extern IntPtr GetCurrentProcess();
    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool IsProcessInJob(IntPtr ProcessHandle, IntPtr JobHandle, out bool Result);
    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr OpenInputDesktop(uint dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr OpenDesktopW(string lpszDesktop, uint dwFlags, bool fInherit, uint dwDesiredAccess);
    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool SetThreadDesktop(IntPtr hDesktop);
    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool CloseDesktop(IntPtr hDesktop);
    private delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")]
    private static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")]
    private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    public static bool IsInJob()
    {
        bool res = false;
        return IsProcessInJob(GetCurrentProcess(), IntPtr.Zero, out res) && res;
    }

    public static WidgetStatus ProbeOnDefaultDesktop(int[] candidatePids)
    {
        WidgetStatus status = new WidgetStatus();
        HashSet<uint> pidSet = new HashSet<uint>();
        foreach (int p in candidatePids) if (p > 0) pidSet.Add((uint)p);

        Thread t = new Thread(() =>
        {
            IntPtr hDesk = IntPtr.Zero;
            try
            {
                hDesk = OpenInputDesktop(0, false, DESKTOP_ALL);
                if (hDesk == IntPtr.Zero) hDesk = OpenDesktopW("Default", 0, false, DESKTOP_ALL);
                if (hDesk == IntPtr.Zero || !SetThreadDesktop(hDesk)) { status.Error = "Desktop attach failed"; return; }
                EnumWindows((hWnd, lp) =>
                {
                    uint pid;
                    GetWindowThreadProcessId(hWnd, out pid);
                    if (!pidSet.Contains(pid)) return true;
                    status.OnDefaultDesktop = true;
                    status.Pid = pid;
                    RECT r;
                    if (IsWindowVisible(hWnd) && GetWindowRect(hWnd, out r))
                    {
                        int w = r.Right - r.Left;
                        int h = r.Bottom - r.Top;
                        if (w >= 140 && w <= 280 && h >= 28 && h <= 64 && r.Left >= 0)
                        {
                            status.IsVisible = true;
                            status.Hwnd = hWnd;
                            status.Left = r.Left; status.Top = r.Top;
                            status.Right = r.Right; status.Bottom = r.Bottom;
                            return false;
                        }
                    }
                    return true;
                }, IntPtr.Zero);
            }
            catch (Exception ex) { status.Error = ex.Message; }
            finally { if (hDesk != IntPtr.Zero) CloseDesktop(hDesk); }
        });
        t.SetApartmentState(ApartmentState.STA);
        t.Start();
        t.Join();
        return status;
    }
}
'@
}

function Invoke-InteractiveLaunch([string]$ExePath, [string]$WorkDir) {
    $taskName = "XD4TIeltsBar_InteractiveLaunch"
    $svc = New-Object -ComObject "Schedule.Service"
    $svc.Connect()
    $root = $svc.GetFolder("\")
    try { $root.DeleteTask($taskName, 0) } catch { }
    $def = $svc.NewTask(0)
    $def.Principal.LogonType = 3
    $def.Principal.RunLevel = 0
    $def.Settings.Enabled = $true
    $def.Settings.AllowDemandStart = $true
    $def.Settings.DisallowStartIfOnBatteries = $false
    $def.Settings.StopIfGoingOnBatteries = $false
    $def.Settings.ExecutionTimeLimit = "PT0S"
    $action = $def.Actions.Create(0)
    $action.Path = $ExePath
    $action.WorkingDirectory = $WorkDir
    $reg = $root.RegisterTaskDefinition($taskName, $def, 6, $null, $null, 3)
    [void]$reg.Run($null)
    Start-Sleep -Milliseconds 800
    try { $root.DeleteTask($taskName, 0) } catch { }
}

$existing = @(Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue)
if ($existing.Count -gt 0) {
    $pids = @($existing | ForEach-Object { [int]$_.Id })
    $probe = [XD4TIeltsLauncher]::ProbeOnDefaultDesktop($pids)
    if ($ForceRestart -or (-not $probe.OnDefaultDesktop) -or (-not $probe.IsVisible)) {
        Stop-Process -Name "XD4TIeltsBar" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 400
    } else {
        Write-Host "[install] XD4TIeltsBar already running on WinSta0\Default (PID=$($probe.Pid), RECT=($($probe.Left),$($probe.Top),$($probe.Right),$($probe.Bottom)))"
        return
    }
}

Invoke-InteractiveLaunch -ExePath $exePath -WorkDir $PSScriptRoot

$deadline = (Get-Date).AddSeconds(6)
$finalProbe = $null
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 400
    $procs = @(Get-Process -Name "XD4TIeltsBar" -ErrorAction SilentlyContinue)
    if ($procs.Count -gt 0) {
        $pids = @($procs | ForEach-Object { [int]$_.Id })
        $finalProbe = [XD4TIeltsLauncher]::ProbeOnDefaultDesktop($pids)
        if ($finalProbe.OnDefaultDesktop -and $finalProbe.IsVisible) { break }
    }
}

if ($null -eq $finalProbe -or (-not $finalProbe.OnDefaultDesktop) -or (-not $finalProbe.IsVisible)) {
    throw "[install] XD4TIeltsBar failed verification on WinSta0\Default"
}
Write-Host "[install] Verified on WinSta0\Default: PID=$($finalProbe.Pid) HWND=$($finalProbe.Hwnd) RECT=($($finalProbe.Left),$($finalProbe.Top),$($finalProbe.Right),$($finalProbe.Bottom))"
