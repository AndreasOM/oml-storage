use crate::StorageId;
use color_eyre::eyre::{eyre, Result};
use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
/// A sequential numeric identifier
///
/// This ID type represents incremental numbers.
/// It's suitable for systems that need human-readable, ordered IDs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct SequentialId(u64);

impl SequentialId {
    /// Create a new sequential ID with the given value
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    
    /// Get the inner numeric value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl StorageId for SequentialId {
    fn from_string(s: &str) -> Result<Self> {
        match u64::from_str(s) {
            Ok(num) => Ok(Self(num)),
            Err(e) => Err(eyre!("Invalid sequential ID format: {}", e)),
        }
    }
    
    fn generate_new(previous: Option<&Self>) -> Self {
        match previous {
            Some(prev) => Self(prev.0 + 1),
            None => Self(1), // Start from 1 if no previous ID
        }
    }
    
    fn is_valid_format(s: &str) -> bool {
        s.parse::<u64>().is_ok()
    }
}

impl fmt::Display for SequentialId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

