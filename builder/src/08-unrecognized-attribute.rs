use proc_macro2::{Ident, Literal, Span};
// use proc_macro2::{Ident, TokenStream};
use quote::{format_ident,
            quote,
            };
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Type, PathArguments, GenericArgument, Meta, Token, Error, Attribute};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

const CUSTOM_ATTR_BUILDER_NOT_FOUND: &str = "Could not find any custom attribute named builder";

#[proc_macro_derive(Builder, attributes(builder))]
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

    let struct_fields = get_fields_with_ident(&input.data);

    // Check for valid builder attribute - output a compile_error! before expanding the macro
    for f in struct_fields.iter() {
        if has_field_builder_attr(f) {
            if let Err(e) = builder_attribute_ident(f) {
                return e.to_compile_error().into();
            }
        }
    }

    // println!("struct fields: {:?}", struct fields);
    let command_builder_builder_it = struct_fields.iter().map(|f| {

        let has_field_builder_attr = has_field_builder_attr(f);

        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);

            if has_field_builder_attr {
                quote! {
                    #id_: vec![],
                }
            } else {
                quote! {
                    #id_: None,
                }
            }
        } else { quote! {} }
    });
    let command_builder_builder = quote! {
        #(#command_builder_builder_it)*
    };
    // println!("command builder builder(): {:?}", command_buidler_builder);

    let command_builder_struct_types_it = struct_fields.iter().map(|f| {
        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);
            let ty = &f.ty;
            let is_an_option = is_field_an_option(f);
            let has_field_builder_attr = has_field_builder_attr(f);

            if is_an_option || has_field_builder_attr {
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
    let command_builder_struct_types = quote! {
        #(#command_builder_struct_types_it)*
    };
    // println!("field_types: {:?}", field_types);

    let command_builder_setters_it = struct_fields.iter().map(|f| {
        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);
            let ty = &f.ty;
            let is_an_option = is_field_an_option_ty(f);
            let has_field_builder_attr = has_field_builder_attr(f);
            let builder_attr_ident = builder_attribute_ident(f);

            if let Some(option_inner_ty) = is_an_option {
               quote! {
                    fn #id(&mut self, #id: #option_inner_ty) -> &mut Self {
                        self.#id_ = Some(#id);
                        self
                    }
               }
            } else {

                if has_field_builder_attr {

                    // safe to unwrap() as it has already been checked
                    let builder_attr_id = builder_attr_ident.unwrap();

                    let inner_ty = get_vec_inner_ty(&ty);

                    quote! {
                        fn #builder_attr_id(&mut self, #builder_attr_id: #inner_ty) -> &mut Self {
                            self.#id_.push(#builder_attr_id);
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
            }
        } else { quote! {} }
    });
    let command_builder_setters = quote! {
        #(#command_builder_setters_it)*
    };
    // println!("command builder setters: {:?}", command_builder_setters);

    let command_builder_build_code_it = struct_fields.iter().map(|f| {
        if let Some(id) = &f.ident {
            let id_ = format_ident!("_{}", id);
            let is_an_option = is_field_an_option(f);
            // let has_builder_attr = has_field_builder_attr(f);
            let has_builder_attr = has_field_builder_attr(f);

            if has_builder_attr {
                quote! {
                    #id: std::mem::replace(&mut self.#id_, vec![]),
                }
            } else {
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
                    #command_builder_builder
                }
            }
        }

        pub struct #builder_name {
            #command_builder_struct_types
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

    // get all fields (as syn::Field) from struct

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

// From: https://blog.turbo.fish/proc-macro-parsing/
// Struct to parse the custom attribute of our proc macro: #[builder(each = "env")]
#[derive(Clone, Debug)]
struct BuilderMeta {
    #[allow(dead_code)]
    id: Ident,
    value: Literal,
}

impl Parse for BuilderMeta {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let arg_each: Ident = input.parse()?;

        if arg_each != "each" {
            return Err(syn::Error::new_spanned(
                arg_each,
                "expected `builder(each = \"...\")`"
            ));
        }

        // Parse (and discard the span of) the `=` token
        let _: Token![=] = input.parse()?;

        // Parse the argument value
        let each_value: Literal = input.parse()?;

        Ok(Self {
           id: arg_each,
           value: each_value
        })
    }
}

fn has_field_builder_attr(f: &Field) -> bool {
    let builder_attributes: Vec<&Attribute> = f.attrs.iter().filter(|a| {
        if let Meta::List(ml) = &a.meta {
            let path_segment_ = ml.path.segments.first();
            if let Some(path_segment) = path_segment_ {
                if path_segment.ident.to_string() == "builder" {
                    return true;
                }
            }
        }
        false
    }).collect();

    builder_attributes.len() == 1
}

fn builder_attribute_ident(f: &Field) -> syn::Result<Ident> {

    let builder_attributes: Vec<&Attribute> = f.attrs.iter().filter(|a| {
        if let Meta::List(ml) = &a.meta {
            let path_segment_ = ml.path.segments.first();
            if let Some(path_segment) = path_segment_ {
                if path_segment.ident.to_string() == "builder" {
                    return true;
                }
            }
        }
        false
    }).collect();

    if !builder_attributes.is_empty() {
        let builder_attribute = builder_attributes[0];

        if let Meta::List(ml) = &builder_attribute.meta {

            // Need to parse the token stream from the meta list
            let each_args: BuilderMeta = ml.parse_args()?;
            // println!("each_args: {:?}", each_args);

            let lit_str = each_args.value.to_string();
            // https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html
            // remove "" from literal (first and last character)
            let lit_str_stripped = lit_str
                .chars()
                .skip(1)
                .take(lit_str.len() - 2)
                .collect::<String>();

            let id = Ident::new(lit_str_stripped.as_str(), Span::call_site());

            return Ok(id)
        }

    }

    return Err(Error::new(f.span(), CUSTOM_ATTR_BUILDER_NOT_FOUND));
}

/*
fn has_field_builder_attr(f: &Field) -> syn::Result<Ident> {

    let builder_attributes: Vec<&Attribute> = f.attrs.iter().filter(|a| {
        if let Meta::List(ml) = &a.meta {
            let path_segment_ = ml.path.segments.first();
            if let Some(path_segment) = path_segment_ {
                if path_segment.ident.to_string() == "builder" {
                    return true;
                }
            }
        }
        false
    }).collect();

    if !builder_attributes.is_empty() {
        let builder_attribute = builder_attributes[0];

        if let Meta::List(ml) = &builder_attribute.meta {

            // Need to parse the token stream from the meta list
            let each_args: BuilderMeta = ml.parse_args()?;
            // println!("each_args: {:?}", each_args);

            let lit_str = each_args.value.to_string();
            // https://docs.rs/proc-macro2/latest/proc_macro2/struct.Literal.html
            // remove "" from literal (first and last character)
            let lit_str_stripped = lit_str
                .chars()
                .skip(1)
                .take(lit_str.len() - 2)
                .collect::<String>();

            let id = Ident::new(lit_str_stripped.as_str(), Span::call_site());

            return Ok(id)
        }

    }

    return Err(Error::new(f.span(), CUSTOM_ATTR_BUILDER_NOT_FOUND));
}
*/

fn get_vec_inner_ty(ty: &Type) -> Option<Type> {

    // From Vec<String> -> String
    // Parse:
    // TypePath { qself: None,
    //  path: Path { leading_colon: None,
    //      segments: [
    //          PathSegment { ident: Ident { ident: "Vec", span: #0 bytes(831..834) },
    //              arguments:
    //              PathArguments::AngleBracketed { colon2_token: None, lt_token: Lt,
    //                  args: [
    //                      GenericArgument::Type(Type::Path { qself: None, path:
    //                          Path { leading_colon: None, segments:
    //                              [PathSegment { ident:
    //                                  Ident { ident: "String", span: #0 bytes(835..841) },
    //                                  arguments: PathArguments::None }] } })], gt_token: Gt } }] } }

    match ty {
        Type::Path(type_path) => {
            let path_segment_ = type_path.path.segments.first();
            if let Some(path_segment) = path_segment_ {
                if path_segment.ident.to_string() == "Vec" {
                    if let PathArguments::AngleBracketed(vec_generic_args) = &path_segment.arguments {
                        let vec_arg = &vec_generic_args.args[0];
                        if let GenericArgument::Type(vec_arg_type) = vec_arg {
                            return Some(vec_arg_type.clone());
                        }
                    }
                }
            }
        },
        _ => unimplemented!()
    }

    None
}
