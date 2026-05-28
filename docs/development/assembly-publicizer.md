# Assembly Publicizer

## What Is It?

[BepInEx.AssemblyPublicizer.MSBuild](https://github.com/BepInEx/BepInEx.AssemblyPublicizer) is an MSBuild-integrated tool that rewrites `internal` and `private` accessibility modifiers in referenced assemblies to `public` **at compile time only** — the game DLLs on disk are never modified. The publicized copies are generated into `obj/` and are only used during compilation.

## Why DINOForge Needs It

DINO uses Unity DOTS (ECS) with many internal types scattered across `Unity.Entities.dll` and `DNO.Main.dll`, including:

- `EntityManager` internal methods (chunk access, archetype internals)
- `EntityManagerInternal` helper types
- Internal ECS component readers used by `AssetSwapSystem`
- Game-specific component types in `DNO.Main` with `internal` visibility

Without publicization, accessing these members requires runtime reflection:

```csharp
// Fragile: breaks on Mono type-identity mismatch, slow, loses type safety
var method = typeof(EntityManager).GetMethod("GetChunkComponentData",
    BindingFlags.Instance | BindingFlags.NonPublic);
var result = method?.Invoke(em, new object[] { entity, componentType });
```

With publicization, access is direct:

```csharp
// Safe: compile-time checked, fast, no reflection
var data = em.GetChunkComponentData<MyComponent>(entity);
```

The **Mono type-identity bug** (Pattern #233 / iter-142) is particularly relevant here: when reflection-resolved types from `netstandard2.0` code are compared against types loaded by Mono's CLR inside the game, `typeof(T) == resolved.GetType()` can return `false` even for the same logical type. Direct compile-time calls bypass this entirely because the type resolution happens at build time against the actual game DLL, not at runtime through reflection.

## Configuration in DINOForge.Runtime.csproj

The publicizer is wired up in three places:

### 1. Tool package (build-time only, never shipped)

```xml
<ItemGroup Condition="'$(GameInstalled)' == 'true'">
  <PackageReference Include="BepInEx.AssemblyPublicizer.MSBuild" Version="0.4.3" PrivateAssets="all" />
</ItemGroup>
```

`PrivateAssets="all"` ensures the package is a pure build tool — it is never listed as a runtime or transitive dependency in `DINOForge.Runtime.dll`'s metadata.

### 2. Unity.Entities publicized reference

```xml
<Reference Include="Unity.Entities">
  <HintPath>$(ManagedDir)\Unity.Entities.dll</HintPath>
  <Private>false</Private>
  <Publicize>true</Publicize>
</Reference>
```

### 3. DNO.Main publicized reference

```xml
<Reference Include="DNO.Main">
  <HintPath>$(ManagedDir)\DNO.Main.dll</HintPath>
  <Private>false</Private>
  <Publicize>true</Publicize>
</Reference>
```

All game-assembly references are guarded by `Condition="'$(GameInstalled)' == 'true'"`, which means CI builds (where the game is absent) still compile successfully by falling back to the stub `netstandard2.0` allowlist.

## How to Verify Publicization Worked

After a successful local build with the game installed, check that previously-internal types are now callable without reflection:

```csharp
// In any Runtime source file (GameInstalled=true build only):
// This line should compile without error if publicization worked:
var archetypeChunks = entityManager.GetAllChunks(); // internal in stock Unity.Entities
```

Alternatively, inspect the generated publicized assembly in the `obj/` tree:

```powershell
# Find the publicized cache
Get-ChildItem "src\Runtime\obj" -Recurse -Filter "*.publicized.dll" | Select-Object Name, LastWriteTime
```

You should see `Unity.Entities.publicized.dll` and `DNO.Main.publicized.dll` with timestamps matching the most recent build.

## Caveats

- **Only use `<Publicize>true</Publicize>` for game DLLs**, never for DINOForge's own assemblies. Publicizing your own assemblies defeats encapsulation and produces confusing API surfaces.
- The publicized DLLs are **only used during compilation** — the actual game DLLs loaded at runtime retain their original accessibility. Your code must not assume internal members are accessible via reflection at runtime just because they were accessible during compilation.
- If a game update changes the internal layout of a publicized type, the compiler will produce errors. This is the desired behavior — it surfaces API breakage at build time rather than silently at runtime.
- Assembly publicization does **not** bypass Unity's `[NativeDisableUnsafePtrRestriction]` or Burst safety checks — those are runtime enforcements orthogonal to C# accessibility.
