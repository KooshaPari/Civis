#nullable enable
using System;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using UnityEngine;
using UnityEngine.LowLevel;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// SPEC-004 integration tests (KIS-IT2, KIS-IT4) for PlayerLoop injection and re-injection
/// after <see cref="PlayerLoop.SetPlayerLoop"/>. Uses in-memory <see cref="PlayerLoopSystem"/>
/// trees and a simulated host — no DINO launch and no Unity native PlayerLoop ECalls.
/// </summary>
[Trait("Category", "KeyInputSystem")]
public sealed class KeyInputSystemIntegrationTests
{
    private static readonly Type PluginMarker = typeof(PlayerLoopKeyInputInjection.DINOForgeUpdateMarker);
    private static readonly Type KeyLoopMarker = typeof(PlayerLoopKeyInputInjection.DINOForgeKeyLoopMarker);

    // ── KIS-IT2: injection survives a replacement loop until re-injection ─────────

    [Fact]
    public void KIS_IT2_AfterInjectThenSetPlayerLoopWithoutMarker_MarkerAbsentUntilReinject()
    {
        var host = new SimulatedPlayerLoopHost(CreateMinimalLoop());
        host.InjectMarker(PluginMarker).Should().BeTrue();
        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, PluginMarker).Should().BeTrue();

        PlayerLoopSystem evicted = PlayerLoopKeyInputInjection.EvictMarkersFromUpdate(host.Current, PluginMarker);
        host.SetPlayerLoop(evicted);

        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, PluginMarker)
          .Should().BeFalse("DINO-style SetPlayerLoop replaces the loop without our entry");
    }

    [Fact]
    public void KIS_IT2_AfterInjectThenSimulatedSetPlayerLoop_ReinjectRestoresDINOForgeUpdate()
    {
        var host = new SimulatedPlayerLoopHost(CreateMinimalLoop());
        host.InjectMarker(PluginMarker).Should().BeTrue();

        PlayerLoopSystem evicted = PlayerLoopKeyInputInjection.EvictMarkersFromUpdate(host.Current, PluginMarker);
        host.SetPlayerLoop(evicted);

        host.ReinjectAfterSet(PluginMarker).Should().BeTrue();
        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, PluginMarker)
          .Should().BeTrue("Harmony postfix OnPlayerLoopSet should re-append DINOForgeUpdate");
    }

    // ── KIS-IT4: eviction + postfix re-injection (_reinjecting guard) ─────────────

    [Fact]
    public void KIS_IT4_SimulateDinoEvictionThenOnAfterSetPlayerLoop_RestoresInjectedEntry()
    {
        var host = new SimulatedPlayerLoopHost(CreateMinimalLoop());
        host.InjectMarker(PluginMarker).Should().BeTrue();
        host.InjectMarker(KeyLoopMarker).Should().BeTrue();

        PlayerLoopSystem stripped = PlayerLoopKeyInputInjection.EvictMarkersFromUpdate(
          host.Current,
          PluginMarker,
          KeyLoopMarker);
        host.SetPlayerLoop(stripped);

        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, PluginMarker).Should().BeFalse();
        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, KeyLoopMarker).Should().BeFalse();

        host.ReinjectAfterSet(PluginMarker).Should().BeTrue();
        host.ReinjectAfterSet(KeyLoopMarker).Should().BeTrue();

        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, PluginMarker).Should().BeTrue();
        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, KeyLoopMarker).Should().BeTrue();
    }

    [Fact]
    public void KIS_IT4_ReinjectGuard_PreventsRecursiveOnAfterSetPlayerLoopDuringInject()
    {
        var host = new SimulatedPlayerLoopHost(CreateMinimalLoop());
        int onAfterSetInvocations = 0;
        int injectBuildCalls = 0;

        host.SetPlayerLoopWithPostfix(() =>
        {
            onAfterSetInvocations++;
            if (!PlayerLoopKeyInputInjection.TryBuildInjectedLoop(host.Current, PluginMarker, null, out PlayerLoopSystem injected))
            {
                return false;
            }

            injectBuildCalls++;
            host.SetPlayerLoop(injected);

            host.InvokePostfixOnly(() =>
            {
                onAfterSetInvocations++;
                return true;
            });

            return PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, PluginMarker);
        });

        onAfterSetInvocations.Should().Be(1,
          "SPEC-004 _reinjecting guard: nested SetPlayerLoop postfix must not recurse into OnAfterSetPlayerLoop");
        injectBuildCalls.Should().Be(1);
        PlayerLoopKeyInputInjection.IsReinjecting.Should().BeFalse();
        PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(host.Current, PluginMarker).Should().BeTrue();
    }

    [Fact]
    public void KIS_IT4_TryBuildInjectedLoop_IsIdempotentWhenMarkerAlreadyPresent()
    {
        PlayerLoopSystem loop = CreateMinimalLoop();
        PlayerLoopKeyInputInjection.TryBuildInjectedLoop(loop, PluginMarker, null, out PlayerLoopSystem first)
          .Should().BeTrue();
        int countAfterFirst = CountMarkerEntriesInUpdate(first, PluginMarker);

        PlayerLoopKeyInputInjection.TryBuildInjectedLoop(first, PluginMarker, null, out PlayerLoopSystem second)
          .Should().BeTrue();
        CountMarkerEntriesInUpdate(second, PluginMarker)
          .Should().Be(countAfterFirst, "duplicate injection must not append a second marker entry");
    }

    private static PlayerLoopSystem CreateMinimalLoop()
    {
        return new PlayerLoopSystem
        {
            subSystemList = new[]
          {
        new PlayerLoopSystem
        {
          type = typeof(UnityEngine.PlayerLoop.Update),
          subSystemList = Array.Empty<PlayerLoopSystem>(),
        },
      },
        };
    }

    private static int CountMarkerEntriesInUpdate(PlayerLoopSystem loop, Type markerType)
    {
        int count = 0;
        if (loop.subSystemList == null)
        {
            return 0;
        }

        for (int i = 0; i < loop.subSystemList.Length; i++)
        {
            PlayerLoopSystem subsystem = loop.subSystemList[i];
            if (subsystem.type != typeof(UnityEngine.PlayerLoop.Update) || subsystem.subSystemList == null)
            {
                continue;
            }

            for (int j = 0; j < subsystem.subSystemList.Length; j++)
            {
                if (subsystem.subSystemList[j].type == markerType)
                {
                    count++;
                }
            }
        }

        return count;
    }

    /// <summary>
    /// In-memory stand-in for <see cref="PlayerLoop.SetPlayerLoop"/> + Harmony postfix (KIS-IT2/IT4).
    /// </summary>
    private sealed class SimulatedPlayerLoopHost
    {
        private PlayerLoopSystem _current;

        public SimulatedPlayerLoopHost(PlayerLoopSystem initial) => _current = initial;

        public PlayerLoopSystem Current => _current;

        public bool InjectMarker(Type marker)
        {
            if (!PlayerLoopKeyInputInjection.TryBuildInjectedLoop(_current, marker, null, out PlayerLoopSystem injected))
            {
                return false;
            }

            _current = injected;
            return PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(_current, marker);
        }

        public void SetPlayerLoop(PlayerLoopSystem loop) => _current = loop;

        public bool ReinjectAfterSet(Type marker)
        {
            bool success = false;
            PlayerLoopKeyInputInjection.OnAfterSetPlayerLoop(() =>
            {
                success = InjectMarker(marker);
                return success;
            });
            return success;
        }

        public void SetPlayerLoopWithPostfix(Func<bool> inject)
        {
            PlayerLoopKeyInputInjection.OnAfterSetPlayerLoop(inject);
        }

        public void InvokePostfixOnly(Func<bool> body)
        {
            PlayerLoopKeyInputInjection.OnAfterSetPlayerLoop(body);
        }
    }
}
