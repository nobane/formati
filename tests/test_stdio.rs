#![cfg(feature = "stdio")]
mod test_stdio {
    use formati::{print, println};
    use std::fs::{read_to_string, remove_file};
    use std::io::{self, Write};
    use std::path::PathBuf;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };
    use stdio_override::StdoutOverride;

    static CAPTURE_LOCK: Mutex<()> = Mutex::new(());

    // Generate a unique tmp‑file path for every capture.
    fn temp_path() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut p = std::env::temp_dir();
        p.push(format!("formati_stdio_test_{id}.txt"));
        p
    }

    /// Redirect stdout to a temp file while running `f` **inside a new
    /// thread** (so we escape the test‑harness’s per‑thread capture).
    fn with_captured_stdout<F>(f: F) -> String
    where
        F: FnOnce() + Send + 'static,
    {
        let _lock = CAPTURE_LOCK.lock().unwrap();

        let path = temp_path();
        io::stdout().flush().ok();
        let guard = StdoutOverride::from_file(&path).expect("override failed");

        std::thread::spawn(move || {
            f();
            io::stdout().flush().ok();
        })
        .join()
        .expect("thread panicked in with_captured_stdout");

        // drop the guard *after* the prints are done so everything is flushed
        drop(guard);

        let contents = read_to_string(&path).expect("read capture file");
        let _ = remove_file(&path);
        contents
    }

    // Tests

    #[test]
    fn test_print_basic() {
        let (name, age) = ("Alice", 30);
        let out = with_captured_stdout(move || {
            print!("Name: {name}, Age: {age}");
        });
        assert_eq!(out, "Name: Alice, Age: 30");
    }

    #[test]
    fn test_println_basic() {
        let (name, age) = ("Bob", 25);
        let out = with_captured_stdout(move || {
            println!("Name: {name}, Age: {age}");
        });
        assert_eq!(out, "Name: Bob, Age: 25\n");
    }

    #[test]
    fn test_stdio_dotted_access() {
        struct User {
            id: u32,
            name: String,
        }
        impl User {
            fn format_id(&self) -> String {
                format!("USER-{:<04}", self.id)
            }
        }

        let user = User {
            id: 42,
            name: "Carol".into(),
        };

        let out = with_captured_stdout(move || {
            println!("User: {user.name} (ID: {user.id})");
            print!("Formatted ID: {user.format_id()}");
        });

        assert_eq!(out, "User: Carol (ID: 42)\nFormatted ID: USER-0042");
    }

    #[test]
    fn test_stdio_repeated_expressions() {
        let point = (3.1, 2.71);
        let out = with_captured_stdout(move || {
            println!("Point: ({point.0}, {point.1})");
            println!(
                "Normalized: ({point.0}/{point.0+point.1:.2}, {point.1}/{point.0+point.1:.2})"
            );
        });
        assert!(out.contains("Point: (3.1, 2.71)"));
        assert!(out.contains("Normalized: (3.1/5.81, 2.71/5.81)"));
    }
}
