# PowerShell script to properly test game UI with window activation

Add-Type @"
using System;
using System.Runtime.InteropServices;

public class WindowHelper {
    [DllImport("user32.dll")]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool IsIconic(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    public static void ActivateWindow(string windowTitle) {
        IntPtr hWnd = FindWindow(null, windowTitle);
        if (hWnd == IntPtr.Zero) {
            Console.WriteLine("[ERROR] Window not found: " + windowTitle);
            return;
        }

        if (IsIconic(hWnd)) {
            ShowWindow(hWnd, 9); // SW_RESTORE
        }

        SetForegroundWindow(hWnd);
        System.Threading.Thread.Sleep(500);
        Console.WriteLine("[OK] Window activated: " + windowTitle);
    }
}
"@

function Take-Screenshot {
    param([string]$Path)
    $image = [System.Windows.Forms.Screen]::PrimaryScreen.Capture()
    $image.Save($Path)
    Write-Host "[SCREENSHOT] Saved: $Path"
}

function Press-Key {
    param([string]$Key)
    $keyCode = [System.Windows.Forms.Keys]::$Key
    $simulator = New-Object "System.Windows.Forms.SendKeys"
    [System.Windows.Forms.SendKeys]::SendWait($Key)
    Start-Sleep -Milliseconds 300
    Write-Host "[KEY] Pressed: $Key"
}

# Load Windows.Forms for screenshot
[System.Reflection.Assembly]::LoadAssemblyName("System.Windows.Forms") | Out-Null

Write-Host "================================"
Write-Host "DINOForge UI Test"
Write-Host "================================"

# Activate game window
$gameWindowTitle = "Diplomacy is Not an Option"
Write-Host "[ACTION] Activating game window..."
[WindowHelper]::ActivateWindow($gameWindowTitle)

Start-Sleep -Seconds 2

# Baseline
Write-Host "[TEST 1] Taking baseline screenshot..."
Take-Screenshot "C:\Users\koosh\Dino\ui_test_screenshots\ps1_baseline.png"

Start-Sleep -Seconds 1

# F10
Write-Host "[TEST 1] Pressing F10..."
[System.Windows.Forms.SendKeys]::SendWait("{F10}")
Start-Sleep -Seconds 3

Write-Host "[TEST 1] Capturing F10 pack list..."
Take-Screenshot "C:\Users\koosh\Dino\ui_test_screenshots\ps1_f10_pack_list.png"

Start-Sleep -Seconds 1

# Close F10
Write-Host "[TEST 1] Closing F10..."
[System.Windows.Forms.SendKeys]::SendWait("{F10}")
Start-Sleep -Seconds 2

# F9
Write-Host "[TEST 2] Pressing F9..."
[System.Windows.Forms.SendKeys]::SendWait("{F9}")
Start-Sleep -Seconds 3

Write-Host "[TEST 2] Capturing F9 debug panel..."
Take-Screenshot "C:\Users\koosh\Dino\ui_test_screenshots\ps1_f9_debug.png"

Start-Sleep -Seconds 1

# Close F9
Write-Host "[TEST 2] Closing F9..."
[System.Windows.Forms.SendKeys]::SendWait("{F9}")
Start-Sleep -Seconds 2

Write-Host ""
Write-Host "================================"
Write-Host "Tests Complete!"
Write-Host "================================"
Write-Host "Check screenshots to see:"
Write-Host "  1. ps1_f10_pack_list.png - Pack list sidebar visible?"
Write-Host "  2. ps1_f9_debug.png - Debug panel visible?"
