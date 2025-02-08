use proc_macro::TokenStream;
use syn::FieldMutability;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    use syn::Data::Struct;
    use syn::DataStruct;
    use syn::Fields;
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;
    let fields = if let Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = input.data
    {
        fields.named
    } else {
        panic!("We don't support anything but structs")
    };

    /* fields.iter_mut().map(|field| {
        let ty = match field.ty.clone() {
            syn::Type::Path(type_path) => type_path,
            other => unimplemented!("not implemented")
        };

        field.ty = quote::quote! { Option<ty> }.into().parse();
    }); */

    let fields = fields
        .into_iter()
        .map(|field| {
            let syn::Field {
                ident,
                ty,
                ..
            } = field;
            (ident.unwrap(), ty)
        });

    let names = fields.clone().map(|(ident, ..)| {
        ident
    });

    let fields = fields.map(|(name, ty)| {
        quote::quote! {
            #name: Option<#ty>
        }
    });

    let builder_name = syn::Ident::new(&format!("{name}Builder"), name.span());

    let expanded = quote::quote! {
        impl #name {
            fn builder() -> #builder_name {
                #builder_name {
                    #(#names: None),*
                }
            }
        }

        struct #builder_name {
            #(#fields),*
        }
    };


    expanded.into()
}
