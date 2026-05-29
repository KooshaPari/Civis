# Security Review: Iter-148 Session Commits (2026-05-28)

**Reviewed Commits**: 14 (0cf468b4 through 7fdb1ffc)  
**Review Date**: 2026-05-28  
**Severity Distribution**: 3 HIGH, 5 MEDIUM, 7 LOW  

## Executive Summary

The recent session adds significant new CLI tooling (build/deploy/relaunch), UI features (F10 detail pane with external links), and development tool bundling (UnityExplorer download). Critical security issues center on **unsanitized URL passing to Application.OpenURL** (HIGH), **unvalidated file downloads from GitHub** (HIGH), and **shell script injection via user-controlled paths** (HIGH). Two additional MEDIUM-severity issues involve path traversal risks and unsafe process execution.

**Recommendation**: All HIGH findings require fixes before merge. MEDIUM findings should be addressed in follow-up PRs.

---

## HIGH-SEVERITY FINDINGS

### Finding 1: Application.OpenURL (Unvalidated User-Supplied URLs)
**Severity**: HIGH  
**Commits**: 427323a2, 2510e6cf  
**Files**: 
- `src/Runtime/UI/ModMenuOverlay.cs` (line ~500)
- `src/Runtime/UI/ModMenuPanel.cs` (added in 2510e6cf)

**What's wrong**:
The F10 detail pane accepts `HomepageUrl`, `GithubUrl`, and `DiscordUrl` directly from pack.yaml manifests and passes them to `Application.OpenURL()` without validation:

```csharp
try { Application.OpenURL(capturedUrl); }  // 2510e6cf
```

**Attack vector**:
1. User (or attacker) crafts malicious pack.yaml with `homepage_url: "javascript:alert('xss')"` or `"file:///etc/passwd"`
2. User loads the pack via `dinoforge install` or manual placement
3. User clicks the "Homepage" button in F10 mod menu
4. Application.OpenURL **opens the arbitrary URL in the system default browser**
5. Browser executes the payload (XSS, RCE via file:// protocol, or JavaScript protocol handlers)

**Risk**: 
- RCE if browser is configured to handle `javascript://` or custom protocol handlers
- Information disclosure via `file://` paths
- Redirect attacks (social engineering via malicious Discord link)
- Depends on OS and browser configuration, but assumed unsafe-by-default

**Recommended fix**:
```csharp
// Validate URL format before opening
if (!Uri.TryCreate(capturedUrl, UriKind.Absolute, out Uri? uri))
{
    _logger.LogWarning("Invalid URL format: {0}", capturedUrl);
    return;
}

// Allow only http/https schemes
if (uri.Scheme != Uri.UriSchemeHttp && uri.Scheme != Uri.UriSchemeHttps)
{
    _logger.LogWarning("URL scheme not allowed: {0}", uri.Scheme);
    return;
}

// Validate hostname is public (not localhost, file://, etc.)
if (uri.IsLoopback || uri.IsFile)
{
    _logger.LogWarning("URL points to loopback or file: {0}", uri);
    return;
}

try { Application.OpenURL(uri.AbsoluteUri); }
catch (Exception ex) { _logger.LogError(ex, "Failed to open URL"); }
```

---

### Finding 2: Unvalidated GitHub Release URL Download (install-dev-tools.ps1)
**Severity**: HIGH  
**Commits**: 0cf468b4  
**Files**: `scripts/install-dev-tools.ps1` (lines 56–71), `src/Tools/Cli/Commands/DevToolsCommand.cs`

**What's wrong**:
The PowerShell script downloads UnityExplorer.BepInEx.Mono.zip from GitHub releases without cryptographic signature verification:

```powershell
$releaseUri = 'https://api.github.com/repos/sinai-dev/UnityExplorer/releases/latest'
$release = Invoke-RestMethod -Uri $releaseUri -Headers $headers -ErrorAction Stop
foreach ($asset in $release.assets) {
    if ($asset.name -eq 'UnityExplorer.BepInEx.Mono.zip') {
        return $asset.browser_download_url  # Direct use of attacker-controlled URL
    }
}

$downloadUrl = Get-LatestUnityExplorerZipUrl
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing -ErrorAction Stop
```

**Attack vector**:
1. **GitHub API compromise**: If GitHub API is compromised (or MITM on unencrypted admin network), attacker can return malicious download URL or asset URL
2. **Release file replacement**: If UnityExplorer GitHub account is compromised, attacker can replace the .zip with malware
3. **DNS hijacking**: HTTPS is used, but if root CA is compromised, attacker can serve malware from a spoofed github.com
4. **No integrity check**: Downloaded file is extracted and executed (BepInEx DLL) without hash verification

**Risk**: 
- Installation of arbitrary DLL into BepInEx plugins directory (RCE on next game launch)
- Trojanized mod development environment
- Silent persistence (user won't detect malicious plugin)

**Recommended fix**:
```powershell
# Option 1: Publish a SHA256 checksum alongside the script (most practical)
$expectedHash = "abc123def456..."  # Published in docs/dev-tools/checksums.txt
$downloadedHash = (Get-FileHash $tempZip -Algorithm SHA256).Hash
if ($downloadedHash -ne $expectedHash) {
    Write-Error "Downloaded file hash mismatch. Expected: $expectedHash, Got: $downloadedHash"
    exit 1
}

# Option 2: Use GitHub release signatures (requires signing the release)
# This requires upstream UnityExplorer to sign releases (unlikely)

# Option 3: Bundle the plugin in the repo (avoids download)
# Recommended: commit UnityExplorer.BepInEx.Mono.zip to .gitignore-tracked assets/
```

---

### Finding 3: Shell Injection via User-Controlled Path (install-dev-tools.sh)
**Severity**: HIGH  
**Commits**: 0cf468b4  
**Files**: `scripts/install-dev-tools.sh` (lines 68–84, 132)

**What's wrong**:
The Bash script uses user-controlled game paths in shell commands without proper quoting:

```bash
install_dir="${game_path%/}/BepInEx/plugins/dev/UnityExplorer"  # User path
mkdir -p "$install_dir"  # Correctly quoted

# Later:
cp -R "$plugin_dir"/. "$install_dir"/  # Correctly quoted in this part
```

However, the zip URL itself is passed to `download_zip()` without validation, and the `unzip` call uses variable substitution:

```bash
download_zip() {
  local url="$1"
  local out="$2"
  
  if command -v curl >/dev/null 2>&1; then
    curl -fL --retry 3 --retry-delay 1 -o "$out" "$url"  # Properly quoted
  fi
}

# But game_path is used in directory construction:
mkdir -p "$install_dir"
cp -R "$plugin_dir"/. "$install_dir"/  # Path traversal possible if plugin_dir contains ..
```

**Attack vector**:
If a user sets `GAME_PATH=/tmp/game/../etc/passwd` or similar, directory traversal could occur:
```bash
export GAME_PATH="/tmp/game/../../../etc"
./install-dev-tools.sh  # Attempts cp -R into /etc/plugins/dev/UnityExplorer
```

More critical: if `$plugin_dir` is extracted from the .zip without sanitization and contains `../`, the `cp -R` would traverse outside intended directory.

**Risk**: 
- Directory traversal during extraction
- Writing files to arbitrary locations
- Potential privilege escalation if run with elevated privileges

**Recommended fix**:
```bash
# Validate game path (must exist and not contain ..)
if [[ "$game_path" == *".."* ]]; then
  err "Game path contains '..' — rejecting for safety: $game_path"
  return 1
fi

# Canonicalize path to resolve any .. or symlinks
if ! game_path="$(cd "$game_path" 2>/dev/null && pwd)"; then
  err "Game path does not exist: $game_path"
  return 1
fi

# Validate extracted plugin directory before copy
if [[ "$plugin_dir" == *".."* ]]; then
  err "Extracted plugin path contains '..' — rejecting: $plugin_dir"
  return 1
fi
```

---

## MEDIUM-SEVERITY FINDINGS

### Finding 4: Path Traversal in Pack Deployment (DeployCommand.cs)
**Severity**: MEDIUM  
**Commits**: 1d33275a  
**Files**: `src/Tools/Cli/Commands/DeployCommand.cs` (lines 107–119)

**What's wrong**:
The pack deployment mirrors all files from `packs/<name>/` to the game directory without validating for path traversal:

```csharp
foreach (string file in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
{
    string rel = Path.GetRelativePath(source, file);
    string destFile = Path.Combine(dest, rel);  // rel could be ../../evil.dll
    Directory.CreateDirectory(Path.GetDirectoryName(destFile)!);
    File.Copy(file, destFile, overwrite: true);
}
```

If a pack contains a file named `../../../BepInEx/plugins/evil.dll`, `Path.GetRelativePath()` would preserve the `../` sequences, allowing writes outside the intended pack directory.

**Attack vector**:
1. Attacker publishes a malicious pack with `../../../../../../Windows/System32/evil.dll` in the pack archive
2. User downloads and installs the pack via `dinoforge deploy`
3. The `rel` path calculation preserves the `../` sequences
4. File is copied to an arbitrary location on the system

**Risk**: 
- Writing arbitrary files to system directories
- DLL injection into system directories
- Privilege escalation if run as admin

**Recommended fix**:
```csharp
foreach (string file in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
{
    string rel = Path.GetRelativePath(source, file);
    
    // Reject any .. path segments
    if (rel.Contains("..") || rel.StartsWith("/") || rel.StartsWith("\\"))
    {
        AnsiConsole.MarkupLine($"[yellow]Skipping file with dangerous path:[/] {Markup.Escape(rel)}");
        continue;
    }
    
    string destFile = Path.Combine(dest, rel);
    
    // Final safety check: ensure destFile is still within dest
    string fullDest = Path.GetFullPath(destFile);
    string fullDestBase = Path.GetFullPath(dest);
    if (!fullDest.StartsWith(fullDestBase + Path.DirectorySeparatorChar) && fullDest != fullDestBase)
    {
        AnsiConsole.MarkupLine($"[red]Path traversal detected:[/] {Markup.Escape(destFile)}");
        return 1;
    }
    
    Directory.CreateDirectory(Path.GetDirectoryName(destFile)!);
    File.Copy(file, destFile, overwrite: true);
}
```

---

### Finding 5: Unsafe Process.Start Without Timeout/Cleanup (RelaunchCommand.cs)
**Severity**: MEDIUM  
**Commits**: 1d33275a  
**Files**: `src/Tools/Cli/Commands/RelaunchCommand.cs` (lines 87–115)

**What's wrong**:
The relaunch command starts the game process but does not wrap it in a `using` statement or ensure disposal on early exit:

```csharp
Process? launched;
try
{
    ProcessStartInfo psi = new(exePath) { ... };
    launched = Process.Start(psi);
}
catch (Exception ex) { ... return 1; }

if (launched is null) { ... return 1; }

// Wait and then dispose later, but if an exception occurs in between, Dispose is not called
try { await Task.Delay(...); }
finally { /* no dispose here */ }

// Dispose only in the outer finally block
finally { foreach (Process p in postLaunch) p.Dispose(); }
```

If an exception occurs before the outer `finally`, the handle for `launched` is not disposed.

**Risk**: 
- Process handle leak (Handles are a limited OS resource; ~65k per process)
- Repeated invocations could exhaust the handle table
- Symptom: "Too many open files" or "Cannot allocate memory" on subsequent launches

**Recommended fix**:
```csharp
Process? launched = null;
try
{
    ProcessStartInfo psi = new(exePath) { ... };
    launched = Process.Start(psi) ?? throw new InvalidOperationException("Process.Start returned null");
    
    // Wait
    await Task.Delay(TimeSpan.FromSeconds(waitSeconds), ct).ConfigureAwait(false);
    
    // Verify post-launch
    Process[] postLaunch = Process.GetProcessesByName(ProcessName);
    using var _ = postLaunch;  // Dispose all query results
    
    if (postLaunch.Length == 0) { ... return 1; }
    // ... rest of verification
}
finally
{
    launched?.Dispose();  // Always dispose the launched process
}
```

---

### Finding 6: No Validation of Pack Manifest URLs (PackManifest.cs)
**Severity**: MEDIUM  
**Commits**: 427323a2  
**Files**: `src/SDK/PackManifest.cs` (new fields)

**What's wrong**:
New fields are added to `PackManifest` without schema validation:

```csharp
[YamlMember(Alias = "homepage_url")]
public string? HomepageUrl { get; set; }

[YamlMember(Alias = "github_url")]
public string? GithubUrl { get; set; }

[YamlMember(Alias = "discord_url")]
public string? DiscordUrl { get; set; }
```

These are deserialized directly from YAML without URI format validation in the schema or in code.

**Risk**: 
- Allows attacker-controlled input to flow through without validation
- Pairs with Finding 1 (unvalidated Application.OpenURL call)
- Breaks the "validate early" principle

**Recommended fix**:
Add JSON schema validation to `schemas/pack.schema.json`:

```json
{
  "properties": {
    "homepage_url": {
      "type": ["string", "null"],
      "format": "uri",
      "pattern": "^https?://"
    },
    "github_url": {
      "type": ["string", "null"],
      "format": "uri",
      "pattern": "^https://github\\.com/"
    },
    "discord_url": {
      "type": ["string", "null"],
      "format": "uri",
      "pattern": "^https://(discord\\.gg|discordapp\\.com)"
    }
  }
}
```

And validate in code:
```csharp
public void Validate()
{
    if (!string.IsNullOrEmpty(HomepageUrl) && !Uri.TryCreate(HomepageUrl, UriKind.Absolute, out var uri))
        throw new ValidationException($"Invalid homepage_url: {HomepageUrl}");
    
    if (uri?.Scheme != Uri.UriSchemeHttps)
        throw new ValidationException($"Homepage URL must use HTTPS: {HomepageUrl}");
}
```

---

## LOW-SEVERITY FINDINGS

### Finding 7: Process Handle Leak Pattern (Pattern #102 violation)
**Severity**: LOW  
**Commits**: 0cf468b4, 1d33275a  
**Files**: 
- `src/Tools/Cli/Commands/DevToolsCommand.cs` (line ~170)
- `src/Tools/Cli/Commands/RelaunchCommand.cs` (line ~60)

**What's wrong**:
Process objects are not always disposed immediately after use, violating Pattern #102:

```csharp
Process[] existing = Process.GetProcessesByName(ProcessName);
if (existing.Length > 0)
{
    foreach (Process p in existing)
    {
        try { p.Kill(entireProcessTree: true); }
        catch { }
        finally { p.Dispose(); }  // Good — but only here
    }
}
```

Later query results are disposed in a finally block, but the pattern could be cleaner.

**Risk**: Minimal (finally block ensures cleanup), but inconsistent with codebase patterns.

**Recommended fix**: Use `using var` or `using` statement consistently:
```csharp
using (Process[] existing = Process.GetProcessesByName(ProcessName))
{
    foreach (Process p in existing)
    {
        try { p.Kill(entireProcessTree: true); }
        finally { p.Dispose(); }
    }
}
```

---

### Finding 8: No Validation of Downloaded File MIME Type
**Severity**: LOW  
**Commits**: 0cf468b4  
**Files**: `scripts/install-dev-tools.ps1` (line 98)

**What's wrong**:
Downloaded file is assumed to be a .zip without checking Content-Type:

```powershell
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing
# Immediately proceeds to extract without validation
Expand-Archive -LiteralPath $tempZip -DestinationPath $targetDir -Force
```

**Risk**: Low (file extension check `if ($asset.name -eq 'UnityExplorer.BepInEx.Mono.zip')` provides some safety), but attacker could return an HTML error page with a .zip extension.

**Recommended fix**:
```powershell
$headers = @{ 'Accept' = 'application/zip' }
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing -Headers $headers

# Validate that the file is actually a ZIP (magic bytes: 50 4B 03 04)
$bytes = Get-Content $tempZip -Raw -AsByteStream -TotalCount 4
if ($bytes -ne @(0x50, 0x4B, 0x03, 0x04)) {
    Write-Error "Downloaded file is not a valid ZIP archive"
    exit 1
}
```

---

### Finding 9: Insufficient Input Validation on Pack Paths (YAML scanning)
**Severity**: LOW  
**Commits**: 427323a2, 7fdb1ffc  
**Files**: `src/Runtime/ModPlatform.cs` (lines ~900–950)

**What's wrong**:
YAML file scanning uses user-provided pack paths without strict validation:

```csharp
string resolved = Path.Combine("packs", manifestLoadSpec);  // manifestLoadSpec from pack.yaml
foreach (string yamlFile in Directory.GetFiles(resolved, "*.yaml", SearchOption.TopDirectoryOnly))
{
    string? itemName = ReadYamlNameField(yamlFile) ?? Path.GetFileNameWithoutExtension(yamlFile);
}
```

If `manifestLoadSpec` contains `../`, the scan could read outside `packs/`.

**Risk**: Low (reads are non-destructive), but violates defense-in-depth.

**Recommended fix**: Validate pack manifest paths:
```csharp
string packPath = Path.GetFullPath(Path.Combine("packs", manifestLoadSpec));
string basePackPath = Path.GetFullPath("packs");
if (!packPath.StartsWith(basePackPath + Path.DirectorySeparatorChar))
    throw new InvalidOperationException($"Pack path escapes packs/ directory: {manifestLoadSpec}");
```

---

### Finding 10: Missing Output Escaping in Console (Low impact, but pattern)
**Severity**: LOW  
**Commits**: 1d33275a, 0cf468b4  
**Files**: Multiple (BuildCommand, DeployCommand, etc.)

**What's wrong**:
Output uses `Markup.Escape()` inconsistently. Some file paths are escaped:

```csharp
AnsiConsole.MarkupLine($"[red]Failed to copy DLL:[/] {Markup.Escape(ex.Message)}");
```

But others might not be in edge cases.

**Risk**: Low (Spectre.Console provides good defaults), but could lead to console injection if escaping is missed.

**Recommended fix**: Establish pattern — always use `Markup.Escape()` for user-controlled output.

---

## FINDINGS SUMMARY TABLE

| # | Severity | File(s) | Issue | Fix Effort |
|---|----------|---------|-------|-----------|
| 1 | HIGH | ModMenuPanel.cs | Unvalidated URLs to Application.OpenURL | 2h |
| 2 | HIGH | install-dev-tools.ps1 | Download without hash verification | 3h |
| 3 | HIGH | install-dev-tools.sh | Path traversal risk in unzip | 2h |
| 4 | MEDIUM | DeployCommand.cs | Path traversal in pack copy | 2h |
| 5 | MEDIUM | RelaunchCommand.cs | Process handle leak | 1h |
| 6 | MEDIUM | PackManifest.cs | No URI format validation | 1h |
| 7 | LOW | DevToolsCommand.cs | Process handle cleanup pattern | <1h |
| 8 | LOW | install-dev-tools.ps1 | No MIME type check on download | <1h |
| 9 | LOW | ModPlatform.cs | Path traversal in YAML scan | 1h |
| 10 | LOW | Multiple | Inconsistent output escaping | <1h |

---

## RECOMMENDED ACTION PLAN

### Phase 1: Block Merge (within 24h)
1. **Finding 1**: Add URI validation before Application.OpenURL
2. **Finding 2**: Add SHA256 verification to install-dev-tools.ps1
3. **Finding 3**: Add path canonicalization to install-dev-tools.sh

### Phase 2: Follow-up PR (within 1 week)
4. **Finding 4**: Add path traversal validation to DeployCommand
5. **Finding 5**: Fix process handle cleanup in RelaunchCommand
6. **Finding 6**: Add schema validation for URLs in PackManifest

### Phase 3: Polish (next sprint)
7–10: Address MEDIUM/LOW findings

---

## CI Gate Recommendation

Add a pre-commit hook check to reject commits that:
- Reference `Application.OpenURL` without a preceding `Uri.TryCreate()` and scheme validation
- Use `Invoke-WebRequest` without hash verification
- Use `Path.Combine` on user-supplied pack paths without `GetFullPath()` canonicalization

---

## References

- Pattern #102 (Orphan Process Handle Leakage)
- Pattern #106 (Implicit `File.ReadAllText` Encoding)
- OWASP URL Validation: https://cheatsheetseries.owasp.org/cheatsheets/URL_Redirect_Validation_Cheat_Sheet.html
- CWE-22 (Path Traversal)
- CWE-426 (Untrusted Search Path)

