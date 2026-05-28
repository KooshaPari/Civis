Option Explicit
Dim sh, logPath, cmd
Set sh = CreateObject("WScript.Shell")
logPath = "C:\Users\koosh\Dino\scripts\coderabbit-orchestration.log"
cmd = "cmd /c ""C:\Users\koosh\.cursor\fix-cursor-shell.cmd && C:\Users\koosh\Dino\scripts\coderabbit-main-config.bat > """ & logPath & """ 2>&1"""
sh.Run cmd, 0, True
WScript.Echo "Done. Log: " & logPath
