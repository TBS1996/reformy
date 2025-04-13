use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(FormRenderable)]
pub fn derive_form_renderable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => fields.named,
            _ => {
                return syn::Error::new_spanned(name, "Only named fields supported")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(name, "Only structs supported")
                .to_compile_error()
                .into();
        }
    };

    let get_matches = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let key = ident.to_string();
        quote! {
            #key => &self.#ident,
        }
    });

    let set_matches = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let key = ident.to_string();
        quote! {
            #key => self.#ident = value,
        }
    });

    let field_names = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap().to_string();
        quote! { #ident }
    });

    let expanded = quote! {
        impl ::reformy_core::FormRenderable for #name {
            fn field_names(&self) -> Vec<&'static str> {
                vec![#(#field_names),*]
            }

            fn get_field(&self, field: &str) -> &str {
                match field {
                    #(#get_matches)*
                    _ => "",
                }
            }

            fn set_field(&mut self, field: &str, value: String) {
                match field {
                    #(#set_matches)*
                    _ => {},
                }
            }
        }

        impl #name {
            pub fn form<'a>() -> ::reformy_core::Former<'a, Self> {
                ::reformy_core::Former::new(Self::default())
            }
        }
    };

    TokenStream::from(expanded)
}
