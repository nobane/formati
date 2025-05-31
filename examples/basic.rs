// Run with: cargo run --example basic

use formati::format;

struct User {
    id: u32,
    name: String,
    role: String,
}

impl User {
    fn display_name(&self) -> String {
        format!("USER-{self.id}")
    }
}

fn main() {
    let point = (10.5, 20.3);
    // Basic dotted notation
    let msg = format!("Point: ({point.0}, {point.1})");
    assert_eq!(msg, "Point: (10.5, 20.3)");

    // Complex expressions
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        role: "Admin".to_string(),
    };

    let welcome = format!("Welcome {user.display_name()}! Role: {user.role}");
    assert_eq!(welcome, "Welcome USER-42! Role: Admin");

    // Expression deduplication - user.id is only evaluated once
    let info = format!("User {user.id} has ID {user.id} and name {user.name}");
    assert_eq!(info, "User 42 has ID 42 and name Alice");

    // Works with format specifiers
    let coords = (12.34567, 98.76543);
    let formatted = format!("Coordinates: ({coords.0:.2}, {coords.1:.3})");
    assert_eq!(formatted, "Coordinates: (12.35, 98.765)");

    println!("All assertions passed!");
}
