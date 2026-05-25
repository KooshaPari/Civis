using System;
using System.IO;
using UnrealBuildTool;

public class CivShow : ModuleRules
{
    public CivShow(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;

        PublicDependencyModuleNames.AddRange(new[]
        {
            "Core",
            "CoreUObject",
            "Engine",
            "InputCore",
            "HTTP",
            "Json",
            "JsonUtilities",
            "ProceduralMeshComponent",
        });

        PrivateDependencyModuleNames.AddRange(new[]
        {
            "Slate",
            "SlateCore",
            "WebSockets",
        });

        if (Target.Platform == UnrealTargetPlatform.Win64)
        {
            string ModuleDir = ModuleDirectory;
            string LibDir = Path.GetFullPath(Path.Combine(ModuleDir, "..", "..", "Civis", "lib"));
            string IncludeDir = Path.GetFullPath(Path.Combine(ModuleDir, "..", "..", "Civis", "include"));

            PublicIncludePaths.Add(IncludeDir);
            PublicAdditionalLibraries.Add(Path.Combine(LibDir, "civis_unreal_ffi.lib"));

#if UE_5_4_OR_LATER
            PrivateDependencyModuleNames.Add("HTTPServer");
#endif
        }
    }
}
