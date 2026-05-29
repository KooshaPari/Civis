# Audio Takeover Survey

Goal: replace almost all in-game music and SFX from a BepInEx plugin for DINOForge. The practical target is a Unity 2021.3 game on Mono, so the best solution is usually a layered one:

1. Find the game-owned `AudioSource`/music manager and swap its clip at runtime.
2. Intercept generic playback paths such as `AudioSource.Play`, `PlayOneShot`, and any custom audio wrapper methods.
3. Load replacement audio from pack content at runtime, not from Unity editor assets.
4. Treat menu music, battle music, UI sounds, and one-shot SFX as separate replacement classes.

## 1) How Unity games typically play music and SFX

Unity audio in shipped games usually falls into a few patterns:

- A persistent music manager owns one looping `AudioSource` for menus and another for battle/gameplay.
- SFX are often short-lived `AudioSource` instances created on demand, pooled, or played with `PlayOneShot`.
- Some games route audio through wrappers such as `AudioManager`, `SoundManager`, `MusicController`, or a custom event bus.
- Addressables-based games may not hold `AudioClip` references in code directly; they resolve them from keys/labels at runtime using Addressables APIs.

Relevant Unity behavior:

- `AudioSource.clip` is the next clip that will be played.
- `AudioSource.Play()` starts the current clip on that source.
- `AudioSource.PlayOneShot()` plays a clip without replacing the source’s main `clip`.
- `UnityWebRequestMultimedia.GetAudioClip()` can create an `AudioClip` from a file or URL at runtime, including local `file:///` paths.
- Addressables can load an `AudioClip` asynchronously by key or resource location.

Implication for takeover:

- Menu music is usually easiest to replace by swapping the clip on the persistent source.
- SFX are usually easier to replace by patching playback APIs or the game’s sound manager rather than trying to enumerate every `AudioSource` in the scene.
- If the game uses Addressables, a clean solution is often to intercept the load path and return a replacement clip for a known key.

## 2) Finding and swapping the menu music `AudioSource`

The common BepInEx approach is:

1. Wait until the main menu scene or title screen has loaded.
2. Walk the scene hierarchy or inspect loaded objects for likely music controllers.
3. Use reflection when fields are private or obfuscated.
4. Swap the `AudioSource.clip`, then restart playback if needed.

Search strategy:

- Start from the active scene root objects and search for components whose names look like `Music`, `Menu`, `Title`, `MainMenu`, or `Audio`.
- For each candidate component, inspect all `AudioSource` fields and properties via reflection.
- If there is no obvious manager, search for any active `AudioSource` with `loop == true`, `playOnAwake == true`, or a long music-like clip length.
- If the menu music is created dynamically, hook the method that initializes it, then replace the clip before the first play call.

Practical swap pattern:

```csharp
AudioSource source = ...; // found by scene walk or reflection
AudioClip replacement = ...; // loaded from pack content

source.Stop();
source.clip = replacement;
source.loop = true;
source.Play();
```

If the source is private, cached in a singleton, or hidden behind obfuscated fields, reflection is usually enough:

```csharp
FieldInfo field = typeof(SomeMusicManager).GetField("_musicSource", BindingFlags.Instance | BindingFlags.NonPublic);
AudioSource source = (AudioSource)field.GetValue(instance);
```

If the game reassigns the clip later, you need a patch on the setter path or on the method that chooses the music track.

## 3) Loading custom `AudioClip` from pack files at runtime

For a total conversion, audio should come from the pack pipeline, not from Unity project assets.

### Preferred runtime loading options

1. `UnityWebRequestMultimedia.GetAudioClip`
   - Good for `.ogg`, `.wav`, and `.mp3` files stored on disk or fetched over HTTP.
   - Works well with `file:///...` paths to pack-installed audio.
   - Best choice when you want Unity to decode the file for you.

2. `AudioClip.Create`
   - Best when you already decoded PCM samples in managed code.
   - Useful for procedural synthesis or when a custom decoder is used outside Unity.
   - More work than file-based loading, but useful if you need full control.

3. Addressables/AssetBundles
   - Best when the pack wants to ship Unity-native audio assets and keep load behavior aligned with the game.
   - Good for large content sets and games already using the Addressables pipeline.

### Pack-storage recommendation

Store replacement audio as ordinary pack files, then copy or stage them into a runtime-accessible folder. The plugin should resolve replacement paths from pack metadata and load them asynchronously.

Suggested file types:

- Music: `.ogg` preferred, `.wav` acceptable for simplest decode/debugging.
- SFX: `.wav` for short, uncompressed effects; `.ogg` if size matters more than decode cost.

Recommended loading flow:

1. Read replacement file path from pack metadata.
2. Convert it to a `file:///` URI.
3. Load it with `UnityWebRequestMultimedia.GetAudioClip`.
4. Cache the resulting `AudioClip` by logical asset key.
5. Reuse the cached clip for all later substitutions.

If the game needs immediate playback and the clip is not yet ready, delay the play call or use a fallback until `loadState == Loaded`.

## 4) Harmony-patching `AudioSource.Play` to substitute clips

This is the broadest replacement strategy and usually the most robust when you do not know every game-specific audio manager.

### What to patch

- `AudioSource.Play()`
- `AudioSource.PlayOneShot(AudioClip, float)`
- Potentially `AudioSource.PlayDelayed()` and any custom helper methods the game uses
- Game-specific music manager methods if they can be identified

### Recommended patch shape

Use a Harmony prefix or postfix to substitute the clip before playback:

- For `Play()`, inspect `__instance.clip`.
- For `PlayOneShot()`, inspect the supplied clip argument.
- If a replacement exists, swap it and allow the original call to continue.
- Keep a cache of already resolved replacements so the patch stays cheap.

Example pattern:

```csharp
[HarmonyPatch(typeof(AudioSource), nameof(AudioSource.Play))]
internal static class AudioSourcePlayPatch
{
    private static void Prefix(AudioSource __instance)
    {
        if (__instance == null || __instance.clip == null)
        {
            return;
        }

        AudioClip replacement = ReplacementRegistry.TryGetReplacement(__instance.clip.name);
        if (replacement != null)
        {
            __instance.clip = replacement;
        }
    }
}
```

For one-shot audio, the replacement key is usually the original clip name, path, hash, or a pack-defined logical ID.

### Limits of this approach

- It can miss audio that is played through custom native code, FMOD, Wwise, or bespoke wrapper APIs.
- It may overmatch if a single clip name is reused for multiple semantic sounds.
- It is still useful as a safety net even when you also patch game-specific managers.

## 5) How a pack should declare audio replacements

The pack model should not just say "replace audio"; it should declare which logical game asset or semantic audio role is being replaced.

### Proposed extension: `asset_replacements.audio`

Recommended shape:

```yaml
asset_replacements:
  audio:
    music:
      menu:
        source: packs/starwars/audio/menu_theme.ogg
        match:
          component: MusicManager
          field: menuMusicSource
          clipName: MainMenuTheme
      battle:
        source: packs/starwars/audio/battle_theme.ogg
        match:
          clipName: BattleTheme
    sfx:
      ui_click:
        source: packs/starwars/audio/ui_click.wav
        match:
          clipName: ButtonClick
      weapon_fire:
        source: packs/starwars/audio/blaster_fire.ogg
        match:
          clipName: RifleShot
```

### Fields worth supporting

- `source`: path to the replacement file inside the pack.
- `match.clipName`: original `AudioClip.name` or another stable identifier.
- `match.component`: a component type or manager type the runtime should inspect.
- `match.field`: private/public field name to reflect on when the game owns a source.
- `match.scene`: optional scene name for scene-specific sounds.
- `match.tags`: optional semantic labels such as `menu`, `ui`, `weapon`, `ambient`, `voice`.
- `volumeScale`: optional override if the replacement needs gain correction.
- `loop`: optional override for looping tracks.
- `priority`: useful if several matches could apply.

### Runtime resolution order

1. Exact semantic match from pack metadata.
2. Exact clip name match.
3. Scene/component-specific match.
4. Fallback to generic `AudioSource.Play` interception.

This keeps the mod deterministic and avoids relying only on brittle clip-name heuristics.

## 6) SOTA BepInEx music mod approaches

The strongest current pattern in Unity/BepInEx music mods is layered, not singular:

- Use HarmonyX for method patching, because BepInEx ships with runtime patching support and HarmonyX is the default Unity modding toolchain.
- Prefer game-specific manager patches when available, because they are more precise than global hooks.
- Fall back to `AudioSource` interception for broad coverage.
- Cache decoded clips and avoid reloading from disk on every play event.
- Use file-based audio loading for loose files and AssetBundle/Addressables loading when the mod wants Unity-native packaging.
- Add configuration and logging so the user can see which clips were replaced and which were missed.
- For menu music, many mods simply identify the persistent music source and replace its clip on scene load or startup, because that path is predictable and low-risk.

The most reliable modern mod stack is:

1. Specific patch for the game’s music manager.
2. Generic `AudioSource` patch as a catch-all.
3. Pack-defined audio mapping for exact replacements.
4. Runtime clip cache with async loading.

## 7) Recommended implementation shape for DINOForge

If this becomes a real feature, the runtime should probably expose:

- A pack-level audio replacement manifest.
- A replacement registry keyed by semantic role and clip name.
- A loader that resolves pack file paths to `AudioClip` objects.
- One or more Harmony patches for `AudioSource` plus any discovered menu/music controllers.
- A debug surface that lists current audio mappings and misses.

That gives the conversion three layers of control:

- precise replacement for menu/music tracks,
- broad replacement for one-shot SFX,
- and an escape hatch for edge cases where the game uses custom playback code.

## References

- Unity `AudioSource` API: https://docs.unity3d.com/ja/2021.3/ScriptReference/AudioSource.html
- Unity `AudioSource.clip` API: https://docs.unity3d.com/ja/2021.3/ScriptReference/AudioSource-clip.html
- Unity `UnityWebRequestMultimedia.GetAudioClip`: https://docs.unity3d.com/ja/2020.1/ScriptReference/Networking.UnityWebRequestMultimedia.GetAudioClip.html
- Unity Addressables `LoadAssetAsync`: https://docs.unity3d.com/ja/Packages/com.unity.addressables%401.20/api/UnityEngine.AddressableAssets.Addressables.LoadAssetAsync.html
- Unity Addressables overview: https://docs.unity3d.com/ja/2023.2/Manual/com.unity.addressables.html
- BepInEx runtime patching: https://docs.bepinex.dev/articles/dev_guide/runtime_patching.html
- BepInEx HarmonyX: https://github.com/BepInEx/HarmonyX
- Harmony patching model: https://github.com/pardeike/Harmony/wiki/Patching
