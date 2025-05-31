// Run with: cargo run --example tracing_integration --features tracing

#[cfg(feature = "tracing")]
use formati::{debug, error, info, trace, warn};
#[cfg(feature = "tracing")]
use tracing_subscriber::FmtSubscriber;

#[cfg(feature = "tracing")]
fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::builder().finish()).unwrap();

    let user = ("Alice", 101);
    let session = ("sess_abc123", 3600);

    trace!("Starting authentication for {user.0}");
    debug!("Loading profile for user ID {user.1}");
    info!("User {user.0} logged in with session {session.0}");
    warn!("Session {session.0} expires in {session.1} seconds");
    error!("Failed to process request for user {user.0}");

    // With structured fields
    info!(
        user_id = user.1,
        session_id = session.0,
        "Session created for {user.0} lasting {session.1}s"
    );
}

#[cfg(not(feature = "tracing"))]
fn main() {
    println!(
        "This example requires the 'tracing' feature. Run with: cargo run --example tracing_integration --features tracing"
    );
}
