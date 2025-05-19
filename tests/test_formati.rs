use formati::*;
use std::f32::consts;
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

#[test]
fn test_formati_basic() {
    let answer = 42;
    let result = formati!("The answer is {answer}");
    assert_eq!(result, "The answer is 42");
}

#[test]
fn test_formati_dotted_access() {
    let user = (String::from("Alice"), 30);
    let result = formati!("Name: {user.0}, Age: {user.1}");
    assert_eq!(result, "Name: Alice, Age: 30");
}

#[test]
fn test_formati_formatting_specs() {
    let value = consts::PI;
    let result = formati!("Pi: {value:.2}");
    assert_eq!(result, "Pi: 3.14");
}

#[test]
fn test_formati_repeated_expression() {
    let person = (String::from("Bob"), 25);
    let result = formati!("Name: {person.0}, Info: {person.0} is {person.1} years old");
    assert_eq!(result, "Name: Bob, Info: Bob is 25 years old");
}

#[test]
fn test_formati_nested_structures() {
    let data = ((1, 2), (3, 4));
    let result = formati!("Values: {data.0.0}, {data.0.1}, {data.1.0}, {data.1.1}");
    assert_eq!(result, "Values: 1, 2, 3, 4");
}

#[test]
fn test_formati_with_named_args() {
    let x = 10;
    let y = 20;
    let result = formati!("{x} + {y} = {sum}", sum = x + y);
    assert_eq!(result, "10 + 20 = 30");
}

#[test]
fn test_formati_escaped_braces() {
    let result = formati!("{{escaped}} but {not}", not = "interpolated");
    assert_eq!(result, "{escaped} but interpolated");
}

// Full functional test of tracing macros
#[test]
fn test_tracing_macros() {
    let (writer, _guard) = setup_tracing();

    let user_id = 42;
    let username = "test_user";

    // Test info! macro with simple values
    info!("User logged in: {username} with id {user_id}");
    let output = writer.captured_output();
    assert!(output.contains("User logged in: test_user with id 42"));

    // Reset capture
    let (writer, _guard) = setup_tracing();

    // Test debug! macro with dotted notation
    let user = (username, user_id);
    debug!("Debug info: username={user.0}, id={user.1}");
    let output = writer.captured_output();
    assert!(output.contains("Debug info: username=test_user, id=42"));

    // Reset capture
    let (writer, _guard) = setup_tracing();

    // Test error! macro with named arguments
    error!("Error processing user {name}", name = username);
    let output = writer.captured_output();
    assert!(output.contains("Error processing user test_user"));

    // Reset capture
    let (writer, _guard) = setup_tracing();

    // Test trace! with target and repeated expressions
    trace!(target: "auth_service", "User {user.0} logged in with ID {user.1}, welcome {user.0}!");
    let output = writer.captured_output();
    assert!(output.contains("User test_user logged in with ID 42, welcome test_user!"));

    // Reset capture
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

// Advanced usage with mixed features
#[test]
fn test_complex_integration() {
    let person = ("Alice", 30, "Engineer");
    let address = ("123 Main St", "Anytown", "USA");

    let complex = formati!(
        "Person: {person.0}, Age: {person.1}\n\
             Job: {person.2}\n\
             Address: {address.0}, {address.1}, {address.2}\n\
             Person again: {person.0}",
    );

    assert_eq!(
        complex,
        "Person: Alice, Age: 30\n\
             Job: Engineer\n\
             Address: 123 Main St, Anytown, USA\n\
             Person again: Alice"
    );

    // Also test with tracing
    let (writer, _guard) = setup_tracing();

    info!(
        department = "Engineering",
        years_of_service = 5,
        "Complex info: {person.0} is a {person.2} living in {address.1}, {address.2}"
    );

    let output = writer.captured_output();
    assert!(output.contains("Complex info: Alice is a Engineer living in Anytown, USA"));
}

#[test]
fn test_formati_struct_fields_and_methods() {
    // Define a realistic struct with fields and methods
    struct Employee {
        id: u32,
        salary: i32,
        department: String,
    }

    impl Employee {
        fn get_title(&self) -> &str {
            "Software Engineer"
        }

        fn format_id(&self) -> String {
            format!("EMP-{:04}", self.id)
        }
    }

    // Create test instance
    let employee = Employee {
        id: 157,
        salary: 85000,
        department: String::from("Engineering"),
    };

    // Additional tuple for testing
    let project_data = (2023, "Project Formati");

    // Test accessing struct fields, method calls, and repeated expressions
    let result = formati!(
        "Employee ID: {employee.id}, Department: {employee.department}\n\
         Salary: ${employee.salary}, Title: {employee.get_title()}\n\
         Formatted ID: {employee.format_id()}, Employee ID again: {employee.id}\n\
         Project Year: {project_data.0}, Project Name: {project_data.1}"
    );

    assert_eq!(
        result,
        "Employee ID: 157, Department: Engineering\n\
         Salary: $85000, Title: Software Engineer\n\
         Formatted ID: EMP-0157, Employee ID again: 157\n\
         Project Year: 2023, Project Name: Project Formati"
    );

    // Verify that method calls are handled correctly
    let method_result = formati!("Employee: {employee.format_id()} - {employee.get_title()}");
    assert_eq!(method_result, "Employee: EMP-0157 - Software Engineer");

    // Test with formatting specifiers
    let detail_result = formati!(
        "ID: {employee.id:04}, Salary: {employee.salary:+}, Department: {employee.department:.5}"
    );
    assert_eq!(detail_result, "ID: 0157, Salary: +85000, Department: Engin");
}
