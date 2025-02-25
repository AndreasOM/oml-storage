use crate::StorageId;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
/// A nanoid-based random identifier
///
/// This ID type generates random, unique strings using the nanoid library.
/// It's suitable for distributed systems where coordination is difficult.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RandomId(String);

impl RandomId {
    /// Create a new random ID
    pub fn new() -> Self {
        Self(nanoid::nanoid!())
    }

    /// Create from an existing string
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    /// Get the inner string value
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl StorageId for RandomId {
    fn from_string(s: &str) -> Result<Self> {
        Ok(Self(s.to_string()))
    }

    fn generate_new(_previous: Option<&Self>) -> Self {
        Self::new()
    }

    fn is_valid_format(_s: &str) -> bool {
        // nanoid can be any string, so we don't need specific validation
        // Could add length checks or character set validation if needed
        true
    }
}

impl fmt::Display for RandomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
