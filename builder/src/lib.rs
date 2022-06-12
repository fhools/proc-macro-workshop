use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use quote::TokenStreamExt;
use syn::Error;
use syn::{parenthesized, parse::Parse, parse::ParseStream, parse_macro_input, DeriveInput, Token};
use syn::{
    AngleBracketedGenericArguments, Data, Fields, GenericArgument, Ident, PathArguments, Type,
    TypePath,
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

fn vec_inner_ty(ty: &Type) -> &syn::Ident {
    if let Type::Path(TypePath { ref path, .. }) = ty {
        if path.segments.first().unwrap().ident == "Vec" {
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
    panic!("tried to get non-vec inner type");
}
#[derive(Debug, Default)]
struct BuilderAttr {
    method_name: String,
    is_same: bool,
}

impl Parse for BuilderAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        eprintln!("content: {}", content);
        let ident: Ident = content.parse()?;
        if ident != "each" {
            return syn::Result::Err(Error::new(ident.span(), "expected each"));
        }
        let equals: Token![=] = content.parse()?;
        let method_name: syn::LitStr = content.parse()?;
        eprintln!("builder ident = {}", ident);
        eprintln!("equals = {:?}", equals);
        eprintln!("litstr = {:?}", method_name);
        Ok(BuilderAttr {
            method_name: method_name.value(),
            is_same: false,
        })
    }
}


fn has_builder_same_name(name: &Option<Ident>, attrs: &Vec<syn::Attribute>) -> bool {
    if attrs.len() > 0
        && attrs[0].path.segments.first().unwrap().ident == "builder"
    {
        eprintln!(
            "===ATTRS=== path: {:?} tokens: {}",
            attrs[0].path, attrs[0].tokens
        );
        let attr_tokens = attrs[0].tokens.clone().into();
        let builder_attr : Result<BuilderAttr, syn::Error> = syn::parse(attr_tokens);
        if builder_attr.is_err() {
            return false;
        }
        let method_name = format_ident!("{}", builder_attr.unwrap().method_name);
        *name.as_ref().unwrap() == method_name
    } else {
        return false;
    }
}
#[proc_macro_derive(Builder, attributes(builder, footest))]
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
                        let attrs = &f.attrs;
                        let mut builder_attr: BuilderAttr = Default::default();
                        let mut my_compile_err_q = Default::default();
                        if attrs.len() > 0
                            && attrs[0].path.segments.first().unwrap().ident == "builder"
                        {
                            eprintln!(
                                "===ATTRS=== path: {:?} tokens: {}",
                                attrs[0].path, attrs[0].tokens
                            );
                            let attr_tokens = attrs[0].tokens.clone().into();
                            let builder_attr_result = syn::parse(attr_tokens);
                            if builder_attr_result.is_err() {
                                   eprintln!("got an error with builder attr");
                                   match attrs[0].parse_meta() {
                                       Ok(syn::Meta::List(mut nvs)) => {
                                            let my_compile_error = syn::Error::new_spanned(nvs, "expected `builder(each = \"...\")`").to_compile_error();
                                            my_compile_err_q = quote! {
                                                #my_compile_error 
                                            };
                                       },
                                       _ => {}
                                   }
                            } else {
                                builder_attr = builder_attr_result.unwrap();
                            }
                            eprintln!("attribute ident: {:?}", builder_attr.method_name);
                            eprintln!(
                                "===ATTRS=== path: {:?} tokens: {}",
                                attrs[0].path, attrs[0].tokens
                            );
                        }
                        let mut all_q: proc_macro2::TokenStream;

                        // Generate the one-at-a-time build method
                        let one_at_a_time_method = if builder_attr.method_name.len() > 0 {
                            let method_name = format_ident!("{}", builder_attr.method_name);
                            if *name.as_ref().unwrap() == method_name {
                                builder_attr.is_same = true;
                            }
                            let method_type = vec_inner_ty(ty);
                            quote! {
                                pub fn #method_name(&mut self, #name: #method_type) -> #struct_name_builder {
                                    let mut old_vec =  self.#name.take().unwrap_or_default();
                                    old_vec.push(#name);
                                    self.#name = Some(old_vec);
                                    self.clone()
                                }
                            }
                        } else {
                            quote! {
                            }
                        };
                       
                        // Generate the build method, if the argument is an Option take the inner
                        // type, not an Option type
                        let all_at_once_method = if ! builder_attr.is_same {
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
                        } else {
                            quote! {}
                        };

                        quote! {
                            #my_compile_err_q
                            #one_at_a_time_method
                            #all_at_once_method
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
                    if has_builder_same_name(name, &f.attrs) {
                        quote! {
                        }
                    } else {
                        quote! {
                            if let None = self.#name {
                                return Err(String::from(stringify!(missing #name)).into());
                            }
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
                            #name : self.#name.unwrap_or_default(),
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
