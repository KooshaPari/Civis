# ADR-007: Neural Text-to-Speech for Proof-of-Features Videos

**Status**: Accepted
**Deciders**: DINOForge Core Team
**Date**: 2026-03-24
**Context**: /prove-features video command requires professional voiceover

---

## Problem Statement

The `/prove-features` command must generate voiceover for automated demo videos. Current Windows SAPI voices (David, Zira) sound robotic and unprofessional, unsuitable for public demo distribution.

Options:
1. **Windows SAPI (built-in)**: Free, zero setup, but robotic voice quality
2. **Azure Cognitive Services API**: Professional neural voices, requires API key, costs money
3. **edge-tts (Microsoft neural)**: Professional neural voices, free, offline after cache, no API key
4. **Google Cloud TTS**: Similar to Azure, requires API key and billing
5. **AWS Polly**: Similar to Azure, requires API key and billing

---

## Decision

**Use edge-tts as primary TTS provider; fall back to Windows SAPI if Python unavailable.**

### Rationale

**edge-tts is optimal because**:
- **Voice quality**: Voices identical to Azure Cognitive Services (Aria, Jenny, Guy neural models)
- **Cost**: $0 — no API key, no billing, no accounts
- **Offline capability**: First run downloads neural model (~50MB), then works offline
- **Python available**: Python 3.11 already installed at `C:\Users\koosh\AppData\Local\Programs\Python\Python311\python.exe`
- **Simple integration**: One-liner package install: `pip install edge-tts`
- **Open source**: Community-maintained, auditable code

**Why not Azure/Google/AWS**:
- Require API key management, billing setup, credential rotation
- Adds complexity for zero additional quality gain
- Creates cloud dependency; breaks on network outage or account issues
- edge-tts provides identical voice models for free

**Why SAPI fallback**:
- Built-in, no dependencies
- If Python not installed or network unavailable, SAPI is reliable backup
- Acceptable quality for minimal-viable demo (if neural unavailable)

---

## Implementation

### 1. Voice Selection

**Primary voice**: `en-US-AriaNeural` (female, natural, friendly tone, 25kHz sample rate)

**Alternative voice**: `en-US-GuyNeural` (male, natural, authoritative, available for contrast)

**Additional options** (future):
- `en-US-JennyNeural` (female, youthful)
- `en-US-SaraNeural` (female, professional)

### 2. Installation

Edge-tts is installed on-demand:

```powershell
$pythonExe = "C:\Users\koosh\AppData\Local\Programs\Python\Python311\python.exe"

# Check if edge-tts is available
try {
    & $pythonExe -c "import edge_tts; print('edge-tts available')" 2>$null
    $edgeTtsAvailable = $?
} catch {
    $edgeTtsAvailable = $false
}

# Install if missing
if (-not $edgeTtsAvailable) {
    Write-Host "Installing edge-tts..."
    & $pythonExe -m pip install edge-tts -q
}
```

### 3. Voice Generation

```powershell
function New-NeuralVoiceover {
    param(
        [string]$Text,
        [string]$OutputPath,
        [string]$Voice = "en-US-AriaNeural",
        [double]$Rate = 1.0
    )

    $pythonExe = "C:\Users\koosh\AppData\Local\Programs\Python\Python311\python.exe"

    # Ensure edge-tts installed
    & $pythonExe -m pip install edge-tts -q 2>$null

    # Generate voice (edge-tts outputs MP3 by default)
    & $pythonExe -m edge_tts `
        --voice $Voice `
        --text $Text `
        --rate $Rate `
        --write-media $OutputPath

    if (-not (Test-Path $OutputPath)) {
        Write-Warning "edge-tts generation failed for '$Text'"
        return $false
    }

    return $true
}
```

**Parameters**:
- `--voice`: Voice identifier (e.g., `en-US-AriaNeural`)
- `--text`: String to synthesize (auto-escapes special characters)
- `--rate`: Speed multiplier (1.0 = normal, 0.5 = half speed, 1.5 = faster)
- `--write-media`: Output file path (auto-detects .mp3 from extension)

### 4. Fallback to SAPI

If edge-tts fails, fall back to Windows SAPI:

```powershell
function New-SapiVoiceover {
    param(
        [string]$Text,
        [string]$OutputPath
    )

    Add-Type -AssemblyName System.Speech
    $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer

    # Select voice (prefer "Aria" if available, fallback to default)
    try {
        $synth.SelectVoice("Microsoft Aria")
    } catch {
        Write-Warning "Aria voice not available; using default SAPI voice"
    }

    $synth.SetOutputToWaveFile($OutputPath)
    $synth.Speak($Text)
    $synth.SetOutputToDefaultAudioDevice()
}

function New-Voiceover {
    param(
        [string]$Text,
        [string]$OutputPath,
        [string]$Voice = "en-US-AriaNeural"
    )

    # Try edge-tts first
    if (New-NeuralVoiceover @PSBoundParameters) {
        Write-Host "Generated voice: $OutputPath (neural)"
        return $true
    }

    # Fall back to SAPI
    Write-Warning "Falling back to SAPI voice"
    New-SapiVoiceover -Text $Text -OutputPath $OutputPath

    if (Test-Path $OutputPath) {
        Write-Host "Generated voice: $OutputPath (SAPI fallback)"
        return $true
    }

    Write-Error "Failed to generate voiceover"
    return $false
}
```

### 5. Voice Line Script

Used for `/prove-features` demo:

```yaml
features:
  - id: intro
    text: "DINOForge mod platform. Feature demonstration."
    voice: en-US-AriaNeural
    duration_expected: 3s

  - id: mods_button
    text: "Mods button successfully injected into the native main menu — auto-detected in under 10 seconds."
    voice: en-US-AriaNeural
    duration_expected: 5s

  - id: f9
    text: "Pressing F 9 opens the debug overlay panel."
    voice: en-US-AriaNeural
    duration_expected: 3s

  - id: f10
    text: "Pressing F 10 opens the mod menu panel."
    voice: en-US-AriaNeural
    duration_expected: 3s

  - id: outro
    text: "All three features confirmed working."
    voice: en-US-AriaNeural
    duration_expected: 2s
```

---

## Consequences

### Positive

1. **Professional quality**: Neural voices are natural, no uncanny valley effect
2. **Zero cost**: No API keys, billing, or account management
3. **Offline capable**: Caches neural models after first use
4. **Simple setup**: Single `pip install` command
5. **Fallback reliability**: SAPI available if Python missing
6. **Repeatable demos**: Same voice/speed every run, consistent branding
7. **Scriptable**: Easy to vary voice per video or user preference

### Negative (Mitigations)

1. **Python dependency**: Requires Python 3.11 installed
   - *Mitigation*: Check at startup; document Python requirement in README

2. **Network on first use**: Neural model downloads (~50MB) on first voice generation
   - *Mitigation*: Pre-cache models during setup; document offline requirement

3. **Quality variance**: If fallback to SAPI, voice quality drops noticeably
   - *Mitigation*: Warn user; suggest edge-tts reinstall; offer manual voice record

4. **Voice selection UI**: Not all Azure voices available in edge-tts
   - *Mitigation*: Hardcode 3 voices (Aria, Guy, Jenny); document in help

---

## Voice Configuration Standards

### Selection Rules

| Context | Voice | Rationale |
|---------|-------|-----------|
| Feature demo (primary) | en-US-AriaNeural | Female, friendly, professional |
| Alternative demo | en-US-GuyNeural | Male, authoritative, contrast option |
| Fallback (SAPI) | Default / Aria | Best available Windows voice |

### Rate Adjustment

| Use Case | Rate | Duration Impact |
|----------|------|-----------------|
| Normal voiceover (default) | 1.0 | ~100% baseline |
| Slower (emphasis) | 0.85 | ~118% longer |
| Faster (quick demo) | 1.15 | ~87% shorter |

For `/prove-features`, use rate=1.0 (normal speed) for all lines.

---

## Network & Caching

### Initial Setup (One-time)

```powershell
# On first /prove-features run:
# 1. edge-tts -m pip install (if missing)
# 2. edge-tts --voice en-US-AriaNeural --text "test" (downloads ~50MB model)
# 3. Model cached in ~/.cache/edge-tts/ (Windows: %APPDATA%\.cache\edge-tts\)
# 4. Subsequent calls use cached model (offline)
```

**Time**: ~30-60s for first run (model download), <1s per voice line after.

### Offline Availability

Once cached, edge-tts works fully offline. No internet required after initial setup.

### Cache Invalidation

Users can clear cache to force re-download (if corrupted):
```powershell
Remove-Item $env:APPDATA\.cache\edge-tts -Recurse -Force
```

---

## Integration with /prove-features

The decision applies directly to the `/prove-features` command:

1. **Voice generation phase**: Call `New-NeuralVoiceover` for each voice line
2. **Fallback**: If Python unavailable or network blocked, use `New-SapiVoiceover`
3. **Logging**: Log which voice system was used (neural vs. SAPI)
4. **User messaging**: Report voice quality in console output:
   - "Using neural TTS (Aria)..." (good)
   - "Using SAPI fallback (quality reduced)..." (acceptable)
   - "Failed to generate voice. Check Python installation." (error)

---

## Testing

### Unit Tests

```powershell
# Test edge-tts availability
function Test-EdgeTtsInstalled {
    $pythonExe = "C:\Users\koosh\AppData\Local\Programs\Python\Python311\python.exe"
    try {
        & $pythonExe -c "import edge_tts" 2>$null
        return $?
    } catch { return $false }
}

# Test voice generation
function Test-NeuralVoiceGeneration {
    $tmpFile = [System.IO.Path]::GetTempFileName() + ".mp3"
    try {
        New-NeuralVoiceover -Text "Test" -OutputPath $tmpFile
        return (Test-Path $tmpFile) -and ((Get-Item $tmpFile).Length -gt 1000)
    } finally {
        Remove-Item $tmpFile -Force -ErrorAction SilentlyContinue
    }
}

# Test SAPI fallback
function Test-SapiVoiceGeneration {
    $tmpFile = [System.IO.Path]::GetTempFileName() + ".wav"
    try {
        New-SapiVoiceover -Text "Test" -OutputPath $tmpFile
        return (Test-Path $tmpFile) -and ((Get-Item $tmpFile).Length -gt 1000)
    } finally {
        Remove-Item $tmpFile -Force -ErrorAction SilentlyContinue
    }
}
```

### Integration Tests

- Record real demo video with neural TTS
- Verify audio quality (listenability)
- Verify ffmpeg mux works with MP3 input
- Verify fallback to SAPI if Python missing
- Verify offline operation (cache hit after first run)

---

## Documentation

Update these docs:

1. **README.md**: Add Python 3.11 requirement, edge-tts optional dependency
2. **SETUP.md**: Document edge-tts installation (automatic on first `/prove-features` run)
3. **PROVE_FEATURES.md**: Document voice selection, quality expectations, troubleshooting
4. **CLAUDE.md**: Add to "Wrap, don't handroll" guidelines: "TTS: Use edge-tts (neural); fallback to SAPI (robotic)"

---

## References

- **edge-tts repository**: https://github.com/rany2/edge-tts
- **PyPI edge-tts**: https://pypi.org/project/edge-tts/
- **Azure Neural Voices**: https://learn.microsoft.com/en-us/azure/cognitive-services/speech-service/language-support?tabs=prebuilt
- **Windows SAPI**: https://learn.microsoft.com/en-us/previous-versions/windows/desktop/ms723612(v=vs.85)

---

## Approval

- [ ] DINOForge architect
- [ ] Team lead
- [ ] Security/privacy review (edge-tts caches models locally)
