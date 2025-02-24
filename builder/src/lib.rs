use proc_macro::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;

#[proc_macro_derive(Builder, attributes(builder))]
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
                attrs,
                ..
            } = field;
            (ident.unwrap(), ty, attrs)
        });

    let names = fields.clone().map(|(ident, ..)| {
        ident
    });

    let functions = fields.clone().map(|(ident, orig_ty, attrs)| {

        match extract_inner_vec(&orig_ty) {
            Some(single_type) => {
                if !attrs.is_empty() {
                    let functions = attrs.into_iter().filter_map(|attr| { 
                        if !attr.path().is_ident("builder")
                        {
                            return None;
                        }
                        match attr.parse_args::<syn::MetaNameValue>().expect("builder Attribute should have an assignement argument") {
                            syn::MetaNameValue {
                                path,
                                value: syn::Expr::Lit(
                                    syn::ExprLit {
                                        lit: syn::Lit::Str(function_name),
                                        ..
                                    }
                                ),
                                ..
                            } => {
                                let function_name = syn::Ident::new(&function_name.value(), function_name.span());
                                if !path.is_ident("each") {
                                    return Some(syn::Error::new(attr.meta.span(), "expected `builder(each = \"...\")`").into_compile_error());
                                }
                                if ident == function_name {
                                    None
                                } else {
                                    Some(quote::quote! {
                                        fn #function_name(&mut self, value: #single_type) -> &mut Self {
                                            self.#ident.push(value);
                                            self
                                        }
                                    })
                                }
                            },
                                _ => panic!("builder Attribute should have an assignement argument"),
                        }
                    });


                    quote::quote! {
                        fn #ident(&mut self, #ident: #orig_ty) -> &mut Self {
                            self.#ident = #ident;
                            self
                        }
                        #(
                            #functions
                        )*
                    }
                } else {
                    quote::quote! {
                        fn #ident(&mut self, #ident: #orig_ty) -> &mut Self {
                            self.#ident = #ident;
                            self
                        }
                    }
                }
            },
            None => {
                let ty = match extract_inner_option(&orig_ty) {
                    Some(inner_ty) => inner_ty,
                    None => orig_ty,
                };

                quote::quote! {
                    fn #ident(&mut self, #ident: #ty) -> &mut Self {
                        self.#ident = Some(#ident);
                        self
                    }
                }
            },
        }
    });

    let src_construction = fields.clone().map(|(name, ty, _)| {
        if extract_inner_option(&ty).is_some() { 
            quote::quote! {
                #name: self.#name.take()
            }
        } else if extract_inner_vec(&ty).is_some() {
            quote::quote! {
                #name: self.#name.clone()
            }
        } else {
            quote::quote! {
                #name: self.#name.take().ok_or(std::convert::Into::<Box<dyn std::error::Error>>::into(format!("Field #name not set")))?
            } 
        }
    });

    let builder_construction = fields.clone().map(|(name, ty, attrs)| {
        if extract_inner_vec(&ty).is_some() {
            quote::quote! {
                #name: vec![]
            }
        } else {
            quote::quote! {
                #name: None
            }
        }
    });

    let builder_fields = fields.map(|(name, ty, _)| {
        if let Some(inner_ty) = extract_inner_option(&ty) {
            quote::quote! {
                #name: Option<#inner_ty>
            }
        } else if extract_inner_vec(&ty).is_some() {
            quote::quote! {
                #name: #ty
            }
        } else {
            quote::quote! {
                #name: Option<#ty>
            }
        }
    });

    let builder_name = syn::Ident::new(&format!("{name}Builder"), name.span());

    let expanded = quote::quote! {
        impl #name {
            fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_construction),*
                }
            }
        }

        struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            #(#functions)
            *
            pub fn build(&mut self) -> Result<#name, Box<dyn ::std::error::Error>> {
                Ok(#name {
                    #(#src_construction),*
                })
            }
        }
    };

    expanded.into()
}

fn extract_inner_option(ty: &syn::Type) -> Option<syn::Type> {
    match *ty {
        syn::Type::Path(
            syn::TypePath {
                qself: None,
                ref path,
            }
        ) if path.segments[0].ident == "Option" => {
            match &path.segments[0].arguments {
                syn::PathArguments::AngleBracketed(generic) => {
                    match generic.args.first().unwrap() {
                        syn::GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    }
                },
                _ => None
            }
        }
        _ => None,
    }
}

fn extract_inner_vec(ty: &syn::Type) -> Option<syn::Type> {
    match &ty {
        syn::Type::Path(
            syn::TypePath {
                qself:None,
                ref path
            }
        ) if path.segments[0].ident=="Vec" => {
            match &path.segments[0].arguments {
                syn::PathArguments::AngleBracketed(generic) => {
                    match generic.args.first().unwrap() {
                        syn::GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    }
                }
                _ => None,
            }
        },
        _ => None

    }
}
