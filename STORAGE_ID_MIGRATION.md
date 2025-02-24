# Migration Guide for StorageId

This guide explains how to migrate your code from using the previous ID system to the new `StorageId` trait.

## Changes Overview

1. The `StorageId` trait replaces the `generate_next_id` and `make_id` methods in `StorageItem`
2. Several built-in ID implementations are provided: `RandomId`, `SequentialId`, and `ExternalId`
3. `StorageItem::ID` now requires the `StorageId` trait bound

## Migrating Your Code

### 1. Update Your Item Implementation

Before:
```rust
impl StorageItem for TestItem {
    type ID = String;

    fn serialize(&self) -> Result<Vec<u8>> {
        // ...
    }
    
    fn deserialize(data: &[u8]) -> Result<Self> {
        // ...
    }
    
    fn generate_next_id(_a_previous_id: Option<&Self::ID>) -> Self::ID {
        nanoid::nanoid!()
    }
    
    fn make_id(id: &str) -> Result<Self::ID> {
        let id = id.parse::<Self::ID>()?;
        Ok(id)
    }
}
```

After:
```rust
use oml_storage::RandomId;

impl StorageItem for TestItem {
    type ID = RandomId;

    fn serialize(&self) -> Result<Vec<u8>> {
        // ...
    }
    
    fn deserialize(data: &[u8]) -> Result<Self> {
        // ...
    }
}
```

### 2. Replace String IDs with Appropriate StorageId Implementation

Choose the appropriate ID type based on your needs:

- `RandomId`: For nanoid-style random unique IDs
- `SequentialId`: For incremental numeric IDs
- `ExternalId`: For IDs with a prefix indicating the source system

### 3. Update ID Creation and Loading Code

Before:
```rust
// Create a new ID
let id = TestItem::generate_next_id(None);

// Parse an ID from a string
let id = TestItem::make_id("some-id")?;
```

After:
```rust
// Create a new ID
let id = RandomId::generate_new(None);

// Parse an ID from a string
let id = RandomId::from_string("some-id")?;
```

### 4. Creating Custom ID Types

If you need a custom ID type, implement the `StorageId` trait:

```rust
use oml_storage::StorageId;
use color_eyre::eyre::Result;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct MyCustomId {
    // Your ID fields
}

impl StorageId for MyCustomId {
    fn from_string(s: &str) -> Result<Self> {
        // Parse from string
    }
    
    fn generate_new(previous: Option<&Self>) -> Self {
        // Generate a new ID
    }
    
    fn is_valid_format(s: &str) -> bool {
        // Check if string is valid format
    }
}

impl fmt::Display for MyCustomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format for display
    }
}

impl ToString for MyCustomId {
    fn to_string(&self) -> String {
        // Convert to string
    }
}
```

## Questions?

If you have questions about migrating to the new ID system, please open an issue on the GitHub repository.