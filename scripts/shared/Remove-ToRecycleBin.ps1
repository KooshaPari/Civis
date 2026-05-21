<#
.SYNOPSIS
    Shared helper for safe file/directory deletion via the Windows Recycle Bin.

.DESCRIPTION
    Enforces CLAUDE.md "File Deletion Protocol" mandate:
      "NEVER use rm, del, Remove-Item, or any command that permanently deletes
       files. ALWAYS send files to the Windows Recycle Bin."

    Detects file vs directory automatically. Missing paths are a no-op (logged).

.PARAMETER Path
    Absolute or relative path to a file or directory to send to the Recycle Bin.

.EXAMPLE
    pwsh -File scripts/shared/Remove-ToRecycleBin.ps1 -Path C:\tmp\old.log

.EXAMPLE
    . scripts/shared/Remove-ToRecycleBin.ps1
    Remove-ToRecycleBin -Path C:\tmp\stale-dir

.NOTES
    CI detector: scripts/ci/detect_raw_remove_item.py
    Governance: CLAUDE.md > "File Deletion Protocol (MANDATORY)"
#>
param(
    [Parameter(Mandatory = $false, Position = 0)]
    [string]$Path
)

function Remove-ToRecycleBin {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        Write-Host "[Remove-ToRecycleBin] SKIP (missing): $Path"
        return $true
    }

    try {
        Add-Type -AssemblyName Microsoft.VisualBasic -ErrorAction Stop
    } catch {
        Write-Error "[Remove-ToRecycleBin] Failed to load Microsoft.VisualBasic: $($_.Exception.Message)"
        return $false
    }

    $resolved = (Resolve-Path -LiteralPath $Path).ProviderPath
    $isDir = (Get-Item -LiteralPath $resolved -Force).PSIsContainer

    try {
        if ($isDir) {
            [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory(
                $resolved,
                [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
                [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin)
            Write-Host "[Remove-ToRecycleBin] DIR  -> RecycleBin: $resolved"
        } else {
            [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile(
                $resolved,
                [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
                [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin)
            Write-Host "[Remove-ToRecycleBin] FILE -> RecycleBin: $resolved"
        }
        return $true
    } catch {
        Write-Error "[Remove-ToRecycleBin] FAILED for '$resolved': $($_.Exception.Message)"
        return $false
    }
}

# When invoked directly with -Path (not dot-sourced), execute against that path.
if ($PSBoundParameters.ContainsKey('Path') -and -not [string]::IsNullOrWhiteSpace($Path)) {
    Remove-ToRecycleBin -Path $Path
}
