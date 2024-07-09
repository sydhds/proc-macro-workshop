// #![feature(proc_macro_diagnostic)]

use proc_macro::{TokenStream, TokenTree};
use proc_macro2::Delimiter;
// use proc_macro2::TokenTree::Literal;
use syn::{parse_macro_input, Ident, Token, LitInt, Error, braced};
use syn::parse::{Parse, ParseStream, Result};
use quote::{quote, ToTokens};
// use syn::token::Token;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {

    // let ts = parse_macro_input!(input as TokenTree);
    // panic!();
    println!("input: {:?}", input);

    let SeqInput {
        n: _n,
        start,
        end,
        brace_in
    } = parse_macro_input!(input as SeqInput);

    let start_v_ = start.base10_parse();
    let end_v_ = end.base10_parse();

    if let Err(e) = start_v_ {
        return e.into_compile_error().into();
    }

    if let Err(e) = end_v_ {
        return e.into_compile_error().into();
    }

    let start_v: i32 = start_v_.unwrap();
    let end_v: i32 = end_v_.unwrap();

    /*
    // This requires nightly features: #![feature(proc_macro_diagnostic)]
    if start_v >= end_v {
        start.span()
            .unwrap()
            .warning(format!("Require start value ({}) to be lower than end value ({})", start_v, end_v))
            .emit();
    }
    */

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

    if detect_partial_repeat(brace_in.clone()) {
        let brace_in_token_stream = brace_in.clone();
        // println!("brace_in_token_stream: {:?}", brace_in_token_stream);
        let new_brace_in_token_stream = expand_n_v3(brace_in_token_stream, start_v, end_v, 0);
        // println!("===");
        // println!("new_brace_in_token_stream: {:?}", new_brace_in_token_stream);
        expanded.extend(quote! {
            #new_brace_in_token_stream
        });
    } else {
        for i in start_v..end_v {
            let brace_in_token_stream = brace_in.clone();
            // println!("brace_in_token_stream: {:?}", brace_in_token_stream);
            let new_brace_in_token_stream = expand_n_v3(brace_in_token_stream, start_v, end_v, i);
            // println!("===");
            // println!("new_brace_in_token_stream: {:?}", new_brace_in_token_stream);
            expanded.extend(quote! {
            #new_brace_in_token_stream
        });
        }
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

#[allow(dead_code)]
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
                        let literal = proc_macro2::Literal::i32_unsuffixed(n);
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

#[allow(dead_code)]
fn expand_n_v2(ts: proc_macro2::TokenStream, n: i32) -> proc_macro2::TokenStream {

    // Expand a TokenStream - replace [Punct("~") + Ident ("N")] with Literal (n)

    let mut res = vec![];
    let mut it = ts.into_iter().peekable();
    while let Some(token_tree) = it.next() {
        match token_tree {
            proc_macro2::TokenTree::Group(group) => {
                let expanded_ts = expand_n_v2(group.stream(), n);
                let mut group = proc_macro2::Group::new(group.delimiter(), expanded_ts);
                group.set_span(group.span().clone());
                res.push(
                    proc_macro2::TokenTree::Group(group)
                );
            },
            proc_macro2::TokenTree::Ident(ref id) => {

                if id.to_string().as_str() == "N" {
                    
                    // let id = Ident::new(format!("{}", n).as_str(), id.span().clone());
                    // res.push(proc_macro2::TokenTree::Ident(id));
                    let lit_n = proc_macro2::Literal::i32_unsuffixed(n);
                    res.push(proc_macro2::TokenTree::Literal(lit_n));
                }
                else if let Some(proc_macro2::TokenTree::Punct(next_punct)) = it.peek() {
                    
                    if next_punct.to_string() == "~" {

                        // We have a Ident("...") followed by a Punct("~")
                        let next_punct = it.next();
                        // Fetch the next ident as we expect to have: Ident("...") + Punct("~") + Ident("N")
                        let next_next_ident_ = it.next();

                        if let Some(proc_macro2::TokenTree::Ident(next_next_ident)) = next_next_ident_ {
                            if next_next_ident.to_string() == "N" {
                                let id = Ident::new(format!("{}{}", id.to_string(), n).as_str(), id.span().clone());
                                res.push(proc_macro2::TokenTree::Ident(id))
                            }
                        } else {
                            // We have: Ident("XXX") + Punct("~") + TokenTree (!= Ident("N")
                            // This is likely an error from the user
                            // Push the token tree unmodified
                            res.push(token_tree.clone());
                            res.push(next_punct.unwrap());
                            if next_next_ident_.is_some() {
                                res.push(next_next_ident_.unwrap());
                            }
                        }
                    } else {
                        // Ident is followed by a Punct (but this is not a '~')
                        res.push(token_tree);
                    }
                } else {
                    // default case
                    res.push(token_tree)
                }
            },
            _ => {
                res.push(token_tree);
            }
        }
    }

    res
        .into_iter()
        .collect()
}

fn expand_n_v3(ts: proc_macro2::TokenStream, start: i32, end: i32, n: i32) -> proc_macro2::TokenStream {

    // ...

    let mut res = vec![];
    let mut it = ts.into_iter().peekable();
    while let Some(token_tree) = it.next() {
        match token_tree {
            proc_macro2::TokenTree::Group(group) => {
                let expanded_ts = expand_n_v3(group.stream(), start, end, n);
                let mut group = proc_macro2::Group::new(group.delimiter(), expanded_ts);
                group.set_span(group.span().clone());
                res.push(
                    proc_macro2::TokenTree::Group(group)
                );
            },
            proc_macro2::TokenTree::Ident(ref id) => {

                if id.to_string().as_str() == "N" {

                    // let id = Ident::new(format!("{}", n).as_str(), id.span().clone());
                    // res.push(proc_macro2::TokenTree::Ident(id));
                    let lit_n = proc_macro2::Literal::i32_unsuffixed(n);
                    res.push(proc_macro2::TokenTree::Literal(lit_n));
                }
                else if let Some(proc_macro2::TokenTree::Punct(next_punct)) = it.peek() {

                    if next_punct.to_string() == "~" {

                        // We have a Ident("...") followed by a Punct("~")
                        let next_punct = it.next();
                        // Fetch the next ident as we expect to have: Ident("...") + Punct("~") + Ident("N")
                        let next_next_ident_ = it.next();

                        if let Some(proc_macro2::TokenTree::Ident(next_next_ident)) = next_next_ident_ {
                            if next_next_ident.to_string() == "N" {
                                let id = Ident::new(format!("{}{}", id.to_string(), n).as_str(), id.span().clone());
                                res.push(proc_macro2::TokenTree::Ident(id))
                            }
                        } else {
                            // We have: Ident("XXX") + Punct("~") + TokenTree (!= Ident("N")
                            // This is likely an error from the user
                            // Push the token tree unmodified
                            res.push(token_tree.clone());
                            res.push(next_punct.unwrap());
                            if next_next_ident_.is_some() {
                                res.push(next_next_ident_.unwrap());
                            }
                        }
                    } else {
                        // Ident is followed by a Punct (but this is not a '~')
                        res.push(token_tree);
                    }
                } else {
                    // default case
                    res.push(token_tree)
                }
            },
            proc_macro2::TokenTree::Punct(ref punct) => {

                let mut to_push = true;

                // Punct('#') - Group(delimiter=parenthesis, stream)
                if punct.as_char() == '#' {
                    // Found a '#'
                    let next_ = it.peek();
                    if let Some(proc_macro2::TokenTree::Group(group_)) = next_ {
                        if group_.delimiter() == Delimiter::Parenthesis {
                            // Found a group with delimiter ()
                            let group = it.next().unwrap(); // already peek and checked
                            let after_group = it.next();
                            if let Some(proc_macro2::TokenTree::Punct(ref after_group_punct)) = after_group {
                                if after_group_punct.as_char() == '*' {
                                    // Found #(...)*
                                    if let proc_macro2::TokenTree::Group(group__) = group {
                                        for i in start..end {
                                            let group_stream = group__.stream().clone();
                                            let new_group_stream = expand_n_v3(group_stream, start, end, i);
                                            res.extend(new_group_stream.into_iter());
                                        }
                                        to_push = false;
                                    } else {
                                        unreachable!()
                                    }
                                    
                                    
                                } else {
                                    res.push(group);
                                    res.push(after_group.unwrap());
                                    to_push = false;
                                }
                            } else {
                                res.push(group);
                                if after_group.is_some() {
                                    res.push(after_group.unwrap());
                                }
                                to_push = false;
                            }
                        }
                    }
                }

                if to_push {
                    res.push(token_tree);
                }
            }
            _ => {
                res.push(token_tree);
            }
        }
    }

    res
        .into_iter()
        .collect()
}

fn detect_partial_repeat(ts: proc_macro2::TokenStream) -> bool {
    
    let mut it = ts.into_iter().peekable();
    while let Some(token_tree) = it.next() {
        match token_tree {
            proc_macro2::TokenTree::Group(group) => {
                let has_partial_repeat = detect_partial_repeat(group.stream());
                if has_partial_repeat {
                    return true;
                }
            },
            proc_macro2::TokenTree::Punct(ref punct) => {
                // Punct('#') - Group(delimiter=parenthesis, stream)
                if punct.as_char() == '#' {
                    // Found a '#'
                    let next_ = it.peek();
                    if let Some(proc_macro2::TokenTree::Group(group_)) = next_ {
                        if group_.delimiter() == Delimiter::Parenthesis {
                            // Found a group with delimiter ()
                            let group = it.next().unwrap(); // already peek and checked
                            let after_group = it.next();
                            if let Some(proc_macro2::TokenTree::Punct(ref after_group_punct)) = after_group {
                                if after_group_punct.as_char() == '*' {
                                    // Found #(...)*
                                    return true;
                                } 
                            }
                        }
                    }
                }
            }
            _ => {
            }
        }
    }
    
    false
}