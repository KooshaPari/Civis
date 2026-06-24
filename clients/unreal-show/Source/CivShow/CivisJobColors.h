#pragma once

#include "CoreMinimal.h"

/** Job tint palette aligned with Godot `main.gd` JOB_COLORS. */
struct FCivisJobColors
{
    static FLinearColor FromJobName(const FString& Job)
    {
        const FString Key = Job.ToLower();
        if (Key == TEXT("farmer"))
        {
            return FLinearColor(0.49f, 0.85f, 0.34f);
        }
        if (Key == TEXT("warrior"))
        {
            return FLinearColor(1.0f, 0.42f, 0.42f);
        }
        if (Key == TEXT("scholar"))
        {
            return FLinearColor(0.44f, 0.73f, 1.0f);
        }
        if (Key == TEXT("trader"))
        {
            return FLinearColor(1.0f, 0.82f, 0.4f);
        }
        if (Key == TEXT("priest"))
        {
            return FLinearColor(0.75f, 0.52f, 0.99f);
        }
        if (Key == TEXT("admin"))
        {
            return FLinearColor(0.72f, 0.75f, 0.8f);
        }
        return FLinearColor(0.72f, 0.75f, 0.8f);
    }
};
