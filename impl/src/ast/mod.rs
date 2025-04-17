// --------------------------------------------------
// mods
// --------------------------------------------------
mod variant;
mod container;

// --------------------------------------------------
// re-exports
// --------------------------------------------------
use container::*;

// --------------------------------------------------
// external
// --------------------------------------------------
use syn::{punctuated::Punctuated, Token};

// --------------------------------------------------
// local
// --------------------------------------------------
use crate::Ctxt;

/// A source data structure annotated with `#[derive(Const)]` parsed into an internal representation.
pub(crate) struct MainContainer<'a> {
    /// The struct or enum name (without generics).
    pub ident: syn::Ident,
    /// Attributes on the structure, parsed for `thisenum`.
    pub attrs: container::Container,
    /// The contents of the enum
    pub data: Vec<MainVariant<'a>>,
    /// Any generics on the enum
    pub generics: &'a syn::Generics,
    /// Original input.
    pub _original: &'a syn::DeriveInput,
}

/// A variant of an enum.
pub(crate) struct MainVariant<'a> {
    pub ident: syn::Ident,
    pub attrs: variant::Variant,
    pub original: &'a syn::Variant,
}

/// [`MainContainer`] implementation
impl<'a> MainContainer<'a> {
    /// Convert the raw [`syn`] ast into a parsed container object, collecting errors in `cx`.
    pub fn from_ast(
        cx: &Ctxt,
        item: &'a syn::DeriveInput,
    ) -> Option<MainContainer<'a>> {

        let attrs = Container::from_ast(cx, item);

        let data = match &item.data {
            syn::Data::Enum(data) => Some(enum_from_ast(cx, &data.variants)),
            
            syn::Data::Struct(_) => {
                cx.error_spanned_by(item, err!(UnsupportedContainer("struct")));
                return None;
            }

            syn::Data::Union(_) => {
                cx.error_spanned_by(item, err!(UnsupportedContainer("union")));
                return None;
            }
        }?;

        Some(MainContainer {
            ident: item.ident.clone(),
            attrs,
            data,
            generics: &item.generics,
            _original: item,
        })
    }
}

#[inline(always)]
fn enum_from_ast<'a>(
    cx: &Ctxt,
    variants: &'a Punctuated<syn::Variant, Token![,]>,
) -> Vec<MainVariant<'a>> {
    variants
        .iter()
        .map(|variant| {
            MainVariant {
                ident: variant.ident.clone(),
                attrs: variant::Variant::from_ast(cx, variant),
                original: variant,
            }
        })
        .collect()
}