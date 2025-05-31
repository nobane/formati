use proc_macro::TokenStream;
use proc_macro2::{Spacing, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};
use syn::{parse2, Expr, ExprAssign, ExprLit, Lit, LitStr};

use crate::formati_args::formati_args;

/// Split on *top-level* commas — nothing else
fn split_top_level(stream: TokenStream2) -> Vec<TokenStream2> {
    let mut segs = Vec::<TokenStream2>::new();
    let mut cur = TokenStream2::new();
    for tt in stream {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == ',' && p.spacing() == Spacing::Alone => {
                segs.push(cur);
                cur = TokenStream2::new();
            }
            _ => cur.extend(std::iter::once(tt)),
        }
    }
    segs.push(cur);
    segs
}

/// Find format string and process tracing-like macros
pub fn wrap(kind: &str, input: proc_macro::TokenStream) -> TokenStream {
    let segments = split_top_level(TokenStream2::from(input));

    // find the *last* string-literal segment — that starts the template
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
        .expect("tracing macro needs a string literal message");

    let (front, back) = segments.split_at(split_at);
    let fmt_seg = &back[0]; // the literal
    let rest = &back[1..]; // possible extra exprs

    // pull out the literal text & span
    let lit_expr: Expr = parse2(fmt_seg.clone()).unwrap();
    let lit_str: LitStr = match lit_expr {
        Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) => s,
        _ => unreachable!(),
    };

    let (fmt, expr) = formati_args(&lit_str);
    let fmt_str = LitStr::new(&fmt, lit_str.span());

    // extra args (after the literal): named first, then positional
    let mut named = Vec::<TokenStream2>::new();
    let mut positional = Vec::<TokenStream2>::new();
    for seg in rest {
        // Skip empty segments (from trailing commas)
        if seg.is_empty() {
            continue;
        }

        let expr: Expr = parse2(seg.clone()).expect("invalid expression after template");
        match expr {
            Expr::Assign(ExprAssign { .. }) => named.push(expr.to_token_stream()),
            _ => positional.push(expr.to_token_stream()),
        }
    }

    // emit the real tracing macro call
    let tracing_macro = syn::Ident::new(kind, proc_macro2::Span::call_site());
    let front: Vec<&TokenStream2> = front.iter().collect();

    quote! {
        ::tracing::#tracing_macro!(
            #(#front ,)*
            #fmt_str
            #(, #named)*
            #(, #expr)*
            #(, #positional)*
        )
    }
    .into()
}
