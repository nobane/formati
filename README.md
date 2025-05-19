# formati

> **Enhanced Rust formatting macros with dotted notation and expression interpolation**

`formati` is a collection of procedural macros that extend Rust's standard formatting facilities in two key ways:

- Automatic handling of dotted notation for struct fields, tuple elements, and method calls
- Deduplication of identical expressions that appear multiple times in the format string

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **Dotted notation**: Access struct fields and tuple elements with natural dot notation
- **Expression deduplication**: Automatically prevents repeated evaluation of the same expressions
- **Full format specifier support**: Works with all standard format specifiers (`{:?}`, `{:.2}`, etc.)
- **Tracing integration**: Enhanced versions of common tracing macros
- **Standard library wrappers**: Drop-in replacements for `print!` and `println!`

## Installation

Add `formati` to your `Cargo.toml`:

```toml
[dependencies]
formati = "0.1"
```

## Usage

### Basic Formatting with `formati!`

```rust
use formati::formati;

struct User {
    id: u32,
    role: String,
}

impl User {
    fn display_name(&self) -> String {
        formati!("USER-{self.id}")
    }
}

fn main() {
    let coordinates = (42.5, 73.1);
    let user = User { id: 101, role: String::from("Admin") };

    let s = formati!(
        "Position: ({coordinates.0}, {coordinates.1})\n\
         User: {user.display_name()} (ID: {user.id}, Role: {user.role})"
    );

    assert_eq!(
        s,
        "Position: (42.5, 73.1)\nUser: USER-101 (ID: 101, Role: Admin)"
    );
}
```

### Format Specifiers

```rust
use formati::formati;

fn main() {
    let coords = (10.12345, 20.67890);
    let formatted = formati!("Location: ({coords.0:.2}, {coords.1:.2})");
    // "Location: (10.12, 20.68)"
}
```

### Tracing Integration

`formati`-style versions of `tracing` macros that support dotted notation:

```rust
use formati::{debug, error, info, trace}; // use in place of tracing::{debug, error, info, trace}
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Set up tracing
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder().finish()
    ).unwrap();

    let user = (String::from("Alice"), 101);

    // Use enhanced tracing macros
    info!("User {user.0} logged in with ID {user.1}");
    debug!(user_type = "admin", "Processing request for {user.0}");
    error!("Failed to process request for user.id = {user.1}");
}
```

### Print and Println Wrappers

```rust
use formati::{printi, printlni};

fn main() {
    let point = (5, 10);

    // Use enhanced print macros
    printi!("Starting at ({point.0}, {point.1})...");
    printlni!("Coordinates: ({point.0}, {point.1})");
}
```

## How It Works

The `formati` crate processes format strings at compile time to:

1. Find placeholders with dotted notation (`{example.field}`)
2. Extract these expressions and deduplicate them
3. Replace them with indexed placeholders
4. Add the extracted expressions as arguments to the underlying format macro

This approach avoids evaluating the same expression multiple times and makes your format strings more readable.

### Expansion Demonstration

```rust
struct Point {
    x: f32,
    y: f32,
}

let point = Point { x: 3.0, y: 4.0 };

let info = formati!("Point: ({point.x}, {point.y}), X-coord: {point.x}, Y-coord: {point.y}");
```

The `format!` macro would expand to:

```rust
alloc::__export::must_use({
    let res = alloc::fmt::format(alloc::__export::format_args!(
        "Point: ({0}, {1}), X-coord: {0}, Y-coord: {1}",
        point.x,
        point.y
    ));
    res
})
```


## License

This project is licensed under the MIT License.

