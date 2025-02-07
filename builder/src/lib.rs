use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;
    let builder_name = syn::Ident::new(&format!("{name}Builder"), name.span());

    quote::quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    
                }
            }
        }

        struct #builder_name {

        }
    }.into()
}
