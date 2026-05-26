$ErrorActionPreference = 'SilentlyContinue'
$results = @()
Get-ChildItem -Path 'C:\Users\koosh\Dino\src\Tests' -Filter '*.cs' -Recurse | Where-Object { $_.Name -notmatch 'AssemblyInfo|GlobalUsings|\.g\.cs' } | ForEach-Object {
    $lines = (Get-Content $_.FullName).Count
    $results += [PSCustomObject]@{
        File = $_.FullName.Replace('C:\Users\koosh\Dino\src\Tests\', '')
        Lines = $lines
        Size = $_.Length
    }
}
$results | Sort-Object Lines -Descending | Format-Table File, Lines, Size -AutoSize
$total = ($results | Measure-Object Lines -Sum).Sum
Write-Host ""
Write-Host "Total: $($results.Count) files, $total lines"
