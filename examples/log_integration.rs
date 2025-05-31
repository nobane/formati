// Run with: cargo run --example log_integration --features log

#[cfg(feature = "log")]
use formati::{debug, error, info, trace, warn};
#[cfg(feature = "log")]
use log::{LevelFilter, Log, Metadata, Record};

#[cfg(feature = "log")]
struct SimpleLogger {
    level: LevelFilter,
}

#[cfg(feature = "log")]
impl SimpleLogger {
    fn new(level: LevelFilter) -> Self {
        Self { level }
    }
}

#[cfg(feature = "log")]
impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}] {}: {}",
                record.target(),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

#[cfg(feature = "log")]
fn main() {
    let logger = SimpleLogger::new(LevelFilter::Trace);
    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(LevelFilter::Trace);

    let user = ("Bob", 42);
    let request = ("GET", "/api/users");

    trace!("Received {request.0} request to {request.1}");
    debug!("Processing request for user {user.0}");
    info!("User {user.0} accessed {request.1}");
    warn!("High request rate from user {user.1}");
    error!("Failed to handle {request.0} {request.1} for user {user.0}");
}

#[cfg(not(feature = "log"))]
fn main() {
    println!(
        "This example requires the 'log' feature. Run with: cargo run --example log_integration --features log"
    );
}
