param(
    [Parameter(Mandatory=$true)][string]$OutputPath,
    [string]$ProcessName = 'Diplomacy is Not an Option'
)
$ErrorActionPreference = 'Stop'

Add-Type @"
using System;
using System.Text;
using System.Collections.Generic;
using System.Drawing;
using System.Runtime.InteropServices;
public static class PW {
    public delegate bool EnumProc(IntPtr hwnd, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr lp);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll")] public static extern int GetClassName(IntPtr hwnd, StringBuilder s, int max);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int n);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

    public static List<IntPtr> Windows(uint targetPid) {
        var list = new List<IntPtr>();
        EnumWindows((h, l) => {
            uint pid; GetWindowThreadProcessId(h, out pid);
            if (pid == targetPid && IsWindowVisible(h)) list.Add(h);
            return true;
        }, IntPtr.Zero);
        return list;
    }
    public static string Cls(IntPtr h) { var sb = new StringBuilder(256); GetClassName(h, sb, 256); return sb.ToString(); }
}
"@ -ReferencedAssemblies System.Drawing, System.Collections, System.Runtime, System.Drawing.Primitives

$proc = Get-Process -Name $ProcessName -ErrorAction Stop | Select-Object -First 1
$wins = [PW]::Windows([uint32]$proc.Id)
"Found $($wins.Count) visible windows for pid $($proc.Id):"
$target = $null; $bestArea = 0
foreach ($h in $wins) {
    $cls = [PW]::Cls($h)
    $r = New-Object PW+RECT; [PW]::GetWindowRect($h, [ref]$r) | Out-Null
    $w = $r.Right - $r.Left; $ht = $r.Bottom - $r.Top
    "  hwnd=$h class='$cls' ${w}x${ht}"
    # Unity render window class is UnityWndClass; pick it, else largest non-console
    if ($cls -eq 'UnityWndClass') { $target = $h }
    elseif (-not $target -and $cls -ne 'ConsoleWindowClass' -and ($w*$ht) -gt $bestArea) { $bestArea = $w*$ht }
}
if (-not $target) {
    # fallback: largest visible window that's not the console
    $target = ($wins | Where-Object { [PW]::Cls($_) -ne 'ConsoleWindowClass' } | Sort-Object {
        $r = New-Object PW+RECT; [PW]::GetWindowRect($_, [ref]$r) | Out-Null; ($r.Right-$r.Left)*($r.Bottom-$r.Top)
    } -Descending | Select-Object -First 1)
}
if (-not $target) { throw "No Unity render window found" }
"TARGET hwnd=$target class='$([PW]::Cls($target))'"

[PW]::ShowWindow($target, 9) | Out-Null
[PW]::SetForegroundWindow($target) | Out-Null
Start-Sleep -Milliseconds 1000

$rc = New-Object PW+RECT; [PW]::GetClientRect($target, [ref]$rc) | Out-Null
$w = $rc.Right - $rc.Left; $ht = $rc.Bottom - $rc.Top
"client ${w}x${ht}"
$bmp = New-Object System.Drawing.Bitmap($w, $ht)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
$ok = [PW]::PrintWindow($target, $hdc, 2)   # PW_RENDERFULLCONTENT
$g.ReleaseHdc($hdc); $g.Dispose()
if (-not $ok) { Write-Warning "PrintWindow returned false" }
$dir = Split-Path $OutputPath
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
$bmp.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
"SAVED $OutputPath ($((Get-Item $OutputPath).Length) bytes)"
