use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syn::{
    AngleBracketedGenericArguments, Data, Fields, GenericArgument, PathArguments, Type, TypePath,
};
extern crate proc_macro;
fn is_optional_ty(ty: &Type) -> bool {
    if let Type::Path(TypePath { ref path, .. }) = ty {
        if path.segments.first().unwrap().ident == "Option" {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                ref args, ..
            }) = path.segments.first().unwrap().arguments
            {
                if let GenericArgument::Type(Type::Path(TypePath { ref path, .. })) =
                    args.first().unwrap()
                {
                    return true;
                }
            }
        }
    }
    return false;
}
fn option_inner_ty(ty: &Type) -> &syn::Ident {
    if let Type::Path(TypePath { ref path, .. }) = ty {
        if path.segments.first().unwrap().ident == "Option" {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                ref args, ..
            }) = path.segments.first().unwrap().arguments
            {
                if let GenericArgument::Type(Type::Path(TypePath { ref path, .. })) =
                    args.first().unwrap()
                {
                    let inner_ty = &path.segments.first().unwrap().ident;
                    return inner_ty;
                }
            }
        }
    }
    panic!("tried to get non-optional inner type");
}

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
                        let mut field_decl = Default::default();
                        if let Type::Path(TypePath { ref path, .. }) = ty {
                            if path.segments.first().unwrap().ident == "Option" {
                                if let PathArguments::AngleBracketed(
                                    AngleBracketedGenericArguments { ref args, .. },
                                ) = path.segments.first().unwrap().arguments
                                {
                                    if let GenericArgument::Type(Type::Path(TypePath {
                                        ref path,
                                        ..
                                    })) = args.first().unwrap()
                                    {
                                        let inner_ty = &path.segments.first().unwrap().ident;
                                        field_decl = quote! {
                                            pub #name: std::option::Option<#inner_ty>,
                                        };
                                    }
                                }
                            } else {
                                field_decl = quote! {
                                    pub #name: std::option::Option<#ty>,
                                };
                            }
                        }
                        field_decl
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

    let setter_methods: Vec<proc_macro2::TokenStream> = match input.data {
        Data::Struct(ref s) => {
            struct_name_builder = format_ident!("{}Builder", input.ident);
            match s.fields {
                Fields::Named(ref fieldsnamed) => fieldsnamed
                    .named
                    .iter()
                    .map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        if is_optional_ty(ty) {
                            let inner_ty = option_inner_ty(ty);
                            quote! {
                                pub fn #name(&mut self, #name: #inner_ty) -> #struct_name_builder {
                                    self.#name = Some(#name);
                                    self.clone()
                                }
                            }
                        } else {
                            quote! {
                                pub fn #name(&mut self, #name: #ty) -> #struct_name_builder {
                                    self.#name = Some(#name);
                                    self.clone()
                                }
                            }
                        }
                    })
                    .collect(),
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    };

    let check_has_values: Vec<proc_macro2::TokenStream> = match input.data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref fieldsnamed) => fieldsnamed
                .named
                .iter()
                .filter(|f| match f.ty {
                    Type::Path(TypePath { ref path, .. }) => {
                        !(path.segments.first().unwrap().ident == "Option")
                    }
                    _ => true,
                })
                .map(|f| {
                    let name = &f.ident;
                    quote! {
                        if let None = self.#name {
                            return Err(String::from(stringify!(missing #name)).into());
                        }
                    }
                })
                .collect(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let fields_init: Vec<proc_macro2::TokenStream> = match input.data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref fieldsnamed) => fieldsnamed
                .named
                .iter()
                .map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    if is_optional_ty(ty) {
                        quote! {
                            #name : self.#name,
                        }
                    } else {
                        quote! {
                            #name : self.#name.unwrap(),
                        }
                    }
                })
                .collect(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let expanded = quote! {
        #[derive(Clone)]
        pub struct #struct_name_builder {
            #( #namedfields )*
        }

        impl #struct_name_builder {
            #( #setter_methods )*

            pub fn build(self) -> Result<#original_struct_name, Box<dyn std::error::Error>> {
                #( #check_has_values )*
                Ok( #original_struct_name {
                    #( #fields_init )*
                })

            }
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
