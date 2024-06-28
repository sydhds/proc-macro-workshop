// use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident,
            quote,
            //quote_spanned
};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field};

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
    // let fields_init: TokenStream = fields_init(&input.data).into();
    // // println!("fields_initializer: {}", fields_initializer);
    // let field_types = field_types(&input.data).into();
    // // let command_builder_setters = fields_setter(&input.data).into();
    // let fields_build: TokenStream = fields_build(&input.data).into();

    let fields = get_fields_with_ident(&input.data);
    println!("fields: {:?}", fields);
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
            quote! {
                #id_: Option<#ty>,
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
            quote! {
                fn #id(&mut self, #id: #ty) -> &mut Self {
                    self.#id_ = Some(#id);
                    self
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
            let ty = &f.ty;

            quote! {
                #id: self.#id_.take().ok_or(
                    format!("Field {} is not set", stringify!(#id))
                )?,
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

/*
fn fields_init(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let all_fields = fields.named.iter().map(|f| {
                        if let Some(name) = &f.ident {}
                    });
                    quote! {
                        #(#all_fields)*
                    }
                },
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}
*/

fn get_fields_with_ident(data: &Data) -> Vec<Field> {

    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    fields.named.iter().filter_map(|f| {
                        if let Some(name) = &f.ident {
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

fn fields_build(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let all_fields = fields.named.iter().map(|f| {

                        if let Some(name) = &f.ident {
                            let bname = format_ident!("_{}", name);
                            quote! {
                                #name: self.#bname.take().ok_or(
                                    format!("Field {} is not set", stringify!(#name))
                                )?,
                            }
                        } else {
                            quote! {}
                        }
                    });
                    quote! {
                        #(#all_fields)*
                    }
                }
                Fields::Unnamed(_) => unimplemented!(),
                Fields::Unit => unimplemented!(),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

/*
// #[proc_macro_derive(Builder)]
// 03-call-setters
fn derive_3(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    // https://docs.rs/syn/latest/syn/struct.DeriveInput.html
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    println!("input ident: {:?}", input.ident); // A word of Rust code, which may be a keyword or legal variable name
    let name = input.ident;

    // https://docs.rs/quote/latest/quote/macro.format_ident.html
    let builder_name = format_ident!("{}Builder", name);

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #builder_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl CommandBuilder {
            fn executable(&mut self, executable: String) -> &mut Self {
               self.executable = Some(executable);
               self
            }

            fn args(&mut self, args: Vec<String>) -> &mut Self {
               self.args = Some(args);
               self
            }

            fn env(&mut self, env: Vec<String>) -> &mut Self {
               self.env = Some(env);
               self
            }

            fn current_dir(&mut self, current_dir: String) -> &mut Self {
               self.current_dir = Some(current_dir);
               self
            }
        }

    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

// #[proc_macro_derive(Builder)]
// 02-create-builder
fn derive_2(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    // https://docs.rs/syn/latest/syn/struct.DeriveInput.html
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    println!("input ident: {:?}", input.ident); // A word of Rust code, which may be a keyword or legal variable name
    let name = input.ident;

    // https://docs.rs/quote/latest/quote/macro.format_ident.html
    let builder_name = format_ident!("{}Builder", name);

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #builder_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }


    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

// #[proc_macro_derive(Builder)]
// 01-parse
fn derive_1(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    // https://docs.rs/syn/latest/syn/struct.DeriveInput.html
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    println!("input attrs: {:?}", input.attrs); // An attribute, like #[repr(transparent)]
    println!("input vis: {:?}", input.vis); // The visibility level of an item: inherited or pub or pub(restricted)
    println!("input ident: {:?}", input.ident); // A word of Rust code, which may be a keyword or legal variable name
    println!("input generics: {:?}", input.generics); // Lifetimes and type parameters attached to a declaration of a function, enum, trait, etc.
    println!("input data: {:?}", input.data); // The storage of a struct, enum or union data structure.

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // ...
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

// #[proc_macro_derive(Builder)]
fn derive_0(input: TokenStream) -> TokenStream {
    let _ = input;
    unimplemented!()
}
*/
