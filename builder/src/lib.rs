use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syn::{Data, Fields};
extern crate proc_macro;
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut struct_name_builder;
    let original_struct_name = &input.ident;
    let namedfields: Vec<proc_macro2::TokenStream> = match input.data {
        Data::Struct(ref s) => {
            struct_name_builder = format_ident!("{}Builder", input.ident);
            match s.fields {
                Fields::Named(ref fieldsnamed) => fieldsnamed
                    .named
                    .iter()
                    .map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        quote! {
                            #name: std::option::Option<#ty>,
                        }
                    })
                    .collect(),
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    };
    let none_default: Vec<proc_macro2::TokenStream> = match input.data {
        Data::Struct(ref s) => {
            struct_name_builder = format_ident!("{}Builder", input.ident);
            match s.fields {
                Fields::Named(ref fieldsnamed) => fieldsnamed
                    .named
                    .iter()
                    .map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        quote! {
                            #name: None,
                        }
                    })
                    .collect(),
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    };
    let expanded = quote! {
        pub struct #struct_name_builder {
            #( #namedfields )*
        }

        impl #original_struct_name {
            pub fn builder() -> #struct_name_builder {
               #struct_name_builder {
                   #( #none_default)*
               }
            }
        }
    };
    eprintln!("TOKENS: {}", expanded);
    TokenStream::from(expanded)
}
