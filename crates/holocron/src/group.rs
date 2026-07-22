use serde::{Deserialize, Serialize};

/// Stable UI grouping for catalogued godgame verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VerbGroup {
    Civic,
    Economic,
    Divine,
    Debug,
}
