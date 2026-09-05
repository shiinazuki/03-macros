//! 练手用的过程宏

use proc_macro::TokenStream;

#[proc_macro_derive(EnumFrom)]
pub fn enum_from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    eprintln!("{input:#?}");
    TokenStream::new()
}

#[proc_macro_derive(TypeName)]
pub fn type_name_macros(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
    let name_str = name.to_string();
    quote::quote! {
        impl #name {
            fn type_name() -> &'static str {
                #name_str
            }
        }
    }
    .into()
}
