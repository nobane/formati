#[cfg(feature = "tracing")]
mod test_tracing {
    use formati::{debug, error, info, trace, warn};
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use tracing::Level;
    use tracing_subscriber::{
        fmt::{format::FmtSpan, MakeWriter},
        FmtSubscriber,
    };

    // Create a custom writer to capture log output
    #[derive(Clone, Default)]
    struct TestWriter {
        captured: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn new() -> Self {
            Self {
                captured: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn captured_output(&self) -> String {
            let captured = self.captured.lock().unwrap();
            String::from_utf8(captured.clone()).unwrap()
        }
    }

    impl<'a> MakeWriter<'a> for TestWriter {
        type Writer = GuardedWriter<'a>;

        fn make_writer(&'a self) -> Self::Writer {
            GuardedWriter {
                guard: self.captured.lock().unwrap(),
            }
        }
    }

    struct GuardedWriter<'a> {
        guard: std::sync::MutexGuard<'a, Vec<u8>>,
    }

    impl Write for GuardedWriter<'_> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.guard.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    // Helper to set up tracing for tests
    fn setup_tracing() -> (TestWriter, tracing::subscriber::DefaultGuard) {
        let writer = TestWriter::new();
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .with_writer(writer.clone())
            .with_span_events(FmtSpan::NONE)
            .finish();

        let guard = tracing::subscriber::set_default(subscriber);
        (writer, guard)
    }

    // Tests

    // Full functional test of tracing macros
    #[test]
    fn test_tracing_events() {
        let (writer, _guard) = setup_tracing();

        let user_id = 42;
        let username = "test_user";

        // Test info! macro with simple values
        info!("User logged in: {username} with id {user_id}");
        let output = writer.captured_output();
        assert!(output.contains("User logged in: test_user with id 42"));

        let (writer, _guard) = setup_tracing();

        // Test debug! macro with dotted notation
        let user = (username, user_id);
        debug!("Debug info: username={user.0}, id={user.1}");
        let output = writer.captured_output();
        assert!(output.contains("Debug info: username=test_user, id=42"));

        let (writer, _guard) = setup_tracing();

        // Test error! macro with named arguments
        error!("Error processing user {name}", name = username);
        let output = writer.captured_output();
        assert!(output.contains("Error processing user test_user"));

        let (writer, _guard) = setup_tracing();

        // Test trace! with target and repeated expressions
        trace!(target: "auth_service", "User {user.0} logged in with ID {user.1}, welcome {user.0}!");
        let output = writer.captured_output();
        assert!(output.contains("User test_user logged in with ID 42, welcome test_user!"));

        let (writer, _guard) = setup_tracing();

        // Test with complex objects and attributes
        let account = (("premium", true), 365);
        info!(
            account.type = account.0.0,
            active = account.0.1,
            "Account details: type={account.0.0}, active={account.0.1}, days={account.1}"
        );
        let output = writer.captured_output();
        assert!(output.contains("Account details: type=premium, active=true, days=365"));
    }

    #[test]
    fn test_event_fields() {
        let person = ("Alice", 30, "Engineer");
        let address = ("123 Main St", "Anytown", "USA");

        let (writer, _guard) = setup_tracing();

        info!(
            department = "Engineering",
            years_of_service = 5,
            "Complex info: {person.0} is a {person.2} living in {address.1}, {address.2}"
        );

        let output = writer.captured_output();
        assert!(output.contains("Complex info: Alice is a Engineer living in Anytown, USA"));
    }
}
