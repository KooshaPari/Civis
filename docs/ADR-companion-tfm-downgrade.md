# ADR: DesktopCompanion TFM Downgrade to Stable .NET 8 / Windows App SDK 1.6

## Status

Accepted

## Context

DesktopCompanion was previously moved onto a bleeding-edge stack:

- `net11.0-windows10.0.26100.0`
- `Microsoft.WindowsAppSDK` `2.0.0-preview1`
- `LangVersion` `preview`

That direction was intentional at the time, but it made the build dependent on preview tooling and caused host-side XAML compilation friction. The stable runtime path is now available through the installed Windows App Runtime 1.6 channel.

## Decision

Keep the original upgrade comment in `src/Tools/DesktopCompanion/DesktopCompanion.csproj` as historical documentation, but move the project back to the stable LTS stack:

- `net8.0-windows10.0.19041.0`
- `TargetPlatformMinVersion` `10.0.17763.0`
- `Microsoft.WindowsAppSDK` `1.6.250108002`
- `Microsoft.Extensions.*` `8.0.1`
- `LangVersion` `12`

## Consequences

- The companion should build and publish against a stable Windows App SDK line.
- The project stays compatible with the installed Windows App Runtime 1.6 package.
- Any remaining XAML issues now point to net8-compatible markup or API usage instead of preview toolchain drift.
