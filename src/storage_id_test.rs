#[cfg(test)]
mod tests {
    use crate::storage_id::*;
    use crate::StorageId;
    use color_eyre::Result;

    #[test]
    fn test_random_id() -> Result<()> {
        // Generate a new random ID
        let id1 = RandomId::new();
        println!("Generated random ID: {}", id1);

        // Create from string
        let id_str = id1.to_string();
        let id2 = RandomId::from_string(&id_str)?;

        // Verify they're equal
        assert_eq!(id1, id2);

        // Generate a new one - should be different
        let id3 = RandomId::generate_new(None);
        assert_ne!(id1, id3);

        // Generate from previous - with RandomId this still creates a new random ID
        let id4 = RandomId::generate_new(Some(&id1));
        assert_ne!(id1, id4);

        // Format validation - should always be true for RandomId
        assert!(RandomId::is_valid_format(&id_str));

        Ok(())
    }

    #[test]
    fn test_sequential_id() -> Result<()> {
        // Start with ID 1
        let id1 = SequentialId::new(1);
        println!("Initial sequential ID: {}", id1);

        // Create from string
        let id_str = id1.to_string();
        let id2 = SequentialId::from_string(&id_str)?;

        // Verify they're equal
        assert_eq!(id1, id2);

        // Generate a new one without previous - should start at 1
        let id3 = SequentialId::generate_new(None);
        assert_eq!(id3.value(), 1);

        // Generate from previous - should increment
        let id4 = SequentialId::generate_new(Some(&id1));
        assert_eq!(id4.value(), 2);

        // Format validation
        assert!(SequentialId::is_valid_format("123"));
        assert!(!SequentialId::is_valid_format("abc"));

        Ok(())
    }

    #[test]
    fn test_external_id() -> Result<()> {
        // Create an external ID for a Facebook user
        let id1 = ExternalId::new("facebook", "12345678");
        println!("External ID: {}", id1);

        // Create from string
        let id_str = id1.to_string();
        let id2 = ExternalId::from_string(&id_str)?;

        // Verify they're equal
        assert_eq!(id1, id2);

        // Test prefix and ID getters
        assert_eq!(id1.prefix(), "facebook");
        assert_eq!(id1.id(), "12345678");

        // Format validation
        assert!(ExternalId::is_valid_format("facebook:12345678"));
        assert!(!ExternalId::is_valid_format("facebook"));
        assert!(!ExternalId::is_valid_format("facebook:"));
        assert!(!ExternalId::is_valid_format(":12345678"));

        // Invalid format should fail parsing
        assert!(ExternalId::from_string("invalid").is_err());

        Ok(())
    }

    #[test]
    fn test_simple_external_id() -> Result<()> {
        // Create an external ID
        let id1 = SimpleExternalId::new("12344321");
        println!("Simple External ID: {}", id1);

        // Create from string
        let id_str = id1.to_string();
        let id2 = SimpleExternalId::from_string(&id_str)?;

        // Verify they're equal
        assert_eq!(id1, id2);

        // Test ID getter
        assert_eq!(id1.id(), "12344321");

        // Format validation
        assert!(SimpleExternalId::is_valid_format("12344321"));
        assert!(!SimpleExternalId::is_valid_format(""));

        // Invalid format should fail parsing
        assert!(SimpleExternalId::from_string("").is_err());

        Ok(())
    }
}
