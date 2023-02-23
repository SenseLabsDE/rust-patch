use proc_macro2::{Group, Literal, Span, TokenStream, TokenTree};
use proc_macro_error::proc_macro_error;
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Data, DataStruct, LitStr, Token, Type, TypePath};
use syn::{DeriveInput, Fields};

struct PatchAttr {
    _eq_token: Token![=],
    path: LitStr,
}

impl Parse for PatchAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _eq_token: input.parse()?,
            path: input.parse()?,
        })
    }
}

#[proc_macro_derive(Patch, attributes(patch))]
#[proc_macro_error]
pub fn derive_patch(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
            let Ok(PatchAttr { path, ..}) = syn::parse2(tokens) else { abort!(&path, r#"Patch target must be specified in the form `#[patch = "path::to::Type"]`"#) };
            let Ok(path) = parse_lit_str::<TypePath>(&path) else { abort!(&path, "`{}` is not a valid path", path.value())};
            targets.push(path);
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

    proc_macro::TokenStream::from(output)
}

// Taken from https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals
fn parse_lit_str<T>(s: &syn::LitStr) -> syn::parse::Result<T>
where
    T: Parse,
{
    let tokens = spanned_tokens(s)?;
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> syn::parse::Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan(stream, s.span()))
}

fn respan(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| respan_token(token, span))
        .collect()
}

fn respan_token(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan(g.stream(), span));
    }
    token.set_span(span);
    token
}
