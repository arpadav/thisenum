// --------------------------------------------------
// external
// --------------------------------------------------
use quote::ToTokens;

// --------------------------------------------------
// local
// --------------------------------------------------
use crate::Ctxt;
use crate::symbol;

#[derive(Debug)]
/// Represents variant attribute information
pub(crate) struct Variant {
    pub value: Option<syn::Expr>,
}
/// [`Variant`] implementation
impl Variant {
    /// Extract out the `#[value(...)]` attributes from an enum variant.
    pub fn from_ast(
        cx: &Ctxt,
        variant: &syn::Variant,
    ) -> Self {
        let mut result = Self { value: None };
        // --------------------------------------------------
        // return if no attrs on variant
        // --------------------------------------------------
        if variant.attrs.is_empty() {
            return result;
        }

        // --------------------------------------------------
        // loop through attrs
        // --------------------------------------------------
        for attr in &variant.attrs {
            // --------------------------------------------------
            // only look for `value`
            // --------------------------------------------------
            if attr.path() != symbol::VALUE {
                continue;
            }
            // --------------------------------------------------
            // parse out `value` attributes
            // --------------------------------------------------
            match attr.meta {
                // --------------------------------------------------
                // handle all: `syn::NameValue`
                // e.g. `value = <..>,`
                // --------------------------------------------------
                syn::Meta::NameValue(ref value) => match symbol::Symbol::from(&value.path) {
                    symbol::VALUE => match result.value {
                        Some(_) => cx.error_spanned_by(attr, err!(DuplicateValueVariantAttribute)),
                        None => result.value = Some(value.value.clone()),
                    },

                    _ => cx.error_spanned_by(attr, err!(UnknownVariantAttribute(value.path))),
                }

                _ => cx.error_spanned_by(attr, err!(MalformedValueAttribute)),
            }
        }

        if result.value.is_none() {
            cx.error_spanned_by(variant, err!(MissingValueVariantAttribute));
        }

        // --------------------------------------------------
        // return
        // --------------------------------------------------
        result
    }
}