# formati

> **Evaluate dot notation and arbitrary expressions in Rust format! macros**

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/formati.svg
[crates-url]: https://crates.io/crates/formati
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/nobane/formati/blob/master/LICENSE

Normally, in Rust's format macros, dotted notation or arbitrary expressions must be placed after the format string:

```rust
format!("User ID #{}: {:?}", user.id, user.display_name());
```

Using `formati::format!` extends Rust's standard formatting to handle arbitrary expressions (dot notation, function calls, etc.) directly within the format string:

```rust
use formati::format;

format!("User ID #{user.id}: {user.display_name():?}");
```


- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Basic Formatting](#basic-formatting)
  - [Format Specifiers](#format-specifiers)
  - [`print!` / `println!`](#print--println)
- [Integration Wrappers](#integration-wrappers)
  - [Anyhow](#anyhow-anyhow-bail)
  - [Log](#log)
  - [Tracing](#tracing)
- [How It Works](#how-it-works)
- [What's the catch?](#whats-the-catch)
- [Tests](#tests)
- [License](#license)


## Features

- **Dotted notation**: Access fields and elements with natural dot notation
- **Expression evaluation**: Run arbitrary expressions
- **Argument deduplication**: Simplifies repeated evaluation of the same arguments
- **Fully backwards compatible**: Works with all standard format specifiers (`{:?}`, `{:.2}`, etc.)
- **Integration wrappers**: Drop-in replacements for `std::io` (e.g. `println!`), [anyhow](https://docs.rs/anyhow/latest/anyhow/), [tracing](https://docs.rs/tracing/latest/tracing/) and [log](https://docs.rs/log/latest/log/).


## Installation

Add `formati` to your `Cargo.toml`:

```toml
[dependencies]
formati = "0.1"
```


## Usage

### Basic Formatting

```rust
use formati::format;

struct User {
    id: u32,
    role: String,
}

impl User {
    fn display_name(&self) -> String {
        format!("USER-{self.id}")
    }
}

fn main() {
    let coordinates = (42.5, 73.1);
    let user = User { id: 101, role: String::from("Admin") };

    let s = format!(
        "Position: ({coordinates.0}, {coordinates.1})\n\
         User: {user.display_name()} (ID: {user.id}, Role: {user.role})",
    );

    assert_eq!(
        s,
        "Position: (42.5, 73.1)\nUser: USER-101 (ID: 101, Role: Admin)",
    );
}
```


### Format Specifiers

```rust
use formati::format;

fn main() {
    let coords = (10.12345, 20.67890);
    let s = format!("Location: ({coords.0:.2}, {coords.1:.2})");

    assert_eq!(
        s,
        "Location: (10.12, 20.68)",
    );
}
```


### `print!` / `println!`

Requires `stdio` feature:

```toml
[dependencies]
formati = { version = "0.1", features = ["stdio"] }
```

```rust
use formati::{print, println};

fn main() {
    let point = (5, 10);

    print!("Starting at ({point.0}, {point.1})..."); // prints "Starting at (5, 10)..."
    println!("Coordinates: ({point.0}, {point.1})"); // prints "Coordinates: (5, 10)\n"
}
```


## Integration Wrappers

### Anyhow

Requires `anyhow` feature:


```toml
[dependencies]
formati = { version = "0.1", features = ["anyhow"] }
```

`formati`-style versions of `anyhow` macros:

```rust
use formati::{anyhow, bail};

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

fn process(user: &User) -> anyhow::Result<()> {
    if user.id == 0 {
        bail!("Cannot process zero-id user {user.name} (ID {user.id})");
    }
    Ok(())
}

fn main() {
    let user = User { id: 0, name: "Bob".into() };

    if let Err(e) = process(&user) {
        // Produces: "Cannot process zero-id user Bob (ID 0)"
        eprintln!("{e}");
    }

    // Build an error directly
    let err = anyhow!("Unexpected error for {user.name} with id {user.id}");
    assert_eq!(err.to_string(), "Unexpected error for Bob with id 0");
}

```


### Log

Requires `log` feature:

(**NOTE**: the `log` feature  *cannot* be enabled together with `tracing`)

```toml
[dependencies]
formati = { version = "0.1", features = ["log"] }
```


`formati`-style versions of `log` macros:

```rust
use formati::{debug, error, info, trace, warn}; // instead of log::{…}
use log::LevelFilter;

fn main() {
    simple_logger::init_with_level(LevelFilter::Trace).unwrap();

    let user = ("Alice", 42);

    trace!("Starting auth flow for {user.0} ({user.1})…");
    debug!("Loaded profile for {user.0}");
    info!("User {user.0} logged in with ID {user.1}");
    warn!("Suspicious activity detected for ID {user.1}");
    error!("Failed to handle request for user {user.0}");
}

```


### Tracing

Requires `tracing` feature:

(**NOTE**: the `tracing` feature  *cannot* be enabled together with `log`)

```toml
[dependencies]
formati = { version = "0.1", features = ["tracing"] }
```

`formati`-style versions of `tracing` macros:

```rust
use formati::{debug, error, info, trace}; // use in place of tracing::{debug, error, info, trace}
use tracing_subscriber::FmtSubscriber;

fn main() {
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder().finish()
    ).unwrap();

    let user = (String::from("Alice"), 101);

    trace!(target: "auth", "Authenticating")
    debug!(user_type = "admin", "Processing request for {user.0}");
    info!("User {user.0} logged in with ID {user.1}");
    warn!(data = (13, 37), "Bad data from ID {user.1}")
    error!("Failed to process request for ID = {user.1}");
}
```


## How It Works

The macros processes format strings at compile time to:

1. Find placeholders with dotted notation (`{example.field}`)
2. Extract these expressions and deduplicate them
3. Replace them with indexed placeholders
4. Add the extracted expressions as arguments to the underlying format macro

This approach avoids evaluating the same expression multiple times and makes your format strings more readable.


### Backwards compatibility

The macros are all backwards compatible and can be used as a drop-in replacement.

```rust
struct Point {
    x: f32,
    y: f32,
}

let point = Point { x: 3.0, y: 4.0 };

let info = format!(
    "Point: ({point.x}, {point.y}), X-coord: {point.x}, Y-coord: {point.y}, ({},{})",
    point.x,
    point.y,
);
```

The `format!` macro would expand to:

```rust
alloc::__export::must_use({
    let res = alloc::fmt::format(alloc::__export::format_args!(
        "Point: ({0}, {1}), X-coord: {0}, Y-coord: {1}, ({0}, {1})",
        point.x,
        point.y
    ));
    res
})
```

## What's the catch?

While `formati` makes format strings more readable and convenient at no extra runtime cost, there are some trade-offs to be aware of:

**IDE/Editor Limitations**: Most code editors and IDEs don't recognize variables and expressions inside format string literals. This will likely mean:
- **No syntax highlighting** for the expressions within `{}`
- **No autocomplete** for field names or method calls
- **Refactoring tools most likely won't work** - if you rename a variable or field, the IDE won't automatically update references inside format strings
- **"Find usages"** and similar navigation features may miss references inside format strings




## Tests

Test `formati::format!` macro:

```
cargo test
```

Test `stdio` integration:

```
cargo test-stdio
```

Test `anyhow` integration:

```
cargo test-anyhow
```


Test `log` integration:

```
cargo test-log
```

Test `tracing` integration:

```
cargo test-tracing
```


## License

This project is licensed under the MIT License.
