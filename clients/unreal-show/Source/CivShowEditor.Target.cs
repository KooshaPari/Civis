using UnrealBuildTool;
using System.Collections.Generic;

public class CivShowEditorTarget : TargetRules
{
	public CivShowEditorTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Editor;
		DefaultBuildSettings = BuildSettingsVersion.V6;
		IncludeOrderVersion = global::UnrealBuildTool.EngineIncludeOrderVersion.Unreal5_7;

		ExtraModuleNames.AddRange(new string[] { "CivShow" });
	}
}
