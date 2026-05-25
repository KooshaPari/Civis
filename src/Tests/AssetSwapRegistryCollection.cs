using Xunit;

namespace DINOForge.Tests
{
    [CollectionDefinition(Name, DisableParallelization = true)]
    public sealed class AssetSwapRegistryCollection
    {
        public const string Name = "AssetSwapRegistry";
    }
}
