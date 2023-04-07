use darling::{ast::Data, FromDeriveInput, FromField, FromMeta, FromVariant};
use syn::{punctuated::Punctuated, ExprPath, Token, TypePath};

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(patch))]
pub struct PatchStruct {
    pub ident: syn::Ident,
    pub data: Data<(), PatchField>,
    pub targets: Punctuated<TypePath, Token![,]>,
    pub apply: Option<ExprPath>,
}

#[derive(FromField, Debug)]
#[darling(attributes(patch))]
pub struct PatchField {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,
    pub skip: Option<bool>,
    pub skip_if: Option<ExprPath>,
    pub apply: Option<ExprPath>,
}
