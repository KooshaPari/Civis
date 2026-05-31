param([string[]]$TypeNames)
$dll = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option_Data\Managed\DNO.Main.dll"
$bytes = [System.IO.File]::ReadAllBytes($dll)
$asm = [System.Reflection.Assembly]::Load($bytes)
foreach ($tn in $TypeNames) {
    $t = $asm.GetType($tn)
    if ($null -eq $t) { Write-Output "TYPE NOT FOUND: $tn"; continue }
    Write-Output "==== $($t.FullName) (base=$($t.BaseType.Name)) ===="
    $flags = [System.Reflection.BindingFlags]::Public -bor [System.Reflection.BindingFlags]::NonPublic -bor [System.Reflection.BindingFlags]::Instance
    foreach ($f in $t.GetFields($flags)) {
        Write-Output ("  FIELD {0} : {1}" -f $f.Name, $f.FieldType.FullName)
    }
    foreach ($p in $t.GetProperties($flags)) {
        Write-Output ("  PROP  {0} : {1}" -f $p.Name, $p.PropertyType.FullName)
    }
}
