# DINOForge NuGet Package Publishing Guide

## Status: Ready for Publishing (Milestone M10)

Date: 2026-03-25

### Packages Generated

**Location**: `./nupkg/`

| Package | File | Size | Type | Status |
|---------|------|------|------|--------|
| **DINOForge.SDK** | `DINOForge.SDK.0.3.0.nupkg` | 91 KB | Main | ✅ Ready |
| **DINOForge.SDK** | `DINOForge.SDK.0.3.0.snupkg` | 58 KB | Symbols | ✅ Ready |
| **DINOForge.Bridge.Protocol** | `DINOForge.Bridge.Protocol.0.3.0.nupkg` | 17 KB | Main | ✅ Ready |

### NuGet Metadata Audit Results

#### DINOForge.SDK (src/SDK/DINOForge.SDK.csproj)
- ✅ **TargetFramework**: netstandard2.0 (broad compatibility)
- ✅ **PackageId**: DINOForge.SDK
- ✅ **Version**: 0.3.0
- ✅ **Authors**: KooshaPari
- ✅ **Description**: Complete mod platform SDK
- ✅ **PackageLicenseExpression**: MIT
- ✅ **PackageProjectUrl**: https://github.com/KooshaPari/Dino
- ✅ **RepositoryUrl**: https://github.com/KooshaPari/Dino
- ✅ **RepositoryType**: git
- ✅ **PackageReadmeFile**: README.md
- ✅ **GenerateDocumentationFile**: true (XML docs included)
- ✅ **IncludeSymbols**: true
- ✅ **SymbolPackageFormat**: snupkg
- ✅ **IsPackable**: true
- ✅ **PackageTags**: dinoforge;gamemod;diplomacy-is-not-an-option;bepinex;unity-ecs

#### DINOForge.Bridge.Protocol (src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj)
- ✅ **TargetFramework**: netstandard2.0 (broad compatibility)
- ✅ **PackageId**: DINOForge.Bridge.Protocol
- ✅ **Version**: 0.3.0
- ✅ **Authors**: KooshaPari
- ✅ **Description**: JSON-RPC 2.0 protocol DTOs for game bridge
- ✅ **PackageLicenseExpression**: MIT
- ✅ **PackageProjectUrl**: https://github.com/KooshaPari/Dino
- ✅ **RepositoryUrl**: https://github.com/KooshaPari/Dino
- ✅ **RepositoryType**: git
- ✅ **GenerateDocumentationFile**: true (XML docs included)
- ✅ **IsPackable**: true
- ✅ **PackageTags**: dinoforge;bridge;protocol;json-rpc;bepinex

### Publish Instructions

#### Option 1: Publish with API Key (Automated)
```bash
export NUGET_API_KEY="your-api-key-here"
dotnet nuget push ./nupkg/DINOForge.SDK.0.3.0.nupkg --api-key $NUGET_API_KEY --source https://api.nuget.org/v3/index.json --skip-duplicate
dotnet nuget push ./nupkg/DINOForge.SDK.0.3.0.snupkg --api-key $NUGET_API_KEY --source https://api.nuget.org/v3/index.json --skip-duplicate
dotnet nuget push ./nupkg/DINOForge.Bridge.Protocol.0.3.0.nupkg --api-key $NUGET_API_KEY --source https://api.nuget.org/v3/index.json --skip-duplicate
```

#### Option 2: Publish with Interactive Prompt
```bash
dotnet nuget push ./nupkg/DINOForge.SDK.0.3.0.nupkg --source https://api.nuget.org/v3/index.json --skip-duplicate
dotnet nuget push ./nupkg/DINOForge.SDK.0.3.0.snupkg --source https://api.nuget.org/v3/index.json --skip-duplicate
dotnet nuget push ./nupkg/DINOForge.Bridge.Protocol.0.3.0.nupkg --source https://api.nuget.org/v3/index.json --skip-duplicate
```

#### Option 3: Use Local NuGet.config
```bash
# Configure credentials in ~/.nuget/NuGet/NuGet.Config
dotnet nuget push ./nupkg/*.nupkg --source https://api.nuget.org/v3/index.json --skip-duplicate
```

### Package Contents Verification

#### DINOForge.SDK.0.3.0.nupkg
- ✅ DLL: `lib/netstandard2.0/DINOForge.SDK.dll` (138 KB)
- ✅ XML Documentation: `lib/netstandard2.0/DINOForge.SDK.xml` (171 KB)
- ✅ README: Included (11 KB)
- ✅ Nuspec: Valid metadata

#### DINOForge.SDK.0.3.0.snupkg
- ✅ PDB: `lib/netstandard2.0/DINOForge.SDK.pdb` (55 KB)
- ✅ Source debugging support enabled

#### DINOForge.Bridge.Protocol.0.3.0.nupkg
- ✅ DLL: `lib/netstandard2.0/DINOForge.Bridge.Protocol.dll` (26 KB)
- ✅ XML Documentation: `lib/netstandard2.0/DINOForge.Bridge.Protocol.xml` (30 KB)
- ✅ Nuspec: Valid metadata

### Notes

- **XML Documentation**: Both packages include full XML documentation files for IntelliSense support in Visual Studio, Rider, and VS Code.
- **Symbol Debugging**: Symbol package (.snupkg) included for SDK enables source-level debugging of NuGet-consumed SDK code.
- **License**: MIT (compatible with commercial and open-source projects)
- **Compatibility**: Both packages target netstandard2.0 for broad framework support (.NET Framework 4.6.1+, .NET 5.0+, .NET 6.0+, etc.)
- **Dependency Chain**: SDK depends on YamlDotNet, Newtonsoft.Json, NJsonSchema, AssetsTools.NET, System.Text.Json. Bridge.Protocol depends only on Newtonsoft.Json.

### Bridge.Protocol Warnings

Warning: Missing readme for Bridge.Protocol. This is non-blocking. Optional: add `<PackageReadmeFile>` entry to point to a dedicated README or just use the repo README.

### Next Steps

1. Set the `NUGET_API_KEY` environment variable with your nuget.org API key
2. Run the publish command from Option 1 above
3. Verify on https://www.nuget.org/packages/DINOForge.SDK/ and https://www.nuget.org/packages/DINOForge.Bridge.Protocol/
4. Update `CHANGELOG.md` to reflect the release (version 0.3.0)
5. Create a GitHub release tag (v0.3.0) linking to CHANGELOG

