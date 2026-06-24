using UnrealBuildTool;
using System.Collections.Generic;

public class CivShowTarget : TargetRules
{
	public CivShowTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Game;
		DefaultBuildSettings = BuildSettingsVersion.V6;
		IncludeOrderVersion = global::UnrealBuildTool.EngineIncludeOrderVersion.Unreal5_7;

		ExtraModuleNames.AddRange(new string[] { "CivShow" });
	}
}
