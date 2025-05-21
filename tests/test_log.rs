#[cfg(feature = "log")]
mod test {
    use log::{LevelFilter, Log, Metadata, Record};
    use std::sync::{Arc, Mutex, OnceLock};

    use formati::{debug, error, format, info, trace, warn};

    // Global static logger for all tests
    static LOGGER: OnceLock<TestLogger> = OnceLock::new();

    #[derive(Clone)]
    struct TestLogger {
        captured: Arc<Mutex<Vec<String>>>,
        level: LevelFilter,
    }

    impl TestLogger {
        fn new() -> Self {
            Self {
                captured: Arc::new(Mutex::new(Vec::new())),
                level: LevelFilter::Trace,
            }
        }

        fn captured_logs(&self) -> Vec<String> {
            let guard = self.captured.lock().unwrap();
            guard.clone()
        }

        fn clear(&self) {
            let mut guard = self.captured.lock().unwrap();
            guard.clear();
        }

        // Get the global instance, initializing if needed
        fn get_instance() -> &'static TestLogger {
            LOGGER.get_or_init(|| {
                let logger = TestLogger::new();
                // Initialize logger only once
                let _ = log::set_boxed_logger(Box::new(logger.clone()));
                log::set_max_level(LevelFilter::Trace);
                logger
            })
        }
    }

    impl Log for TestLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= self.level
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                let message = format!("[{record.target()}] {record.level()}: {record.args()}");
                let mut guard = self.captured.lock().unwrap();
                guard.push(message);
            }
        }

        fn flush(&self) {}
    }

    // Instead of setting up a new logger each time, get the global instance
    fn setup_logger() -> &'static TestLogger {
        TestLogger::get_instance()
    }

    // Test log enhanced macros
    #[test]
    fn test_log_macros_basic() {
        let logger = setup_logger();
        logger.clear(); // Start with a clean state

        let user_id = 42;
        let username = "test_user";

        info!("User {username} logged in with ID {user_id}");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("User test_user logged in with ID 42"));

        logger.clear();

        error!("Failed to process user {username}");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("Failed to process user test_user"));

        logger.clear();

        warn!("User {username} has suspicious activity");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("User test_user has suspicious activity"));

        logger.clear();

        debug!("Debug: user={username}, id={user_id}");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("Debug: user=test_user, id=42"));

        logger.clear();

        trace!("Trace: user_login(user={username}, id={user_id})");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("Trace: user_login(user=test_user, id=42)"));
    }

    #[test]
    fn test_log_macros_dotted_access() {
        let logger = setup_logger();
        logger.clear(); // Start with a clean state

        struct User {
            id: u32,
            name: String,
            access_level: String,
        }

        let user = User {
            id: 42,
            name: String::from("Alice"),
            access_level: String::from("admin"),
        };

        info!("User {user.name} logged in with ID {user.id}");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("User Alice logged in with ID 42"));

        logger.clear();

        error!("Cannot grant {user.access_level} privileges to user {user.name}");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("Cannot grant admin privileges to user Alice"));

        logger.clear();

        // Test with target specification
        debug!("User {user.name} authenticated with level {user.access_level}");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("User Alice authenticated with level admin"));
    }

    #[test]
    fn test_log_macros_nested_access() {
        let logger = setup_logger();
        logger.clear(); // Start with a clean state

        let data = ((1, 2), (3, "test"));

        info!("Values: {data.0.0}, {data.0.1}, {data.1.0}, {data.1.1}");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("Values: 1, 2, 3, test"));

        logger.clear();

        // Test with method calls
        struct Database {
            name: String,
            size: usize,
        }

        impl Database {
            fn connection_string(&self) -> String {
                format!("db://{}", self.name)
            }

            fn format_size(&self) -> String {
                format!("{} MB", self.size)
            }
        }

        let db = Database {
            name: String::from("users_db"),
            size: 1024,
        };

        info!("Connected to {db.connection_string()} (size: {db.format_size()})");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("Connected to db://users_db (size: 1024 MB)"));
    }

    #[test]
    fn test_log_macros_repeated_expression() {
        let logger = setup_logger();
        logger.clear(); // Start with a clean state

        struct User {
            id: u32,
            name: String,
        }

        let user = User {
            id: 42,
            name: String::from("Alice"),
        };

        // The user.name expression should only be evaluated once
        info!("User {user.name} logged in. Welcome back, {user.name}!");
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("User Alice logged in. Welcome back, Alice!"));

        logger.clear();

        // Complex case with multiple repeated expressions
        debug!(
            "User ID: {user.id}, Name: {user.name}\n\
             Processing request for {user.name} (ID: {user.id})"
        );
        let logs = logger.captured_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("User ID: 42, Name: Alice"));
        assert!(logs[0].contains("Processing request for Alice (ID: 42)"));
    }
}
