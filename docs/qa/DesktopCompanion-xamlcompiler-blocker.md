# DesktopCompanion XAML Compiler Blocker

**Date**: 2026-05-24  
**Scope**: `src/Tools/DesktopCompanion/**` only

## Current Failure

`dotnet build src/Tools/DesktopCompanion/DesktopCompanion.csproj -v minimal --no-restore`
fails in WinUI XAML compilation with:

```text
MSB3073: ... XamlCompiler.exe ... exited with code 1
```

The generated XAML compiler input for `obj/Debug/net8.0-windows10.0.19041.0/input.json`
shows the project XAML graph is populated normally:

- `App.xaml`
- `MainWindow.xaml`
- `Themes/DinoForgeTheme.xaml`
- `Views/*.xaml`

No actionable source-level XAML error was found in the DesktopCompanion XAML files during this pass.

## Local Toolchain Gaps

The local machine is missing the VC++ toolchain that WinUI XAML compilation depends on:

- `cl.exe` not found in PATH
- Visual Studio Community 2022 is installed, but the `VC\Tools\MSVC` tree is absent under
  `C:\Program Files\Microsoft Visual Studio\2022\Community`
- `vswhere.exe -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -requires Microsoft.VisualStudio.Component.VC.ATL -requires Microsoft.VisualStudio.Component.VC.MFC`
  returned no matching installation

Windows SDK headers/libs are present under `C:\Program Files (x86)\Windows Kits\10\`, so the gap is
the Visual C++ workload, specifically the ATL/MFC-capable toolchain that WinUI XAML compilation
expects.

## Conclusion

This appears to be a toolchain-only blocker, not a DesktopCompanion source/project defect.
No code changes were made.
