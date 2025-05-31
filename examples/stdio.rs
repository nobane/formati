// Run with: cargo run --example stdio --features stdio

#[cfg(feature = "stdio")]
use formati::{eprint, eprintln, print, println};

#[cfg(feature = "stdio")]
fn main() {
    let user = ("Bob", 25);
    let location = ("New York", "NY");

    println!("User: {user.0}, Age: {user.1}");
    print!("Location: {location.0}, {location.1}");

    // Error output
    eprintln!("Error: Failed to process user {user.0}");
    eprint!("Warning: User {user.0} from {location.0}");
}

#[cfg(not(feature = "stdio"))]
fn main() {
    println!(
        "This example requires the 'stdio' feature. Run with: cargo run --example stdio --features stdio"
    );
}
