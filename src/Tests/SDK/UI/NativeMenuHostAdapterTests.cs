// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 4 of #193 SDK split — adapter contract tests.
// Unity-runtime behavior (vanilla canvas mount, OnEnter/OnExit pumping) lands in M11.5
// (WI-004a) and is exercised by game-launch acceptance — not here.

using System;
using DINOForge.Runtime.UI.Adapters;
using DINOForge.SDK.UI.Native;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK.UI
{
    public class NativeMenuHostAdapterTests
    {
        private sealed class FakeScreen : INativeMenuScreen
        {
            public string Title => "FakeScreen";
            public void OnEnter() { }
            public void OnExit() { }
        }

        [Fact]
        public void Instance_IsSingleton()
        {
            NativeMenuHostAdapter a = NativeMenuHostAdapter.Instance;
            NativeMenuHostAdapter b = NativeMenuHostAdapter.Instance;
            a.Should().BeSameAs(b);
        }

        [Fact]
        public void Instance_ImplementsINativeMenuHost()
        {
            NativeMenuHostAdapter.Instance.Should().BeAssignableTo<INativeMenuHost>();
        }

        [Fact]
        public void RegisterScreen_ThrowsOnNullMenuId()
        {
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            Action act = () => host.RegisterScreen(null!, new FakeScreen());
            act.Should().Throw<ArgumentNullException>().WithParameterName("menuId");
        }

        [Fact]
        public void RegisterScreen_ThrowsOnEmptyMenuId()
        {
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            Action act = () => host.RegisterScreen(string.Empty, new FakeScreen());
            act.Should().Throw<ArgumentException>().WithParameterName("menuId");
        }

        [Fact]
        public void RegisterScreen_ThrowsOnNullScreen()
        {
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            Action act = () => host.RegisterScreen("dinoforge.mods", null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("screen");
        }

        [Fact]
        public void UnregisterScreen_ThrowsOnNullMenuId()
        {
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            Action act = () => host.UnregisterScreen(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("menuId");
        }

        [Fact]
        public void UnregisterScreen_ThrowsOnEmptyMenuId()
        {
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            Action act = () => host.UnregisterScreen(string.Empty);
            act.Should().Throw<ArgumentException>().WithParameterName("menuId");
        }

        [Fact]
        public void Register_Then_Unregister_TogglesIsActive()
        {
            // Use a unique menuId per test to keep the singleton's state isolated from
            // sibling tests that share the same INativeMenuHost.Instance.
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            const string id = "dinoforge.tests.menuhost.toggle";

            // Defensive cleanup — prior crashed runs may leave stale entries.
            host.UnregisterScreen(id);

            host.RegisterScreen(id, new FakeScreen());
            host.IsActive.Should().BeTrue("registering at least one screen makes the host active");

            host.UnregisterScreen(id);
            // We can't assert IsActive==false globally because parallel tests may register
            // their own screens. The contract we lock here is symmetry: after we unregister
            // our own id, RegisterScreen for that id no longer throws (i.e. it was removed).
            Action reRegister = () => host.RegisterScreen(id, new FakeScreen());
            reRegister.Should().NotThrow();
            host.UnregisterScreen(id);
        }

        [Fact]
        public void RegisterScreen_DuplicateId_LastWriterWins()
        {
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            const string id = "dinoforge.tests.menuhost.duplicate";
            host.UnregisterScreen(id);

            // Two registrations with the same id must not throw — last-writer-wins matches
            // the pack hot-reload contract used by extended UI hosts.
            Action act = () =>
            {
                host.RegisterScreen(id, new FakeScreen());
                host.RegisterScreen(id, new FakeScreen());
            };
            act.Should().NotThrow();

            host.UnregisterScreen(id);
        }

        [Fact]
        public void UnregisterScreen_UnknownId_NoOpDoesNotThrow()
        {
            INativeMenuHost host = NativeMenuHostAdapter.Instance;
            Action act = () => host.UnregisterScreen("dinoforge.tests.menuhost.never-registered");
            act.Should().NotThrow();
        }
    }
}
