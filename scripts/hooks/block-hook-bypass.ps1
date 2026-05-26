#Requires -Version 7.0
<#
.SYNOPSIS
PreToolUse hook to block git/lefthook bypass attempts.
Forbidden per CLAUDE.md governance (feedback_no_verify_forbidden, feedback_no_lefthook_bypass).
#>

param()

# Read from pipeline stdin only if redirected (avoid blocking in interactive shell)
$input = $null
if ([Console]::IsInputRedirected) {
    $input = [Console]::In.ReadToEnd()
}
if ([string]::IsNullOrWhiteSpace($input)) {
    $input = $env:CLAUDE_TOOL_INPUT
}

# Forbidden substrings (case-sensitive for env vars, case-insensitive for flags)
$forbidden = @(
    @{ pattern = '--no-verify';              rule = 'feedback_no_verify_forbidden' },
    @{ pattern = 'LEFTHOOK=0';                rule = 'feedback_no_lefthook_bypass' },
    @{ pattern = 'LEFTHOOK_EXCLUDE';          rule = 'feedback_no_lefthook_bypass' },
    @{ pattern = '--no-gpg-sign';             rule = 'feedback_no_verify_forbidden' },
    @{ pattern = '-c commit.gpgsign=false';   rule = 'feedback_no_verify_forbidden' },
    @{ pattern = '-c core.hooksPath=';        rule = 'feedback_no_verify_forbidden' }
)

try {
    if ($input -like '*"command"*') {
        $payload = $input | ConvertFrom-Json
        $cmd = $payload.tool_input.command
    } else {
        $cmd = $input
    }

    if ($null -eq $cmd) {
        exit 0
    }

    foreach ($f in $forbidden) {
        if ($cmd -like "*$($f.pattern)*") {
            Write-Error "Hook-bypass attempt BLOCKED: '$($f.pattern)' is forbidden per CLAUDE.md governance ($($f.rule)). Fix the underlying hook failure instead of bypassing."
            exit 2
        }
    }

    exit 0
} catch {
    Write-Error "Hook error: $_"
    exit 1
}
