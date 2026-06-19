//! Provider registry (design §4.2) — feature services request a **role**, not a
//! concrete provider. Config-driven ("provider interface + registry > N
//! classes").
//!
//! Loud-failure on a missing **required** provider: [`ProviderRegistry::require`]
//! returns a named error rather than silently substituting (`CLAUDE.md`
//! failure stance).

use std::collections::HashMap;
use std::sync::Arc;

use crate::AiProvider;

/// A logical provider role requested by feature services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderRole {
    /// Legends / chatter / headlines flavor (≤1.5B narrator).
    Narrator,
    /// Drift / speciation / log-triage embeddings.
    Embedder,
    /// Heavy reasoning (sagas, tech R&D) — cloud OK.
    Heavy,
}

impl ProviderRole {
    /// Stable name for error messages.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderRole::Narrator => "narrator",
            ProviderRole::Embedder => "embedder",
            ProviderRole::Heavy => "heavy",
        }
    }
}

/// A required provider role was not registered.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error("civ-ai: required provider role '{0}' not registered")]
pub struct MissingProvider(pub &'static str);

/// Maps roles to Arc-shared providers, built from config.
#[derive(Default, Clone)]
pub struct ProviderRegistry {
    providers: HashMap<ProviderRole, Arc<dyn AiProvider>>,
}

impl ProviderRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `provider` under `role` (last write wins).
    pub fn register(&mut self, role: ProviderRole, provider: Arc<dyn AiProvider>) -> &mut Self {
        self.providers.insert(role, provider);
        self
    }

    /// Optional lookup — `None` if the role is unregistered.
    #[must_use]
    pub fn get(&self, role: ProviderRole) -> Option<Arc<dyn AiProvider>> {
        self.providers.get(&role).map(Arc::clone)
    }

    /// Required lookup — **loud, named failure** if the role is unregistered.
    ///
    /// # Errors
    /// Returns [`MissingProvider`] when no provider is registered for `role`.
    pub fn require(&self, role: ProviderRole) -> Result<Arc<dyn AiProvider>, MissingProvider> {
        self.get(role).ok_or(MissingProvider(role.as_str()))
    }
}
