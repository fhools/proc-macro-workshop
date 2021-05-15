use proc_macro::TokenStream;
// #[macro_export]
// macro_rules! Builder {
//     () => {}
// }

use syn;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;
    let ast = syn::parse(input).unwrap();
    impl_builder_macro(&ast)
}


fn impl_builder_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        trait BuilderMacro {
            fn builder_macro();
        }

        impl BuilderMacro for #name {
            fn builder_macro() {

            }
        }
    };
    gen.into()
}