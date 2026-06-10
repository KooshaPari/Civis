//! Provider adapters behind the [`crate::AiProvider`] port (FR-CIV-AI-002..006).
//!
//! | Provider | generate | embed | Feature | Status (P1) |
//! |---|---|---|---|---|
//! | [`DummyAiProvider`] | yes (deterministic) | yes (fixed) | — | **working** |
//! | `FirepassKimiProvider` | yes (wraps cloud) | no | `cloud` | **wired** |
//! | `LocalSlmProvider` | yes (mistral.rs GGUF) | no | `local` | **stub** |
//! | `EmbedProvider` | no | yes (MiniLM) | `embed` | **stub** |
//! | `OllamaDevProvider` | yes (HTTP) | no | `dev` | **stub** |

mod dummy;
pub use dummy::DummyAiProvider;

#[cfg(feature = "cloud")]
mod firepass_kimi;
#[cfg(feature = "cloud")]
pub use firepass_kimi::FirepassKimiProvider;

#[cfg(feature = "local")]
mod local_slm;
#[cfg(feature = "local")]
pub use local_slm::LocalSlmProvider;

#[cfg(feature = "embed")]
mod embed;
#[cfg(feature = "embed")]
pub use embed::EmbedProvider;

#[cfg(feature = "dev")]
mod ollama_dev;
#[cfg(feature = "dev")]
pub use ollama_dev::OllamaDevProvider;
