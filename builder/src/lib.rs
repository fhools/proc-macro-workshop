use proc_macro;
use proc_macro2::{TokenStream, Span};
use syn;
use syn::{Ident, Fields, Data};
use syn::spanned::Spanned;
use quote::{quote, quote_spanned};

// use proc_macro::Ident;
// #[macro_export]
// macro_rules! Builder {
//     () => {}
// }

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = input;
    let ast = syn::parse(input).unwrap();
    let expand = impl_builder_macro(&ast);
    proc_macro::TokenStream::from(expand)
}

fn gen_fields(data: &Data) -> TokenStream {
    let fields = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        quote_spanned! {f.span()=> 
                            #name: Option<#ty> 
                        }
                    });
                    quote! {
                        #(#recurse,)*
                    }
                },
                Fields::Unit | Fields::Unnamed(_) => unimplemented!()           
             }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };
    fields
}

fn gen_fields_init(data: &Data) -> TokenStream {
    let fields = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=> 
                            #name: None 
                        }
                    });
                    quote! {
                        #(#recurse,)*
                    }
                },
                Fields::Unit | Fields::Unnamed(_) => unimplemented!()           
             }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };
    fields
}
fn gen_fields_setters(data: &Data) -> TokenStream {
    let fields = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        quote_spanned! {f.span()=> 
                            fn #name (&mut self, #name : #ty) -> &mut Self {
                                self.#name = Some(#name);
                                self
                            } 
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                },
                Fields::Unit | Fields::Unnamed(_) => unimplemented!()           
             }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };
    fields
}


fn gen_builder_fields_check(data: &Data) -> TokenStream {
    let fields = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=> 
                            if self.#name.is_none()  {
                                return Err(format!("{} is not set", stringify!(#name)).to_string().into());
                            }
                        }
                    });
                    quote! {
                        #(#recurse )*
                    }
                },
                Fields::Unit | Fields::Unnamed(_) => unimplemented!()           
             }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };
    fields
}

fn gen_builder_fields_to_field(data: &Data) -> TokenStream {
    let fields = match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=> 
                            #name: self.#name.clone().unwrap()
                        }
                    });
                    quote! {
                        #(#recurse ,)*
                    }
                },
                Fields::Unit | Fields::Unnamed(_) => unimplemented!()           
             }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };
    fields
}

fn impl_builder_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let namebuilder =  &Ident::new(&format!("{}Builder", name), Span::call_site());

    let fields = gen_fields(&ast.data);
    let fields_init = gen_fields_init(&ast.data);
    let field_setters = gen_fields_setters(&ast.data);
    let builder_fields_check = gen_builder_fields_check(&ast.data);
    let gen_builder_fields_to_fields = gen_builder_fields_to_field(&ast.data);
    let gen = quote! {
         impl #name  {
            fn builder() -> #namebuilder {
                #namebuilder {
                    #fields_init
                }
            }
         }

        pub struct #namebuilder {
            #fields
        }

        impl #namebuilder {
           #field_setters 

           pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
               #builder_fields_check
               Ok(#name {
                    #gen_builder_fields_to_fields
               })
          }
        }


    };
    gen.into()
}