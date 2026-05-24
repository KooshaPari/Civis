using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Centralized collection-name registry for xUnit <c>[Collection]</c>
    /// attributes. Tests touching process-global state MUST opt into the
    /// appropriate collection to prevent xUnit's default class-parallelism
    /// from racing on shared state.
    /// </summary>
    /// <remarks>
    /// Pattern #93 (Order-Dependent / Process-Global State) is detected by
    /// <c>scripts/ci/detect_global_state_tests.py</c>. See
    /// <c>docs/qa/test-isolation-policy.md</c> for the full rationale +
    /// snapshot-in-ctor pattern + required <c>DINOFORGE_TEST_</c>
    /// env-var prefix.
    ///
    /// The collection-name strings here are duplicated by the
    /// <c>[CollectionDefinition]</c> declarations in their respective
    /// fixture files (e.g. <c>EnvVarMutationCollection.cs</c>,
    /// <c>AssetSwapRegistryCollection.cs</c>) — those files own the
    /// <c>DisableParallelization = true</c> flag and the
    /// <c>ICollectionFixture&lt;T&gt;</c> binding (if any).
    /// </remarks>
    public static class Collections
    {
        /// <summary>For tests that mutate environment variables.</summary>
        /// <remarks>
        /// Pairs with <see cref="EnvVarMutationCollection"/>. Tests in this
        /// collection MUST snapshot the var in the ctor and restore in
        /// <c>Dispose()</c>. Use the <c>DINOFORGE_TEST_</c> name prefix.
        /// </remarks>
        public const string EnvVarMutation = "EnvVarMutation";

        /// <summary>
        /// For tests that change the process-wide current working directory
        /// via <c>Directory</c>.<c>SetCurrentDirectory</c>.
        /// </summary>
        /// <remarks>
        /// Pairs with <see cref="WorkingDirectoryCollection"/>. Process-wide
        /// cwd mutation is fundamentally unsafe under default xUnit
        /// parallelism — opt in here, snapshot in ctor, restore in
        /// <c>Dispose()</c>.
        /// </remarks>
        public const string WorkingDirectory = "WorkingDirectory";

        /// <summary>
        /// For tests that touch the local dinoforge MCP named pipe.
        /// </summary>
        /// <remarks>
        /// Pairs with <see cref="BridgePipeCollection"/>. Two tests binding
        /// the same pipe name (<c>dinoforge-bridge</c>) cannot run in
        /// parallel; use <see cref="System.Guid"/>-discriminated names
        /// where possible and gate the rest under this collection.
        /// </remarks>
        public const string BridgePipe = "BridgePipe";

        /// <summary>For tests that bind to specific TCP ports.</summary>
        /// <remarks>
        /// Pairs with <see cref="NetworkPortCollection"/>. Hardcoded port
        /// usage is fragile across CI shards; prefer ephemeral ports
        /// (port 0 → OS-assigned). Use this collection only when a fixed
        /// port is unavoidable.
        /// </remarks>
        public const string NetworkPort = "NetworkPort";

        /// <summary>
        /// For tests that mutate the <c>AssetSwapRegistry</c> singleton.
        /// </summary>
        /// <remarks>
        /// Already paired with <see cref="AssetSwapRegistryCollection"/>
        /// (defined separately). Listed here for completeness; new tests
        /// touching the registry should use this constant.
        /// </remarks>
        public const string AssetSwapRegistry = "AssetSwapRegistry";

        /// <summary>For UI automation tests against the desktop companion.</summary>
        /// <remarks>
        /// Already paired with <see cref="UiAutomationCollection"/> (defined
        /// in <c>UiAutomation/CompanionFixture.cs</c>). Listed here for
        /// completeness.
        /// </remarks>
        public const string UiAutomation = "UiAutomation";

        /// <summary>For real-game launch + bridge tests.</summary>
        /// <remarks>
        /// Already paired with <c>GameLaunchCollection</c> (defined in
        /// <c>GameLaunch/GameLaunchFixture.cs</c>). Listed here for
        /// completeness.
        /// </remarks>
        public const string GameLaunch = "GameLaunch";

        /// <summary>
        /// For tests that rely on <see cref="System.IO.FileSystemWatcher"/> under
        /// Windows. Serializes to avoid OS buffer overflow and startup races.
        /// </summary>
        public const string FileSystemWatcher = "FileSystemWatcher";
    }

    /// <summary>
    /// xUnit collection for tests that mutate the process-wide current
    /// working directory via <c>Directory</c>.<c>SetCurrentDirectory</c>.
    /// Process-wide cwd is the most fragile global state in .NET; serial
    /// execution is the only safe mode.
    /// Pattern #93.
    /// </summary>
    [CollectionDefinition(Collections.WorkingDirectory, DisableParallelization = true)]
    public sealed class WorkingDirectoryCollection
    {
    }

    /// <summary>
    /// xUnit collection for tests that bind the local dinoforge MCP named
    /// pipe (or any other process-scoped named pipe). Disables
    /// parallelization to prevent <c>ERROR_PIPE_BUSY</c> on CI shards.
    /// Pattern #93.
    /// </summary>
    [CollectionDefinition(Collections.BridgePipe, DisableParallelization = true)]
    public sealed class BridgePipeCollection
    {
    }

    /// <summary>
    /// xUnit collection for tests that bind to specific TCP ports. Disables
    /// parallelization to prevent <c>SocketException(AddressAlreadyInUse)</c>.
    /// Prefer ephemeral ports (port 0) over this collection where possible.
    /// Pattern #93.
    /// </summary>
    [CollectionDefinition(Collections.NetworkPort, DisableParallelization = true)]
    public sealed class NetworkPortCollection
    {
    }

    /// <summary>
    /// xUnit collection for <see cref="System.IO.FileSystemWatcher"/> integration
    /// tests. Disables parallelization to prevent missed events under CI load.
    /// Pattern #108.
    /// </summary>
    [CollectionDefinition(Collections.FileSystemWatcher, DisableParallelization = true)]
    public sealed class FileSystemWatcherCollection
    {
    }
}
