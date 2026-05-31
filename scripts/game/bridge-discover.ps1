param([string]$Method = "status", [string]$ParamsJson = "{}")
$pipeName = "dinoforge-game-bridge"
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", $pipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::Asynchronous)
$pipe.Connect(3000)
$req = @{ jsonrpc = "2.0"; id = 1; method = $Method; params = ($ParamsJson | ConvertFrom-Json) } | ConvertTo-Json -Depth 10 -Compress
$writer = New-Object System.IO.StreamWriter($pipe)
$writer.AutoFlush = $true
$writer.WriteLine($req)
$reader = New-Object System.IO.StreamReader($pipe)
$line = $reader.ReadLine()
$line
$pipe.Close()
