#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using DINOForge.Runtime.Diagnostics;
using Unity.Collections;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// ECS system that turns vanilla DINO projectiles (arrows / bolts / ballista shots) into
    /// glowing Star-Wars blaster bolts WITHOUT requiring new asset bundles.
    ///
    /// Per #975, DINO renders projectiles through <c>Unity.Rendering.RenderMesh</c> exactly like
    /// every other entity. Projectile archetypes carry <c>Components.ProjectileDataBase</c>
    /// (+ RenderMesh + a VFX/trail). This system extends the same RenderMesh-swap mechanism that
    /// <see cref="AssetSwapSystem"/> uses for units, but instead of substituting a bundle mesh it
    /// recolours the existing projectile material's emission to a faction-keyed energy-bolt colour
    /// (red for CIS / Separatist droids, blue for the Galactic Republic — see
    /// <see cref="BlasterBoltConfig"/>).
    ///
    /// Faction is inferred from the vanilla <c>Components.Enemy</c> tag (DINO has no explicit
    /// Faction component): enemy-tagged projectiles are CIS (red), friendly ones are Republic (blue).
    ///
    /// Lifecycle mirrors <see cref="AssetSwapSystem"/>: wait <see cref="MinFrameDelay"/> frames for
    /// the gameplay world to populate, then on each update recolour projectile RenderMesh materials.
    /// Recoloured Material clones are cached per (faction, source-material) so we never leak a new
    /// Material per projectile.
    ///
    /// Reflection is used for the EntityManager shared-component accessors because DINO's Mono 4.x
    /// runtime exhibits a cross-assembly type-identity bug (typeof(Entity) != param type) — the same
    /// arity-filter pattern proven in <see cref="AssetSwapSystem"/> is reused here.
    /// </summary>
    [UpdateInGroup(typeof(PresentationSystemGroup))]
    public class ProjectileMeshSwapSystem : SystemBase
    {
        /// <summary>Minimum frames to wait before recolouring (world must be populated).</summary>
        private const int MinFrameDelay = 600;

        /// <summary>Safety cap on projectiles recoloured per update to bound main-thread cost.</summary>
        private const int MaxProjectilesPerUpdate = 200;

        private int _frameCount;
        private bool _queryInitialized;
        private EntityQuery _projectileQuery;
        private Type? _renderMeshType;
        private FieldInfo? _materialField;
        private MethodInfo? _getSharedNonGeneric;
        private MethodInfo? _setSharedGeneric;
        // BUG A fix (#101): DINO's Unity 2021.3 EntityManager has both
        // GetSharedComponentData(Entity, ComponentType) and (Entity, int typeIndex)
        // non-generic overloads. When the Int32 overload is bound, invoking it with a boxed
        // ComponentType throws "ComponentType cannot be converted to Int32" — the cause of
        // ~22k '[BlasterBolt] Recolour failed on projectile' log spam. Track which arg the
        // bound overload wants so we can pass ComponentType.TypeIndex (int) instead.
        private bool _getSharedWantsTypeIndex;
        private bool _reflectionResolved;
        private bool _reflectionOk;

        // Cache of recoloured materials keyed by (faction, source material instance id) so we
        // create at most one clone per distinct source material per faction.
        private readonly Dictionary<string, Material> _recolouredMaterials =
            new Dictionary<string, Material>(StringComparer.Ordinal);

        // Track which entities we've already recoloured (by Index:Version) to avoid redundant work.
        private readonly HashSet<long> _recolouredEntities = new HashSet<long>();

        private bool _loggedDisabled;
        private int _recolourCount;

        /// <inheritdoc/>
#if NET8_0
        public override void OnCreate()
#else
        protected override void OnCreate()
#endif
        {
            base.OnCreate();
            DebugLog.Write("BlasterBolt", "ProjectileMeshSwapSystem.OnCreate");
        }

        /// <inheritdoc/>
#if NET8_0
        public override void OnUpdate()
#else
        protected override void OnUpdate()
#endif
        {
            _frameCount++;
            if (_frameCount < MinFrameDelay)
                return;

            if (!BlasterBoltConfig.Enabled)
            {
                if (!_loggedDisabled)
                {
                    _loggedDisabled = true;
                    DebugLog.Write("BlasterBolt", "Disabled via BlasterBoltConfig.Enabled=false — skipping projectile recolour.");
                }
                return;
            }

            EntityManager em = World.DefaultGameObjectInjectionWorld?.EntityManager ?? EntityManager;

            if (!ResolveReflection())
                return;

            if (!_queryInitialized)
            {
                ComponentType? projType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.ProjectileDataBase");
                if (projType == null)
                {
                    if (_frameCount % 120 == 0)
                        DebugLog.Write("BlasterBolt", "Components.ProjectileDataBase not resolved yet — retrying.");
                    return;
                }

                _projectileQuery = em.CreateEntityQuery(new EntityQueryDesc
                {
                    All = new[]
                    {
                        ComponentType.ReadOnly(projType.Value.TypeIndex),
                        ComponentType.ReadOnly(_renderMeshType!),
                    },
                    Options = EntityQueryOptions.IncludePrefab,
                });
                _queryInitialized = true;
            }

            NativeArray<Entity> projectiles = _projectileQuery.ToEntityArray(Allocator.Temp);
            try
            {
                if (projectiles.Length == 0)
                    return;

                ComponentType renderMeshComponentType = ComponentType.ReadOnly(_renderMeshType!);
                int processed = 0;

                for (int i = 0; i < projectiles.Length && processed < MaxProjectilesPerUpdate; i++)
                {
                    Entity e = projectiles[i];
                    long key = ((long)e.Index << 32) | (uint)e.Version;
                    if (_recolouredEntities.Contains(key))
                        continue;

                    try
                    {
                        if (RecolourProjectile(em, e, renderMeshComponentType))
                        {
                            _recolouredEntities.Add(key);
                            processed++;
                            _recolourCount++;
                        }
                    }
                    catch (TargetInvocationException ex) when (
                        ex.InnerException?.Message.Contains("Ambiguous match found") == true)
                    {
                        // Multi-RenderMesh projectile (trail + body) — skip, one mesh is enough.
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("BlasterBolt", $"Recolour failed on projectile {e.Index}: {ex.Message}");
                    }
                }

                if (processed > 0)
                {
                    DebugLog.Write("BlasterBolt",
                        $"Recoloured {processed} projectile(s) this pass (total={_recolourCount}, matching={projectiles.Length}).");
                }
            }
            finally
            {
                projectiles.Dispose();
            }
        }

        /// <summary>
        /// Recolours a single projectile's RenderMesh material emission to its faction bolt colour.
        /// Returns true if a recoloured material was applied.
        /// </summary>
        private bool RecolourProjectile(EntityManager em, Entity entity, ComponentType renderMeshComponentType)
        {
            if (!em.HasComponent(entity, renderMeshComponentType))
                return false;

            // BUG A fix (#101): pass TypeIndex (int) when the bound overload is (Entity, int).
            object sharedArg = _getSharedWantsTypeIndex
                ? (object)renderMeshComponentType.TypeIndex
                : renderMeshComponentType;
            object? renderMesh = _getSharedNonGeneric!.Invoke(em, new object[] { entity, sharedArg });
            if (renderMesh == null)
                return false;

            object? matObj = _materialField!.GetValue(renderMesh);
            if (matObj is not Material sourceMat || sourceMat == null)
                return false;

            bool isEnemy = HasEnemyTag(em, entity);
            BlasterBoltConfig.BoltColor boltColor = BlasterBoltConfig.ResolveBoltColor(isEnemy);
            string faction = isEnemy ? "cis" : "rep";

            Material recoloured = GetOrCreateRecolouredMaterial(faction, sourceMat, boltColor);

            _materialField.SetValue(renderMesh, recoloured);
            _setSharedGeneric!.Invoke(em, new object[] { entity, renderMesh });
            return true;
        }

        /// <summary>
        /// Returns a cached recoloured clone of <paramref name="sourceMat"/> for the given faction,
        /// creating it (emissive energy-bolt look) on first request.
        /// </summary>
        private Material GetOrCreateRecolouredMaterial(string faction, Material sourceMat, BlasterBoltConfig.BoltColor c)
        {
            string cacheKey = faction + ":" + sourceMat.GetInstanceID().ToString();
            if (_recolouredMaterials.TryGetValue(cacheKey, out Material existing) && existing != null)
                return existing;

            Material clone = new Material(sourceMat);
            float intensity = BlasterBoltConfig.EmissionIntensity;
            Color baseColor = new Color(c.R, c.G, c.B, c.A);
            Color emissive = new Color(c.R * intensity, c.G * intensity, c.B * intensity, 1f);

            try
            {
                clone.EnableKeyword("_EMISSION");
                if (clone.HasProperty("_EmissionColor"))
                    clone.SetColor("_EmissionColor", emissive);
                if (clone.HasProperty("_Color"))
                    clone.SetColor("_Color", baseColor);
                if (clone.HasProperty("_BaseColor"))
                    clone.SetColor("_BaseColor", baseColor);
                if (clone.HasProperty("_TintColor"))
                    clone.SetColor("_TintColor", baseColor);
            }
            catch (Exception ex)
            {
                DebugLog.Write("BlasterBolt", $"Material property set partial-failure ({faction}): {ex.Message}");
            }

            _recolouredMaterials[cacheKey] = clone;
            DebugLog.Write("BlasterBolt",
                $"Created {faction} bolt material from '{sourceMat.name}' (rgb={c.R:F2},{c.G:F2},{c.B:F2} intensity={intensity:F1}).");
            return clone;
        }

        private static bool HasEnemyTag(EntityManager em, Entity entity)
        {
            try
            {
                ComponentType? enemyType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.Enemy");
                if (enemyType == null)
                    return false;
                return em.HasComponent(entity, ComponentType.ReadOnly(enemyType.Value.TypeIndex));
            }
            catch
            {
                return false;
            }
        }

        /// <summary>
        /// Resolves the RenderMesh type + material field + EntityManager shared-component accessors.
        /// Mirrors <see cref="AssetSwapSystem"/>'s Mono-safe arity-filter reflection. HRV2 variants
        /// (RenderMeshUnmanaged / MaterialMeshInfo) have no mutable material field and are skipped.
        /// </summary>
        private bool ResolveReflection()
        {
            if (_reflectionResolved)
                return _reflectionOk;
            _reflectionResolved = true;

            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    Type? t = asm.GetType("Unity.Rendering.RenderMesh", throwOnError: false);
                    if (t != null) { _renderMeshType = t; break; }
                }
                catch { /* assembly may not contain Unity.Rendering */ }
            }

            if (_renderMeshType == null)
            {
                DebugLog.Write("BlasterBolt", "Unity.Rendering.RenderMesh (HRV1) not found — projectile recolour disabled.");
                _reflectionOk = false;
                return false;
            }

            _materialField = _renderMeshType.GetField("material");
            if (_materialField == null)
            {
                DebugLog.Write("BlasterBolt", "RenderMesh.material field not found (HRV2?) — projectile recolour disabled.");
                _reflectionOk = false;
                return false;
            }

            _getSharedNonGeneric = typeof(EntityManager).GetMethods(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                .FirstOrDefault(m =>
                    m.Name == "GetSharedComponentData"
                    && !m.IsGenericMethodDefinition
                    && m.GetParameters().Length == 2
                    && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity"
                    && (m.GetParameters()[1].ParameterType.FullName == "Unity.Entities.ComponentType"
                        || m.GetParameters()[1].ParameterType.FullName == "System.Int32"));

            // BUG A fix (#101): record whether the bound overload takes an int TypeIndex.
            _getSharedWantsTypeIndex = _getSharedNonGeneric != null
                && _getSharedNonGeneric.GetParameters()[1].ParameterType.FullName == "System.Int32";

            MethodInfo? setSharedDef = typeof(EntityManager).GetMethods(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                .FirstOrDefault(m =>
                    m.Name == "SetSharedComponentData"
                    && m.IsGenericMethodDefinition
                    && m.GetParameters().Length == 2
                    && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity");

            if (_getSharedNonGeneric == null || setSharedDef == null)
            {
                DebugLog.Write("BlasterBolt",
                    $"EntityManager shared-component reflection failed (get={_getSharedNonGeneric != null}, set={setSharedDef != null}) — projectile recolour disabled.");
                _reflectionOk = false;
                return false;
            }

            _setSharedGeneric = setSharedDef.MakeGenericMethod(_renderMeshType);
            _reflectionOk = true;
            DebugLog.Write("BlasterBolt", "Reflection resolved — projectile bolt recolour active.");
            return true;
        }
    }
}
