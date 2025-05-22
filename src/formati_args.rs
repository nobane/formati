use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens as _};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprAssign, LitStr, Token,
};

/// input: `"literal"` [`,` expr ]*
struct Input {
    fmt_lit: LitStr,
    rest: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // the format string literal is always required
        let fmt_lit: LitStr = input.parse()?;

        // no arguments at all
        if input.is_empty() {
            return Ok(Self {
                fmt_lit,
                rest: Punctuated::new(),
            });
        }

        // if we **do** see a comma, decide whether it starts a real argument list
        // or is just a trailing comma before the right-paren / end-of-macro.
        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?; // eat the comma

            // it's a lone trailing comma
            if input.is_empty() {
                return Ok(Self {
                    fmt_lit,
                    rest: Punctuated::new(),
                });
            }

            // more input, parse the normal arg list
            let rest = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
            return Ok(Self { fmt_lit, rest });
        }

        // anything else after the literal is a syntax error; let syn report it
        Err(input.error("expected `,` or end of macro input"))
    }
}

/// Process anyhow-like error macros with dotted notation support
pub fn wrap(wrapped: TokenStream2, input: TokenStream) -> TokenStream {
    let Input { fmt_lit, rest } = parse_macro_input!(input as Input);

    let (out_lit, dot_args) = formati_args(&fmt_lit);

    let mut named = Vec::new();
    let mut positional = Vec::new();
    for expr in rest {
        match expr {
            x @ Expr::Assign(ExprAssign { .. }) => named.push(x.to_token_stream()),
            x => positional.push(x.to_token_stream()),
        }
    }

    let lit = LitStr::new(&out_lit, fmt_lit.span());

    TokenStream::from(quote! {
        ::#wrapped!(
            #lit
            #(, #named)*
            #(, #dot_args)*
            #(, #positional)*
        )
    })
}

/// Process a format string, handling dotted/tuple notations and complex expressions
pub fn formati_args(fmt_lit: &LitStr) -> (String, Vec<proc_macro2::TokenStream>) {
    let src = fmt_lit.value();
    let mut out_lit = String::with_capacity(src.len());
    let mut dot_args = Vec::<proc_macro2::TokenStream>::new();
    let mut expr_map: HashMap<String, usize> = HashMap::new();

    let bytes = src.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'{' if bytes.get(i + 1) == Some(&b'{') => {
                out_lit.push_str("{{");
                i += 2;
            }
            b'}' if bytes.get(i + 1) == Some(&b'}') => {
                out_lit.push_str("}}");
                i += 2;
            }
            b'{' => {
                // Find the matching closing brace, properly handling nested braces
                let start_inner = i + 1;
                let mut j = start_inner;
                let mut depth = 1;
                let mut in_string = false;
                let mut in_char = false;
                let mut escape_next = false;

                while j < bytes.len() && depth != 0 {
                    let ch = bytes[j] as char;

                    if escape_next {
                        escape_next = false;
                        j += 1;
                        continue;
                    }

                    match ch {
                        '\\' if in_string || in_char => {
                            escape_next = true;
                        }
                        '"' if !in_char => {
                            in_string = !in_string;
                        }
                        '\'' if !in_string => {
                            // Simple char literal detection
                            in_char = !in_char;
                        }
                        '{' if !in_string && !in_char => {
                            depth += 1;
                        }
                        '}' if !in_string && !in_char => {
                            depth -= 1;
                        }
                        _ => {}
                    }
                    j += 1;
                }

                if depth != 0 {
                    panic!("formati!: unmatched `{{` at position {}", i);
                }

                let piece = &src[start_inner..j - 1];
                i = j;

                let (head, spec) = split_head_spec(piece);

                if should_extract_expression(head) {
                    // Try to parse the expression - if it fails, treat as regular placeholder
                    match syn::parse_str::<Expr>(head) {
                        Ok(expr) => {
                            // Successfully parsed - extract it
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
                        }
                        Err(_) => {
                            // Failed to parse - keep as regular placeholder
                            out_lit.push('{');
                            out_lit.push_str(piece);
                            out_lit.push('}');
                        }
                    }
                } else {
                    // keep original placeholder verbatim
                    out_lit.push('{');
                    out_lit.push_str(piece);
                    out_lit.push('}');
                }
            }
            ch => {
                out_lit.push(ch as char);
                i += 1;
            }
        }
    }

    (out_lit, dot_args)
}

// split `HEAD[:SPEC]`, ignoring `::` (path separators) and handling complex expressions
fn split_head_spec(s: &str) -> (&str, &str) {
    let mut chars = s.char_indices().peekable();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut brace_depth = 0;
    let mut angle_depth = 0;
    let mut in_string = false;
    let mut in_char = false;
    let mut escape_next = false;

    while let Some((idx, c)) = chars.next() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string || in_char => {
                escape_next = true;
            }
            '"' if !in_char => {
                in_string = !in_string;
            }
            '\'' if !in_string => {
                // Handle char literals and lifetimes
                if in_char {
                    in_char = false;
                } else {
                    // Check if this is a lifetime (preceded by &, comma, space, or start)
                    let is_lifetime = if idx == 0 {
                        false
                    } else {
                        match s.chars().nth(idx - 1) {
                            Some('&') | Some(',') | Some(' ') | Some('<') => {
                                // Look ahead to see if it's a lifetime
                                let rest: String = chars.clone().map(|(_, c)| c).collect();
                                rest.chars()
                                    .next()
                                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                            }
                            _ => false,
                        }
                    };

                    if !is_lifetime {
                        in_char = true;
                    }
                }
            }
            _ if in_string || in_char => {
                continue;
            }
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '<' => {
                // More sophisticated generic detection
                if should_count_as_generic(s, idx) {
                    angle_depth += 1;
                }
            }
            '>' => {
                if angle_depth > 0 {
                    angle_depth -= 1;
                }
            }
            ':' if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && angle_depth == 0 =>
            {
                // Check if this is part of '::'
                if let Some((_, ':')) = chars.peek() {
                    chars.next(); // consume the second ':'
                    continue;
                }
                // Found a format specifier separator
                return (&s[..idx], &s[idx + 1..]);
            }
            _ => {}
        }
    }

    (s, "")
}

fn should_count_as_generic(s: &str, idx: usize) -> bool {
    if idx == 0 {
        return false;
    }

    match s.chars().nth(idx - 1) {
        // Definitely generic contexts
        Some(c) if c.is_alphanumeric() || c == '_' => true, // identifier
        Some(':') => true,                                  // ::< or :
        Some('>') => true,                                  // nested generics Type<U>

        // Definitely comparison contexts
        Some('=') | Some('!') | Some('<') => false, // ==<, !=<, <<, etc.

        // Whitespace - need deeper analysis
        Some(' ') | Some('\t') | Some('\n') => is_likely_generic_with_space(s, idx),

        // Default to comparison for safety
        _ => false,
    }
}

fn is_likely_generic_with_space(s: &str, idx: usize) -> bool {
    // Look backward to find the last non-whitespace token
    let before_whitespace = s[..idx].trim_end();

    if before_whitespace.is_empty() {
        return false;
    }

    // Check if it ends with something that could take generics
    let last_token = before_whitespace.split_whitespace().last().unwrap_or("");

    // Pattern matching for likely generic contexts
    last_token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
        && (last_token.contains("::") ||
    last_token.chars().next().is_some_and(|c| c.is_uppercase()) || // PascalCase
    last_token.ends_with("_t") || // C-style type suffixes
    last_token.len() > 1) // Avoid single letters which are usually variables
}

fn should_extract_expression(head: &str) -> bool {
    // Don't extract if it's just a simple identifier or number
    if head.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return false;
    }

    // Don't extract simple literals
    if head.parse::<i64>().is_ok() || head.parse::<f64>().is_ok() {
        return false;
    }

    // Extract if it contains dots, method calls, array indexing, references, or complex expressions
    head.contains('.')
       || head.contains("::")
       || head.contains('(')
       || head.contains('[')
       || head.contains('<')
       || head.starts_with('&')       // Reference expressions
       || head.starts_with("mut ")    // Mutable references
       || head.starts_with("&mut ")   // &mut expressions
       || head.starts_with('*')       // Dereference expressions
       || head.contains(" as ")       // Type casting
       || head.contains('?')          // Try operator
       || head.contains("..")         // Range expressions
       || head.contains('{')          // Struct literals, closures, blocks
       || head.contains(" if ")       // if expressions
       || head.contains(" match ")    // match expressions
       || is_complex_expression(head) // More sophisticated detection
}

fn is_complex_expression(head: &str) -> bool {
    // Check for operators that indicate complex expressions
    let operators = [
        "+", "-", "*", "/", "%", // Arithmetic
        "==", "!=", "<", ">", "<=", ">=", // Comparison
        "&&", "||", "!", // Logical
        "&", "|", "^", "<<", ">>", // Bitwise
        "=", "+=", "-=", "*=", "/=", // Assignment
    ];

    // Check for operators (but be careful about false positives in strings)
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = head.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];

        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => {
                escape_next = true;
            }
            '"' => {
                in_string = !in_string;
            }
            _ if in_string => {
                continue;
            }
            _ => {
                // Check if any operator starts at this position
                let remaining = &head[i..];
                for op in &operators {
                    if remaining.starts_with(op) {
                        // Make sure it's not part of a larger token
                        let after_op = i + op.len();
                        let before_ok = i == 0 || !chars[i - 1].is_alphanumeric();
                        let after_ok =
                            after_op >= chars.len() || !chars[after_op].is_alphanumeric();

                        if before_ok && after_ok {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}
