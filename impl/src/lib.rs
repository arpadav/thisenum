#![doc = include_str!("../README.md")]
// --------------------------------------------------
// mods
// --------------------------------------------------
#[macro_use]
mod err;
mod ast;
mod ctxt;
mod expand;
mod symbol;
mod prelude;
// TODO: remove this once done
// mod lib_old;

// --------------------------------------------------
// external
// --------------------------------------------------
use syn::{
    DeriveInput,
    parse_macro_input,
};
use thiserror::Error;

// --------------------------------------------------
// re-exports
// --------------------------------------------------
use crate::ctxt::Ctxt;

// --------------------------------------------------
// constants
// --------------------------------------------------
const CRATE_NAME: &str = "thisenum";
const DERIVE_NAME: &str = "Const";

#[proc_macro_derive(Const, attributes(value, armtype))]
/// [`thisenum`](crate) proc-macro to implement `Const`
pub fn const_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[derive(Debug, Error)]
/// [`tinyklv`](crate) proc-macro errors
enum Error {
    
    // --------------------------------------------------
    // container parsing
    // --------------------------------------------------

    #[error("\
        {c} does not support #[derive({d})] for {0}s.",
        c = CRATE_NAME,
        d = DERIVE_NAME,
    )]
    UnsupportedContainer(String),

    // --------------------------------------------------
    // container (only enum) attributes
    // --------------------------------------------------

    #[error("\
        Unknown enum attribute: `{0}`.
Expected {s}.",
        s = symbol::CONT_SYMBOLS,
    )]
    UnknownContainerAttribute(String),

    #[error("\
        Malformed container attribute, expected: `#[{s}(<type>)]`.",
        s = symbol::ARMTYPE,
    )]
    MalformedContainerAttribute,

    #[error("\
        Missing required `{s}` enum attribute to describe return type of the value's.",
        s = symbol::ARMTYPE,
    )]
    MissingArmtypeEnumAttribute,

    #[error("\
        Duplicate `{s}` enum attribute.",
        s = symbol::ARMTYPE,
    )]
    DuplicateArmtypeEnumAttribute,

    // --------------------------------------------------
    // variant attribute (only value is used)
    // --------------------------------------------------

    #[error("\
        Unknown variant attribute: `{0}`.
Expected {s}.",
        s = symbol::VARIANT_SYMBOLS
    )]
    UnknownVariantAttribute(String),

    #[error("\
        Malformed variant attribute, expected name-value attribute: `#[{s} = <value>]`.",
        s = symbol::VALUE,
    )]
    MalformedValueAttribute,

    #[error("\
        Missing required `{s}` variant attribute.",
        s = symbol::VALUE,
    )]
    MissingValueVariantAttribute,

    #[error("\
        Duplicate `{s}` variant attribute.",
        s = symbol::VALUE,
    )]
    DuplicateValueVariantAttribute,
}
/// [`Error`] implementation
impl Error {
    fn as_str(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Owned(self.to_string())
    }
}