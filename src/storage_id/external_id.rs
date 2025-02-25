use crate::StorageId;
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
/// An identifier for external systems with a prefix
///
/// This ID type is useful for wrapping external IDs (e.g., from social platforms)
/// with a prefix to identify the source system.
/// Format: "prefix:actual-id"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExternalId {
    prefix: String,
    id: String,
}

impl ExternalId {
    /// Create a new external ID with the given prefix and ID
    pub fn new(prefix: &str, id: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            id: id.to_string(),
        }
    }

    /// Get the prefix (source system)
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Get the ID part
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Default for ExternalId {
    fn default() -> Self {
        Self {
            prefix: "unknown".to_string(),
            id: "default".to_string(),
        }
    }
}

impl StorageId for ExternalId {
    fn from_string(s: &str) -> Result<Self> {
        if let Some((prefix, id)) = s.split_once(':') {
            if prefix.is_empty() || id.is_empty() {
                return Err(eyre!(
                    "Invalid external ID: prefix and ID must not be empty"
                ));
            }
            Ok(Self {
                prefix: prefix.to_string(),
                id: id.to_string(),
            })
        } else {
            Err(eyre!("Invalid external ID format: must be 'prefix:id'"))
        }
    }

    fn generate_new(_previous: Option<&Self>) -> Self {
        // External IDs are typically provided by external systems,
        // so we just return a placeholder. This should rarely be used.
        Self::default()
    }

    fn is_valid_format(s: &str) -> bool {
        if let Some((prefix, id)) = s.split_once(':') {
            !prefix.is_empty() && !id.is_empty()
        } else {
            false
        }
    }
}

impl fmt::Display for ExternalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.prefix, self.id)
    }
}
