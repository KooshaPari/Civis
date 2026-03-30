$content = Get-Content 'C:\Users\koosh\Dino\src\Tests\DINOForge.Tests.csproj' -Raw
$content = $content -replace '(?s)<!-- Code coverage configuration.*?<PropertyGroup>.*?</PropertyGroup>', @'
  <!-- Code coverage configuration -->
  <PropertyGroup>
    <CollectCoverage>true</CollectCoverage>
    <CoverletOutputFormat>cobertura</CoverletOutputFormat>
    <!-- Coverage gates enforced on every PR. Raise incrementally as tests are added.
         Current state (77.2%): SDK 73.6%, Bridge.Client 56.8%, Economy 80.3%,
         Scenario 74.4%, UI 77.7%, Installer 76.0%, Warfare 95.6%, Protocol 100%
         To raise: add tests for uncovered lines, then bump Threshold. -->
    <Threshold>75</Threshold>
    <ThresholdType>line</ThresholdType>
    <ThresholdStat>total</ThresholdStat>
  </PropertyGroup>
'@
[System.IO.File]::WriteAllText('C:\Users\koosh\Dino\src\Tests\DINOForge.Tests.csproj', $content, [System.Text.Encoding]::UTF8)
Write-Host 'Done'
