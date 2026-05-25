using UnrealBuildTool;
using System.Collections.Generic;

public class CivShowEditorTarget : TargetRules
{
	public CivShowEditorTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Editor;
		DefaultBuildSettings = BuildSettingsVersion.V5;
		IncludeOrderVersion = IncludeOrderVersion.Unreal5_4;

		ExtraModuleNames.AddRange(new string[] { "CivShow" });
	}
}
