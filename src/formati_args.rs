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

/// Process a format string, handling dotted/tuple notations
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
                assert!(depth == 0, "format!: unmatched `{{`");

                let piece = &src[start_inner..j - 1];
                i = j;

                let (head, spec) = split_head_spec(piece);

                if should_extract_expression(head) {
                    let expr: Expr = syn::parse_str(head).expect("format!: invalid expression");
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

                    out_lit.push('{');
                    out_lit.push_str(&idx.to_string());
                    if !spec.is_empty() {
                        out_lit.push(':');
                        out_lit.push_str(spec);
                    }
                    out_lit.push('}');
                } else {
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

// split `HEAD[:SPEC]`, ignoring `::` (path separators)
fn split_head_spec(s: &str) -> (&str, &str) {
    let mut chars = s.char_indices().peekable();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut angle_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    while let Some((idx, c)) = chars.next() {
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
                // Skip everything inside strings
                continue;
            }
            '(' => {
                paren_depth += 1;
            }
            ')' => {
                paren_depth -= 1;
            }
            '[' => {
                bracket_depth += 1;
            }
            ']' => {
                bracket_depth -= 1;
            }
            '<' => {
                // Only count as generic if it looks like one (letter/underscore before)
                if idx > 0 {
                    let prev_char = s.chars().nth(idx.saturating_sub(1));
                    if let Some(prev) = prev_char {
                        if prev.is_alphanumeric() || prev == '_' || prev == '>' {
                            angle_depth += 1;
                        }
                    }
                }
            }
            '>' => {
                if angle_depth > 0 {
                    angle_depth -= 1;
                }
            }
            ':' if paren_depth == 0 && bracket_depth == 0 && angle_depth == 0 => {
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

fn should_extract_expression(head: &str) -> bool {
    // Extract if it contains dots, method calls, array indexing, or complex expressions
    head.contains('.')
        || head.contains("::")
        || head.contains('(')
        || head.contains('[')
        || head.contains('<')
}
