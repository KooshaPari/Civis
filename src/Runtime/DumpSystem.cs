using System;
using System.IO;
using BepInEx;
using BepInEx.Logging;
using Unity.Entities;

namespace DINOForge.Runtime
{
    /// <summary>
    /// ECS System that runs the entity dump after a delay.
    /// This survives MonoBehaviour destruction because it lives in the ECS World.
    /// Based on dno-mods pattern: inherits SystemBase and uses OnUpdate.
    /// </summary>
    public class DumpSystem : SystemBase
    {
        private static ManualLogSource? _log;
        private static string? _outputDir;
        private int _frameCount;
        private bool _dumpCompleted;

        public static void Configure(ManualLogSource log, string outputDir)
        {
            _log = log;
            _outputDir = outputDir;
        }

        protected override void OnCreate()
        {
            base.OnCreate();
            WriteDebug("DumpSystem.OnCreate called");
        }

        protected override void OnUpdate()
        {
            _frameCount++;

            if (_frameCount <= 3 || _frameCount % 100 == 0)
            {
                WriteDebug($"DumpSystem.OnUpdate frame={_frameCount} entities={EntityManager.GetAllEntities(Unity.Collections.Allocator.Temp).Length}");
            }

            // Wait ~30s (1800 frames at 60fps) to capture gameplay entities
            if (!_dumpCompleted && _frameCount >= 1800)
            {
                WriteDebug("DumpSystem triggering dump");
                try
                {
                    if (_log != null && _outputDir != null)
                    {
                        var dumper = new EntityDumper(_log, _outputDir);
                        dumper.DumpAll();

                        var enumerator = new SystemEnumerator(_log);
                        enumerator.EnumerateAll();

                        _log.LogInfo("ECS dump completed successfully.");
                    }
                }
                catch (Exception ex)
                {
                    WriteDebug($"Dump failed: {ex}");
                    _log?.LogError($"ECS dump failed: {ex}");
                }
                _dumpCompleted = true;

                // Disable self after dump
                Enabled = false;
            }
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] {msg}\n");
            }
            catch { } // safe-swallow: best-effort debug I/O, non-critical
        }
    }
}
