
use proc_macro::TokenStream;

use syn::{parse_macro_input, Ident, Token, LitInt, Error, braced};
use syn::parse::{Parse, ParseStream, Result};
use quote::{quote};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {

    let SeqInput {
        n: _n,
        start,
        end
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

    let start_v: u64 = start_v_.unwrap();
    let end_v: u64 = end_v_.unwrap();

    if start_v >= end_v {
        return Error::new(start.span(),
                               format!("Require start value ({}) to be lower than end value ({})", start_v, end_v)
        )
            .into_compile_error() // proc_macro2::TokenStream
            .into(); // proc_macro::TokenStream
    }

    let expanded = quote! {
    };

    TokenStream::from(expanded)
}

// parse N in 0..8 {}
struct SeqInput {
    n: Ident,
    start: LitInt,
    end: LitInt,
}

impl Parse for SeqInput {
    fn parse(input: ParseStream) -> Result<Self> {

        let _content;
        let n = input.parse::<Ident>()?;
        input.parse::<Token![in]>()?;
        let start = input.parse::<LitInt>()?;
        input.parse::<Token![..]>()?;
        let end = input.parse::<LitInt>()?;
        braced!(_content in input);

        Ok(SeqInput {
            n,
            start,
            end
        })
    }
}
