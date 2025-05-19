//! `formati!` – identical to `format!`, plus dotted / tuple‑index placeholders
//! and automatic de‑duplication of repeated expressions.

use proc_macro::TokenStream;
use proc_macro2::{Spacing, TokenStream as Ts, TokenTree};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprAssign, ExprLit, Lit, LitStr, Token,
};

/// # formati
///
/// Format strings with enhanced dotted notation and expression deduplication.
///
/// This macro extends Rust's standard `format!` macro with two key features:
/// - Automatic handling of dotted notation for struct fields, tuple elements, and method calls
/// - Deduplication of identical expressions that appear multiple times in the format string
///
/// ## Example
///
/// ```
/// use formati::formati;
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
/// let formatted = formati!(
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
/// use formati::formati;
///
/// let point = (3.14159, 2.71828);
/// let formatted = formati!("Coordinates: ({point.0:.2}, {point.1:.3})");
/// assert_eq!(formatted, "Coordinates: (3.14, 2.718)");
/// ```
///
/// ## How It Works
///
/// The macro processes the format string at compile time, extracting dotted expressions,
/// deduplicating them, and transforming the format string to use standard formatting syntax.
/// This avoids evaluating repeated expressions multiple times at runtime.
#[proc_macro]
pub fn formati(input: TokenStream) -> TokenStream {
    let Input { fmt_lit, rest } = parse_macro_input!(input as Input);

    // Process the format string
    let (out_lit, dot_args) = process_format_string(&fmt_lit);

    let (named, positional) = categorize_arguments(rest);

    let lit = LitStr::new(&out_lit, fmt_lit.span());

    TokenStream::from(quote! {
        ::std::format!(
            #lit
            #(, #named)*
            #(, #dot_args)*
            #(, #positional)*
        )
    })
}

/// `infox!( target: "my_app", user.id, ?error, "Hello {user.name}" )`
#[proc_macro]
pub fn info(input: TokenStream) -> TokenStream {
    tracing_like("info", input)
}

/// `debugx!(a.b, "state = {a.b:?}")`
#[proc_macro]
pub fn debug(input: TokenStream) -> TokenStream {
    tracing_like("debug", input)
}

#[proc_macro]
pub fn trace(input: TokenStream) -> TokenStream {
    tracing_like("trace", input)
}

#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream {
    tracing_like("error", input)
}

/// Wrapper for print! macro with dotted/tuple notation
#[proc_macro]
pub fn printi(input: TokenStream) -> TokenStream {
    std_io_like("print", input)
}

/// Wrapper for println! macro with dotted/tuple notation
#[proc_macro]
pub fn printlni(input: TokenStream) -> TokenStream {
    std_io_like("println", input)
}

/// Process a format string, handling dotted/tuple notations
fn process_format_string(fmt_lit: &LitStr) -> (String, Vec<proc_macro2::TokenStream>) {
    let src = fmt_lit.value();
    let mut out_lit = String::with_capacity(src.len());
    let mut dot_args = Vec::<proc_macro2::TokenStream>::new();
    let mut expr_map: HashMap<String, usize> = HashMap::new(); // expression → index

    let bytes = src.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            // escaped '{{'
            b'{' if bytes.get(i + 1) == Some(&b'{') => {
                out_lit.push_str("{{");
                i += 2;
            }
            // escaped '}}'
            b'}' if bytes.get(i + 1) == Some(&b'}') => {
                out_lit.push_str("}}");
                i += 2;
            }
            // start of a real placeholder
            b'{' => {
                let start_inner = i + 1;
                let mut j = start_inner;
                let mut depth = 1;
                while j < bytes.len() && depth != 0 {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                assert!(depth == 0, "formati!: unmatched `{{`");

                // j is now *one past* the matching `}`
                let piece = &src[start_inner..j - 1]; // inside braces
                i = j; // continue after `}`

                let (head, spec) = split_head_spec(piece);
                if head.contains('.') {
                    // dotted/tuple placeholder: de‑duplicate identical expressions
                    let expr: Expr = syn::parse_str(head).expect("formati!: invalid expression");
                    let key = head.to_string();

                    let idx = match expr_map.get(&key) {
                        Some(&idx) => idx,
                        None => {
                            let idx = dot_args.len();
                            expr_map.insert(key, idx);
                            dot_args.push(expr.to_token_stream());
                            idx
                        }
                    };

                    // replace with indexed `{idx[:spec]}` placeholder
                    out_lit.push('{');
                    out_lit.push_str(&idx.to_string());
                    if !spec.is_empty() {
                        out_lit.push(':');
                        out_lit.push_str(spec);
                    }
                    out_lit.push('}');
                } else {
                    // keep original placeholder verbatim
                    out_lit.push('{');
                    out_lit.push_str(piece);
                    out_lit.push('}');
                }
            }
            // ordinary character
            ch => {
                out_lit.push(ch as char);
                i += 1;
            }
        }
    }

    (out_lit, dot_args)
}

/// Split arguments into named and positional
fn categorize_arguments(args: Punctuated<Expr, Token![,]>) -> (Vec<Ts>, Vec<Ts>) {
    let mut named = Vec::new();
    let mut positional = Vec::new();
    for expr in args {
        match expr {
            x @ Expr::Assign(ExprAssign { .. }) => named.push(x.to_token_stream()),
            x => positional.push(x.to_token_stream()),
        }
    }
    (named, positional)
}

// split `HEAD[:SPEC]`, ignoring `::` (path separators)
fn split_head_spec(s: &str) -> (&str, &str) {
    let mut chars = s.char_indices();
    while let Some((idx, c)) = chars.next() {
        if c == ':' {
            // not part of '::'
            if !matches!(chars.next(), Some((_, ':'))) {
                return (&s[..idx], &s[idx + 1..]);
            }
        }
    }
    (s, "")
}

/// Split on *top-level* commas — nothing else
fn split_top_level(stream: Ts) -> Vec<Ts> {
    let mut segs = Vec::<Ts>::new();
    let mut cur = Ts::new();
    for tt in stream {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == ',' && p.spacing() == Spacing::Alone => {
                segs.push(cur);
                cur = Ts::new();
            }
            _ => cur.extend(std::iter::once(tt)),
        }
    }
    segs.push(cur);
    segs
}

/// Find format string and process tracing-like macros
fn tracing_like(kind: &str, input: TokenStream) -> TokenStream {
    let segments = split_top_level(Ts::from(input));

    // ─── find the *last* string-literal segment — that starts the template –──
    let split_at = segments
        .iter()
        .rposition(|seg| {
            parse2::<Expr>(seg.clone())
                .ok()
                .and_then(|e| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(_), ..
                    }) = e
                    {
                        Some(())
                    } else {
                        None
                    }
                })
                .is_some()
        })
        .expect("`*x!` macro needs a string literal message");

    let (front_segs, back_segs) = segments.split_at(split_at);
    let fmt_seg = &back_segs[0]; // the literal
    let rest_segs = &back_segs[1..]; // possible extra exprs

    // ─── pull out the literal text & span —──────────────────────────────────
    let lit_expr: Expr = parse2(fmt_seg.clone()).unwrap();
    let lit_str: LitStr = match lit_expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => s,
        _ => unreachable!(),
    };

    // Process the format string to handle dotted/tuple notation
    let (out, dot_args) = process_format_string(&lit_str);
    let lit = LitStr::new(&out, lit_str.span());

    // ─── extra args (after the literal): named first, then positional –──────
    let mut named = Vec::<Ts>::new();
    let mut positional = Vec::<Ts>::new();
    for seg in rest_segs {
        let expr: Expr = parse2(seg.clone()).expect("invalid expression after template");
        match expr {
            Expr::Assign(ExprAssign { .. }) => named.push(expr.to_token_stream()),
            _ => positional.push(expr.to_token_stream()),
        }
    }

    // ─── emit the real tracing macro call –──────────────────────────────────
    let tracing_macro = syn::Ident::new(kind, proc_macro2::Span::call_site());
    let front: Vec<&Ts> = front_segs.iter().collect();

    quote! {
        ::tracing::#tracing_macro!(
            #(#front ,)*
            #lit
            #(, #named)*
            #(, #dot_args)*
            #(, #positional)*
        )
    }
    .into()
}

/// Process std I/O macros like print! and println!
fn std_io_like(kind: &str, input: TokenStream) -> TokenStream {
    // Parse the input similarly to formati
    let Input { fmt_lit, rest } = parse_macro_input!(input as Input);

    // Process the format string
    let (out_lit, dot_args) = process_format_string(&fmt_lit);

    // Categorize arguments
    let (named, positional) = categorize_arguments(rest);

    // Create the final output
    let lit = LitStr::new(&out_lit, fmt_lit.span());
    let io_macro = syn::Ident::new(kind, proc_macro2::Span::call_site());

    TokenStream::from(quote! {
        ::std::#io_macro!(
            #lit
            #(, #named)*
            #(, #dot_args)*
            #(, #positional)*
        )
    })
}

/// input: `"literal"` [`,` expr ]*
struct Input {
    fmt_lit: LitStr,
    rest: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let fmt_lit: LitStr = input.parse()?;
        let rest = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            Punctuated::parse_terminated(input)?
        } else {
            Punctuated::new()
        };
        Ok(Self { fmt_lit, rest })
    }
}
