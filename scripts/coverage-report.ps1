$content = Get-Content -Path 'C:\Users\koosh\Dino\src\Tests\coverage.cobertura.xml' -Raw

# Find all class elements with low line rates
$classMatches = [regex]::Matches($content, '<class[^>]*filename="([^"]+)"[^>]*line-rate="([^"]+)"[^>]*>', [System.Text.RegularExpressions.RegexOptions]::Singleline)

$lowCoverage = @()
foreach ($m in $classMatches) {
    $file = $m.Groups[1].Value
    $rate = [double]$m.Groups[2].Value
    if ($rate -lt 0.7 -and $file -notmatch 'AssemblyInfo') {
        $lowCoverage += [PSCustomObject]@{ File = $file; Rate = $rate }
    }
}

$lowCoverage | Sort-Object Rate | Format-Table -AutoSize
