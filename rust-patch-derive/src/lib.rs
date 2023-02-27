use proc_macro2::{Group, Ident, Literal, Span, TokenStream, TokenTree};
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    token, Attribute, Data, DataStruct, DeriveInput, Fields, Token, Type, TypePath,
};

struct PatchEqAttr {
    _eq_token: Token![=],
    path: TypePath,
}

impl Parse for PatchEqAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _eq_token: input.parse()?,
            path: parse_lit_str(&input.parse()?)?,
        })
    }
}

struct PatchParenAttr {
    _paren_token: token::Paren,
    content: Ident,
}

impl Parse for PatchParenAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _paren_token: parenthesized!(content in input),
            content: content.parse()?,
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
            .map(|f| (TokenTree::from(f.ident.unwrap()), f.ty, f.attrs))
            .collect::<Vec<_>>(),
        Fields::Unnamed(f) => f
            .unnamed
            .into_pairs()
            .map(|p| p.into_value())
            .enumerate()
            .map(|(i, f)| {
                (
                    TokenTree::from(Literal::u32_unsuffixed(i as u32)),
                    f.ty,
                    f.attrs,
                )
            })
            .collect::<Vec<_>>(),
        Fields::Unit => Vec::new(),
    };

    let mut targets = Vec::new();
    for patch_target in get_patch_attrs(input.attrs) {
        let span = patch_target.span();
        let Ok(PatchEqAttr { path, ..}) = syn::parse2(patch_target) else { abort!(span, r#"Patch target must be specified in the form `#[patch = "path::to::Type"]`"#) };
        targets.push(path);
    }

    let mut apply_sets = Vec::new();
    for (name, ty, attrs) in fields {
        let Type::Path(TypePath { path, .. }) = &ty else { abort!(&ty, "Failed parsing field type as type path") };
        let Some(ident) = path.segments.first().map(|e| &e.ident) else { abort!(&ty, "Field does not contain a valid ident") };
        let mut direct = false;
        let mut as_option = false;
        for attr in get_patch_attrs(attrs) {
            let span = attr.span();
            let content = match syn::parse2(attr) {
                Ok(PatchParenAttr { content, .. }) => content,
                Err(e) => abort!(span, "Failed parsing attribute: {}", e),
            };
            match content.to_string().as_str() {
                "direct" => direct = true,
                "as_option" => as_option = true,
                a => {
                    abort!(span, "Unknown attribute `{}`", a)
                }
            }
        }
        if direct && as_option {
            abort!(&ty, "Only one of `#[patch(direct)]` or `#[patch(as_option)]` may be specified for given field");
        }
        if as_option {
            apply_sets.push(quote! {
                if self.#name.is_some() {
                    target.#name = self.#name;
                }
            })
        } else if &ident.to_string() == "Option" && !direct {
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

fn get_patch_attrs(attrs: Vec<Attribute>) -> Vec<TokenStream> {
    let mut result = Vec::new();
    for Attribute { path, tokens, .. } in attrs {
        if path
            .segments
            .first()
            .map(|e| e.ident.to_string())
            .as_deref()
            == Some("patch")
        {
            result.push(tokens);
        }
    }
    result
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
