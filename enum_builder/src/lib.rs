use std::collections::{
    BTreeMap, 
    // BTreeSet
};
use quote::{quote, ToTokens};
use syn::{
    // fold::{self, Fold},
    parse::{
        // Error, 
        Parse, ParseStream, Result
    },
    parse_macro_input,
    // parse_quote,
    spanned::Spanned,
    // token::Comma,
    // visit::{self, Visit},
    // Attribute,
    // FnArg,
    GenericParam,
    // Generics,
    Ident,
    ItemTrait,
    // LitInt,
    // LitStr,
    // TraitBound,
    // TraitItem,
    // TraitItemFn,
    // TypeParamBound
};
use proc_macro2::{
    Delimiter, 
    Group, 
    // Span, 
    TokenStream, 
    Punct, 
    Spacing
};
// use itertools::Itertools;

/// The structure used for parsing the runtime api declarations.
struct RuntimeApiDecls {
    decls: Vec<ItemTrait>,
}

impl Parse for RuntimeApiDecls {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut decls = Vec::new();

        while !input.is_empty() {
            decls.push(ItemTrait::parse(input)?);
        }

        Ok(Self { decls })
    }
}

#[proc_macro]
pub fn enum_build(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // Parse all trait declarations
    let RuntimeApiDecls { decls: api_decls } = parse_macro_input!(input as RuntimeApiDecls);

    // eprintln!("decls: {:?}", api_decls);

    // let mut enum_generics = vec![];
    let mut enum_generics = BTreeMap::new();
    let mut enum_runtime_arg_fields = TokenStream::new();

    for decl in api_decls {
        for item in &decl.items {
            let syn::TraitItem::Fn(method) = item else { continue };

            let signature = &method.sig;
            let mut types = vec![];

            method.sig.generics.params.iter().for_each(|gp| {
               if let GenericParam::Type(type_param) = gp {
                   // enum_generics.push(type_param.ident.clone());
                   enum_generics.insert(type_param.ident.to_string(), type_param.ident.clone()); 
               }
            });

            // println!("signature: {:?}", signature);
            println!("sig");

            for input in &signature.inputs {
                // Exclude `self` from metadata collection.
                let syn::FnArg::Typed(typed) = input else { continue };

                let pat = &typed.pat;
                let name = quote!(#pat).to_string();
                let ty = &typed.ty;

                eprintln!("name: {:?} - ty: {:?}", name, ty);
                eprintln!("===");

                types.push(ty.clone());
            }

            let group_types_stream_it = types
                .iter()
                .map(|ty| ty.to_token_stream());
            let group_types_stream = itertools::Itertools::intersperse(group_types_stream_it, Punct::new(',', Spacing::Alone).to_token_stream())
                // .intersperse(Punct::new(',', Spacing::Alone).to_token_stream())
                .collect::<TokenStream>();
            let group_types = Group::new(Delimiter::Parenthesis, group_types_stream);

            // eprintln!("group_types: {}", group_types);

            let field_name = Ident::new(
                format!("{}Arg", signature.ident.to_string()).as_str(),
                signature.span().clone(),
            );

            enum_runtime_arg_fields.extend(field_name.to_token_stream());
            enum_runtime_arg_fields.extend(group_types.to_token_stream());
            enum_runtime_arg_fields.extend(Punct::new(',', Spacing::Alone).to_token_stream());
        }
    }

    // unimplemented!()

    let enum_generics_it = enum_generics.values();
    let enum_generics_expanded = quote! {
        #(#enum_generics_it),*
    };

    let expanded = quote! {
        enum FnArgs<Data: DataTrait, #enum_generics_expanded> {
            #enum_runtime_arg_fields
        }
    };

    proc_macro::TokenStream::from(expanded)
}