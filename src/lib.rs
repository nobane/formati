use proc_macro::TokenStream;
use proc_macro2::Span;

mod formati_args;
use formati_args::wrap;

/// # format
///
/// Format strings with enhanced dot notation and arbitrary expression support.
///
/// This macro extends Rust's standard `format!` macro with two key features:
/// - Automatic handling of dot notation and arbitrary expressions for struct fields, tuple elements, method calls, and more
/// - Deduplication of identical expressions that appear multiple times in the format string
///
/// ## Example
///
/// ```
/// use formati::format;
///
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// impl Point {
///     fn distance_from_origin(&self) -> f32 {
///         (self.x * self.x + self.y * self.y).sqrt()
///     }
/// }
///
/// let point = Point { x: 3.0, y: 4.0 };
///
/// // Multiple uses of point.x, point.y, and point.distance_from_origin()
/// // will only be evaluated once each
/// let formatted = format!(
///     "Point: ({point.x}, {point.y})\n\
///      Distance: {point.distance_from_origin()}\n\
///      Normalized: ({point.x}/{point.distance_from_origin()}, {point.y}/{point.distance_from_origin()})"
/// );
/// ```
///
/// ## Format Specifiers
///
/// All standard format specifiers are supported, just like in `format!`:
///
/// ```
/// use formati::format;
///
/// let point = (3.14159, 2.71828);
/// let formatted = format!("Coordinates: ({point.0:.2}, {point.1:.3})");
/// assert_eq!(formatted, "Coordinates: (3.14, 2.718)");
/// ```
///
/// ## How It Works
///
/// The macro processes the format string at compile time, extracting dot notation and arbitrary expressions,
/// deduplicating them, and transforming the format string to use standard formatting syntax.
/// This avoids evaluating repeated expressions multiple times at runtime.
#[proc_macro]
pub fn format(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => std::format);
    wrap(wrapped, input)
}

/// Enhanced version of print! with dot notation and arbitrary expression support
///
/// This macro wraps the standard print! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::print;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// print!("User {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(feature = "stdio")]
pub fn print(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => std::print);
    wrap(wrapped, input)
}

/// Enhanced version of println! with dot notation and arbitrary expression support
///
/// This macro wraps the standard println! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::println;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// println!("User {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(feature = "stdio")]
pub fn println(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => std::println);
    wrap(wrapped, input)
}

/// Enhanced version of eprint! with dot notation and arbitrary expression support
///
/// This macro wraps the standard eprint! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::eprint;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// eprint!("Error: Failed to process user {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(feature = "stdio")]
pub fn eprint(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => std::eprint);
    wrap(wrapped, input)
}

/// Enhanced version of eprintln! with dot notation and arbitrary expression support
///
/// This macro wraps the standard eprintln! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::eprintln;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// eprintln!("Error: Failed to process user {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(feature = "stdio")]
pub fn eprintln(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => std::eprintln);
    wrap(wrapped, input)
}

/// Enhanced version of dbg! with dot notation and arbitrary expression support
///
/// This macro wraps the standard dbg! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::dbg;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// dbg!("Debug: user {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(feature = "stdio")]
pub fn dbg(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => std::dbg);
    wrap(wrapped, input)
}

/// Enhanced version of panic! with dot notation and arbitrary expression support
///
/// This macro wraps the standard panic! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::panic;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// panic!("Critical error: user {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(feature = "stdio")]
pub fn panic(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => std::panic);
    wrap(wrapped, input)
}

/// Enhanced version of anyhow! with dot notation and arbitrary expression support
///
/// This macro wraps the standard anyhow! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::anyhow;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// let err = anyhow!("Failed to process user {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(feature = "anyhow")]
pub fn anyhow(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => anyhow::anyhow);
    wrap(wrapped, input)
}

/// Enhanced version of bail! with dot notation and arbitrary expression support
///
/// This macro wraps the standard bail! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::bail;
/// use anyhow::Result;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// fn process_user(user: &User) -> Result<()> {
///     if user.id == 0 {
///         bail!("Invalid user {user.name} with ID {user.id}");
///     }
///     Ok(())
/// }
/// ```
#[proc_macro]
#[cfg(feature = "anyhow")]
pub fn bail(input: TokenStream) -> TokenStream {
    let wrapped = syn::parse_quote_spanned!(Span::call_site() => anyhow::bail);
    wrap(wrapped, input)
}

#[cfg(feature = "tracing")]
mod like_tracing;

/// Enhanced version of trace! with dot notation and arbitrary expression support
///
/// This macro wraps the standard trace! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::trace;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// trace!("Entering function with user {user.name} and ID {user.id}");
/// ```
#[proc_macro]
#[cfg(any(feature = "log", feature = "tracing"))]
pub fn trace(input: TokenStream) -> TokenStream {
    #[cfg(feature = "log")]
    {
        let wrapped = syn::parse_quote_spanned!(Span::call_site() => log::trace);
        wrap(wrapped, input)
    }
    #[cfg(feature = "tracing")]
    {
        like_tracing::wrap("trace", input)
    }
}

/// Enhanced version of debug! with dot notation and arbitrary expression support
///
/// This macro wraps the standard debug! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::debug;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// debug!("Debug user object state: name={user.name}, id={user.id}");
/// ```
#[proc_macro]
#[cfg(any(feature = "log", feature = "tracing"))]
pub fn debug(input: TokenStream) -> TokenStream {
    #[cfg(feature = "log")]
    {
        let wrapped = syn::parse_quote_spanned!(Span::call_site() => log::debug);
        wrap(wrapped, input)
    }
    #[cfg(feature = "tracing")]
    {
        like_tracing::wrap("debug", input)
    }
}

/// Enhanced version of info! with dot notation and arbitrary expression support
///
/// This macro wraps the standard info! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::info;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// info!("Processing user {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(any(feature = "log", feature = "tracing"))]
pub fn info(input: TokenStream) -> TokenStream {
    #[cfg(feature = "log")]
    {
        let wrapped = syn::parse_quote_spanned!(Span::call_site() => log::info);
        wrap(wrapped, input)
    }
    #[cfg(feature = "tracing")]
    {
        like_tracing::wrap("info", input)
    }
}

/// Enhanced version of warn! with dot notation and arbitrary expression support
///
/// This macro wraps the standard warn! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::warn;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// warn!("Warning: user {user.name} has suspicious activity");
/// ```
#[proc_macro]
#[cfg(any(feature = "log", feature = "tracing"))]
pub fn warn(input: TokenStream) -> TokenStream {
    #[cfg(feature = "log")]
    {
        let wrapped = syn::parse_quote_spanned!(Span::call_site() => log::warn);
        wrap(wrapped, input)
    }
    #[cfg(feature = "tracing")]
    {
        like_tracing::wrap("warn", input)
    }
}

/// Enhanced version of error! with dot notation and arbitrary expression support
///
/// This macro wraps the standard error! macro with support for
/// dot notation and arbitrary expressions with automatic expression deduplication.
///
/// # Example
///
/// ```
/// use formati::error;
///
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// let user = User {
///    id: 42,
///    name: String::from("Alice"),
/// };
///
/// error!("Failed to process user {user.name} with ID {user.id}");
/// ```
#[proc_macro]
#[cfg(any(feature = "log", feature = "tracing"))]
pub fn error(input: TokenStream) -> TokenStream {
    #[cfg(feature = "log")]
    {
        let wrapped = syn::parse_quote_spanned!(Span::call_site() => log::error);
        wrap(wrapped, input)
    }
    #[cfg(feature = "tracing")]
    {
        like_tracing::wrap("error", input)
    }
}
