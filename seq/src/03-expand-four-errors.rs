
use proc_macro::{Group, TokenStream, TokenTree};
use proc_macro2::TokenTree::Literal;
use syn::{parse_macro_input, Ident, Token, LitInt, Error, braced};
use syn::parse::{Parse, ParseStream, Result};
use quote::{quote, ToTokens};
use syn::token::Token;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {

    let SeqInput {
        n: _n,
        start,
        end,
        brace_in
    } = parse_macro_input!(input as SeqInput);

    let start_v_ = start.base10_parse();
    let end_v_ = end.base10_parse();

    // This requires nightly features:
    // #![feature(proc_macro_diagnostic)]
    /*
    if start_v >= end_v {
        start.span()
            .unwrap()
            .error(format!("Require start value ({}) to be lower than end value ({})", start_v, end_v))
            .emit();
    }
    */

    if let Err(e) = start_v_ {
        return e.into_compile_error().into();
    }

    if let Err(e) = end_v_ {
        return e.into_compile_error().into();
    }

    let start_v: i32 = start_v_.unwrap();
    let end_v: i32 = end_v_.unwrap();

    if start_v >= end_v {
        return Error::new(start.span(),
                               format!("Require start value ({}) to be lower than end value ({})", start_v, end_v)
        )
            .into_compile_error() // proc_macro2::TokenStream
            .into(); // proc_macro::TokenStream
    }

    /*
    let expanded = quote! {
        #brace_in
    };
    */

    let mut expanded = quote! {
    };

    for i in start_v..end_v {

        let brace_in_token_stream = brace_in.clone();
        println!("brace_in_token_stream: {:?}", brace_in_token_stream);
        let new_brace_in_token_stream = expand_n(brace_in_token_stream, i);
        println!("new_brace_in_token_stream: {:?}", new_brace_in_token_stream);
        expanded.extend(quote! {
            #new_brace_in_token_stream
        });
    }

    TokenStream::from(expanded)
}

// parse N in 0..8 {}
struct SeqInput {
    n: Ident,
    start: LitInt,
    end: LitInt,
    brace_in: proc_macro2::TokenStream,
}

impl Parse for SeqInput {
    fn parse(input: ParseStream) -> Result<Self> {

        let n = input.parse::<Ident>()?;
        input.parse::<Token![in]>()?;
        let start = input.parse::<LitInt>()?;
        input.parse::<Token![..]>()?;
        let end = input.parse::<LitInt>()?;

        // https://docs.rs/syn/2.0.68/syn/macro.braced.html
        let content;
        let _res = braced!(content in input);
        let brace_in_ = content
            .parse_terminated(proc_macro2::TokenStream::parse, Token![;])?;
        // eprintln!("brace_in: --{:?}--", brace_in_);
        // Get TokenStream from brace_in_ (or empty TokenStream)
        let brace_in = brace_in_
            .first()
            .cloned()
            .unwrap_or_else(|| proc_macro2::TokenStream::new())
            ;

        Ok(SeqInput {
            n,
            start,
            end,
            brace_in
        })
    }
}

fn expand_n(ts: proc_macro2::TokenStream, n: i32) -> proc_macro2::TokenStream {
    ts.
        into_iter()
        .map(|token_tree| {
            match token_tree {
                proc_macro2::TokenTree::Group(group) => {
                    let expanded_ts = expand_n(group.stream(), n);
                    proc_macro2::TokenTree::Group(proc_macro2::Group::new(group.delimiter(), expanded_ts))
                }
                proc_macro2::TokenTree::Ident(ref id) => {
                    if id.to_string() == "N" {
                        // Transform Ident to Literal (replace N to a const value)
                        // https://docs.rs/proc-macro2/1.0.86/proc_macro2/struct.Literal.html
                        let mut literal = proc_macro2::Literal::i32_unsuffixed(n);
                        // literal.set_span(id.span().clone());
                        // literal.set_span(token_tree.span());
                        proc_macro2::TokenTree::Literal(literal)
                    } else {
                        token_tree.clone()
                    }
                }
                _ => token_tree.clone()
            }
        })
        .collect()
}