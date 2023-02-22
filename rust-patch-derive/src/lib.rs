use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span, TokenTree};
use proc_macro_error::proc_macro_error;
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DataStruct, Type, TypePath};
use syn::{DeriveInput, Fields};

#[proc_macro_derive(Patch, attributes(patch))]
#[proc_macro_error]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident;
    let Data::Struct(DataStruct { fields, ..}) = input.data else { abort_call_site!("Patch can only be derived on structs") };
    let fields = match fields {
        Fields::Named(f) => f
            .named
            .into_pairs()
            .map(|p| p.into_value())
            .map(|f| (TokenTree::from(f.ident.unwrap()), f.ty))
            .collect::<Vec<_>>(),
        Fields::Unnamed(f) => f
            .unnamed
            .into_pairs()
            .map(|p| p.into_value())
            .enumerate()
            .map(|(i, f)| (TokenTree::from(Literal::u32_unsuffixed(i as u32)), f.ty))
            .collect::<Vec<_>>(),
        Fields::Unit => Vec::new(),
    };

    let mut targets = Vec::new();
    for Attribute { path, tokens, .. } in input.attrs {
        if path
            .segments
            .first()
            .map(|e| e.ident.to_string())
            .as_deref()
            == Some("patch")
        {
            let Some(TokenTree::Literal(l)) = tokens.into_iter().nth(1) else { abort_call_site!("Failed parsing patch target") };
            let target = Ident::new(l.to_string().trim_matches('"'), Span::call_site());
            targets.push(target);
        }
    }

    let mut apply_sets = Vec::new();
    for (name, ty) in fields {
        if let Type::Path(TypePath { path, .. }) = &ty {
            let Some(ident) = path.segments.first().map(|e| &e.ident) else { abort!(&ty, "Failed parsing field") };
            if &ident.to_string() == "Option" {
                apply_sets.push(quote! {
                    if let Some(val) = self.#name {
                        target.#name = val;
                    }
                });
            } else {
                apply_sets.push(quote! {
                    target.#name = self.#name;
                });
            }
        }
    }

    let apply_content = quote! {
        #(
            #apply_sets
        )*
    };

    let output = quote! {
        #(
            impl ::rust_patch::Patch<#targets> for #ident {
                fn apply(self, mut target: #targets) -> #targets {
                    { #apply_content }
                    target
                }
            }
        )*
    };

    TokenStream::from(output)
}
