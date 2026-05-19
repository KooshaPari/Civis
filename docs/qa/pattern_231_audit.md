# Pattern #231 Audit: Static Field Initializers with Side Effects
**Date**: 1779100736.4381647
**Total Violations**: 36
**Severity Breakdown**: HIGH=11, MED=2, LOW=23
**Tier**: mod

## Top Violations
1. `src\Bridge\Client\GameClient.cs:101` — `?` (Environment variable read) [HIGH]
2. `src\SDK\Dependencies\PackSubmoduleManager.cs:266` — `ct` (Process start) [HIGH]
3. `src\SDK\Dependencies\PackSubmoduleManager.cs:266` — `ct` (Process start) [HIGH]
4. `src\SDK\Dependencies\PackSubmoduleManager.cs:290` — `psi` (Process start) [HIGH]
5. `src\SDK\IO\SafeFileIO.cs:16` — `?` (File I/O) [HIGH]
6. `src\SDK\IO\SafeFileIO.cs:16` — `?` (File I/O) [HIGH]
7. `src\SDK\IO\SafeFileIO.cs:19` — `?` (File I/O) [HIGH]
8. `src\SDK\IO\SafeFileIO.cs:19` — `?` (File I/O) [HIGH]
9. `src\SDK\NativeInterop\GoDependencyResolver.cs:177` — `?` (Environment variable read) [HIGH]
10. `src\SDK\NativeInterop\RustAssetPipeline.cs:32` — `_httpClient` (HttpClient instantiation) [HIGH]

... and 26 more violations
