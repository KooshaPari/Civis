using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// xUnit collection for tests that mutate process-global environment variables.
    /// Disables parallelization to prevent races between tests reading/writing the
    /// same env var (e.g. <c>DINOFORGE_RESOLVER_PATH</c>,
    /// <c>DINOFORGE_TEST_NATIVE_DEP_VAR</c>).
    /// Pattern #93: order-dependent / process-global state.
    /// </summary>
    [CollectionDefinition(Name, DisableParallelization = true)]
    public sealed class EnvVarMutationCollection
    {
        public const string Name = "EnvVarMutation";
    }
}
