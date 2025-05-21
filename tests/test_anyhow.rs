#[cfg(feature = "anyhow")]
mod test_anyhow {
    use formati::{anyhow, bail};

    #[test]
    fn test_anyhow_basic() {
        let user_id = 42;
        let username = "test_user";

        let err = anyhow!("Failed to process user {username} with ID {user_id}");
        assert_eq!(
            err.to_string(),
            "Failed to process user test_user with ID 42"
        );
    }

    #[test]
    fn test_anyhow_dotted_access() {
        struct User {
            id: u32,
            name: String,
        }

        let user = User {
            id: 42,
            name: String::from("Alice"),
        };

        let err = anyhow!("Failed to process user {user.name} with ID {user.id}");
        assert_eq!(err.to_string(), "Failed to process user Alice with ID 42");
    }

    #[test]
    fn test_anyhow_repeated_expression() {
        struct User {
            id: u32,
            name: String,
        }

        let user = User {
            id: 42,
            name: String::from("Alice"),
        };

        let err = anyhow!(
            "User {user.name} with ID {user.id} not found. Cannot process user {user.name}."
        );
        assert_eq!(
            err.to_string(),
            "User Alice with ID 42 not found. Cannot process user Alice."
        );
    }

    #[test]
    fn test_bail_function() {
        fn process_user(user_id: u32, admin: bool) -> anyhow::Result<()> {
            if !admin {
                bail!("User {user_id} is not an admin");
            }
            Ok(())
        }

        // This should return an error
        let result = process_user(42, false);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "User 42 is not an admin");

        // This should succeed
        let result = process_user(42, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bail_dotted_access() {
        struct User {
            id: u32,
            name: String,
            active: bool,
        }

        fn validate_user(user: &User) -> anyhow::Result<()> {
            if !user.active {
                bail!("User {user.name} (ID: {user.id}) is not active");
            }
            Ok(())
        }

        let inactive_user = User {
            id: 42,
            name: String::from("Alice"),
            active: false,
        };

        let active_user = User {
            id: 43,
            name: String::from("Bob"),
            active: true,
        };

        // This should return an error
        let result = validate_user(&inactive_user);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "User Alice (ID: 42) is not active"
        );

        // This should succeed
        let result = validate_user(&active_user);
        assert!(result.is_ok());
    }
}
