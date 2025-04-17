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
/// Represents struct attribute information
pub(crate) struct Container {
    pub armtype: Option<syn::Type>,
}
/// [`Container`] implementation
impl Container {
    /// Extract out the `#[armtype(...)]` attributes from a container.
    /// 
    /// Only container implemented is struct.
    pub fn from_ast(cx: &Ctxt, item: &syn::DeriveInput) -> Self {
        // --------------------------------------------------
        // init
        // --------------------------------------------------
        let mut armtype = None;

        // --------------------------------------------------
        // loop through attrs
        // --------------------------------------------------
        for attr in &item.attrs {
            // --------------------------------------------------
            // only look for `armtype`
            // --------------------------------------------------
            if attr.path() != symbol::ARMTYPE {
                continue;
            }
            // --------------------------------------------------
            // parse out `armtype` attributes
            // --------------------------------------------------
            match &attr.meta {
                // --------------------------------------------------
                // handle all: `syn::MetaList`
                // e.g. `armtype(<..>,),`
                // --------------------------------------------------
                syn::Meta::List(list) => match symbol::Symbol::from(&list.path) {
                    
                    symbol::ARMTYPE => match armtype {
                        Some(_) => cx.error_spanned_by(&list, err!(DuplicateArmtypeEnumAttribute)),
                        None => armtype = syn::parse2(list.tokens.clone())
                            .map_err(|err| cx.syn_error(err))
                            .ok(),
                    },

                    _ => cx.error_spanned_by(&list.path, err!(UnknownContainerAttribute(list.path))),
                },

                _ => cx.error_spanned_by(&attr.meta.path(), err!(MalformedContainerAttribute)),
            }
        }

        // --------------------------------------------------
        // return
        // --------------------------------------------------
        Container {
            armtype: armtype.or_else(|| {
                cx.error_spanned_by(&item.ident, err!(MissingArmtypeEnumAttribute));
                None
            })
        }
    }
}