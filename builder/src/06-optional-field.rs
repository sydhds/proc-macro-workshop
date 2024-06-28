
// use proc_macro2::{Ident, TokenStream};
use quote::{format_ident,
            quote,
            };
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Type, PathArguments, GenericArgument};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    // https://docs.rs/syn/latest/syn/struct.DeriveInput.html

    // let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let input = parse_macro_input!(input as DeriveInput);

    println!("input ident: {:?}", input.ident); // A word of Rust code, which may be a keyword or legal variable name
    let name = input.ident;

    // https://docs.rs/quote/latest/quote/macro.format_ident.html
    let builder_name = format_ident!("{}Builder", name);

    // Inspired from:
    // https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs

    let fields = get_fields_with_ident(&input.data);
    // println!("fields: {:?}", fields);
    let fields_init_it = fields.iter().map(|f| {
        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);
            quote! {
                #id_: None,
            }
        } else { quote! {} }
    });
    let fields_init = quote! {
        #(#fields_init_it)*
    };
    // println!("fields_init: {:?}", fields_init);

    let field_types_it = fields.iter().map(|f| {
        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);
            let ty = &f.ty;
            let is_an_option = is_field_an_option(f);

            if is_an_option {
                quote! {
                    #id_: #ty,
                }
            } else {
                quote! {
                    #id_: Option<#ty>,
                }
            }
        } else { quote! {} }
    });
    let field_types = quote! {
        #(#field_types_it)*
    };
    // println!("field_types: {:?}", field_types);

    let command_builder_setters_it = fields.iter().map(|f| {
        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);
            let ty = &f.ty;
            let is_an_option = is_field_an_option_ty(f);

            if let Some(option_inner_ty) = is_an_option {
               quote! {
                    fn #id(&mut self, #id: #option_inner_ty) -> &mut Self {
                        self.#id_ = Some(#id);
                        self
                    }
               }
            } else {
                quote! {
                    fn #id(&mut self, #id: #ty) -> &mut Self {
                        self.#id_ = Some(#id);
                        self
                    }
                }
            }
        } else { quote! {} }
    });
    let command_builder_setters = quote! {
        #(#command_builder_setters_it)*
    };
    // println!("command builder setters: {:?}", command_builder_setters);

    let command_builder_build_code_it = fields.iter().map(|f| {
        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);
            let is_an_option = is_field_an_option(f);

            if is_an_option {
                quote! {
                    #id: self.#id_.take(),
                }
            } else {
                quote! {
                    #id: self.#id_.take().ok_or(
                        format!("Field {} is not set", stringify!(#id))
                    )?,
                }
            }
        } else {
            quote! {}
        }
    });

    let command_builder_build_code = quote! {
        #(#command_builder_build_code_it)*
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {

        use std::error::Error;

        impl #name {
            pub fn builder() -> #builder_name {
                CommandBuilder {
                    #fields_init
                }
            }
        }

        pub struct #builder_name {
            #field_types
        }

        impl CommandBuilder {

            #command_builder_setters

            pub fn build(&mut self) -> Result<#name, Box<dyn Error>> {
                let st = #name {
                    #command_builder_build_code
                };

                Ok(st)
            }
        }

    };

    // Hand the output tokens back to the compiler
    // TokenStream::from(expanded)
    proc_macro::TokenStream::from(expanded)
}

fn get_fields_with_ident(data: &Data) -> Vec<Field> {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    fields.named.iter().filter_map(|f| {
                        if let Some(_name) = &f.ident {
                            Some(f.clone())
                        } else {
                            None
                        }
                    }).collect()
                },
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

fn is_field_an_option(field: &Field) -> bool {

    // simple version - return bool
    // From 06-optional-field.rs, parse:
    //     Type::Path(
    //         TypePath {
    //             qself: None,
    //             path: Path {
    //                 segments: [
    //                     PathSegment {
    //                         ident: "Option",
    //                         arguments: PathArguments::AngleBracketed(
    //                             AngleBracketedGenericArguments {
    //                                 args: [
    //                                     GenericArgument::Type(
    //                                         ...
    //                                     ),

    if let Type::Path(type_path) = &field.ty {
       let segments = type_path.path.segments.first();
        if let Some(path_segment) = segments {
            if path_segment.ident.to_string() == "Option" {
                return true;
            }
        }
    }

    false
}

fn is_field_an_option_ty(field: &Field) -> Option<Type> {

    // return inner type (e.g. Option<String> -> String)
    // From 06-optional-field.rs, parse:
    //     Type::Path(
    //         TypePath {
    //             qself: None,
    //             path: Path {
    //                 segments: [
    //                     PathSegment {
    //                         ident: "Option",
    //                         arguments: PathArguments::AngleBracketed(
    //                             AngleBracketedGenericArguments {
    //                                 args: [
    //                                     GenericArgument::Type(
    //                                         ...
    //                                     ),

    if let Type::Path(type_path) = &field.ty {
        let segments = type_path.path.segments.first();
        if let Some(path_segment) = segments {
            if path_segment.ident.to_string() == "Option" {
                if let PathArguments::AngleBracketed(generic_args) = &path_segment.arguments {
                    // TODO: error generic_args.args len != 1?
                    if let GenericArgument::Type(ty) = &generic_args.args[0] {
                        return Some(ty.clone());
                    }
                }
            }
        }
    }

    None
}
