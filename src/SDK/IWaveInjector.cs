namespace DINOForge.SDK
{
    /// <summary>
    /// Public interface for queuing and monitoring wave spawning in DINOForge.
    /// Provides a clean abstraction over the WaveInjector ECS system.
    /// </summary>
    public interface IWaveInjector
    {
        /// <summary>
        /// Queue a wave to be spawned by its definition ID.
        /// </summary>
        /// <param name="waveDefinitionId">The ID of the wave definition to spawn.</param>
        /// <param name="delaySeconds">Optional delay in seconds before spawning begins. Default 0.</param>
        void QueueWave(string waveDefinitionId, float delaySeconds = 0f);

        /// <summary>
        /// Get the number of currently active waves being processed.
        /// </summary>
        int ActiveWaveCount { get; }
    }
}
