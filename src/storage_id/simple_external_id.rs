use crate::StorageId;
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
/// An identifier for external systems
///
/// This ID type is useful for wrapping external IDs (e.g., from social platforms)
/// Format: "actual-id"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct SimpleExternalId {
    id: String,
}

impl SimpleExternalId {
    /// Create a new external ID
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }

    /// Get the ID part
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Default for SimpleExternalId {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
        }
    }
}

impl StorageId for SimpleExternalId {
    fn from_string(id: &str) -> Result<Self> {
        if id.is_empty() {
            return Err(eyre!("Invalid simple external ID: ID must not be empty"));
        }
        Ok(Self { id: id.to_string() })
    }

    fn generate_new(_previous: Option<&Self>) -> Self {
        // External IDs are typically provided by external systems,
        // so we just return a placeholder. This should rarely be used.
        Self::default()
    }

    fn is_valid_format(s: &str) -> bool {
        !s.is_empty()
    }
}

impl fmt::Display for SimpleExternalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
