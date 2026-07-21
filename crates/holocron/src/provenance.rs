use serde::{Deserialize, Serialize};

/// Surface through which a verb is exposed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provenance {
    Mcp,
    JsonRpc,
    Egui,
    Holocron,
    Other(String),
}

impl Provenance {
    pub fn label(&self) -> String {
        match self {
            Self::Mcp => "MCP".to_owned(),
            Self::JsonRpc => "JSON-RPC".to_owned(),
            Self::Egui => "egui".to_owned(),
            Self::Holocron => "Holocron".to_owned(),
            Self::Other(value) => value.clone(),
        }
    }
}
