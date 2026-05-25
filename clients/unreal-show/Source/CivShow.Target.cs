using UnrealBuildTool;
using System.Collections.Generic;

public class CivShowTarget : TargetRules
{
	public CivShowTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Game;
		DefaultBuildSettings = BuildSettingsVersion.V5;
		IncludeOrderVersion = IncludeOrderVersion.Unreal5_4;

		ExtraModuleNames.AddRange(new string[] { "CivShow" });
	}
}
