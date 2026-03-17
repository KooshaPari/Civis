using System.Runtime.CompilerServices;

// Expose internals to the main test project so UiActionTrace (internal) can be unit tested.
[assembly: InternalsVisibleTo("DINOForge.Tests")]
